use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use clap::{Parser, Subcommand};
use console::{style, Emoji};
use ed25519_dalek::{Signer, SigningKey};
use indicatif::{ProgressBar, ProgressStyle};
use pbkdf2::pbkdf2_hmac_array;
use rand::Rng;
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "");

#[derive(Parser)]
#[command(name = "ezer")]
#[command(about = "Ezerdesk Plugin CLI - Herramienta para automatizar la creación de plugins", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa un nuevo plugin de Ezerdesk
    Init {
        /// Nombre del plugin
        name: String,
    },
    /// Compila y firma el plugin a WebAssembly (wasm32-unknown-unknown)
    Build,
    /// Inicia un servidor de desarrollo local para probar el plugin en vivo
    Dev {
        /// Puerto del servidor (default: 3030)
        #[arg(short, long, default_value = "3030")]
        port: u16,
    },
    /// Publica el plugin en un servidor EzerDesk
    Publish {
        /// URL del servidor (ej: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,

        /// Token CSRF (opcional, se puede leer de EZER_TOKEN)
        #[arg(short = 't', long)]
        token: Option<String>,

        /// Precio del plugin (default: 0.0)
        #[arg(short, long, default_value = "0.0")]
        precio: f64,

        /// Si es de pago (default: false)
        #[arg(short = 'p', long)]
        es_pago: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => init_plugin(&name),
        Commands::Build => build_plugin(),
        Commands::Dev { port } => dev_server(port),
        Commands::Publish { server, token, precio, es_pago } => publish_plugin(server, token, precio, es_pago),
    };

    if let Err(msg) = result {
        eprintln!("{} {}", CROSS_MARK, style(msg).red());
        std::process::exit(1);
    }
}

fn encrypt_key(key_bytes: &[u8; 32], password: &str) -> Result<String, String> {
    let mut rng = rand::thread_rng();

    // Salt para PBKDF2
    let mut salt = [0u8; 16];
    rng.fill(&mut salt);

    // Derivar clave de 32 bytes desde la contraseña
    let derived: [u8; 32] = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), &salt, 100_000);

    // Nonce para AES-GCM (12 bytes)
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&derived);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, key_bytes.as_ref())
        .map_err(|_| "Error cifrando la clave.".to_string())?;

    // Formato: salt[16] + nonce[12] + ciphertext
    let mut output = Vec::with_capacity(16 + 12 + ciphertext.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(hex::encode(output))
}

fn decrypt_key(encrypted_hex: &str, password: &str) -> Result<[u8; 32], String> {
    let data = hex::decode(encrypted_hex)
        .map_err(|e| format!("Formato de clave cifrada inválido: {}", e))?;

    if data.len() < 28 {
        return Err("Archivo de clave demasiado corto.".to_string());
    }

    let salt = &data[0..16];
    let nonce_bytes = &data[16..28];
    let ciphertext = &data[28..];

    let derived: [u8; 32] = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, 100_000);

    let key = Key::<Aes256Gcm>::from_slice(&derived);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Contraseña incorrecta.".to_string())?;

    let mut result = [0u8; 32];
    result.copy_from_slice(&plaintext);
    Ok(result)
}

fn prompt_password(prompt: &str) -> Result<String, String> {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let password = rpassword::read_password()
        .map_err(|e| format!("Error leyendo contraseña: {}", e))?;
    Ok(password)
}

fn generate_keypair() -> Result<(String, String), String> {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);

    let signing_key = SigningKey::from_bytes(&bytes);
    let verifying_key = signing_key.verifying_key();

    let private_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());

    Ok((private_hex, public_hex))
}

fn init_plugin(name: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;
    init_plugin_at(&cwd, name)
}

