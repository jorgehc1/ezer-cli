use base64::Engine;
use clap::{Parser, Subcommand};
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

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
        /// Usar un ejemplo como plantilla
        #[arg(short, long)]
        example: Option<String>,
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

        /// Precio del plugin en centavos (default: 0)
        #[arg(short, long, default_value = "0")]
        precio: i64,

        /// Si es de pago (default: false)
        #[arg(short = 'p', long)]
        es_pago: bool,

        /// Publicar activado (default: false)
        #[arg(long, default_value = "false", action = clap::ArgAction::Set)]
        activo: bool,

        /// Ruta a una imagen (default: busca plugin.png en el directorio actual)
        #[arg(short, long)]
        imagen: Option<String>,
    },
    /// Envía un plugin para revisión (cambia estado a "pendiente")
    Submit {
        /// ID del plugin a enviar para revisión
        plugin_id: String,

        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,
    },
    /// Aprove un plugin (solo Genesis/org admin)
    Approve {
        /// ID del plugin a aprobar
        plugin_id: String,

        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,

        /// Slug personalizado para el marketplace (opcional)
        #[arg(short, long)]
        slug: Option<String>,
    },
    /// Rechaza un plugin con un motivo (solo Genesis/org admin)
    Reject {
        /// ID del plugin a rechazar
        plugin_id: String,

        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,

        /// Motivo del rechazo
        #[arg(short, long)]
        motivo: String,
    },
    /// Lista plugins pendientes de revisión
    Pending {
        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,
    },
    /// Muestra ejemplos de plugins disponibles
    Examples {
        /// Nombre del ejemplo a copiar (opcional)
        name: Option<String>,
    },
    /// Ejecuta los tests del plugin
    Test,
    /// Despliega el plugin a producción
    Deploy {
        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,
    },
    /// Muestra los logs del plugin en tiempo real
    Logs {
        /// ID del plugin a monitorear
        plugin_id: String,

        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,

        /// Número de líneas a mostrar (default: 50)
        #[arg(short, long, default_value = "50")]
        lines: u32,
    },
    /// Inicia una consola interactiva para el plugin
    Console {
        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,
    },
    /// Publica el plugin en el marketplace
    Marketplace {
        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,
    },
    /// Genera documentación del plugin
    Docs,
    /// Despublica un plugin del marketplace
    Withdraw {
        /// ID del plugin a despublicar
        plugin_id: String,

        /// URL del servidor (default: http://localhost:8000)
        #[arg(short, long)]
        server: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, example } => init_plugin(&name, example),
        Commands::Build => build_plugin(),
        Commands::Dev { port } => dev_server(port),
        Commands::Publish { server, precio, es_pago, activo, imagen } => publish_plugin(server, precio, es_pago, activo, imagen),
        Commands::Submit { plugin_id, server } => submit_plugin(plugin_id, server),
        Commands::Approve { plugin_id, server, slug } => approve_plugin(plugin_id, server, slug),
        Commands::Reject { plugin_id, server, motivo } => reject_plugin(plugin_id, server, motivo),
        Commands::Pending { server } => list_pending(server),
        Commands::Examples { name } => show_examples(name),
        Commands::Test => test_plugin(),
        Commands::Deploy { server } => deploy_plugin(server),
        Commands::Logs { plugin_id, server, lines } => show_logs(plugin_id, server, lines),
        Commands::Console { server } => start_console(server),
        Commands::Marketplace { server } => publish_to_marketplace(server),
        Commands::Docs => generate_docs(),
        Commands::Withdraw { plugin_id, server } => withdraw_from_marketplace(plugin_id, server),
    };

    if let Err(msg) = result {
        eprintln!("{} {}", CROSS_MARK, style(msg).red());
        std::process::exit(1);
    }
}

// ── Plugin Management Commands ───────────────────────────────────────────────

fn test_plugin() -> Result<(), String> {
    println!("{} Ejecutando tests del plugin...", style("🧪").bold());
    
    let output = Command::new("cargo")
        .args(["test"])
        .output()
        .map_err(|e| format!("Error ejecutando tests: {}", e))?;
    
    if output.status.success() {
        println!("{} Todos los tests pasaron exitosamente.", style("✓").green());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Tests fallaron:\n{}", stderr))
    }
}

