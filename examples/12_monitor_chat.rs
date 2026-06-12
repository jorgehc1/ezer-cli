// Ejemplo 12: Monitor de Chat en Vivo
// Features: Chat queries, Sentiment analysis, KV store, Charts
// Demuestra: Monitoreo en tiempo real, análisis de sentimiento, métricas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("chat_monitor", "Monitor Chat", "message-3-line")
                        .category("operaciones")
                        .priority(11)
                )
                .name("Monitor de Chat en Vivo")
                .description("Monitorea chats en tiempo real con análisis de sentimiento")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "chat_monitor" => render_chat_monitor(),
                "active_chats" => render_active_chats(),
                "chat_detail" => render_chat_detail(),
                "sentiment" => render_sentiment_analysis(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "refresh" => render_chat_monitor(),
                "join_chat" => join_chat_session(&data),
                "end_chat" => end_chat_session(&data),
                "send_message" => send_chat_message(&data),
                "view_sentiment" => render_sentiment_analysis(),
                "export_chat" => export_chat_log(&data),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        // Evento de mensaje de chat entrante
        PluginEvent::ChatMessage(msg) => {
            handle_chat_message_event(&msg);
        }

        // Evento de chat iniciado
        PluginEvent::ChatStarted(session) => {
            sdk::log(&format!("Chat iniciado: sesión {}", session.id));
            let count = sdk::kv_get_val("active_chat_count")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0) + 1;
            sdk::kv_set_val("active_chat_count", &count.to_string());
        }

        // Evento de chat finalizado
        PluginEvent::ChatEnded(session) => {
            sdk::log(&format!("Chat finalizado: sesión {}", session.id));
            let count = sdk::kv_get_val("active_chat_count")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0) - 1;
            sdk::kv_set_val("active_chat_count", &count.to_string().max(&"0".to_string()).clone());
        }

        _ => {}
    }
    0
}

// Dashboard principal del monitor
fn render_chat_monitor() {
    let active_count = sdk::kv_get_val("active_chat_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let total_today = sdk::kv_get_val("chats_today")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let avg_response = sdk::kv_get_val("avg_response_time")
        .unwrap_or("0".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Monitor de Chat en Vivo", vec![
            sdk::text("Monitorea todas las sesiones de chat en tiempo real", "info"),
            sdk::divider(),

            // Métricas en tiempo real
            sdk::card("Métricas en Tiempo Real", vec![
                sdk::text(&format!("💬 Chats activos: {}", active_count), "info"),
                sdk::text(&format!("📊 Chats hoy: {}", total_today), "info"),
                sdk::text(&format!("⏱️ Tiempo promedio respuesta: {}s", avg_response), "info"),
                sdk::chart("Chats por Hora", vec![
                    ("09:00", 5.0),
                    ("10:00", 8.0),
                    ("11:00", 12.0),
                    ("12:00", 7.0),
                    ("13:00", 10.0),
                    ("14:00", 15.0),
                    ("15:00", 11.0),
                    ("16:00", 9.0),
                ], "line"),
            ]),

            // Análisis de sentimiento rápido
            sdk::card("Sentimiento General", vec![
                sdk::chart("Distribución de Sentimiento", vec![
                    ("Positivo", 45.0),
                    ("Neutral", 35.0),
                    ("Negativo", 20.0),
                ], "pie"),
            ]),

            sdk::divider(),

            // Acciones
            sdk::card("Acciones", vec![
                sdk::button("Ver Chats Activos", "active_chats", "primary"),
                sdk::button("Análisis de Sentimiento", "sentiment", "secondary"),
                sdk::button("Actualizar", "refresh", "outline"),
            ]),
        ]),
    ]);
}

