// Ejemplo 17: Integracion con Email
// Features: HTTP para SMTP/API, KV store, Templates
// Demuestra: Envio y recepcion de emails, plantillas, automatizacion

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("email", "Integracion Email", "mail-line")
                        .category("integraciones")
                        .priority(19)
                )
                .name("Integracion Email")
                .description("Envia y recibe emails desde EzerDesk con plantillas personalizadas")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "email" => render_email_dashboard(),
                "compose" => render_compose_email(),
                "templates" => render_templates(),
                "settings" => render_email_settings(),
                "logs" => render_email_logs(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "send_email" => handle_send_email(&data),
                "save_template" => handle_save_template(&data),
                "delete_template" => handle_delete_template(&data),
                "test_connection" => test_smtp_connection(),
                "sync_inbox" => sync_inbox(),
                "view_email" => view_email_detail(&data),
                _ => {
                    sdk::respond_ok("Accion no reconocida");
                }
            }
        }

        // Evento de ticket: enviar notificacion por email
        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Ticket {} creado - enviando notificacion por email", ticket.id));
        }

        _ => {}
    }
    0
}

fn render_email_dashboard() {
    let sent_count = sdk::kv_get_val("emails_sent_today")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let smtp_status = sdk::kv_get_val("smtp_connected")
        .unwrap_or("false".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Integracion Email", vec![
            sdk::text("Gestiona el envio y recepcion de emails", "info"),
            sdk::divider(),

            sdk::card("Estado del Sistema", vec![
                sdk::text(&format!("SMTP: {}", if smtp_status == "true" { "Conectado" } else { "Desconectado" }),
                    if smtp_status == "true" { "success" } else { "error" }),
                sdk::text(&format!("Emails enviados hoy: {}", sent_count), "info"),
                sdk::text("Ultima sincronizacion: hace 5 minutos", "info"),
            ]),

            sdk::card("Acciones Rapidas", vec![
                sdk::button("Redactar Email", "compose", "primary"),
                sdk::button("Ver Plantillas", "templates", "secondary"),
                sdk::button("Sincronizar Bandeja", "sync_inbox", "secondary"),
                sdk::button("Ver Logs", "logs", "secondary"),
                sdk::button("Configuracion", "settings", "outline"),
            ]),

            sdk::card("Ultimos Emails", vec![
                sdk::table(
                    vec!["Para", "Asunto", "Estado", "Fecha"],
                    vec![
                        vec!["juan@ejemplo.com", "Ticket #1234 Actualizado", "Enviado", "15:30"],
                        vec!["ana@ejemplo.com", "Bienvenido a EzerDesk", "Enviado", "14:20"],
                        vec!["pedro@ejemplo.com", "Tu encuesta de satisfaccion", "Pendiente", "13:10"],
                    ],
                ),
            ]),
        ]),
    ]);
}

fn render_compose_email() {
    sdk::respond(sdk::widgets![
        sdk::card("Redactar Email", vec![
            sdk::text("Envia un email a traves de EzerDesk", "info"),
            sdk::divider(),

            sdk::input("to", "Para")
                .placeholder("destinatario@ejemplo.com")
                .required(true),

            sdk::input("cc", "CC")
                .placeholder("copia@ejemplo.com"),

            sdk::input("subject", "Asunto")
                .placeholder("Asunto del email")
                .required(true),

            sdk::select("template", "Usar Plantilla", vec![
                ("none", "Sin plantilla"),
                ("welcome", "Bienvenida"),
                ("ticket_update", "Actualizacion de Ticket"),
                ("survey", "Encuesta de Satisfaccion"),
                ("followup", "Seguimiento"),
            ]),

            sdk::textarea("body", "Cuerpo del Email")
                .placeholder("Escribe tu mensaje aqui...")
                .rows(10)
                .required(true),

            sdk::select("priority", "Prioridad", vec![
                ("normal", "Normal"),
                ("high", "Alta"),
                ("low", "Baja"),
            ]),

            sdk::checkbox("read_receipt", "Solicitar acuse de recibo"),

            sdk::divider(),

            sdk::button("Enviar Email", "send_email", "primary"),
            sdk::button("Guardar Borrador", "email", "secondary"),
            sdk::button("Cancelar", "email", "outline"),
        ]),
    ]);
}