fn deploy_plugin(server: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');
    
    println!("{} Desplegando plugin a producción...", style("🚀").bold());
    println!("  Servidor: {}", base);
    
    // Primero compilar
    build_plugin()?;
    
    // Luego publicar con activo=true
    publish_plugin(Some(base.to_string()), 0, false, true, None)?;
    
    println!("{} Plugin desplegado exitosamente!", style("✓").green());
    Ok(())
}

fn show_logs(plugin_id: String, server: Option<String>, lines: u32) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');

    println!("{} Monitoreando logs del plugin {}...", style("📋").bold(), style(&plugin_id).cyan());
    println!("  Servidor: {}", base);
    println!("  Líneas históricas: {}", lines);
    println!();

    // Leer sesión
    let session_path = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer directorio: {}", e))?
        .join(".ezer-session");
    let cookies = read_session(&session_path)?;

    // Obtener ticket WebSocket
    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let ticket_resp = client
        .post(format!("{}/api/v1/auth/ws-ticket", base))
        .header("Content-Type", "application/json")
        .header("x-ezerdesk-request", "true")
        .header("Cookie", &cookies)
        .body("{}".to_string())
        .send()
        .map_err(|e| format!("Error obteniendo ticket WS: {}", e))?;

    let ticket_json: serde_json::Value = serde_json::from_str(
        &ticket_resp
            .text()
            .map_err(|_| "Error leyendo ticket".to_string())?,
    )
    .map_err(|_| "Error parseando ticket".to_string())?;

    let ticket = ticket_json["ticket"]
        .as_str()
        .ok_or("No se recibió ticket WebSocket")?;

    // Construir URL WebSocket
    let ws_url = format!(
        "ws://{}/api/v1/plugins/{}/logs/ws?ticket={}",
        base.trim_start_matches("http://").trim_start_matches("https://"),
        plugin_id,
        ticket
    );

    println!("{} Conectando...", style("⏳").yellow());

    // Conectar y recibir
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Error creando runtime: {}", e))?;

    rt.block_on(async {
        use futures_util::StreamExt;
        use tokio_tungstenite::{connect_async, tungstenite::Message};

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("Error conectando WebSocket: {}", e))?;

        let (_, mut read) = ws_stream.split();

        println!(
            "{} Logs en tiempo real (Ctrl+C para salir):",
            style("✓").green()
        );
        println!();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(events) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Si es un array (logs históricos)
                        if let Some(arr) = events.as_array() {
                            for entry in arr {
                                print_log_entry(entry);
                            }
                        } else {
                            // Evento individual
                            print_log_entry(&events);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!(
                        "{} Conexión cerrada por el servidor.",
                        style("⚠").yellow()
                    );
                    break;
                }
                Err(e) => {
                    eprintln!("{} Error WebSocket: {}", style("❌").bold(), e);
                    break;
                }
                _ => {}
            }
        }
        Ok::<(), String>(())
    })
}

