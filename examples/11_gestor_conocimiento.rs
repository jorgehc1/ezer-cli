// Ejemplo 11: Gestor de Conocimiento
// Features: CRUD de artículos, búsquedas, categorías, analytics
// Demuestra: Gestión de base de conocimiento, versionado, estadísticas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("conocimiento", "Base Conocimiento", "book-open-line")
                        .category("operaciones")
                        .priority(14)
                )
                .name("Gestor de Conocimiento")
                .description("Administra artículos de ayuda, FAQs y documentación")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "conocimiento" => render_knowledge_dashboard(),
                "create" => render_create_article(),
                "article" => render_article_detail(),
                "categories" => render_categories(),
                "analytics" => render_knowledge_analytics(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "search" => search_articles(&data),
                "create_article" => handle_create_article(&data),
                "edit_article" => handle_edit_article(&data),
                "delete_article" => handle_delete_article(&data),
                "view_article" => view_article(&data),
                "publish" => publish_article(&data),
                "archive" => archive_article(&data),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        // Eventos del sistema para actualizaciones
        PluginEvent::TicketCreated(ticket) => {
            // Buscar artículos relevantes para el nuevo ticket
            let relevant = sdk::query::knowledge()
                .search(&ticket.asunto)
                .limit(3)
                .all();
            if let Ok(articles) = relevant {
                sdk::log(&format!(
                    "Artículos sugeridos para ticket {}: {}",
                    ticket.id,
                    articles.len()
                ));
            }
        }

        _ => {}
    }
    0
}

// Dashboard principal de conocimiento
fn render_knowledge_dashboard() {
    let article_count = sdk::kv_get_val("article_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Base de Conocimiento", vec![
            sdk::text("Gestiona artículos, FAQs y documentación de ayuda", "info"),
            sdk::divider(),

            // Estadísticas
            sdk::card("Estadísticas", vec![
                sdk::text(&format!("📚 Artículos totales: {}", article_count), "info"),
                sdk::chart("Artículos por Categoría", vec![
                    ("Instalación", 12.0),
                    ("Configuración", 18.0),
                    ("Solución de Problemas", 25.0),
                    ("API", 8.0),
                    ("FAQ", 15.0),
                ], "bar"),
            ]),

            // Búsqueda
            sdk::card("Búsqueda Rápida", vec![
                sdk::input("search_query", "Buscar artículos")
                    .placeholder("Buscar por título, contenido o categoría..."),
                sdk::button("Buscar", "search", "primary"),
            ]),

            sdk::divider(),

            // Acciones
            sdk::card("Acciones", vec![
                sdk::button("Crear Nuevo Artículo", "create", "primary"),
                sdk::button("Ver Categorías", "categories", "secondary"),
                sdk::button("Ver Analytics", "analytics", "secondary"),
            ]),

            // Últimos artículos
            sdk::card("Últimos Artículos", vec![
                sdk::table(
                    vec!["ID", "Título", "Categoría", "Estado", "Vistas"],
                    vec![
                        vec!["1", "Guía de Instalación", "Instalación", "Publicado", "234"],
                        vec!["2", "Configuración de Email", "Configuración", "Publicado", "189"],
                        vec!["3", "Resolución de Errores", "Soporte", "Borrador", "0"],
                    ],
                ),
            ]),
        ]),
    ]);
}

