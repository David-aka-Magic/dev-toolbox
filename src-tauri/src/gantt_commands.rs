use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;
use uuid::Uuid;

use crate::gantt_db::GanttDb;

// ─── Public data structures ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttProject {
    pub id:           String,
    pub name:         String,
    pub description:  Option<String>,
    pub start_date:   String,
    pub color_scheme: String,
    pub zoom_level:   String,
    pub created_at:   String,
    pub updated_at:   String,
}

#[derive(Debug, Deserialize)]
pub struct GanttProjectInput {
    pub name:         String,
    pub description:  Option<String>,
    pub start_date:   String,
    pub color_scheme: Option<String>,
    pub zoom_level:   Option<String>,
}

/// Full project payload including all related data — returned by `get_gantt_project`.
#[derive(Debug, Serialize)]
pub struct GanttProjectDetail {
    pub project:      GanttProject,
    pub tasks:        Vec<GanttTask>,
    pub dependencies: Vec<GanttDependency>,
    pub milestones:   Vec<GanttMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttTask {
    pub id:             String,
    pub project_id:     String,
    pub title:          String,
    pub start_date:     String,
    pub end_date:       String,
    pub progress:       i32,
    pub color:          String,
    pub group_name:     Option<String>,
    pub sort_order:     i32,
    pub collapsed:      bool,
    pub parent_task_id: Option<String>,
    pub assigned_to:    Option<String>,
    pub notes:          Option<String>,
    pub created_at:     String,
    pub updated_at:     String,
}

#[derive(Debug, Deserialize)]
pub struct GanttTaskInput {
    pub project_id:     String,
    pub title:          String,
    pub start_date:     String,
    pub end_date:       String,
    pub progress:       Option<i32>,
    pub color:          Option<String>,
    pub group_name:     Option<String>,
    pub sort_order:     Option<i32>,
    pub collapsed:      Option<bool>,
    pub parent_task_id: Option<String>,
    pub assigned_to:    Option<String>,
    pub notes:          Option<String>,
}

/// Sparse update used in batch drag operations — only provided fields are changed.
#[derive(Debug, Deserialize)]
pub struct GanttTaskUpdate {
    pub id:         String,
    pub start_date: Option<String>,
    pub end_date:   Option<String>,
    pub progress:   Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttDependency {
    pub id:              String,
    pub project_id:      String,
    pub from_task_id:    String,
    pub to_task_id:      String,
    pub dependency_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GanttDependencyInput {
    pub project_id:      String,
    pub from_task_id:    String,
    pub to_task_id:      String,
    pub dependency_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttMilestone {
    pub id:         String,
    pub project_id: String,
    pub title:      String,
    pub date:       String,
    pub color:      String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GanttMilestoneInput {
    pub project_id: String,
    pub title:      String,
    pub date:       String,
    pub color:      Option<String>,
    pub sort_order: Option<i32>,
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ─── Row-mapping helpers ──────────────────────────────────────────────────────

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<GanttProject> {
    Ok(GanttProject {
        id:           row.get(0)?,
        name:         row.get(1)?,
        description:  row.get(2)?,
        start_date:   row.get(3)?,
        color_scheme: row.get(4)?,
        zoom_level:   row.get(5)?,
        created_at:   row.get(6)?,
        updated_at:   row.get(7)?,
    })
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<GanttTask> {
    Ok(GanttTask {
        id:             row.get(0)?,
        project_id:     row.get(1)?,
        title:          row.get(2)?,
        start_date:     row.get(3)?,
        end_date:       row.get(4)?,
        progress:       row.get(5)?,
        color:          row.get(6)?,
        group_name:     row.get(7)?,
        sort_order:     row.get(8)?,
        collapsed:      row.get::<_, i32>(9)? != 0,
        parent_task_id: row.get(10)?,
        assigned_to:    row.get(11)?,
        notes:          row.get(12)?,
        created_at:     row.get(13)?,
        updated_at:     row.get(14)?,
    })
}

fn row_to_dependency(row: &rusqlite::Row) -> rusqlite::Result<GanttDependency> {
    Ok(GanttDependency {
        id:              row.get(0)?,
        project_id:      row.get(1)?,
        from_task_id:    row.get(2)?,
        to_task_id:      row.get(3)?,
        dependency_type: row.get(4)?,
    })
}

fn row_to_milestone(row: &rusqlite::Row) -> rusqlite::Result<GanttMilestone> {
    Ok(GanttMilestone {
        id:         row.get(0)?,
        project_id: row.get(1)?,
        title:      row.get(2)?,
        date:       row.get(3)?,
        color:      row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

// ─── Cycle detection ──────────────────────────────────────────────────────────

/// DFS-based cycle detection over a directed dependency graph.
/// Returns one warning string per back-edge (cycle detected).
fn detect_cycles(
    task_ids: &[String],
    edges: &[(String, String)],
) -> Vec<String> {
    // Build adjacency list: from_task_id → [to_task_id, ...]
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for id in task_ids {
        adj.entry(id.as_str()).or_default();
    }
    for (from, to) in edges {
        adj.entry(from.as_str()).or_default().push(to.as_str());
    }

    // 0 = unvisited, 1 = in current path, 2 = done
    let mut color: HashMap<&str, u8> = task_ids.iter().map(|id| (id.as_str(), 0u8)).collect();
    let mut warnings = Vec::new();

    for start in task_ids {
        if *color.get(start.as_str()).unwrap_or(&0) != 0 {
            continue;
        }
        // Iterative DFS: stack holds (node, index into neighbor list)
        let mut stack: Vec<(&str, usize)> = vec![(start.as_str(), 0)];
        color.insert(start.as_str(), 1);

        while let Some((node, idx)) = stack.last().copied() {
            let neighbors: Vec<&str> = adj
                .get(node)
                .cloned()
                .unwrap_or_default();

            if idx < neighbors.len() {
                // Advance the index on the top frame
                stack.last_mut().unwrap().1 += 1;
                let next = neighbors[idx];
                match color.get(next).copied().unwrap_or(0) {
                    1 => {
                        // Back-edge → cycle
                        warnings.push(format!(
                            "Circular dependency detected: task '{}' → task '{}'",
                            node, next
                        ));
                    }
                    0 => {
                        color.insert(next, 1);
                        stack.push((next, 0));
                    }
                    _ => {} // already finished, safe
                }
            } else {
                // All neighbors explored
                color.insert(node, 2);
                stack.pop();
            }
        }
    }

    warnings
}

// ─── Project commands ─────────────────────────────────────────────────────────

/// Returns all projects ordered by created_at.
#[tauri::command]
pub fn get_gantt_projects(
    state: State<'_, GanttDb>,
) -> Result<Vec<GanttProject>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, start_date, color_scheme, zoom_level,
                    created_at, updated_at
             FROM   gantt_projects
             ORDER  BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_project)
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

/// Returns a single project with all of its tasks, dependencies, and milestones.
#[tauri::command]
pub fn get_gantt_project(
    state: State<'_, GanttDb>,
    id: String,
) -> Result<GanttProjectDetail, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let project = conn
        .query_row(
            "SELECT id, name, description, start_date, color_scheme, zoom_level,
                    created_at, updated_at
             FROM   gantt_projects
             WHERE  id = ?1",
            params![id],
            row_to_project,
        )
        .map_err(|e| e.to_string())?;

    let tasks = {
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, start_date, end_date, progress, color,
                        group_name, sort_order, collapsed, parent_task_id,
                        assigned_to, notes, created_at, updated_at
                 FROM   gantt_tasks
                 WHERE  project_id = ?1
                 ORDER  BY sort_order ASC",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![id], row_to_task)
            .map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    };

    let dependencies = {
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, from_task_id, to_task_id, dependency_type
                 FROM   gantt_dependencies
                 WHERE  project_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![id], row_to_dependency)
            .map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    };

    let milestones = {
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, date, color, sort_order,
                        created_at, updated_at
                 FROM   gantt_milestones
                 WHERE  project_id = ?1
                 ORDER  BY date ASC, sort_order ASC",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![id], row_to_milestone)
            .map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(GanttProjectDetail { project, tasks, dependencies, milestones })
}

/// Creates a new project and returns it.
#[tauri::command]
pub fn create_gantt_project(
    state: State<'_, GanttDb>,
    project: GanttProjectInput,
) -> Result<GanttProject, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id  = new_id();
    let now = now_iso();
    conn.execute(
        "INSERT INTO gantt_projects
             (id, name, description, start_date, color_scheme, zoom_level,
              created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            project.name,
            project.description,
            project.start_date,
            project.color_scheme.unwrap_or_else(|| "default".into()),
            project.zoom_level.unwrap_or_else(|| "weeks".into()),
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, description, start_date, color_scheme, zoom_level,
                created_at, updated_at
         FROM   gantt_projects
         WHERE  id = ?1",
        params![id],
        row_to_project,
    )
    .map_err(|e| e.to_string())
}

/// Updates a project's metadata and returns the updated record.
#[tauri::command]
pub fn update_gantt_project(
    state: State<'_, GanttDb>,
    id: String,
    project: GanttProjectInput,
) -> Result<GanttProject, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now  = now_iso();
    conn.execute(
        "UPDATE gantt_projects
         SET    name         = ?2,
                description  = ?3,
                start_date   = ?4,
                color_scheme = ?5,
                zoom_level   = ?6,
                updated_at   = ?7
         WHERE  id = ?1",
        params![
            id,
            project.name,
            project.description,
            project.start_date,
            project.color_scheme.unwrap_or_else(|| "default".into()),
            project.zoom_level.unwrap_or_else(|| "weeks".into()),
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, description, start_date, color_scheme, zoom_level,
                created_at, updated_at
         FROM   gantt_projects
         WHERE  id = ?1",
        params![id],
        row_to_project,
    )
    .map_err(|e| e.to_string())
}

/// Deletes a project and all its tasks, dependencies, and milestones (via CASCADE).
#[tauri::command]
pub fn delete_gantt_project(
    state: State<'_, GanttDb>,
    id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM gantt_projects WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Task commands ────────────────────────────────────────────────────────────

/// Returns all tasks for a project, ordered by sort_order.
#[tauri::command]
pub fn get_gantt_tasks(
    state: State<'_, GanttDb>,
    project_id: String,
) -> Result<Vec<GanttTask>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, start_date, end_date, progress, color,
                    group_name, sort_order, collapsed, parent_task_id,
                    assigned_to, notes, created_at, updated_at
             FROM   gantt_tasks
             WHERE  project_id = ?1
             ORDER  BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![project_id], row_to_task)
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

/// Creates a new task and returns it.
#[tauri::command]
pub fn create_gantt_task(
    state: State<'_, GanttDb>,
    task: GanttTaskInput,
) -> Result<GanttTask, String> {
    let conn      = state.0.lock().map_err(|e| e.to_string())?;
    let id        = new_id();
    let now       = now_iso();
    let progress  = task.progress.unwrap_or(0).clamp(0, 100);
    let color     = task.color.unwrap_or_else(|| "#3b82f6".into());
    let order     = task.sort_order.unwrap_or(0);
    let collapsed = task.collapsed.unwrap_or(false) as i32;

    conn.execute(
        "INSERT INTO gantt_tasks
             (id, project_id, title, start_date, end_date, progress, color,
              group_name, sort_order, collapsed, parent_task_id,
              assigned_to, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
        params![
            id, task.project_id, task.title,
            task.start_date, task.end_date,
            progress, color,
            task.group_name, order, collapsed, task.parent_task_id,
            task.assigned_to, task.notes, now,
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, project_id, title, start_date, end_date, progress, color,
                group_name, sort_order, collapsed, parent_task_id,
                assigned_to, notes, created_at, updated_at
         FROM   gantt_tasks
         WHERE  id = ?1",
        params![id],
        row_to_task,
    )
    .map_err(|e| e.to_string())
}

/// Fully replaces a task's fields and returns the updated record.
#[tauri::command]
pub fn update_gantt_task(
    state: State<'_, GanttDb>,
    id: String,
    task: GanttTaskInput,
) -> Result<GanttTask, String> {
    let conn      = state.0.lock().map_err(|e| e.to_string())?;
    let now       = now_iso();
    let progress  = task.progress.unwrap_or(0).clamp(0, 100);
    let color     = task.color.unwrap_or_else(|| "#3b82f6".into());
    let order     = task.sort_order.unwrap_or(0);
    let collapsed = task.collapsed.unwrap_or(false) as i32;

    conn.execute(
        "UPDATE gantt_tasks
         SET    project_id     = ?2,
                title          = ?3,
                start_date     = ?4,
                end_date       = ?5,
                progress       = ?6,
                color          = ?7,
                group_name     = ?8,
                sort_order     = ?9,
                collapsed      = ?10,
                parent_task_id = ?11,
                assigned_to    = ?12,
                notes          = ?13,
                updated_at     = ?14
         WHERE  id = ?1",
        params![
            id, task.project_id, task.title,
            task.start_date, task.end_date,
            progress, color,
            task.group_name, order, collapsed, task.parent_task_id,
            task.assigned_to, task.notes, now,
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, project_id, title, start_date, end_date, progress, color,
                group_name, sort_order, collapsed, parent_task_id,
                assigned_to, notes, created_at, updated_at
         FROM   gantt_tasks
         WHERE  id = ?1",
        params![id],
        row_to_task,
    )
    .map_err(|e| e.to_string())
}

/// Deletes a task (and any child tasks / dependencies via CASCADE).
#[tauri::command]
pub fn delete_gantt_task(
    state: State<'_, GanttDb>,
    id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM gantt_tasks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Assigns consecutive sort_order values (0, 1, 2 …) to the supplied task IDs.
#[tauri::command]
pub fn reorder_gantt_tasks(
    state: State<'_, GanttDb>,
    task_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    for (i, id) in task_ids.iter().enumerate() {
        conn.execute(
            "UPDATE gantt_tasks SET sort_order = ?2 WHERE id = ?1",
            params![id, i as i32],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Applies sparse date / progress patches to multiple tasks in one round trip.
/// Only non-`None` fields are updated. Returns the updated tasks.
#[tauri::command]
pub fn batch_update_gantt_tasks(
    state: State<'_, GanttDb>,
    updates: Vec<GanttTaskUpdate>,
) -> Result<Vec<GanttTask>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now  = now_iso();
    let mut updated_ids: Vec<String> = Vec::with_capacity(updates.len());

    for upd in &updates {
        // Build a targeted UPDATE that only touches supplied fields.
        if let Some(ref sd) = upd.start_date {
            conn.execute(
                "UPDATE gantt_tasks SET start_date = ?2, updated_at = ?3 WHERE id = ?1",
                params![upd.id, sd, now],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(ref ed) = upd.end_date {
            conn.execute(
                "UPDATE gantt_tasks SET end_date = ?2, updated_at = ?3 WHERE id = ?1",
                params![upd.id, ed, now],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(p) = upd.progress {
            let p = p.clamp(0, 100);
            conn.execute(
                "UPDATE gantt_tasks SET progress = ?2, updated_at = ?3 WHERE id = ?1",
                params![upd.id, p, now],
            )
            .map_err(|e| e.to_string())?;
        }
        updated_ids.push(upd.id.clone());
    }

    // Re-fetch and return every touched task.
    let mut result = Vec::with_capacity(updated_ids.len());
    for id in &updated_ids {
        let task = conn
            .query_row(
                "SELECT id, project_id, title, start_date, end_date, progress, color,
                        group_name, sort_order, collapsed, parent_task_id,
                        assigned_to, notes, created_at, updated_at
                 FROM   gantt_tasks
                 WHERE  id = ?1",
                params![id],
                row_to_task,
            )
            .map_err(|e| e.to_string())?;
        result.push(task);
    }
    Ok(result)
}

// ─── Dependency commands ──────────────────────────────────────────────────────

/// Returns all dependencies for a project.
#[tauri::command]
pub fn get_gantt_dependencies(
    state: State<'_, GanttDb>,
    project_id: String,
) -> Result<Vec<GanttDependency>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, from_task_id, to_task_id, dependency_type
             FROM   gantt_dependencies
             WHERE  project_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![project_id], row_to_dependency)
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

/// Creates a dependency link between two tasks and returns the new record.
/// Returns an error if the dependency would immediately create a cycle.
#[tauri::command]
pub fn create_gantt_dependency(
    state: State<'_, GanttDb>,
    dep: GanttDependencyInput,
) -> Result<GanttDependency, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Pre-flight cycle check: fetch existing deps + the proposed one, then validate.
    let existing_edges: Vec<(String, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT from_task_id, to_task_id
                 FROM   gantt_dependencies
                 WHERE  project_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![dep.project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?
    };

    let task_ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM gantt_tasks WHERE project_id = ?1")
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![dep.project_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    };

    let mut trial_edges = existing_edges;
    trial_edges.push((dep.from_task_id.clone(), dep.to_task_id.clone()));

    let warnings = detect_cycles(&task_ids, &trial_edges);
    if !warnings.is_empty() {
        return Err(warnings.join("; "));
    }

    let id  = new_id();
    let dep_type = dep
        .dependency_type
        .unwrap_or_else(|| "finish_to_start".into());

    conn.execute(
        "INSERT INTO gantt_dependencies
             (id, project_id, from_task_id, to_task_id, dependency_type)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, dep.project_id, dep.from_task_id, dep.to_task_id, dep_type],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, project_id, from_task_id, to_task_id, dependency_type
         FROM   gantt_dependencies
         WHERE  id = ?1",
        params![id],
        row_to_dependency,
    )
    .map_err(|e| e.to_string())
}

/// Deletes a dependency link by ID.
#[tauri::command]
pub fn delete_gantt_dependency(
    state: State<'_, GanttDb>,
    id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM gantt_dependencies WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Scans all dependencies in a project and returns human-readable cycle warnings.
/// Returns an empty list when the graph is acyclic.
#[tauri::command]
pub fn validate_dependencies(
    state: State<'_, GanttDb>,
    project_id: String,
) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let task_ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM gantt_tasks WHERE project_id = ?1")
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![project_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .map(|r| r.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    };

    let edges: Vec<(String, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT from_task_id, to_task_id
                 FROM   gantt_dependencies
                 WHERE  project_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?
    };

    Ok(detect_cycles(&task_ids, &edges))
}

// ─── Milestone commands ───────────────────────────────────────────────────────

/// Returns all milestones for a project ordered by date.
#[tauri::command]
pub fn get_gantt_milestones(
    state: State<'_, GanttDb>,
    project_id: String,
) -> Result<Vec<GanttMilestone>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, date, color, sort_order,
                    created_at, updated_at
             FROM   gantt_milestones
             WHERE  project_id = ?1
             ORDER  BY date ASC, sort_order ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![project_id], row_to_milestone)
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

/// Creates a new milestone and returns it.
#[tauri::command]
pub fn create_gantt_milestone(
    state: State<'_, GanttDb>,
    milestone: GanttMilestoneInput,
) -> Result<GanttMilestone, String> {
    let conn  = state.0.lock().map_err(|e| e.to_string())?;
    let id    = new_id();
    let now   = now_iso();
    let color = milestone.color.unwrap_or_else(|| "#f59e0b".into());
    let order = milestone.sort_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO gantt_milestones
             (id, project_id, title, date, color, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![id, milestone.project_id, milestone.title,
                milestone.date, color, order, now],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, project_id, title, date, color, sort_order, created_at, updated_at
         FROM   gantt_milestones
         WHERE  id = ?1",
        params![id],
        row_to_milestone,
    )
    .map_err(|e| e.to_string())
}

/// Updates a milestone's fields and returns the updated record.
#[tauri::command]
pub fn update_gantt_milestone(
    state: State<'_, GanttDb>,
    id: String,
    milestone: GanttMilestoneInput,
) -> Result<GanttMilestone, String> {
    let conn  = state.0.lock().map_err(|e| e.to_string())?;
    let now   = now_iso();
    let color = milestone.color.unwrap_or_else(|| "#f59e0b".into());
    let order = milestone.sort_order.unwrap_or(0);

    conn.execute(
        "UPDATE gantt_milestones
         SET    project_id = ?2,
                title      = ?3,
                date       = ?4,
                color      = ?5,
                sort_order = ?6,
                updated_at = ?7
         WHERE  id = ?1",
        params![id, milestone.project_id, milestone.title,
                milestone.date, color, order, now],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, project_id, title, date, color, sort_order, created_at, updated_at
         FROM   gantt_milestones
         WHERE  id = ?1",
        params![id],
        row_to_milestone,
    )
    .map_err(|e| e.to_string())
}

/// Deletes a milestone by ID.
#[tauri::command]
pub fn delete_gantt_milestone(
    state: State<'_, GanttDb>,
    id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM gantt_milestones WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