fn init_plugin_at(base: &Path, name: &str) -> Result<(), String> {
    println!(
        "{} {}Creando nuevo plugin: {}...",
        SPARKLES,
        style("Ezerdesk").bold().cyan(),
        style(name).yellow()
    );

    let path = base.join(name);
    if path.exists() {
        return Err(format!("La carpeta '{}' ya existe.", name));
    }

    let src = path.join("src");
    fs::create_dir_all(&src).map_err(|e| format!("No se pudo crear 'src': {}", e))?;

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
ezerdesk-sdk = "0.1.3"
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("No se pudo escribir Cargo.toml: {}", e))?;

    let lib_rs = r#"use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        // ════════════════════════════════════════════════════════════════
        //  1. SIDEBAR — Menú de navegación lateral
        //  ════════════════════════════════════════════════════════════════
        //  Define cómo aparece tu plugin en el menú lateral.
        //  📍 category: "operaciones" | "administracion" | "sistema"
        //  ════════════════════════════════════════════════════════════════
        PluginEvent::GetMetadata => {
            let meta = sdk::metadata()
                .nav_item(NavItem::new("dashboard", "Mi Plugin", "rocket-line")
                    .category("operaciones")
                    .priority(10))
                .name("Mi Plugin")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        // ════════════════════════════════════════════════════════════════
        //  2. PÁGINA DEL MÓDULO — Contenido al hacer clic en el menú
        //  ════════════════════════════════════════════════════════════════
        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "dashboard" => {
                    sdk::respond(sdk::widgets![
                        sdk::card("Panel Principal", [
                            sdk::text("Bienvenido a tu plugin.", "info")
                        ])
                    ]);
                },
                _ => {}
            }
        }

        // ════════════════════════════════════════════════════════════════
        //  3. CONFIGURACIÓN — Gestión → Plugins → ⚙️
        //  ════════════════════════════════════════════════════════════════
        //  📍 "plugin_settings" → modal de ajustes
        //  📍 "dashboard_widget" → Dashboard
        //  📍 "ticket_detail" → vista del ticket
        //  ════════════════════════════════════════════════════════════════
        PluginEvent::GetUiFragments { location } => {
            match location.as_str() {
                "plugin_settings" => {
                    sdk::respond(sdk::widgets![
                        sdk::card("Configuración", [
                            sdk::text("Ajusta los parámetros del plugin.", "muted"),
                            sdk::input("API Key", "api_key", "Ingresa tu llave..."),
                        ])
                    ]);
                },
                _ => {}
            }
        }

        // ════════════════════════════════════════════════════════════════
        //  4. ACCIONES — Botones y formularios
        //  ════════════════════════════════════════════════════════════════
        PluginEvent::PluginAction { action, data } => {
            sdk::log(&format!("Acción '{}' con datos: {:?}", action, data));
            sdk::respond_ok("Procesado");
        }

        // ════════════════════════════════════════════════════════════════
        //  5. EVENTOS DEL SISTEMA
        //  ════════════════════════════════════════════════════════════════
        //  TicketCreated, TicketStatusChanged, CommentAdded, etc.
        //  Usa sdk::kv_set_val() / sdk::kv_get_val() para persistir.
        //  ════════════════════════════════════════════════════════════════
        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Ticket creado: {}", ticket.asunto));
        }

        // ════════════════════════════════════════════════════════════════
        //  6. CONSULTAS AL SISTEMA — Obtener datos del host
        //  ════════════════════════════════════════════════════════════════
        //  Tipos: query::tickets(), query::agents(), query::departments()
        //  Uso:
        //    let tickets = sdk::query::tickets().limit(10).all();
        //    match tickets {
        //        Ok(list) => sdk::log(&format!("{} tickets", list.len())),
        //        Err(e) => sdk::log(&format!("Error: {:?}", e)),
        //    }
        //  ════════════════════════════════════════════════════════════════
        _ => {}
    }

    0
}
"#;
    fs::write(src.join("lib.rs"), lib_rs)
        .map_err(|e| format!("No se pudo escribir src/lib.rs: {}", e))?;

    // Generar par de claves Ed25519 para firma automática
    let (priv_hex, pub_hex) = generate_keypair()?;

    // Solicitar contraseña para cifrar la clave privada
    let password = prompt_password("🔐 Contraseña para proteger la clave de firma: ")?;
    let confirm = prompt_password("🔐 Confirma la contraseña: ")?;

    if password != confirm {
        return Err("Las contraseñas no coinciden.".to_string());
    }

    let private_key_bytes = hex::decode(&priv_hex)
        .map_err(|_| "Error decodificando clave privada.".to_string())?;
    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&private_key_bytes);

    let encrypted = encrypt_key(&key_arr, &password)?;

    fs::write(path.join(".ezer-key"), &encrypted)
        .map_err(|e| format!("No se pudo guardar la clave cifrada: {}", e))?;

    // Guardar clave pública como referencia
    fs::write(path.join(".ezer-key.pub"), &pub_hex)
        .map_err(|e| format!("No se pudo guardar la clave pública: {}", e))?;

    println!(
        "{} {}Plugin {} listo para desarrollar!",
        ROCKET,
        style("Éxito:").green(),
        style(name).yellow()
    );
    println!("Prueba ejecutando: {} {}", style("cd").cyan(), name);
    println!("Luego: {} {}", style("ezer").cyan(), "build");
    println!(
        "{} Clave pública: {}",
        style("🔑").bold(),
        style(&pub_hex[..20]).dim()
    );
    println!(
        "{} La clave privada está cifrada con contraseña.",
        style("🔐").bold()
    );

    Ok(())
}

