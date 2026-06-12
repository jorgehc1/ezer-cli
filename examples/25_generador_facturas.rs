// Ejemplo 25: Generador de Facturas
// Features: Templates, HTTP, KV store, Email, Table, NumberInput
// Demuestra: Generación de facturas, envío por email, historial

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("invoices", "Facturas", "file-text-line")
                        .category("administracion")
                        .priority(16)
                )
                .name("Generador de Facturas")
                .description("Crea y envía facturas profesionales a clientes")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "invoices" => render_invoice_dashboard(),
                "create" => render_create_invoice(),
                "templates" => render_templates(),
                "history" => render_invoice_history(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "create_invoice" => create_invoice(&data),
                "send_invoice" => send_invoice(&data),
                "download_pdf" => download_pdf(&data),
                "duplicate_invoice" => duplicate_invoice(&data),
                "void_invoice" => void_invoice(&data),
                "add_line_item" => add_line_item(&data),
                "save_template" => save_template(&data),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_invoice_dashboard() {
    let invoice_count = sdk::kv_get_val("invoice_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let total_revenue = sdk::kv_get_val("total_revenue")
        .unwrap_or("0".to_string())
        .parse::<f64>()
        .unwrap_or(0.0);
    let pending_count = sdk::kv_get_val("pending_invoices")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Generador de Facturas", vec![
            sdk::text("Crea y gestiona facturas profesionales", "info"),
            sdk::divider(),
            sdk::text(&format!("📄 Facturas creadas: {}", invoice_count), "info"),
            sdk::text(&format!("💰 Ingresos totales: ${:.2}", total_revenue), "success"),
            sdk::text(&format!("⏳ Pendientes: {}", pending_count), "warning"),
        ]),

        sdk::card("Acciones", vec![
            sdk::button("Crear Factura", "create_invoice", "primary"),
            sdk::button("Ver Plantillas", "templates", "secondary"),
            sdk::button("Historial", "history", "outline"),
        ]),

        sdk::card("Últimas Facturas", vec![
            sdk::table(
                vec!["Número", "Cliente", "Monto", "Estado", "Fecha"],
                vec![
                    vec!["INV-001", "Acme Corp", "$1,250.00", "Pagada", "2024-01-15"],
                    vec!["INV-002", "Tech Solutions", "$850.00", "Pendiente", "2024-01-14"],
                    vec!["INV-003", "Global Inc", "$2,100.00", "Enviada", "2024-01-13"],
                ],
            ),
        ]),

        sdk::card("Gráficos", vec![
            sdk::chart("Facturas por Mes", vec![
                ("Ene", 12.0), ("Feb", 18.0), ("Mar", 15.0),
                ("Abr", 22.0), ("May", 20.0),
            ], "line"),
        ]),
    ]);
}

fn render_create_invoice() {
    sdk::respond(sdk::widgets![
        sdk::card("Nueva Factura", vec![
            sdk::input("Número", "invoice_number", "INV-001"),
            sdk::input("Cliente", "client_name", "Nombre del cliente"),
            sdk::input("Email del Cliente", "client_email", "cliente@email.com"),
            sdk::input("Fecha", "invoice_date", "2024-01-15"),
            sdk::input("Fecha de Vencimiento", "due_date", "2024-02-15"),
            sdk::divider(),
            sdk::text("Conceptos:", "info"),
            sdk::input("Descripción", "item_desc", "Servicio de soporte técnico"),
            sdk::number_input_with_limits("Cantidad", "item_qty", "1", "1", 1.0, 1000.0, 1.0),
            sdk::number_input_with_limits("Precio Unitario ($)", "item_price", "0.00", "0.00", 0.0, 100000.0, 0.01),
            sdk::divider(),
            sdk::input("Notas", "notes", "Notas adicionales (opcional)"),
            sdk::input("Términos y Condiciones", "terms", "Pago a 30 días"),
            sdk::button("Crear Factura", "create_invoice", "primary"),
            sdk::button("Guardar como Plantilla", "save_template", "secondary"),
        ]),
    ]);
}

fn render_templates() {
    sdk::respond(sdk::widgets![
        sdk::card("Plantillas de Factura", vec![
            sdk::text("Selecciona una plantilla para crear facturas", "info"),
            sdk::table(
                vec!["Nombre", "Descripción", "Predeterminada"],
                vec![
                    vec!["Estándar", "Plantilla básica con logo", "Sí"],
                    vec!["Profesional", "Diseño moderno con colores", "No"],
                    vec!["Simple", "Formato minimalista", "No"],
                    vec!["Corporativo", "Para empresas grandes", "No"],
                ],
            ),
        ]),
    ]);
}

fn render_invoice_history() {
    sdk::respond(sdk::widgets![
        sdk::card("Historial de Facturas", vec![
            sdk::table(
                vec!["Número", "Cliente", "Monto", "Estado", "Fecha", "Acciones"],
                vec![
                    vec!["INV-001", "Acme Corp", "$1,250.00", "Pagada", "2024-01-15", "PDF"],
                    vec!["INV-002", "Tech Solutions", "$850.00", "Pendiente", "2024-01-14", "Enviar"],
                    vec!["INV-003", "Global Inc", "$2,100.00", "Enviada", "2024-01-13", "PDF"],
                ],
            ),
        ]),
    ]);
}

fn create_invoice(data: &str) {
    sdk::log(&format!("Creando factura: {}", data));
    
    let count = sdk::kv_get_val("invoice_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("invoice_count", &count.to_string());
    
    sdk::respond_ok(&format!("Factura INV-{:03} creada exitosamente", count));
}

fn send_invoice(data: &str) {
    sdk::log(&format!("Enviando factura por email: {}", data));
    sdk::respond_ok("Factura enviada por email");
}

fn download_pdf(data: &str) {
    sdk::log(&format!("Generando PDF de factura: {}", data));
    sdk::respond_ok("PDF generado y listo para descargar");
}

fn duplicate_invoice(data: &str) {
    sdk::log(&format!("Duplicando factura: {}", data));
    sdk::respond_ok("Factura duplicada");
}

fn void_invoice(data: &str) {
    sdk::log(&format!("Anulando factura: {}", data));
    sdk::respond_ok("Factura anulada");
}

fn add_line_item(data: &str) {
    sdk::log(&format!("Agregando concepto: {}", data));
    sdk::respond_ok("Concepto agregado");
}

fn save_template(data: &str) {
    sdk::log(&format!("Guardando plantilla: {}", data));
    sdk::respond_ok("Plantilla guardada");
}
