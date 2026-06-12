// Ejemplo 14: Gestor de Cupones
// Features: CRUD de cupones, validación, KV store, Charts
// Demuestra: Sistema de descuentos, validación de uso, estadísticas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("cupones", "Gestor Cupones", "ticket-2-line")
                        .category("herramientas")
                        .priority(20)
                )
                .name("Gestor de Cupones")
                .description("Crea y gestiona cupones de descuento para clientes")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "cupones" => render_coupons_dashboard(),
                "create" => render_create_coupon(),
                "validate" => render_validate_coupon(),
                "usage" => render_usage_stats(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "create_coupon" => handle_create_coupon(&data),
                "validate_coupon" => handle_validate_coupon(&data),
                "deactivate_coupon" => deactivate_coupon(&data),
                "delete_coupon" => delete_coupon(&data),
                "copy_coupon" => copy_coupon_code(&data),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

// Dashboard principal de cupones
fn render_coupons_dashboard() {
    let coupon_count = sdk::kv_get_val("coupon_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let total_usage = sdk::kv_get_val("total_coupon_usage")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Gestor de Cupones", vec![
            sdk::text("Administra cupones de descuento para tus clientes", "info"),
            sdk::divider(),

            // Estadísticas
            sdk::card("Estadísticas", vec![
                sdk::text(&format!("🎫 Cupones creados: {}", coupon_count), "info"),
                sdk::text(&format!("📊 Usos totales: {}", total_usage), "info"),
                sdk::chart("Cupones por Tipo", vec![
                    ("Porcentaje", 12.0),
                    ("Monto Fijo", 8.0),
                    ("Envío Gratis", 5.0),
                    ("Regalo", 3.0),
                ], "pie"),
            ]),

            // Lista de cupones activos
            sdk::card("Cupones Activos", vec![
                sdk::table(
                    vec!["Código", "Tipo", "Valor", "Usos", "Expira", "Estado"],
                    vec![
                        vec!["BIENVENIDO20", "Porcentaje", "20%", "45", "2024-12-31", "✅ Activo"],
                        vec!["FIJO10", "Monto Fijo", "$10", "23", "2024-06-30", "✅ Activo"],
                        vec!["ENVIOGratis", "Envío Gratis", "100%", "67", "2024-12-31", "✅ Activo"],
                        vec!["VERANO15", "Porcentaje", "15%", "0", "2024-03-31", "⏸️ Pausado"],
                    ],
                ),
            ]),

            sdk::divider(),

            // Acciones
            sdk::card("Acciones", vec![
                sdk::button("Crear Nuevo Cupón", "create", "primary"),
                sdk::button("Validar Cupón", "validate", "secondary"),
                sdk::button("Estadísticas de Uso", "usage", "secondary"),
            ]),
        ]),
    ]);
}

// Formulario para crear un cupón
fn render_create_coupon() {
    sdk::respond(sdk::widgets![
        sdk::card("Crear Nuevo Cupón", vec![
            sdk::text("Define los detalles del cupón de descuento", "info"),
            sdk::divider(),

            sdk::input("coupon_code", "Código del Cupón")
                .placeholder("DESCUENTO20")
                .required(true),

            sdk::textarea("coupon_description", "Descripción")
                .placeholder("Descuento especial de bienvenida..."),

            sdk::select("coupon_type", "Tipo de Cupón", vec![
                ("percentage", "Porcentaje de Descuento"),
                ("fixed", "Monto Fijo"),
                ("shipping", "Envío Gratis"),
                ("gift", "Regalo / Producto Gratis"),
            ]),

            sdk::number_input("discount_value", "Valor del Descuento")
                .min(1)
                .max(100)
                .placeholder("20"),

            sdk::select("discount_unit", "Unidad", vec![
                ("percent", "Porcentaje (%)"),
                ("currency", "Moneda ($)"),
                ("free", "Gratis"),
            ]),

            sdk::number_input("min_purchase", "Compra Mínima")
                .min(0)
                .placeholder("50"),

            sdk::number_input("max_uses", "Usos Máximos (0 = ilimitado)")
                .min(0)
                .placeholder("100"),

            sdk::number_input("max_per_user", "Usos por Usuario (0 = ilimitado)")
                .min(0)
                .placeholder("1"),

            sdk::date_input("valid_from", "Válido Desde")
                .required(true),

            sdk::date_input("valid_until", "Válido Hasta")
                .required(true),

            sdk::select("applicable_to", "Aplicable A", vec![
                ("all", "Todos los productos"),
                ("specific", "Productos específicos"),
                ("category", "Categorías específicas"),
                ("new_users", "Solo nuevos usuarios"),
            ]),

            sdk::select("coupon_status", "Estado", vec![
                ("active", "Activo"),
                ("inactive", "Inactivo"),
                ("scheduled", "Programado"),
            ]),

            sdk::divider(),

            sdk::button("Crear Cupón", "create_coupon", "primary"),
            sdk::button("Cancelar", "cupones", "outline"),
        ]),
    ]);
}