fn build_plugin() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;
    build_plugin_at(&cwd)
}

fn read_signing_key(cwd: &Path) -> Result<String, String> {
    let key_path = cwd.join(".ezer-key");
    let pub_key_path = cwd.join(".ezer-key.pub");

    if !key_path.exists() {
        return match std::env::var("PLUGIN_SIGN_KEY") {
            Ok(key) => {
                // Si la variable de entorno tiene la clave, podría estar en hex directo
                // (para CI/CD sin interacción)
                Ok(key.trim().to_string())
            }
            Err(_) => Err(format!(
                "No se encontró {}.\n\
                 También puedes usar la variable PLUGIN_SIGN_KEY.",
                key_path.display()
            )),
        };
    }

    // Leer clave cifrada
    let encrypted = fs::read_to_string(&key_path)
        .map_err(|e| format!("No se pudo leer {}: {}", key_path.display(), e))?;
    let encrypted = encrypted.trim().to_string();

    // Si hay PLUGIN_SIGN_PASS, usarla sin preguntar (útil en CI/CD)
    // El .ezer-key.pub es solo referencia, podemos leerlo para mostrar info
    let _pub_info = if pub_key_path.exists() {
        fs::read_to_string(&pub_key_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Pedir contraseña
    let password = prompt_password("🔐 Contraseña de la clave de firma: ")?;
    let key_bytes = decrypt_key(&encrypted, &password)?;
    Ok(hex::encode(key_bytes))
}

fn dev_server(port: u16) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;

    let wasm_path = get_wasm_path(&cwd)?;
    let wasm_bytes = fs::read(&wasm_path)
        .map_err(|e| format!("No se pudo leer el .wasm: {}", e))?;

    println!(
        "{} {}Servidor de desarrollo",
        style("🌐").bold(),
        style("Ezerdesk Dev Server").cyan()
    );
    println!("  Plugin:  {}", wasm_path.display());
    println!("  Puerto:  http://localhost:{}", port);
    println!();
    println!("  {} Abriendo navegador...", style("→").bold());

    // Abrir navegador
    let url = format!("http://localhost:{}", port);
    let _ = webbrowser::open(&url);

    // Iniciar servidor HTTP
    let address = format!("127.0.0.1:{}", port);
    let listener = std::net::TcpListener::bind(&address)
        .map_err(|e| format!("No se pudo iniciar el servidor en {}: {}", address, e))?;

    println!("  {} Servidor corriendo. Presiona Ctrl+C para detener.", style("✓").green());

    use base64::Engine;
    let wasm_b64 = base64::engine::general_purpose::STANDARD.encode(&wasm_bytes);
    let html = build_dev_html(&wasm_b64);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    html.len(),
                    html
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
            Err(e) => {
                eprintln!("Error en conexión: {}", e);
            }
        }
    }

    Ok(())
}