fn print_log_entry(event: &serde_json::Value) {
    let tipo = event["tipo"].as_str().unwrap_or("unknown");

    match tipo {
        "plugin_log_historico" | "plugin_log_creado" => {
            let datos = &event["datos"];
            let timestamp = datos["creado_en"].as_i64().unwrap_or(0);
            let nivel = datos["nivel"].as_str().unwrap_or("info");
            let mensaje = datos["mensaje"].as_str().unwrap_or("");
            let fuel = datos["fuel_consumido"].as_i64().unwrap_or(0);
            let memory = datos["memoria_usada"].as_i64().unwrap_or(0);

            // Timestamp formateado
            let dt = chrono::DateTime::from_timestamp(timestamp, 0)
                .map(|d| d.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".to_string());

            // Color según nivel
            let nivel_colored = match nivel {
                "ERROR" | "error" => style(nivel).red().bold(),
                "WARN" | "warn" => style(nivel).yellow(),
                "INFO" | "info" => style(nivel).green(),
                _ => style(nivel).dim(),
            };

            let fuel_str = if fuel > 0 {
                format!(" fuel:{}", fuel)
            } else {
                String::new()
            };
            let mem_str = if memory > 0 {
                format!(" mem:{}", memory)
            } else {
                String::new()
            };

            println!(
                "  {} {} {}{}{}",
                style(dt).dim(),
                nivel_colored,
                style(mensaje).white(),
                style(fuel_str).dim(),
                style(mem_str).dim(),
            );
        }
        _ => {
            // Otros eventos ignorados
        }
    }
}

fn start_console(server: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');
    
    println!("{} Consola interactiva del plugin", style("💻").bold());
    println!("  Servidor: {}", base);
    println!("  Escribe 'help' para ver comandos disponibles");
    println!("  Escribe 'exit' para salir");
    println!();
    
    // TODO: Implementar consola interactiva
    println!("  (Funcionalidad en desarrollo)");
    Ok(())
}

fn publish_to_marketplace(server: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');
    
    println!("{} Publicando plugin en marketplace...", style("🏪").bold());
    println!("  Servidor: {}", base);
    
    // Primero compilar y publicar
    build_plugin()?;
    publish_plugin(Some(base.to_string()), 0, false, false, None)?;
    
    // Luego enviar para revisión
    // TODO: Obtener el ID del plugin recién creado
    println!("  Plugin publicado. Use 'ezer submit <plugin_id>' para enviar a revisión.");
    Ok(())
}

fn generate_docs() -> Result<(), String> {
    println!("{} Generando documentación del plugin...", style("📝").bold());
    
    // Generar documentación básica
    let docs = r#"# Documentación del Plugin

## Uso
1. Instalar el plugin desde el marketplace
2. Activar el plugin desde Gestión → Plugins
3. Acceder desde el menú lateral

## Eventos Soportados
- GetMetadata: Retorna metadatos del plugin
- PageRequest: Renderiza páginas del plugin
- PluginAction: Maneja acciones del usuario

## Widgets Disponibles
- Card, Text, Button, Input, Select, Switch
- Table, Chart, Badge, Icon, Divider, Modal
- NumberInput, DateInput
"#;
    
    std::fs::write("PLUGIN_DOCS.md", docs)
        .map_err(|e| format!("Error escribiendo documentación: {}", e))?;
    
    println!("{} Documentación generada: PLUGIN_DOCS.md", style("✓").green());
    Ok(())
}

fn withdraw_from_marketplace(plugin_id: String, server: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');
    
    println!("{} Despublicando plugin del marketplace...", style("📤").bold());
    println!("  Plugin ID: {}", plugin_id);
    println!("  Servidor: {}", base);
    
    // Autenticarse
    let session_path = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer directorio: {}", e))?
        .join(".ezer-session");
    
    let cookies = if session_path.exists() {
        fs::read_to_string(&session_path)
            .map(|c| c.trim().to_string())
            .unwrap_or_default()
    } else {
        return Err("No hay sesión guardada. Ejecuta 'ezer publish' primero.".to_string());
    };
    
    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;
    
    let resp = client.post(format!("{}/api/v1/plugins/{}/withdraw", base, plugin_id))
        .header("Content-Type", "application/json")
        .header("x-ezerdesk-request", "true")
        .header("Cookie", &cookies)
        .body("{}".to_string())
        .send()
        .map_err(|e| format!("Error HTTP: {}", e))?;
    
    let status = resp.status();
    let text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;
    
    if status.is_success() {
        println!("{} Plugin despublicado del marketplace.", style("✓").green());
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(msg) = json.get("mensaje").and_then(|m| m.as_str()) {
                println!("  {}", msg);
            }
            if let Some(note) = json.get("nota").and_then(|n| n.as_str()) {
                println!("  {}", note);
            }
        }
        Ok(())
    } else {
        Err(format!("Error {}: {}", status, text))
    }
}

fn prompt_password(prompt: &str) -> Result<String, String> {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let password = rpassword::read_password()
        .map_err(|e| format!("Error leyendo contraseña: {}", e))?;
    Ok(password)
}

fn init_plugin(name: &str, example: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;
    init_plugin_at(&cwd, name, example)
}

