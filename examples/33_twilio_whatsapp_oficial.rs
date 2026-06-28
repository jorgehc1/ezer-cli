// Ejemplo 33: WhatsApp Business API via Twilio
// Features: Twilio API oficial, SMS/WhatsApp, KV store, Chat UI
// Demuestra: Plugin que usa la API oficial de Twilio para WhatsApp
//            0% riesgo de ban — canal oficial soportado por Meta
// Diferencia con ejemplo 32: NO usa Baileys, usa API oficial

use ezerdesk_sdk as sdk;
use sdk::prelude::*;
use serde::Serialize;

// ── Constantes ──────────────────────────────────────────────────────────

const KV_CONTACTS: &str = "twilio_contacts";
const KV_MSG_COUNTER: &str = "twilio_msg_counter";
const TWILIO_WHATSAPP_PREFIX: &str = "whatsapp:+593"; // Ecuador default

fn get_default_to() -> String {
    sdk::kv_get_val("twilio_default_to").unwrap_or_else(|| format!("{}999999999", TWILIO_WHATSAPP_PREFIX))
}

// ── Tipos ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct SentMessage {
    id: u64,
    to: String,
    body: String,
    timestamp: u64,
    status: String,
}

// ── Entry point ─────────────────────────────────────────────────────────

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::SmsInbound { id_organizacion: _, from, to, body } => {
            sdk::log(&format!("SMS entrante de {} a {}: {}", from, to, body));

            // Store incoming message in KV store
            let counter = get_msg_counter() + 1;
            let msg = SentMessage {
                id: counter,
                to: from.clone(),
                body: body.clone(),
                timestamp: sdk::now(),
                status: "recibido".to_string(),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                sdk::kv_set_val(&format!("twilio_msg_{}", counter % 1000), &json);
            }
            sdk::kv_set_val(KV_MSG_COUNTER, &counter.to_string());
            save_contact(&from);
        }

        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(NavItem::new("twilio_send", "WhatsApp Twilio", "whatsapp-line")
                    .category("operaciones").priority(3))
                .nav_item(NavItem::new("twilio_history", "Historial", "chat-history-line")
                    .category("operaciones").priority(5))
                .nav_item(NavItem::new("twilio_config", "Config. Twilio", "settings-line")
                    .category("operaciones").priority(9))
                .name("WhatsApp via Twilio")
                .description("API oficial de WhatsApp Business via Twilio. Sin riesgo de ban.")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "twilio_send" => render_send_page(),
                "twilio_history" => render_history_page(),
                "twilio_config" => render_config_page(),
                _ => sdk::respond(vec![sdk::text("Página no encontrada", "error")]),
            }
        }

        PluginEvent::PluginAction { action, data } => {
            handle_action(&action, &data);
        }

        _ => {}
    }
    0
}

// ── Página 1: Enviar mensaje ────────────────────────────────────────────

fn render_send_page() {
    let default_to = get_default_to();
    let mut widgets: Vec<UiWidget> = vec![
        sdk::text("WhatsApp Business API — Twilio", "heading"),
        sdk::text("Canal oficial de Meta. Los mensajes se envían via API REST de Twilio.", "success"),
        sdk::divider(),
        sdk::card("Nuevo Mensaje", vec![
            sdk::input("Número (con código país)", "to_number", &default_to),
            sdk::textarea("Mensaje", "message_body", "Escribe tu mensaje aquí..."),
            sdk::button("Enviar por WhatsApp", "send_twilio", "primary"),
        ]),
    ];

    // Show last 5 messages
    let mut rows: Vec<Vec<&str>> = Vec::new();
    let counter = get_msg_counter();
    for i in (0.max(counter as i64 - 5) as u64..counter).rev() {
        let raw = sdk::kv_get_val(&format!("twilio_msg_{}", i % 1000)).unwrap_or_default();
        if raw.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            let to = v["to"].as_str().unwrap_or("");
            let body = v["body"].as_str().unwrap_or("");
            let short = if body.len() > 35 { &body[..32] } else { body };
            let status = v["status"].as_str().unwrap_or("");
            rows.push(vec![to, short, status]);
        }
    }

    if !rows.is_empty() {
        widgets.push(sdk::divider());
        widgets.push(sdk::card("Últimos envíos", vec![
            sdk::table(vec!["Número", "Mensaje", "Estado"], rows),
        ]));
    }

    sdk::respond(widgets);
}

// ── Página 2: Historial de mensajes ─────────────────────────────────────

