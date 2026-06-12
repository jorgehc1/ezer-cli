// Ejemplo 4: Reportes Programados
// Features: Cron, Analytics, Email, HTTP
// Demuestra: Automatización programada, generación de reportes

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("reports", "Reportes", "file-list-line")
                        .category("administracion")
                        .priority(15)
                )
                .name("Reportes Programados")
                .description("Genera y envía reportes automáticamente")
                .version("1.0.0")
                .cron("86400");  // Ejecutar cada 24 horas
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "reports" => render_reports_page(),
                "daily" => render_daily_report(),
                "weekly" => render_weekly_report(),
                _ => {}
            }
        }

        // ════════════════════════════════════════════════════════════════
        //  CRON TICK - Ejecución programada
        // ════════════════════════════════════════════════════════════════
        PluginEvent::CronTick => {
            sdk::log("⏰ Ejecutando reporte diario programado...");
            generate_daily_report();
        }

        PluginEvent::PluginAction { action, .. } => {
            match action.as_str() {
                "generate_daily" => generate_daily_report(),
                "generate_weekly" => generate_weekly_report(),
                "send_email" => send_report_email(),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

fn render_reports_page() {
    sdk::respond(sdk::widgets![
        sdk::card("Centro de Reportes", vec![
            sdk::text("Genera reportes automáticos del helpdesk", "info"),
            sdk::divider(),
            
            sdk::card("Reporte Diario", vec![
                sdk::text("Resumen de actividad del día anterior", "default"),
                sdk::button("Generar Ahora", "generate_daily", "primary"),
            ]),
            
            sdk::card("Reporte Semanal", vec![
                sdk::text("Resumen de actividad de la semana", "default"),
                sdk::button("Generar Ahora", "generate_weekly", "secondary"),
            ]),
            
            sdk::divider(),
            
            sdk::card("Configuración", vec![
                sdk::text("Los reportes se generan automáticamente cada 24 horas", "info"),
                sdk::text("Último reporte: " , "default"),
                sdk::text(
                    &sdk::kv_get_val("last_report_date").unwrap_or("Nunca".to_string()),
                    "info"
                ),
                sdk::button("Enviar por Email", "send_email", "outline"),
            ]),
        ]),
    ]);
}

fn render_daily_report() {
    let analytics = sdk::query::analytics().get();
    let ticket_stats = sdk::query::ticket_stats();
    let daily = sdk::query::analytics_daily().limit(7).all();

    match (analytics, ticket_stats, daily) {
        (Ok(stats), Ok(ts), Ok(daily_data)) => {
            sdk::respond(sdk::widgets![
                sdk::card("Reporte Diario - Resumen", vec![
                    sdk::text("📊 Métricas del día:", "info"),
                    sdk::text(&format!("• Tickets totales: {}", stats.total_tickets), "default"),
                    sdk::text(&format!("• Tickets abiertos: {}", stats.tickets_abiertos), "warning"),
                    sdk::text(&format!("• Tickets cerrados: {}", stats.tickets_cerrados), "success"),
                    sdk::text(&format!("• Agentes activos: {}", stats.agentes_activos), "info"),
                ]),
                
                sdk::card("Distribución por Prioridad", vec![
                    sdk::chart("Prioridad", vec![
                        ("Alta", ts.prioridad_alta as f64),
                        ("Media", ts.prioridad_media as f64),
                        ("Baja", ts.prioridad_baja as f64),
                    ], "bar"),
                ]),
                
                sdk::card("Tendencia de los Últimos 7 Días", vec![
                    sdk::chart("Tickets por Día", 
                        daily_data.iter().map(|d| (d.fecha.as_str(), d.total as f64)).collect(),
                        "line"
                    ),
                ]),
                
                sdk::button("Generar Reporte Completo", "generate_daily", "primary"),
            ]);
        }
        _ => {
            sdk::respond(sdk::widgets![
                sdk::text("Error generando reporte diario", "error")
            ]);
        }
    }
}

fn render_weekly_report() {
    let analytics = sdk::query::analytics().get();
    let dept_stats = sdk::query::analytics_by_department().all();
    let agent_perf = sdk::query::report_agents_performance().all();

    match (analytics, dept_stats, agent_perf) {
        (Ok(stats), Ok(departments), Ok(agents)) => {
            sdk::respond(sdk::widgets![
                sdk::card("Reporte Semanal - Resumen Ejecutivo", vec![
                    sdk::text("📈 Métricas de la semana:", "info"),
                    sdk::text(&format!("• Tickets procesados: {}", stats.total_tickets), "default"),
                    sdk::text(&format!("• Tasa de resolución: {}%", 
                        if stats.total_tickets > 0 {
                            (stats.tickets_cerrados * 100 / stats.total_tickets)
                        } else { 0 }
                    ), "info"),
                ]),
                
                sdk::card("Rendimiento por Departamento", vec![
                    sdk::chart("Tickets por Departamento",
                        departments.iter().map(|d| (d.departamento.as_str(), d.total as f64)).collect(),
                        "bar"
                    ),
                ]),
                
                sdk::card("Top Agentes", vec![
                    sdk::table(
                        vec!["Agente", "Tickets Asignados"],
                        agents.iter().take(10).map(|a| {
                            vec![a.agente.as_str(), &a.tickets_asignados.to_string()]
                        }).collect(),
                    ),
                ]),
                
                sdk::button("Generar Reporte Semanal", "generate_weekly", "primary"),
            ]);
        }
        _ => {
            sdk::respond(sdk::widgets![
                sdk::text("Error generando reporte semanal", "error")
            ]);
        }
    }
}

fn generate_daily_report() {
    sdk::log("Generando reporte diario...");
    
    let analytics = sdk::query::analytics().get();
    let ticket_stats = sdk::query::ticket_stats();
    
    match (analytics, ticket_stats) {
        (Ok(stats), Ok(ts)) => {
            let report = format!(
                "REPORTE DIARIO - {}\n\
                 ====================\n\
                 Tickets totales: {}\n\
                 Tickets abiertos: {}\n\
                 Tickets cerrados: {}\n\
                 Prioridad alta: {}\n\
                 Prioridad media: {}\n\
                 Prioridad baja: {}\n\
                 Agentes activos: {}\n",
                chrono::Utc::now().format("%Y-%m-%d"),
                stats.total_tickets, stats.tickets_abiertos, stats.tickets_cerrados,
                ts.prioridad_alta, ts.prioridad_media, ts.prioridad_baja,
                stats.agentes_activos
            );
            
            sdk::kv_set_val("last_daily_report", &report);
            sdk::kv_set_val("last_report_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
            
            sdk::respond(sdk::widgets![
                sdk::card("Reporte Diario Generado", vec![
                    sdk::text("✅ Reporte generado exitosamente", "success"),
                    sdk::text(&format!("📅 Fecha: {}", chrono::Utc::now().format("%Y-%m-%d")), "info"),
                    sdk::text(&format!("📊 Resumen: {} tickets, {} abiertos, {} cerrados", 
                        stats.total_tickets, stats.tickets_abiertos, stats.tickets_cerrados), "info"),
                ]),
            ]);
        }
        _ => {
            sdk::respond(sdk::widgets![
                sdk::text("Error generando reporte", "error")
            ]);
        }
    }
}

fn generate_weekly_report() {
    sdk::log("Generando reporte semanal...");
    
    let analytics = sdk::query::analytics().get();
    let dept_stats = sdk::query::analytics_by_department().all();
    let agent_perf = sdk::query::report_agents_performance().all();
    
    match (analytics, dept_stats, agent_perf) {
        (Ok(stats), Ok(departments), Ok(agents)) => {
            let report = format!(
                "REPORTE SEMANAL\n\
                 ==============\n\
                 Tickets totales: {}\n\
                 Tickets cerrados: {}\n\
                 Tasa de resolución: {}%\n\
                 Departamentos: {}\n\
                 Agentes activos: {}\n",
                stats.total_tickets, stats.tickets_cerrados,
                if stats.total_tickets > 0 { stats.tickets_cerrados * 100 / stats.total_tickets } else { 0 },
                departments.len(), stats.agentes_activos
            );
            
            sdk::kv_set_val("last_weekly_report", &report);
            
            sdk::respond(sdk::widgets![
                sdk::card("Reporte Semanal Generado", vec![
                    sdk::text("✅ Reporte semanal generado exitosamente", "success"),
                    sdk::text(&format!("📊 Resumen: {} tickets, {} departamentos", 
                        stats.total_tickets, departments.len()), "info"),
                ]),
            ]);
        }
        _ => {
            sdk::respond(sdk::widgets![
                sdk::text("Error generando reporte semanal", "error")
            ]);
        }
    }
}

fn send_report_email() {
    // Simular envío de email con el reporte
    let report = sdk::kv_get_val("last_daily_report")
        .unwrap_or("No hay reporte disponible".to_string());
    
    sdk::log(&format!("Enviando reporte por email: {} chars", report.len()));
    
    sdk::respond(sdk::widgets![
        sdk::card("Email Enviado", vec![
            sdk::text("✅ Reporte enviado por email exitosamente", "success"),
            sdk::text("El reporte ha sido enviado a los administradores del sistema", "info"),
        ]),
    ]);
}
