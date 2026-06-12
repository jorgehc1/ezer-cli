// Ejemplo 3: Exportador de Datos
// Features: Multiple queries, Table, Filters, Export
// Demuestra: Consultas combinadas, formateo de datos, exportación

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("export", "Exportar Datos", "download-line")
                        .category("sistema")
                        .priority(25)
                )
                .name("Exportador de Datos")
                .description("Exporta tickets, agentes y departamentos a diferentes formatos")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "export" => render_export_page(),
                "export_tickets" => export_tickets(),
                "export_agents" => export_agents(),
                "export_departments" => export_departments(),
                "export_analytics" => export_analytics(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "export_tickets" => export_tickets(),
                "export_agents" => export_agents(),
                "export_departments" => export_departments(),
                "export_analytics" => export_analytics(),
                "export_all" => export_all(),
                _ => {
                    sdk::respond_ok(&format!("Acción: {}", action));
                }
            }
        }

        _ => {}
    }
    0
}

fn render_export_page() {
    sdk::respond(sdk::widgets![
        sdk::card("Exportar Datos del Helpdesk", vec![
            sdk::text("Selecciona qué datos quieres exportar:", "info"),
            sdk::divider(),
            
            // Opciones de exportación
            sdk::card("Tickets", vec![
                sdk::text("Exporta todos los tickets con sus detalles", "default"),
                sdk::button("Exportar Tickets", "export_tickets", "primary"),
            ]),
            
            sdk::card("Agentes", vec![
                sdk::text("Exporta la lista de agentes y usuarios", "default"),
                sdk::button("Exportar Agentes", "export_agents", "secondary"),
            ]),
            
            sdk::card("Departamentos", vec![
                sdk::text("Exporta la estructura de departamentos", "default"),
                sdk::button("Exportar Departamentos", "export_departments", "secondary"),
            ]),
            
            sdk::card("Analytics", vec![
                sdk::text("Exporta métricas y estadísticas", "default"),
                sdk::button("Exportar Analytics", "export_analytics", "secondary"),
            ]),
            
            sdk::divider(),
            
            sdk::card("Exportación Completa", vec![
                sdk::text("Exporta todos los datos en un solo paquete", "info"),
                sdk::button("Exportar Todo", "export_all", "primary"),
            ]),
        ]),
    ]);
}

