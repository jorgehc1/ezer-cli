// Ejemplo 18: Gestor de Notificaciones
// Features: KV store, Templates, Events, Multi-canal
// Demuestra: Sistema de notificaciones por email, push, SMS y webhook

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("notificaciones", "Notificaciones", "notification-3-line")
                        .category("sistema")
                        .priority(17)
                )
                .name("Gestor de Notificaciones")
                .description("Configura y gestiona notificaciones por multiples canales")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "notificaciones" => render_notifications_dashboard(),
                "create" => render_create_notification(),
                "channels" => render_channel_config(),
                "history" => render_notification_history(),
                "rules" => render_notification_rules(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "create_notification" => handle_create_notification(&data),
                "send_test" => send_test_notification(&data),
                "toggle_channel" => toggle_channel(&data),
                "add_rule" => add_notification_rule(&data),
                "delete_rule" => delete_notification_rule(&data),
                "mark_read" => mark_notification_read(&data),
                _ => {
                    sdk::respond_ok("Accion no reconocida");
                }
            }
        }

        PluginEvent::TicketCreated(ticket) => {
            send_notification("new_ticket", &format!("Nuevo ticket: {}", ticket.asunto));
        }

        PluginEvent::TicketStatusChanged(ticket) => {
            send_notification("status_changed", &format!("Ticket {} cambio a {}", ticket.id, ticket.new_status));
        }

        _ => {}
    }
    0
}

fn render_notifications_dashboard() {
    let total_sent = sdk::kv_get_val("notifications_sent")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let unread = sdk::kv_get_val("unread_notifications")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Gestor de Notificaciones", vec![
            sdk::text("Administra las notificaciones del sistema", "info"),
            sdk::divider(),

            sdk::card("Resumen", vec![
                sdk::text(&format!("Notificaciones enviadas: {}", total_sent), "info"),
                sdk::text(&format!("Sin leer: {}", unread), if unread > 0 { "warning" } else { "success" }),
                sdk::chart("Notificaciones por Canal", vec![
                    ("Email", 45.0),
                    ("Push", 30.0),
                    ("SMS", 10.0),
                    ("Webhook", 15.0),
                ], "pie"),
            ]),

            sdk::card("Notificaciones Recientes", vec![
                sdk::table(
                    vec!["Tipo", "Mensaje", "Canal", "Estado", "Fecha"],
                    vec![
                        vec!["Ticket", "Nuevo ticket #1234", "Email", "Leido", "15:30"],
                        vec!["SLA", "Ticket #1230 viola SLA", "Push", "No leido", "15:25"],
                        vec!["Chat", "Nuevo mensaje de chat", "Push", "No leido", "15:20"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::card("Acciones", vec![
                sdk::button("Crear Notificacion", "create", "primary"),
                sdk::button("Configurar Canales", "channels", "secondary"),
                sdk::button("Ver Historial", "history", "secondary"),
                sdk::button("Gestionar Reglas", "rules", "secondary"),
            ]),
        ]),
    ]);
}

fn render_create_notification() {
    sdk::respond(sdk::widgets![
        sdk::card("Crear Notificacion", vec![
            sdk::text("Envia una notificacion personalizada", "info"),
            sdk::divider(),

            sdk::input("notification_title", "Titulo")
                .placeholder("Titulo de la notificacion")
                .required(true),

            sdk::textarea("notification_message", "Mensaje")
                .placeholder("Escribe el mensaje...")
                .rows(5)
                .required(true),

            sdk::select("notification_type", "Tipo", vec![
                ("info", "Informativa"),
                ("warning", "Advertencia"),
                ("error", "Error"),
                ("success", "Exito"),
            ]),

            sdk::select("notification_channel", "Canal", vec![
                ("email", "Email"),
                ("push", "Push"),
                ("sms", "SMS"),
                ("webhook", "Webhook"),
                ("all", "Todos los canales"),
            ]),

            sdk::input("recipients", "Destinatarios")
                .placeholder("email@ejemplo.com (separados por coma)"),

            sdk::checkbox("send_now", "Enviar inmediatamente"),

            sdk::divider(),

            sdk::button("Enviar Notificacion", "create_notification", "primary"),
            sdk::button("Enviar Prueba", "send_test", "secondary"),
            sdk::button("Cancelar", "notificaciones", "outline"),
        ]),
    ]);
}

fn render_channel_config() {
    let email_enabled = sdk::kv_get_val("channel_email")
        .unwrap_or("true".to_string()) == "true";
    let push_enabled = sdk::kv_get_val("channel_push")
        .unwrap_or("true".to_string()) == "true";
    let sms_enabled = sdk::kv_get_val("channel_sms")
        .unwrap_or("false".to_string()) == "true";
    let webhook_enabled = sdk::kv_get_val("channel_webhook")
        .unwrap_or("true".to_string()) == "true";

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion de Canales", vec![
            sdk::text("Activa o desactiva los canales de notificacion", "info"),
            sdk::divider(),

            sdk::card("Canales Disponibles", vec![
                sdk::table(
                    vec!["Canal", "Estado", "Configuracion"],
                    vec![
                        vec!["Email", if email_enabled { "Activo" } else { "Inactivo" }, "SMTP configurado"],
                        vec!["Push", if push_enabled { "Activo" } else { "Inactivo" }, "Firebase configurado"],
                        vec!["SMS", if sms_enabled { "Activo" } else { "Inactivo" }, "Twilio configurado"],
                        vec!["Webhook", if webhook_enabled { "Activo" } else { "Inactivo" }, "URL configurada"],
                    ],
                ),
            ]),

            sdk::card("Configuracion Email", vec![
                sdk::input("email_to", "Email de Destino")
                    .placeholder("admin@ejemplo.com"),
                sdk::checkbox("email_enabled", "Email Habilitado"),
            ]),

            sdk::card("Configuracion Push", vec![
                sdk::input("push_key", "Firebase Server Key")
                    .placeholder("Tu server key de Firebase"),
                sdk::checkbox("push_enabled", "Push Habilitado"),
            ]),

            sdk::card("Configuracion Webhook", vec![
                sdk::input("webhook_url", "URL del Webhook")
                    .placeholder("https://api.ejemplo.com/webhook"),
                sdk::input("webhook_secret", "Secret")
                    .placeholder("Tu secreto de verificacion"),
                sdk::checkbox("webhook_enabled", "Webhook Habilitado"),
            ]),

            sdk::divider(),

            sdk::button("Guardar Cambios", "notificaciones", "primary"),
        ]),
    ]);
}

