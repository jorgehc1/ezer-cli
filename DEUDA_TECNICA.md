# Reporte de Deuda Técnica: `ezer-cli`

> **Nota del Analista:** Como en las anteriores evaluaciones, te recuerdo que `ezer-cli` es un proyecto desarrollado completamente en **Rust** (con utilidades como `clap` y llamadas criptográficas), y no en Gleam. A continuación, presento un análisis profundo de la deuda técnica de esta herramienta de línea de comandos (CLI) desde mi perspectiva como ingeniero experto, listo para que lo procese otra IA o el equipo.

## 1. Dependencia Externa del Sistema para Peticiones HTTP (Severidad: Alta)
- **Problema:** En la función `curl_json` (línea 443), el CLI hace un "shell out" para ejecutar el binario `curl` a través de `std::process::Command`, pasándole argumentos para subir archivos y manejar cookies.
- **Impacto:** Esto destruye la portabilidad de la herramienta. Si el desarrollador que usa el CLI está en Windows o en un entorno (Docker/CI) donde `curl` no está instalado en el PATH o tiene una versión antigua, la publicación fallará estrepitosamente. Además, hacer parsing del `stdout/stderr` de curl es muy frágil.
- **Acción a tomar:** 
  - Reemplazar completamente las llamadas a `Command::new("curl")` por un cliente HTTP nativo en Rust, como la librería `reqwest` (preferiblemente en modo blocking para mantener la sincronía del CLI o usando `tokio` si se requieren operaciones asíncronas).

## 2. Inyección de Base64 Masiva en el Dev Server (Severidad: Media)
- **Problema:** El comando `ezer dev` lee el binario WebAssembly compilado, lo convierte a Base64 usando `base64::engine` y lo inyecta completo haciendo un `replace` de la cadena `__WASM_BASE64__` directamente en el contenido de `dev.html`.
- **Impacto:** Los binarios WASM (incluso optimizados) pueden pesar cientos de kilobytes o megabytes. Inyectar todo ese payload en el string HTML no es eficiente, hace que el navegador tenga que parsear un string inmenso, y consume mucha memoria en el CLI (ya que crea buffers enormes de String).
- **Acción a tomar:**
  - El servidor local (el `TcpListener`) debería enrutar dos endpoints distintos: uno `/` que devuelva el HTML estándar, y otro endpoint (ej. `/plugin.wasm`) que sirva directamente el archivo `.wasm` en formato binario puro (con el header `application/wasm`).

## 3. Servidor HTTP Manual (Severidad: Media/Baja)
- **Problema:** El servidor local está escrito usando sockets crudos (`std::net::TcpListener`) y responde a las peticiones escribiendo strings hardcodeados (`"HTTP/1.1 200 OK\r\n..."`).
- **Impacto:** Un cliente que no cumpla exactamente con lo esperado o que pida archivos adyacentes podría romper el hilo del CLI o provocar comportamientos no definidos (como un pánico silencioso).
- **Acción a tomar:**
  - Integrar un servidor web súper ligero como `axum`, `warp` o incluso `tiny_http` para servir el entorno de desarrollo local con un manejo correcto del protocolo HTTP.

## 4. Versión del SDK Hardcodeada en la Plantilla (Severidad: Baja)
- **Problema:** El comando `init_plugin_at` inyecta una cadena de texto para el `Cargo.toml` con la versión del SDK fijada rígidamente a `ezerdesk-sdk = "0.1.3"`.
- **Impacto:** A medida que el SDK evolucione, los plugins inicializados nacerán con una versión desactualizada y los desarrolladores tendrán que arreglar el archivo a mano.
- **Acción a tomar:**
  - Leer dinámicamente la última versión del crate o proveer la versión al momento de la compilación/actualización del CLI.

## 5. Prácticas Criptográficas y Manejo de Secretos en Memoria (Severidad: Baja/Mejora Continua)
- **Problema:** Se usan `aes_gcm`, `ed25519_dalek` y `pbkdf2` para cifrar la llave de firma del usuario. Aunque el uso de PBKDF2 (100k iteraciones) + AES-GCM es bastante seguro, las contraseñas planas y llaves desencriptadas viven en memoria temporal (`String` y `[u8; 32]`) durante la operación sin forzar la limpieza de memoria al finalizar (Zeroize).
- **Acción a tomar:**
  - (Opcional) Evaluar el uso de Argon2id en lugar de PBKDF2 para derivación de contraseñas de última generación.
  - Usar los crates `secrecy` o `zeroize` para limpiar los arreglos que contienen las claves simétricas en texto plano apenas dejan de ser utilizados, mitigando posibles volcados de memoria (memory dumps).

## Resumen de Tareas Prioritarias:
1. Desacoplar el proyecto de `curl` agregando la dependencia `reqwest` para manejar todo el flujo de autenticación, MFA y subida de archivos de manera programática en el comando `publish`.
2. Refactorizar el comando `dev` para no inyectar WASM en el HTML por Base64, sino montando un pequeño servidor HTTP que sirva ambos archivos por separado.
