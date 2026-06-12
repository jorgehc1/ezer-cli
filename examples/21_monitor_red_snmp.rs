// Ejemplo 21: Monitor de Red SNMP
// Features: HTTP requests, Charts, Table, KV store, Cron
// Demuestra: Monitoreo de dispositivos de red via SNMP proxy/API

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("network", "Monitoreo Red", "wifi-line")
                        .category("sistema")
                        .priority(25)
                )
                .name("Monitor de Red SNMP")
                .description("Monitorea dispositivos de red via API SNMP/Zabbix")
                .version("1.0.0")
                .cron("300"); // Cada 5 minutos
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "network" => render_network_dashboard(),
                "devices" => render_device_list(),
                "alerts" => render_alerts(),
                _ => {}
            }
        }

        PluginEvent::CronTick => {
            // Monitorear dispositivos periódicamente
            check_device_status();
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "refresh" => render_network_dashboard(),
                "scan_network" => scan_network(),
                "add_device" => render_add_device_form(),
                "save_device" => save_device(&data),
                "ping_device" => ping_device(&data),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_network_dashboard() {
    let devices = get_device_list();
    let online_count = devices.iter().filter(|d| d.status == "online").count();
    let offline_count = devices.len() - online_count;

    sdk::respond(sdk::widgets![
        sdk::card("Monitoreo de Red", vec![
            sdk::text(&format!("🟢 En línea: {} | 🔴 Fuera de línea: {}", online_count, offline_count), "info"),
            sdk::divider(),
            sdk::chart("Estado de Dispositivos", vec![
                ("En Línea", online_count as f64),
                ("Fuera de Línea", offline_count as f64),
            ], "pie"),
        ]),

        sdk::card("Dispositivos", vec![
            sdk::table(
                vec!["Nombre", "IP", "Tipo", "Estado", "Último Check"],
                devices.iter().map(|d| {
                    vec![
                        d.name.as_str(),
                        d.ip.as_str(),
                        d.device_type.as_str(),
                        d.status.as_str(),
                        d.last_check.as_str(),
                    ]
                }).collect(),
            ),
            sdk::button("Escanear Red", "scan_network", "primary"),
            sdk::button("Agregar Dispositivo", "add_device", "secondary"),
        ]),

        sdk::card("Alertas Recientes", vec![
            sdk::text("Últimas alertas de monitoreo", "default"),
            sdk::button("Ver Todas", "alerts", "outline"),
        ]),
    ]);
}

fn render_device_list() {
    let devices = get_device_list();
    
    sdk::respond(sdk::widgets![
        sdk::card("Lista de Dispositivos", vec![
            sdk::table(
                vec!["ID", "Nombre", "IP", "Tipo", "Estado", "Vendor", "Modelo"],
                devices.iter().map(|d| {
                    vec![
                        d.id.as_str(),
                        d.name.as_str(),
                        d.ip.as_str(),
                        d.device_type.as_str(),
                        d.status.as_str(),
                        d.vendor.as_str(),
                        d.model.as_str(),
                    ]
                }).collect(),
            ),
            sdk::button("Volver", "refresh", "outline"),
        ]),
    ]);
}

fn render_alerts() {
    let alerts = get_recent_alerts();
    
    sdk::respond(sdk::widgets![
        sdk::card("Alertas de Red", vec![
            sdk::table(
                vec!["Timestamp", "Dispositivo", "Tipo", "Mensaje", "Severidad"],
                alerts.iter().map(|a| {
                    vec![
                        a.timestamp.as_str(),
                        a.device.as_str(),
                        a.alert_type.as_str(),
                        a.message.as_str(),
                        a.severity.as_str(),
                    ]
                }).collect(),
            ),
        ]),
    ]);
}

fn render_add_device_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Agregar Dispositivo de Red", vec![
            sdk::input("Nombre", "device_name", "Router Principal"),
            sdk::input("Dirección IP", "device_ip", "192.168.1.1"),
            sdk::select_widget("Tipo", "device_type", vec![
                ("router".to_string(), "Router".to_string()),
                ("switch".to_string(), "Switch".to_string()),
                ("firewall".to_string(), "Firewall".to_string()),
                ("server".to_string(), "Servidor".to_string()),
                ("printer".to_string(), "Impresora".to_string()),
                ("access_point".to_string(), "Access Point".to_string()),
            ], "router".to_string()),
            sdk::input("Vendor", "vendor", "Cisco"),
            sdk::input("Modelo", "model", "ISR 4331"),
            sdk::input("SNMP Community", "snmp_community", "public"),
            sdk::number_input_with_limits("Puerto SNMP", "snmp_port", "161", "161", 1.0, 65535.0, 1.0),
            sdk::button("Guardar Dispositivo", "save_device", "primary"),
            sdk::button("Cancelar", "refresh", "outline"),
        ]),
    ]);
}