// Formulario para crear un artículo
fn render_create_article() {
    sdk::respond(sdk::widgets![
        sdk::card("Crear Nuevo Artículo", vec![
            sdk::text("Completa los campos para crear un artículo de conocimiento", "info"),
            sdk::divider(),

            sdk::input("article_title", "Título del Artículo")
                .placeholder("Cómo configurar el correo electrónico")
                .required(true),

            sdk::select("category", "Categoría", vec![
                ("instalacion", "Instalación"),
                ("configuracion", "Configuración"),
                ("soporte", "Solución de Problemas"),
                ("api", "API"),
                ("faq", "Preguntas Frecuentes"),
                ("guia", "Guías"),
                ("tutorial", "Tutoriales"),
            ]),

            sdk::select("difficulty", "Nivel de Dificultad", vec![
                ("principiante", "Principiante"),
                ("intermedio", "Intermedio"),
                ("avanzado", "Avanzado"),
            ]),

            sdk::textarea("article_content", "Contenido")
                .placeholder("Escribe el contenido del artículo aquí...")
                .rows(10)
                .required(true),

            sdk::textarea("article_summary", "Resumen")
                .placeholder("Breve resumen del artículo...")
                .rows(3),

            sdk::input("tags", "Etiquetas")
                .placeholder("email, configuración, smtp (separadas por coma)"),

            sdk::input("related_articles", "Artículos Relacionados")
                .placeholder("IDs de artículos relacionados (separados por coma)"),

            sdk::select("status", "Estado", vec![
                ("borrador", "Borrador"),
                ("revision", "En Revisión"),
                ("publicado", "Publicado"),
            ]),

            sdk::divider(),

            sdk::button("Guardar Artículo", "create_article", "primary"),
            sdk::button("Guardar como Borrador", "create_article", "secondary"),
            sdk::button("Cancelar", "conocimiento", "outline"),
        ]),
    ]);
}

// Detalle de un artículo
fn render_article_detail() {
    let article_id = sdk::kv_get_val("current_article_id")
        .unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Detalle del Artículo", vec![
            sdk::text(&format!("Artículo ID: {}", article_id), "info"),
            sdk::divider(),
            sdk::text("Título: Guía de Instalación", "info"),
            sdk::text("Categoría: Instalación", "info"),
            sdk::text("Estado: Publicado", "info"),
            sdk::text("Vistas: 234", "info"),
            sdk::divider(),
            sdk::text("Contenido del artículo aquí...", "default"),
            sdk::divider(),
            sdk::button("Editar", "edit_article", "primary"),
            sdk::button("Publicar", "publish", "secondary"),
            sdk::button("Archivar", "archive", "warning"),
            sdk::button("Eliminar", "delete_article", "danger"),
            sdk::button("Volver", "conocimiento", "outline"),
        ]),
    ]);
}

// Lista de categorías
fn render_categories() {
    sdk::respond(sdk::widgets![
        sdk::card("Categorías de Conocimiento", vec![
            sdk::table(
                vec!["Categoría", "Artículos", "Vistas Totales"],
                vec![
                    vec!["Instalación", "12", "1,234"],
                    vec!["Configuración", "18", "2,456"],
                    vec!["Solución de Problemas", "25", "3,789"],
                    vec!["API", "8", "567"],
                    vec!["FAQ", "15", "1,890"],
                    vec!["Guías", "10", "1,123"],
                    vec!["Tutoriales", "7", "890"],
                ],
            ),
            sdk::divider(),
            sdk::button("Volver al Dashboard", "conocimiento", "outline"),
        ]),
    ]);
}

// Analytics de conocimiento
fn render_knowledge_analytics() {
    sdk::respond(sdk::widgets![
        sdk::card("Analytics de Conocimiento", vec![
            sdk::chart("Vistas por Mes", vec![
                ("Ene", 1200.0),
                ("Feb", 1450.0),
                ("Mar", 1380.0),
                ("Abr", 1620.0),
                ("May", 1580.0),
            ], "line"),

            sdk::chart("Artículos Más Vistos", vec![
                ("Guía de Instalación", 234.0),
                ("Config Email", 189.0),
                ("Error 500", 156.0),
                ("API REST", 134.0),
                ("FAQ General", 123.0),
            ], "bar"),

            sdk::chart("Satisfacción de Artículos", vec![
                ("Útil", 85.0),
                ("No útil", 10.0),
                ("Parcialmente", 5.0),
            ], "pie"),

            sdk::divider(),
            sdk::button("Volver al Dashboard", "conocimiento", "outline"),
        ]),
    ]);
}

// Busca artículos
fn search_articles(data: &str) {
    let query = extract_field(data, "search_query").unwrap_or_default();

    let results = sdk::query::knowledge()
        .search(&query)
        .limit(20)
        .all();

    match results {
        Ok(articles) => {
            sdk::respond(sdk::widgets![
                sdk::card(&format!("Resultados para \"{}\"", query), vec![
                    sdk::text(&format!("📚 {} artículos encontrados", articles.len()), "info"),
                    sdk::table(
                        vec!["ID", "Título", "Categoría", "Vistas"],
                        articles.iter().map(|a| {
                            vec![
                                a.id.as_str(),
                                a.titulo.as_str(),
                                a.categoria.as_str(),
                                &a.vistas.to_string(),
                            ]
                        }).collect(),
                    ),
                    sdk::divider(),
                    sdk::button("Volver", "conocimiento", "outline"),
                ]),
            ]);
        }
        Err(_) => {
            sdk::respond(sdk::widgets![
                sdk::text("Error buscando artículos", "error"),
            ]);
        }
    }
}

