// Ejemplo 13: Administración de Workflows
// Features: Workflow queries, Steps, Conditions, KV store
// Demuestra: Creación y gestión de flujos de trabajo automatizados

use ezerdesk_sdk as sdk;
use sdk::prelude::*;

#[sdk::main]
fn main(event: PluginEvent) -> i32 {
    match event {
        PluginEvent::GetMetadata => {
            let meta = PluginMetadata::new()
                .nav_item(
                    NavItem::new("workflows", "Workflows", "git-branch-line")
                        .category("sistema")
                        .priority(22)
                )
                .name("Administrador de Workflows")
                .description("Crea y gestiona flujos de trabajo automatizados para tickets")
                .version("1.0.0");
            sdk::to_host_response(&meta);
        }

        PluginEvent::PageRequest { page_id } => {
            match page_id.as_str() {
                "workflows" => render_workflows_dashboard(),
                "create" => render_create_workflow(),
                "detail" => render_workflow_detail(),
                "executions" => render_executions(),
                _ => {}
            }
        }

        PluginEvent::PluginAction { action, data } => {
            match action.as_str() {
                "create_workflow" => handle_create_workflow(&data),
                "activate" => activate_workflow(&data),
                "deactivate" => deactivate_workflow(&data),
                "delete" => delete_workflow(&data),
                "execute" => execute_workflow(&data),
                "view_execution" => view_execution(&data),
                _ => {
                    sdk::respond_ok("Acción no reconocida");
                }
            }
        }

        // Eventos que pueden activar workflows
        PluginEvent::TicketCreated(ticket) => {
            check_workflow_triggers("ticket_created", &ticket.id);
        }

        PluginEvent::TicketStatusChanged(ticket) => {
            check_workflow_triggers("status_changed", &ticket.id);
        }

        _ => {}
    }
    0
}

