// Ejemplo 32: WhatsApp Chat Conversacional
// Features: Bridge webhook, Chat burbujas, Contactos, Data Models, Input
// Demuestra: Plugin que replica la experiencia de WhatsApp Web
//            Lista de contactos + vista de conversación con burbujas

use ezerdesk_sdk as sdk;
use sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── Tipos de datos ──────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BridgePayload {
    #[serde(rename = "sessionId")]
    session_id: String,
    event: String,
    status: Option<String>,
    message: Option<BridgeMessage>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BridgeMessage {
    id: String,
    from: String,
    push_name: Option<String>,
    timestamp: u64,
    text: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BridgeSessionState {
    status: String,
    qr: Option<String>,
    qr_ascii: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChatRecord {
    session_id: String,
    chat_jid: String,
    from_jid: String,
    push_name: String,
    body: String,
    timestamp: u64,
    ingested_at: u64,
}

// ── Constantes ──────────────────────────────────────────────────────────

const KV_BRIDGE_URL: &str = "bridge_url";
const KV_SESSION_ID: &str = "wa_session_id";
const DEFAULT_BRIDGE_URL: &str = "http://localhost:3000";

fn get_bridge_url() -> String {
    sdk::kv_get_val(KV_BRIDGE_URL).unwrap_or_else(|| DEFAULT_BRIDGE_URL.to_string())
}

fn get_session_id() -> String {
    sdk::kv_get_val(KV_SESSION_ID).unwrap_or_default()
}

fn display_name(record: &ChatRecord) -> String {
    if !record.push_name.is_empty() {
        record.push_name.clone()
    } else {
        record.chat_jid.split('@').next().unwrap_or(&record.chat_jid).to_string()
    }
}

fn is_group(jid: &str) -> bool {
    jid.contains("@g.us")
}

fn is_from_agent(record: &ChatRecord, current_session: &str) -> bool {
    // Messages from the session itself are outgoing (agent → contact)
    // Messages where from_jid != chat_jid are outgoing
    record.from_jid.contains(&current_session) || record.from_jid == record.session_id
}

// ── HTTP helpers ────────────────────────────────────────────────────────

fn http_get(url: &str) -> Option<HttpResponse> {
    sdk::http_request(&HttpRequest {
        method: "GET".to_string(), url: url.to_string(),
        body: "".to_string(), headers: vec![],
    })
}

fn http_post(url: &str, body: &str) -> Option<HttpResponse> {
    sdk::http_request(&HttpRequest {
        method: "POST".to_string(), url: url.to_string(),
        body: body.to_string(),
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
    })
}

fn http_delete(url: &str) -> Option<HttpResponse> {
    sdk::http_request(&HttpRequest {
        method: "DELETE".to_string(), url: url.to_string(),
        body: "".to_string(), headers: vec![],
    })
}

// ── Entry point ─────────────────────────────────────────────────────────

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            sdk::create_data_model(
                "whatsapp_messages",
                "Mensajes de WhatsApp",
                r#"{"fields": [
                    {"name": "session_id", "type": "string"},
                    {"name": "chat_jid", "type": "string"},
                    {"name": "from_jid", "type": "string"},
                    {"name": "push_name", "type": "string"},
                    {"name": "body", "type": "string"},
                    {"name": "timestamp", "type": "number"},
                    {"name": "ingested_at", "type": "number"}
                ]}"#
            );

            let meta = PluginMetadata::new()
                .nav_item(NavItem::new("wa_connect", "WhatsApp", "whatsapp-line")
                    .category("operaciones").priority(4))
                .nav_item(NavItem::new("wa_chats", "Chats WhatsApp", "chat-1-line")
                    .category("operaciones").priority(3))
                .nav_item(NavItem::new("wa_config", "Config. WA", "settings-line")
                    .category("operaciones").priority(9))
                .name("WhatsApp Chat")
                .description("Interfaz conversacional de WhatsApp con QR, contactos y burbujas de chat")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "wa_connect" => render_connect_page(),
                "wa_chats" => render_chats_page(),
                "wa_config" => render_config_page(),
                _ => sdk::respond(vec![sdk::text("Página no encontrada", "error")]),
            }
        }

        PluginEvent::PluginAction { action, data } => {
            handle_action(&action, &data);
        }

        PluginEvent::BridgeWebhook { payload } => {
            handle_webhook(&payload);
        }

        _ => {}
    }
    0
}

