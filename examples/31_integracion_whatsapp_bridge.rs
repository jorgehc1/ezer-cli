// Ejemplo 24: Integración WhatsApp Bridge
// Features: Bridge webhook, Data Models, HTTP requests, QR codes, Table
// Demuestra: Plugin que recibe eventos del whatsapp-web-bridge (Node.js + Baileys)
//            Gestiona sesiones, muestra QR para vincular, y almacena mensajes entrantes

use ezerdesk_sdk as sdk;
use sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Bridge payload types ──────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug)]
struct BridgePayload {
    #[serde(rename = "sessionId")]
    session_id: String,
    event: String,
    status: Option<String>,
    #[serde(default)]
    error: Option<String>,
    message: Option<BridgeMessage>,
}

#[derive(Deserialize, Serialize, Debug)]
struct BridgeMessage {
    id: String,
    from: String,
    #[serde(rename = "pushName")]
    push_name: String,
    timestamp: u64,
    text: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct BridgeSessionState {
    status: String,
    qr: Option<String>,
    qr_ascii: Option<String>,
    error: Option<String>,
}

// ── Configuration keys ───────────────────────────────────────────────────

const KV_BRIDGE_URL: &str = "bridge_url";
const DEFAULT_BRIDGE_URL: &str = "http://localhost:3000";

fn get_bridge_url() -> String {
    sdk::kv_get_val(KV_BRIDGE_URL).unwrap_or_else(|| DEFAULT_BRIDGE_URL.to_string())
}

// ── HTTP helpers (call the bridge REST API) ──────────────────────────────

fn http_get(url: &str) -> Option<HttpResponse> {
    sdk::http_request(&HttpRequest {
        method: "GET".to_string(),
        url: url.to_string(),
        body: "".to_string(),
        headers: vec![],
    })
}

fn http_post(url: &str, body: &str) -> Option<HttpResponse> {
    sdk::http_request(&HttpRequest {
        method: "POST".to_string(),
        url: url.to_string(),
        body: body.to_string(),
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
    })
}

fn http_delete(url: &str) -> Option<HttpResponse> {
    sdk::http_request(&HttpRequest {
        method: "DELETE".to_string(),
        url: url.to_string(),
        body: "".to_string(),
        headers: vec![],
    })
}

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            // Initialize custom data models for persistence
            sdk::create_data_model(
                "whatsapp_sessions",
                "Sesiones activas y códigos QR",
                r#"{"fields": [
                    {"name": "session_id", "type": "string", "required": true},
                    {"name": "status", "type": "string", "required": true},
                    {"name": "qr", "type": "string"},
                    {"name": "error", "type": "string"},
                    {"name": "created_at", "type": "number"},
                    {"name": "updated_at", "type": "number"}
                ]}"#
            );

            sdk::create_data_model(
                "whatsapp_messages",
                "Mensajes de WhatsApp monitoreados",
                r#"{"fields": [
                    {"name": "session_id", "type": "string", "required": true},
                    {"name": "msg_id", "type": "string", "required": true},
                    {"name": "chat_jid", "type": "string", "required": true},
                    {"name": "from_jid", "type": "string", "required": true},
                    {"name": "push_name", "type": "string"},
                    {"name": "body", "type": "string", "required": true},
                    {"name": "timestamp", "type": "number", "required": true},
                    {"name": "ingested_at", "type": "number", "required": true}
                ]}"#
            );

            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("whatsapp_config", "Conexión WhatsApp", "qr-code-line")
                        .category("operaciones")
                        .priority(5)
                )
                .nav_item(
                    NavItem::new("whatsapp_settings", "Config. Bridge", "settings-line")
                        .category("operaciones")
                        .priority(7)
                )
                .nav_item(
                    NavItem::new("whatsapp_history", "Historial WhatsApp", "chat-history-line")
                        .category("operaciones")
                        .priority(6)
                )
                .name("Monitoreo WhatsApp")
                .description("Monitorea y audita chats de WhatsApp sin interferir con la app móvil")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "whatsapp_config" => render_config_page(),
                "whatsapp_settings" => render_settings_page(),
                "whatsapp_history" => render_history_page(),
                _ => sdk::respond(vec![sdk::text("Página no encontrada", "error")]),
            }
        }

        PluginEvent::PluginAction { action, data } => {
            handle_action(&action, &data);
        }

        // Bridge webhook events from whatsapp-web-bridge
        PluginEvent::BridgeWebhook { payload } => {
            handle_bridge_webhook(&payload);
        }

        _ => {}
    }
    0
}