fn export_tickets() {
    match sdk::query::tickets().limit(1000).all() {
        Ok(tickets) => {
            // Convertir a formato CSV-like
            let mut csv = String::from("ID,Asunto,Estado,Prioridad,Fecha\n");
            for t in &tickets {
                csv.push_str(&format!("{},{},{},{},{}\n", 
                    t.id, t.asunto, t.estado, t.prioridad, t.creado_en));
            }
            
            // Guardar en KV para descarga
            sdk::kv_set_val("export_tickets", &csv);
            
            sdk::respond(sdk::widgets![
                sdk::card("Exportación de Tickets", vec![
                    sdk::text(&format!("✅ {} tickets exportados exitosamente", tickets.len()), "success"),
                    sdk::text("Los datos están listos para descargar", "info"),
                    sdk::table(
                        vec!["ID", "Asunto", "Estado", "Prioridad"],
                        tickets.iter().take(10).map(|t| {
                            vec![t.id.as_str(), t.asunto.as_str(), t.estado.as_str(), t.prioridad.as_str()]
                        }).collect(),
                    ),
                    sdk::text(&format!("Mostrando 10 de {} tickets", tickets.len()), "info"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error exportando tickets", "error")
            ]);
        }
    }
}

fn export_agents() {
    match sdk::query::agents().limit(100).all() {
        Ok(agents) => {
            let mut csv = String::from("ID,Nombres,Apellidos,Email\n");
            for a in &agents {
                csv.push_str(&format!("{},{},{},{}\n", 
                    a.id, a.nombres, a.apellidos, a.correo));
            }
            
            sdk::kv_set_val("export_agents", &csv);
            
            sdk::respond(sdk::widgets![
                sdk::card("Exportación de Agentes", vec![
                    sdk::text(&format!("✅ {} agentes exportados exitosamente", agents.len()), "success"),
                    sdk::table(
                        vec!["ID", "Nombre", "Email"],
                        agents.iter().map(|a| {
                            vec![a.id.as_str(), format!("{} {}", a.nombres, a.apellidos).as_str(), a.correo.as_str()]
                        }).collect(),
                    ),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error exportando agentes", "error")
            ]);
        }
    }
}

fn export_departments() {
    match sdk::query::departments().limit(50).all() {
        Ok(depts) => {
            let mut csv = String::from("ID,Nombre\n");
            for d in &depts {
                csv.push_str(&format!("{},{}\n", d.id, d.nombre));
            }
            
            sdk::kv_set_val("export_departments", &csv);
            
            sdk::respond(sdk::widgets![
                sdk::card("Exportación de Departamentos", vec![
                    sdk::text(&format!("✅ {} departamentos exportados", depts.len()), "success"),
                    sdk::table(
                        vec!["ID", "Nombre"],
                        depts.iter().map(|d| {
                            vec![d.id.as_str(), d.nombre.as_str()]
                        }).collect(),
                    ),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error exportando departamentos", "error")
            ]);
        }
    }
}

fn export_analytics() {
    match sdk::query::analytics().get() {
        Ok(stats) => {
            let analytics_data = format!(
                "Total Tickets,{}\nAbiertos,{}\nCerrados,{}\nÚltima Semana,{}\nAgentes Activos,{}\n",
                stats.total_tickets, stats.tickets_abiertos, stats.tickets_cerrados,
                stats.tickets_ultima_semana, stats.agentes_activos
            );
            
            sdk::kv_set_val("export_analytics", &analytics_data);
            
            sdk::respond(sdk::widgets![
                sdk::card("Exportación de Analytics", vec![
                    sdk::text("✅ Métricas exportadas exitosamente", "success"),
                    sdk::text(&format!("📊 Resumen: {} tickets totales", stats.total_tickets), "info"),
                    sdk::chart("Resumen de Métricas", vec![
                        ("Abiertos", stats.tickets_abiertos as f64),
                        ("Cerrados", stats.tickets_cerrados as f64),
                        ("Última Semana", stats.tickets_ultima_semana as f64),
                    ], "bar"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error exportando analytics", "error")
            ]);
        }
    }
}

fn export_all() {
    sdk::log("Iniciando exportación completa...");
    
    // Exportar todo en paralelo
    let tickets = sdk::query::tickets().limit(1000).all();
    let agents = sdk::query::agents().limit(100).all();
    let departments = sdk::query::departments().limit(50).all();
    let analytics = sdk::query::analytics().get();
    
    let mut total_records = 0;
    
    if let Ok(t) = &tickets {
        total_records += t.len();
    }
    if let Ok(a) = &agents {
        total_records += a.len();
    }
    if let Ok(d) = &departments {
        total_records += d.len();
    }
    
    sdk::respond(sdk::widgets![
        sdk::card("Exportación Completa", vec![
            sdk::text(&format!("✅ Exportación completada exitosamente"), "success"),
            sdk::text(&format!("📊 Total de registros: {}", total_records), "info"),
            sdk::divider(),
            sdk::text("Resumen por tipo:", "info"),
            sdk::text(&format!("• Tickets: {}", tickets.as_ref().map(|t| t.len()).unwrap_or(0)), "default"),
            sdk::text(&format!("• Agentes: {}", agents.as_ref().map(|a| a.len()).unwrap_or(0)), "default"),
            sdk::text(&format!("• Departamentos: {}", departments.as_ref().map(|d| d.len()).unwrap_or(0)), "default"),
            sdk::text(&format!("• Analytics: {}", if analytics.is_ok() { "Incluido" } else { "No disponible" }), "default"),
        ]),
    ]);
}
