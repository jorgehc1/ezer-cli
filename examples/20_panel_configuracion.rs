// Ejemplo 20: Panel de Configuracion
// Features: KV store, Settings, Multi-seccion, Validacion
// Demuestra: Panel de configuracion completo del sistema y plugins

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("config", "Configuracion", "settings-3-line")
                        .category("sistema")
                        .priority(1)
                )
                .name("Panel de Configuracion")
                .description("Configura los ajustes generales del sistema y plugins")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "config" => render_config_dashboard(),
                "general" => render_general_settings(),
                "appearance" => render_appearance_settings(),
                "security" => render_security_settings(),
                "plugins" => render_plugin_settings(),
                "backup" => render_backup_settings(),
                "about" => render_about(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "save_general" => save_general_settings(&data),
                "save_appearance" => save_appearance_settings(&data),
                "save_security" => save_security_settings(&data),
                "toggle_plugin" => toggle_plugin(&data),
                "create_backup" => create_backup(),
                "restore_backup" => restore_backup(&data),
                "reset_settings" => reset_all_settings(),
                "export_config" => export_configuration(),
                _ => {
                    sdk::respond_ok("Accion no reconocida");
                }
            }
        }

        _ => {}
    }
    0
}

fn render_config_dashboard() {
    sdk::respond(sdk::widgets![
        sdk::card("Panel de Configuracion", vec![
            sdk::text("Administra la configuracion general del sistema", "info"),
            sdk::divider(),

            sdk::card("Secciones", vec![
                sdk::table(
                    vec!["Seccion", "Descripcion", "Estado"],
                    vec![
                        vec!["General", "Nombre del sistema, idioma, zona horaria", "Configurado"],
                        vec!["Apariencia", "Tema, colores, logo", "Configurado"],
                        vec!["Seguridad", "Autenticacion, sesiones, encriptacion", "Configurado"],
                        vec!["Plugins", "Gestionar plugins instalados", "3 activos"],
                        vec!["Backup", "Copias de seguridad y restauracion", "Ultimo: hace 24h"],
                    ],
                ),
            ]),

            sdk::card("Acciones Rapidas", vec![
                sdk::button("Configuracion General", "general", "primary"),
                sdk::button("Apariencia", "appearance", "secondary"),
                sdk::button("Seguridad", "security", "secondary"),
                sdk::button("Plugins", "plugins", "secondary"),
                sdk::button("Backup", "backup", "secondary"),
                sdk::button("Acerca de", "about", "outline"),
            ]),

            sdk::card("Estado del Sistema", vec![
                sdk::text("Version: EzerDesk v2.5.0", "info"),
                sdk::text("Uptime: 15 dias, 3 horas", "info"),
                sdk::text("Licencia: Enterprise", "success"),
                sdk::text("Soporte: Activo hasta 2025-01-15", "info"),
            ]),
        ]),
    ]);
}

fn render_general_settings() {
    let company = sdk::kv_get_val("company_name")
        .unwrap_or("Mi Empresa".to_string());
    let language = sdk::kv_get_val("language")
        .unwrap_or("es".to_string());
    let timezone = sdk::kv_get_val("timezone")
        .unwrap_or("America/Mexico_City".to_string());
    let currency = sdk::kv_get_val("currency")
        .unwrap_or("MXN".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion General", vec![
            sdk::text("Ajustes generales del sistema", "info"),
            sdk::divider(),

            sdk::card("Informacion de la Empresa", vec![
                sdk::input("company_name", "Nombre de la Empresa", &company)
                    .required(true),
                sdk::input("company_email", "Email de Contacto")
                    .placeholder("admin@empresa.com"),
                sdk::input("company_phone", "Telefono")
                    .placeholder("+52 55 1234 5678"),
                sdk::input("company_website", "Sitio Web")
                    .placeholder("https://www.empresa.com"),
            ]),

            sdk::card("Idioma y Region", vec![
                sdk::select("language", "Idioma", vec![
                    ("es", "Espanol"),
                    ("en", "Ingles"),
                    ("pt", "Portugues"),
                    ("fr", "Frances"),
                ]),
                sdk::select("timezone", "Zona Horaria", vec![
                    ("America/Mexico_City", "Ciudad de Mexico (UTC-6)"),
                    ("America/Bogota", "Bogota (UTC-5)"),
                    ("America/Buenos_Aires", "Buenos Aires (UTC-3)"),
                    ("Europe/Madrid", "Madrid (UTC+1)"),
                ]),
                sdk::select("currency", "Moneda", vec![
                    ("MXN", "Peso Mexicano (MXN)"),
                    ("USD", "Dolar Americano (USD)"),
                    ("EUR", "Euro (EUR)"),
                    ("COP", "Peso Colombiano (COP)"),
                ]),
                sdk::select("date_format", "Formato de Fecha", vec![
                    ("dd/mm/yyyy", "DD/MM/YYYY"),
                    ("mm/dd/yyyy", "MM/DD/YYYY"),
                    ("yyyy-mm-dd", "YYYY-MM-DD"),
                ]),
            ]),

            sdk::card("Notificaciones del Sistema", vec![
                sdk::checkbox("email_notifications", "Notificaciones por email"),
                sdk::checkbox("desktop_notifications", "Notificaciones de escritorio"),
                sdk::checkbox("sound_notifications", "Sonido de notificaciones"),
                sdk::select("notification_frequency", "Frecuencia", vec![
                    ("realtime", "Tiempo real"),
                    ("hourly", "Cada hora"),
                    ("daily", "Diario"),
                ]),
            ]),

            sdk::divider(),

            sdk::button("Guardar Cambios", "save_general", "primary"),
            sdk::button("Restablecer Valores por Defecto", "reset_settings", "warning"),
        ]),
    ]);
}

