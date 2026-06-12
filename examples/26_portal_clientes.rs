// Ejemplo 26: Portal de Clientes
// Features: Auth, Queries, Forms, Real-time, Tables, KV store
// Demuestra: Experiencia de cliente completa, self-service

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("client_portal", "Portal Clientes", "user-line")
                        .category("sistema")
                        .priority(22)
                )
                .name("Portal de Clientes")
                .description("Portal self-service para clientes del helpdesk")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "client_portal" => render_portal_dashboard(),
                "my_tickets" => render_my_tickets(),
                "new_ticket" => render_new_ticket_form(),
                "knowledge" => render_knowledge_base(),
                "subscription" => render_subscription_info(),
                "profile" => render_profile(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "create_ticket" => create_ticket(&data),
                "update_ticket" => update_ticket(&data),
                "close_ticket" => close_ticket(&data),
                "add_comment" => add_comment(&data),
                "search_knowledge" => search_knowledge(&data),
                "update_profile" => update_profile(&data),
                "request_callback" => request_callback(&data),
                "rate_service" => rate_service(&data),
                _ => {}
            }
        }

        // Notificar al cliente cuando se actualiza su ticket
        PluginEvent::TicketStatusChanged(ticket) => {
            notify_client_ticket_update(&ticket);
        }

        _ => {}
    }
    0
}

fn render_portal_dashboard() {
    sdk::respond(sdk::widgets![
        sdk::card("Portal de Clientes", vec![
            sdk::text("Bienvenido a tu portal de soporte", "info"),
            sdk::divider(),
            sdk::text("📊 Resumen de tu cuenta:", "info"),
            sdk::text("• Tickets abiertos: 3", "default"),
            sdk::text("• Tickets resueltos este mes: 12", "default"),
            sdk::text("• Tiempo promedio de respuesta: 2 horas", "default"),
        ]),

        sdk::card("Acciones Rápidas", vec![
            sdk::button("Abrir Ticket", "new_ticket", "primary"),
            sdk::button("Ver Mis Tickets", "my_tickets", "secondary"),
            sdk::button("Base de Conocimiento", "knowledge", "outline"),
        ]),

        sdk::card("Tickets Recientes", vec![
            sdk::table(
                vec!["ID", "Asunto", "Estado", "Última Actualización"],
                vec![
                    vec!["T-123", "Error al iniciar sesión", "Abierto", "Hace 2 horas"],
                    vec!["T-122", "Solicitud de funcionalidad", "En progreso", "Ayer"],
                    vec!["T-121", "Consulta sobre facturación", "Cerrado", "Hace 3 días"],
                ],
            ),
        ]),

        sdk::card("Soporte", vec![
            sdk::text("¿Necesitas ayuda?", "info"),
            sdk::button("Solicitar Callback", "request_callback", "secondary"),
            sdk::button("Calificar Servicio", "rate_service", "outline"),
        ]),
    ]);
}

fn render_my_tickets() {
    sdk::respond(sdk::widgets![
        sdk::card("Mis Tickets", vec![
            sdk::text("Gestiona tus tickets de soporte", "info"),
            sdk::table(
                vec!["ID", "Asunto", "Estado", "Prioridad", "Creado", "Última Actualización"],
                vec![
                    vec!["T-123", "Error al iniciar sesión", "Abierto", "Alta", "2024-01-15", "Hace 2 horas"],
                    vec!["T-122", "Solicitud de funcionalidad", "En progreso", "Media", "2024-01-14", "Ayer"],
                    vec!["T-121", "Consulta sobre facturación", "Cerrado", "Baja", "2024-01-10", "Hace 3 días"],
                ],
            ),
            sdk::button("Abrir Nuevo Ticket", "new_ticket", "primary"),
        ]),
    ]);
}

fn render_new_ticket_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Nuevo Ticket de Soporte", vec![
            sdk::input("Asunto", "subject", "Describe brevemente tu problema"),
            sdk::select_widget("Categoría", "category", vec![
                ("technical".to_string(), "Soporte Técnico".to_string()),
                ("billing".to_string(), "Facturación".to_string()),
                ("feature".to_string(), "Solicitud de Funcionalidad".to_string()),
                ("bug".to_string(), "Reporte de Bug".to_string()),
                ("other".to_string(), "Otro".to_string()),
            ], "technical".to_string()),
            sdk::select_widget("Prioridad", "priority", vec![
                ("low".to_string(), "Baja".to_string()),
                ("medium".to_string(), "Media".to_string()),
                ("high".to_string(), "Alta".to_string()),
                ("urgent".to_string(), "Urgente".to_string()),
            ], "medium".to_string()),
            sdk::textarea("Descripción", "description", "Describe tu problema en detalle..."),
            sdk::input("Adjuntos (URLs)", "attachments", "https://ejemplo.com/imagen.png"),
            sdk::button("Enviar Ticket", "create_ticket", "primary"),
            sdk::button("Cancelar", "client_portal", "outline"),
        ]),
    ]);
}