// Formulario para validar un cupón
fn render_validate_coupon() {
    sdk::respond(sdk::widgets![
        sdk::card("Validar Cupón", vec![
            sdk::text("Ingresa un código de cupón para verificar su validez", "info"),
            sdk::divider(),

            sdk::input("validate_code", "Código de Cupón")
                .placeholder("DESCUENTO20")
                .required(true),

            sdk::number_input("purchase_amount", "Monto de Compra")
                .min(0)
                .placeholder("100"),

            sdk::divider(),

            sdk::button("Validar Cupón", "validate_coupon", "primary"),
            sdk::button("Cancelar", "cupones", "outline"),
        ]),
    ]);
}

// Estadísticas de uso
fn render_usage_stats() {
    sdk::respond(sdk::widgets![
        sdk::card("Estadísticas de Uso de Cupones", vec![
            sdk::chart("Usos por Mes", vec![
                ("Ene", 45.0),
                ("Feb", 52.0),
                ("Mar", 38.0),
                ("Abr", 61.0),
                ("May", 55.0),
            ], "bar"),

            sdk::chart("Cupones Más Usados", vec![
                ("BIENVENIDO20", 45.0),
                ("ENVIOGratis", 67.0),
                ("FIJO10", 23.0),
                ("VERANO15", 12.0),
            ], "bar"),

            sdk::chart("Descuentos Otorgados", vec![
                ("Ene", 450.0),
                ("Feb", 520.0),
                ("Mar", 380.0),
                ("Abr", 610.0),
                ("May", 550.0),
            ], "line"),

            sdk::card("Top Cupones", vec![
                sdk::table(
                    vec!["Código", "Usos", "Descuento Total", "ROI"],
                    vec![
                        vec!["BIENVENIDO20", "45", "$900", "180%"],
                        vec!["ENVIOGratis", "67", "$670", "134%"],
                        vec!["FIJO10", "23", "$230", "115%"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::button("Volver al Dashboard", "cupones", "outline"),
        ]),
    ]);
}

// Crea un cupón
fn handle_create_coupon(data: &str) {
    let code = extract_field(data, "coupon_code").unwrap_or_default();
    let description = extract_field(data, "coupon_description").unwrap_or_default();
    let coupon_type = extract_field(data, "coupon_type").unwrap_or_default();
    let value = extract_field(data, "discount_value").unwrap_or_default();
    let min_purchase = extract_field(data, "min_purchase").unwrap_or_default();
    let max_uses = extract_field(data, "max_uses").unwrap_or_default();
    let valid_from = extract_field(data, "valid_from").unwrap_or_default();
    let valid_until = extract_field(data, "valid_until").unwrap_or_default();
    let status = extract_field(data, "coupon_status").unwrap_or("active".to_string());

    if code.is_empty() || valid_from.is_empty() || valid_until.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Código, fecha de inicio y fin son obligatorios", "warning"),
        ]);
        return;
    }

    // Verificar que el código no exista
    let existing = sdk::kv_get_val(&format!("coupon_{}", code));
    if existing.is_some() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Ya existe un cupón con ese código", "warning"),
        ]);
        return;
    }

    // Guardar cupón
    let coupon_data = format!(
        "code:{}|desc:{}|type:{}|value:{}|min:{}|max_uses:{}|from:{}|until:{}|status:{}|uses:0",
        code, description, coupon_type, value, min_purchase, max_uses, valid_from, valid_until, status
    );

    sdk::kv_set_val(&format!("coupon_{}", code), &coupon_data);

    let count = sdk::kv_get_val("coupon_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("coupon_count", &count.to_string());

    sdk::log(&format!("Cupón creado: {} ({})", code, coupon_type));

    sdk::respond(sdk::widgets![
        sdk::card("Cupón Creado", vec![
            sdk::text("✅ Cupón creado exitosamente", "success"),
            sdk::divider(),
            sdk::text(&format!("🎫 Código: {}", code), "info"),
            sdk::text(&format!("📝 Descripción: {}", description), "info"),
            sdk::text(&format!("🏷️ Tipo: {}", coupon_type), "info"),
            sdk::text(&format!("💰 Valor: {}{}", value, if coupon_type == "percentage" { "%" } else { "$" }), "info"),
            sdk::text(&format!("🛒 Compra mínima: ${}", min_purchase), "info"),
            sdk::text(&format!("📊 Usos máximos: {}", max_uses), "info"),
            sdk::text(&format!("📅 Válido: {} hasta {}", valid_from, valid_until), "info"),
            sdk::text(&format!("📌 Estado: {}", status), "info"),
            sdk::divider(),
            sdk::button("Ver Cupones", "cupones", "primary"),
            sdk::button("Crear Otro", "create", "secondary"),
        ]),
    ]);
}

