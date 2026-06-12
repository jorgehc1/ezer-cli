// Ejemplo 28: Integración Phone/SMS
// Features: SMS sending, HTTP, KV store, Events, Table
// Demuestra: Notificaciones SMS, creación de tickets desde SMS, comandos SMS

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("phone_sms", "Phone/SMS", "phone-line")
                        .category("sistema")
                        .priority(29)
                )
                .name("Integración Phone/SMS")
                .description("Envía notificaciones SMS y recibe mensajes para crear tickets")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "phone_sms" => render_sms_dashboard(),
                "send" => render_send_sms_form(),
                "templates" => render_sms_templates(),
                "history" => render_sms_history(),
                "commands" => render_sms_commands(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "send_sms" => send_sms_message(&data),
                "send_bulk" => send_bulk_sms(&data),
                "add_template" => render_add_template_form(),
                "save_template" => save_template(&data),
                "test_connection" => test_sms_connection(),
                "view_history" => render_sms_history(),
                _ => {}
            }
        }

        // Enviar SMS cuando se crea un ticket crítico
        PluginEvent::TicketCreated(ticket) => {
            if ticket.prioridad == "Alta" || ticket.prioridad == "Urgente" {
                notify_agent_sms(&ticket);
            }
        }

        _ => {}
    }
    0
}

fn render_sms_dashboard() {
    let sent_count = sdk::kv_get_val("sms_sent_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let received_count = sdk::kv_get_val("sms_received_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Integración Phone/SMS", vec![
            sdk::text("Gestiona notificaciones SMS y recibidos", "info"),
            sdk::divider(),
            sdk::text(&format!("📤 SMS enviados: {}", sent_count), "info"),
            sdk::text(&format!("📥 SMS recibidos: {}", received_count), "info"),
            sdk::text(&format!("🔗 Estado: {}", 
                if sdk::kv_get_val("sms_configured").unwrap_or("false".to_string()) == "true" 
                { "✅ Conectado" } else { "❌ No configurado" }
            ), "info"),
        ]),

        sdk::card("Envío Rápido", vec![
            sdk::input("Número destino", "to_number", "+593 99 123 4567"),
            sdk::textarea("Mensaje", "message", "Escribe tu mensaje SMS..."),
            sdk::button("Enviar SMS", "send_sms", "primary"),
        ]),

        sdk::card("Acciones", vec![
            sdk::button("Envío Masivo", "send_bulk", "secondary"),
            sdk::button("Plantillas", "templates", "outline"),
            sdk::button("Historial", "history", "outline"),
            sdk::button("Probar Conexión", "test_connection", "outline"),
        ]),

        sdk::card("Estadísticas", vec![
            sdk::chart("SMS por Día", vec![
                ("Lun", 25.0), ("Mar", 32.0), ("Mié", 28.0),
                ("Jue", 35.0), ("Vie", 30.0),
            ], "bar"),
        ]),
    ]);
}

fn render_send_sms_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Enviar SMS", vec![
            sdk::input("Número destino", "to_number", "+593 99 123 4567"),
            sdk::textarea("Mensaje", "message", "Escribe tu mensaje SMS (máx 160 caracteres)..."),
            sdk::number_input_with_limits("Caracteres", "char_count", "0", "0", 0.0, 160.0, 1.0),
            sdk::button("Enviar SMS", "send_sms", "primary"),
            sdk::button("Cancelar", "phone_sms", "outline"),
        ]),
    ]);
}

fn render_sms_templates() {
    sdk::respond(sdk::widgets![
        sdk::card("Plantillas SMS", vec![
            sdk::text("Plantillas para mensajes SMS predefinidos", "info"),
            sdk::table(
                vec!["Nombre", "Mensaje", "Uso"],
                vec![
                    vec!["Ticket Creado", "Nuevo ticket #{id}: {asunto}", "Notificar agente"],
                    vec!["Ticket Actualizado", "Ticket #{id} actualizado a {estado}", "Notificar cliente"],
                    vec!["Recordatorio", "Recordatorio: ticket #{id} pendiente", "Seguimiento"],
                    vec!["SLA Vencido", "⚠️ SLA vencido en ticket #{id}", "Alerta crítica"],
                ],
            ),
            sdk::button("Agregar Plantilla", "add_template", "primary"),
        ]),
    ]);
}

fn render_sms_history() {
    sdk::respond(sdk::widgets![
        sdk::card("Historial de SMS", vec![
            sdk::table(
                vec!["Timestamp", "Dirección", "Tipo", "Mensaje", "Estado"],
                vec![
                    vec!["10:30", "+593 99 123 4567", "Enviado", "Ticket #123 creado", "✅ Entregado"],
                    vec!["10:15", "+593 99 765 4321", "Recibido", "Consultar estado", "Procesado"],
                    vec!["09:00", "+593 99 111 222", "Enviado", "Recordatorio SLA", "✅ Entregado"],
                ],
            ),
        ]),
    ]);
}

fn render_sms_commands() {
    sdk::respond(sdk::widgets![
        sdk::card("Comandos SMS Disponibles", vec![
            sdk::text("Los usuarios pueden enviar estos comandos por SMS:", "info"),
            sdk::table(
                vec!["Comando", "Descripción", "Ejemplo"],
                vec![
                    vec!["/start", "Iniciar servicio", "/start"],
                    vec!["/ticket <id>", "Consultar ticket", "/ticket 123"],
                    vec!["/cerrar <id>", "Cerrar ticket", "/cerrar 123"],
                    vec!["/estatus", "Ver estado", "/estatus"],
                    vec!["/ayuda", "Ver comandos", "/ayuda"],
                ],
            ),
        ]),
    ]);
}

fn render_add_template_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Nueva Plantilla SMS", vec![
            sdk::input("Nombre", "template_name", "Mi Plantilla"),
            sdk::textarea("Mensaje", "message", "Usa {variable} para datos dinámicos"),
            sdk::input("Variables disponibles", "variables", "{id}, {asunto}, {estado}, {cliente}"),
            sdk::button("Guardar", "save_template", "primary"),
            sdk::button("Cancelar", "templates", "outline"),
        ]),
    ]);
}

fn send_sms_message(data: &str) {
    sdk::log(&format!("Enviando SMS: {}", data));
    
    let count = sdk::kv_get_val("sms_sent_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("sms_sent_count", &count.to_string());
    
    sdk::respond_ok("SMS enviado exitosamente");
}

fn send_bulk_sms(data: &str) {
    sdk::log(&format!("Enviando SMS masivo: {}", data));
    sdk::respond_ok("SMS masivo encolado");
}

fn save_template(data: &str) {
    sdk::log(&format!("Guardando plantilla SMS: {}", data));
    sdk::respond_ok("Plantilla guardada");
}

fn test_sms_connection() {
    sdk::log("Probando conexión SMS...");
    sdk::kv_set_val("sms_configured", "true");
    sdk::respond_ok("Conexión SMS exitosa");
}

fn notify_agent_sms(ticket: &sdk::Ticket) {
    let msg = format!(
        "🎫 Ticket #{}: {} (Prioridad: {})",
        ticket.id, ticket.asunto, ticket.prioridad
    );
    sdk::log(&format!("Enviando SMS de notificación: {}", msg));
    // En producción: enviar via send_sms()
}
