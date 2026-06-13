// Ejemplo 30: Integración Phone/Call
// Features: Phone calls, HTTP, KV store, Events, Table
// Demuestra: Llamadas salientes/entrantes, logging de llamadas, integración telefónica

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("phone_call", "Phone/Call", "phone-line")
                        .category("sistema")
                        .priority(30)
                )
                .name("Integración Phone/Call")
                .description("Gestiona llamadas telefónicas entrantes y salientes")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "phone_call" => render_phone_dashboard(),
                "make_call" => render_make_call_form(),
                "call_log" => render_call_log(),
                "settings" => render_phone_settings(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "make_call" => make_phone_call(&data),
                "end_call" => end_phone_call(&data),
                "transfer_call" => transfer_call(&data),
                "test_connection" => test_phone_connection(),
                "save_settings" => save_phone_settings(&data),
                _ => {}
            }
        }

        // Notificar cuando se crea un ticket desde una llamada
        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Ticket creado desde llamada: {}", ticket.id));
        }

        _ => {}
    }
    0
}

fn render_phone_dashboard() {
    let call_count = sdk::kv_get_val("calls_today")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let total_duration = sdk::kv_get_val("total_duration")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Integración Phone/Call", vec![
            sdk::text("Gestiona llamadas telefónicas del helpdesk", "info"),
            sdk::divider(),
            sdk::text(&format!("📞 Llamadas hoy: {}", call_count), "info"),
            sdk::text(&format!("⏱️ Duración total: {} min", total_duration / 60), "info"),
            sdk::text(&format!("🔗 Estado: {}", 
                if sdk::kv_get_val("phone_configured").unwrap_or("false".to_string()) == "true" 
                { "✅ Conectado" } else { "❌ No configurado" }
            ), "info"),
        ]),

        sdk::card("Acciones Rápidas", vec![
            sdk::button("Hacer Llamada", "make_call", "primary"),
            sdk::button("Registro de Llamadas", "call_log", "secondary"),
            sdk::button("Configuración", "settings", "outline"),
            sdk::button("Probar Conexión", "test_connection", "outline"),
        ]),

        sdk::card("Últimas Llamadas", vec![
            sdk::table(
                vec!["Hora", "Número", "Tipo", "Duración", "Estado"],
                vec![
                    vec!["10:30", "+593 99 123 4567", "Entrante", "5:32", "Completada"],
                    vec!["10:15", "+593 99 765 4321", "Saliente", "2:15", "Completada"],
                    vec!["09:45", "+593 99 111 222", "Entrante", "0:45", "Perdida"],
                ],
            ),
        ]),
    ]);
}

fn render_make_call_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Hacer Llamada", vec![
            sdk::input("Número destino", "to_number", "+593 99 123 4567"),
            sdk::input("URL de respuesta", "response_url", "https://api.ezerdesk.com/webhooks/phone"),
            sdk::textarea("Notas", "notes", "Notas sobre la llamada..."),
            sdk::button("Iniciar Llamada", "make_call", "primary"),
            sdk::button("Cancelar", "phone_call", "outline"),
        ]),
    ]);
}

fn render_call_log() {
    sdk::respond(sdk::widgets![
        sdk::card("Registro de Llamadas", vec![
            sdk::table(
                vec!["ID", "Hora", "Número", "Tipo", "Duración", "Estado", "Agente"],
                vec![
                    vec!["C-001", "10:30", "+593 99 123 4567", "Entrante", "5:32", "Completada", "Agente 1"],
                    vec!["C-002", "10:15", "+593 99 765 4321", "Saliente", "2:15", "Completada", "Agente 2"],
                    vec!["C-003", "09:45", "+593 99 111 222", "Entrante", "0:45", "Perdida", "N/A"],
                ],
            ),
            sdk::button("Exportar", "export_calls", "outline"),
        ]),
    ]);
}

fn render_phone_settings() {
    sdk::respond(sdk::widgets![
        sdk::card("Configuración Phone/Call", vec![
            sdk::input("Account SID", "account_sid", "AC1234567890"),
            sdk::input("Auth Token", "auth_token", "tu_auth_token"),
            sdk::input("Número origen", "from_number", "+1234567890"),
            sdk::input("URL webhook llamadas", "webhook_url", "https://api.ezerdesk.com/webhooks/phone"),
            sdk::switch_widget("Auto-crear tickets", "auto_create_tickets", true),
            sdk::switch_widget("Grabar llamadas", "record_calls", false),
            sdk::button("Guardar Configuración", "save_settings", "primary"),
        ]),
    ]);
}

fn make_phone_call(data: &str) {
    sdk::log(&format!("Iniciando llamada: {}", data));
    
    let count = sdk::kv_get_val("calls_today")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("calls_today", &count.to_string());
    
    sdk::respond_ok("Llamada iniciada exitosamente");
}

fn end_phone_call(data: &str) {
    sdk::log(&format!("Finalizando llamada: {}", data));
    sdk::respond_ok("Llamada finalizada");
}

fn transfer_call(data: &str) {
    sdk::log(&format!("Transfiriendo llamada: {}", data));
    sdk::respond_ok("Llamada transferida");
}

fn test_phone_connection() {
    sdk::log("Probando conexión telefónica...");
    sdk::kv_set_val("phone_configured", "true");
    sdk::respond_ok("Conexión telefónica exitosa");
}

fn save_phone_settings(data: &str) {
    sdk::log(&format!("Guardando configuración: {}", data));
    sdk::respond_ok("Configuración guardada");
}
