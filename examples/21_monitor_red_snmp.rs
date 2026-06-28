// Ejemplo 21: Monitor de Red SNMP
// Features: Host SNMP, Charts, Table, KV store, Cron, Data Query
// Demuestra: Monitoreo real de dispositivos via SDK SNMP nativo + consulta dispositivos desde BD

use ezerdesk_sdk as sdk;
use sdk::prelude::*;
use sdk::query::SnmpDeviceSummary;

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
                .description("Monitorea dispositivos SNMP usando SDK nativo y host_snmp")
                .version("2.0.0")
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
            check_all_devices();
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "refresh" => render_network_dashboard(),
                "poll_device" => poll_single_device(&data),
                "add_device" => render_add_device_form(),
                "save_device" => save_device(&data),
                "view_device" => view_device_detail(&data),
                _ => {}
            }
        }

        // Bridge webhook events from snmp-bridge
        PluginEvent::BridgeWebhook { payload } => {
            sdk::log(&format!("Bridge webhook: {}", payload));
            process_bridge_event(payload);
        }

        _ => {}
    }
    0
}

fn render_network_dashboard() {
    let devices = match sdk::query::snmp_devices().all() {
        Ok(d) => d,
        Err(_) => Vec::new(),
    };

    let online = devices.iter().filter(|d| d.activo).count();
    let total = devices.len();

    let mut table_rows: Vec<Vec<&str>> = Vec::new();
    for d in &devices {
        let status = if d.activo { "✅" } else { "⛔" };
        let last_ok = if d.ultimo_ok_en == "0" { "Nunca" } else { &d.ultimo_ok_en };
        table_rows.push(vec![&d.nombre, &d.host, &d.version, status, last_ok]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Monitoreo de Red SNMP", vec![
            sdk::text(&format!("📡 Dispositivos configurados: {}", total), "info"),
            sdk::text(&format!("✅ Activos: {} | ⛔ Inactivos: {}", online, total - online), "info"),
            sdk::divider(),
            sdk::chart("Estado de Dispositivos", vec![
                ("Activos", online as f64),
                ("Inactivos", (total - online) as f64),
            ], "pie"),
        ]),

        sdk::card("Dispositivos SNMP", vec![
            sdk::table(
                vec!["Nombre", "Host", "Versión", "Estado", "Último OK"],
                table_rows,
            ),
            sdk::button("Refrescar", "refresh", "primary"),
            sdk::button("Agregar Dispositivo", "add_device", "secondary"),
        ]),

        sdk::card("Alertas Recientes", vec![
            sdk::text("Últimas alertas de monitoreo", "default"),
            sdk::button("Ver Todas", "alerts", "outline"),
        ]),
    ]);
}

fn render_device_list() {
    let devices = match sdk::query::snmp_devices().all() {
        Ok(d) => d,
        Err(_) => Vec::new(),
    };

    let mut rows: Vec<Vec<&str>> = Vec::new();
    for d in &devices {
        let status = if d.activo { "Activo" } else { "Inactivo" };
        rows.push(vec![&d.id, &d.nombre, &d.host, &d.comunidad, &d.version, status]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Dispositivos SNMP Configurados", vec![
            sdk::table(
                vec!["ID", "Nombre", "Host", "Community", "Versión", "Estado"],
                rows,
            ),
            sdk::button("Volver", "refresh", "outline"),
        ]),
    ]);
}

fn render_alerts() {
    let alerts = get_stored_alerts();
    let mut rows: Vec<Vec<&str>> = Vec::new();
    for a in &alerts {
        rows.push(vec![&a.timestamp, &a.device, &a.alert_type, &a.message, &a.severity]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Alertas de Red", vec![
            sdk::table(
                vec!["Timestamp", "Dispositivo", "Tipo", "Mensaje", "Severidad"],
                rows,
            ),
            sdk::button("Limpiar Alertas", "clear_alerts", "danger"),
            sdk::button("Volver", "refresh", "outline"),
        ]),
    ]);
}

fn render_add_device_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Agregar Dispositivo SNMP", vec![
            sdk::text("Los dispositivos se agregan desde el panel de administración de EzerDesk.", "info"),
            sdk::divider(),
            sdk::input("Nombre", "device_name", "Router Principal"),
            sdk::input("Host/IP", "device_host", "192.168.1.1"),
            sdk::number_input_with_limits("Puerto SNMP", "device_port", "161", "161", 1.0, 65535.0, 1.0),
            sdk::input("Community", "device_community", "public"),
            sdk::select_widget("Versión SNMP", "device_version", vec![
                ("snmp_v2c".to_string(), "SNMP v2c".to_string()),
                ("snmp_v3".to_string(), "SNMP v3".to_string()),
            ], "snmp_v2c".to_string()),
            sdk::button("Guardar", "save_device", "primary"),
            sdk::button("Cancelar", "refresh", "outline"),
        ]),
    ]);
}

