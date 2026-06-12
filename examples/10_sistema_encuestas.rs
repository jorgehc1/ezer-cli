// Ejemplo 10: Sistema de Encuestas
// Features: Formularios, Charts, KV store, Analytics
// Demuestra: Creación de encuestas, recolección de respuestas, análisis

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("encuestas", "Encuestas", "questionnaire-line")
                        .category("operaciones")
                        .priority(18)
                )
                .name("Sistema de Encuestas")
                .description("Crea y gestiona encuestas de satisfacción y retroalimentación")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "encuestas" => render_surveys_dashboard(),
                "create" => render_create_survey(),
                "respond" => render_respond_survey(),
                "results" => render_survey_results(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "create_survey" => handle_create_survey(&data),
                "submit_response" => handle_submit_response(&data),
                "view_results" => render_survey_results(),
                "delete_survey" => delete_survey(&data),
                "export_results" => export_survey_results(),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

// Dashboard principal de encuestas
fn render_surveys_dashboard() {
    let survey_count = sdk::kv_get_val("survey_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let response_count = sdk::kv_get_val("response_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Sistema de Encuestas", vec![
            sdk::text("Gestiona encuestas de satisfacción y retroalimentación", "info"),
            sdk::divider(),

            // Estadísticas
            sdk::card("Estadísticas Generales", vec![
                sdk::text(&format!("📋 Encuestas creadas: {}", survey_count), "info"),
                sdk::text(&format!("📝 Respuestas recibidas: {}", response_count), "info"),
                sdk::chart("Respuestas por Día", vec![
                    ("Lunes", 12.0),
                    ("Martes", 19.0),
                    ("Miércoles", 15.0),
                    ("Jueves", 22.0),
                    ("Viernes", 18.0),
                ], "bar"),
            ]),

            // Acciones
            sdk::card("Acciones", vec![
                sdk::button("Crear Nueva Encuesta", "create", "primary"),
                sdk::button("Ver Encuestas Activas", "respond", "secondary"),
                sdk::button("Ver Resultados", "results", "secondary"),
                sdk::button("Exportar Resultados", "export_results", "outline"),
            ]),
        ]),
    ]);
}

// Formulario para crear una encuesta
fn render_create_survey() {
    sdk::respond(sdk::widgets![
        sdk::card("Crear Nueva Encuesta", vec![
            sdk::text("Define los detalles de tu encuesta", "info"),
            sdk::divider(),

            sdk::input("survey_title", "Título de la Encuesta")
                .placeholder("Encuesta de Satisfacción Q1 2024")
                .required(true),

            sdk::textarea("survey_description", "Descripción")
                .placeholder("Describe el propósito de la encuesta..."),

            sdk::select("survey_type", "Tipo de Encuesta", vec![
                ("satisfaccion", "Satisfacción del Cliente"),
                ("feedback", "Feedback de Producto"),
                ("soporte", "Calidad de Soporte"),
                ("nps", "Net Promoter Score (NPS)"),
                ("custom", "Personalizada"),
            ]),

            sdk::select("target_audience", "Público Objetivo", vec![
                ("all", "Todos los clientes"),
                ("active", "Clientes activos"),
                ("new", "Nuevos clientes"),
                ("churned", "Clientes perdidos"),
            ]),

            sdk::divider(),

            sdk::text("Preguntas de la Encuesta", "info"),

            // Preguntas predefinidas
            sdk::input("q1", "Pregunta 1")
                .placeholder("¿Qué tan satisfecho estás con nuestro servicio?")
                .required(true),
            sdk::select("q1_type", "Tipo de Respuesta", vec![
                ("rating", "Calificación (1-5)"),
                ("yes_no", "Sí/No"),
                ("text", "Texto libre"),
            ]),

            sdk::input("q2", "Pregunta 2")
                .placeholder("¿Recomendarías nuestro servicio?")
                .required(true),
            sdk::select("q2_type", "Tipo de Respuesta", vec![
                ("nps", "NPS (0-10)"),
                ("rating", "Calificación (1-5)"),
                ("text", "Texto libre"),
            ]),

            sdk::input("q3", "Pregunta 3")
                .placeholder("¿Qué改进as sugerirías?")
                .required(false),

            sdk::divider(),

            sdk::button("Crear Encuesta", "create_survey", "primary"),
            sdk::button("Cancelar", "encuestas", "outline"),
        ]),
    ]);
}

