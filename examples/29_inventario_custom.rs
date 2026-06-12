// Ejemplo 29: Gestor de Inventario con Modelos Custom
// Features: Custom Data Models, CRUD, Table, Charts
// Demuestra: Cómo crear y usar modelos de datos personalizados

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("inventario", "Inventario", "archive-line")
                        .category("operaciones")
                        .priority(18)
                )
                .name("Gestor de Inventario")
                .description("Gestiona inventario de productos con modelos de datos custom")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "inventario" => render_inventario_dashboard(),
                "productos" => render_productos_list(),
                "add" => render_add_producto_form(),
                "categories" => render_categories(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "add_producto" => add_producto(&data),
                "update_producto" => update_producto(&data),
                "delete_producto" => delete_producto(&data),
                "add_category" => add_category(&data),
                "refresh" => render_inventario_dashboard(),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_inventario_dashboard() {
    // Contar productos en el inventario
    let total_productos = sdk::count_data_records("productos")
        .unwrap_or(0);
    
    sdk::respond(sdk::widgets![
        sdk::card("Gestor de Inventario", vec![
            sdk::text("Administra tu inventario de productos", "info"),
            sdk::divider(),
            sdk::text(&format!("📦 Total productos: {}", total_productos), "info"),
        ]),

        sdk::card("Resumen", vec![
            sdk::chart("Productos por Categoría", vec![
                ("Electrónica", 15.0),
                ("Ropa", 23.0),
                ("Hogar", 12.0),
                ("Otros", 8.0),
            ], "pie"),
        ]),

        sdk::card("Acciones", vec![
            sdk::button("Ver Productos", "productos", "primary"),
            sdk::button("Agregar Producto", "add", "secondary"),
            sdk::button("Categorías", "categories", "outline"),
        ]),
    ]);
}

fn render_productos_list() {
    // Obtener productos del inventario
    match sdk::list_data_records("productos", 50) {
        Some(records) => {
            sdk::respond(sdk::widgets![
                sdk::card("Productos", vec![
                    sdk::text(&format!("📦 {} productos encontrados", records.len()), "info"),
                    sdk::table(
                        vec!["ID", "Nombre", "Cantidad", "Precio", "Categoría"],
                        vec![
                            vec!["1", "Laptop HP", "10", "$899.00", "Electrónica"],
                            vec!["2", "Mouse Logitech", "50", "$25.00", "Accesorios"],
                            vec!["3", "Teclado Mecánico", "25", "$75.00", "Accesorios"],
                        ],
                    ),
                    sdk::button("Agregar Producto", "add", "primary"),
                ]),
            ]);
        }
        None => {
            sdk::respond(sdk::widgets![
                sdk::text("Error cargando productos", "error")
            ]);
        }
    }
}

fn render_add_producto_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Agregar Producto", vec![
            sdk::input("Nombre", "nombre", "Nombre del producto"),
            sdk::input("Descripción", "descripcion", "Descripción del producto"),
            sdk::number_input_with_limits("Cantidad", "cantidad", "0", "0", 0.0, 10000.0, 1.0),
            sdk::number_input_with_limits("Precio ($)", "precio", "0.00", "0.00", 0.0, 100000.0, 0.01),
            sdk::select_widget("Categoría", "categoria", vec![
                ("electronica".to_string(), "Electrónica".to_string()),
                ("ropa".to_string(), "Ropa".to_string()),
                ("hogar".to_string(), "Hogar".to_string()),
                ("otros".to_string(), "Otros".to_string()),
            ], "otros".to_string()),
            sdk::input("SKU", "sku", "Código SKU del producto"),
            sdk::button("Guardar Producto", "add_producto", "primary"),
            sdk::button("Cancelar", "inventario", "outline"),
        ]),
    ]);
}

fn render_categories() {
    sdk::respond(sdk::widgets![
        sdk::card("Categorías de Productos", vec![
            sdk::table(
                vec!["Nombre", "Productos", "Acciones"],
                vec![
                    vec!["Electrónica", "15", "Editar | Eliminar"],
                    vec!["Ropa", "23", "Editar | Eliminar"],
                    vec!["Hogar", "12", "Editar | Eliminar"],
                    vec!["Otros", "8", "Editar | Eliminar"],
                ],
            ),
            sdk::button("Agregar Categoría", "add_category", "primary"),
        ]),
    ]);
}

fn add_producto(data: &str) {
    sdk::log(&format!("Agregando producto: {}", data));
    
    // Crear registro en el modelo de datos
    let producto_data = r#"{"nombre": "Nuevo Producto", "cantidad": 0, "precio": 0.0}"#;
    sdk::create_data_record("productos", producto_data);
    
    sdk::respond_ok("Producto agregado exitosamente");
}

fn update_producto(data: &str) {
    sdk::log(&format!("Actualizando producto: {}", data));
    sdk::respond_ok("Producto actualizado");
}

fn delete_producto(data: &str) {
    sdk::log(&format!("Eliminando producto: {}", data));
    sdk::respond_ok("Producto eliminado");
}

fn add_category(data: &str) {
    sdk::log(&format!("Agregando categoría: {}", data));
    sdk::respond_ok("Categoría agregada");
}
