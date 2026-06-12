// Ejemplo 24: Notificador de Telegram
// Features: HTTP, KV store, Events, Charts, Cron
// Demuestra: Sistema de notificaciones automáticas via Telegram

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("telegram_notif", "Notif. Telegram", "notification-3-line")
                        .category("sistema")
                        .priority(28)
                )
                .name("Notificador de Telegram")
                .description("Envía notificaciones automáticas del sistema a Telegram")
                .version("1.0.0")
                .cron("3600"); // Cada hora
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "telegram_notif" => render_notification_dashboard(),
                "templates" => render_templates(),
                "history" => render_notification_history(),
                "rules" => render_notification_rules(),
                _ => {}
            }
        }

        PluginEvent::CronTick => {
            // Verificar y enviar notificaciones pendientes
            check_pending_notifications();
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "send_test" => send_test_notification(),
                "add_template" => render_add_template_form(),
                "save_template" => save_template(&data),
                "add_rule" => render_add_rule_form(),
                "save_rule" => save_rule(&data),
                "clear_history" => clear_notification_history(),
                "export_history" => export_notification_history(),
                _ => {}
            }
        }

        // Enviar notificaciones basadas en eventos
        PluginEvent::TicketCreated(ticket) => {
            notify_ticket_created(&ticket);
        }

        PluginEvent::SlaBreachDetected(breach) => {
            notify_sla_breach(&breach);
        }

        _ => {}
    }
    0
}

fn render_notification_dashboard() {
    let sent_count = sdk::kv_get_val("notifications_sent")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let failed_count = sdk::kv_get_val("notifications_failed")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Notificador de Telegram", vec![
            sdk::text("Sistema de notificaciones automáticas", "info"),
            sdk::divider(),
            sdk::text(&format!("✅ Enviadas: {}", sent_count), "success"),
            sdk::text(&format!("❌ Fallidas: {}", failed_count), if failed_count > 0 { "warning" } else { "info" }),
        ]),

        sdk::card("Envío Rápido", vec![
            sdk::input("Chat ID", "chat_id", "123456789"),
            sdk::textarea("Mensaje", "message", "Escribe tu mensaje..."),
            sdk::select_widget("Prioridad", "priority", vec![
                ("low".to_string(), "Baja".to_string()),
                ("medium".to_string(), "Media".to_string()),
                ("high".to_string(), "Alta".to_string()),
                ("critical".to_string(), "Crítica".to_string()),
            ], "medium".to_string()),
            sdk::button("Enviar", "send_notification", "primary"),
        ]),

        sdk::card("Estadísticas", vec![
            sdk::chart("Notificaciones por Día", vec![
                ("Lun", 15.0), ("Mar", 22.0), ("Mié", 18.0),
                ("Jue", 25.0), ("Vie", 20.0),
            ], "bar"),
        ]),

        sdk::card("Acciones", vec![
            sdk::button("Gestionar Plantillas", "templates", "secondary"),
            sdk::button("Ver Historial", "history", "outline"),
            sdk::button("Configurar Reglas", "rules", "outline"),
        ]),
    ]);
}

fn render_templates() {
    sdk::respond(sdk::widgets![
        sdk::card("Plantillas de Notificación", vec![
            sdk::text("Plantillas para diferentes tipos de notificaciones", "info"),
            sdk::table(
                vec!["Nombre", "Tipo", "Preview", "Activa"],
                vec![
                    vec!["Ticket Creado", "ticket", "🎫 Nuevo ticket: {titulo}", "Sí"],
                    vec!["SLA Violado", "alerta", "⚠️ SLA violado en ticket #{id}", "Sí"],
                    vec!["Reporte Diario", "reporte", "📊 Reporte diario del sistema", "Sí"],
                    vec!["Backup Completado", "sistema", "✅ Backup completado exitosamente", "No"],
                ],
            ),
            sdk::button("Agregar Plantilla", "add_template", "primary"),
        ]),
    ]);
}

