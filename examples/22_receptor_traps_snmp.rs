// Ejemplo 22: Receptor de Traps SNMP
// Features: KV store, Events, Table, Charts, SnmpTrapReceived event
// Demuestra: Recepción de traps SNMP via evento nativo snmp.trap.received del bus de EzerDesk

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
                .description("Procesa traps SNMP nativos del bus de eventos EzerDesk (snmp.trap.received)")
                .version("2.0.0");
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

        // Trap recibido desde el bus de eventos nativo (snmp.trap.received)
        PluginEvent::SnmpTrapReceived { payload, agent_ip, .. } => {
            process_native_trap(&agent_ip, &payload);
        }

        // Bridge webhook (trap via bridge Node.js)
        PluginEvent::BridgeWebhook { payload } => {
            sdk::log(&format!("Bridge trap: {}", payload));
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&payload) {
                if let Some(event_type) = v.get("event").and_then(|e| e.as_str()) {
                    if event_type == "trap" {
                        let source = v.get("source").and_then(|s| s.as_str()).unwrap_or("unknown");
                        let raw = v.get("raw").and_then(|r| r.as_str()).unwrap_or("");
                        process_native_trap(source, raw);
                    }
                }
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "add_rule" => render_add_rule_form(),
                "save_rule" => save_rule(&data),
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
    let rules_count = get_active_rules_count();

    // Build recent traps table from KV store
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let recent = get_recent_traps(20);
    for t in &recent {
        rows.push(vec![&t.timestamp, &t.source, &t.trap_oid, &t.message, &t.severity]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Receptor de Traps SNMP Nativo", vec![
            sdk::text("Traps recibidos desde el bus de eventos de EzerDesk (snmpm)", "info"),
            sdk::text("No requiere infraestructura adicional — corre dentro del BEAM VM", "info"),
            sdk::divider(),
            sdk::text(&format!("📊 Traps recibidos: {}", trap_count), "info"),
            sdk::text(&format!("🔴 Críticos: {}", critical_count), "warning"),
            sdk::text(&format!("📋 Reglas activas: {}", rules_count), "info"),
            sdk::divider(),
            sdk::chart("Traps por Severidad", vec![
                ("Críticos", critical_count as f64),
                ("Altos", (trap_count - critical_count) as f64),
            ], "doughnut"),
        ]),

        sdk::card("Últimos Traps", vec![
            sdk::table(
                vec!["Timestamp", "Origen", "OID Trap", "Variable", "Severidad"],
                rows,
            ),
        ]),

        sdk::card("Acciones", vec![
            sdk::button("Ver Historial", "trap_history", "secondary"),
            sdk::button("Exportar Traps", "export_traps", "outline"),
            sdk::button("Limpiar", "clear_history", "danger"),
        ]),
    ]);
}

fn render_trap_history() {
    let traps = get_recent_traps(100);
    let mut rows: Vec<Vec<&str>> = Vec::new();
    for t in &traps {
        rows.push(vec![&t.id, &t.timestamp, &t.source, &t.trap_oid, &t.message, &t.severity]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Historial de Traps SNMP", vec![
            sdk::text(&format!("Mostrando últimos {} traps", traps.len()), "info"),
            sdk::table(
                vec!["ID", "Timestamp", "Origen", "OID", "Variable", "Severidad"],
                rows,
            ),
            sdk::button("Volver", "snmp_traps", "outline"),
        ]),
    ]);
}

fn render_trap_rules() {
    let rules = get_trap_rules();
    let mut rows: Vec<Vec<&str>> = Vec::new();
    for r in &rules {
        let active = if r.active { "Sí" } else { "No" };
        rows.push(vec![&r.name, &r.oid, &r.action, &r.severity, active]);
    }

    sdk::respond(sdk::widgets![
        sdk::card("Reglas de Procesamiento de Traps", vec![
            sdk::text("Define cómo se procesan los traps según su OID", "info"),
            sdk::table(
                vec!["Nombre", "OID", "Acción", "Severidad", "Activa"],
                rows,
            ),
            sdk::button("Agregar Regla", "add_rule", "primary"),
            sdk::button("Volver", "snmp_traps", "outline"),
        ]),
    ]);
}