// ── Página 1: Conexión QR ──────────────────────────────────────────────

fn render_connect_page() {
    let bridge_url = get_bridge_url();
    let session_id = get_session_id();
    let mut widgets: Vec<UiWidget> = vec![
        sdk::text("Vincular WhatsApp con EzerDesk", "heading"),
        sdk::text("Conecta un número de WhatsApp para monitorear conversaciones.", "info"),
    ];

    if session_id.is_empty() {
        widgets.push(sdk::card("Nueva Conexión", vec![
            sdk::input("ID de Sesión (ej: +593999999999)", "session_id_input",
                       "Número o identificador único"),
            sdk::button("Generar QR", "conectar", "primary"),
        ]));
    } else {
        widgets.push(sdk::card(&format!("Sesión activa: {}", session_id), vec![
            sdk::badge("Conectando...", "info"),
            sdk::button("Desconectar", &format!("desconectar:{}", session_id), "danger"),
        ]));
    }

    // Fetch session state from bridge
    let state_url = format!("{}/sessions/{}", bridge_url, session_id);
    if let Some(res) = http_get(&state_url) {
        if let Ok(state) = serde_json::from_str::<BridgeSessionState>(&res.body) {
            match state.status.as_str() {
                "qr" => {
                    if let Some(qr) = &state.qr_ascii {
                        widgets.push(sdk::card("Escanea el QR", vec![
                            sdk::text(&qr, "code"),
                        ]));
                    }
                    widgets.push(sdk::text("Abre WhatsApp → Menú → Dispositivos vinculados", "info"));
                }
                "connected" => {
                    widgets.push(sdk::card("✅ Conectado", vec![
                        sdk::text(&format!("Sesión {} activa. Ve a Chats WhatsApp.", session_id), "success"),
                        sdk::button("Ir a Chats", "go_chats", "primary"),
                    ]));
                }
                "connecting" => {
                    widgets.push(sdk::text("Conectando con WhatsApp...", "info"));
                }
                _ => {
                    if let Some(err) = &state.error {
                        widgets.push(sdk::text(&format!("Error: {}", err), "error"));
                    }
                }
            }
        }
    }

    sdk::respond(widgets);
}

// ── Página 2: Chats (lista de contactos + conversación) ─────────────────

fn render_chats_page() {
    let session_id = get_session_id();
    if session_id.is_empty() {
        return sdk::respond(vec![
            sdk::text("Primero vincula un número de WhatsApp", "heading"),
            sdk::button("Ir a Conexión", "go_connect", "primary"),
        ]);
    }

    // Read all messages from data store
    let messages = fetch_messages();
    if messages.is_empty() {
        return sdk::respond(vec![
            sdk::text("Chats WhatsApp", "heading"),
            sdk::text("Aún no hay mensajes. Los mensajes aparecerán automáticamente.", "info"),
            sdk::button("Refrescar", "refresh_chats", "outline"),
        ]);
    }

    // Group by chat_jid (each unique contact/group)
    let mut chats: HashMap<String, Vec<ChatRecord>> = HashMap::new();
    for msg in &messages {
        chats.entry(msg.chat_jid.clone()).or_default().push(msg.clone());
    }

    // Sort each chat by timestamp
    for msgs in chats.values_mut() {
        msgs.sort_by_key(|m| m.timestamp);
    }

    // Build contact list sorted by most recent message
    let mut contact_list: Vec<(&str, &Vec<ChatRecord>)> = chats.iter()
        .map(|(jid, msgs)| (jid.as_str(), msgs))
        .collect();
    contact_list.sort_by(|a, b| {
        let last_a = a.1.last().map(|m| m.timestamp).unwrap_or(0);
        let last_b = b.1.last().map(|m| m.timestamp).unwrap_or(0);
        last_b.cmp(&last_a) // most recent first
    });

    let mut widgets: Vec<UiWidget> = vec![
        sdk::text("Chats WhatsApp", "heading"),
        sdk::text(&format!("{} conversaciones", contact_list.len()), "info"),
        sdk::divider(),
    ];

    for (jid, msgs) in &contact_list {
        let last = msgs.last().unwrap();
        let name = if !last.push_name.is_empty() { &last.push_name } else {
            jid.split('@').next().unwrap_or(jid)
        };
        let group_tag = if is_group(jid) { "👥 " } else { "💬 " };
        let last_msg = if last.body.len() > 50 {
            format!("{}...", &last.body[..47])
        } else {
            last.body.clone()
        };

        widgets.push(sdk::card(&format!("{} {} — {} mensajes", group_tag, name, msgs.len()), vec![
            sdk::text(&last_msg, "muted"),
            sdk::button("Abrir Chat", &format!("open_chat:{}", jid), "outline"),
        ]));
    }

    widgets.push(sdk::divider());
    widgets.push(sdk::button("Refrescar", "refresh_chats", "primary"));

    sdk::respond(widgets);
}

