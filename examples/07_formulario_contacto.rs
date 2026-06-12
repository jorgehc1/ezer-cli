// Ejemplo 7: Formulario de Contacto
// Features: Input, Select, Textarea, Button, NumberInput, DateInput, KV store
// Demuestra: Formularios completos, validación, persistencia de datos

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("contacto", "Formulario Contacto", "user-line")
                        .category("herramientas")
                        .priority(15)
                )
                .name("Formulario de Contacto")
                .description("Formulario completo con múltiples tipos de campo y validación")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "contacto" => render_contact_form(),
                "contacto_exito" => render_success(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "submit_contact" => handle_submit(&data),
                "reset_form" => render_contact_form(),
                "view_contacts" => render_contacts_list(),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        // Guardar contacto cuando se crea un ticket relacionado
        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Ticket de contacto creado: {}", ticket.id));
            let count = sdk::kv_get_val("contact_form_count")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0) + 1;
            sdk::kv_set_val("contact_form_count", &count.to_string());
        }

        _ => {}
    }
    0
}

// Renderiza el formulario de contacto con todos los tipos de campo
fn render_contact_form() {
    // Cargar datos previos del KV si existen (para edición)
    let saved_name = sdk::kv_get_val("form_name").unwrap_or_default();
    let saved_email = sdk::kv_get_val("form_email").unwrap_or_default();

    sdk::respond(sdk::widgets![
        sdk::card("Formulario de Contacto", vec![
            sdk::text("Completa todos los campos para registrar un nuevo contacto", "info"),
            sdk::divider(),

            // Campo de texto: Nombre completo
            sdk::input("nombre", "Nombre Completo", &saved_name)
                .placeholder("Juan Pérez")
                .required(true),

            // Campo de email
            sdk::input("email", "Correo Electrónico", &saved_email)
                .placeholder("juan@ejemplo.com")
                .input_type("email")
                .required(true),

            // Campo numérico: Teléfono
            sdk::number_input("telefono", "Teléfono")
                .min(1000000)
                .max(9999999999)
                .placeholder("1234567890"),

            // Selector de tipo de contacto
            sdk::select("tipo_contacto", "Tipo de Contacto", vec![
                ("cliente", "Cliente"),
                ("proveedor", "Proveedor"),
                ("partner", "Socio Comercial"),
                ("otro", "Otro"),
            ]),

            // Selector de departamento de interés
            sdk::select("departamento", "Departamento de Interés", vec![
                ("ventas", "Ventas"),
                ("soporte", "Soporte Técnico"),
                ("facturacion", "Facturación"),
                ("general", "Información General"),
            ]),

            // Selector de prioridad
            sdk::select("prioridad", "Prioridad", vec![
                ("baja", "Baja"),
                ("media", "Media"),
                ("alta", "Alta"),
                ("urgente", "Urgente"),
            ]),

            // Campo de fecha: Fecha de nacimiento
            sdk::date_input("fecha_nacimiento", "Fecha de Nacimiento"),

            // Selector de fecha de contacto preferida
            sdk::date_input("fecha_contacto", "Fecha Preferida de Contacto"),

            // Área de texto: Observaciones
            sdk::textarea("observaciones", "Observaciones")
                .placeholder("Escribe tus observaciones aquí...")
                .rows(5),

            // Checkbox de consentimiento
            sdk::checkbox("consentimiento", "Acepto la política de privacidad y el tratamiento de mis datos"),

            sdk::divider(),

            // Botones de acción
            sdk::button("Enviar Contacto", "submit_contact", "primary"),
            sdk::button("Limpiar Formulario", "reset_form", "outline"),
            sdk::button("Ver Contactos Guardados", "view_contacts", "secondary"),
        ]),
    ]);
}

// Procesa el envío del formulario
fn handle_submit(data: &str) {
    // Extraer datos del formulario
    let nombre = extract_field(data, "nombre").unwrap_or_default();
    let email = extract_field(data, "email").unwrap_or_default();
    let telefono = extract_field(data, "telefono").unwrap_or_default();
    let tipo = extract_field(data, "tipo_contacto").unwrap_or_default();
    let departamento = extract_field(data, "departamento").unwrap_or_default();
    let prioridad = extract_field(data, "prioridad").unwrap_or_default();
    let observaciones = extract_field(data, "observaciones").unwrap_or_default();

    // Validación básica
    if nombre.is_empty() || email.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::card("Error de Validación", vec![
                sdk::text("❌ El nombre y el correo electrónico son obligatorios", "error"),
                sdk::button("Volver al Formulario", "reset_form", "primary"),
            ]),
        ]);
        return;
    }

    // Guardar en KV store
    let contact_id = format!("contact_{}", chrono::Utc::now().timestamp());
    let contact_data = format!(
        "nombre:{}|email:{}|telefono:{}|tipo:{}|depto:{}|prioridad:{}|obs:{}",
        nombre, email, telefono, tipo, departamento, prioridad, observaciones
    );

    sdk::kv_set_val(&contact_id, &contact_data);
    sdk::kv_set_val("form_name", &nombre);
    sdk::kv_set_val("form_email", &email);

    // Incrementar contador
    let count = sdk::kv_get_val("contact_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("contact_count", &count.to_string());

    sdk::log(&format!("Contacto registrado: {} ({})", nombre, email));

    // Mostrar éxito
    sdk::respond(sdk::widgets![
        sdk::card("Contacto Registrado", vec![
            sdk::text("✅ Contacto registrado exitosamente", "success"),
            sdk::divider(),
            sdk::text(&format!("👤 Nombre: {}", nombre), "info"),
            sdk::text(&format!("📧 Email: {}", email), "info"),
            sdk::text(&format!("📞 Teléfono: {}", telefono), "info"),
            sdk::text(&format!("🏷️ Tipo: {}", tipo), "info"),
            sdk::text(&format!("🏢 Departamento: {}", departamento), "info"),
            sdk::text(&format!("⚡ Prioridad: {}", prioridad), "info"),
            sdk::text(&format!("📝 Observaciones: {}", observaciones), "default"),
            sdk::divider(),
            sdk::text(&format!("Total contactos registrados: {}", count), "info"),
            sdk::button("Registrar Otro Contacto", "reset_form", "primary"),
            sdk::button("Ver Todos los Contactos", "view_contacts", "secondary"),
        ]),
    ]);
}

// Muestra la lista de contactos guardados
fn render_contacts_list() {
    let count = sdk::kv_get_val("contact_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Contactos Registrados", vec![
            sdk::text(&format!("📋 Total de contactos: {}", count), "info"),
            sdk::divider(),
            sdk::text("Los contactos se almacenan en el KV store del plugin", "info"),
            sdk::text("Cada contacto incluye: nombre, email, teléfono, tipo, departamento, prioridad", "default"),
            sdk::divider(),
            sdk::button("Registrar Nuevo Contacto", "reset_form", "primary"),
        ]),
    ]);
}

// Muestra pantalla de éxito
fn render_success() {
    sdk::respond(sdk::widgets![
        sdk::card("Éxito", vec![
            sdk::text("✅ Operación completada exitosamente", "success"),
            sdk::button("Volver al Formulario", "reset_form", "primary"),
        ]),
    ]);
}

// Helper para extraer campos del JSON de datos
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