// Valida un cupón
fn handle_validate_coupon(data: &str) {
    let code = extract_field(data, "validate_code").unwrap_or_default();
    let purchase_amount = extract_field(data, "purchase_amount")
        .unwrap_or("0".to_string())
        .parse::<f64>()
        .unwrap_or(0.0);

    if code.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ Ingresa un código de cupón", "warning"),
        ]);
        return;
    }

    // Buscar cupón
    let coupon_data = sdk::kv_get_val(&format!("coupon_{}", code));

    match coupon_data {
        Some(data) => {
            // Parsear datos del cupón
            let coupon_type = extract_field_from_str(&data, "type").unwrap_or_default();
            let value = extract_field_from_str(&data, "value").unwrap_or_default();
            let min_purchase = extract_field_from_str(&data, "min")
                .unwrap_or("0".to_string())
                .parse::<f64>()
                .unwrap_or(0.0);
            let max_uses = extract_field_from_str(&data, "max_uses")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            let current_uses = extract_field_from_str(&data, "uses")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            let valid_from = extract_field_from_str(&data, "from").unwrap_or_default();
            let valid_until = extract_field_from_str(&data, "until").unwrap_or_default();
            let status = extract_field_from_str(&data, "status").unwrap_or_default();

            // Validaciones
            if status != "active" {
                sdk::respond(sdk::widgets![
                    sdk::card("Cupón Inválido", vec![
                        sdk::text("❌ Este cupón no está activo", "error"),
                        sdk::button("Volver", "validate", "outline"),
                    ]),
                ]);
                return;
            }

            if purchase_amount < min_purchase {
                sdk::respond(sdk::widgets![
                    sdk::card("Cupón Inválido", vec![
                        sdk::text(&format!("❌ Compra mínima requerida: ${}", min_purchase), "error"),
                        sdk::text(&format!("Tu compra: ${}", purchase_amount), "info"),
                        sdk::button("Volver", "validate", "outline"),
                    ]),
                ]);
                return;
            }

            if max_uses > 0 && current_uses >= max_uses {
                sdk::respond(sdk::widgets![
                    sdk::card("Cupón Inválido", vec![
                        sdk::text("❌ Este cupón ha alcanzado su límite de usos", "error"),
                        sdk::button("Volver", "validate", "outline"),
                    ]),
                ]);
                return;
            }

            // Calcular descuento
            let discount = match coupon_type.as_str() {
                "percentage" => {
                    let val = value.parse::<f64>().unwrap_or(0.0);
                    purchase_amount * (val / 100.0)
                }
                "fixed" => {
                    value.parse::<f64>().unwrap_or(0.0)
                }
                "shipping" => 9.99, // Costo estándar de envío
                _ => 0.0,
            };

            let final_price = (purchase_amount - discount).max(0.0);

            sdk::respond(sdk::widgets![
                sdk::card("Cupón Válido", vec![
                    sdk::text("✅ Este cupón es válido", "success"),
                    sdk::divider(),
                    sdk::text(&format!("🎫 Código: {}", code), "info"),
                    sdk::text(&format!("🏷️ Tipo: {}", coupon_type), "info"),
                    sdk::text(&format!("💰 Descuento: ${:.2}", discount), "success"),
                    sdk::text(&format!("🛒 Compra original: ${:.2}", purchase_amount), "info"),
                    sdk::text(&format!("✨ Precio final: ${:.2}", final_price), "success"),
                    sdk::text(&format!("📊 Usos: {}/{}", current_uses, if max_uses == 0 { "∞".to_string() } else { max_uses.to_string() }), "info"),
                    sdk::text(&format!("📅 Válido hasta: {}", valid_until), "info"),
                    sdk::divider(),
                    sdk::button("Aplicar Cupón", "cupones", "primary"),
                    sdk::button("Volver", "validate", "outline"),
                ]),
            ]);
        }
        None => {
            sdk::respond(sdk::widgets![
                sdk::card("Cupón No Encontrado", vec![
                    sdk::text("❌ No se encontró un cupón con ese código", "error"),
                    sdk::button("Intentar de Nuevo", "validate", "primary"),
                ]),
            ]);
        }
    }
}

