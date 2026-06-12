// Ejemplo 9: Chatbot Inteligente
// Features: Chat queries, Knowledge base, HTTP para AI, KV store
// Demuestra: Integración con IA, búsqueda de contexto, conversación

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("chatbot", "Chatbot IA", "robot-line")
                        .category("herramientas")
                        .priority(7)
                )
                .name("Chatbot Inteligente")
                .description("Asistente virtual con IA para responder consultas de clientes")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "chatbot" => render_chatbot(),
                "chat_history" => render_chat_history(),
                "config_bot" => render_bot_config(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "send_message" => handle_chat_message(&data),
                "clear_chat" => clear_chat_history(),
                "search_context" => search_knowledge_context(&data),
                "escalate" => escalate_to_human(&data),
                "save_config" => save_bot_config(&data),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        // Evento de chat: responder automáticamente
        PluginEvent::ChatMessage(msg) => {
            handle_incoming_chat(&msg);
        }

        _ => {}
    }
    0
}

// Renderiza la interfaz del chatbot
fn render_chatbot() {
    // Obtener historial de chat del KV store
    let history = sdk::kv_get_val("chat_history").unwrap_or_default();
    let bot_name = sdk::kv_get_val("bot_name")
        .unwrap_or("EzerBot".to_string());
    let msg_count = sdk::kv_get_val("chat_msg_count")
        .unwrap_or("0".to_string());

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Chatbot: {}", bot_name), vec![
            sdk::text("Asistente virtual potenciado por Inteligencia Artificial", "info"),
            sdk::divider(),

            // Estadísticas del chat
            sdk::card("Estadísticas", vec![
                sdk::text(&format!("💬 Mensajes en esta sesión: {}", msg_count), "info"),
                sdk::text(&format!("🤖 Nombre del bot: {}", bot_name), "info"),
                sdk::button("Ver Historial Completo", "chat_history", "outline"),
            ]),

            sdk::divider(),

            // Campo de entrada del chat
            sdk::input("user_message", "Escribe tu mensaje")
                .placeholder("¿En qué puedo ayudarte?")
                .required(true),

            sdk::divider(),

            // Botones de acción
            sdk::button("Enviar Mensaje", "send_message", "primary"),
            sdk::button("Buscar en Conocimiento", "search_context", "secondary"),
            sdk::button("Escalar a Humano", "escalate", "warning"),
            sdk::button("Limpiar Chat", "clear_chat", "outline"),
            sdk::button("Configurar Bot", "config_bot", "outline"),
        ]),
    ]);
}

// Maneja el envío de un mensaje del usuario
fn handle_chat_message(data: &str) {
    let user_msg = extract_field(data, "user_message").unwrap_or_default();

    if user_msg.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Escribe un mensaje para enviar", "warning"),
        ]);
        return;
    }

    // Incrementar contador de mensajes
    let msg_count = sdk::kv_get_val("chat_msg_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("chat_msg_count", &msg_count.to_string());

    // Guardar en historial
    let history_entry = format!("Usuario: {}\n", user_msg);
    let mut history = sdk::kv_get_val("chat_history").unwrap_or_default();
    history.push_str(&history_entry);
    sdk::kv_set_val("chat_history", &history);

    // Buscar contexto en la base de conocimiento
    let context = search_context_for_query(&user_msg);

    // Llamar a la API de IA (simulado con HTTP)
    let ai_response = call_ai_api(&user_msg, &context);

    // Guardar respuesta del bot
    let bot_response = format!("Bot: {}\n", ai_response);
    let mut history = sdk::kv_get_val("chat_history").unwrap_or_default();
    history.push_str(&bot_response);
    sdk::kv_set_val("chat_history", &history);

    // Mostrar respuesta
    sdk::respond(sdk::widgets![
        sdk::card("Respuesta del Chatbot", vec![
            sdk::text(&format!("👤 Usuario: {}", user_msg), "default"),
            sdk::divider(),
            sdk::text(&format!("🤖 Bot: {}", ai_response), "success"),
            sdk::divider(),
            sdk::text(&format!("📝 Mensajes en sesión: {}", msg_count), "info"),
            sdk::button("Continuar Chat", "send_message", "primary"),
            sdk::button("Escalar a Humano", "escalate", "warning"),
        ]),
    ]);
}

