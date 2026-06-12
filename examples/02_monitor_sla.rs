// Ejemplo 2: Monitor de SLA
// Features: SLA queries, Events, Notifications, Cron
// Demuestra: Monitoreo en tiempo real, alertas, cumplimiento

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("sla", "Monitor SLA", "shield-check-line")
                        .category("operaciones")
                        .priority(8)
                )
                .name("Monitor de SLA")
                .description("Monitorea el cumplimiento de acuerdos de nivel de servicio")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "sla" => render_sla_dashboard(),
                _ => {}
            }
        }

        // ════════════════════════════════════════════════════════════════
        //  EVENTOS DE SLA - Reaccionar a violaciones
        // ════════════════════════════════════════════════════════════════
        PluginEvent::SlaBreachDetected(breach) => {
            sdk::log(&format!("⚠️ SLA violado: ticket {}", breach.ticket_id));
            
            // Guardar violación para reportes
            let violations = sdk::kv_get_val("sla_violations")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0) + 1;
            sdk::kv_set_val("sla_violations", &violations.to_string());
            
            // Guardar detalles de la violación
            let details = format!(
                "Ticket: {} | Departamento: {} | Tiempo: {} min",
                breach.ticket_id, breach.department, breach.time_minutes
            );
            sdk::kv_set_val(&format!("violation_{}", breach.ticket_id), &details);
        }

        PluginEvent::TicketCreated(ticket) => {
            // Verificar si el ticket tiene SLA asociado
            sdk::log(&format!("Nuevo ticket con SLA: {}", ticket.asunto));
        }

        PluginEvent::TicketStatusChanged(ticket) => {
            // Verificar si se cumplió el SLA al cerrar
            if ticket.new_status == "Cerrado" {
                sdk::log(&format!("Ticket {} cerrado - verificando SLA", ticket.id));
            }
        }

        _ => {}
    }
    0
}

fn render_sla_dashboard() {
    // Consultar políticas SLA
    let sla_policies = sdk::query::sla_policies().limit(50).all();
    let sla_events = sdk::query::sla_events().limit(20).all();
    let ticket_stats = sdk::query::ticket_stats();

    match (sla_policies, sla_events, ticket_stats) {
        (Ok(policies), Ok(events), Ok(stats)) => {
            // Calcular métricas de cumplimiento
            let total_events = events.len();
            let breach_count = sdk::kv_get_val("sla_violations")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            
            let compliance_rate = if total_events > 0 {
                ((total_events as f64 - breach_count as f64) / total_events as f64 * 100.0) as i32
            } else {
                100
            };

            sdk::respond(sdk::widgets![
                // Resumen de SLA
                sdk::card("Resumen de Cumplimiento", vec![
                    sdk::text(&format!("📋 Políticas activas: {}", policies.len()), "info"),
                    sdk::text(&format!("📊 Total eventos SLA: {}", total_events), "info"),
                    sdk::text(&format!("⚠️ Violaciones: {}", breach_count), "warning"),
                    sdk::text(
                        &format!("✅ Tasa de cumplimiento: {}%", compliance_rate),
                        if compliance_rate >= 95 { "success" } else { "warning" }
                    ),
                ]),

                // Gráfico de cumplimiento
                sdk::card("Cumplimiento por Departamento", vec![
                    sdk::chart("SLA Compliance", vec![
                        ("Cumplidos", (total_events - breach_count) as f64),
                        ("Violados", breach_count as f64),
                    ], "pie"),
                ]),

                // Políticas SLA
                sdk::card("Políticas SLA Activas", vec![
                    sdk::table(
                        vec!["ID", "Departamento", "Prioridad", "Respuesta (min)", "Resolución (min)"],
                        policies.iter().map(|p| {
                            vec![
                                p.id.as_str(),
                                p.id_departamento.as_str(),
                                p.id_prioridad.as_str(),
                                &p.tiempo_respuesta_minutos.to_string(),
                                &p.tiempo_resolucion_minutos.to_string(),
                            ]
                        }).collect(),
                    ),
                ]),

                // Últimas violaciones
                sdk::card("Últimas Violaciones", vec![
                    sdk::table(
                        vec!["ID", "Ticket", "Tipo", "Estado", "Fecha"],
                        events.iter().take(10).map(|e| {
                            vec![
                                e.id.as_str(),
                                e.id_ticket.as_str(),
                                e.tipo.as_str(),
                                e.estado.as_str(),
                                e.creado_en.as_str(),
                            ]
                        }).collect(),
                    ),
                ]),

                // Tickets próximos a vencer
                sdk::card("Tickets Críticos", vec![
                    sdk::text(
                        &format!("Tickets abiertos: {}", stats.abiertos),
                        if stats.abiertos > 10 { "warning" } else { "info" }
                    ),
                    sdk::text(
                        &format!("Tickets en progreso: {}", stats.en_progreso),
                        "info"
                    ),
                    sdk::button("Actualizar", "refresh", "outline"),
                ]),
            ]);
        }
        _ => {
            sdk::respond(sdk::widgets![
                sdk::text("Error cargando datos de SLA", "error")
            ]);
        }
    }
}
