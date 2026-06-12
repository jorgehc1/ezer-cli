// Ejemplo 1: Dashboard de Métricas
// Features: Chart, Table, Analytics queries, Ticket stats
// Demuestra: Queries múltiples, visualización de datos, formateo

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        // ════════════════════════════════════════════════════════════════
        //  METADATA - Configuración del plugin en el sistema
        // ════════════════════════════════════════════════════════════════
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("dashboard", "Métricas", "bar-chart-line")
                        .category("operaciones")
                        .priority(5)
                )
                .name("Dashboard de Métricas")
                .description("Vista general del helpdesk con métricas en tiempo real")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        // ════════════════════════════════════════════════════════════════
        //  PÁGINA PRINCIPAL - Dashboard con métricas
        // ════════════════════════════════════════════════════════════════
        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "dashboard" => {
                    render_dashboard();
                }
                _ => {}
            }
        }

        // ════════════════════════════════════════════════════════════════
        //  ACCIONES - Botones interactivos
        // ════════════════════════════════════════════════════════════════
        PluginEvent::PluginAction { action, .. } => {
            match action.as_str() {
                "refresh" => {
                    render_dashboard();
                }
                "view_tickets" => {
                    render_ticket_list();
                }
                "view_agents" => {
                    render_agent_list();
                }
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        // ════════════════════════════════════════════════════════════════
        //  EVENTOS DEL SISTEMA - Reaccionar a cambios
        // ════════════════════════════════════════════════════════════════
        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Nuevo ticket: {} - {}", ticket.id, ticket.asunto));
            // Guardar contador de tickets hoy
            let today_count = sdk::kv_get_val("tickets_today")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0) + 1;
            sdk::kv_set_val("tickets_today", &today_count.to_string());
        }

        PluginEvent::TicketStatusChanged(ticket) => {
            sdk::log(&format!("Ticket {} cambió estado a {}", ticket.id, ticket.new_status));
        }

        _ => {}
    }
    0
}

// ══════════════════════════════════════════════════════════════════════════════
//  FUNCIONES HELPER
// ══════════════════════════════════════════════════════════════════════════════

fn render_dashboard() {
    // Consultar analytics del sistema
    let analytics = sdk::query::analytics().get();
    let ticket_stats = sdk::query::ticket_stats();

    match (analytics, ticket_stats) {
        (Ok(stats), Ok(ts)) => {
            sdk::respond(sdk::widgets![
                // Tarjetas de resumen
                sdk::card("Resumen del Sistema", vec![
                    sdk::text(&format!("📊 Tickets totales: {}", stats.total_tickets), "info"),
                    sdk::text(&format!("🔓 Abiertos: {}", stats.tickets_abiertos), "warning"),
                    sdk::text(&format!("✅ Cerrados: {}", stats.tickets_cerrados), "success"),
                    sdk::text(&format!("👥 Agentes activos: {}", stats.agentes_activos), "info"),
                    sdk::text(&format!("📅 Última semana: {}", stats.tickets_ultima_semana), "info"),
                ]),

                // Gráfico de distribución por estado
                sdk::card("Distribución por Estado", vec![
                    sdk::chart("Tickets por Estado", vec![
                        ("Abiertos", ts.abiertos as f64),
                        ("En Progreso", ts.en_progreso as f64),
                        ("Cerrados", ts.cerrados as f64),
                    ], "pie"),
                ]),

                // Gráfico de prioridad
                sdk::card("Distribución por Prioridad", vec![
                    sdk::chart("Tickets por Prioridad", vec![
                        ("Alta", ts.prioridad_alta as f64),
                        ("Media", ts.prioridad_media as f64),
                        ("Baja", ts.prioridad_baja as f64),
                    ], "bar"),
                ]),

                // Botones de acción
                sdk::card("Acciones Rápidas", vec![
                    sdk::button("Ver Tickets Abiertos", "view_tickets", "primary"),
                    sdk::button("Ver Agentes", "view_agents", "secondary"),
                    sdk::button("Actualizar", "refresh", "outline"),
                ]),
            ]);
        }
        _ => {
            sdk::respond(sdk::widgets![
                sdk::text("Error cargando métricas del sistema", "error")
            ]);
        }
    }
}

fn render_ticket_list() {
    match sdk::query::tickets()
        .by_status("Abierto")
        .limit(20)
        .all()
    {
        Ok(tickets) => {
            let rows: Vec<Vec<&str>> = tickets.iter().map(|t| {
                vec![t.id.as_str(), t.asunto.as_str(), t.prioridad.as_str()]
            }).collect();

            sdk::respond(sdk::widgets![
                sdk::card("Tickets Abiertos", vec![
                    sdk::table(
                        vec!["ID", "Asunto", "Prioridad"],
                        rows,
                    ),
                    sdk::text(&format!("Total: {} tickets", tickets.len()), "info"),
                    sdk::button("Volver al Dashboard", "refresh", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error cargando tickets", "error")
            ]);
        }
    }
}

fn render_agent_list() {
    match sdk::query::agents().limit(20).all() {
        Ok(agents) => {
            let rows: Vec<Vec<&str>> = agents.iter().map(|a| {
                vec![a.id.as_str(), format!("{} {}", a.nombres, a.apellidos).as_str(), a.correo.as_str()]
            }).collect();

            sdk::respond(sdk::widgets![
                sdk::card("Agentes Activos", vec![
                    sdk::table(
                        vec!["ID", "Nombre", "Email"],
                        rows,
                    ),
                    sdk::text(&format!("Total: {} agentes", agents.len()), "info"),
                    sdk::button("Volver al Dashboard", "refresh", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error cargando agentes", "error")
            ]);
        }
    }
}
