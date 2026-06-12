// Ejemplo 8: Buscador Avanzado
// Features: Búsqueda multi-entidad, filtros combinados, tabla de resultados
// Demuestra: Queries combinadas de tickets, agentes y knowledge base

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("buscador", "Buscador Avanzado", "search-line")
                        .category("herramientas")
                        .priority(10)
                )
                .name("Buscador Avanzado")
                .description("Búsqueda global en tickets, agentes y base de conocimiento")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "buscador" => render_search_page(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "search" => handle_search(&data),
                "search_tickets" => search_tickets_only(&data),
                "search_agents" => search_agents_only(&data),
                "search_knowledge" => search_knowledge_only(&data),
                "clear" => render_search_page(),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

// Renderiza la página de búsqueda con filtros
fn render_search_page() {
    sdk::respond(sdk::widgets![
        sdk::card("Búsqueda Avanzada", vec![
            sdk::text("Busca en tickets, agentes y base de conocimiento", "info"),
            sdk::divider(),

            // Campo de búsqueda principal
            sdk::input("query", "Término de Búsqueda")
                .placeholder("Buscar tickets, agentes, artículos...")
                .required(true),

            sdk::divider(),

            // Filtros de búsqueda
            sdk::card("Filtros", vec![
                // Tipo de entidad a buscar
                sdk::select("entity", "Buscar en", vec![
                    ("all", "Todo"),
                    ("tickets", "Solo Tickets"),
                    ("agents", "Solo Agentes"),
                    ("knowledge", "Solo Conocimiento"),
                ]),

                // Filtro de estado (para tickets)
                sdk::select("status", "Estado del Ticket", vec![
                    ("all", "Todos los estados"),
                    ("Abierto", "Abierto"),
                    ("En Progreso", "En Progreso"),
                    ("Cerrado", "Cerrado"),
                ]),

                // Filtro de prioridad
                sdk::select("priority", "Prioridad", vec![
                    ("all", "Todas"),
                    ("Alta", "Alta"),
                    ("Media", "Media"),
                    ("Baja", "Baja"),
                ]),

                // Filtro de fecha desde
                sdk::date_input("date_from", "Desde"),

                // Filtro de fecha hasta
                sdk::date_input("date_to", "Hasta"),
            ]),

            sdk::divider(),

            // Botones de búsqueda
            sdk::button("Buscar en Todo", "search", "primary"),
            sdk::button("Buscar Tickets", "search_tickets", "secondary"),
            sdk::button("Buscar Agentes", "search_agents", "secondary"),
            sdk::button("Buscar Conocimiento", "search_knowledge", "secondary"),
            sdk::button("Limpiar", "clear", "outline"),
        ]),
    ]);
}

// Búsqueda general en todas las entidades
fn handle_search(data: &str) {
    let query = extract_field(data, "query").unwrap_or_default();

    if query.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Ingresa un término de búsqueda", "warning"),
        ]);
        return;
    }

    // Buscar en todas las fuentes
    let tickets = sdk::query::tickets()
        .search(&query)
        .limit(20)
        .all();

    let agents = sdk::query::agents()
        .search(&query)
        .limit(20)
        .all();

    let knowledge = sdk::query::knowledge()
        .search(&query)
        .limit(20)
        .all();

    // Contar resultados
    let ticket_count = tickets.as_ref().map(|t| t.len()).unwrap_or(0);
    let agent_count = agents.as_ref().map(|a| a.len()).unwrap_or(0);
    let knowledge_count = knowledge.as_ref().map(|k| k.len()).unwrap_or(0);
    let total = ticket_count + agent_count + knowledge_count;

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Resultados para \"{}\"", query), vec![
            sdk::text(&format!("📊 {} resultados encontrados en total", total), "info"),
            sdk::text(&format!("Tickets: {} | Agentes: {} | Conocimiento: {}",
                ticket_count, agent_count, knowledge_count), "info"),
            sdk::divider(),

            // Resultados de tickets
            if ticket_count > 0 {
                sdk::card("Tickets Encontrados", vec![
                    sdk::table(
                        vec!["ID", "Asunto", "Estado", "Prioridad"],
                        tickets.unwrap_or_default().iter().map(|t| {
                            vec![t.id.as_str(), t.asunto.as_str(), t.estado.as_str(), t.prioridad.as_str()]
                        }).collect(),
                    ),
                ])
            } else {
                sdk::text("No se encontraron tickets", "default")
            },

            // Resultados de agentes
            if agent_count > 0 {
                sdk::card("Agentes Encontrados", vec![
                    sdk::table(
                        vec!["ID", "Nombre", "Email", "Departamento"],
                        agents.unwrap_or_default().iter().map(|a| {
                            vec![
                                a.id.as_str(),
                                format!("{} {}", a.nombres, a.apellidos).as_str(),
                                a.correo.as_str(),
                                a.departamento.as_str(),
                            ]
                        }).collect(),
                    ),
                ])
            } else {
                sdk::text("No se encontraron agentes", "default")
            },

            // Resultados de conocimiento
            if knowledge_count > 0 {
                sdk::card("Artículos de Conocimiento", vec![
                    sdk::table(
                        vec!["ID", "Título", "Categoría"],
                        knowledge.unwrap_or_default().iter().map(|k| {
                            vec![k.id.as_str(), k.titulo.as_str(), k.categoria.as_str()]
                        }).collect(),
                    ),
                ])
            } else {
                sdk::text("No se encontraron artículos de conocimiento", "default")
            },

            sdk::divider(),
            sdk::button("Nueva Búsqueda", "clear", "outline"),
        ]),
    ]);
}