fn build_dev_html(wasm_b64: &str) -> String {
    let html = include_str!("dev.html");
    html.replace("__WASM_BASE64__", wasm_b64)
}

fn curl_json(url: &str, method: &str, body: &str, jar: &Path) -> Result<String, String> {
    let output = Command::new("curl")
        .arg("-s")
        .arg("-X").arg(method)
        .arg("-H").arg("Content-Type: application/json")
        .arg("--cookie-jar").arg(jar)
        .arg("--cookie").arg(jar.to_str().unwrap_or(""))
        .arg("-d").arg(body)
        .arg(url)
        .output()
        .map_err(|e| format!("Error ejecutando curl: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let msg = if !stdout.is_empty() { stdout } else { stderr };
        Err(msg)
    }
}

fn read_plugin_manifest(cwd: &Path) -> Result<(String, String), String> {
    let cargo = fs::read_to_string(cwd.join("Cargo.toml"))
        .map_err(|e| format!("No se pudo leer Cargo.toml: {}", e))?;

    let parsed: toml::Value = toml::from_str(&cargo)
        .map_err(|e| format!("Error parseando Cargo.toml: {}", e))?;

    let name = parsed["package"]["name"]
        .as_str()
        .ok_or_else(|| "No se encontró [package].name en Cargo.toml".to_string())?
        .to_string();

    let version = parsed["package"]["version"]
        .as_str()
        .unwrap_or("0.0.0")
        .to_string();

    Ok((name, version))
}

fn publish_plugin(
    server: Option<String>,
    _token: Option<String>,
    precio: f64,
    es_pago: bool,
) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;

    // Leer nombre y versión desde Cargo.toml
    let (plugin_name, plugin_version) = read_plugin_manifest(&cwd)?;

    // Leer .wasm
    let wasm_path = get_wasm_path(&cwd)?;
    let wasm_bytes = fs::read(&wasm_path)
        .map_err(|e| format!("No se pudo leer .wasm: {}", e))?;

    // Leer .sig (firma)
    let sig_path = wasm_path.with_extension("sig");
    let firma = if sig_path.exists() {
        Some(
            fs::read_to_string(&sig_path)
                .map_err(|e| format!("No se pudo leer .sig: {}", e))?
                .trim()
                .to_string(),
        )
    } else {
        None
    };

    // Codificar WASM a base64
    use base64::Engine;
    let codigo_b64 = base64::engine::general_purpose::STANDARD.encode(&wasm_bytes);

    // Construir JSON del upload
    let mut upload_body = serde_json::json!({
        "codigo_wasm_base64": codigo_b64,
        "precio": precio,
        "es_pago": es_pago,
        "nombre": plugin_name,
        "version": plugin_version,
    });
    if let Some(ref sig) = firma {
        upload_body["firma"] = serde_json::json!(sig);
    }
    let upload_json = serde_json::to_string(&upload_body)
        .map_err(|e| format!("Error serializando JSON: {}", e))?;

    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');

    println!();
    println!(
        "{} {}Publicando plugin v{}",
        PACKAGE,
        style(&plugin_name).bold().cyan(),
        style(&plugin_version).yellow()
    );
    println!("  Servidor: {}", base);
    println!("  Archivo:  {}", wasm_path.display());
    if let Some(ref s) = firma {
        println!("  Firma:    {}...", &s[..16]);
    }
    println!();

    // ── Autenticación ──────────────────────────────────────────────────────
    let jar = cwd.join(".ezer-cookie-jar");

    let email = prompt("📧 Correo electrónico: ")?;
    let password = prompt_password("🔑 Contraseña: ")?;

    let login_body = serde_json::json!({"correo": email, "clave": password});
    let login_json = serde_json::to_string(&login_body)
        .map_err(|_| "Error serializando login.".to_string())?;

    let login_url = format!("{}/api/v1/auth/login", base);
    println!("  → Iniciando sesión...");
    let login_resp = curl_json(&login_url, "POST", &login_json, &jar)?;

    // Verificar si requiere MFA
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&login_resp) {
        if json.get("status").and_then(|s| s.as_str()) == Some("mfa_required") {
            let mfa_token = json["mfa_token"].as_str().unwrap_or("");
            let code = prompt("🔐 Código OTP (6 dígitos): ")?;

            let challenge_body = serde_json::json!({"code": code, "mfa_token": mfa_token});
            let challenge_json = serde_json::to_string(&challenge_body)
                .map_err(|_| "Error serializando MFA.".to_string())?;

            let challenge_url = format!("{}/api/v1/auth/otp/challenge", base);
            println!("  → Verificando OTP...");
            curl_json(&challenge_url, "POST", &challenge_json, &jar)?;
        }
    }

    // ── Subir plugin ───────────────────────────────────────────────────────
    let upload_url = format!("{}/api/v1/plugins/upload", base);
    println!("  → Subiendo plugin...");

    let resp = curl_json(&upload_url, "POST", &upload_json, &jar)?;

    // Verificar respuesta
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            return Err(format!("El servidor rechazó el plugin: {}", error));
        }
        if let Some(codigo) = json.get("codigo").and_then(|c| c.as_str()) {
            if codigo == "VALIDACION_FALLIDA" {
                if let Some(detalles) = json.get("detalles").and_then(|d| d.as_array()) {
                    let msgs: Vec<String> = detalles.iter()
                        .filter_map(|d| d.as_str().map(|s| s.to_string()))
                        .collect();
                    return Err(format!("Validación fallida:\n  • {}", msgs.join("\n  • ")));
                }
            }
        }
    }

    // Limpiar cookie jar
    let _ = fs::remove_file(&jar);

    println!("  {} Plugin publicado exitosamente!", style("✓").green());
    Ok(())
}