fn render_notification_history() {
    sdk::respond(sdk::widgets![
        sdk::card("Historial de Notificaciones", vec![
            sdk::table(
                vec!["Timestamp", "Tipo", "Destino", "Mensaje", "Estado"],
                vec![
                    vec!["10:30", "ticket", "Soporte General", "Nuevo ticket #123", "✅ Enviado"],
                    vec!["10:15", "alerta", "Admin", "SLA violado", "✅ Enviado"],
                    vec!["09:00", "reporte", "Soporte General", "Reporte diario", "✅ Enviado"],
                ],
            ),
            sdk::button("Limpiar Historial", "clear_history", "danger"),
            sdk::button("Exportar", "export_history", "outline"),
        ]),
    ]);
}

fn render_notification_rules() {
    sdk::respond(sdk::widgets![
        sdk::card("Reglas de Notificación", vec![
            sdk::text("Define cuándo enviar notificaciones", "info"),
            sdk::table(
                vec!["Evento", "Canal", "Filtro", "Activa"],
                vec![
                    vec!["Ticket Creado", "Soporte General", "Prioridad Alta", "Sí"],
                    vec!["SLA Violado", "Admin", "Siempre", "Sí"],
                    vec!["Ticket Cerrado", "Soporte General", "Días pares", "No"],
                ],
            ),
            sdk::button("Agregar Regla", "add_rule", "primary"),
        ]),
    ]);
}

fn render_add_template_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Nueva Plantilla", vec![
            sdk::input("Nombre", "template_name", "Mi Plantilla"),
            sdk::select_widget("Tipo", "template_type", vec![
                ("ticket".to_string(), "Ticket".to_string()),
                ("alerta".to_string(), "Alerta".to_string()),
                ("reporte".to_string(), "Reporte".to_string()),
                ("sistema".to_string(), "Sistema".to_string()),
            ], "ticket".to_string()),
            sdk::textarea("Contenido", "content", "Usa {variable} para datos dinámicos..."),
            sdk::button("Guardar", "save_template", "primary"),
            sdk::button("Cancelar", "templates", "outline"),
        ]),
    ]);
}

fn render_add_rule_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Nueva Regla", vec![
            sdk::input("Nombre", "rule_name", "Mi Regla"),
            sdk::select_widget("Evento", "event", vec![
                ("ticket_created".to_string(), "Ticket Creado".to_string()),
                ("ticket_closed".to_string(), "Ticket Cerrado".to_string()),
                ("sla_breach".to_string(), "SLA Violado".to_string()),
                ("daily_report".to_string(), "Reporte Diario".to_string()),
            ], "ticket_created".to_string()),
            sdk::input("Chat ID Destino", "chat_id", "123456789"),
            sdk::select_widget("Plantilla", "template", vec![
                ("ticket_created".to_string(), "Ticket Creado".to_string()),
                ("sla_breach".to_string(), "SLA Violado".to_string()),
            ], "ticket_created".to_string()),
            sdk::button("Guardar", "save_rule", "primary"),
            sdk::button("Cancelar", "rules", "outline"),
        ]),
    ]);
}

fn send_test_notification() {
    sdk::log("Enviando notificación de prueba...");
    sdk::respond_ok("Notificación de prueba enviada");
}

fn save_template(data: &str) {
    sdk::log(&format!("Guardando plantilla: {}", data));
    sdk::respond_ok("Plantilla guardada");
}

fn save_rule(data: &str) {
    sdk::log(&format!("Guardando regla: {}", data));
    sdk::respond_ok("Regla guardada");
}

fn clear_notification_history() {
    sdk::kv_set_val("notifications_sent", "0");
    sdk::kv_set_val("notifications_failed", "0");
    sdk::respond_ok("Historial limpiado");
}

fn export_notification_history() {
    sdk::respond_ok("Historial exportado");
}

fn check_pending_notifications() {
    sdk::log("Verificando notificaciones pendientes...");
}

fn notify_ticket_created(ticket: &sdk::Ticket) {
    let msg = format!("🎫 Nuevo ticket #{}\n\n{}\n\nPrioridad: {}", 
        ticket.id, ticket.asunto, ticket.prioridad);
    sdk::log(&format!("Notificación de ticket: {}", msg));
}

fn notify_sla_breach(breach: &sdk::Ticket) {
    let msg = format!("⚠️ SLA violado en ticket #{}\n\n{}", 
        breach.id, breach.asunto);
    sdk::log(&format!("Notificación de SLA: {}", msg));
}