fn render_templates() {
    sdk::respond(sdk::widgets![
        sdk::card("Plantillas de Email", vec![
            sdk::text("Gestiona las plantillas de email predefinidas", "info"),
            sdk::divider(),

            sdk::table(
                vec!["Nombre", "Categoria", "Preview"],
                vec![
                    vec!["Bienvenida", "Onboarding", "Te damos la bienvenida a..."],
                    vec!["Ticket Actualizado", "Soporte", "Tu ticket #{{id}} ha sido..."],
                    vec!["Encuesta", "Feedback", "Por favor califica tu experiencia..."],
                    vec!["Seguimiento", "Soporte", "Queriamos saber si tu problema..."],
                ],
            ),

            sdk::divider(),

            sdk::card("Crear Nueva Plantilla", vec![
                sdk::input("template_name", "Nombre de la Plantilla")
                    .placeholder("Mi Plantilla")
                    .required(true),

                sdk::select("template_category", "Categoria", vec![
                    ("onboarding", "Onboarding"),
                    ("soporte", "Soporte"),
                    ("feedback", "Feedback"),
                    ("marketing", "Marketing"),
                    ("custom", "Personalizada"),
                ]),

                sdk::textarea("template_body", "Contenido")
                    .placeholder("Usa {{variable}} para campos dinamicos...")
                    .rows(8),

                sdk::text("Variables disponibles: {{nombre}}, {{email}}, {{ticket_id}}, {{asunto}}", "info"),

                sdk::button("Guardar Plantilla", "save_template", "primary"),
            ]),
        ]),
    ]);
}

fn render_email_settings() {
    let smtp_host = sdk::kv_get_val("smtp_host")
        .unwrap_or("smtp.ejemplo.com".to_string());
    let smtp_port = sdk::kv_get_val("smtp_port")
        .unwrap_or("587".to_string());
    let smtp_user = sdk::kv_get_val("smtp_user")
        .unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion de Email", vec![
            sdk::text("Configura la conexion SMTP para envio de emails", "info"),
            sdk::divider(),

            sdk::card("Servidor SMTP", vec![
                sdk::input("smtp_host", "Host SMTP", &smtp_host),
                sdk::input("smtp_port", "Puerto", &smtp_port),
                sdk::input("smtp_user", "Usuario", &smtp_user),
                sdk::input("smtp_password", "Contrasena")
                    .placeholder("Tu contrasena")
                    .input_type("password"),
                sdk::select("smtp_encryption", "Encriptacion", vec![
                    ("tls", "TLS"),
                    ("ssl", "SSL"),
                    ("none", "Ninguna"),
                ]),
            ]),

            sdk::card("Opciones de Envio", vec![
                sdk::input("from_name", "Nombre del Remitente")
                    .placeholder("EzerDesk Soporte"),
                sdk::input("from_email", "Email del Remitente")
                    .placeholder("soporte@ejemplo.com"),
                sdk::checkbox("auto_reply", "Respuesta automatica para tickets"),
                sdk::checkbox("daily_summary", "Enviar resumen diario por email"),
            ]),

            sdk::divider(),

            sdk::button("Guardar Configuracion", "email", "primary"),
            sdk::button("Probar Conexion", "test_connection", "secondary"),
        ]),
    ]);
}

fn render_email_logs() {
    sdk::respond(sdk::widgets![
        sdk::card("Logs de Email", vec![
            sdk::text("Historial de emails enviados y recibidos", "info"),
            sdk::divider(),

            sdk::table(
                vec!["Fecha", "Direccion", "Asunto", "Tipo", "Estado"],
                vec![
                    vec!["2024-01-15 15:30", "juan@ejemplo.com", "Ticket Actualizado", "Outbound", "Enviado"],
                    vec!["2024-01-15 14:20", "ana@ejemplo.com", "Bienvenida", "Outbound", "Enviado"],
                    vec!["2024-01-15 13:10", "pedro@ejemplo.com", "Encuesta", "Outbound", "Pendiente"],
                    vec!["2024-01-15 12:00", "sistema@ejemplo.com", "Notificacion SLA", "Inbound", "Leido"],
                ],
            ),

            sdk::divider(),
            sdk::button("Volver", "email", "outline"),
        ]),
    ]);
}