fn render_notification_history() {
    sdk::respond(sdk::widgets![
        sdk::card("Historial de Notificaciones", vec![
            sdk::text("Registro de todas las notificaciones enviadas", "info"),
            sdk::divider(),

            sdk::table(
                vec!["Fecha", "Tipo", "Canal", "Destinatario", "Estado"],
                vec![
                    vec!["2024-01-15 15:30", "Ticket", "Email", "maria@ejemplo.com", "Entregado"],
                    vec!["2024-01-15 15:25", "SLA", "Push", "carlos@ejemplo.com", "Entregado"],
                    vec!["2024-01-15 15:20", "Chat", "Push", "ana@ejemplo.com", "Pendiente"],
                    vec!["2024-01-15 15:15", "Sistema", "Email", "admin@ejemplo.com", "Entregado"],
                ],
            ),

            sdk::divider(),
            sdk::button("Volver", "notificaciones", "outline"),
        ]),
    ]);
}

fn render_notification_rules() {
    sdk::respond(sdk::widgets![
        sdk::card("Reglas de Notificacion", vec![
            sdk::text("Define cuando y como se envian las notificaciones", "info"),
            sdk::divider(),

            sdk::table(
                vec!["Evento", "Canal", "Destinatario", "Estado"],
                vec![
                    vec!["Ticket Creado", "Email", "Equipo Soporte", "Activo"],
                    vec!["SLA Violado", "Push + Email", "Supervisor", "Activo"],
                    vec!["Ticket Cerrado", "Email", "Cliente", "Activo"],
                    vec!["Chat Esperando", "Push", "Agentes Disponibles", "Inactivo"],
                ],
            ),

            sdk::divider(),

            sdk::card("Nueva Regla", vec![
                sdk::select("rule_event", "Evento", vec![
                    ("ticket_created", "Ticket Creado"),
                    ("status_changed", "Cambio de Estado"),
                    ("sla_breach", "Violacion de SLA"),
                    ("chat_waiting", "Chat en Espera"),
                    ("assignment", "Asignacion de Ticket"),
                ]),

                sdk::select("rule_channel", "Canal", vec![
                    ("email", "Email"),
                    ("push", "Push"),
                    ("sms", "SMS"),
                    ("webhook", "Webhook"),
                ]),

                sdk::input("rule_recipients", "Destinatarios")
                    .placeholder("email@ejemplo.com"),

                sdk::button("Agregar Regla", "add_rule", "primary"),
            ]),

            sdk::button("Volver", "notificaciones", "outline"),
        ]),
    ]);
}