fn init_plugin_at(base: &Path, name: &str, example: Option<String>) -> Result<(), String> {
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
edition = "2024"

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

    let lib_rs = match example {
        Some(example_name) => {
            // Buscar y usar el ejemplo como plantilla
            let examples_dir = env!("CARGO_MANIFEST_DIR");
            let example_path = Path::new(examples_dir).join("examples").join(format!("{}.rs", example_name));
            
            if !example_path.exists() {
                // Buscar por prefijo
                let examples_dir = Path::new(examples_dir).join("examples");
                let entries: Vec<_> = fs::read_dir(&examples_dir)
                    .map_err(|e| format!("Error leyendo directorio de ejemplos: {}", e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
                    .collect();
                
                let matching = entries.iter().find(|e| {
                    e.path().file_stem()
                        .and_then(|s| s.to_str())
                        .map_or(false, |s| s.starts_with(&example_name))
                });
                
                match matching {
                    Some(entry) => {
                        let content = fs::read_to_string(entry.path())
                            .map_err(|e| format!("Error leyendo ejemplo: {}", e))?;
                        println!("  {} Usando ejemplo: {}", style("✓").green(), 
                            entry.path().file_stem().and_then(|s| s.to_str()).unwrap_or("?"));
                        content
                    }
                    None => {
                        return Err(format!("No se encontró el ejemplo '{}'. Usa 'ezer examples' para ver los disponibles.", example_name));
                    }
                }
            } else {
                let content = fs::read_to_string(&example_path)
                    .map_err(|e| format!("Error leyendo ejemplo: {}", e))?;
                println!("  {} Usando ejemplo: {}", style("✓").green(), example_name);
                content
            }
        }
        None => {
            // Usar template por defecto
            r#"use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(NavItem::new("dashboard", "Mi Plugin", "rocket-line")
                    .category("operaciones")
                    .priority(10))
                .name("Mi Plugin")
                .description("Describe brevemente qué hace tu plugin.")
                .version(env!("CARGO_PKG_VERSION"));
            sdk::to_host_response(&meta);
        }

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

        PluginEvent::PluginAction { action, data } => {
            sdk::log(&format!("Acción '{}' con datos: {:?}", action, data));
            sdk::respond_ok("Procesado");
        }

        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Ticket creado: {}", ticket.asunto));
        }

        _ => {}
    }

    0
}
"#.to_string()
        }
    };
    fs::write(src.join("lib.rs"), lib_rs)
        .map_err(|e| format!("No se pudo escribir src/lib.rs: {}", e))?;

    // Crear imagen placeholder (plugin.png) — se reemplaza con la imagen real del plugin
    let placeholder_b64 = "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAMklEQVQ4T2NkYPj/n4EBBJgYKAQMFFnAxEBBA8aBpoAjQ4ECUUiRmYwUiIaIJAAAAP//wzYRkQKZQpZbAAAAAElFTkSuQmCC";
    let placeholder_png = base64::engine::general_purpose::STANDARD
        .decode(placeholder_b64)
        .map_err(|_| "Error decodificando placeholder PNG".to_string())?;
    fs::write(path.join("plugin.png"), &placeholder_png)
        .map_err(|e| format!("No se pudo escribir plugin.png: {}", e))?;

    println!(
        "{} {} Plugin {} listo para desarrollar!",
        ROCKET,
        style("Éxito:").green(),
        style(name).yellow()
    );
    println!("Prueba ejecutando: {} {}", style("cd").cyan(), name);
    println!("Luego: {} build", style("ezer").cyan());

    Ok(())
}

fn build_plugin() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;
    build_plugin_at(&cwd)
}