// ── Página 2b: Conversación individual (chat burbujas) ──────────────────

fn render_conversation(chat_jid: &str) {
    let session_id = get_session_id();
    let messages = fetch_messages();
    let mut conv: Vec<&ChatRecord> = messages.iter()
        .filter(|m| m.chat_jid == chat_jid)
        .collect();
    conv.sort_by_key(|m| m.timestamp);

    let contact_name = conv.first()
        .map(|m| display_name(m))
        .unwrap_or_else(|| chat_jid.split('@').next().unwrap_or(chat_jid).to_string());

    let mut widgets: Vec<UiWidget> = vec![
        sdk::card(&format!("💬 {}", contact_name), vec![
            sdk::button("← Volver", "go_chats", "outline"),
            sdk::button("Refrescar", &format!("open_chat:{}", chat_jid), "primary"),
        ]),
    ];

    // Message bubbles — each message in its own card styled as bubble
    for msg in &conv {
        let is_out = is_from_agent(msg, &session_id);
        let sender = if is_out {
            format!("Tú — {}", format_timestamp(msg.timestamp))
        } else {
            format!("{} — {}", display_name(msg), format_timestamp(msg.timestamp))
        };
        let bubble_style = if is_out { "success" } else { "default" };
        let media_tag = if msg.body.is_empty() { "📎 Multimedia" } else { &msg.body };

        widgets.push(sdk::card(&sender, vec![
            sdk::text(&format!("{} {}", if is_out { "→" } else { "←" }, media_tag), bubble_style),
        ]));
    }

    // Input area to send a message
    widgets.push(sdk::divider());
    widgets.push(sdk::card("Escribir mensaje", vec![
        sdk::input("Mensaje", &format!("msg_input:{}", chat_jid), "Escribe un mensaje..."),
        sdk::button("Enviar", &format!("send_msg:{}", chat_jid), "primary"),
    ]));

    sdk::respond(widgets);
}

// ── Página 3: Configuración ─────────────────────────────────────────────

fn render_config_page() {
    let bridge_url = get_bridge_url();
    let session_id = get_session_id();
    sdk::respond(vec![
        sdk::card("Configuración WhatsApp Bridge", vec![
            sdk::input("URL del Bridge", "bridge_url", &bridge_url),
            sdk::text(&format!("Sesión activa: {}", if session_id.is_empty() { "Ninguna" } else { &session_id }), "info"),
            sdk::button("Guardar", "save_config", "primary"),
            sdk::button("Re-conectar", "reconnect", "warning"),
        ]),
    ]);
}

// ── Action handler ──────────────────────────────────────────────────────