// Crea un artículo
fn handle_create_article(data: &str) {
    let title = extract_field(data, "article_title").unwrap_or_default();
    let category = extract_field(data, "category").unwrap_or_default();
    let content = extract_field(data, "article_content").unwrap_or_default();
    let summary = extract_field(data, "article_summary").unwrap_or_default();
    let tags = extract_field(data, "tags").unwrap_or_default();
    let status = extract_field(data, "status").unwrap_or("borrador".to_string());

    if title.is_empty() || content.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Título y contenido son obligatorios", "warning"),
        ]);
        return;
    }

    // Guardar artículo
    let article_id = format!("article_{}", chrono::Utc::now().timestamp());
    let article_data = format!(
        "title:{}|cat:{}|content:{}|summary:{}|tags:{}|status:{}|views:0",
        title, category, content, summary, tags, status
    );

    sdk::kv_set_val(&article_id, &article_data);

    let count = sdk::kv_get_val("article_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("article_count", &count.to_string());

    sdk::log(&format!("Artículo creado: {} ({})", title, article_id));

    sdk::respond(sdk::widgets![
        sdk::card("Artículo Creado", vec![
            sdk::text("✅ Artículo creado exitosamente", "success"),
            sdk::text(&format!("📋 Título: {}", title), "info"),
            sdk::text(&format!("📂 Categoría: {}", category), "info"),
            sdk::text(&format!("📝 Estado: {}", status), "info"),
            sdk::button("Ver Artículo", "conocimiento", "primary"),
        ]),
    ]);
}

// Edita un artículo
fn handle_edit_article(data: &str) {
    let article_id = extract_field(data, "article_id").unwrap_or_default();
    sdk::log(&format!("Editando artículo: {}", article_id));

    sdk::respond(sdk::widgets![
        sdk::card("Editar Artículo", vec![
            sdk::text("Modifica los campos del artículo", "info"),
            sdk::input("article_title", "Título", "Título actualizado"),
            sdk::textarea("article_content", "Contenido", "Contenido actualizado"),
            sdk::button("Guardar Cambios", "conocimiento", "primary"),
        ]),
    ]);
}

// Elimina un artículo
fn handle_delete_article(data: &str) {
    let article_id = extract_field(data, "article_id").unwrap_or_default();
    if !article_id.is_empty() {
        sdk::kv_set_val(&article_id, "");
        sdk::log(&format!("Artículo eliminado: {}", article_id));
    }

    sdk::respond(sdk::widgets![
        sdk::card("Artículo Eliminado", vec![
            sdk::text("✅ Artículo eliminado exitosamente", "success"),
            sdk::button("Volver al Dashboard", "conocimiento", "primary"),
        ]),
    ]);
}

// Visualiza un artículo
fn view_article(data: &str) {
    let article_id = extract_field(data, "article_id").unwrap_or_default();
    sdk::kv_set_val("current_article_id", &article_id);
    render_article_detail();
}

// Publica un artículo
fn publish_article(data: &str) {
    let article_id = extract_field(data, "article_id").unwrap_or_default();
    sdk::log(&format!("Artículo publicado: {}", article_id));

    sdk::respond(sdk::widgets![
        sdk::card("Artículo Publicado", vec![
            sdk::text("✅ Artículo publicado exitosamente", "success"),
            sdk::button("Volver al Dashboard", "conocimiento", "primary"),
        ]),
    ]);
}

// Archiva un artículo
fn archive_article(data: &str) {
    let article_id = extract_field(data, "article_id").unwrap_or_default();
    sdk::log(&format!("Artículo archivado: {}", article_id));

    sdk::respond(sdk::widgets![
        sdk::card("Artículo Archivado", vec![
            sdk::text("✅ Artículo archivado exitosamente", "success"),
            sdk::button("Volver al Dashboard", "conocimiento", "primary"),
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