// Busca contexto en la base de conocimiento
fn search_knowledge_context(data: &str) {
    let query = extract_field(data, "query").unwrap_or_default();

    if query.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Ingresa un término para buscar", "warning"),
        ]);
        return;
    }

    let results = sdk::query::knowledge()
        .search(&query)
        .limit(5)
        .all();

    match results {
        Ok(articles) => {
            sdk::respond(sdk::widgets![
                sdk::card(&format!("Contexto para: \"{}\"", query), vec![
                    sdk::text(&format!("📚 {} artículos encontrados", articles.len()), "info"),
                    sdk::table(
                        vec!["Título", "Categoría", "Contenido"],
                        articles.iter().map(|a| {
                            vec![
                                a.titulo.as_str(),
                                a.categoria.as_str(),
                                a.contenido.as_str(),
                            ]
                        }).collect(),
                    ),
                    sdk::button("Usar como Contexto", "send_message", "primary"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("No se encontró contexto relevante", "warning"),
            ]);
        }
    }
}

// Escala la conversación a un agente humano
fn escalate_to_human(data: &str) {
    // Crear ticket de soporte para escalamiento
    let ticket_id = sdk::query::tickets()
        .create()
        .asunto("Escalamiento desde Chatbot")
        .descripcion("Cliente solicitó hablar con un agente humano")
        .prioridad("Alta")
        .estado("Abierto")
        .build();

    match ticket_id {
        Ok(id) => {
            sdk::kv_set_val("escalated_ticket", &id);
            sdk::respond(sdk::widgets![
                sdk::card("Escalamiento a Humano", vec![
                    sdk::text("✅ Ticket de escalamiento creado", "success"),
                    sdk::text(&format!("📋 Ticket ID: {}", id), "info"),
                    sdk::text("Un agente se pondrá en contacto contigo pronto", "info"),
                    sdk::button("Volver al Chat", "clear_chat", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error creando ticket de escalamiento", "error"),
            ]);
        }
    }
}

// Llama a la API de IA para generar respuesta
fn call_ai_api(user_msg: &str, context: &str) -> String {
    // Construir el prompt con contexto
    let prompt = if !context.is_empty() {
        format!(
            "Contexto del helpdesk:\n{}\n\nPregunta del cliente: {}",
            context, user_msg
        )
    } else {
        format!("Pregunta del cliente: {}", user_msg)
    };

    // Simular llamada HTTP a API de IA
    // En producción: sdk::http_post("https://api.openai.com/v1/chat/completions", ...)
    sdk::log(&format!("Llamando a API de IA con prompt: {}", &prompt[..50.min(prompt.len())]));

    // Respuesta simulada del bot
    let response = format!(
        "Gracias por tu consulta. Basándome en nuestra documentación, \
         puedo ayudarte con: {}. Si necesitas más asistencia, \
         puedo conectarte con un agente humano.",
        user_msg
    );

    response
}

// Busca contexto para una consulta
fn search_context_for_query(query: &str) -> String {
    match sdk::query::knowledge().search(query).limit(3).all() {
        Ok(articles) => {
            articles.iter()
                .map(|a| format!("{}: {}", a.titulo, a.contenido))
                .collect::<Vec<_>>()
                .join("\n")
        }
        Err(_) => String::new(),
    }
}

// Renderiza el historial de chat
fn render_chat_history() {
    let history = sdk::kv_get_val("chat_history").unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Historial de Chat", vec![
            if history.is_empty() {
                sdk::text("No hay mensajes en el historial", "default")
            } else {
                sdk::text(&history, "default")
            },
            sdk::divider(),
            sdk::button("Volver al Chat", "clear_chat", "outline"),
        ]),
    ]);
}

// Renderiza configuración del bot
fn render_bot_config() {
    let bot_name = sdk::kv_get_val("bot_name")
        .unwrap_or("EzerBot".to_string());
    let model = sdk::kv_get_val("ai_model")
        .unwrap_or("gpt-3.5-turbo".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Configuración del Chatbot", vec![
            sdk::input("bot_name", "Nombre del Bot", &bot_name),
            sdk::select("ai_model", "Modelo de IA", vec![
                ("gpt-3.5-turbo", "GPT-3.5 Turbo"),
                ("gpt-4", "GPT-4"),
                ("claude-3", "Claude 3"),
                ("local", "Modelo Local"),
            ]),
            sdk::select("response_style", "Estilo de Respuesta", vec![
                ("formal", "Formal"),
                ("casual", "Casual"),
                ("tecnico", "Técnico"),
            ]),
            sdk::divider(),
            sdk::button("Guardar Configuración", "save_config", "primary"),
        ]),
    ]);
}

// Guarda la configuración del bot
fn save_bot_config(data: &str) {
    let bot_name = extract_field(data, "bot_name").unwrap_or("EzerBot".to_string());
    let model = extract_field(data, "ai_model").unwrap_or("gpt-3.5-turbo".to_string());
    let style = extract_field(data, "response_style").unwrap_or("formal".to_string());

    sdk::kv_set_val("bot_name", &bot_name);
    sdk::kv_set_val("ai_model", &model);
    sdk::kv_set_val("response_style", &style);

    sdk::respond(sdk::widgets![
        sdk::card("Configuración Guardada", vec![
            sdk::text("✅ Configuración actualizada exitosamente", "success"),
            sdk::text(&format!("🤖 Nombre: {}", bot_name), "info"),
            sdk::text(&format!("🧠 Modelo: {}", model), "info"),
            sdk::text(&format!("💬 Estilo: {}", style), "info"),
            sdk::button("Volver al Chat", "clear_chat", "outline"),
        ]),
    ]);
}

// Limpia el historial de chat
fn clear_chat_history() {
    sdk::kv_set_val("chat_history", "");
    sdk::kv_set_val("chat_msg_count", "0");
    sdk::log("Historial de chat limpiado");
    render_chatbot();
}

// Maneja mensajes entrantes de chat (evento del sistema)
fn handle_incoming_chat(msg: &sdk::ChatMessage) {
    sdk::log(&format!("Mensaje de chat recibido de: {}", msg.sender));

    // Buscar contexto y generar respuesta automática
    let context = search_context_for_query(&msg.content);
    let response = call_ai_api(&msg.content, &context);

    sdk::log(&format!("Respuesta automática generada para {}", msg.sender));
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