// ── Estructuras de datos ─────────────────────────────────────────────────────

struct Device {
    id: String,
    name: String,
    ip: String,
    device_type: String,
    status: String,
    vendor: String,
    model: String,
    last_check: String,
}

struct Alert {
    timestamp: String,
    device: String,
    alert_type: String,
    message: String,
    severity: String,
}

// ── Funciones de datos (simuladas) ──────────────────────────────────────────

fn get_device_list() -> Vec<Device> {
    // En producción, esto vendría de una query o HTTP request
    vec![
        Device { id: "1".into(), name: "Router Principal".into(), ip: "192.168.1.1".into(), device_type: "router".into(), status: "online".into(), vendor: "Cisco".into(), model: "ISR 4331".into(), last_check: "Hace 2 min".into() },
        Device { id: "2".into(), name: "Switch Piso 1".into(), ip: "192.168.1.2".into(), device_type: "switch".into(), status: "online".into(), vendor: "Cisco".into(), model: "C9200".into(), last_check: "Hace 3 min".into() },
        Device { id: "3".into(), name: "Firewall".into(), ip: "192.168.1.3".into(), device_type: "firewall".into(), status: "offline".into(), vendor: "Fortinet".into(), model: "FortiGate 60F".into(), last_check: "Hace 15 min".into() },
        Device { id: "4".into(), name: "Servidor Web".into(), ip: "192.168.1.10".into(), device_type: "server".into(), status: "online".into(), vendor: "Dell".into(), model: "PowerEdge R740".into(), last_check: "Hace 1 min".into() },
    ]
}

fn get_recent_alerts() -> Vec<Alert> {
    vec![
        Alert { timestamp: "2024-01-15 10:30".into(), device: "Firewall".into(), alert_type: "Dispositivo fuera de línea".into(), message: "No responde a SNMP polls".into(), severity: "Crítica".into() },
        Alert { timestamp: "2024-01-15 10:25".into(), device: "Servidor Web".into(), alert_type: "CPU alto".into(), message: "Uso de CPU > 90%".into(), severity: "Alta".into() },
        Alert { timestamp: "2024-01-15 10:20".into(), device: "Router Principal".into(), alert_type: "Ancho de banda".into(), message: "Utilización > 80%".into(), severity: "Media".into() },
    ]
}

fn check_device_status() {
    sdk::log("Verificando estado de dispositivos...");
    // En producción: hacer requests SNMP/Zabbix API
}

fn scan_network() {
    sdk::log("Escaneando red...");
    sdk::respond_ok("Escaneo de red iniciado");
}

fn save_device(data: &str) {
    sdk::log(&format!("Guardando dispositivo: {}", data));
    sdk::respond_ok("Dispositivo guardado exitosamente");
}

fn ping_device(data: &str) {
    sdk::log(&format!("Haciendo ping a dispositivo: {}", data));
    sdk::respond_ok("Ping enviado");
}
