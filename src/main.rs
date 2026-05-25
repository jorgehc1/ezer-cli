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
    /// Compila el plugin a WebAssembly (wasm32-unknown-unknown)
    Build,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            init_plugin(&name);
        }
        Commands::Build => {
            build_plugin();
        }
    }
}

fn init_plugin(name: &str) {
    println!(
        "{} {}Creando nuevo plugin: {}...",
        SPARKLES,
        style("Ezerdesk").bold().cyan(),
        style(name).yellow()
    );

    let path = Path::new(name);
    if path.exists() {
        println!("{}", style("Error: La carpeta ya existe.").red());
        return;
    }

    // Crear estructura de directorios
    fs::create_dir_all(path.join("src")).expect("No se pudo crear la carpeta src");

    // Crear Cargo.toml
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

    let mut file = fs::File::create(path.join("Cargo.toml")).expect("No se pudo crear Cargo.toml");
    file.write_all(cargo_toml.as_bytes()).expect("No se pudo escribir en Cargo.toml");

    // Crear src/lib.rs
    let lib_rs = r#"use ezerdesk_sdk as sdk;
use sdk::{UiWidget, PluginEvent, PluginResponse, PluginMetadata, NavItem};

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        // 1. Metadatos: Define cómo aparece el plugin en la navegación lateral
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata {
                host: "ezerdesk".to_string(),
                navigation: vec![NavItem {
                    page_id: "dashboard".to_string(),
                    label: "Mi Plugin".to_string(),
                    icon: "rocket-line".to_string(),
                    category: "operaciones".to_string(),
                    priority: 10,
                }]
            };
            sdk::to_host_response(&meta);
        }

        // 2. Vistas de Módulo: Se dispara cuando el usuario entra a una página completa del plugin
        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "dashboard" => {
                    let response = PluginResponse {
                        success: true,
                        ui_widgets: vec![
                            UiWidget::Card {
                                title: "Panel Principal del Módulo".to_string(),
                                children: vec![
                                    UiWidget::Text { 
                                        content: "Bienvenido a la vista principal de tu plugin.".to_string(), 
                                        style: "info".to_string() 
                                    }
                                ]
                            }
                        ]
                    };
                    sdk::to_host_response(&response);
                },
                _ => {}
            }
        }

        // 3. Fragmentos de UI: Se dispara para inyectar UI en lugares específicos del Host
        PluginEvent::GetUiFragments { location } => {
            match location.as_str() {
                // Seccion de "Configuración del Módulo" en el panel administrativo
                "plugin_settings" => {
                    let response = PluginResponse {
                        success: true,
                        ui_widgets: vec![
                            UiWidget::Card {
                                title: "Configuración Personalizada".to_string(),
                                children: vec![
                                    UiWidget::Text { 
                                        content: "Ajusta los parámetros de funcionamiento de este módulo.".to_string(), 
                                        style: "muted".to_string() 
                                    },
                                    UiWidget::Input {
                                        label: "API Key de Servicio".to_string(),
                                        name: "api_key".to_string(),
                                        placeholder: "Ingresa tu llave...".to_string(),
                                        value: "".to_string(),
                                    }
                                ]
                            }
                        ]
                    };
                    sdk::to_host_response(&response);
                },
                _ => {}
            }
        }

        // 4. Acciones: Manejo de clics en botones y envíos de formularios
        PluginEvent::PluginAction { action, data } => {
            sdk::log(&format!("Acción recibida: {} con datos: {:?}", action, data));
        }

        _ => {}
    }

    0
}
"#;

    let mut file = fs::File::create(path.join("src/lib.rs")).expect("No se pudo crear src/lib.rs");
    file.write_all(lib_rs.as_bytes()).expect("No se pudo escribir en src/lib.rs");

    println!(
        "{} {}Plugin {} listo para desarrollar!",
        ROCKET,
        style("Éxito:").green(),
        style(name).yellow()
    );
    println!("Prueba ejecutando: {} {}", style("cd").cyan(), name);
    println!("Luego: {} {}", style("ezer").cyan(), "build");
}

fn build_plugin() {
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
        .output();

    pb.finish_and_clear();

    match output {
        Ok(out) if out.status.success() => {
            println!(
                "{} {}Plugin compilado con éxito.",
                SPARKLES,
                style("¡Listo!").green()
            );
            
            // Intentar encontrar el archivo .wasm
            println!(
                "{} {}El binario se encuentra en: {}",
                LOOKING_GLASS,
                style("Nota:").blue(),
                style("target/wasm32-unknown-unknown/release/*.wasm").yellow()
            );
        }
        Ok(out) => {
            println!(
                "{} {}Fallo en la compilación:",
                Emoji("❌ ", ""),
                style("Error:").red()
            );
            println!("{}", String::from_utf8_lossy(&out.stderr));
        }
        Err(e) => {
            println!(
                "{} {}No se pudo ejecutar cargo: {}",
                Emoji("❌ ", ""),
                style("Error:").red(),
                e
            );
        }
    }
}
