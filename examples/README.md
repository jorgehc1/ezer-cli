# Ejemplos de Plugins EzerDesk

Colección de plugins de ejemplo que demuestran las capacidades del motor de plugins.

## Ejemplos Disponibles

### 1. Dashboard de Métricas (`01_dashboard_metricas.rs`)
**Features:** Chart, Table, Analytics queries, Ticket stats
**Demuestra:** Queries múltiples, visualización de datos, formateo

```rust
// Consultas disponibles
sdk::query::analytics().get()      // Métricas generales
sdk::query::ticket_stats()         // Stats de tickets
sdk::query::analytics_daily()      // Tendencia diaria
sdk::query::agents().limit(10)     // Agentes con límite
sdk::query::departments().all()    // Todos los departamentos

// Widgets de UI
sdk::chart("Título", data, "bar")  // Gráfico de barras
sdk::table(headers, rows)          // Tabla de datos
sdk::card("Título", children)      // Tarjeta contenedor
sdk::text("Contenido", "style")    // Texto con estilo
sdk::button("Acción", "action", "variant")  // Botón
```

### 2. Monitor de SLA (`02_monitor_sla.rs`)
**Features:** SLA queries, Events, Notifications, Cron
**Demuestra:** Monitoreo en tiempo real, alertas, cumplimiento

```rust
// Consultas SLA
sdk::query::sla_policies()    // Políticas SLA
sdk::query::sla_events()      // Eventos de violación
sdk::query::ticket_stats()    // Tickets por prioridad

// Eventos del sistema
PluginEvent::SlaBreachDetected(breach) => { ... }
PluginEvent::TicketStatusChanged(ticket) => { ... }

// Persistencia
sdk::kv_set_val("sla_violations", &count.to_string());
sdk::kv_get_val("sla_violations")
```

### 3. Exportador de Datos (`03_exportador_datos.rs`)
**Features:** Multiple queries, Table, Filters, Export
**Demuestra:** Consultas combinadas, formateo de datos, exportación

```rust
// Consultas para exportación
sdk::query::tickets().limit(1000).all()
sdk::query::agents().limit(100).all()
sdk::query::departments().limit(50).all()
sdk::query::analytics().get()

// Generar CSV
let mut csv = String::from("ID,Asunto,Estado\n");
for t in &tickets {
    csv.push_str(&format!("{},{},{}\n", t.id, t.asunto, t.estado));
}

// Guardar para descarga
sdk::kv_set_val("export_tickets", &csv);
```

### 4. Reportes Programados (`04_reportes_programados.rs`)
**Features:** Cron, Analytics, Email, HTTP
**Demuestra:** Automatización programada, generación de reportes

```rust
// Metadata con cron
PluginMetadata::new()
    .cron("86400")  // Cada 24 horas

// Evento de cron
PluginEvent::CronTick => {
    generate_daily_report();
}

// Generar reporte
let analytics = sdk::query::analytics().get();
let report = format!("Total: {}", stats.total_tickets);
sdk::kv_set_val("last_report", &report);
```

### 5. Hub de Integraciones (`05_hub_integraciones.rs`)
**Features:** OAuth (múltiples providers), HTTP, KV store
**Demuestra:** Conexión con múltiples servicios, autenticación

```rust
// OAuth con diferentes providers
sdk::oauth_start("google")   // Google Workspace
sdk::oauth_start("slack")    // Slack
sdk::oauth_start("github")   // GitHub

// Persistir tokens
sdk::kv_set_val("google_token", &token);
sdk::kv_get_val("google_token")

// HTTP requests
let req = HttpRequest {
    method: "GET".to_string(),
    url: "https://api.github.com/user".to_string(),
    body: "".to_string(),
    headers: vec![],
};
sdk::http_request(&req)
```

## Cómo Usar los Ejemplos

1. Copia el código del ejemplo que quieras
2. Reemplaza `src/lib.rs` de tu plugin
3. Ejecuta `ezer build`
4. Ejecuta `ezer publish`
5. Activa el plugin en EzerDesk

## Features Cubiertas

| Feature | Ejemplos |
|---------|----------|
| Charts | 1, 2, 4 |
| Tables | 1, 2, 3, 4, 5 |
| Forms | 3, 5 |
| Queries | 1, 2, 3, 4 |
| Filters | 2, 3 |
| HTTP | 4, 5 |
| OAuth | 5 |
| KV Store | 1, 2, 3, 4, 5 |
| Cron | 2, 4 |
| Events | 1, 2 |
| Chat | - |
| SLA | 2 |
| Analytics | 1, 2, 4 |
| Reports | 4 |
| Integrations | 5 |

### 21. Monitor de Red SNMP (`21_monitor_red_snmp.rs`)
**Features:** HTTP requests, Charts, Table, KV store, Cron
**Demuestra:** Monitoreo de dispositivos de red via SNMP proxy/API

### 22. Receptor de Traps SNMP (`22_receptor_traps_snmp.rs`)
**Features:** HTTP, KV store, Events, Table, Charts
**Demuestra:** Recepción y procesamiento de traps SNMP via webhook

### 23. Bot de Telegram para Soporte (`23_bot_telegram_soporte.rs`)
**Features:** HTTP, KV store, Events, Chat queries, Tables
**Demuestra:** Bot de Telegram que responde consultas de soporte

### 24. Notificador de Telegram (`24_notificador_telegram.rs`)
**Features:** HTTP, KV store, Events, Charts, Cron
**Demuestra:** Sistema de notificaciones automáticas via Telegram

### 25. Generador de Facturas (`25_generador_facturas.rs`)
**Features:** Templates, HTTP, KV store, Email, Table, NumberInput
**Demuestra:** Generación de facturas, envío por email, historial

### 26. Portal de Clientes (`26_portal_clientes.rs`)
**Features:** Auth, Queries, Forms, Real-time, Table, KV store
**Demuestra:** Experiencia de cliente completa, self-service

### 27. API Integration Hub (`27_api_integration_hub.rs`)
**Features:** HTTP, OAuth, Webhooks, KV store, Table
**Demuestra:** Conexión con servicios externos, webhooks, integraciones

### 28. Integración Phone/SMS (`28_integracion_phone_sms.rs`)
**Features:** SMS sending, HTTP, KV store, Events, Table
**Demuestra:** Notificaciones SMS, creación de tickets desde SMS, comandos SMS
