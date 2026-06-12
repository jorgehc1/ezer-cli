// Ejemplo 22: Receptor de Traps SNMP
// Features: HTTP, KV store, Events, Table, Charts
// Demuestra: Recepción y procesamiento de traps SNMP via webhook

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("snmp_traps", "Traps SNMP", "alarm-warning-line")
                        .category("sistema")
                        .priority(26)
                )
                .name("Receptor de Traps SNMP")
                .description("Recibe y procesa traps SNMP de dispositivos de red")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "snmp_traps" => render_trap_dashboard(),
                "trap_history" => render_trap_history(),
                "trap_rules" => render_trap_rules(),
                _ => {}
            }
        }

        // Recibir traps via webhook (simulado)
        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "process_trap" => process_trap(&data),
                "add_rule" => render_add_rule_form(),
                "save_rule" => save_rule(&data),
                "test_trap" => test_trap(),
                "clear_history" => clear_trap_history(),
                "export_traps" => export_traps(),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_trap_dashboard() {
    let trap_count = sdk::kv_get_val("trap_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let critical_count = sdk::kv_get_val("critical_traps")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Receptor de Traps SNMP", vec![
            sdk::text("Monitorea alertas de dispositivos de red en tiempo real", "info"),
            sdk::divider(),
            sdk::text(&format!("📊 Traps recibidos: {}", trap_count), "info"),
            sdk::text(&format!("🔴 Traps críticos: {}", critical_count), "warning"),
        ]),

        sdk::card("Traps Recientes", vec![
            sdk::table(
                vec!["Timestamp", "Origen", "Tipo", "Mensaje", "Severidad"],
                vec![
                    vec!["10:30:15", "192.168.1.1", "linkDown", "Interface Gi0/1 down", "Crítica"],
                    vec!["10:28:42", "192.168.1.2", "cpuHigh", "CPU > 95%", "Alta"],
                    vec!["10:25:10", "192.168.1.10", "diskFull", "Disco / en 98%", "Alta"],
                    vec!["10:20:33", "192.168.1.3", "authFailure", "3 intentos fallidos", "Media"],
                ],
            ),
        ]),

        sdk::card("Acciones", vec![
            sdk::button("Agregar Regla", "add_rule", "primary"),
            sdk::button("Ver Historial", "trap_history", "secondary"),
            sdk::button("Exportar Traps", "export_traps", "outline"),
            sdk::button("Limpiar Historial", "clear_history", "danger"),
        ]),
    ]);
}

fn render_trap_history() {
    sdk::respond(sdk::widgets![
        sdk::card("Historial de Traps", vec![
            sdk::text("Últimos 100 traps recibidos", "info"),
            sdk::table(
                vec!["ID", "Timestamp", "Origen", "OID", "Tipo", "Valor"],
                vec![
                    vec!["1", "2024-01-15 10:30:15", "192.168.1.1", "1.3.6.1.2.1.2.2.1.2", "linkDown", "2"],
                    vec!["2", "2024-01-15 10:28:42", "192.168.1.2", "1.3.6.1.4.1.2021.13.15.1", "cpuHigh", "95"],
                ],
            ),
            sdk::button("Volver", "refresh", "outline"),
        ]),
    ]);
}

fn render_trap_rules() {
    sdk::respond(sdk::widgets![
        sdk::card("Reglas de Trap", vec![
            sdk::text("Define reglas para procesar traps SNMP", "info"),
            sdk::table(
                vec!["Nombre", "OID", "Acción", "Severidad", "Activa"],
                vec![
                    vec!["linkDown", "1.3.6.1.2.1.2.2.1.2", "Notificar admin", "Crítica", "Sí"],
                    vec!["cpuHigh", "1.3.6.1.4.1.2021.13.15.1", "Log + Alerta", "Alta", "Sí"],
                    vec!["diskFull", "1.3.6.1.4.1.2021.1.3.10", "Ticket automático", "Alta", "Sí"],
                ],
            ),
            sdk::button("Agregar Regla", "add_rule", "primary"),
        ]),
    ]);
}

fn render_add_rule_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Nueva Regla de Trap", vec![
            sdk::input("Nombre", "rule_name", "Mi Regla"),
            sdk::input("OID del Trap", "trap_oid", "1.3.6.1.2.1.2.2.1.2"),
            sdk::select_widget("Acción", "action", vec![
                ("notify".to_string(), "Notificar por email".to_string()),
                ("ticket".to_string(), "Crear ticket".to_string()),
                ("log".to_string(), "Solo registrar".to_string()),
                ("webhook".to_string(), "Enviar webhook".to_string()),
            ], "notify".to_string()),
            sdk::select_widget("Severidad", "severity", vec![
                ("critical".to_string(), "Crítica".to_string()),
                ("high".to_string(), "Alta".to_string()),
                ("medium".to_string(), "Media".to_string()),
                ("low".to_string(), "Baja".to_string()),
            ], "medium".to_string()),
            sdk::number_input_with_limits("Umbral (repeticiones)", "threshold", "1", "1", 1.0, 100.0, 1.0),
            sdk::button("Guardar Regla", "save_rule", "primary"),
            sdk::button("Cancelar", "trap_rules", "outline"),
        ]),
    ]);
}

fn process_trap(data: &str) {
    sdk::log(&format!("Procesando trap SNMP: {}", data));
    
    // Incrementar contador de traps
    let count = sdk::kv_get_val("trap_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("trap_count", &count.to_string());
    
    sdk::respond_ok(&format!("Trap procesado. Total: {}", count));
}

fn test_trap() {
    sdk::log("Enviando trap de prueba...");
    sdk::respond_ok("Trap de prueba enviado");
}

fn clear_trap_history() {
    sdk::kv_set_val("trap_count", "0");
    sdk::kv_set_val("critical_traps", "0");
    sdk::respond_ok("Historial de traps limpiado");
}

fn export_traps() {
    sdk::log("Exportando traps...");
    sdk::respond_ok("Traps exportados exitosamente");
}

fn save_rule(data: &str) {
    sdk::log(&format!("Guardando regla: {}", data));
    sdk::respond_ok("Regla guardada exitosamente");
}