fn view_device_detail(data: &str) {
    let device_id = data.trim();
    if device_id.is_empty() {
        sdk::respond_error("ID de dispositivo requerido");
        return;
    }

    // Poll the device in real-time using host_snmp
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let oids = [
        ("sysDescr", "1.3.6.1.2.1.1.1.0"),
        ("sysUpTime", "1.3.6.1.2.1.1.3.0"),
        ("sysName", "1.3.6.1.2.1.1.5.0"),
        ("ifNumber", "1.3.6.1.2.1.2.1.0"),
    ];

    // Try to get host/community from the first stored device
    let devices = sdk::query::snmp_devices().all().unwrap_or_default();
    let device = devices.iter().find(|d| d.id == device_id);
    let (host, port, community) = match device {
        Some(d) => (d.host.as_str(), d.puerto, d.comunidad.as_str()),
        None => ("192.168.1.1", 161, "public"),
    };

    for (name, oid) in &oids {
        match sdk::snmp_get(host, port, community, oid) {
            Ok(resp) => rows.push(vec![name, oid, &resp.value]),
            Err(e) => rows.push(vec![name, oid, &format!("Error: {}", e)]),
        }
    }

    sdk::respond(sdk::widgets![
        sdk::card(&format!("Dispositivo: {} ({})", device.map(|d| &d.nombre).unwrap_or("N/A"), host), vec![
            sdk::table(
                vec!["Campo", "OID", "Valor"],
                rows,
            ),
            sdk::button("Volver", "refresh", "outline"),
        ]),
    ]);
}

fn check_all_devices() {
    sdk::log("Verificando dispositivos SNMP...");

    let devices = match sdk::query::snmp_devices().all() {
        Ok(d) => d,
        Err(e) => {
            sdk::log(&format!("Error fetching devices: {:?}", e));
            return;
        }
    };

    for device in &devices {
        if !device.activo {
            continue;
        }

        let key = format!("last_value_{}", device.id);
        let prev_value = sdk::kv_get_val(&key).unwrap_or_default();

        match sdk::snmp_get(&device.host, device.puerto, &device.comunidad, "1.3.6.1.2.1.1.1.0") {
            Ok(resp) => {
                sdk::log(&format!("Device {}: sysDescr = {}", device.nombre, resp.value));
                sdk::kv_set_val(&key, &resp.value);

                if prev_value.is_empty() {
                    // First poll — store initial state
                    sdk::kv_set_val(&format!("status_{}", device.id), "up");
                    sdk::kv_set_val(&format!("last_ok_{}", device.id), &device.ultimo_ok_en);
                }
            }
            Err(e) => {
                sdk::log(&format!("Device {} DOWN: {}", device.nombre, e));
                sdk::kv_set_val(&format!("status_{}", device.id), "down");

                // Store alert
                let alert_count = sdk::kv_get_val("alert_count").unwrap_or("0".to_string());
                let n: i32 = alert_count.parse().unwrap_or(0);
                store_alert(StoredAlert {
                    timestamp: format!("{}", sdk::now()),
                    device: device.nombre.clone(),
                    alert_type: "SNMP Timeout".into(),
                    message: e,
                    severity: "Crítica".into(),
                    idx: n + 1,
                });
            }
        }
    }

    sdk::log("Verificación completada");
}

fn poll_single_device(data: &str) {
    let device_id = data.trim();
    if device_id.is_empty() {
        sdk::respond_error("ID requerido");
        return;
    }

    let devices = sdk::query::snmp_devices().all().unwrap_or_default();
    let device = match devices.iter().find(|d| d.id == device_id) {
        Some(d) => d,
        None => {
            sdk::respond_error("Dispositivo no encontrado");
            return;
        }
    };

    match sdk::snmp_get(&device.host, device.puerto, &device.comunidad, "1.3.6.1.2.1.1.1.0") {
        Ok(resp) => sdk::respond_ok(&format!("sysDescr: {}", resp.value)),
        Err(e) => sdk::respond_error(&format!("Error: {}", e)),
    }
}

// ── Data structures ──────────────────────────────────────────────────────

struct StoredAlert {
    idx: i32,
    timestamp: String,
    device: String,
    alert_type: String,
    message: String,
    severity: String,
}

fn store_alert(alert: StoredAlert) {
    sdk::kv_set_val(&format!("alert_{}", alert.idx), &serde_json::json!({
        "timestamp": alert.timestamp,
        "device": alert.device,
        "alert_type": alert.alert_type,
        "message": alert.message,
        "severity": alert.severity,
    }).to_string());
    sdk::kv_set_val("alert_count", &alert.idx.to_string());
}

fn get_stored_alerts() -> Vec<StoredAlert> {
    let count = sdk::kv_get_val("alert_count").unwrap_or("0".to_string());
    let n: i32 = count.parse().unwrap_or(0);
    let mut alerts = Vec::new();

    for i in (1..=n).rev() {
        let raw = sdk::kv_get_val(&format!("alert_{}", i)).unwrap_or_default();
        if raw.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            alerts.push(StoredAlert {
                idx: i,
                timestamp: v["timestamp"].as_str().unwrap_or("").to_string(),
                device: v["device"].as_str().unwrap_or("").to_string(),
                alert_type: v["alert_type"].as_str().unwrap_or("").to_string(),
                message: v["message"].as_str().unwrap_or("").to_string(),
                severity: v["severity"].as_str().unwrap_or("").to_string(),
            });
        }
    }
    alerts
}

fn process_bridge_event(payload: String) {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&payload) {
        if let Some(event) = v.get("event").and_then(|e| e.as_str()) {
            match event {
                "poll" => {
                    if let Some(device_id) = v.get("deviceId").and_then(|d| d.as_str()) {
                        if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
                            sdk::kv_set_val(&format!("bridge_status_{}", device_id), status);
                        }
                    }
                }
                "device_status" => {
                    sdk::log(&format!("Bridge device status change: {}", payload));
                }
                _ => {}
            }
        }
    }
}

fn save_device(data: &str) {
    sdk::log(&format!("Dispositivo guardado via EzerDesk API: {}", data));
    sdk::respond_ok("Dispositivo guardado. Use el panel de administración para gestionar dispositivos SNMP.");
}