fn prompt(msg: &str) -> Result<String, String> {
    print!("{}", msg);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Error leyendo entrada: {}", e))?;
    Ok(input.trim().to_string())
}

fn get_wasm_path(cwd: &Path) -> Result<std::path::PathBuf, String> {
    let target_dir = cwd.join("target").join("wasm32-unknown-unknown").join("release");
    let entries = fs::read_dir(&target_dir)
        .map_err(|e| format!("No se pudo leer el directorio de salida: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "wasm") {
            return Ok(path);
        }
    }
    Err("No se encontró el archivo .wasm compilado.".to_string())
}

fn sign_wasm_with_key(wasm_path: &Path, key_hex: &str) -> Result<(), String> {

    let key_bytes = hex::decode(key_hex.trim())
        .map_err(|e| format!("La clave privada no es un hex válido: {}", e))?;

    if key_bytes.len() != 32 {
        return Err("La clave privada debe tener 32 bytes (64 caracteres hex).".to_string());
    }

    let key_array: [u8; 32] = key_bytes[..32]
        .try_into()
        .map_err(|_| "Error al convertir la clave.".to_string())?;

    let signing_key = SigningKey::from_bytes(&key_array);
    let verifying_key = signing_key.verifying_key();
    let public_key_hex = hex::encode(verifying_key.to_bytes());

    let wasm_bytes = fs::read(wasm_path)
        .map_err(|e| format!("No se pudo leer el .wasm: {}", e))?;

    let signature = signing_key.sign(&wasm_bytes);
    let signature_hex = hex::encode(signature.to_bytes());

    // Guardar la firma en un archivo junto al .wasm
    let sig_path = wasm_path.with_extension("sig");
    fs::write(&sig_path, &signature_hex)
        .map_err(|e| format!("No se pudo guardar la firma: {}", e))?;

    println!(
        "{} Plugin firmado correctamente.",
        style("🔐").bold()
    );
    println!("  {} Firma:        {}", style("📝").bold(), &signature_hex[..16]);
    println!("  {} Clave pública: {}", style("🔑").bold(), &public_key_hex[..16]);
    println!(
        "  {} Archivo:      {}",
        style("💾").bold(),
        sig_path.display()
    );

    // Mostrar la clave pública para configurar en el servidor
    println!();
    println!(
        "{} {}",
        style("⚠️  IMPORTANTE:").yellow().bold(),
        style("Agrega esta clave pública al .env del servidor:").yellow()
    );
    println!("  PLUGIN_PUBLIC_KEY={}", public_key_hex);

    Ok(())
}

