use clap::{Parser, Subcommand};
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
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
    /// Compila el plugin a WebAssembly (wasm32-unknown-unknown)
    Build,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => init_plugin(&name),
        Commands::Build => build_plugin(),
    };

    if let Err(msg) = result {
        eprintln!("{} {}", CROSS_MARK, style(msg).red());
        std::process::exit(1);
    }
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
    fs::write(src.join("lib.rs"), lib_rs)
        .map_err(|e| format!("No se pudo escribir src/lib.rs: {}", e))?;

    println!(
        "{} {}Plugin {} listo para desarrollar!",
        ROCKET,
        style("Éxito:").green(),
        style(name).yellow()
    );
    println!("Prueba ejecutando: {} {}", style("cd").cyan(), name);
    println!("Luego: {} {}", style("ezer").cyan(), "build");

    Ok(())
}

fn build_plugin() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("No se pudo leer el directorio actual: {}", e))?;
    build_plugin_at(&cwd)
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
        println!(
            "{} {}El binario se encuentra en: {}",
            LOOKING_GLASS,
            style("Nota:").blue(),
            style("target/wasm32-unknown-unknown/release/*.wasm").yellow()
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