fn dev_server(port: u16) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;

    let wasm_path = get_wasm_path(&cwd)?;
    let src_dir = cwd.join("src");
    let cargo_toml = cwd.join("Cargo.toml");

    println!(
        "{} {}Servidor de desarrollo",
        style("🌐").bold(),
        style("Ezerdesk Dev Server").cyan()
    );
    println!("  Plugin:  {}", wasm_path.display());
    println!("  Puerto:  http://localhost:{}", port);
    println!();

    // Contador de rebuilds compartido entre hilos
    let rebuild_count: std::sync::Arc<std::sync::atomic::AtomicUsize> =
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let rebuild_count_watcher = rebuild_count.clone();

    // Hilo: vigilar cambios y auto-rebuild
    let cwd_clone = cwd.clone();
    std::thread::spawn(move || {
        use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
        let (watch_tx, watch_rx) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();
        if let Ok(mut watcher) = RecommendedWatcher::new(move |res| {
            let _ = watch_tx.send(res);
        }, Config::default()) {
            let _ = watcher.watch(&src_dir, RecursiveMode::Recursive);
            let _ = watcher.watch(&cargo_toml, RecursiveMode::NonRecursive);
            loop {
                match watch_rx.recv() {
                    Ok(Ok(_)) => {
                        // Pequeña pausa para evitar builds múltiples por un solo cambio
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        // Vaciar eventos encolados durante el debounce
                        while let Ok(_) = watch_rx.try_recv() {}

                        println!("  {} Cambio detectado, recompilando...", style("🔄").bold());
                        let output = std::process::Command::new("cargo")
                            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
                            .current_dir(&cwd_clone)
                            .output();
                        match output {
                            Ok(out) if out.status.success() => {
                                rebuild_count_watcher.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                println!("  {} Recompilación exitosa.", style("✓").green());
                            }
                            Ok(out) => {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                eprintln!("  {} Error de compilación:\n{}", style("❌").bold(), stderr);
                            }
                            Err(e) => {
                                eprintln!("  {} Error al ejecutar cargo: {}", style("❌").bold(), e);
                            }
                        }
                    }
                    _ => break,
                }
            }
        }
    });

    let address = format!("127.0.0.1:{}", port);
    let server = tiny_http::Server::http(&address)
        .map_err(|e| format!("No se pudo iniciar el servidor en {}: {}", address, e))?;

    println!("  {} Abriendo navegador...", style("→").bold());
    let url = format!("http://localhost:{}", port);
    let _ = webbrowser::open(&url);
    println!("  {} Servidor corriendo. Presiona Ctrl+C para detener.", style("✓").green());

    let html = include_str!("dev.html");

    for request in server.incoming_requests() {
        let url_path = request.url().split('?').next().unwrap_or("").to_string();
        let response = match url_path.as_str() {
            "/plugin.wasm" => {
                let mut data = Vec::new();
                for _ in 0..5 {
                    if let Ok(content) = fs::read(&wasm_path) {
                        if !content.is_empty() {
                            data = content;
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if data.is_empty() {
                    let resp = tiny_http::Response::from_string("WASM no disponible o en compilación.")
                        .with_status_code(503);
                    request.respond(resp)
                } else {
                    let resp = tiny_http::Response::from_data(data)
                        .with_header(
                            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/wasm"[..])
                                .unwrap(),
                        );
                    request.respond(resp)
                }
            }
            "/status" => {
                let count = rebuild_count.load(std::sync::atomic::Ordering::SeqCst);
                let body = format!("{{\"rebuild\":{}}}", count);
                let resp = tiny_http::Response::from_string(body)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
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
    precio: i64,
    es_pago: bool,
    activo: bool,
    imagen: Option<String>,
) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;

    // Leer nombre y versión desde Cargo.toml
    let (plugin_name, plugin_version) = read_plugin_manifest(&cwd)?;

    // Obtener ruta del WASM
    let wasm_path = get_wasm_path(&cwd)?;

    // Leer WASM
    let wasm_bytes = fs::read(&wasm_path)
        .map_err(|e| format!("No se pudo leer .wasm: {}", e))?;

    // Codificar WASM a base64
    let codigo_b64 = base64::engine::general_purpose::STANDARD.encode(&wasm_bytes);

    // Construir JSON del upload
    let mut upload_body = serde_json::json!({
        "codigo_wasm_base64": codigo_b64,
        "precio": precio,
        "es_pago": es_pago,
        "activo": activo,
    });

    // Procesar imagen — por defecto busca plugin.png, o la ruta indicada con --image
    let img_path = imagen.clone().unwrap_or_else(|| {
        let default = cwd.join("plugin.png");
        if default.exists() {
            default.to_string_lossy().to_string()
        } else {
            String::new()
        }
    });
    if !img_path.is_empty() {
        let img_bytes = fs::read(&img_path)
            .map_err(|e| format!("No se pudo leer la imagen '{}': {}", img_path, e))?;
        let ext = img_path
            .rsplit('.')
            .next()
            .unwrap_or("png")
            .to_lowercase();
        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            _ => "image/png",
        };
        let img_b64 = base64::engine::general_purpose::STANDARD.encode(&img_bytes);
        println!("  {} Imagen adjunta: {} ({})", style("🖼").bold(), img_path, mime);
        upload_body["imagen_base64"] = serde_json::json!(img_b64);
        upload_body["imagen_tipo"] = serde_json::json!(mime);
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
        let password = prompt_password("🔑 Contraseña: ")?;

        let login_body = serde_json::json!({"correo": email, "clave": &password});
        let login_json = serde_json::to_string(&login_body)
            .map_err(|_| "Error serializando login.".to_string())?;

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
        }

        // Guardar la sesión SOLO si todo el flujo (incluido MFA) fue exitoso, y con permisos estrictos
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let _ = options.open(&session_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, session_cookies.as_bytes()));
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

    // ── Subir plugin ───────────────────────────────────────────────────────
    let upload_url = format!("{}/api/v1/plugins/upload", base);
    println!("  → Subiendo plugin...");

    let resp = auth_request(&upload_url, "POST", &upload_json)?;

    // Verificar respuesta
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
        if let Some(codigo) = json.get("codigo").and_then(|c| c.as_str())
            && codigo == "VALIDACION_FALLIDA"
            && let Some(detalles) = json.get("detalles").and_then(|d| d.as_array())
        {
            let msgs: Vec<String> = detalles.iter()
                .filter_map(|d| d.as_str().map(|s| s.to_string()))
                .collect();
            return Err(format!("Validación fallida:\n  • {}", msgs.join("\n  • ")));
        }

        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            let codigo = json.get("codigo").and_then(|c| c.as_str()).unwrap_or("DESCONOCIDO");
            return Err(format!("El servidor rechazó el plugin ({}): {}", codigo, error));
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

// ── Plugin Review Commands ──────────────────────────────────────────────────

fn submit_plugin(plugin_id: String, server: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');

    println!(
        "{} Enviando plugin {} para revisión...",
        ROCKET,
        style(&plugin_id).cyan()
    );

    let session_path = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer directorio: {}", e))?
        .join(".ezer-session");

    let cookies = read_session(&session_path)?;
    let url = format!("{}/api/v1/plugins/{}/submit", base, plugin_id);

    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let resp = client.post(&url)
        .header("Content-Type", "application/json")
        .header("x-ezerdesk-request", "true")
        .header("Cookie", &cookies)
        .body("{}".to_string())
        .send()
        .map_err(|e| format!("Error HTTP: {}", e))?;

    let status = resp.status();
    let text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;

    if status.is_success() {
        println!("{} Plugin enviado para revisión exitosamente.", style("✓").green());
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(msg) = json.get("mensaje").and_then(|m| m.as_str()) {
                println!("  {}", msg);
            }
        }
        Ok(())
    } else {
        Err(format!("Error {}: {}", status, text))
    }
}

fn approve_plugin(plugin_id: String, server: Option<String>, slug: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');

    println!(
        "{} Aprobando plugin {}...",
        style("✓").green(),
        style(&plugin_id).cyan()
    );

    let session_path = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer directorio: {}", e))?
        .join(".ezer-session");

    let cookies = read_session(&session_path)?;
    let url = format!("{}/api/v1/plugins/{}/approve", base, plugin_id);

    let body = match slug {
        Some(s) => serde_json::json!({"slug": s}).to_string(),
        None => "{}".to_string(),
    };

    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let resp = client.post(&url)
        .header("Content-Type", "application/json")
        .header("x-ezerdesk-request", "true")
        .header("Cookie", &cookies)
        .body(body)
        .send()
        .map_err(|e| format!("Error HTTP: {}", e))?;

    let status = resp.status();
    let text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;

    if status.is_success() {
        println!("{} Plugin aprobado y publicado en marketplace.", style("✓").green());
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(msg) = json.get("mensaje").and_then(|m| m.as_str()) {
                println!("  {}", msg);
            }
        }
        Ok(())
    } else {
        Err(format!("Error {}: {}", status, text))
    }
}

fn reject_plugin(plugin_id: String, server: Option<String>, motivo: String) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');

    println!(
        "{} Rechazando plugin {}...",
        CROSS_MARK,
        style(&plugin_id).cyan()
    );

    let session_path = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer directorio: {}", e))?
        .join(".ezer-session");

    let cookies = read_session(&session_path)?;
    let url = format!("{}/api/v1/plugins/{}/reject", base, plugin_id);

    let body = serde_json::json!({"motivo": motivo}).to_string();

    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let resp = client.post(&url)
        .header("Content-Type", "application/json")
        .header("x-ezerdesk-request", "true")
        .header("Cookie", &cookies)
        .body(body)
        .send()
        .map_err(|e| format!("Error HTTP: {}", e))?;

    let status = resp.status();
    let text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;

    if status.is_success() {
        println!("{} Plugin rechazado.", style("✓").green());
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(msg) = json.get("mensaje").and_then(|m| m.as_str()) {
                println!("  {}", msg);
            }
        }
        Ok(())
    } else {
        Err(format!("Error {}: {}", status, text))
    }
}