// Lista de chats activos
fn render_active_chats() {
    let active_count = sdk::kv_get_val("active_chat_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Chats Activos", vec![
            sdk::text(&format!("💬 {} sesiones activas", active_count), "info"),
            sdk::divider(),

            sdk::table(
                vec!["Sesión", "Cliente", "Agente", "Duración", "Sentimiento", "Estado"],
                vec![
                    vec!["CHT-001", "Juan Pérez", "María García", "5m", "😊 Positivo", "Activo"],
                    vec!["CHT-002", "Ana López", "Carlos Ruiz", "12m", "😐 Neutral", "Activo"],
                    vec!["CHT-003", "Pedro Martín", "Sin asignar", "2m", "😟 Negativo", "Esperando"],
                    vec!["CHT-004", "Laura Sánchez", "Roberto Díaz", "8m", "😊 Positivo", "Activo"],
                ],
            ),

            sdk::divider(),

            sdk::card("Detalle de Sesión", vec![
                sdk::input("session_id", "ID de Sesión")
                    .placeholder("CHT-001"),
                sdk::button("Unirse al Chat", "join_chat", "primary"),
                sdk::button("Finalizar Chat", "end_chat", "warning"),
                sdk::button("Exportar Log", "export_chat", "secondary"),
                sdk::button("Volver", "chat_monitor", "outline"),
            ]),
        ]),
    ]);
}

// Detalle de un chat específico
fn render_chat_detail() {
    let session_id = sdk::kv_get_val("current_chat_session")
        .unwrap_or("CHT-001".to_string());

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Chat: {}", session_id), vec![
            sdk::text("Sesión de chat en tiempo real", "info"),
            sdk::divider(),

            // Información de la sesión
            sdk::card("Información", vec![
                sdk::text("👤 Cliente: Juan Pérez", "info"),
                sdk::text("👨‍💼 Agente: María García", "info"),
                sdk::text("⏱️ Duración: 5 minutos", "info"),
                sdk::text("😊 Sentimiento: Positivo", "success"),
            ]),

            // Mensajes del chat
            sdk::card("Mensajes", vec![
                sdk::text("[09:15] Cliente: Hola, tengo un problema con mi cuenta", "default"),
                sdk::text("[09:15] Agente: Buenos días, ¿en qué puedo ayudarle?", "default"),
                sdk::text("[09:16] Cliente: No puedo acceder a mi panel de control", "default"),
                sdk::text("[09:16] Agente: Permítame verificar su cuenta...", "default"),
                sdk::text("[09:17] Agente: Ya identifiqué el problema, es un error de sesión", "default"),
                sdk::text("[09:17] Cliente: ¡Perfecto, muchas gracias!", "success"),
            ]),

            sdk::divider(),

            // Enviar mensaje
            sdk::input("chat_message", "Enviar mensaje")
                .placeholder("Escribe un mensaje..."),
            sdk::button("Enviar", "send_message", "primary"),
            sdk::button("Finalizar Chat", "end_chat", "warning"),
            sdk::button("Volver", "active_chats", "outline"),
        ]),
    ]);
}

// Análisis de sentimiento
fn render_sentiment_analysis() {
    sdk::respond(sdk::widgets![
        sdk::card("Análisis de Sentimiento", vec![
            sdk::text("Análisis de sentimiento de todas las conversaciones", "info"),
            sdk::divider(),

            // Distribución general
            sdk::card("Distribución General", vec![
                sdk::chart("Sentimiento General", vec![
                    ("Positivo", 45.0),
                    ("Neutral", 35.0),
                    ("Negativo", 20.0),
                ], "pie"),
            ]),

            // Sentimiento por agente
            sdk::card("Sentimiento por Agente", vec![
                sdk::chart("Por Agente", vec![
                    ("María García", 85.0),
                    ("Carlos Ruiz", 72.0),
                    ("Roberto Díaz", 90.0),
                    ("Ana Torres", 78.0),
                ], "bar"),
            ]),

            // Tendencia
            sdk::card("Tendencia de Sentimiento", vec![
                sdk::chart("Última Semana", vec![
                    ("Lun", 78.0),
                    ("Mar", 82.0),
                    ("Mié", 75.0),
                    ("Jue", 88.0),
                    ("Vie", 85.0),
                ], "line"),
            ]),

            // Alertas de sentimiento negativo
            sdk::card("Alertas de Sentimiento Negativo", vec![
                sdk::table(
                    vec!["Sesión", "Cliente", "Agente", "Mensaje Clave"],
                    vec![
                        vec!["CHT-003", "Pedro Martín", "Sin asignar", "Muy frustrado con el servicio"],
                        vec!["CHT-007", "Luis Hernández", "Ana Torres", "Espera muy larga"],
                    ],
                ),
                sdk::text("⚠️ 2 sesiones con sentimiento negativo requieren atención", "warning"),
            ]),

            sdk::divider(),

            sdk::button("Actualizar Análisis", "sentiment", "primary"),
            sdk::button("Volver al Monitor", "chat_monitor", "outline"),
        ]),
    ]);
}

