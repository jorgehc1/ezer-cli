// Ejemplo 34: Meta WhatsApp Business API (Directo)
// Features: Meta Cloud API, Envío de WhatsApp, Webhook entrante, KV store
// Demuestra: Plugin que usa la API directa de Meta (Cloud API)
//            Sin intermediarios — conexión directa a graph.facebook.com
//            0% riesgo de ban — canal oficial certificado por Meta

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

const KV_COUNTER: &str = "meta_msg_counter";
const KV_WEBHOOK_MSG: &str = "meta_last_webhook";

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(NavItem::new("meta_send", "Meta WhatsApp", "whatsapp-line")
                    .category("operaciones").priority(3))
                .nav_item(NavItem::new("meta_config", "Config. Meta", "settings-line")
                    .category("operaciones").priority(9))
                .name("WhatsApp Meta Cloud API")
                .description("API directa de Meta para WhatsApp Business. Clientes certificados.")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "meta_send" => render_send_page(),
                "meta_config" => render_config_page(),
                _ => sdk::respond(vec![sdk::text("Página no encontrada", "error")]),
            }
        }

        PluginEvent::PluginAction { action, data } => {
            handle_action(&action, &data);
        }

        // Incoming messages via local_bus → plugin_subscriber
        PluginEvent::SmsInbound { from, body, .. } => {
            if from == "meta_whatsapp" {
                sdk::log(&format!("Meta webhook recibido: {}", body));
                sdk::kv_set_val(KV_WEBHOOK_MSG, &body);
            }
        }

        _ => {}
    }
    0
}

fn render_send_page() {
    let counter = sdk::kv_get_val(KV_COUNTER).unwrap_or("0".to_string());
    let last_webhook = sdk::kv_get_val(KV_WEBHOOK_MSG).unwrap_or_default();
    let webhook_preview = if last_webhook.len() > 100 {
        format!("{}...", &last_webhook[..97])
    } else {
        last_webhook
    };

    sdk::respond(sdk::widgets![
        sdk::card("Meta Cloud API — WhatsApp Business", vec![
            sdk::text("Conexión directa a graph.facebook.com. Clientes certificados por Meta.", "success"),
            sdk::divider(),
            sdk::input("Número (código país + número)", "meta_to", "+593999999999"),
            sdk::textarea("Mensaje", "meta_body", "Escribe tu mensaje..."),
            sdk::button("Enviar por Meta API", "send_meta", "primary"),
        ]),
        sdk::card("Estadísticas", vec![
            sdk::text(&format!("📤 Mensajes enviados: {}", counter), "info"),
        ]),
        if webhook_preview.is_empty() {
            sdk::text("No hay webhooks entrantes.", "muted")
        } else {
            sdk::card("Último webhook Meta", vec![
                sdk::text(&webhook_preview, "code"),
            ])
        },
    ]);
}

fn render_config_page() {
    sdk::respond(sdk::widgets![
        sdk::card("Configuración Meta Cloud API", vec![
            sdk::text("Requisitos:", "info"),
            sdk::text("1. WhatsApp Business Account aprobada por Meta", "muted"),
            sdk::text("2. Token de acceso permanente de Meta Business", "muted"),
            sdk::text("3. Número de teléfono registrado en Meta", "muted"),
            sdk::text("4. Configurar en el backend:", "muted"),
            sdk::text("   META_WHATSAPP_TOKEN=<token>", "code"),
            sdk::text("   META_PHONE_NUMBER_ID=<id>", "code"),
            sdk::text("   META_WEBHOOK_VERIFY_TOKEN=<token>", "code"),
            sdk::divider(),
            sdk::text("Webhook entrante (configurar en Meta Business):", "muted"),
            sdk::text("   POST {APP_URL}/api/v1/webhooks/meta/whatsapp", "code"),
            sdk::text("   Verify Token: META_WEBHOOK_VERIFY_TOKEN", "code"),
        ]),
    ]);
}

fn handle_action(action: &str, data: &serde_json::Value) {
    match action {
        "send_meta" => {
            let to = data.get("meta_to")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let body = data.get("meta_body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if to.is_empty() || body.is_empty() {
                sdk::respond_error("Número y mensaje requeridos");
                return;
            }

            // Send via Meta Cloud API (uses backend's integration adapter)
            sdk::send_sms(&to, &body);

            let counter = sdk::kv_get_val(KV_COUNTER)
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0) + 1;
            sdk::kv_set_val(KV_COUNTER, &counter.to_string());

            sdk::respond_ok(&format!("Mensaje enviado a {} via Meta Cloud API", to));
        }

        _ => sdk::respond_error(&format!("Acción desconocida: {}", action)),
    }
}
