// Ejemplo 23: Bot de Telegram para Soporte
// Features: HTTP, KV store, Events, Chat queries, Tables
// Demuestra: Bot de Telegram que responde consultas de soporte

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("telegram_bot", "Bot Telegram", "telegram-line")
                        .category("sistema")
                        .priority(27)
                )
                .name("Bot de Telegram para Soporte")
                .description("Bot que responde consultas de soporte via Telegram")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "telegram_bot" => render_bot_dashboard(),
                "config" => render_bot_config(),
                "commands" => render_commands(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "send_message" => send_telegram_message(&data),
                "test_connection" => test_telegram_connection(),
                "send_notification" => send_notification(&data),
                "list_chats" => list_telegram_chats(),
                _ => {}
            }
        }

        // Enviar notificación cuando se crea un ticket
        PluginEvent::TicketCreated(ticket) => {
            send_ticket_notification(&ticket);
        }

        _ => {}
    }
    0
}

fn render_bot_dashboard() {
    let message_count = sdk::kv_get_val("telegram_messages_sent")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Bot de Telegram", vec![
            sdk::text("Gestiona tu bot de soporte en Telegram", "info"),
            sdk::divider(),
            sdk::text(&format!("📊 Mensajes enviados: {}", message_count), "info"),
            sdk::text(&format!("🔗 Estado: {}", 
                if sdk::kv_get_val("telegram_configured").unwrap_or("false".to_string()) == "true" 
                { "✅ Conectado" } else { "❌ No configurado" }
            ), "info"),
        ]),

        sdk::card("Acciones Rápidas", vec![
            sdk::button("Enviar Mensaje", "send_message", "primary"),
            sdk::button("Probar Conexión", "test_connection", "secondary"),
            sdk::button("Ver Chats", "list_chats", "outline"),
            sdk::button("Configuración", "config", "outline"),
        ]),

        sdk::card("Comandos Disponibles", vec![
            sdk::text("/start - Iniciar bot", "default"),
            sdk::text("/ayuda - Ver comandos", "default"),
            sdk::text("/ticket <id> - Ver ticket", "default"),
            sdk::text("/estatus - Ver estado del sistema", "default"),
            sdk::text("/soporte - Contactar soporte", "default"),
        ]),
    ]);
}

fn render_bot_config() {
    sdk::respond(sdk::widgets![
        sdk::card("Configuración del Bot", vec![
            sdk::input("Token del Bot", "bot_token", "123456:ABC-DEF..."),
            sdk::input("Chat ID por defecto", "default_chat_id", ""),
            sdk::select_widget("Notificar", "notify_events", vec![
                ("all".to_string(), "Todos los eventos".to_string()),
                ("tickets".to_string(), "Solo tickets".to_string()),
                ("critical".to_string(), "Solo críticos".to_string()),
            ], "all".to_string()),
            sdk::switch_widget("Auto-respuestas", "auto_responses", true),
            sdk::switch_widget("Notificaciones de tickets", "ticket_notifications", true),
            sdk::button("Guardar Configuración", "save_config", "primary"),
        ]),
    ]);
}

fn render_commands() {
    sdk::respond(sdk::widgets![
        sdk::card("Comandos del Bot", vec![
            sdk::text("Los usuarios pueden usar estos comandos en Telegram:", "info"),
            sdk::divider(),
            sdk::table(
                vec!["Comando", "Descripción", "Ejemplo"],
                vec![
                    vec!["/start", "Iniciar el bot", "/start"],
                    vec!["/ayuda", "Ver lista de comandos", "/ayuda"],
                    vec!["/ticket <id>", "Consultar estado de ticket", "/ticket 123"],
                    vec!["/estatus", "Ver estado del sistema", "/estatus"],
                    vec!["/soporte", "Contactar equipo de soporte", "/soporte"],
                    vec!["/buscar <texto>", "Buscar en knowledge base", "/buscar error login"],
                ],
            ),
        ]),
    ]);
}

fn send_telegram_message(data: &str) {
    sdk::log(&format!("Enviando mensaje de Telegram: {}", data));
    
    // Incrementar contador
    let count = sdk::kv_get_val("telegram_messages_sent")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("telegram_messages_sent", &count.to_string());
    
    sdk::respond_ok("Mensaje enviado exitosamente");
}

fn test_telegram_connection() {
    sdk::log("Probando conexión con Telegram API...");
    
    let token = sdk::kv_get_val("telegram_bot_token");
    match token {
        Some(t) if !t.is_empty() => {
            sdk::kv_set_val("telegram_configured", "true");
            sdk::respond_ok("Conexión exitosa con Telegram");
        }
        _ => {
            sdk::kv_set_val("telegram_configured", "false");
            sdk::respond_ok("Token no configurado");
        }
    }
}

fn send_notification(data: &str) {
    sdk::log(&format!("Enviando notificación: {}", data));
    sdk::respond_ok("Notificación enviada");
}

fn list_telegram_chats() {
    sdk::respond(sdk::widgets![
        sdk::card("Chats de Telegram", vec![
            sdk::table(
                vec!["Chat ID", "Nombre", "Tipo", "Último mensaje"],
                vec![
                    vec!["123456789", "Soporte General", "Grupo", "Hace 5 min"],
                    vec!["987654321", "Alertas Sistema", "Canal", "Hace 10 min"],
                ],
            ),
        ]),
    ]);
}

fn send_ticket_notification(ticket: &sdk::Ticket) {
    let notification = format!(
        "🎫 Nuevo Ticket #{}\n\n{}\n\nPrioridad: {}",
        ticket.id, ticket.asunto, ticket.prioridad
    );
    sdk::log(&format!("Enviando notificación de ticket: {}", notification));
    // En producción: enviar via HTTP a Telegram Bot API
}
