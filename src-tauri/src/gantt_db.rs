use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

/// Managed state wrapping the Gantt SQLite connection behind a Mutex.
pub struct GanttDb(pub Mutex<Connection>);

/// Open (or create) `gantt.db` at `path` and run all CREATE TABLE migrations.
pub fn initialize(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS gantt_projects (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            description  TEXT,
            start_date   TEXT NOT NULL,
            color_scheme TEXT DEFAULT 'default',
            zoom_level   TEXT DEFAULT 'weeks',
            created_at   TEXT,
            updated_at   TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_gp_name ON gantt_projects(name);

        CREATE TABLE IF NOT EXISTS gantt_tasks (
            id             TEXT PRIMARY KEY,
            project_id     TEXT NOT NULL
                               REFERENCES gantt_projects(id) ON DELETE CASCADE,
            title          TEXT NOT NULL,
            start_date     TEXT NOT NULL,
            end_date       TEXT NOT NULL,
            progress       INTEGER DEFAULT 0,
            color          TEXT DEFAULT '#3b82f6',
            group_name     TEXT,
            sort_order     INTEGER DEFAULT 0,
            collapsed      INTEGER DEFAULT 0,
            parent_task_id TEXT REFERENCES gantt_tasks(id),
            assigned_to    TEXT,
            notes          TEXT,
            created_at     TEXT,
            updated_at     TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_gt_project ON gantt_tasks(project_id);
        CREATE INDEX IF NOT EXISTS idx_gt_parent  ON gantt_tasks(parent_task_id);
        CREATE INDEX IF NOT EXISTS idx_gt_order   ON gantt_tasks(project_id, sort_order);
        CREATE INDEX IF NOT EXISTS idx_gt_dates   ON gantt_tasks(start_date, end_date);

        CREATE TABLE IF NOT EXISTS gantt_dependencies (
            id              TEXT PRIMARY KEY,
            project_id      TEXT NOT NULL
                                REFERENCES gantt_projects(id) ON DELETE CASCADE,
            from_task_id    TEXT NOT NULL
                                REFERENCES gantt_tasks(id) ON DELETE CASCADE,
            to_task_id      TEXT NOT NULL
                                REFERENCES gantt_tasks(id) ON DELETE CASCADE,
            dependency_type TEXT DEFAULT 'finish_to_start',
            UNIQUE(from_task_id, to_task_id)
        );

        CREATE INDEX IF NOT EXISTS idx_gd_project  ON gantt_dependencies(project_id);
        CREATE INDEX IF NOT EXISTS idx_gd_from     ON gantt_dependencies(from_task_id);
        CREATE INDEX IF NOT EXISTS idx_gd_to       ON gantt_dependencies(to_task_id);

        CREATE TABLE IF NOT EXISTS gantt_milestones (
            id         TEXT PRIMARY KEY,
            project_id TEXT NOT NULL
                           REFERENCES gantt_projects(id) ON DELETE CASCADE,
            title      TEXT NOT NULL,
            date       TEXT NOT NULL,
            color      TEXT DEFAULT '#f59e0b',
            sort_order INTEGER DEFAULT 0,
            created_at TEXT,
            updated_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_gm_project ON gantt_milestones(project_id);
        CREATE INDEX IF NOT EXISTS idx_gm_date    ON gantt_milestones(date);
        ",
    )?;

    Ok(conn)
}
