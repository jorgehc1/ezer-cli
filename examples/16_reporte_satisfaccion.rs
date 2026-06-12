// Ejemplo 16: Reporte de Satisfaccion
// Features: Analytics, Charts, Survey queries, Export
// Demuestra: Analisis de satisfaccion del cliente, tendencias, reportes

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("satisfaccion", "Satisfaccion", "emotion-happy-line")
                        .category("reportes")
                        .priority(16)
                )
                .name("Reporte de Satisfaccion")
                .description("Analiza la satisfaccion del cliente con encuestas y metricas")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "satisfaccion" => render_satisfaction_dashboard(),
                "nps" => render_nps_report(),
                "csat" => render_csat_report(),
                "trends" => render_satisfaction_trends(),
                "feedback" => render_feedback_analysis(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "refresh" => render_satisfaction_dashboard(),
                "export" => export_satisfaction_report(),
                "send_survey" => send_satisfaction_survey(&data),
                "view_ticket_feedback" => view_ticket_feedback(&data),
                _ => {
                    sdk::respond_ok("Accion no reconocida");
                }
            }
        }

        PluginEvent::TicketStatusChanged(ticket) => {
            if ticket.new_status == "Cerrado" {
                sdk::log(&format!(
                    "Ticket {} cerrado - enviando encuesta de satisfaccion",
                    ticket.id
                ));
            }
        }

        _ => {}
    }
    0
}

fn render_satisfaction_dashboard() {
    sdk::respond(sdk::widgets![
        sdk::card("Reporte de Satisfaccion del Cliente", vec![
            sdk::text("Analisis completo de la satisfaccion de tus clientes", "info"),
            sdk::divider(),

            sdk::card("Metricas Principales", vec![
                sdk::text("CSAT Score: 4.3/5 (86%)", "success"),
                sdk::text("NPS Score: +32 (Excelente)", "success"),
                sdk::text("Tasa de Respuesta: 78%", "info"),
                sdk::text("Total Encuestas: 234", "info"),
            ]),

            sdk::card("Satisfaccion General", vec![
                sdk::chart("Distribucion de Calificaciones", vec![
                    ("5 - Excelente", 42.0),
                    ("4 - Bueno", 28.0),
                    ("3 - Regular", 18.0),
                    ("2 - Malo", 8.0),
                    ("1 - Muy Malo", 4.0),
                ], "pie"),
            ]),

            sdk::card("Tendencia Mensual", vec![
                sdk::chart("CSAT por Mes", vec![
                    ("Ene", 4.1),
                    ("Feb", 4.2),
                    ("Mar", 4.0),
                    ("Abr", 4.3),
                    ("May", 4.4),
                ], "line"),
            ]),

            sdk::divider(),

            sdk::card("Reportes", vec![
                sdk::button("Ver NPS Completo", "nps", "primary"),
                sdk::button("Ver CSAT Detallado", "csat", "secondary"),
                sdk::button("Ver Tendencias", "trends", "secondary"),
                sdk::button("Analisis de Feedback", "feedback", "secondary"),
                sdk::button("Exportar Reporte", "export", "outline"),
            ]),
        ]),
    ]);
}