// Dashboard principal de workflows
fn render_workflows_dashboard() {
    let workflow_count = sdk::kv_get_val("workflow_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let execution_count = sdk::kv_get_val("execution_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    sdk::respond(sdk::widgets![
        sdk::card("Administrador de Workflows", vec![
            sdk::text("Automatiza procesos del helpdesk con flujos de trabajo", "info"),
            sdk::divider(),

            // Estadísticas
            sdk::card("Estadísticas", vec![
                sdk::text(&format!("⚙️ Workflows configurados: {}", workflow_count), "info"),
                sdk::text(&format!("▶️ Ejecuciones totales: {}", execution_count), "info"),
                sdk::chart("Workflows por Tipo", vec![
                    ("Automatización", 5.0),
                    ("Notificación", 3.0),
                    ("Escalamiento", 2.0),
                    ("Asignación", 4.0),
                ], "pie"),
            ]),

            // Lista de workflows
            sdk::card("Workflows Activos", vec![
                sdk::table(
                    vec!["ID", "Nombre", "Trigger", "Estado", "Ejecuciones"],
                    vec![
                        vec!["WF-001", "Notificación SLA", "ticket_created", "✅ Activo", "156"],
                        vec!["WF-002", "Escalamiento Automático", "status_changed", "✅ Activo", "89"],
                        vec!["WF-003", "Asignación por Depto", "ticket_created", "✅ Activo", "234"],
                        vec!["WF-004", "Cierre Automático", "sla_met", "⏸️ Pausado", "45"],
                    ],
                ),
            ]),

            sdk::divider(),

            // Acciones
            sdk::card("Acciones", vec![
                sdk::button("Crear Nuevo Workflow", "create", "primary"),
                sdk::button("Ver Ejecuciones", "executions", "secondary"),
            ]),
        ]),
    ]);
}

// Formulario para crear un workflow
fn render_create_workflow() {
    sdk::respond(sdk::widgets![
        sdk::card("Crear Nuevo Workflow", vec![
            sdk::text("Define un nuevo flujo de trabajo automatizado", "info"),
            sdk::divider(),

            sdk::input("workflow_name", "Nombre del Workflow")
                .placeholder("Notificación de Ticket Urgente")
                .required(true),

            sdk::textarea("workflow_description", "Descripción")
                .placeholder("Describe qué hace este workflow..."),

            sdk::select("trigger_type", "Evento Trigger", vec![
                ("ticket_created", "Cuando se crea un ticket"),
                ("status_changed", "Cuando cambia el estado"),
                ("sla_breach", "Cuando se viola un SLA"),
                ("assignment", "Cuando se asigna un ticket"),
                ("scheduled", "Ejecución programada"),
            ]),

            sdk::select("trigger_condition", "Condición", vec![
                ("priority_equals", "Prioridad es igual a"),
                ("status_equals", "Estado es igual a"),
                ("department_equals", "Departamento es igual a"),
                ("time_elapsed", "Ha pasado tiempo"),
                ("always", "Siempre"),
            ]),

            sdk::select("trigger_value", "Valor de Condición", vec![
                ("Alta", "Prioridad Alta"),
                ("Urgente", "Prioridad Urgente"),
                ("Abierto", "Estado Abierto"),
                ("Soporte", "Departamento Soporte"),
            ]),

            sdk::divider(),

            sdk::text("Acciones del Workflow", "info"),

            // Acción 1
            sdk::select("action1_type", "Acción 1", vec![
                ("notify", "Enviar notificación"),
                ("assign", "Asignar a agente"),
                ("escalate", "Escalar a supervisor"),
                ("tag", "Agregar etiqueta"),
                ("change_status", "Cambiar estado"),
                ("send_email", "Enviar email"),
            ]),

            sdk::input("action1_params", "Parámetros de Acción 1")
                .placeholder("email@ejemplo.com o nombre_agente"),

            // Acción 2
            sdk::select("action2_type", "Acción 2", vec![
                ("notify", "Enviar notificación"),
                ("assign", "Asignar a agente"),
                ("escalate", "Escalar a supervisor"),
                ("tag", "Agregar etiqueta"),
                ("none", "Sin segunda acción"),
            ]),

            sdk::input("action2_params", "Parámetros de Acción 2")
                .placeholder("Parámetros adicionales"),

            sdk::divider(),

            sdk::button("Crear Workflow", "create_workflow", "primary"),
            sdk::button("Cancelar", "workflows", "outline"),
        ]),
    ]);
}

// Detalle de un workflow
fn render_workflow_detail() {
    let workflow_id = sdk::kv_get_val("current_workflow_id")
        .unwrap_or("WF-001".to_string());

    sdk::respond(sdk::widgets![
        sdk::card("Detalle del Workflow", vec![
            sdk::text(&format!("ID: {}", workflow_id), "info"),
            sdk::divider(),
            sdk::text("Nombre: Notificación de SLA", "info"),
            sdk::text("Trigger: ticket_created", "info"),
            sdk::text("Condición: prioridad = Alta", "info"),
            sdk::text("Estado: ✅ Activo", "success"),
            sdk::divider(),

            sdk::text("Pasos del Workflow:", "info"),
            sdk::text("1. Verificar prioridad del ticket", "default"),
            sdk::text("2. Si prioridad = Alta, enviar notificación", "default"),
            skill::text("3. Asignar ticket a equipo de soporte", "default"),
            sdk::text("4. Registrar ejecución en logs", "default"),

            sdk::divider(),

            sdk::text("Estadísticas:", "info"),
            sdk::text("• Ejecuciones: 156", "default"),
            sdk::text("• Tasa de éxito: 98.7%", "default"),
            sdk::text("• Última ejecución: hace 5 minutos", "default"),

            sdk::divider(),

            sdk::button("Ejecutar Ahora", "execute", "primary"),
            sdk::button("Desactivar", "deactivate", "warning"),
            sdk::button("Eliminar", "delete", "danger"),
            sdk::button("Volver", "workflows", "outline"),
        ]),
    ]);
}

// Historial de ejecuciones
fn render_executions() {
    sdk::respond(sdk::widgets![
        sdk::card("Historial de Ejecuciones", vec![
            sdk::table(
                vec!["ID", "Workflow", "Trigger", "Estado", "Fecha"],
                vec![
                    vec!["EX-001", "Notificación SLA", "ticket_created", "✅ Éxito", "2024-01-15 09:30"],
                    vec!["EX-002", "Escalamiento Auto", "status_changed", "✅ Éxito", "2024-01-15 09:25"],
                    vec!["EX-003", "Asignación Depto", "ticket_created", "⚠️ Parcial", "2024-01-15 09:20"],
                    vec!["EX-004", "Notificación SLA", "ticket_created", "✅ Éxito", "2024-01-15 09:15"],
                    vec!["EX-005", "Cierre Automático", "sla_met", "❌ Error", "2024-01-15 09:10"],
                ],
            ),
            sdk::divider(),
            sdk::button("Volver", "workflows", "outline"),
        ]),
    ]);
}

// Crea un workflow
fn handle_create_workflow(data: &str) {
    let name = extract_field(data, "workflow_name").unwrap_or_default();
    let trigger = extract_field(data, "trigger_type").unwrap_or_default();
    let condition = extract_field(data, "trigger_condition").unwrap_or_default();
    let value = extract_field(data, "trigger_value").unwrap_or_default();
    let action1 = extract_field(data, "action1_type").unwrap_or_default();

    if name.is_empty() {
        sdk::respond(sdk::widgets![
            sdk::text("⚠️ El nombre es obligatorio", "warning"),
        ]);
        return;
    }

    let workflow_id = format!("WF-{}", chrono::Utc::now().timestamp());
    let workflow_data = format!(
        "name:{}|trigger:{}|condition:{}|value:{}|action1:{}|status:active",
        name, trigger, condition, value, action1
    );

    sdk::kv_set_val(&workflow_id, &workflow_data);

    let count = sdk::kv_get_val("workflow_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("workflow_count", &count.to_string());

    sdk::log(&format!("Workflow creado: {} ({})", name, workflow_id));

    sdk::respond(sdk::widgets![
        sdk::card("Workflow Creado", vec![
            sdk::text("✅ Workflow creado exitosamente", "success"),
            sdk::text(&format!("📋 Nombre: {}", name), "info"),
            sdk::text(&format!("⚡ Trigger: {}", trigger), "info"),
            sdk::text(&format!("🔀 Condición: {} = {}", condition, value), "info"),
            sdk::text(&format!("🎯 Acción: {}", action1), "info"),
            sdk::button("Ver Workflows", "workflows", "primary"),
        ]),
    ]);
}

// Activa un workflow
fn activate_workflow(data: &str) {
    let workflow_id = extract_field(data, "workflow_id").unwrap_or_default();
    sdk::log(&format!("Workflow activado: {}", workflow_id));

    sdk::respond(sdk::widgets![
        sdk::text("✅ Workflow activado", "success"),
        sdk::button("Volver", "workflows", "outline"),
    ]);
}

// Desactiva un workflow
fn deactivate_workflow(data: &str) {
    let workflow_id = extract_field(data, "workflow_id").unwrap_or_default();
    sdk::log(&format!("Workflow desactivado: {}", workflow_id));

    sdk::respond(sdk::widgets![
        sdk::text("⏸️ Workflow desactivado", "warning"),
        sdk::button("Volver", "workflows", "outline"),
    ]);
}

// Elimina un workflow
fn delete_workflow(data: &str) {
    let workflow_id = extract_field(data, "workflow_id").unwrap_or_default();
    if !workflow_id.is_empty() {
        sdk::kv_set_val(&workflow_id, "");
        sdk::log(&format!("Workflow eliminado: {}", workflow_id));
    }

    sdk::respond(sdk::widgets![
        sdk::text("🗑️ Workflow eliminado", "success"),
        sdk::button("Volver", "workflows", "outline"),
    ]);
}

// Ejecuta un workflow manualmente
fn execute_workflow(data: &str) {
    let workflow_id = extract_field(data, "workflow_id").unwrap_or_default();

    let count = sdk::kv_get_val("execution_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0) + 1;
    sdk::kv_set_val("execution_count", &count.to_string());

    sdk::log(&format!("Workflow ejecutado manualmente: {}", workflow_id));

    sdk::respond(sdk::widgets![
        sdk::card("Workflow Ejecutado", vec![
            sdk::text("✅ Workflow ejecutado exitosamente", "success"),
            sdk::text(&format!("📊 Total ejecuciones: {}", count), "info"),
            sdk::button("Volver", "workflows", "primary"),
        ]),
    ]);
}

// Ver ejecución específica
fn view_execution(data: &str) {
    let exec_id = extract_field(data, "execution_id").unwrap_or_default();
    sdk::kv_set_val("current_execution_id", &exec_id);

    sdk::respond(sdk::widgets![
        sdk::card("Detalle de Ejecución", vec![
            sdk::text(&format!("ID: {}", exec_id), "info"),
            sdk::text("Estado: ✅ Completada", "success"),
            sdk::text("Inicio: 2024-01-15 09:30:00", "info"),
            sdk::text("Fin: 2024-01-15 09:30:02", "info"),
            sdk::text("Duración: 2 segundos", "info"),
            sdk::divider(),
            sdk::text("Pasos ejecutados:", "info"),
            sdk::text("1. ✅ Verificar condición", "success"),
            sdk::text("2. ✅ Enviar notificación", "success"),
            sdk::text("3. ✅ Registrar log", "success"),
            sdk::button("Volver", "executions", "outline"),
        ]),
    ]);
}

// Verifica triggers de workflows para un evento
fn check_workflow_triggers(event_type: &str, entity_id: &str) {
    sdk::log(&format!(
        "Verificando workflows para evento: {} en {}",
        event_type, entity_id
    ));

    // En producción, se buscarían workflows activos que coincidan con el trigger
    let workflow_count = sdk::kv_get_val("workflow_count")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    if workflow_count > 0 {
        sdk::log(&format!("{} workflows candidatos encontrados", workflow_count));
    }
}

// Helper para extraer campos del JSON
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
