// Ejemplo 15: Dashboard de Agentes
// Features: Agent queries, Performance metrics, Charts, KV store
// Demuestra: Rendimiento individual de agentes, comparativas, métricas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("agentes", "Dashboard Agentes", "team-line")
                        .category("operaciones")
                        .priority(9)
                )
                .name("Dashboard de Agentes")
                .description("Monitorea el rendimiento y métricas de los agentes de soporte")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "agentes" => render_agents_dashboard(),
                "detail" => render_agent_detail(),
                "ranking" => render_agent_ranking(),
                "comparison" => render_agent_comparison(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "view_agent" => view_agent_detail(&data),
                "refresh" => render_agents_dashboard(),
                "export_report" => export_agent_report(),
                "send_feedback" => send_agent_feedback(&data),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

// Dashboard principal de agentes
fn render_agents_dashboard() {
    let agent_count = sdk::kv_get_val("agent_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Dashboard de Agentes", vec![
            sdk::text("Monitorea el rendimiento de tu equipo de soporte", "info"),
            sdk::divider(),

            // Métricas generales
            sdk::card("Métricas Generales", vec![
                sdk::text(&format!("👥 Agentes activos: {}", agent_count), "info"),
                sdk::text("📊 Tiempo promedio de respuesta: 4.2 min", "info"),
                sdk::text("✅ Tasa de resolución: 87%", "info"),
                sdk::text("⭐ Satisfacción promedio: 4.3/5", "info"),
                sdk::chart("Rendimiento General", vec![
                    ("Tickets Resueltos", 156.0),
                    ("En Progreso", 23.0),
                    ("Pendientes", 12.0),
                ], "bar"),
            ]),

            // Tabla de agentes
            sdk::card("Agentes Activos", vec![
                sdk::table(
                    vec!["Agente", "Tickets", "Tiempo Prom", "Resolución", "Satisfacción", "Estado"],
                    vec![
                        vec!["María García", "45", "3.2 min", "92%", "4.8", "🟢 Activo"],
                        vec!["Carlos Ruiz", "38", "4.5 min", "85%", "4.5", "🟢 Activo"],
                        vec!["Ana Torres", "42", "3.8 min", "88%", "4.6", "🟢 Activo"],
                        vec!["Roberto Díaz", "31", "5.1 min", "82%", "4.2", "🟡 Ocupado"],
                        vec!["Laura Sánchez", "0", "-", "-", "-", "🔴 Inactivo"],
                    ],
                ),
            ]),

            sdk::divider(),

            // Acciones
            sdk::card("Acciones", vec![
                sdk::button("Ver Ranking", "ranking", "primary"),
                sdk::button("Comparar Agentes", "comparison", "secondary"),
                sdk::button("Exportar Reporte", "export_report", "secondary"),
                sdk::button("Actualizar", "refresh", "outline"),
            ]),
        ]),
    ]);
}

