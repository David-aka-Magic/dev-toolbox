use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

/// Managed state wrapping the SQLite connection behind a Mutex.
pub struct PlannerDb(pub Mutex<Connection>);

/// Open (or create) the SQLite database at `path` and run all CREATE TABLE
/// migrations. Returns the ready-to-use connection.
pub fn initialize(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS events (
            id               TEXT PRIMARY KEY,
            title            TEXT NOT NULL,
            description      TEXT,
            start_time       TEXT NOT NULL,
            end_time         TEXT NOT NULL,
            all_day          INTEGER DEFAULT 0,
            color            TEXT DEFAULT '#3b82f6',
            recurrence_rule  TEXT,
            created_at       TEXT,
            updated_at       TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_events_start ON events(start_time);

        CREATE TABLE IF NOT EXISTS task_lists (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            color      TEXT DEFAULT '#3b82f6',
            sort_order INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id           TEXT PRIMARY KEY,
            title        TEXT NOT NULL,
            description  TEXT,
            due_date     TEXT,
            due_time     TEXT,
            priority     INTEGER DEFAULT 0,
            completed    INTEGER DEFAULT 0,
            completed_at TEXT,
            list_id      TEXT,
            sort_order   INTEGER DEFAULT 0,
            created_at   TEXT,
            updated_at   TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(list_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_due  ON tasks(due_date);

        CREATE TABLE IF NOT EXISTS time_blocks (
            id         TEXT PRIMARY KEY,
            title      TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time   TEXT NOT NULL,
            color      TEXT DEFAULT '#8b5cf6',
            event_id   TEXT,
            task_id    TEXT,
            created_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_time_blocks_start ON time_blocks(start_time);
        ",
    )?;

    Ok(conn)
}