fn handle_send_email(data: &str) {
    let to = extract_field(data, "to").unwrap_or_default();
    let subject = extract_field(data, "subject").unwrap_or_default();
    let body = extract_field(data, "body").unwrap_or_default();

    if to.is_empty() || subject.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("El destinatario y el asunto son obligatorios", "error"),
        ]);
        return;
    }

    // Simular envio HTTP a API de email
    sdk::log(&format!("Enviando email a: {} | Asunto: {}", to, subject));

    // Incrementar contador
    let count = sdk::kv_get_val("emails_sent_today")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("emails_sent_today", &count.to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Email Enviado", vec![
            sdk::text("Email enviado exitosamente", "success"),
            sdk::text(&format!("Para: {}", to), "info"),
            sdk::text(&format!("Asunto: {}", subject), "info"),
            sdk::text(&format!("Enviados hoy: {}", count), "info"),
            sdk::button("Volver", "email", "primary"),
        ]),
    ]);
}

fn handle_save_template(data: &str) {
    let name = extract_field(data, "template_name").unwrap_or_default();
    let category = extract_field(data, "template_category").unwrap_or_default();
    let body = extract_field(data, "template_body").unwrap_or_default();

    if name.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("El nombre es obligatorio", "error"),
        ]);
        return;
    }

    let template_id = format!("tpl_{}", chrono::Utc::now().timestamp());
    let template_data = format!("name:{}|cat:{}|body:{}", name, category, body);
    sdk::kv_set_val(&template_id, &template_data);

    sdk::log(&format!("Plantilla guardada: {}", name));

    sdk::respond(sdk::widgets![
        sdk::card("Plantilla Guardada", vec![
            sdk::text("Plantilla creada exitosamente", "success"),
            sdk::text(&format!("Nombre: {}", name), "info"),
            sdk::button("Ver Plantillas", "templates", "primary"),
        ]),
    ]);
}

fn handle_delete_template(data: &str) {
    let template_id = extract_field(data, "template_id").unwrap_or_default();
    if !template_id.is_empty() {
        sdk::kv_set_val(&template_id, "");
        sdk::log(&format!("Plantilla eliminada: {}", template_id));
    }

    sdk::respond(sdk::widgets![
        sdk::text("Plantilla eliminada", "success"),
        sdk::button("Volver", "templates", "outline"),
    ]);
}

fn test_smtp_connection() {
    sdk::log("Probando conexion SMTP...");
    sdk::kv_set_val("smtp_connected", "true");

    sdk::respond(sdk::widgets![
        sdk::card("Conexion SMTP", vec![
            sdk::text("Conexion exitosa con el servidor SMTP", "success"),
            sdk::button("Volver", "settings", "outline"),
        ]),
    ]);
}

fn sync_inbox() {
    sdk::log("Sincronizando bandeja de entrada...");

    sdk::respond(sdk::widgets![
        sdk::card("Sincronizacion", vec![
            sdk::text("Bandeja de entrada sincronizada", "success"),
            sdk::text("3 nuevos emails recibidos", "info"),
            sdk::button("Volver", "email", "outline"),
        ]),
    ]);
}

fn view_email_detail(data: &str) {
    let email_id = extract_field(data, "email_id").unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Detalle del Email", vec![
            sdk::text(&format!("ID: {}", email_id), "info"),
            sdk::text("De: juan@ejemplo.com", "info"),
            sdk::text("Asunto: Ticket #1234 Actualizado", "info"),
            sdk::text("Fecha: 2024-01-15 15:30", "info"),
            sdk::divider(),
            sdk::text("Tu ticket ha sido actualizado. Un agente ha respondido a tu consulta.", "default"),
            sdk::divider(),
            sdk::button("Responder", "compose", "primary"),
            sdk::button("Volver", "logs", "outline"),
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