fn render_nps_report() {
    sdk::respond(sdk::widgets![
        sdk::card("Net Promoter Score (NPS)", vec![
            sdk::text("Mide la lealtad de tus clientes", "info"),
            sdk::divider(),

            sdk::card("NPS Score", vec![
                sdk::text("NPS Score: +32", "success"),
                sdk::text("Calificacion: EXCELENTE", "success"),
                sdk::divider(),
                sdk::text("Promotores (9-10): 52% - Clientes leales que recomiendan tu servicio", "success"),
                sdk::text("Pasivos (7-8): 30% - Clientes satisfechos pero vulnerables a competencia", "info"),
                sdk::text("Detractores (0-6): 18% - Clientes insatisfechos", "warning"),
            ]),

            sdk::card("Distribucion de Puntuaciones", vec![
                sdk::chart("NPS Distribution", vec![
                    ("10 - Promotor", 15.0),
                    ("9 - Promotor", 12.0),
                    ("8 - Pasivo", 18.0),
                    ("7 - Pasivo", 12.0),
                    ("6 - Detractor", 8.0),
                    ("5 - Detractor", 5.0),
                    ("4 - Detractor", 3.0),
                    ("3 - Detractor", 2.0),
                ], "bar"),
            ]),

            sdk::card("Tendencia NPS", vec![
                sdk::chart("NPS por Mes", vec![
                    ("Ene", 28.0),
                    ("Feb", 30.0),
                    ("Mar", 25.0),
                    ("Abr", 32.0),
                    ("May", 35.0),
                ], "line"),
            ]),

            sdk::card("NPS por Canal", vec![
                sdk::chart("Por Canal de Soporte", vec![
                    ("Email", 35.0),
                    ("Chat", 40.0),
                    ("Telefono", 28.0),
                    ("Self-Service", 32.0),
                ], "bar"),
            ]),

            sdk::divider(),
            sdk::button("Exportar NPS", "export", "primary"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
}

fn render_csat_report() {
    sdk::respond(sdk::widgets![
        sdk::card("Customer Satisfaction (CSAT)", vec![
            sdk::text("Mide la satisfaccion inmediata del cliente", "info"),
            sdk::divider(),

            sdk::card("CSAT Score", vec![
                sdk::text("CSAT Score: 4.3/5 (86%)", "success"),
                sdk::text("Meta: 4.0/5 (80%) - SUPERADO", "success"),
            ]),

            sdk::card("Distribucion de Calificaciones", vec![
                sdk::chart("CSAT Distribution", vec![
                    ("5 - Muy Satisfecho", 42.0),
                    ("4 - Satisfecho", 28.0),
                    ("3 - Neutral", 18.0),
                    ("2 - Insatisfecho", 8.0),
                    ("1 - Muy Insatisfecho", 4.0),
                ], "pie"),
            ]),

            sdk::card("CSAT por Departamento", vec![
                sdk::chart("Por Departamento", vec![
                    ("Soporte Tecnico", 4.5),
                    ("Ventas", 4.2),
                    ("Facturacion", 3.9),
                    ("General", 4.3),
                ], "bar"),
            ]),

            sdk::card("CSAT por Agente", vec![
                sdk::chart("Por Agente", vec![
                    ("Maria Garcia", 4.8),
                    ("Ana Torres", 4.6),
                    ("Carlos Ruiz", 4.5),
                    ("Roberto Diaz", 4.2),
                ], "bar"),
            ]),

            sdk::divider(),
            sdk::button("Exportar CSAT", "export", "primary"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
}

fn render_satisfaction_trends() {
    sdk::respond(sdk::widgets![
        sdk::card("Tendencias de Satisfaccion", vec![
            sdk::text("Analisis de tendencias a largo plazo", "info"),
            sdk::divider(),

            sdk::card("Tendencia Anual", vec![
                sdk::chart("CSAT Anual", vec![
                    ("2020", 3.8),
                    ("2021", 4.0),
                    ("2022", 4.1),
                    ("2023", 4.3),
                    ("2024", 4.4),
                ], "line"),
            ]),

            sdk::card("NPS por Trimestre", vec![
                sdk::chart("NPS Trimestral", vec![
                    ("Q1 2023", 25.0),
                    ("Q2 2023", 28.0),
                    ("Q3 2023", 30.0),
                    ("Q4 2023", 32.0),
                    ("Q1 2024", 35.0),
                ], "bar"),
            ]),

            sdk::card("Factores de Satisfaccion", vec![
                sdk::chart("Impacto por Factor", vec![
                    ("Tiempo de Respuesta", 85.0),
                    ("Calidad de Solucion", 92.0),
                    ("Amabilidad", 88.0),
                    ("Conocimiento", 78.0),
                ], "bar"),
            ]),

            sdk::divider(),
            sdk::button("Exportar Tendencias", "export", "primary"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
}

fn render_feedback_analysis() {
    sdk::respond(sdk::widgets![
        sdk::card("Analisis de Feedback", vec![
            sdk::text("Analisis cualitativo de comentarios de clientes", "info"),
            sdk::divider(),

            sdk::card("Palabras Frecuentes", vec![
                sdk::chart("Terminos Mas Usados", vec![
                    ("rapido", 45.0),
                    ("util", 38.0),
                    ("amable", 32.0),
                    ("eficiente", 28.0),
                    ("profesional", 25.0),
                ], "bar"),
            ]),

            sdk::card("Sentimiento del Feedback", vec![
                sdk::chart("Distribucion de Sentimiento", vec![
                    ("Positivo", 62.0),
                    ("Neutral", 25.0),
                    ("Negativo", 13.0),
                ], "pie"),
            ]),

            sdk::card("Comentarios Recientes", vec![
                sdk::table(
                    vec!["Fecha", "Cliente", "Comentario", "Sentimiento"],
                    vec![
                        vec!["2024-01-15", "Juan P.", "Excelente servicio, muy rapido", "Positivo"],
                        vec!["2024-01-14", "Ana L.", "Buena atencion pero demora", "Neutral"],
                        vec!["2024-01-13", "Pedro M.", "No resolvieron mi problema", "Negativo"],
                    ],
                ),
            ]),

            sdk::card("Temas Principales", vec![
                sdk::chart("Temas Mas Mencionados", vec![
                    ("Tiempo de espera", 35.0),
                    ("Calidad de solucion", 28.0),
                    ("Amabilidad del agente", 22.0),
                    ("Facilidad de uso", 15.0),
                ], "bar"),
            ]),

            sdk::divider(),
            sdk::button("Exportar Feedback", "export", "primary"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
}

fn export_satisfaction_report() {
    sdk::respond(sdk::widgets![
        sdk::card("Exportar Reporte de Satisfaccion", vec![
            sdk::text("Preparando reporte completo...", "info"),
            sdk::text("El reporte incluira: CSAT, NPS, Tendencias y Feedback", "info"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
}

fn send_satisfaction_survey(data: &str) {
    let ticket_id = extract_field(data, "ticket_id").unwrap_or_default();
    sdk::log(&format!("Enviando encuesta de satisfaccion para ticket: {}", ticket_id));

    sdk::respond(sdk::widgets![
        sdk::card("Encuesta Enviada", vec![
            sdk::text("Encuesta de satisfaccion enviada exitosamente", "success"),
            sdk::text(&format!("Ticket: {}", ticket_id), "info"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
}

fn view_ticket_feedback(data: &str) {
    let ticket_id = extract_field(data, "ticket_id").unwrap_or_default();
    sdk::kv_set_val("current_ticket_id", &ticket_id);

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Feedback del Ticket: {}", ticket_id), vec![
            sdk::text("CSAT: 4/5", "info"),
            sdk::text("Comentario: Buen servicio pero la demora fue mucha", "info"),
            sdk::text("Fecha: 2024-01-15", "info"),
            sdk::button("Volver", "satisfaccion", "outline"),
        ]),
    ]);
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