// Búsqueda solo en tickets
fn search_tickets_only(data: &str) {
    let query = extract_field(data, "query").unwrap_or_default();
    let status = extract_field(data, "status").unwrap_or_default();
    let priority = extract_field(data, "priority").unwrap_or_default();

    let mut ticket_query = sdk::query::tickets();

    if !query.is_empty() {
        ticket_query = ticket_query.search(&query);
    }
    if status != "all" && !status.is_empty() {
        ticket_query = ticket_query.by_status(&status);
    }
    if priority != "all" && !priority.is_empty() {
        ticket_query = ticket_query.by_priority(&priority);
    }

    let tickets = ticket_query.limit(50).all();

    match tickets {
        Ok(ticket_list) => {
            sdk::respond(sdk::widgets![
                sdk::card(&format!("Tickets - {} resultados", ticket_list.len()), vec![
                    sdk::table(
                        vec!["ID", "Asunto", "Estado", "Prioridad", "Creado"],
                        ticket_list.iter().map(|t| {
                            vec![
                                t.id.as_str(),
                                t.asunto.as_str(),
                                t.estado.as_str(),
                                t.prioridad.as_str(),
                                t.creado_en.as_str(),
                            ]
                        }).collect(),
                    ),
                    sdk::button("Volver a Búsqueda", "clear", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error buscando tickets", "error"),
            ]);
        }
    }
}

// Búsqueda solo en agentes
fn search_agents_only(data: &str) {
    let query = extract_field(data, "query").unwrap_or_default();

    let agents = if query.is_empty() {
        sdk::query::agents().limit(50).all()
    } else {
        sdk::query::agents().search(&query).limit(50).all()
    };

    match agents {
        Ok(agent_list) => {
            sdk::respond(sdk::widgets![
                sdk::card(&format!("Agentes - {} resultados", agent_list.len()), vec![
                    sdk::table(
                        vec!["ID", "Nombre", "Email", "Departamento", "Rol"],
                        agent_list.iter().map(|a| {
                            vec![
                                a.id.as_str(),
                                format!("{} {}", a.nombres, a.apellidos).as_str(),
                                a.correo.as_str(),
                                a.departamento.as_str(),
                                a.rol.as_str(),
                            ]
                        }).collect(),
                    ),
                    sdk::button("Volver a Búsqueda", "clear", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error buscando agentes", "error"),
            ]);
        }
    }
}

// Búsqueda solo en conocimiento
fn search_knowledge_only(data: &str) {
    let query = extract_field(data, "query").unwrap_or_default();

    let knowledge = if query.is_empty() {
        sdk::query::knowledge().limit(50).all()
    } else {
        sdk::query::knowledge().search(&query).limit(50).all()
    };

    match knowledge {
        Ok(articles) => {
            sdk::respond(sdk::widgets![
                sdk::card(&format!("Conocimiento - {} artículos", articles.len()), vec![
                    sdk::table(
                        vec!["ID", "Título", "Categoría", "Autor", "Vistas"],
                        articles.iter().map(|k| {
                            vec![
                                k.id.as_str(),
                                k.titulo.as_str(),
                                k.categoria.as_str(),
                                k.autor.as_str(),
                                &k.vistas.to_string(),
                            ]
                        }).collect(),
                    ),
                    sdk::button("Volver a Búsqueda", "clear", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error buscando en conocimiento", "error"),
            ]);
        }
    }
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