fn list_pending(server: Option<String>) -> Result<(), String> {
    let base_url = server.unwrap_or_else(|| {
        std::env::var("EZER_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_string())
    });
    let base = base_url.trim_end_matches('/');

    println!("{} Listando plugins pendientes de revisión...", LOOKING_GLASS);

    let session_path = std::env::current_dir()
        .map_err(|e| format!("No se pudo leer directorio: {}", e))?
        .join(".ezer-session");

    let cookies = read_session(&session_path)?;
    let url = format!("{}/api/v1/plugins/all", base);

    let client = reqwest::blocking::Client::builder()
        .cookie_store(false)
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let resp = client.get(&url)
        .header("Cookie", &cookies)
        .send()
        .map_err(|e| format!("Error HTTP: {}", e))?;

    let status = resp.status();
    let text = resp.text().map_err(|_| "Error leyendo respuesta".to_string())?;

    if !status.is_success() {
        return Err(format!("Error {}: {}", status, text));
    }

    let plugins: Vec<serde_json::Value> = serde_json::from_str(&text)
        .map_err(|e| format!("Error parseando JSON: {}", e))?;

    let pending: Vec<&serde_json::Value> = plugins.iter()
        .filter(|p| p.get("estado_revision").and_then(|s| s.as_str()) == Some("pendiente"))
        .collect();

    if pending.is_empty() {
        println!("  No hay plugins pendientes de revisión.");
    } else {
        println!("  {} plugins pendientes:", pending.len());
        println!();
        for p in &pending {
            let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let name = p.get("nombre").and_then(|v| v.as_str()).unwrap_or("?");
            let version = p.get("version").and_then(|v| v.as_str()).unwrap_or("?");
            let verified = p.get("firma_verificada").and_then(|v| v.as_bool()).unwrap_or(false);
            let verified_str = if verified { "✓" } else { "—" };
            println!(
                "  {} {} v{} [firma: {}]",
                style(name).cyan(),
                style(id).dim(),
                style(version).yellow(),
                verified_str
            );
        }
    }

    Ok(())
}

fn read_session(session_path: &Path) -> Result<String, String> {
    if session_path.exists() {
        fs::read_to_string(session_path)
            .map(|c| c.trim().to_string())
            .map_err(|e| format!("No se pudo leer sesión: {}", e))
    } else {
        Err("No hay sesión guardada. Ejecuta 'ezer publish' primero para autenticarte.".to_string())
    }
}

// ── Examples Command ─────────────────────────────────────────────────────────

fn show_examples(name: Option<String>) -> Result<(), String> {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    
    if !examples_dir.exists() {
        return Err("No se encontró el directorio de ejemplos.".to_string());
    }
    
    match name {
        Some(example_name) => {
            // Buscar ejemplo por nombre o prefijo
            let entries: Vec<_> = fs::read_dir(&examples_dir)
                .map_err(|e| format!("Error leyendo directorio: {}", e))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
                .collect();
            
            let matching = entries.iter().find(|e| {
                e.path().file_stem()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| s == &example_name || s.starts_with(&example_name))
            });
            
            if let Some(entry) = matching {
                let content = fs::read_to_string(entry.path())
                    .map_err(|e| format!("Error leyendo ejemplo: {}", e))?;
                println!("{}", content);
                Ok(())
            } else {
                println!("{} Ejemplos disponibles:", style("📚").bold());
                list_examples(&examples_dir)?;
                Err(format!("No se encontró el ejemplo '{}'", example_name))
            }
        }
        None => {
            // Listar todos los ejemplos
            println!("{} Ejemplos de Plugins EzerDesk", style("📚").bold());
            println!();
            list_examples(&examples_dir)?;
            println!();
            println!("{} Para ver un ejemplo: ezer examples <nombre>", style("💡").cyan());
            println!("{} Ejemplo: ezer examples 01_dashboard", style("💡").cyan());
            Ok(())
        }
    }
}