fn render_appearance_settings() {
    let theme = sdk::kv_get_val("theme")
        .unwrap_or("light".to_string());
    let primary_color = sdk::kv_get_val("primary_color")
        .unwrap_or("#2563eb".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion de Apariencia", vec![
            sdk::text("Personaliza la apariencia del sistema", "info"),
            sdk::divider(),

            sdk::card("Tema", vec![
                sdk::select("theme", "Tema", vec![
                    ("light", "Claro"),
                    ("dark", "Oscuro"),
                    ("auto", "Automatico (sistema)"),
                ]),
            ]),

            sdk::card("Colores", vec![
                sdk::input("primary_color", "Color Primario", &primary_color)
                    .input_type("color"),
                sdk::input("secondary_color", "Color Secundario")
                    .placeholder("#64748b")
                    .input_type("color"),
                sdk::input("accent_color", "Color de Acento")
                    .placeholder("#10b981")
                    .input_type("color"),
            ]),

            sdk::card("Logo y Branding", vec![
                sdk::input("logo_url", "URL del Logo")
                    .placeholder("https://empresa.com/logo.png"),
                sdk::input("favicon_url", "URL del Favicon")
                    .placeholder("https://empresa.com/favicon.ico"),
                sdk::text("Tamano maximo del logo: 200x50px, Formato: PNG o SVG", "info"),
            ]),

            sdk::card("Tipografia", vec![
                sdk::select("font_family", "Fuente", vec![
                    ("inter", "Inter"),
                    ("roboto", "Roboto"),
                    ("open-sans", "Open Sans"),
                    ("lato", "Lato"),
                ]),
                sdk::select("font_size", "Tamano de Fuente", vec![
                    ("small", "Pequena"),
                    ("medium", "Mediana"),
                    ("large", "Grande"),
                ]),
            ]),

            sdk::card("Disposicion", vec![
                sdk::select("sidebar_position", "Posicion del Sidebar", vec![
                    ("left", "Izquierda"),
                    ("right", "Derecha"),
                ]),
                sdk::checkbox("compact_mode", "Modo compacto"),
                sdk::checkbox("show_breadcrumbs", "Mostrar breadcrumbs"),
            ]),

            sdk::divider(),

            sdk::button("Guardar Cambios", "save_appearance", "primary"),
            sdk::button("Vista Previa", "config", "secondary"),
        ]),
    ]);
}