// Une a una sesión de chat
fn join_chat_session(data: &str) {
    let session_id = extract_field(data, "session_id").unwrap_or_default();

    if session_id.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Ingresa un ID de sesión", "warning"),
        ]);
        return;
    }

    sdk::kv_set_val("current_chat_session", &session_id);
    sdk::log(&format!("Uniéndose a sesión: {}", session_id));
    render_chat_detail();
}

// Finaliza una sesión de chat
fn end_chat_session(data: &str) {
    let session_id = extract_field(data, "session_id").unwrap_or_default();

    if !session_id.is_empty() {
        sdk::log(&format!("Finalizando sesión: {}", session_id));

        // Decrementar contador
        let count = sdk::kv_get_val("active_chat_count")
            .unwrap_or("0".to_string())
            .parse::<i32>()
            .unwrap_or(0) - 1;
        sdk::kv_set_val("active_chat_count", &count.max(&0).to_string());

        // Incrementar chats completados hoy
        let today = sdk::kv_get_val("chats_today")
            .unwrap_or("0".to_string())
            .parse::<i32>()
            .unwrap_or(0) + 1;
        sdk::kv_set_val("chats_today", &today.to_string());
    }

    sdk::respond(sdk::widgets![
        sdk::card("Chat Finalizado", vec![
            sdk::text("✅ Sesión de chat finalizada exitosamente", "success"),
            sdk::button("Volver al Monitor", "chat_monitor", "primary"),
        ]),
    ]);
}

// Envía un mensaje en el chat
fn send_chat_message(data: &str) {
    let message = extract_field(data, "chat_message").unwrap_or_default();

    if message.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Escribe un mensaje", "warning"),
        ]);
        return;
    }

    sdk::log(&format!("Mensaje enviado: {}", message));

    // Analizar sentimiento del mensaje
    let sentiment = analyze_sentiment(&message);
    sdk::log(&format!("Sentimiento detectado: {}", sentiment));

    sdk::respond(sdk::widgets![
        sdk::card("Mensaje Enviado", vec![
            sdk::text(&format!("✅ Mensaje enviado: {}", message), "success"),
            sdk::text(&format!("😊 Sentimiento: {}", sentiment), "info"),
            sdk::button("Continuar Chat", "chat_detail", "outline"),
        ]),
    ]);
}

// Analiza el sentimiento de un texto
fn analyze_sentiment(text: &str) -> String {
    // Análisis simplificado de sentimiento
    let positive_words = ["gracias", "perfecto", "excelente", "bueno", "genial", "ayuda"];
    let negative_words = ["problema", "error", "malo", "terrible", "frustrado", "espera"];

    let text_lower = text.to_lowercase();
    let positive_count = positive_words.iter()
        .filter(|w| text_lower.contains(*w))
        .count();
    let negative_count = negative_words.iter()
        .filter(|w| text_lower.contains(*w))
        .count();

    if positive_count > negative_count {
        "Positivo".to_string()
    } else if negative_count > positive_count {
        "Negativo".to_string()
    } else {
        "Neutral".to_string()
    }
}

// Maneja eventos de mensajes de chat
fn handle_chat_message_event(msg: &sdk::ChatMessage) {
    sdk::log(&format!("Mensaje de chat de {}: {}", msg.sender, msg.content));

    // Analizar sentimiento
    let sentiment = analyze_sentiment(&msg.content);
    sdk::log(&format!("Sentimiento: {}", sentiment));

    // Guardar en historial
    let entry = format!("{}: {} [{}]\n", msg.sender, msg.content, sentiment);
    let mut history = sdk::kv_get_val("chat_history_log").unwrap_or_default();
    history.push_str(&entry);
    sdk::kv_set_val("chat_history_log", &history);
}

// Exporta el log de un chat
fn export_chat_log(data: &str) {
    let session_id = extract_field(data, "session_id").unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Exportar Log de Chat", vec![
            sdk::text(&format!("Exportando log de sesión: {}", session_id), "info"),
            sdk::text("El log incluirá todos los mensajes con análisis de sentimiento", "info"),
            sdk::button("Volver", "active_chats", "outline"),
        ]),
    ]);
}

// Helper para extraer campos del JSON
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
