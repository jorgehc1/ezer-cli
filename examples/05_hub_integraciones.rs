// Ejemplo 5: Hub de Integraciones
// Features: OAuth (múltiples providers), HTTP, KV store
// Demuestra: Conexión con múltiples servicios, autenticación

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("integrations", "Integraciones", "links-line")
                        .category("sistema")
                        .priority(30)
                )
                .name("Hub de Integraciones")
                .description("Conecta EzerDesk con servicios externos")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "integrations" => render_integrations_page(),
                "google" => render_google_integration(),
                "slack" => render_slack_integration(),
                "github" => render_github_integration(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, .. } => {
            match action.as_str() {
                "connect_google" => connect_google(),
                "connect_slack" => connect_slack(),
                "connect_github" => connect_github(),
                "sync_google" => sync_google_contacts(),
                "send_slack" => send_slack_message(),
                "fetch_github" => fetch_github_repos(),
                "disconnect" => disconnect_service(),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

fn render_integrations_page() {
    // Verificar estado de integraciones
    let google_token = sdk::kv_get_val("google_token");
    let slack_token = sdk::kv_get_val("slack_token");
    let github_token = sdk::kv_get_val("github_token");

    sdk::respond(sdk::widgets![
        sdk::card("Centro de Integraciones", vec![
            sdk::text("Conecta EzerDesk con tus servicios favoritos", "info"),
            sdk::divider(),
            
            // Google
            sdk::card("Google Workspace", vec![
                sdk::text(
                    &if google_token.is_some() { "✅ Conectado" } else { "❌ No conectado" },
                    if google_token.is_some() { "success" } else { "default" }
                ),
                sdk::text("Accede a Gmail, Calendar, Contacts", "default"),
                if google_token.is_some() {
                    sdk::button("Sincronizar Contactos", "sync_google", "secondary")
                } else {
                    sdk::button("Conectar Google", "connect_google", "primary")
                },
            ]),
            
            // Slack
            sdk::card("Slack", vec![
                sdk::text(
                    &if slack_token.is_some() { "✅ Conectado" } else { "❌ No conectado" },
                    if slack_token.is_some() { "success" } else { "default" }
                ),
                sdk::text("Envía notificaciones a canales de Slack", "default"),
                if slack_token.is_some() {
                    sdk::button("Enviar Mensaje", "send_slack", "secondary")
                } else {
                    sdk::button("Conectar Slack", "connect_slack", "primary")
                },
            ]),
            
            // GitHub
            sdk::card("GitHub", vec![
                sdk::text(
                    &if github_token.is_some() { "✅ Conectado" } else { "❌ No conectado" },
                    if github_token.is_some() { "success" } else { "default" }
                ),
                sdk::text("Accede a repositorios, issues, pull requests", "default"),
                if github_token.is_some() {
                    sdk::button("Ver Repositorios", "fetch_github", "secondary")
                } else {
                    sdk::button("Conectar GitHub", "connect_github", "primary")
                },
            ]),
            
            sdk::divider(),
            
            sdk::card("Gestión", vec![
                sdk::text("Desconectar todos los servicios", "default"),
                sdk::button("Desconectar Todo", "disconnect", "danger"),
            ]),
        ]),
    ]);
}

fn render_google_integration() {
    sdk::respond(sdk::widgets![
        sdk::card("Integración con Google", vec![
            sdk::text("Conecta tu cuenta de Google para acceder a:", "info"),
            sdk::text("• Gmail - Enviar/recibir emails", "default"),
            sdk::text("• Calendar - Gestionar eventos", "default"),
            sdk::text("• Contacts - Sincronizar contactos", "default"),
            sdk::divider(),
            sdk::button("Iniciar Conexión", "connect_google", "primary"),
        ]),
    ]);
}

fn render_slack_integration() {
    sdk::respond(sdk::widgets![
        sdk::card("Integración con Slack", vec![
            sdk::text("Conecta tu workspace de Slack para:", "info"),
            sdk::text("• Enviar notificaciones a canales", "default"),
            sdk::text("• Recibir mensajes de soporte", "default"),
            sdk::text("• Sincronizar usuarios", "default"),
            sdk::divider(),
            sdk::button("Iniciar Conexión", "connect_slack", "primary"),
        ]),
    ]);
}

fn render_github_integration() {
    sdk::respond(sdk::widgets![
        sdk::card("Integración con GitHub", vec![
            sdk::text("Conecta tu cuenta de GitHub para:", "info"),
            sdk::text("• Acceder a repositorios", "default"),
            sdk::text("• Gestionar issues y pull requests", "default"),
            sdk::text("• Sincronizar código con tickets", "default"),
            sdk::divider(),
            sdk::button("Iniciar Conexión", "connect_github", "primary"),
        ]),
    ]);
}

fn connect_google() {
    match sdk::oauth_start("google") {
        Some(auth_url) => {
            sdk::kv_set_val("google_auth_url", &auth_url);
            sdk::respond(sdk::widgets![
                sdk::card("Conectando con Google", vec![
                    sdk::text("Redirigiendo a Google para autorización...", "info"),
                    sdk::text("Haz clic en el botón para completar la conexión", "default"),
                    sdk::button("Completar Conexión", "complete_google_auth", "primary"),
                ]),
            ]);
        }
        None => {
            sdk::respond(sdk::widgets![
                sdk::text("Error iniciando conexión con Google", "error")
            ]);
        }
    }
}

fn connect_slack() {
    match sdk::oauth_start("slack") {
        Some(auth_url) => {
            sdk::kv_set_val("slack_auth_url", &auth_url);
            sdk::respond(sdk::widgets![
                sdk::card("Conectando con Slack", vec![
                    sdk::text("Redirigiendo a Slack para autorización...", "info"),
                    sdk::button("Completar Conexión", "complete_slack_auth", "primary"),
                ]),
            ]);
        }
        None => {
            sdk::respond(sdk::widgets![
                sdk::text("Error iniciando conexión con Slack", "error")
            ]);
        }
    }
}

fn connect_github() {
    match sdk::oauth_start("github") {
        Some(auth_url) => {
            sdk::kv_set_val("github_auth_url", &auth_url);
            sdk::respond(sdk::widgets![
                sdk::card("Conectando con GitHub", vec![
                    sdk::text("Redirigiendo a GitHub para autorización...", "info"),
                    sdk::button("Completar Conexión", "complete_github_auth", "primary"),
                ]),
            ]);
        }
        None => {
            sdk::respond(sdk::widgets![
                sdk::text("Error iniciando conexión con GitHub", "error")
            ]);
        }
    }
}

fn sync_google_contacts() {
    sdk::log("Sincronizando contactos de Google...");
    
    // Simular sincronización
    let contacts_count = 42;
    sdk::kv_set_val("google_contacts_count", &contacts_count.to_string());
    sdk::kv_set_val("google_last_sync", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
    
    sdk::respond(sdk::widgets![
        sdk::card("Sincronización Completada", vec![
            sdk::text(&format!("✅ {} contactos sincronizados desde Google", contacts_count), "success"),
            sdk::text(&format!("📅 Última sincronización: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")), "info"),
        ]),
    ]);
}

fn send_slack_message() {
    sdk::log("Enviando mensaje a Slack...");
    
    // Simular envío de mensaje
    let message = "🎮 Nuevo ticket de soporte creado en EzerDesk";
    let channel = "#soporte";
    
    sdk::log(&format!("Mensaje enviado a {}: {}", channel, message));
    sdk::kv_set_val("slack_last_message", &message);
    
    sdk::respond(sdk::widgets![
        sdk::card("Mensaje Enviado", vec![
            sdk::text("✅ Mensaje enviado a Slack exitosamente", "success"),
            sdk::text(&format!("Canal: {}", channel), "info"),
            sdk::text(&format!("Mensaje: {}", message), "default"),
        ]),
    ]);
}

fn fetch_github_repos() {
    sdk::log("Obteniendo repositorios de GitHub...");
    
    // Simular obtención de repos
    let repos = vec![
        ("ezerdesk-backend", "Rust", 150),
        ("ezerdesk-frontend", "Gleam", 89),
        ("ezerdesk-sdk", "Rust", 45),
    ];
    
    sdk::respond(sdk::widgets![
        sdk::card("Repositorios de GitHub", vec![
            sdk::text(&format!("📦 {} repositorios encontrados", repos.len()), "info"),
            sdk::table(
                vec!["Nombre", "Lenguaje", "Stars"],
                repos.iter().map(|(name, lang, stars)| {
                    vec![name, lang, &stars.to_string()]
                }).collect(),
            ),
        ]),
    ]);
}

fn disconnect_service() {
    sdk::kv_set_val("google_token", "");
    sdk::kv_set_val("slack_token", "");
    sdk::kv_set_val("github_token", "");
    
    sdk::respond(sdk::widgets![
        sdk::card("Desconectado", vec![
            sdk::text("✅ Todas las integraciones han sido desconectadas", "success"),
            sdk::text("Puedes reconectar en cualquier momento desde esta página", "info"),
        ]),
    ]);
}
