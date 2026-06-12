// Ejemplo 27: API Integration Hub
// Features: HTTP, OAuth, Webhooks, KV store, Table
// Demuestra: Conexión con servicios externos, webhooks, integraciones

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("api_hub", "API Hub", "plug-line")
                        .category("sistema")
                        .priority(24)
                )
                .name("API Integration Hub")
                .description("Conecta EzerDesk con servicios externos via API")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "api_hub" => render_hub_dashboard(),
                "integrations" => render_integrations_list(),
                "webhooks" => render_webhooks(),
                "api_keys" => render_api_keys(),
                "logs" => render_api_logs(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "add_integration" => render_add_integration(),
                "save_integration" => save_integration(&data),
                "test_integration" => test_integration(&data),
                "delete_integration" => delete_integration(&data),
                "add_webhook" => render_add_webhook(),
                "save_webhook" => save_webhook(&data),
                "test_webhook" => test_webhook(&data),
                "regenerate_api_key" => regenerate_api_key(),
                "export_logs" => export_logs(),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_hub_dashboard() {
    let integration_count = sdk::kv_get_val("integration_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let api_calls = sdk::kv_get_val("api_calls_today")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("API Integration Hub", vec![
            sdk::text("Gestiona todas tus integraciones externas", "info"),
            sdk::divider(),
            sdk::text(&format!("🔗 Integraciones activas: {}", integration_count), "info"),
            sdk::text(&format!("📊 Llamadas API hoy: {}", api_calls), "info"),
        ]),

        sdk::card("Integraciones Rápidas", vec![
            sdk::button("Zapier", "add_integration:zapier", "primary"),
            sdk::button("Make (Integromat)", "add_integration:make", "secondary"),
            sdk::button("n8n", "add_integration:n8n", "secondary"),
            sdk::button("Custom Webhook", "add_webhook", "outline"),
        ]),

        sdk::card("Estado de Integraciones", vec![
            sdk::table(
                vec!["Servicio", "Estado", "Última Sincronización", "Acciones"],
                vec![
                    vec!["Zapier", "✅ Activo", "Hace 5 min", "Configurar"],
                    vec!["Slack", "✅ Activo", "Hace 10 min", "Configurar"],
                    vec!["GitHub", "❌ Error", "Hace 1 hora", "Reintentar"],
                ],
            ),
        ]),

        sdk::card("Gráficos de Uso", vec![
            sdk::chart("Llamadas API por Día", vec![
                ("Lun", 150.0), ("Mar", 220.0), ("Mié", 180.0),
                ("Jue", 250.0), ("Vie", 200.0),
            ], "bar"),
        ]),
    ]);
}

fn render_integrations_list() {
    sdk::respond(sdk::widgets![
        sdk::card("Integraciones Configuradas", vec![
            sdk::table(
                vec!["Servicio", "Tipo", "Estado", "Última Sync", "Acciones"],
                vec![
                    vec!["Zapier", "Webhook", "✅ Activo", "Hace 5 min", "Editar | Eliminar"],
                    vec!["Slack", "API", "✅ Activo", "Hace 10 min", "Editar | Eliminar"],
                    vec!["GitHub", "OAuth", "❌ Error", "Hace 1 hora", "Reintentar | Eliminar"],
                    vec!["Google Sheets", "API", "✅ Activo", "Ayer", "Editar | Eliminar"],
                ],
            ),
            sdk::button("Agregar Integración", "add_integration", "primary"),
        ]),
    ]);
}

fn render_webhooks() {
    sdk::respond(sdk::widgets![
        sdk::card("Webhooks Configurados", vec![
            sdk::text("Endpoints para recibir datos de servicios externos", "info"),
            sdk::table(
                vec!["Nombre", "URL", "Eventos", "Estado", "Acciones"],
                vec![
                    vec!["Zapier Trigger", "https://api.ezerdesk.com/webhooks/zapier", "ticket.*", "✅", "Editar"],
                    vec!["n8n Workflow", "https://api.ezerdesk.com/webhooks/n8n", "all", "✅", "Editar"],
                ],
            ),
            sdk::button("Agregar Webhook", "add_webhook", "primary"),
        ]),
    ]);
}

