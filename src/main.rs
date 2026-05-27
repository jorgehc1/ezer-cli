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
use zeroize::Zeroize;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "");

const SDK_VERSION: &str = env!("EZERDESK_SDK_VERSION");
const SDK_SOURCE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../ezerdesk-sdk");

#[derive(Parser)]
#[command(name = "ezer", version = "0.1.3")]
#[command(about = "Ezerdesk Plugin CLI - © 2025 RFJ Software (https://rfjsoftware.com)")]
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

    let mut salt = [0u8; 16];
    rng.fill(&mut salt);

    let mut derived: [u8; 32] = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), &salt, 100_000);

    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&derived);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, key_bytes.as_ref())
        .map_err(|_| "Error cifrando la clave.".to_string())?;

    let mut output = Vec::with_capacity(16 + 12 + ciphertext.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    derived.zeroize();

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

    let mut derived: [u8; 32] = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, 100_000);

    let key = Key::<Aes256Gcm>::from_slice(&derived);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Contraseña incorrecta.".to_string())?;

    derived.zeroize();

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
        "{} {} Creando nuevo plugin: {}...",
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

    let sdk_dep = detect_sdk_dependency(base);

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[workspace]

[profile.release]
opt-level = "z"
lto = true
strip = true
panic = "abort"
codegen-units = 1

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
getrandom = {{ version = "0.2", features = ["js"] }}
{sdk_dep}
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
            let meta = PluginMetadata::new()
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
                        sdk::card("Panel Principal", vec![
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
                        sdk::card("Configuración", vec![
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
    let mut password = match std::env::var("EZER_INIT_PASSWORD") {
        Ok(p) => p,
        Err(_) => prompt_password("🔐 Contraseña para proteger la clave de firma: ")?,
    };
    let confirm = match std::env::var("EZER_INIT_PASSWORD") {
        Ok(_) => password.clone(),
        Err(_) => prompt_password("🔐 Confirma la contraseña: ")?,
    };

    if password != confirm {
        return Err("Las contraseñas no coinciden.".to_string());
    }

    let private_key_bytes = hex::decode(&priv_hex)
        .map_err(|_| "Error decodificando clave privada.".to_string())?;
    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&private_key_bytes);

    let encrypted = encrypt_key(&key_arr, &password)?;
    key_arr.zeroize();
    password.zeroize();

    fs::write(path.join(".ezer-key"), &encrypted)
        .map_err(|e| format!("No se pudo guardar la clave cifrada: {}", e))?;

    // Guardar clave pública como referencia
    fs::write(path.join(".ezer-key.pub"), &pub_hex)
        .map_err(|e| format!("No se pudo guardar la clave pública: {}", e))?;

    println!(
        "{} {} Plugin {} listo para desarrollar!",
        ROCKET,
        style("Éxito:").green(),
        style(name).yellow()
    );
    println!("Prueba ejecutando: {} {}", style("cd").cyan(), name);
    println!("Luego: {} build", style("ezer").cyan());
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

    // El .ezer-key.pub es solo referencia, podemos leerlo para mostrar info
    let _pub_info = if pub_key_path.exists() {
        fs::read_to_string(&pub_key_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Pedir contraseña (o usar PLUGIN_SIGN_PASS para CI/CD)
    let mut password = match std::env::var("PLUGIN_SIGN_PASS") {
        Ok(p) => p,
        Err(_) => prompt_password("🔐 Contraseña de la clave de firma: ")?,
    };
    let key_bytes = decrypt_key(&encrypted, &password)?;
    password.zeroize();
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

    let url = format!("http://localhost:{}", port);
    let _ = webbrowser::open(&url);

    let address = format!("127.0.0.1:{}", port);
    let server = tiny_http::Server::http(&address)
        .map_err(|e| format!("No se pudo iniciar el servidor en {}: {}", address, e))?;

    println!("  {} Servidor corriendo. Presiona Ctrl+C para detener.", style("✓").green());

    let html = include_str!("dev.html");

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/plugin.wasm" => {
                let resp = tiny_http::Response::from_data(wasm_bytes.clone())
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/wasm"[..])
                            .unwrap(),
                    );
                request.respond(resp)
            }
            _ => {
                let resp = tiny_http::Response::from_string(html)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .unwrap(),
                    );
                request.respond(resp)
            }
        };

        if let Err(e) = response {
            eprintln!("Error al responder: {}", e);
        }
    }

    Ok(())
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

    // Leer clave pública
    let pub_key_path = cwd.join(".ezer-key.pub");
    let clave_publica = if pub_key_path.exists() {
        Some(
            fs::read_to_string(&pub_key_path)
                .map_err(|e| format!("No se pudo leer .ezer-key.pub: {}", e))?
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
    if let Some(ref pub_key) = clave_publica {
        upload_body["clave_publica"] = serde_json::json!(pub_key);
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
    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let session_path = cwd.join(".ezer-session");
    let mut session_cookies = String::new();

    // Intentar reusar sesión guardada
    if let Ok(content) = fs::read_to_string(&session_path) {
        session_cookies = content.trim().to_string();
        println!("  → Usando sesión guardada...");
    }

    // Si no hay sesión o falla, pedir login
    if session_cookies.is_empty() {
        let email = prompt("📧 Correo electrónico: ")?;
        let mut password = prompt_password("🔑 Contraseña: ")?;

        let login_body = serde_json::json!({"correo": email, "clave": &password});
        let login_json = serde_json::to_string(&login_body)
            .map_err(|_| "Error serializando login.".to_string())?;

        password.zeroize();

        let login_url = format!("{}/api/v1/auth/login", base);
        println!("  → Iniciando sesión...");

        let resp = client.post(&login_url)
            .header("Content-Type", "application/json")
            .header("x-ezerdesk-request", "true")
            .body(login_json)
            .send()
            .map_err(|e| format!("Error en login: {}", e))?;

        let status = resp.status();

        // Guardar cookies de la sesión ANTES de consumir el body
        let cookies: Vec<String> = resp.headers().get_all(reqwest::header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap_or("").to_string())
            .collect();
        session_cookies = cookies.join("; ");

        let login_resp_text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;

        if !status.is_success() {
            return Err(format!("Login fallido: {}", login_resp_text));
        }

        let _ = fs::write(&session_path, &session_cookies);

        // Verificar si requiere MFA
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&login_resp_text)
            && json.get("status").and_then(|s| s.as_str()) == Some("mfa_required")
        {
            let mfa_token = json["mfa_token"].as_str().unwrap_or("");
            let code = prompt("🔐 Código OTP (6 dígitos): ")?;

            let challenge_body = serde_json::json!({"code": code, "mfa_token": mfa_token});
            let challenge_json = serde_json::to_string(&challenge_body)
                .map_err(|_| "Error serializando MFA.".to_string())?;

            let challenge_url = format!("{}/api/v1/auth/otp/challenge", base);
            println!("  → Verificando OTP...");

            let chal_resp = client.post(&challenge_url)
                .header("Content-Type", "application/json")
                .header("x-ezerdesk-request", "true")
                .header("Cookie", &session_cookies)
                .body(challenge_json)
                .send()
                .map_err(|e| format!("Error en OTP: {}", e))?;

            let chal_status = chal_resp.status();

            // Actualizar cookies tras MFA (antes de consumir body)
            let cookies: Vec<String> = chal_resp.headers().get_all(reqwest::header::SET_COOKIE)
                .iter()
                .map(|v| v.to_str().unwrap_or("").to_string())
                .collect();
            session_cookies = cookies.join("; ");

            if !chal_status.is_success() {
                return Err("Código OTP inválido".to_string());
            }

            let _ = fs::write(&session_path, &session_cookies);
        }
    }

    // Helper para peticiones autenticadas
    let auth_request = |url: &str, method: &str, body: &str| -> Result<String, String> {
        let req = match method {
            "POST" => client.post(url)
                .header("Content-Type", "application/json")
                .header("x-ezerdesk-request", "true")
                .header("Cookie", &session_cookies)
                .body(body.to_string()),
            "GET" => client.get(url)
                .header("Cookie", &session_cookies),
            _ => return Err(format!("Unsupported method: {}", method)),
        };
        let resp = req.send().map_err(|e| format!("Error HTTP: {}", e))?;
        let status = resp.status();
        let text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;

        // Si 401, limpiar sesión para forzar login en próxima ejecución
        if status == reqwest::StatusCode::UNAUTHORIZED {
            let _ = fs::remove_file(&session_path);
        }
        if status.is_success() { Ok(text) } else { Err(text) }
    };

    // ── Registrar clave pública de la organización ────────────────────────
    let pub_key_url = format!("{}/api/v1/auth/public-key", base);
    let reg_key_url = format!("{}/api/v1/auth/register-key", base);

    match clave_publica {
        Some(local_key) => {
            let existing_key =
                auth_request(&pub_key_url, "GET", "").ok().and_then(
                    |resp| {
                        serde_json::from_str::<serde_json::Value>(&resp).ok()
                            .and_then(|j| j["clave_publica"].as_str().map(|s| s.to_string()))
                    },
                );

            match existing_key {
                Some(ref server_key) if server_key == &local_key => {
                    println!("  ✓ Clave pública ya registrada, usando existente.");
                }
                Some(_) => {
                    println!(
                        "  ⚠ La clave pública local difiere de la registrada en el servidor."
                    );
                    print!("  ¿Actualizar clave pública? (s/n): ");
                    let _ = std::io::stdout().flush();
                    let mut input = String::new();
                    let should_update = std::io::stdin().read_line(&mut input).is_ok()
                        && input.trim().eq_ignore_ascii_case("s");
                    if should_update {
                        let reg_body =
                            serde_json::json!({"clave_publica": local_key}).to_string();
                        auth_request(&reg_key_url, "POST", &reg_body)?;
                        println!("  ✓ Clave pública actualizada.");
                    }
                }
                None => {
                    println!("  → Registrando clave pública de la organización...");
                    let reg_body =
                        serde_json::json!({"clave_publica": local_key}).to_string();
                    auth_request(&reg_key_url, "POST", &reg_body)?;
                    println!("  ✓ Clave pública registrada.");
                }
            }
        }
        None => {
            println!("  ⚠ No se encontró clave pública local (.ezer-key.pub). La firma no podrá verificarse.");
        }
    }

    // ── Subir plugin ───────────────────────────────────────────────────────
    let upload_url = format!("{}/api/v1/plugins/upload", base);
    println!("  → Subiendo plugin...");

    let resp = auth_request(&upload_url, "POST", &upload_json)?;

    // Verificar respuesta
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            return Err(format!("El servidor rechazó el plugin: {}", error));
        }
        if let Some(codigo) = json.get("codigo").and_then(|c| c.as_str())
            && codigo == "VALIDACION_FALLIDA"
            && let Some(detalles) = json.get("detalles").and_then(|d| d.as_array())
        {
            let msgs: Vec<String> = detalles.iter()
                .filter_map(|d| d.as_str().map(|s| s.to_string()))
                .collect();
            return Err(format!("Validación fallida:\n  • {}", msgs.join("\n  • ")));
        }
    }

    println!("  {} Plugin publicado exitosamente!", style("✓").green());
    Ok(())
}