// Formulario para responder una encuesta
fn render_respond_survey() {
    let survey_title = sdk::kv_get_val("active_survey_title")
        .unwrap_or("Encuesta de Satisfacción".to_string());

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Responder: {}", survey_title), vec![
            sdk::text("Por favor responde las siguientes preguntas", "info"),
            sdk::divider(),

            // Pregunta 1: Calificación
            sdk::card("Pregunta 1", vec![
                sdk::text("¿Qué tan satisfecho estás con nuestro servicio?", "default"),
                sdk::select("rating_1", "Calificación", vec![
                    ("5", "⭐⭐⭐⭐⭐ Excelente"),
                    ("4", "⭐⭐⭐⭐ Bueno"),
                    ("3", "⭐⭐⭐ Regular"),
                    ("2", "⭐⭐ Malo"),
                    ("1", "⭐ Muy Malo"),
                ]),
            ]),

            // Pregunta 2: NPS
            sdk::card("Pregunta 2", vec![
                sdk::text("¿Recomendarías nuestro servicio a un colega?", "default"),
                sdk::number_input("nps_score", "Puntuación (0-10)")
                    .min(0)
                    .max(10),
            ]),

            // Pregunta 3: Texto libre
            sdk::card("Pregunta 3", vec![
                sdk::text("¿Qué改进as nos sugerirías?", "default"),
                sdk::textarea("feedback_text", "Tu sugerencia")
                    .placeholder("Escribe tu sugerencia aquí..."),
            ]),

            sdk::divider(),

            sdk::button("Enviar Respuesta", "submit_response", "primary"),
            sdk::button("Cancelar", "encuestas", "outline"),
        ]),
    ]);
}