fn render_api_keys() {
    sdk::respond(sdk::widgets![
        sdk::card("API Keys", vec![
            sdk::text("Gestiona las claves de acceso a la API", "info"),
            sdk::table(
                vec!["Nombre", "Clave", "Permisos", "Último Uso", "Acciones"],
                vec![
                    vec!["Production", "sk_live_****abc123", "read, write", "Hace 1 min", "Regenerar"],
                    vec!["Development", "sk_test_****def456", "read", "Hace 1 hora", "Eliminar"],
                ],
            ),
            sdk::button("Generar Nueva Clave", "regenerate_api_key", "primary"),
        ]),
    ]);
}

fn render_api_logs() {
    sdk::respond(sdk::widgets![
        sdk::card("Logs de API", vec![
            sdk::table(
                vec!["Timestamp", "Endpoint", "Método", "Status", "Tiempo"],
                vec![
                    vec!["10:30:15", "/api/v1/tickets", "GET", "200", "45ms"],
                    vec!["10:30:10", "/api/v1/agents", "GET", "200", "32ms"],
                    vec!["10:29:55", "/webhooks/zapier", "POST", "200", "120ms"],
                    vec!["10:29:30", "/api/v1/tickets", "POST", "201", "89ms"],
                ],
            ),
            sdk::button("Exportar Logs", "export_logs", "outline"),
        ]),
    ]);
}

fn render_add_integration() {
    sdk::respond(sdk::widgets![
        sdk::card("Agregar Integración", vec![
            sdk::select_widget("Servicio", "service", vec![
                ("zapier".to_string(), "Zapier".to_string()),
                ("make".to_string(), "Make (Integromat)".to_string()),
                ("n8n".to_string(), "n8n".to_string()),
                ("slack".to_string(), "Slack".to_string()),
                ("github".to_string(), "GitHub".to_string()),
                ("google".to_string(), "Google Sheets".to_string()),
                ("custom".to_string(), "Custom API".to_string()),
            ], "zapier".to_string()),
            sdk::input("Nombre", "integration_name", "Mi Integración"),
            sdk::input("API Key / Token", "api_key", "Tu API key"),
            sdk::input("Webhook URL (opcional)", "webhook_url", ""),
            sdk::textarea("Configuración Adicional", "config", "{}"),
            sdk::button("Guardar", "save_integration", "primary"),
            sdk::button("Cancelar", "integrations", "outline"),
        ]),
    ]);
}

fn render_add_webhook() {
    sdk::respond(sdk::widgets![
        sdk::card("Agregar Webhook", vec![
            sdk::input("Nombre", "webhook_name", "Mi Webhook"),
            sdk::input("URL de Destino", "webhook_url", "https://api.ejemplo.com/webhook"),
            sdk::select_widget("Eventos", "events", vec![
                ("all".to_string(), "Todos los eventos".to_string()),
                ("ticket".to_string(), "Eventos de ticket".to_string()),
                ("chat".to_string(), "Eventos de chat".to_string()),
            ], "all".to_string()),
            sdk::input("Secret (HMAC)", "secret", "Tu secreto para firmar"),
            sdk::switch_widget("Activo", "active", true),
            sdk::button("Guardar", "save_webhook", "primary"),
            sdk::button("Cancelar", "webhooks", "outline"),
        ]),
    ]);
}

fn save_integration(data: &str) {
    sdk::log(&format!("Guardando integración: {}", data));
    
    let count = sdk::kv_get_val("integration_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("integration_count", &count.to_string());
    
    sdk::respond_ok("Integración guardada exitosamente");
}

fn test_integration(data: &str) {
    sdk::log(&format!("Probando integración: {}", data));
    sdk::respond_ok("Conexión exitosa con el servicio");
}

fn delete_integration(data: &str) {
    sdk::log(&format!("Eliminando integración: {}", data));
    sdk::respond_ok("Integración eliminada");
}

fn save_webhook(data: &str) {
    sdk::log(&format!("Guardando webhook: {}", data));
    sdk::respond_ok("Webhook guardado");
}

fn test_webhook(data: &str) {
    sdk::log &format!("Probando webhook: {}", data));
    sdk::respond_ok("Webhook probado exitosamente");
}

fn regenerate_api_key() {
    sdk::log("Regenerando API key...");
    sdk::respond_ok("Nueva API key generada");
}

fn export_logs() {
    sdk::log("Exportando logs de API...");
    sdk::respond_ok("Logs exportados");
}