fn detect_sdk_dependency(_base: &Path) -> String {
    let sdk_source = Path::new(SDK_SOURCE_DIR);
    if sdk_source.join("Cargo.toml").exists() {
        if let Ok(canonical) = sdk_source.canonicalize() {
            return format!(
                r#"ezerdesk-sdk = {{ version = "{}" }}

[patch.crates-io]
ezerdesk-sdk = {{ path = "{}" }}"#,
                SDK_VERSION,
                canonical.display()
            );
        }
    }
    format!(r#"ezerdesk-sdk = "{}""#, SDK_VERSION)
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
        if path.extension().is_some_and(|ext| ext == "wasm") {
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
        "{} {} Compilando plugin para WebAssembly...",
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
            "{} {} Plugin compilado con éxito.",
            SPARKLES,
            style("¡Listo!").green()
        );
        let wasm_path = get_wasm_path(cwd)?;

        // Firmar automáticamente
        match read_signing_key(cwd) {
            Ok(key_hex) => {
                sign_wasm_with_key(&wasm_path, &key_hex)?;
                println!(
                    "{} {} Plugin firmado automáticamente.",
                    style("🔐").bold(),
                    style("Firma:").green()
                );
            }
            Err(_) => {
                println!(
                    "{} {} No se encontró clave de firma. El plugin no está firmado.",
                    style("⚠️").bold(),
                    style("Advertencia:").yellow()
                );
                println!(
                    "   Para generar una clave: {}",
                    style("ezer init nuevo-plugin && cp nuevo-plugin/.ezer-key .").dim()
                );
            }
        }

        // Optimizar WASM post-compilación
        let original_size = fs::metadata(&wasm_path).map(|m| m.len()).unwrap_or(0);

        let opt_result = wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
            .enable_feature(wasm_opt::Feature::BulkMemory)
            .run(&wasm_path, &wasm_path);

        let final_size = fs::metadata(&wasm_path).map(|m| m.len()).unwrap_or(0);

        match opt_result {
            Ok(()) => {
                let saved = original_size.saturating_sub(final_size);
                println!(
                    "  {} Optimizado: {} → {} ({} menos)",
                    style("⚡").bold(),
                    style(format_size(original_size)).dim(),
                    style(format_size(final_size)).green(),
                    style(format_size(saved)).dim()
                );
            }
            Err(e) => {
                println!(
                    "  {} Error al optimizar WASM: {}",
                    style("❌").bold(),
                    e
                );
            }
        }

        println!(
            "{} {} Binario: {}",
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

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
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
        unsafe { std::env::set_var("EZER_INIT_PASSWORD", "test-password") };
        let base = tmp_dir("init_creates_files");
        let result = init_plugin_at(&base, "test-plugin");

        let plugin = base.join("test-plugin");
        assert!(result.is_ok(), "init_plugin failed: {:?}", result.err());
        assert!(plugin.join("Cargo.toml").exists());
        assert!(plugin.join("src/lib.rs").exists());

        let cargo = fs::read_to_string(plugin.join("Cargo.toml")).unwrap();
        assert!(cargo.contains(r#"name = "test-plugin""#));
        assert!(cargo.contains(r#"edition = "2021""#));
        assert!(cargo.contains(&format!(r#"ezerdesk-sdk"#)));

        let lib = fs::read_to_string(plugin.join("src/lib.rs")).unwrap();
        assert!(lib.contains("#[sdk::main]"));
        assert!(lib.contains("PluginEvent::GetMetadata"));
        assert!(lib.contains("NavItem::new"));

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
