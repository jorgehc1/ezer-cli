// Ejemplo 23: Bridge Integration SNMP
// Features: Bridge webhook, KV store, Charts, Table, Data persistence
// Demuestra: Plugin que recibe eventos del snmp-bridge (Node.js) y persiste métricas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("bridge_monitor", "Bridge SNMP", "server-line")
                        .category("sistema")
                        .priority(24)
                )
                .name("SNMP Bridge Integration")
                .description("Recibe datos del snmp-bridge (Node.js) via webhook y los persiste en KV store")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "bridge_monitor" => render_bridge_dashboard(),
                "device_detail" => render_device_detail(),
                "config" => render_bridge_config(),
                _ => {}
            }
        }

        // Bridge webhook events from snmp-bridge
        PluginEvent::BridgeWebhook { payload } => {
            process_bridge_webhook(payload);
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "refresh" => render_bridge_dashboard(),
                "save_config" => save_bridge_config(&data),
                "clear_data" => clear_poll_data(),
                _ => {}
            }
        }

        _ => {}
    }
    0
}

fn render_bridge_dashboard() {
    // Load device states from KV
    let devices_count = get_stored_device_count();
    let latest_polls = get_latest_polls(10);

    let mut rows: Vec<Vec<&str>> = Vec::new();
    for p in &latest_polls {
        let status_icon = match p.status.as_str() {
            "up" => "✅",
            "down" => "🔴",
            "error" => "⚠️",
            _ => "❓",
        };
        rows.push(vec![
            &p.timestamp,
            &p.device_name,
            &p.host,
            &format!("{} {}", status_icon, p.status),
            &p.oid,
            &p.value,
        ]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("SNMP Bridge — Monitoreo", vec![
            sdk::text("Datos recibidos del snmp-bridge companion service via webhook", "info"),
            sdk::text(&format!("📡 Dispositivos monitoreados: {}", devices_count), "info"),
            sdk::text(&format!("⏱ Último poll: {}", latest_polls.first().map(|p| &p.timestamp).unwrap_or(&"Nunca".to_string())), "info"),
            sdk::divider(),
            sdk::chart("Estado de Dispositivos", get_device_chart_data(), "doughnut"),
        ]),

        sdk::card("Últimos Polls Recibidos", vec![
            sdk::table(
                vec!["Timestamp", "Dispositivo", "Host", "Estado", "OID", "Valor"],
                rows,
            ),
            sdk::button("Refrescar", "refresh", "primary"),
            sdk::button("Limpiar Datos", "clear_data", "danger"),
        ]),

        sdk::card("Configuración del Bridge", vec![
            sdk::button("Ver Configuración", "config", "outline"),
        ]),
    ]);
}

fn render_device_detail() {
    let polls = get_latest_polls(50);
    let mut rows: Vec<Vec<&str>> = Vec::new();
    for p in &polls {
        rows.push(vec![&p.timestamp, &p.device_name, &p.oid, &p.value, &p.status]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Detalle de Polls", vec![
            sdk::table(
                vec!["Timestamp", "Dispositivo", "OID", "Valor", "Estado"],
                rows,
            ),
            sdk::button("Volver", "bridge_monitor", "outline"),
        ]),
    ]);
}

fn render_bridge_config() {
    let url = sdk::kv_get_val("bridge_url").unwrap_or("http://localhost:3001".to_string());
    let interval = sdk::kv_get_val("poll_interval").unwrap_or("60".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Configuración del Bridge SNMP", vec![
            sdk::text("Configura la conexión con el snmp-bridge companion service", "info"),
            sdk::divider(),
            sdk::input("URL del Bridge", "bridge_url", &url),
            sdk::number_input_with_limits("Intervalo de Poll (s)", "poll_interval", &interval, &interval, 10.0, 3600.0, 10.0),
            sdk::button("Guardar", "save_config", "primary"),
            sdk::button("Volver", "bridge_monitor", "outline"),
        ]),
    ]);
}

// ── Webhook processing ──────────────────────────────────────────────────

fn process_bridge_webhook(payload: String) {
    match serde_json::from_str::<BridgePayload>(&payload) {
        Ok(data) => {
            sdk::log(&format!("Bridge event: {}", data.event));

            match data.event.as_str() {
                "poll" => handle_poll_event(data),
                "device_status" => handle_status_event(data),
                "heartbeat" => {
                    sdk::kv_set_val("bridge_last_heartbeat", &sdk::now().to_string());
                    if let Some(uptime) = data.uptime_sec {
                        sdk::kv_set_val("bridge_uptime", &uptime.to_string());
                    }
                }
                _ => sdk::log(&format!("Unknown bridge event: {}", data.event)),
            }
        }
        Err(e) => {
            sdk::log(&format!("Invalid bridge payload: {} — {}", e, payload));
        }
    }
}