fn render_add_rule_form() {
    sdk::respond(sdk::widgets![
        sdk::card("Nueva Regla de Trap", vec![
            sdk::input("Nombre", "rule_name", "linkDown"),
            sdk::input("OID del Trap", "trap_oid", "1.3.6.1.6.3.1.1.5.3"),
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

// ── Trap processing ──────────────────────────────────────────────────────

fn process_native_trap(agent_ip: &str, payload: &str) {
    sdk::log(&format!("Trap SNMP nativo desde {}: {}", agent_ip, payload));

    // Parse trap payload JSON
    let trap: TrapPayload = match serde_json::from_str(payload) {
        Ok(t) => t,
        Err(_) => {
            sdk::log("Error parsing trap payload, storing raw");
            store_raw_trap(agent_ip, payload, "unknown", "unknown");
            return;
        }
    };

    let target = trap.target.as_deref().unwrap_or(agent_ip);

    // Extract trap OID from first varbind (snmpTrapOID is the key OID)
    let trap_oid = trap.varbinds.first()
        .map(|v| v.oid.as_str())
        .unwrap_or("unknown");

    // Find message from first varbind value
    let message = trap.varbinds.first()
        .map(|v| v.value.as_str())
        .unwrap_or("");

    // Determine severity based on OID
    let severity = classify_trap_severity(trap_oid);

    // Store trap
    store_trap(&TrapRecord {
        id: format!("trap_{}", sdk::now()),
        timestamp: format_epoch(trap.timestamp),
        source: target.to_string(),
        trap_oid: trap_oid.to_string(),
        message: message.to_string(),
        severity,
        payload: payload.to_string(),
    });

    // Check rules
    let rules = get_trap_rules();
    for rule in &rules {
        if rule.active && trap_oid.contains(&rule.oid) {
            sdk::log(&format!("Rule matched: {} → {}", rule.name, rule.action));
            match rule.action.as_str() {
                "notify" => sdk::log(&format!("NOTIFY: {} from {}", rule.name, target)),
                "ticket" => sdk::log(&format!("TICKET: {} from {}", rule.name, target)),
                _ => {}
            }
        }
    }

    sdk::log(&format!("Trap processed: {} from {}, severity: {}", trap_oid, target, severity));
}

fn classify_trap_severity(oid: &str) -> &'static str {
    match oid {
        // Cold/Warm Start, Authentication Failure
        "1.3.6.1.6.3.1.1.5.1" | "1.3.6.1.6.3.1.1.5.2" => "Crítica",
        "1.3.6.1.6.3.1.1.5.5" => "Crítica",
        // linkDown
        "1.3.6.1.6.3.1.1.5.3" => "Alta",
        // linkUp
        "1.3.6.1.6.3.1.1.5.4" => "Media",
        // Default
        _ => {
            if oid.contains("1.3.6.1.2.1.") && oid.contains(".1.0") {
                "Alta"
            } else {
                "Info"
            }
        }
    }
}

fn store_trap(trap: &TrapRecord) {
    // Increment counters
    let count = sdk::kv_get_val("trap_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("trap_count", &count.to_string());

    if trap.severity == "Crítica" || trap.severity == "Alta" {
        let crit = sdk::kv_get_val("critical_traps")
            .unwrap_or("0".to_string())
            .parse::<i32>()
            .unwrap_or(0) + 1;
        sdk::kv_set_val("critical_traps", &crit.to_string());
    }

    // Store in KV (rotate at 1000 records)
    let trap_json = serde_json::json!(trap).to_string();
    sdk::kv_set_val(&format!("trap_record_{}", count % 1000), &trap_json);
    sdk::kv_set_val("trap_last_id", &count.to_string());
}

fn store_raw_trap(agent_ip: &str, payload: &str, trap_oid: &str, message: &str) {
    store_trap(&TrapRecord {
        id: format!("raw_{}", sdk::now()),
        timestamp: format_epoch(0),
        source: agent_ip.to_string(),
        trap_oid: trap_oid.to_string(),
        message: message.to_string(),
        severity: "Info".to_string(),
        payload: payload.to_string(),
    });
}

fn get_recent_traps(limit: i32) -> Vec<TrapRecord> {
    let last_id = sdk::kv_get_val("trap_last_id").unwrap_or("0".to_string());
    let n: i32 = last_id.parse().unwrap_or(0);
    let mut traps = Vec::new();

    for i in (0..=n.min(1000)).rev() {
        if traps.len() >= limit as usize { break; }
        let raw = sdk::kv_get_val(&format!("trap_record_{}", i % 1000)).unwrap_or_default();
        if raw.is_empty() { continue; }
        if let Ok(t) = serde_json::from_str::<TrapRecord>(&raw) {
            traps.push(t);
        }
    }
    traps
}

fn get_active_rules_count() -> i32 {
    let raw = sdk::kv_get_val("trap_rules").unwrap_or("[]".to_string());
    if let Ok(rules) = serde_json::from_str::<Vec<TrapRule>>(&raw) {
        rules.iter().filter(|r| r.active).count() as i32
    } else {
        0
    }
}

fn get_trap_rules() -> Vec<TrapRule> {
    let raw = sdk::kv_get_val("trap_rules").unwrap_or("[]".to_string());
    serde_json::from_str(&raw).unwrap_or_default()
}

fn clear_trap_history() {
    sdk::kv_set_val("trap_count", "0");
    sdk::kv_set_val("critical_traps", "0");
    sdk::kv_set_val("trap_last_id", "0");
    sdk::respond_ok("Historial de traps limpiado");
}

fn export_traps() {
    let traps = get_recent_traps(1000);
    let json = serde_json::to_string_pretty(&traps).unwrap_or("[]".to_string());
    sdk::kv_set_val("export_last", &json);
    sdk::log(&format!("Exportados {} traps", traps.len()));
    sdk::respond_ok(&format!("{} traps exportados a KV (key: export_last)", traps.len()));
}

fn save_rule(data: &str) {
    sdk::log(&format!("Guardando regla: {}", data));

    let mut rules = get_trap_rules();
    if let Ok(new_rule) = serde_json::from_str::<TrapRule>(data) {
        rules.push(new_rule);
        if let Ok(json) = serde_json::to_string(&rules) {
            sdk::kv_set_val("trap_rules", &json);
        }
    }

    sdk::respond_ok("Regla guardada exitosamente");
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn format_epoch(ts: i64) -> String {
    if ts == 0 {
        return "desconocido".to_string();
    }
    let secs = ts as u64;
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else {
        format!("{}h {}m", hours, mins)
    }
}

// ── Data structures ──────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Varbind {
    oid: String,
    value: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TrapPayload {
    #[serde(default)]
    target: Option<String>,
    varbinds: Vec<Varbind>,
    count: i32,
    timestamp: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TrapRecord {
    id: String,
    timestamp: String,
    source: String,
    trap_oid: String,
    message: String,
    severity: String,
    #[serde(default)]
    payload: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TrapRule {
    name: String,
    oid: String,
    action: String,
    severity: String,
    #[serde(default = "default_active")]
    active: bool,
}

fn default_active() -> bool { true }