fn render_security_settings() {
    let session_timeout = sdk::kv_get_val("session_timeout")
        .unwrap_or("30".to_string());
    let max_login_attempts = sdk::kv_get_val("max_login_attempts")
        .unwrap_or("5".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion de Seguridad", vec![
            sdk::text("Ajustes de seguridad y autenticacion", "info"),
            sdk::divider(),

            sdk::card("Autenticacion", vec![
                sdk::checkbox("require_2fa", "Requerir autenticacion de dos factores"),
                sdk::checkbox("sso_enabled", "Habilitar SSO (Single Sign-On)"),
                sdk::select("auth_method", "Metodo de Autenticacion", vec![
                    ("local", "Local"),
                    ("ldap", "LDAP"),
                    ("saml", "SAML"),
                    ("oauth", "OAuth"),
                ]),
            ]),

            sdk::card("Sesiones", vec![
                sdk::input("session_timeout", "Timeout de Sesion (minutos)", &session_timeout),
                sdk::input("max_login_attempts", "Maximos Intentos de Login", &max_login_attempts),
                sdk::checkbox("lock_on_fail", "Bloquear cuenta despues de intentos fallidos"),
                sdk::select("lock_duration", "Duracion del Bloqueo", vec![
                    ("15", "15 minutos"),
                    ("30", "30 minutos"),
                    ("60", "1 hora"),
                    ("1440", "24 horas"),
                ]),
            ]),

            sdk::card("Contrasenas", vec![
                sdk::number_input("min_password_length", "Longitud Minima")
                    .min(8)
                    .max(32),
                sdk::checkbox("require_uppercase", "Requerir mayusculas"),
                sdk::checkbox("require_numbers", "Requerir numeros"),
                sdk::checkbox("require_symbols", "Requerir simbolos"),
                sdk::input("password_expiry", "Expiracion de Contrasena (dias)")
                    .placeholder("90"),
            ]),

            sdk::card("Encriptacion", vec![
                sdk::checkbox("encrypt_data", "Encriptar datos en reposo"),
                sdk::checkbox("ssl_only", "Forzar conexiones SSL/TLS"),
                sdk::select("encryption_level", "Nivel de Encriptacion", vec![
                    ("aes128", "AES-128"),
                    ("aes256", "AES-256"),
                ]),
            ]),

            sdk::card("Auditoria", vec![
                sdk::checkbox("audit_log", "Habilitar registro de auditoria"),
                sdk::checkbox("track_logins", "Rastrear inicios de sesion"),
                sdk::checkbox("track_changes", "Rastrear cambios de configuracion"),
                sdk::input("log_retention", "Retencion de Logs (dias)")
                    .placeholder("90"),
            ]),

            sdk::divider(),

            sdk::button("Guardar Cambios", "save_security", "primary"),
            sdk::button("Restablecer Seguridad", "reset_settings", "warning"),
        ]),
    ]);
}