// Detalle de un agente específico
fn render_agent_detail() {
    let agent_name = sdk::kv_get_val("current_agent_name")
        .unwrap_or("María García".to_string());

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Detalle: {}", agent_name), vec![
            sdk::text("Información del agente y métricas de rendimiento", "info"),
            sdk::divider(),

            // Información personal
            sdk::card("Información", vec![
                sdk::text(&format!("👤 Nombre: {}", agent_name), "info"),
                sdk::text("📧 Email: maria.garcia@empresa.com", "info"),
                sdk::text("🏢 Departamento: Soporte Técnico", "info"),
                sdk::text("📅 Ingreso: 2023-01-15", "info"),
                sdk::text("🟢 Estado: Activo", "success"),
            ]),

            // Métricas de rendimiento
            sdk::card("Métricas de Rendimiento", vec![
                sdk::text("📊 Tickets este mes: 45", "info"),
                sdk::text("⏱️ Tiempo promedio de respuesta: 3.2 min", "info"),
                sdk::text("✅ Tasa de resolución: 92%", "success"),
                sdk::text("⭐ Satisfacción del cliente: 4.8/5", "info"),
                sdk::text("📈 Tickets por día: 2.1", "info"),
            ]),

            // Gráficos de rendimiento
            sdk::card("Tendencia de Rendimiento", vec![
                sdk::chart("Tickets Resueltos por Semana", vec![
                    ("Sem 1", 11.0),
                    ("Sem 2", 12.0),
                    ("Sem 3", 10.0),
                    ("Sem 4", 12.0),
                ], "bar"),
                sdk::chart("Satisfacción por Mes", vec![
                    ("Ene", 4.5),
                    ("Feb", 4.6),
                    ("Mar", 4.7),
                    ("Abr", 4.8),
                ], "line"),
            ]),

            // Últimos tickets
            sdk::card("Últimos Tickets Atendidos", vec![
                sdk::table(
                    vec!["ID", "Asunto", "Estado", "Tiempo", "Satisfacción"],
                    vec![
                        vec!["TK-1234", "Error en login", "Resuelto", "2.5 min", "⭐ 5"],
                        vec!["TK-1230", "Problema de facturación", "Resuelto", "4.1 min", "⭐ 4"],
                        vec!["TK-1225", "Solicitud de cambio", "En progreso", "1.2 min", "-"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::button("Enviar Feedback", "send_feedback", "primary"),
            sdk::button("Volver", "agentes", "outline"),
        ]),
    ]);
}

// Ranking de agentes
fn render_agent_ranking() {
    sdk::respond(sdk::widgets![
        sdk::card("Ranking de Agentes", vec![
            sdk::text("Clasificación de agentes por rendimiento", "info"),
            sdk::divider(),

            // Ranking por satisfacción
            sdk::card("Top por Satisfacción", vec![
                sdk::chart("Satisfacción del Cliente", vec![
                    ("María García", 4.8),
                    ("Ana Torres", 4.6),
                    ("Carlos Ruiz", 4.5),
                    ("Roberto Díaz", 4.2),
                ], "bar"),
            ]),

            // Ranking por resolución
            sdk::card("Top por Tasa de Resolución", vec![
                sdk::chart("Resolución de Tickets", vec![
                    ("María García", 92.0),
                    ("Ana Torres", 88.0),
                    ("Carlos Ruiz", 85.0),
                    ("Roberto Díaz", 82.0),
                ], "bar"),
            ]),

            // Ranking por velocidad
            sdk::card("Top por Velocidad de Respuesta", vec![
                sdk::chart("Tiempo Promedio (min)", vec![
                    ("María García", 3.2),
                    ("Ana Torres", 3.8),
                    ("Carlos Ruiz", 4.5),
                    ("Roberto Díaz", 5.1),
                ], "bar"),
            ]),

            sdk::divider(),

            sdk::button("Volver", "agentes", "outline"),
        ]),
    ]);
}

// Comparación de agentes
fn render_agent_comparison() {
    sdk::respond(sdk::widgets![
        sdk::card("Comparación de Agentes", vec![
            sdk::text("Compara el rendimiento entre agentes", "info"),
            sdk::divider(),

            sdk::select("agent1", "Agente 1", vec![
                ("maria", "María García"),
                ("carlos", "Carlos Ruiz"),
                ("ana", "Ana Torres"),
                ("roberto", "Roberto Díaz"),
            ]),

            sdk::select("agent2", "Agente 2", vec![
                ("maria", "María García"),
                ("carlos", "Carlos Ruiz"),
                ("ana", "Ana Torres"),
                ("roberto", "Roberto Díaz"),
            ]),

            sdk::divider(),

            // Comparación lado a lado
            sdk::card("Resultados", vec![
                sdk::chart("Comparativa", vec![
                    ("María García", 92.0),
                    ("Carlos Ruiz", 85.0),
                ], "bar"),

                sdk::table(
                    vec!["Métrica", "Agente 1", "Agente 2"],
                    vec![
                        vec!["Tickets Resueltos", "45", "38"],
                        vec!["Tiempo Promedio", "3.2 min", "4.5 min"],
                        vec!["Tasa Resolución", "92%", "85%"],
                        vec!["Satisfacción", "4.8", "4.5"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::button("Volver", "agentes", "outline"),
        ]),
    ]);
}

// Ver detalle de un agente
fn view_agent_detail(data: &str) {
    let agent_name = extract_field(data, "agent_name")
        .unwrap_or("María García".to_string());

    sdk::kv_set_val("current_agent_name", &agent_name);
    sdk::log(&format!("Viendo detalle de agente: {}", agent_name));
    render_agent_detail();
}

// Exporta reporte de agentes
fn export_agent_report() {
    sdk::respond(sdk::widgets![
        sdk::card("Exportar Reporte", vec![
            sdk::text("Exportando reporte de rendimiento de agentes...", "info"),
            sdk::text("El reporte incluirá métricas detalladas de cada agente", "info"),
            sdk::button("Volver", "agentes", "outline"),
        ]),
    ]);
}

// Envía feedback a un agente
fn send_agent_feedback(data: &str) {
    let agent_name = extract_field(data, "agent_name").unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Enviar Feedback", vec![
            sdk::text(&format!("Enviando feedback a: {}", agent_name), "info"),
            sdk::textarea("feedback", "Feedback")
                .placeholder("Escribe tu feedback..."),
            sdk::select("feedback_type", "Tipo", vec![
                ("positive", "Positivo"),
                ("improvement", "Área de Mejora"),
                ("training", "Necesita Capacitación"),
            ]),
            sdk::button("Enviar", "agentes", "primary"),
            sdk::button("Cancelar", "agentes", "outline"),
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