fn render_knowledge_base() {
    sdk::respond(sdk::widgets![
        sdk::card("Base de Conocimiento", vec![
            sdk::text("Busca respuestas en nuestra base de conocimiento", "info"),
            sdk::input("Buscar", "search_query", "Escribe tu búsqueda..."),
            sdk::button("Buscar", "search_knowledge", "primary"),
        ]),

        sdk::card("Artículos Populares", vec![
            sdk::table(
                vec!["Título", "Categoría", "Vistas", "Útil"],
                vec![
                    vec!["Cómo cambiar tu contraseña", "Cuenta", "1,234", "95%"],
                    vec!["Guía de facturación", "Facturación", "892", "88%"],
                    vec!["Configurar notificaciones", "Configuración", "654", "92%"],
                ],
            ),
        ]),

        sdk::card("Categorías", vec![
            sdk::button("Guías de Inicio", "search:guia", "secondary"),
            sdk::button("Preguntas Frecuentes", "search:faq", "secondary"),
            sdk::button("Tutoriales", "search:tutorial", "secondary"),
        ]),
    ]);
}

fn render_subscription_info() {
    sdk::respond(sdk::widgets![
        sdk::card("Tu Suscripción", vec![
            sdk::text("Plan: Pro", "info"),
            sdk::text("Estado: Activo", "success"),
            sdk::text("Próximo pago: 15 de Febrero 2024", "default"),
            sdk::text("Monto: $29.99/mes", "default"),
            sdk::divider(),
            sdk::text("Uso actual:", "info"),
            sdk::text("• Tickets: 45/100 (45%)", "default"),
            sdk::text("• Almacenamiento: 2.5GB/10GB (25%)", "default"),
            sdk::text("• Agentes: 3/5 (60%)", "default"),
            sdk::divider(),
            sdk::button("Actualizar Plan", "update_plan", "primary"),
            sdk::button("Ver Facturas", "invoices", "outline"),
        ]),
    ]);
}

fn render_profile() {
    sdk::respond(sdk::widgets![
        sdk::card("Mi Perfil", vec![
            sdk::input("Nombre", "name", "Juan Pérez"),
            sdk::input("Email", "email", "juan@empresa.com"),
            sdk::input("Teléfono", "phone", "+593 99 123 4567"),
            sdk::input("Empresa", "company", "Acme Corp"),
            sdk::input("Cargo", "position", "Gerente de TI"),
            sdk::divider(),
            sdk::switch_widget("Notificaciones por Email", "email_notifications", true),
            sdk::switch_widget("Notificaciones por SMS", "sms_notifications", false),
            sdk::button("Guardar Cambios", "update_profile", "primary"),
        ]),
    ]);
}

fn create_ticket(data: &str) {
    sdk::log(&format!("Creando ticket: {}", data));
    sdk::respond_ok("Ticket creado exitosamente. ID: T-124");
}

fn update_ticket(data: &str) {
    sdk::log(&format!("Actualizando ticket: {}", data));
    sdk::respond_ok("Ticket actualizado");
}

fn close_ticket(data: &str) {
    sdk::log(&format!("Cerrando ticket: {}", data));
    sdk::respond_ok("Ticket cerrado");
}

fn add_comment(data: &str) {
    sdk::log(&format!("Agregando comentario: {}", data));
    sdk::respond_ok("Comentario agregado");
}

fn search_knowledge(data: &str) {
    sdk::log &format!("Buscando en knowledge base: {}", data));
    sdk::respond_ok("Búsqueda completada");
}

fn update_profile(data: &str) {
    sdk::log(&format!("Actualizando perfil: {}", data));
    sdk::respond_ok("Perfil actualizado");
}

fn request_callback(data: &str) {
    sdk::log(&format!("Solicitando callback: {}", data));
    sdk::respond_ok("Callback solicitado. Te contactaremos pronto.");
}

fn rate_service(data: &str) {
    sdk::log(&format!("Calificando servicio: {}", data));
    sdk::respond_ok("Gracias por tu calificación");
}

fn notify_client_ticket_update(ticket: &sdk::Ticket) {
    sdk::log(&format!("Notificando actualización de ticket: {}", ticket.id));
}