fn handle_action(action: &str, data: &serde_json::Value) {
    let bridge_url = get_bridge_url();
    let session_id = get_session_id();

    match action {
        "conectar" => {
            let new_id = match data.get("session_id_input").and_then(|v| v.as_str()) {
                Some(id) if !id.trim().is_empty() => id.trim().to_string(),
                _ => { sdk::respond_error("Ingresa un ID de sesión"); return; }
            };
            sdk::kv_set_val(KV_SESSION_ID, &new_id);
            let url = format!("{}/sessions/{}/connect", bridge_url, new_id);
            match http_post(&url, "{}") {
                Some(r) if r.status == 200 =>
                    sdk::respond_ok(&format!("Sesión '{}' iniciada. Escanea el QR.", new_id)),
                Some(r) => sdk::respond_error(&format!("Bridge error ({}): {}", r.status, r.body)),
                None => sdk::respond_error(&format!("Bridge no disponible en {}", bridge_url)),
            }
        }

        action if action.starts_with("desconectar:") => {
            let sid = action.trim_start_matches("desconectar:");
            let url = format!("{}/sessions/{}", bridge_url, sid);
            http_delete(&url);
            sdk::kv_set_val(KV_SESSION_ID, "");
            sdk::respond_ok("Sesión desconectada");
        }

        action if action.starts_with("open_chat:") => {
            let jid = action.trim_start_matches("open_chat:");
            render_conversation(jid);
        }

        action if action.starts_with("send_msg:") => {
            let chat_jid = action.trim_start_matches("send_msg:");
            let msg_text = data.get(&format!("msg_input:{}", chat_jid))
                .or_else(|| data.get("msg_input"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if msg_text.is_empty() { 
                sdk::respond_error("Escribe un mensaje"); return; 
            }
            if session_id.is_empty() {
                sdk::respond_error("No hay sesión activa"); return;
            }
            // The bridge doesn't support sending messages via Baileys API
            sdk::respond_error("El envío de mensajes requiere una acción personalizada en tu bridge");
        }

        "go_chats" => render_chats_page(),
        "go_connect" => render_connect_page(),
        "refresh_chats" => render_chats_page(),

        "save_config" => {
            if let Some(url) = data.get("bridge_url").and_then(|v| v.as_str()) {
                sdk::kv_set_val(KV_BRIDGE_URL, url);
                sdk::respond_ok("Configuración guardada");
            }
        }

        "reconnect" => {
            let url = format!("{}/sessions/{}/connect", bridge_url, session_id);
            http_post(&url, "{}");
            sdk::respond_ok("Reconectando...");
        }

        _ => sdk::respond_error(&format!("Acción desconocida: {}", action)),
    }
}

// ── Webhook handler ─────────────────────────────────────────────────────

fn handle_webhook(payload: &str) {
    let now = sdk::now();
    let data: BridgePayload = match serde_json::from_str(payload) {
        Ok(d) => d,
        Err(_) => return,
    };

    if data.event != "message" { return; }

    if let Some(msg) = &data.message {
        // Determine chat_jid: for incoming msgs, from IS the chat
        // For outgoing msgs, from is the session
        let chat_jid = msg.from.clone();
        let from_jid = msg.from.clone();

        let record = serde_json::json!({
            "session_id": data.session_id,
            "chat_jid": chat_jid,
            "from_jid": from_jid,
            "push_name": msg.push_name.as_deref().unwrap_or(""),
            "body": if msg.text.is_empty() { "(media)" } else { &msg.text },
            "timestamp": msg.timestamp,
            "ingested_at": now,
        });
        sdk::create_data_record("whatsapp_messages", &record.to_string());
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn fetch_messages() -> Vec<ChatRecord> {
    let mut all = Vec::new();
    if let Some(raw) = sdk::list_data_records("whatsapp_messages", 500) {
        if let Ok(records) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(arr) = records.as_array() {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        all.push(ChatRecord {
                            session_id: obj.get("session_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            chat_jid: obj.get("chat_jid").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            from_jid: obj.get("from_jid").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            push_name: obj.get("push_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            body: obj.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            timestamp: obj.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0),
                            ingested_at: obj.get("ingested_at").and_then(|v| v.as_u64()).unwrap_or(0),
                        });
                    }
                }
            }
        }
    }
    all
}

fn format_timestamp(ts: u64) -> String {
    let secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    format!("{:02}:{:02}", hours, mins)
}