fn build_plugin_at(cwd: &Path) -> Result<(), String> {
    if !cwd.join("Cargo.toml").exists() {
        return Err(
            "No se encontró Cargo.toml en el directorio actual.\n\
             Asegúrate de ejecutar 'ezer build' dentro de la carpeta de un plugin."
                .to_string(),
        );
    }

    if !wasm_target_installed() {
        return Err(
            "El target wasm32-unknown-unknown no está instalado.\n\
             Ejecuta: rustup target add wasm32-unknown-unknown"
                .to_string(),
        );
    }

    println!(
        "{} {}Compilando plugin para WebAssembly...",
        PACKAGE,
        style("Ezerdesk").bold().cyan()
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .expect("Error en template de progreso"),
    );
    pb.set_message("Ejecutando cargo build...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let output = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("No se pudo ejecutar cargo: {}", e))?;

    pb.finish_and_clear();

    if output.status.success() {
        println!(
            "{} {}Plugin compilado con éxito.",
            SPARKLES,
            style("¡Listo!").green()
        );
        let wasm_path = get_wasm_path(cwd)?;

        // Firmar automáticamente
        match read_signing_key(cwd) {
            Ok(key_hex) => {
                sign_wasm_with_key(&wasm_path, &key_hex)?;
                println!(
                    "{} {}Plugin firmado automáticamente.",
                    style("🔐").bold(),
                    style("Firma:").green()
                );
            }
            Err(_) => {
                println!(
                    "{} {}No se encontró clave de firma. El plugin no está firmado.",
                    style("⚠️").bold(),
                    style("Advertencia:").yellow()
                );
                println!(
                    "   Para generar una clave: {}",
                    style("ezer init nuevo-plugin && cp nuevo-plugin/.ezer-key .").dim()
                );
            }
        }

        println!(
            "{} {}Binario: {}",
            LOOKING_GLASS,
            style("Archivo:").blue(),
            style(wasm_path.display()).yellow()
        );

        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Fallo en la compilación:\n{}", stderr))
    }
}

fn wasm_target_installed() -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|line| line.trim() == "wasm32-unknown-unknown")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ezer_cli_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // ── init_plugin_at ────────────────────────────────────────────────────────

    #[test]
    fn test_init_plugin_creates_files() {
        let base = tmp_dir("init_creates_files");
        let result = init_plugin_at(&base, "test-plugin");

        let plugin = base.join("test-plugin");
        assert!(result.is_ok(), "init_plugin failed: {:?}", result.err());
        assert!(plugin.join("Cargo.toml").exists());
        assert!(plugin.join("src/lib.rs").exists());

        let cargo = fs::read_to_string(plugin.join("Cargo.toml")).unwrap();
        assert!(cargo.contains(r#"name = "test-plugin""#));
        assert!(cargo.contains(r#"edition = "2021""#));
        assert!(cargo.contains(r#"ezerdesk-sdk = "0.1.3""#));

        let lib = fs::read_to_string(plugin.join("src/lib.rs")).unwrap();
        assert!(lib.contains("#[sdk::main]"));
        assert!(lib.contains("PluginEvent::GetMetadata"));
        assert!(lib.contains(r#"host: "ezerdesk""#));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_init_plugin_existing_dir() {
        let base = tmp_dir("init_existing_dir");
        let name = "existing-plugin";
        fs::create_dir_all(base.join(name)).unwrap();

        let result = init_plugin_at(&base, name);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ya existe"));

        let _ = fs::remove_dir_all(&base);
    }

    // ── build_plugin_at pre-checks ────────────────────────────────────────────

    #[test]
    fn test_build_plugin_no_cargo_toml() {
        let base = tmp_dir("build_no_cargo_toml");
        let result = build_plugin_at(&base);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cargo.toml"));

        let _ = fs::remove_dir_all(&base);
    }
}