// Desactiva un cupón
fn deactivate_coupon(data: &str) {
    let code = extract_field(data, "coupon_code").unwrap_or_default();
    if !code.is_empty() {
        sdk::log(&format!("Cupón desactivado: {}", code));
    }

    sdk::respond(sdk::widgets![
        sdk::text("⏸️ Cupón desactivado", "warning"),
        sdk::button("Volver", "cupones", "outline"),
    ]);
}

// Elimina un cupón
fn delete_coupon(data: &str) {
    let code = extract_field(data, "coupon_code").unwrap_or_default();
    if !code.is_empty() {
        sdk::kv_set_val(&format!("coupon_{}", code), "");
        sdk::log(&format!("Cupón eliminado: {}", code));
    }

    sdk::respond(sdk::widgets![
        sdk::text("🗑️ Cupón eliminado", "success"),
        sdk::button("Volver", "cupones", "outline"),
    ]);
}

// Copia el código del cupón
fn copy_coupon_code(data: &str) {
    let code = extract_field(data, "coupon_code").unwrap_or_default();
    sdk::log(&format!("Código copiado: {}", code));

    sdk::respond(sdk::widgets![
        sdk::text(&format!("📋 Código copiado: {}", code), "info"),
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

// Helper alternativo para extraer campos de strings con formato key:value
fn extract_field_from_str(data: &str, field: &str) -> Option<String> {
    let search = format!("{}:", field);
    if let Some(pos) = data.find(&search) {
        let start = pos + search.len();
        if let Some(end) = data[start..].find('|') {
            return Some(data[start..start + end].to_string());
        } else {
            return Some(data[start..].to_string());
        }
    }
    None
}