// ── Page renderers ───────────────────────────────────────────────────────

fn render_config_page() {
    let bridge_url = get_bridge_url();
    let mut widgets: Vec<UiWidget> = vec![
        sdk::card("Nueva Conexión WhatsApp", vec![
            sdk::input("ID de Sesión (Ej: +593999999999)", "new_session_id",
                       "Número o ID único para esta sesión"),
            sdk::button("Generar QR", "vincular_numero", "primary"),
        ]),
    ];

    // Fetch active sessions from the bridge
    let sessions_url = format!("{}/sessions", bridge_url);
    if let Some(res) = http_get(&sessions_url) {
        if res.status == 200 {
            if let Ok(sessions) = serde_json::from_str::<HashMap<String, BridgeSessionState>>(&res.body) {
                if sessions.is_empty() {
                    widgets.push(sdk::text("No hay sesiones activas. Crea una nueva arriba.", "info"));
                } else {
                    widgets.push(sdk::divider());
                    for (id, state) in &sessions {
                        let status_icon = match state.status.as_str() {
                            "connected" => "✅ Conectado",
                            "qr" => "⏳ Esperando QR",
                            "connecting" => "🔄 Conectando...",
                            _ => "❌ Desconectado",
                        };

                        let mut session_widgets: Vec<UiWidget> = vec![
                            sdk::text(&format!("{} — {}", status_icon, id), "info"),
                        ];

                        if state.status == "qr" {
                            if let Some(ascii) = &state.qr_ascii {
                                session_widgets.push(sdk::text("Escanea este código QR:", "info"));
                                session_widgets.push(sdk::text(&ascii, "code"));
                            }
                            session_widgets.push(sdk::text(
                                "Abre WhatsApp → Menú → Dispositivos vinculados → Vincular", "muted"));
                        }

                        if let Some(err) = &state.error {
                            session_widgets.push(sdk::text(&format!("Error: {}", err), "error"));
                        }

                        session_widgets.push(sdk::button(
                            &format!("Desconectar"), &format!("desconectar:{}", id), "danger"));

                        widgets.push(sdk::card(&format!("📱 {}", id), session_widgets));
                    }
                }
            }
        } else {
            widgets.push(sdk::text(
                &format!("Bridge HTTP {}. Verifica configuración.", res.status), "error"));
        }
    } else {
        widgets.push(sdk::text(
            &format!("Bridge no disponible en {}. ¿El servicio está corriendo?", bridge_url), "error"));
    }

    sdk::respond(widgets);
}

fn render_settings_page() {
    let current_url = get_bridge_url();
    let widgets = vec![
        sdk::card("Configuración del Bridge", vec![
            sdk::text("URL del servicio whatsapp-web-bridge (Node.js)", "info"),
            sdk::input("URL del Bridge", "bridge_url", &current_url),
            sdk::button("Guardar", "guardar_config", "primary"),
        ]),
    ];
    sdk::respond(widgets);
}

fn render_history_page() {
    let mut widgets: Vec<UiWidget> = vec![
        sdk::text("Historial de mensajes entrantes de WhatsApp", "info"),
    ];

    if let Some(records_str) = sdk::list_data_records("whatsapp_messages", 200) {
        if let Ok(records) = serde_json::from_str::<serde_json::Value>(&records_str) {
            let mut rows: Vec<Vec<&str>> = Vec::new();
            if let Some(arr) = records.as_array() {
                for item in arr {
                    let session = item.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
                    let from = item.get("from_jid").and_then(|v| v.as_str()).unwrap_or("");
                    let name = item.get("push_name").and_then(|v| v.as_str()).unwrap_or("");
                    let body = item.get("body").and_then(|v| v.as_str()).unwrap_or("");
                    let ts = item.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");

                    let short = if body.len() > 60 { &body[..57] } else { body };
                    rows.push(vec![session, from, name, short, ts]);
                }
            }

            if rows.is_empty() {
                widgets.push(sdk::text("Aún no hay mensajes.", "info"));
            } else {
                widgets.push(sdk::table(
                    vec!["Sesión", "Contacto", "Nombre", "Mensaje", "Timestamp"],
                    rows,
                ));
            }
        }
    }

    sdk::respond(widgets);
}