fn handle_create_notification(data: &str) {
    let title = extract_field(data, "notification_title").unwrap_or_default();
    let message = extract_field(data, "notification_message").unwrap_or_default();
    let channel = extract_field(data, "notification_channel").unwrap_or_default();
    let recipients = extract_field(data, "recipients").unwrap_or_default();

    if title.is_empty() || message.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("El titulo y mensaje son obligatorios", "error"),
        ]);
        return;
    }

    let notif_id = format!("notif_{}", chrono::Utc::now().timestamp());
    sdk::kv_set_val(&notif_id, &format!("title:{}|msg:{}|chan:{}", title, message, channel));

    let count = sdk::kv_get_val("notifications_sent")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("notifications_sent", &count.to_string());

    sdk::log(&format!("Notificacion creada: {} via {}", title, channel));

    sdk::respond(sdk::widgets![
        sdk::card("Notificacion Enviada", vec![
            sdk::text("Notificacion enviada exitosamente", "success"),
            sdk::text(&format!("Titulo: {}", title), "info"),
            sdk::text(&format!("Canal: {}", channel), "info"),
            sdk::text(&format!("Destinatarios: {}", recipients), "info"),
            sdk::button("Volver", "notificaciones", "primary"),
        ]),
    ]);
}

fn send_test_notification(data: &str) {
    let channel = extract_field(data, "notification_channel")
        .unwrap_or("email".to_string());
    sdk::log(&format!("Enviando notificacion de prueba por: {}", channel));

    sdk::respond(sdk::widgets![
        sdk::card("Notificacion de Prueba", vec![
            sdk::text("Notificacion de prueba enviada", "success"),
            sdk::text(&format!("Canal: {}", channel), "info"),
            sdk::button("Volver", "create", "outline"),
        ]),
    ]);
}

fn toggle_channel(data: &str) {
    let channel = extract_field(data, "channel").unwrap_or_default();
    sdk::log(&format!("Canal {} toggled", channel));

    sdk::respond(sdk::widgets![
        sdk::text("Canal actualizado", "success"),
        sdk::button("Volver", "channels", "outline"),
    ]);
}

fn add_notification_rule(data: &str) {
    let event = extract_field(data, "rule_event").unwrap_or_default();
    let channel = extract_field(data, "rule_channel").unwrap_or_default();
    let recipients = extract_field(data, "rule_recipients").unwrap_or_default();

    let rule_id = format!("rule_{}", chrono::Utc::now().timestamp());
    sdk::kv_set_val(&rule_id, &format!("event:{}|chan:{}|recipients:{}", event, channel, recipients));

    sdk::log(&format!("Regla creada: {} -> {}", event, channel));

    sdk::respond(sdk::widgets![
        sdk::card("Regla Creada", vec![
            sdk::text("Regla de notificacion creada", "success"),
            sdk::button("Volver", "rules", "primary"),
        ]),
    ]);
}

fn delete_notification_rule(data: &str) {
    let rule_id = extract_field(data, "rule_id").unwrap_or_default();
    if !rule_id.is_empty() {
        sdk::kv_set_val(&rule_id, "");
    }

    sdk::respond(sdk::widgets![
        sdk::text("Regla eliminada", "success"),
        sdk::button("Volver", "rules", "outline"),
    ]);
}

fn mark_notification_read(data: &str) {
    let notif_id = extract_field(data, "notif_id").unwrap_or_default();
    sdk::log(&format!("Notificacion marcada como leida: {}", notif_id));

    sdk::respond(sdk::widgets![
        sdk::text("Notificacion marcada como leida", "success"),
    ]);
}

fn send_notification(event: &str, message: &str) {
    sdk::log(&format!("Evento {} - {}", event, message));

    let count = sdk::kv_get_val("notifications_sent")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("notifications_sent", &count.to_string());
}

fn extract_field(data: &str, field: &str) -> Option<String> {
    let search = format!("\"{}\":\"", field);
    if let Some(pos) = data.find(&search) {
        let start = pos + search.len();
        if let Some(end) = data[start..].find('"') {
            return Some(data[start..start + end].to_string());
        }
    }
    None
}
