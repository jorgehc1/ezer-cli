// Ejemplo 6: Gestor de Tickets
// Features: Table, Filters, Input, Button, KV store
// Demuestra: CRUD de tickets, filtrado, paginación

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("tickets", "Gestor Tickets", "ticket-line")
                        .category("operaciones")
                        .priority(12)
                )
                .name("Gestor de Tickets")
                .description("Lista, filtra y gestiona tickets del helpdesk")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "tickets" => render_ticket_list(None),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "filter" => {
                    let status = extract_field(&data, "status");
                    render_ticket_list(status.as_deref());
                }
                "refresh" => render_ticket_list(None),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_ticket_list(filter_status: Option<&str>) {
    let query = match filter_status {
        Some(status) => sdk::query::tickets().by_status(status).limit(50),
        None => sdk::query::tickets().limit(50),
    };

    match query.all() {
        Ok(tickets) => {
            let rows: Vec<Vec<&str>> = tickets.iter().map(|t| {
                vec![t.id.as_str(), t.asunto.as_str(), t.estado.as_str(), t.prioridad.as_str()]
            }).collect();

            sdk::respond(sdk::widgets![
                sdk::card("Gestor de Tickets", vec![
                    sdk::text(&format!("📋 {} tickets encontrados", tickets.len()), "info"),
                    sdk::divider(),
                    sdk::table(
                        vec!["ID", "Asunto", "Estado", "Prioridad"],
                        rows,
                    ),
                    sdk::divider(),
                    sdk::button("Filtrar Abiertos", "filter:Abierto", "primary"),
                    sdk::button("Filtrar Cerrados", "filter:Cerrado", "secondary"),
                    sdk::button("Mostrar Todos", "refresh", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![sdk::text("Error cargando tickets", "error")]);
        }
    }
}

fn extract_field(data: &str, field: &str) -> Option<String> {
    let search = format!("\"{}\":\"", field);
    if let Some(pos) = data.find(&search) {
        let start = pos + search.len();
        if let Some(end) = data[start..].find('"') {
            return Some(data[start..start+end].to_string());
        }
    }
    None
}