// ── Action handler ───────────────────────────────────────────────────────

fn handle_action(action: &str, data: &serde_json::Value) {
    let bridge_url = get_bridge_url();

    match action {
        "vincular_numero" => {
            let session_id = match data.get("new_session_id").and_then(|v| v.as_str()) {
                Some(id) if !id.trim().is_empty() => id.trim(),
                _ => { sdk::respond_error("ID de sesión requerido"); return; }
            };

            let url = format!("{}/sessions/{}/connect", bridge_url, session_id);
            match http_post(&url, "{}") {
                Some(r) if r.status == 200 =>
                    sdk::respond_ok(&format!("Sesión '{}' creada. Escanea el QR.", session_id)),
                Some(r) => sdk::respond_error(&format!("Bridge error ({}): {}", r.status, r.body)),
                None => sdk::respond_error(&format!("Bridge no disponible en {}", bridge_url)),
            }
        }

        action if action.starts_with("desconectar:") => {
            let session_id = action.trim_start_matches("desconectar:");
            let url = format!("{}/sessions/{}", bridge_url, session_id);
            match http_delete(&url) {
                Some(r) if r.status == 200 =>
                    sdk::respond_ok(&format!("Sesión '{}' eliminada.", session_id)),
                _ => sdk::respond_error("Error al desconectar"),
            }
        }

        "guardar_config" => {
            if let Some(url) = data.get("bridge_url").and_then(|v| v.as_str()) {
                sdk::kv_set_val(KV_BRIDGE_URL, url);
                sdk::respond_ok(&format!("Bridge URL actualizada a {}", url));
            } else {
                sdk::respond_error("URL inválida");
            }
        }

        _ => sdk::respond_error(&format!("Acción desconocida: {}", action)),
    }
}

// ── Bridge webhook handler ───────────────────────────────────────────────

fn handle_bridge_webhook(payload: &str) {
    let now = sdk::now();

    let bridge_data: BridgePayload = match serde_json::from_str(payload) {
        Ok(d) => d,
        Err(_) => { sdk::log(&format!("Invalid bridge payload: {}", payload)); return; }
    };

    match bridge_data.event.as_str() {
        "status" => {
            let status = bridge_data.status.as_deref().unwrap_or("unknown");
            let session_json = serde_json::json!({
                "session_id": bridge_data.session_id,
                "status": status,
                "qr": "",
                "error": bridge_data.error.unwrap_or_default(),
                "created_at": now,
                "updated_at": now,
            });
            sdk::create_data_record("whatsapp_sessions", &session_json.to_string());
            sdk::log(&format!("[WA] Session {}: {}", bridge_data.session_id, status));
        }

        "message" => {
            if let Some(msg) = &bridge_data.message {
                let text = if msg.text.is_empty() { "(media)" } else { &msg.text };
                let msg_json = serde_json::json!({
                    "session_id": bridge_data.session_id,
                    "msg_id": msg.id,
                    "chat_jid": msg.from,
                    "from_jid": msg.from,
                    "push_name": msg.push_name,
                    "body": text,
                    "timestamp": msg.timestamp,
                    "ingested_at": now,
                });
                sdk::create_data_record("whatsapp_messages", &msg_json.to_string());
                sdk::log(&format!("[WA] Msg from {} in {}: {}", msg.from,
                    bridge_data.session_id, text));
            }
        }

        _ => sdk::log(&format!("[WA] Unknown event: {}", bridge_data.event)),
    }
}