fn handle_poll_event(data: BridgePayload) {
    let device_id = data.device_id.unwrap_or_default();
    let host = data.host.unwrap_or_default();
    let status = data.status.unwrap_or("unknown".to_string());
    let timestamp = data.timestamp.map(|t| format_ts(t)).unwrap_or_else(|| sdk::now().to_string());

    // Update device status in KV
    sdk::kv_set_val(&format!("dev_{}_status", device_id), &status);
    sdk::kv_set_val(&format!("dev_{}_last_poll", device_id), &timestamp);
    sdk::kv_set_val(&format!("dev_{}_host", device_id), &host);

    if status == "up" {
        sdk::kv_set_val(&format!("dev_{}_last_ok", device_id), &timestamp);
    }

    // Store poll results in circular buffer
    if let Some(results) = &data.results {
        let poll_idx = sdk::kv_get_val("poll_counter")
            .unwrap_or("0".to_string())
            .parse::<i32>()
            .unwrap_or(0);
        let idx = poll_idx % 1000;

        let entry = PollEntry {
            ts: timestamp.clone(),
            device_id: device_id.clone(),
            host: host.clone(),
            status: status.clone(),
            oid: results.first().map(|r| r.oid.clone()).unwrap_or_default(),
            value: results.first().map(|r| r.value.clone()).unwrap_or_default(),
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            sdk::kv_set_val(&format!("poll_{}", idx), &json);
        }

        sdk::kv_set_val("poll_counter", &(poll_idx + 1).to_string());
        sdk::kv_set_val("poll_last_ts", &timestamp);
    }

    // Track device names for listing
    let device_name = data.device_name.as_deref().unwrap_or(&host);
    let stored_devices = sdk::kv_get_val("device_list").unwrap_or("[]".to_string());
    let mut dev_list: Vec<String> = serde_json::from_str(&stored_devices).unwrap_or_default();
    if !dev_list.contains(&device_id) {
        dev_list.push(device_id.clone());
        if let Ok(json) = serde_json::to_string(&dev_list) {
            sdk::kv_set_val("device_list", &json);
        }
    }
    sdk::kv_set_val(&format!("dev_{}_name", device_id), device_name);
}

fn handle_status_event(data: BridgePayload) {
    let device_id = data.device_id.unwrap_or_default();
    let status = data.status.unwrap_or("unknown".to_string());
    let prev_status = data.previous_status.as_deref().unwrap_or("unknown");
    let error = data.error.as_deref().unwrap_or("");

    sdk::log(&format!("Device {} status changed: {} → {} ({})", device_id, prev_status, status, error));
    sdk::kv_set_val(&format!("dev_{}_status", device_id), &status);
    sdk::kv_set_val(&format!("dev_{}_last_status_change", device_id), &sdk::now().to_string());

    if !error.is_empty() {
        sdk::kv_set_val(&format!("dev_{}_last_error", device_id), error);
    }
}

// ── Data structures ─────────────────────────────────────────────────────

#[derive(serde::Deserialize, Debug)]
struct BridgePayload {
    #[serde(default)]
    event: String,
    #[serde(default)]
    device_id: Option<String>,
    #[serde(default)]
    device_name: Option<String>,
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    previous_status: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    timestamp: Option<i64>,
    #[serde(default)]
    uptime_sec: Option<i64>,
    #[serde(default)]
    results: Option<Vec<PollResult>>,
    #[serde(default)]
    devices_count: Option<i32>,
    #[serde(default)]
    traps_count: Option<i32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct PollResult {
    #[serde(default)]
    oid: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    value: String,
}

#[derive(serde::Serialize, Debug)]
struct PollEntry {
    ts: String,
    device_id: String,
    host: String,
    status: String,
    oid: String,
    value: String,
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn get_stored_device_count() -> i32 {
    let raw = sdk::kv_get_val("device_list").unwrap_or("[]".to_string());
    serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default().len() as i32
}

fn get_latest_polls(count: usize) -> Vec<PollEntry> {
    let counter = sdk::kv_get_val("poll_counter")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    let mut polls = Vec::new();
    for i in (0..counter).rev() {
        if polls.len() >= count { break; }
        let raw = sdk::kv_get_val(&format!("poll_{}", i % 1000)).unwrap_or_default();
        if raw.is_empty() { continue; }
        if let Ok(entry) = serde_json::from_str::<PollEntry>(&raw) {
            polls.push(entry);
        }
    }
    polls
}

fn get_device_chart_data() -> Vec<(&'static str, f64)> {
    let device_list = sdk::kv_get_val("device_list").unwrap_or("[]".to_string());
    let devices: Vec<String> = serde_json::from_str(&device_list).unwrap_or_default();
    let mut up = 0_f64;
    let mut down = 0_f64;
    let mut unknown = 0_f64;

    for id in &devices {
        let status = sdk::kv_get_val(&format!("dev_{}_status", id)).unwrap_or_default();
        match status.as_str() {
            "up" => up += 1.0,
            "down" => down += 1.0,
            _ => unknown += 1.0,
        }
    }

    vec![("En Línea", up), ("Fuera de Línea", down), ("Desconocido", unknown)]
}

fn format_ts(ts: i64) -> String {
    let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts } as u64;
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    format!("{:02}:{:02}:{:02}", days * 24 + hours, mins, secs % 60)
}

fn save_bridge_config(data: &str) {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(url) = v.get("bridge_url").and_then(|s| s.as_str()) {
            sdk::kv_set_val("bridge_url", url);
        }
        if let Some(interval) = v.get("poll_interval").and_then(|s| s.as_str()) {
            sdk::kv_set_val("poll_interval", interval);
        }
        sdk::respond_ok("Configuración guardada");
    } else {
        sdk::respond_error("JSON inválido");
    }
}

fn clear_poll_data() {
    for i in 0..1000 {
        sdk::kv_set_val(&format!("poll_{}", i), "");
    }
    sdk::kv_set_val("poll_counter", "0");
    sdk::kv_set_val("device_list", "[]");
    sdk::respond_ok("Datos de poll eliminados");
}