fn render_plugin_settings() {
    let plugins = vec![
        ("dashboard_metricas", "Dashboard de Metricas", true, "OneTime", "--"),
        ("monitor_sla", "Monitor de SLA", true, "OneTime", "--"),
        ("gestor_tickets", "Gestor de Tickets", true, "Recurring", "2025-03-15"),
        ("buscador_avanzado", "Buscador Avanzado", false, "OneTime", "--"),
        ("chatbot_inteligente", "Chatbot IA", true, "Recurring", "2025-02-28"),
    ];

    sdk::respond(sdk::widgets![
        sdk::card("Gestion de Plugins", vec![
            sdk::text(
                "Los plugins pueden ser de compra unica (OneTime) o suscripcion recurrente (Recurring). \
                 Las suscripciones se renuevan automaticamente cada mes desde tu saldo EzerDesk.",
                "info"
            ),
            sdk::divider(),

            sdk::card("Plugins Instalados", vec![
                sdk::table(
                    vec!["Plugin", "Nombre", "Estado", "Tipo", "Vencimiento", "Accion"],
                    plugins.iter().map(|(id, name, active, tipo, vto)| {
                        vec![
                            id,
                            name,
                            if *active { "Activo" } else { "Inactivo" },
                            tipo,
                            vto,
                            if *active { "Desactivar" } else { "Activar" },
                        ]
                    }).collect(),
                ),
                sdk::text(
                    "Los plugins Recurring se renuevan via saldo_credito. Si no hay saldo suficiente, \
                     la licencia se desactiva automaticamente.",
                    "warning"
                ),
            ]),

            sdk::card("Instalar Nuevo Plugin", vec![
                sdk::input("plugin_url", "URL del Plugin")
                    .placeholder("https://github.com/ezerdesk/plugins/..."),
                sdk::select("install_source", "Fuente", vec![
                    ("github", "GitHub"),
                    ("marketplace", "Marketplace"),
                    ("local", "Archivo Local"),
                ]),
                sdk::select("pricing_type", "Tipo de Precio", vec![
                    ("OneTime", "Compra Unica"),
                    ("Recurring", "Suscripcion Mensual"),
                ]),
                sdk::button("Instalar Plugin", "plugins", "primary"),
            ]),

            sdk::card("Plugins Disponibles", vec![
                sdk::table(
                    vec!["Plugin", "Version", "Tipo", "Precio", "Descripcion"],
                    vec![
                        vec!["reporte_satisfaccion", "1.0.0", "OneTime", "$29.99", "Reportes de satisfaccion del cliente"],
                        vec!["gestor_cupones", "1.0.0", "OneTime", "$19.99", "Sistema de cupones de descuento"],
                        vec!["integracion_email", "1.0.0", "Recurring", "$9.99/mes", "Integracion con email"],
                        vec!["monitor_red_snmp", "2.0.0", "Recurring", "$14.99/mes", "Monitoreo SNMP de dispositivos de red"],
                        vec!["chatbot_ia_premium", "1.5.0", "Recurring", "$49.99/mes", "Chatbot con IA avanzada y analitica"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::card("Resumen de Suscripciones", vec![
                sdk::table(
                    vec!["Concepto", "Monto", "Periodo", "Proximo Pago"],
                    vec![
                        vec!["gestor_tickets", "$19.99", "Mensual", "2025-03-15"],
                        vec!["chatbot_inteligente", "$9.99", "Mensual", "2025-02-28"],
                        vec!["Total Suscripciones", "$29.98/mes", "", ""],
                    ],
                ),
            ]),

            sdk::button("Exportar Configuracion", "export_config", "primary"),
        ]),
    ]);
}

fn render_backup_settings() {
    sdk::respond(sdk::widgets![
        sdk::card("Configuracion de Backup", vec![
            sdk::text("Gestiona las copias de seguridad del sistema", "info"),
            sdk::divider(),

            sdk::card("Estado del Backup", vec![
                sdk::text("Ultimo backup: 2024-01-15 03:00", "info"),
                sdk::text("Proximo backup: 2024-01-16 03:00", "info"),
                sdk::text("Tamano del ultimo backup: 2.3 GB", "info"),
                sdk::text("Ubicacion: /backups/ezerdesk/", "info"),
            ]),

            sdk::card("Programacion", vec![
                sdk::select("backup_frequency", "Frecuencia", vec![
                    ("daily", "Diario"),
                    ("weekly", "Semanal"),
                    ("monthly", "Mensual"),
                ]),
                sdk::input("backup_time", "Hora del Backup")
                    .placeholder("03:00"),
                sdk::checkbox("backup_enabled", "Backup automatico habilitado"),
                sdk::number_input("retention_days", "Retencion (dias)")
                    .min(7)
                    .max(365),
            ]),

            sdk::card("Contenido del Backup", vec![
                sdk::checkbox("backup_database", "Base de datos"),
                sdk::checkbox("backup_files", "Archivos adjuntos"),
                sdk::checkbox("backup_config", "Configuracion"),
                sdk::checkbox("backup_plugins", "Plugins"),
                sdk::checkbox("backup_logs", "Logs"),
            ]),

            sdk::card("Copias Disponibles", vec![
                sdk::table(
                    vec!["Fecha", "Tamano", "Tipo", "Estado"],
                    vec![
                        vec!["2024-01-15 03:00", "2.3 GB", "Completo", "Exitoso"],
                        vec!["2024-01-14 03:00", "2.2 GB", "Completo", "Exitoso"],
                        vec!["2024-01-13 03:00", "2.1 GB", "Completo", "Exitoso"],
                    ],
                ),
            ]),

            sdk::divider(),

            sdk::button("Crear Backup Ahora", "create_backup", "primary"),
            sdk::button("Restaurar Backup", "restore_backup", "warning"),
            sdk::button("Volver", "config", "outline"),
        ]),
    ]);
}

fn render_about() {
    sdk::respond(sdk::widgets![
        sdk::card("Acerca de EzerDesk", vec![
            sdk::text("EzerDesk - Sistema de Helpdesk", "info"),
            sdk::divider(),

            sdk::card("Informacion del Sistema", vec![
                sdk::text("Version: 2.5.0", "info"),
                sdk::text("Build: 20240115", "info"),
                sdk::text("Rust SDK: 0.9.0", "info"),
                sdk::text("Licencia: Enterprise", "success"),
            ]),

            sdk::card("Componentes", vec![
                sdk::table(
                    vec!["Componente", "Version", "Estado"],
                    vec![
                        vec!["Backend API", "2.5.0", "Activo"],
                        vec!["Frontend UI", "2.5.0", "Activo"],
                        vec!["Base de Datos", "PostgreSQL 15", "Activo"],
                        vec!["Cache", "Redis 7", "Activo"],
                        vec!["Workers", "3 activos", "Activo"],
                    ],
                ),
            ]),

            sdk::card("Soporte", vec![
                sdk::text("Documentacion: https://docs.ezerdesk.com", "info"),
                sdk::text("Soporte: soporte@ezerdesk.com", "info"),
                sdk::text("Status: https://status.ezerdesk.com", "info"),
            ]),

            sdk::divider(),

            sdk::button("Volver", "config", "outline"),
        ]),
    ]);
}

fn save_general_settings(data: &str) {
    let company = extract_field(data, "company_name").unwrap_or_default();
    let language = extract_field(data, "language").unwrap_or_default();
    let timezone = extract_field(data, "timezone").unwrap_or_default();
    let currency = extract_field(data, "currency").unwrap_or_default();

    sdk::kv_set_val("company_name", &company);
    sdk::kv_set_val("language", &language);
    sdk::kv_set_val("timezone", &timezone);
    sdk::kv_set_val("currency", &currency);

    sdk::log("Configuracion general guardada");

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion Guardada", vec![
            sdk::text("Configuracion general actualizada exitosamente", "success"),
            sdk::button("Volver", "config", "primary"),
        ]),
    ]);
}

fn save_appearance_settings(data: &str) {
    let theme = extract_field(data, "theme").unwrap_or_default();
    let primary = extract_field(data, "primary_color").unwrap_or_default();

    sdk::kv_set_val("theme", &theme);
    sdk::kv_set_val("primary_color", &primary);

    sdk::log("Configuracion de apariencia guardada");

    sdk::respond(sdk::widgets![
        sdk::card("Apariencia Actualizada", vec![
            sdk::text("Configuracion de apariencia actualizada", "success"),
            sdk::button("Volver", "config", "primary"),
        ]),
    ]);
}

fn save_security_settings(data: &str) {
    let timeout = extract_field(data, "session_timeout").unwrap_or_default();
    let max_attempts = extract_field(data, "max_login_attempts").unwrap_or_default();

    sdk::kv_set_val("session_timeout", &timeout);
    sdk::kv_set_val("max_login_attempts", &max_attempts);

    sdk::log("Configuracion de seguridad guardada");

    sdk::respond(sdk::widgets![
        sdk::card("Seguridad Actualizada", vec![
            sdk::text("Configuracion de seguridad actualizada", "success"),
            sdk::button("Volver", "config", "primary"),
        ]),
    ]);
}

fn toggle_plugin(data: &str) {
    let plugin_id = extract_field(data, "plugin_id").unwrap_or_default();
    let enabled = extract_field(data, "enabled").unwrap_or_default();

    sdk::kv_set_val(&format!("plugin_{}_enabled", plugin_id), &enabled);
    sdk::log(&format!("Plugin {} {}", plugin_id, if enabled == "true" { "activado" } else { "desactivado" }));

    sdk::respond(sdk::widgets![
        sdk::text("Estado del plugin actualizado", "success"),
        sdk::button("Volver", "plugins", "outline"),
    ]);
}

fn create_backup() {
    sdk::log("Iniciando backup manual...");
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let backup_id = format!("backup_{}", timestamp);
    sdk::kv_set_val(&backup_id, "status:completado|size:2.3GB");

    sdk::respond(sdk::widgets![
        sdk::card("Backup Creado", vec![
            sdk::text("Backup creado exitosamente", "success"),
            sdk::text(&format!("ID: {}", backup_id), "info"),
            sdk::text("Tamano: 2.3 GB", "info"),
            sdk::button("Volver", "backup", "primary"),
        ]),
    ]);
}

fn restore_backup(data: &str) {
    let backup_id = extract_field(data, "backup_id").unwrap_or_default();
    sdk::log(&format!("Restaurando backup: {}", backup_id));

    sdk::respond(sdk::widgets![
        sdk::card("Restauracion", vec![
            sdk::text("Backup restaurado exitosamente", "success"),
            sdk::text("El sistema se reiniciara en breve...", "info"),
            sdk::button("Volver", "config", "primary"),
        ]),
    ]);
}

fn reset_all_settings() {
    sdk::log("Restableciendo configuracion por defecto...");

    sdk::respond(sdk::widgets![
        sdk::card("Configuracion Restablecida", vec![
            sdk::text("Todos los ajustes han sido restablecidos a los valores por defecto", "success"),
            sdk::button("Volver", "config", "primary"),
        ]),
    ]);
}

fn export_configuration() {
    sdk::respond(sdk::widgets![
        sdk::card("Exportar Configuracion", vec![
            sdk::text("Preparando exportacion de configuracion...", "info"),
            sdk::text("Se exportara un archivo JSON con todos los ajustes", "info"),
            sdk::button("Volver", "config", "outline"),
        ]),
    ]);
}

fn extract_field(data: &str, field: &str) -> Option<String> {
    let search = format!("\"{}\":\"", field);
    if let Some(pos) = data.find(&search) {
        let start = pos + search.len();
        if let Some(end) = data[start..].find('"') {
            return Some(data[start..start + end].to_string());
        }
    }
    None
}