// Muestra los resultados de las encuestas
fn render_survey_results() {
    let total_responses = sdk::kv_get_val("response_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Resultados de Encuestas", vec![
            sdk::text(&format!("📊 Total de respuestas: {}", total_responses), "info"),
            sdk::divider(),

            // Distribución de calificaciones
            sdk::card("Calificaciones", vec![
                sdk::chart("Distribución de Calificaciones", vec![
                    ("⭐ 5 - Excelente", 35.0),
                    ("⭐ 4 - Bueno", 28.0),
                    ("⭐ 3 - Regular", 18.0),
                    ("⭐ 2 - Malo", 12.0),
                    ("⭐ 1 - Muy Malo", 7.0),
                ], "pie"),
            ]),

            // NPS Score
            sdk::card("Net Promoter Score", vec![
                sdk::chart("NPS Distribution", vec![
                    ("Promotores (9-10)", 45.0),
                    ("Pasivos (7-8)", 35.0),
                    ("Detractores (0-6)", 20.0),
                ], "pie"),
                sdk::text("NPS Score: +25", "info"),
            ]),

            // Satisfacción general
            sdk::card("Satisfacción General", vec![
                sdk::chart("Tendencia de Satisfacción", vec![
                    ("Ene", 4.2),
                    ("Feb", 4.3),
                    ("Mar", 4.1),
                    ("Abr", 4.5),
                    ("May", 4.4),
                ], "line"),
            ]),

            // Tabla de respuestas recientes
            sdk::card("Respuestas Recientes", vec![
                sdk::table(
                    vec!["Fecha", "Calificación", "NPS", "Feedback"],
                    vec![
                        vec!["2024-01-15", "5", "9", "Excelente servicio"],
                        vec!["2024-01-14", "4", "8", "Buen soporte técnico"],
                        vec!["2024-01-13", "3", "6", "Tiempo de respuesta lento"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::button("Exportar Resultados", "export_results", "primary"),
            sdk::button("Volver al Dashboard", "encuestas", "outline"),
        ]),
    ]);
}

// Maneja la creación de una encuesta
fn handle_create_survey(data: &str) {
    let title = extract_field(data, "survey_title").unwrap_or_default();
    let description = extract_field(data, "survey_description").unwrap_or_default();
    let survey_type = extract_field(data, "survey_type").unwrap_or_default();

    if title.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ El título es obligatorio", "warning"),
        ]);
        return;
    }

    // Guardar encuesta en KV store
    let survey_id = format!("survey_{}", chrono::Utc::now().timestamp());
    let survey_data = format!(
        "title:{}|desc:{}|type:{}|status:active",
        title, description, survey_type
    );

    sdk::kv_set_val(&survey_id, &survey_data);
    sdk::kv_set_val("active_survey_title", &title);

    let count = sdk::kv_get_val("survey_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("survey_count", &count.to_string());

    sdk::log(&format!("Encuesta creada: {} ({})", title, survey_id));

    sdk::respond(sdk::widgets![
        sdk::card("Encuesta Creada", vec![
            sdk::text("✅ Encuesta creada exitosamente", "success"),
            sdk::text(&format!("📋 Título: {}", title), "info"),
            sdk::text(&format!("📝 Descripción: {}", description), "info"),
            sdk::text(&format!("🏷️ Tipo: {}", survey_type), "info"),
            sdk::button("Ver Encuestas", "encuestas", "primary"),
        ]),
    ]);
}

// Maneja el envío de una respuesta
fn handle_submit_response(data: &str) {
    let rating = extract_field(data, "rating_1").unwrap_or_default();
    let nps = extract_field(data, "nps_score").unwrap_or_default();
    let feedback = extract_field(data, "feedback_text").unwrap_or_default();

    // Guardar respuesta
    let response_id = format!("response_{}", chrono::Utc::now().timestamp());
    let response_data = format!(
        "rating:{}|nps:{}|feedback:{}",
        rating, nps, feedback
    );

    sdk::kv_set_val(&response_id, &response_data);

    let count = sdk::kv_get_val("response_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("response_count", &count.to_string());

    sdk::log(&format!("Respuesta de encuesta registrada: {}", response_id));

    sdk::respond(sdk::widgets![
        sdk::card("Respuesta Enviada", vec![
            sdk::text("✅ Gracias por tu respuesta", "success"),
            sdk::divider(),
            sdk::text(&format!("⭐ Calificación: {}/5", rating), "info"),
            sdk::text(&format!("📊 NPS: {}/10", nps), "info"),
            sdk::text(&format!("💬 Feedback: {}", feedback), "info"),
            sdk::divider(),
            sdk::text(&format!("Total de respuestas: {}", count), "info"),
            sdk::button("Volver al Dashboard", "encuestas", "primary"),
        ]),
    ]);
}

// Elimina una encuesta
fn delete_survey(data: &str) {
    let survey_id = extract_field(data, "survey_id").unwrap_or_default();
    if !survey_id.is_empty() {
        sdk::kv_set_val(&survey_id, "");
        sdk::log(&format!("Encuesta eliminada: {}", survey_id));
    }
    sdk::respond(sdk::widgets![
        sdk::text("✅ Encuesta eliminada", "success"),
        sdk::button("Volver al Dashboard", "encuestas", "primary"),
    ]);
}

// Exporta los resultados
fn export_survey_results() {
    let count = sdk::kv_get_val("response_count")
        .unwrap_or("0".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Exportar Resultados", vec![
            sdk::text(&format!("📊 Exportando {} respuestas...", count), "info"),
            sdk::text("Los resultados se exportarán en formato CSV", "info"),
            sdk::button("Volver al Dashboard", "encuestas", "primary"),
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
