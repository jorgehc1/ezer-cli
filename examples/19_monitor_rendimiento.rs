// Ejemplo 19: Monitor de Rendimiento
// Features: Analytics queries, Charts, KV store, Real-time metrics
// Demuestra: Monitoreo de rendimiento del sistema, alertas, metricas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("rendimiento", "Monitor Rendimiento", "pulse-line")
                        .category("sistema")
                        .priority(6)
                )
                .name("Monitor de Rendimiento")
                .description("Monitorea el rendimiento y salud del sistema en tiempo real")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "rendimiento" => render_performance_dashboard(),
                "alerts" => render_alerts(),
                "resources" => render_resources(),
                "logs" => render_system_logs(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "refresh" => render_performance_dashboard(),
                "ack_alert" => acknowledge_alert(&data),
                "clear_logs" => clear_system_logs(),
                "export_metrics" => export_performance_metrics(),
                _ => {
                    sdk::respond_ok("Accion no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

fn render_performance_dashboard() {
    sdk::respond(sdk::widgets![
        sdk::card("Monitor de Rendimiento", vec![
            sdk::text("Metricas en tiempo real del sistema", "info"),
            sdk::divider(),

            sdk::card("Estado del Sistema", vec![
                sdk::text("CPU: 45%", "success"),
                sdk::text("Memoria: 68%", "warning"),
                sdk::text("Disco: 52%", "success"),
                sdk::text("Red: 12ms latency", "success"),
            ]),

            sdk::card("Metricas de Aplicacion", vec![
                sdk::text("Request/segundo: 1,234", "info"),
                sdk::text("Tiempo promedio respuesta: 45ms", "success"),
                sdk::text("Tasa de errores: 0.2%", "success"),
                sdk::text("Conexiones activas: 89", "info"),
            ]),

            sdk::card("Uso de Recursos", vec![
                sdk::chart("CPU por Hora", vec![
                    ("00:00", 25.0),
                    ("04:00", 15.0),
                    ("08:00", 65.0),
                    ("12:00", 78.0),
                    ("16:00", 72.0),
                    ("20:00", 45.0),
                ], "line"),
            ]),

            sdk::card("Tasa de Respuesta", vec![
                sdk::chart("Tiempo de Respuesta (ms)", vec![
                    ("< 50ms", 65.0),
                    ("50-100ms", 25.0),
                    ("100-200ms", 8.0),
                    ("> 200ms", 2.0),
                ], "pie"),
            ]),

            sdk::divider(),

            sdk::card("Acciones", vec![
                sdk::button("Ver Alertas", "alerts", "primary"),
                sdk::button("Ver Recursos", "resources", "secondary"),
                sdk::button("Ver Logs", "logs", "secondary"),
                sdk::button("Exportar Metricas", "export_metrics", "secondary"),
                sdk::button("Actualizar", "refresh", "outline"),
            ]),
        ]),
    ]);
}

fn render_alerts() {
    sdk::respond(sdk::widgets![
        sdk::card("Alertas del Sistema", vec![
            sdk::text("Alertas activas y historial de incidentes", "info"),
            sdk::divider(),

            sdk::card("Alertas Activas", vec![
                sdk::table(
                    vec!["Nivel", "Mensaje", "Origen", "Tiempo", "Estado"],
                    vec![
                        vec!["WARNING", "Uso de memoria alto (68%)", "Server-01", "hace 5m", "Activa"],
                        vec!["INFO", "Backup completado exitosamente", "Scheduler", "hace 10m", "Ack"],
                    ],
                ),
            ]),

            sdk::card("Historial de Alertas (24h)", vec![
                sdk::chart("Alertas por Hora", vec![
                    ("00:00", 2.0),
                    ("04:00", 1.0),
                    ("08:00", 5.0),
                    ("12:00", 3.0),
                    ("16:00", 4.0),
                    ("20:00", 1.0),
                ], "bar"),
            ]),

            sdk::card("Distribucion por Nivel", vec![
                sdk::chart("Niveles de Alerta", vec![
                    ("CRITICAL", 2.0),
                    ("WARNING", 8.0),
                    ("INFO", 15.0),
                ], "pie"),
            ]),

            sdk::divider(),

            sdk::button("Reconocer Alerta", "ack_alert", "primary"),
            sdk::button("Volver", "rendimiento", "outline"),
        ]),
    ]);
}

fn render_resources() {
    sdk::respond(sdk::widgets![
        sdk::card("Uso de Recursos", vec![
            sdk::text("Detalle del uso de recursos del servidor", "info"),
            sdk::divider(),

            sdk::card("CPU", vec![
                sdk::chart("Uso de CPU", vec![
                    ("Nucleo 1", 45.0),
                    ("Nucleo 2", 52.0),
                    ("Nucleo 3", 38.0),
                    ("Nucleo 4", 61.0),
                ], "bar"),
                sdk::text("Promedio: 49%", "info"),
            ]),

            sdk::card("Memoria", vec![
                sdk::chart("Uso de Memoria", vec![
                    ("Usada", 68.0),
                    ("Cache", 15.0),
                    ("Libre", 17.0),
                ], "pie"),
                sdk::text("Total: 16GB | Usada: 10.9GB", "info"),
            ]),

            sdk::card("Disco", vec![
                sdk::chart("Uso de Disco", vec![
                    ("/", 52.0),
                    ("/home", 65.0),
                    ("/var", 38.0),
                    ("/tmp", 12.0),
                ], "bar"),
                sdk::text("Total: 500GB | Usado: 260GB", "info"),
            ]),

            sdk::card("Red", vec![
                sdk::chart("Trafico de Red", vec![
                    ("Entrada", 125.0),
                    ("Salida", 89.0),
                ], "bar"),
                sdk::text("Latencia: 12ms | Paquetes perdidos: 0.01%", "info"),
            ]),

            sdk::divider(),

            sdk::button("Volver", "rendimiento", "outline"),
        ]),
    ]);
}

fn render_system_logs() {
    sdk::respond(sdk::widgets![
        sdk::card("Logs del Sistema", vec![
            sdk::text("Registro de eventos del sistema", "info"),
            sdk::divider(),

            sdk::table(
                vec!["Fecha", "Nivel", "Componente", "Mensaje"],
                vec![
                    vec!["15:30:45", "INFO", "API", "Request completado: GET /tickets"],
                    vec!["15:30:42", "WARN", "DB", "Query lenta detectada: 2.3s"],
                    vec!["15:30:40", "INFO", "Auth", "Usuario admin@ejemplo.com logueado"],
                    vec!["15:30:38", "ERROR", "Email", "Error enviando email: SMTP timeout"],
                    vec!["15:30:35", "INFO", "System", "Backup diario completado"],
                ],
            ),

            sdk::divider(),

            sdk::button("Limpiar Logs", "clear_logs", "warning"),
            sdk::button("Volver", "rendimiento", "outline"),
        ]),
    ]);
}

fn acknowledge_alert(data: &str) {
    let alert_id = extract_field(data, "alert_id").unwrap_or_default();
    sdk::log(&format!("Alerta reconocida: {}", alert_id));

    sdk::respond(sdk::widgets![
        sdk::text("Alerta reconocida", "success"),
        sdk::button("Volver", "alerts", "outline"),
    ]);
}

fn clear_system_logs() {
    sdk::kv_set_val("system_logs", "");
    sdk::log("Logs del sistema limpiados");

    sdk::respond(sdk::widgets![
        sdk::text("Logs limpiados exitosamente", "success"),
        sdk::button("Volver", "rendimiento", "outline"),
    ]);
}

fn export_performance_metrics() {
    sdk::respond(sdk::widgets![
        sdk::card("Exportar Metricas", vec![
            sdk::text("Preparando exportacion de metricas de rendimiento...", "info"),
            sdk::text("El reporte incluira: CPU, Memoria, Disco, Red, Tiempos de respuesta", "info"),
            sdk::button("Volver", "rendimiento", "outline"),
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