fn render_history_page() {
    let counter = get_msg_counter();
    if counter == 0 {
        return sdk::respond(vec![
            sdk::text("Historial de WhatsApp", "heading"),
            sdk::text("Aún no hay mensajes enviados.", "info"),
        ]);
    }

    let mut rows: Vec<Vec<&str>> = Vec::new();
    for i in (0..counter).rev() {
        if rows.len() >= 50 { break; }
        let raw = sdk::kv_get_val(&format!("twilio_msg_{}", i % 1000)).unwrap_or_default();
        if raw.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            let to = v["to"].as_str().unwrap_or("");
            let body = v["body"].as_str().unwrap_or("");
            let ts = v["timestamp"].as_u64().unwrap_or(0);
            let status = v["status"].as_str().unwrap_or("");
            let short = if body.len() > 50 { &body[..47] } else { body };
            rows.push(vec![to, short, &format_ts(ts), status]);
        }
    }

    sdk::respond(vec![
        sdk::text(&format!("Historial — {} mensajes enviados", counter), "heading"),
        sdk::table(vec!["Número", "Mensaje", "Fecha", "Estado"], rows),
        sdk::button("Volver", "go_send", "outline"),
    ]);
}

// ── Página 3: Configuración ─────────────────────────────────────────────

fn render_config_page() {
    let default_to = get_default_to();
    let contacts_raw = sdk::kv_get_val(KV_CONTACTS).unwrap_or("[]".to_string());
    let contacts: Vec<String> = serde_json::from_str(&contacts_raw).unwrap_or_default();
    let mut contact_rows: Vec<Vec<&str>> = Vec::new();
    for c in &contacts {
        contact_rows.push(vec![c]);
    }

    sdk::respond(vec![
        sdk::card("Configuración Twilio WhatsApp", vec![
            sdk::text("Requisitos previos:", "info"),
            sdk::text("1. Cuenta de Twilio con WhatsApp Business API activado", "muted"),
            sdk::text("2. Configurar TWILIO_SID, TWILIO_TOKEN, TWILIO_FROM en el backend", "muted"),
            sdk::text("3. Configurar webhook entrante en Twilio Console:", "muted"),
            sdk::text("   → {APP_URL}/api/v1/webhooks/sms/twilio", "code"),
            sdk::divider(),
            sdk::input("Número por defecto (whatsapp:+593...)", "default_to", &default_to),
            sdk::button("Guardar", "save_config", "primary"),
        ]),
        sdk::card("Contactos frecuentes", vec![
            if contact_rows.is_empty() {
                vec![sdk::text("Agrega contactos enviándoles un mensaje.", "info")]
            } else {
                vec![
                    sdk::table(vec!["Número"], contact_rows),
                    sdk::button("Limpiar contactos", "clear_contacts", "danger"),
                ]
            },
        ]),
        sdk::button("Volver", "go_send", "outline"),
    ]);
}

// ── Action handler ──────────────────────────────────────────────────────

fn handle_action(action: &str, data: &serde_json::Value) {
    match action {
        "send_twilio" => {
            let to = data.get("to_number")
                .and_then(|v| v.as_str())
                .map(|s| {
                    if s.starts_with("whatsapp:") { s.to_string() }
                    else if s.starts_with('+') { format!("whatsapp:{}", s) }
                    else { format!("{}", s) }
                })
                .unwrap_or_else(get_default_to);

            let body = data.get("message_body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if body.is_empty() {
                sdk::respond_error("El mensaje no puede estar vacío");
                return;
            }

            // Send via Twilio using SDK's built-in SMS function
            sdk::send_sms(&to, &body);

            // Store in KV for history
            let counter = get_msg_counter() + 1;
            let msg = SentMessage {
                id: counter,
                to: to.clone(),
                body: body.clone(),
                timestamp: sdk::now(),
                status: "enviado".to_string(),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                sdk::kv_set_val(&format!("twilio_msg_{}", counter % 1000), &json);
            }
            sdk::kv_set_val(KV_MSG_COUNTER, &counter.to_string());

            // Save as contact
            save_contact(&to);

            sdk::respond_ok(&format!("Mensaje enviado a {}", to));
        }

        "save_config" => {
            if let Some(to) = data.get("default_to").and_then(|v| v.as_str()) {
                sdk::kv_set_val("twilio_default_to", to);
                sdk::respond_ok("Configuración guardada");
            } else {
                sdk::respond_error("Número inválido");
            }
        }

        "clear_contacts" => {
            sdk::kv_set_val(KV_CONTACTS, "[]");
            sdk::respond_ok("Contactos limpiados");
        }

        "go_send" => render_send_page(),
        "go_history" => render_history_page(),

        _ => sdk::respond_error(&format!("Acción desconocida: {}", action)),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn get_msg_counter() -> u64 {
    sdk::kv_get_val(KV_MSG_COUNTER)
        .unwrap_or("0".to_string())
        .parse()
        .unwrap_or(0)
}

fn save_contact(number: &str) {
    let raw = sdk::kv_get_val(KV_CONTACTS).unwrap_or("[]".to_string());
    let mut list: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
    if !list.contains(&number.to_string()) {
        list.push(number.to_string());
        if let Ok(json) = serde_json::to_string(&list) {
            sdk::kv_set_val(KV_CONTACTS, &json);
        }
    }
}

fn format_ts(ts: u64) -> String {
    let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    format!("{:02}:{:02}", hours, mins)
}
