# Ezer CLI

Herramienta oficial para crear, compilar, probar y publicar plugins WebAssembly para EzerDesk.

## Requisitos

- **Rust** (1.75+) — [rustup.rs](https://rustup.rs)
- **Target WASM**: `rustup target add wasm32-unknown-unknown`
- **curl** (solo para `ezer publish`)

## Instalación

```bash
# Desde el repositorio
cd ezer-cli
cargo install --path .

# O compilado manual
cargo build --release
./target/release/ezer-cli --help
```

## Comandos

### `ezer init <nombre>`

Crea un nuevo proyecto de plugin con todo lo necesario:

```bash
ezer init mi-plugin
cd mi-plugin
```

Esto genera:

```
mi-plugin/
├── Cargo.toml          # Dependencias del SDK
├── src/
│   └── lib.rs          # Código del plugin con ejemplos
├── .ezer-key           # 🔐 Clave privada (cifrada con contraseña)
└── .ezer-key.pub       # 🔑 Clave pública (para el administrador)
```

Al ejecutarlo por primera vez se te pedirá una **contraseña** para cifrar la clave privada. Esta contraseña se solicitará cada vez que compiles (para firmar automáticamente).

---

### `ezer build`

Compila el plugin a WebAssembly y lo firma automáticamente:

```bash
cd mi-plugin
ezer build
```

Salida:

```
📦 Ezerdesk Compilando plugin para WebAssembly...
✨ ¡Listo! Plugin compilado con éxito.
🔐 Plugin firmado automáticamente.
🔍 Archivo: target/wasm32-unknown-unknown/release/mi_plugin.wasm
```

Genera:
- `.wasm` — binario compilado
- `.sig` — firma Ed25519 del binario

---

### `ezer dev`

Inicia un servidor de desarrollo local para probar el plugin en el navegador:

```bash
ezer dev
```

Abre automáticamente `http://localhost:3030` con una interfaz que permite:

| Botón | Evento | Descripción |
|-------|--------|-------------|
| 📋 GetMetadata | `get_metadata` | Ver metadatos y navegación del plugin |
| 📄 PageRequest | `page_request` | Renderizar página completa del módulo |
| 🧩 GetUiFragments | `get_ui_fragments` | Ver widgets en ubicaciones del host |
| ⚡ PluginAction | `plugin_action` | Simular clic en botón |
| 🎫 TicketCreated | `ticket.created` | Simular creación de ticket |

Incluye:
- Visualización en vivo de los widgets devueltos
- Consola de logs del plugin
- Campo `page_id` personalizable
- Botón de recarga

```bash
# Puerto personalizado
ezer dev --port 8080
```

---

### `ezer publish`

Publica el plugin en un servidor EzerDesk:

```bash
ezer publish
```

El comando:
1. Lee el nombre y versión desde `Cargo.toml`
2. Solicita credenciales (email + contraseña)
3. Si el servidor requiere OTP, pide el código
4. Sube el plugin al servidor
5. Si ya existe un plugin con el mismo nombre, **actualiza la versión**

```bash
# Servidor personalizado
ezer publish --server https://tudominio.com

# Plugin de pago
ezer publish --precio 9.99 --es-pago
```

Variables de entorno:

| Variable | Propósito |
|----------|-----------|
| `EZER_SERVER` | URL del servidor (default: `http://localhost:8000`) |

---

## Estructura de un Plugin

### Archivos del proyecto

| Archivo | Propósito |
|---------|-----------|
| `src/lib.rs` | Código principal del plugin |
| `Cargo.toml` | Configuración y dependencias |
| `.ezer-key` | Clave privada Ed25519 (cifrada) |
| `.ezer-key.pub` | Clave pública Ed25519 |

### Ciclo de vida del código

```rust
use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        // 1. SIDEBAR — Menú de navegación lateral
        PluginEvent::GetMetadata => {
            let meta = sdk::metadata()
                .nav_item(NavItem::new("dashboard", "Mi Plugin", "rocket-line")
                    .category("operaciones")
                    .priority(10))
                .name("Mi Plugin")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        // 2. PÁGINA DEL MÓDULO — Al hacer clic en el menú
        PluginEvent::PageRequest { page_id } => {
            sdk::respond(sdk::widgets![
                sdk::card("Panel Principal", [
                    sdk::text("Bienvenido a tu plugin.", "info")
                ])
            ]);
        }

        // 3. CONFIGURACIÓN — Gestión → Plugins → ⚙️
        PluginEvent::GetUiFragments { location } => {
            match location.as_str() {
                "plugin_settings" => {
                    sdk::respond(sdk::widgets![
                        sdk::card("Configuración", [
                            sdk::text("Ajusta los parámetros.", "muted"),
                            sdk::input("API Key", "api_key", "Ingresa tu llave..."),
                        ])
                    ]);
                },
                _ => {}
            }
        }

        // 4. ACCIONES — Botones y formularios
        PluginEvent::PluginAction { action, data } => {
            sdk::respond_ok("Procesado");
        }

        // 5. EVENTOS DEL SISTEMA
        PluginEvent::TicketCreated(ticket) => {
            sdk::log(&format!("Ticket creado: {}", ticket.asunto));
        }

        _ => {}
    }
    0
}
```

### Secciones del código

| # | Evento | Dónde se renderiza |
|---|--------|-------------------|
| 1 | `GetMetadata` | Define el ítem en el menú lateral |
| 2 | `PageRequest` | Página completa al hacer clic en el menú |
| 3 | `GetUiFragments` | Widgets en Dashboard, configuración, tickets |
| 4 | `PluginAction` | Responde a botones y formularios |
| 5 | Eventos (`TicketCreated`, etc.) | Se ejecutan automáticamente en segundo plano |

## Widgets UI Disponibles

```rust
// Contenedor con título y contenido
sdk::card("Título", [widget1, widget2])

// Texto con estilo (info, muted, success, warning, danger)
sdk::text("Contenido", "info")

// Botón que dispara PluginAction
sdk::button("Click aquí", "mi_accion", "primary")

// Campos de formulario
sdk::input("Email", "email", "tu@email.com")
sdk::textarea("Descripción", "desc", "Escribe aquí...")
sdk::select_widget("País", "pais", [("ec", "Ecuador")], "ec")
sdk::switch_widget("Notificar", "notificar", true)

// Etiquetas y decoración
sdk::badge("Nuevo", "success")
sdk::icon("star", "yellow")
sdk::divider()
sdk::modal("Confirmar", [texto], "sm", "cerrar")
```

## Consultas al Sistema

```rust
// Obtener tickets (con filtros)
match query::tickets().limit(10).all() {
    Ok(list) => sdk::log(&format!("{} tickets", list.len())),
    Err(e) => sdk::log(&format!("Error: {:?}", e)),
}

// Tickets por estado
match query::tickets().by_status("abierto").all() {
    Ok(list) => { /* ... */ },
    Err(_) => {}
}

// Agentes y departamentos
match query::agents().all() {
    Ok(agents) => sdk::log(&format!("{} agentes", agents.len())),
    Err(_) => {}
}

match query::departments().all() {
    Ok(depts) => { /* ... */ },
    Err(_) => {}
}
```

## Persistencia (KV Store)

```rust
// Guardar valor
sdk::kv_set_val("telegram_token", "123:ABC");

// Leer valor
let token = sdk::kv_get_val("telegram_token").unwrap_or_default();
```

## Peticiones HTTP

```rust
let req = HttpRequest {
    method: "POST".to_string(),
    url: "https://api.telegram.org/bot{token}/sendMessage".to_string(),
    body: "{\"chat_id\":123,\"text\":\"Hola\"}".to_string(),
    headers: vec![("Content-Type".into(), "application/json".into())],
};

match sdk::http_request(&req) {
    Some(resp) => sdk::log(&format!("Status: {}", resp.status)),
    None => sdk::log("Error de red"),
}
```

**Nota:** El dominio debe estar en `PLUGIN_ALLOWED_DOMAINS` del servidor.

## Seguridad

### Claves y Firmas

- `ezer init` genera un par de claves Ed25519 automáticamente
- La clave privada se **cifra con contraseña** (AES-256-GCM + PBKDF2)
- `ezer build` firma el `.wasm` automáticamente
- El servidor puede verificar la firma si `PLUGIN_PUBLIC_KEY` está configurado

### Configuración del Servidor

```bash
# .env del backend
PLUGIN_PUBLIC_KEY=<clave_pública_del_desarrollador>
PLUGIN_ALLOWED_DOMAINS=api.telegram.org,api.github.com
```

Sin `PLUGIN_PUBLIC_KEY`, el servidor opera en modo desarrollo (no exige firma).

## Solución de Problemas

| Problema | Solución |
|----------|----------|
| `rustup target add wasm32-unknown-unknown` | Instalar target WASM |
| `error: wasm32-unknown-unknown target not installed` | Ejecutar: `rustup target add wasm32-unknown-unknown` |
| `ezer build` falla | Verificar que estás dentro de la carpeta del plugin (debe tener `Cargo.toml`) |
| Contraseña incorrecta al compilar | La contraseña es la que definiste al hacer `ezer init` |
| `curl: command not found` | Instalar curl o usar `EZER_TOKEN` manualmente |
| Plugin no aparece en el servidor | Verificar `PLUGIN_PUBLIC_KEY` en el `.env` del backend |