fn list_examples(examples_dir: &Path) -> Result<(), String> {
    let entries: Vec<_> = fs::read_dir(examples_dir)
        .map_err(|e| format!("Error leyendo directorio: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .collect();
    
    for entry in entries {
        let path = entry.path();
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("?");
        let description = get_example_description(name);
        println!("  {} {} - {}", style("•").green(), style(name).cyan(), description);
    }
    Ok(())
}

fn get_example_description(name: &str) -> String {
    match name {
        "01_dashboard_metricas" => "Dashboard con métricas, charts y tablas".to_string(),
        "02_monitor_sla" => "Monitor de SLA con alertas y eventos".to_string(),
        "03_exportador_datos" => "Exportación de datos a CSV".to_string(),
        "04_reportes_programados" => "Reportes automáticos con cron".to_string(),
        "05_hub_integraciones" => "Integraciones con OAuth (Google, Slack, GitHub)".to_string(),
        "06_gestor_tickets" => "Gestión de tickets con filtros".to_string(),
        "07_formulario_contacto" => "Formulario completo con validación".to_string(),
        "08_buscador_avanzado" => "Búsqueda multi-entidad".to_string(),
        "09_chatbot_inteligente" => "Chatbot con IA y HTTP".to_string(),
        "10_sistema_encuestas" => "Sistema de encuestas con analytics".to_string(),
        "11_gestor_conocimiento" => "Gestión de base de conocimiento".to_string(),
        "12_monitor_chat" => "Monitor de chat en vivo".to_string(),
        "13_admin_workflows" => "Administración de workflows".to_string(),
        "14_gestor_cupones" => "Gestión de cupones de descuento".to_string(),
        "15_dashboard_agentes" => "Dashboard de rendimiento de agentes".to_string(),
        "16_reporte_satisfaccion" => "Reportes de satisfacción del cliente".to_string(),
        "17_integracion_email" => "Integración con email".to_string(),
        "18_gestor_notificaciones" => "Gestor de notificaciones multi-canal".to_string(),
        "19_monitor_rendimiento" => "Monitor de rendimiento del sistema".to_string(),
        "20_panel_configuracion" => "Panel de configuración del sistema".to_string(),
        "21_monitor_red_snmp" => "Monitoreo de red via SNMP/Zabbix".to_string(),
        "22_receptor_traps_snmp" => "Receptor de traps SNMP".to_string(),
        "23_bot_telegram_soporte" => "Bot de Telegram para soporte".to_string(),
        "24_notificador_telegram" => "Notificaciones automáticas via Telegram".to_string(),
        "25_generador_facturas" => "Generación de facturas y envío por email".to_string(),
        "26_portal_clientes" => "Portal self-service para clientes".to_string(),
        "27_api_integration_hub" => "Hub de integraciones con servicios externos".to_string(),
        "28_integracion_phone_sms" => "Integración con Phone/SMS".to_string(),
        "29_inventario_custom" => "Gestión de inventario con modelos de datos custom".to_string(),
        "30_integracion_phone_call" => "Integración con Phone/Call (llamadas)".to_string(),
        _ => "Ejemplo de plugin".to_string(),
    }
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
        assert!(cargo.contains(r#"edition = "2024""#));
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
