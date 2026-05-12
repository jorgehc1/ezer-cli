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
ezerdesk-sdk = {{ path = "../plugins/sdk" }}
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
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata {
                navigation: vec![NavItem {
                    page_id: "main".to_string(),
                    label: "Mi Plugin".to_string(),
                    icon: "rocket-line".to_string(),
                    category: "operaciones".to_string(),
                    priority: 10,
                }]
            };
            sdk::to_host_response(&meta);
        }
        PluginEvent::PageRequest { page_id } => {
            if page_id == "main" {
                let response = PluginResponse {
                    success: true,
                    ui_widgets: vec![
                        UiWidget::Card {
                            title: "Hola desde Ezer-CLI (Diamond Edition)".to_string(),
                            children: vec![
                                UiWidget::Text { 
                                    content: "Plugin generado con blindaje de memoria automático.".to_string(), 
                                    style: "info".to_string() 
                                }
                            ]
                        }
                    ]
                };
                sdk::to_host_response(&response);
            }
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
