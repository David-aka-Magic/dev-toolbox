use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::planner_db::PlannerDb;

// ─── Public data structures ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub all_day: bool,
    pub color: String,
    pub recurrence_rule: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct EventInput {
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub all_day: bool,
    pub color: Option<String>,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub priority: i32,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub list_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskInput {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub priority: i32,
    pub completed: bool,
    pub list_id: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    pub id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlock {
    pub id: String,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub color: String,
    pub event_id: Option<String>,
    pub task_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeBlockInput {
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub color: Option<String>,
    pub event_id: Option<String>,
    pub task_id: Option<String>,
}

// ─── Private helpers ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RecurrenceRule {
    #[serde(rename = "type")]
    rule_type: String,
    interval: i32,
    days: Option<Vec<u32>>,
    end_date: Option<String>,
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Parse the date portion of an event's time string (ISO 8601 or date-only).
fn parse_event_date(time_str: &str, all_day: bool) -> NaiveDate {
    if all_day {
        NaiveDate::parse_from_str(time_str, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
    } else {
        NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%SZ"))
            .map(|dt| dt.date())
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
    }
}

/// Compute the duration of an event in seconds, used when generating instances.
fn compute_duration_secs(start: &str, end: &str, all_day: bool) -> i64 {
    if all_day {
        return 86_400;
    }
    let parse = |s: &str| {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ"))
            .ok()
    };
    match (parse(start), parse(end)) {
        (Some(s), Some(e)) => (e - s).num_seconds().max(0),
        _ => 3_600,
    }
}

/// Build a recurrence instance by transplanting `base`'s time-of-day onto `date`.
fn make_instance(base: &Event, date: NaiveDate, duration_secs: i64) -> Event {
    let (start_time, end_time) = if base.all_day {
        let s = date.format("%Y-%m-%d").to_string();
        (s.clone(), s)
    } else {
        // Re-use the time component from the original start_time.
        let start_dt = if let Some(t) = base.start_time.splitn(2, 'T').nth(1) {
            let t = t.trim_end_matches('Z');
            let mut parts = t.split(':').filter_map(|p| p.parse::<u32>().ok());
            let h = parts.next().unwrap_or(0);
            let m = parts.next().unwrap_or(0);
            let s = parts.next().unwrap_or(0);
            date.and_hms_opt(h, m, s)
                .unwrap_or_else(|| date.and_hms_opt(0, 0, 0).unwrap())
        } else {
            date.and_hms_opt(0, 0, 0).unwrap()
        };
        let end_dt = start_dt + Duration::seconds(duration_secs);
        (
            start_dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            end_dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
        )
    };

    Event {
        id: format!("{}_{}", base.id, date.format("%Y-%m-%d")),
        start_time,
        end_time,
        ..base.clone()
    }
}

/// Returns the number of days in a given month.
fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next.and_then(|d| d.pred_opt()).map(|d| d.day()).unwrap_or(30)
}

/// Advance a date by `months` calendar months, clamping the day to a valid value.
fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    // month() is 1-based; convert to 0-based for the modular arithmetic.
    let total = (date.month() - 1) as i64 + months as i64;
    let year = date.year() + (total / 12) as i32;
    let month = (total % 12) as u32 + 1;
    let day = date.day().min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

/// Advance a date by `years` calendar years, clamping Feb 29 → Feb 28 on non-leap years.
fn add_years(date: NaiveDate, years: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year() + years as i32, date.month(), date.day())
        .unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(date.year() + years as i32, date.month(), 28)
                .unwrap_or(date)
        })
}

/// Expand a recurrence rule into concrete dates within [range_start, range_end].
///
/// `days` uses 1-based ISO weekday numbering: 1 = Monday … 7 = Sunday.
fn expand_recurrence(
    event_start: NaiveDate,
    range_start: NaiveDate,
    range_end: NaiveDate,
    rule_type: &str,
    interval: i64,
    days: &[u32],
) -> Vec<NaiveDate> {
    let mut result = Vec::new();

    match rule_type {
        "daily" => {
            let mut cur = event_start;
            while cur <= range_end {
                if cur >= range_start {
                    result.push(cur);
                }
                cur = cur + Duration::days(interval);
            }
        }

        "weekly" => {
            if days.is_empty() {
                // Simple weekly – same weekday as the event.
                let mut cur = event_start;
                while cur <= range_end {
                    if cur >= range_start {
                        result.push(cur);
                    }
                    cur = cur + Duration::weeks(interval);
                }
            } else {
                // Specific weekdays within each interval-week cycle.
                // Anchor the cycle to the Monday of event_start's week.
                let monday_offset = event_start.weekday().num_days_from_monday() as i64;
                let mut week_monday = event_start - Duration::days(monday_offset);

                while week_monday <= range_end {
                    let mut week_days: Vec<u32> = days.to_vec();
                    week_days.sort_unstable();
                    for day_num in week_days {
                        // day_num 1-7 → offset 0-6 from Monday.
                        let candidate = week_monday + Duration::days(day_num as i64 - 1);
                        if candidate >= event_start
                            && candidate >= range_start
                            && candidate <= range_end
                        {
                            result.push(candidate);
                        }
                    }
                    week_monday = week_monday + Duration::weeks(interval);
                }
            }
        }

        "monthly" => {
            let mut cur = event_start;
            while cur <= range_end {
                if cur >= range_start {
                    result.push(cur);
                }
                cur = add_months(cur, interval as u32);
            }
        }

        "yearly" => {
            let mut cur = event_start;
            while cur <= range_end {
                if cur >= range_start {
                    result.push(cur);
                }
                cur = add_years(cur, interval as u32);
            }
        }

        _ => {}
    }

    result.sort_unstable();
    result
}

// ─── Row-mapping helpers ──────────────────────────────────────────────────────

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
    Ok(Event {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        start_time: row.get(3)?,
        end_time: row.get(4)?,
        all_day: row.get::<_, i32>(5)? != 0,
        color: row.get(6)?,
        recurrence_rule: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        due_date: row.get(3)?,
        due_time: row.get(4)?,
        priority: row.get(5)?,
        completed: row.get::<_, i32>(6)? != 0,
        completed_at: row.get(7)?,
        list_id: row.get(8)?,
        sort_order: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn row_to_task_list(row: &rusqlite::Row) -> rusqlite::Result<TaskList> {
    Ok(TaskList {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        sort_order: row.get(3)?,
    })
}

fn row_to_time_block(row: &rusqlite::Row) -> rusqlite::Result<TimeBlock> {
    Ok(TimeBlock {
        id: row.get(0)?,
        title: row.get(1)?,
        start_time: row.get(2)?,
        end_time: row.get(3)?,
        color: row.get(4)?,
        event_id: row.get(5)?,
        task_id: row.get(6)?,
        created_at: row.get(7)?,
    })
}

// ─── Event commands ───────────────────────────────────────────────────────────

/// Returns non-recurring events whose date range overlaps [start_date, end_date].
#[tauri::command]
pub fn get_events(
    state: State<'_, PlannerDb>,
    start_date: String,
    end_date: String,
) -> Result<Vec<Event>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, start_time, end_time, all_day, color,
                    recurrence_rule, created_at, updated_at
             FROM   events
             WHERE  date(start_time) <= ?2
               AND  date(end_time)   >= ?1
               AND  recurrence_rule IS NULL
             ORDER  BY start_time ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![start_date, end_date], row_to_event)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn create_event(
    state: State<'_, PlannerDb>,
    event: EventInput,
) -> Result<Event, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = new_id();
    let now = now_iso();
    let color = event.color.unwrap_or_else(|| "#3b82f6".to_string());

    conn.execute(
        "INSERT INTO events
             (id, title, description, start_time, end_time, all_day, color, recurrence_rule,
              created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)",
        params![
            id,
            event.title,
            event.description,
            event.start_time,
            event.end_time,
            event.all_day as i32,
            color,
            event.recurrence_rule,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Event {
        id,
        title: event.title,
        description: event.description,
        start_time: event.start_time,
        end_time: event.end_time,
        all_day: event.all_day,
        color,
        recurrence_rule: event.recurrence_rule,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_event(
    state: State<'_, PlannerDb>,
    id: String,
    event: EventInput,
) -> Result<Event, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_iso();
    let color = event.color.unwrap_or_else(|| "#3b82f6".to_string());

    let rows = conn
        .execute(
            "UPDATE events
             SET title=?2, description=?3, start_time=?4, end_time=?5,
                 all_day=?6, color=?7, recurrence_rule=?8, updated_at=?9
             WHERE id=?1",
            params![
                id,
                event.title,
                event.description,
                event.start_time,
                event.end_time,
                event.all_day as i32,
                color,
                event.recurrence_rule,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Event '{}' not found", id));
    }

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM events WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(Event {
        id,
        title: event.title,
        description: event.description,
        start_time: event.start_time,
        end_time: event.end_time,
        all_day: event.all_day,
        color,
        recurrence_rule: event.recurrence_rule,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_event(state: State<'_, PlannerDb>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM events WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Expands all recurring events into concrete instances within [start_date, end_date].
#[tauri::command]
pub fn get_recurring_event_instances(
    state: State<'_, PlannerDb>,
    start_date: String,
    end_date: String,
) -> Result<Vec<Event>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, start_time, end_time, all_day, color,
                    recurrence_rule, created_at, updated_at
             FROM   events
             WHERE  recurrence_rule IS NOT NULL",
        )
        .map_err(|e| e.to_string())?;

    let base_events: Vec<Event> = stmt
        .query_map([], row_to_event)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let range_start =
        NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let range_end =
        NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").map_err(|e| e.to_string())?;

    let mut instances = Vec::new();

    for base in &base_events {
        let rule_str = match base.recurrence_rule.as_deref() {
            Some(r) => r,
            None => continue,
        };
        let rule: RecurrenceRule =
            serde_json::from_str(rule_str).map_err(|e| e.to_string())?;

        let event_start = parse_event_date(&base.start_time, base.all_day);
        let duration_secs = compute_duration_secs(&base.start_time, &base.end_time, base.all_day);

        let rule_end = rule
            .end_date
            .as_deref()
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(range_end);

        let effective_end = range_end.min(rule_end);
        let empty: Vec<u32> = vec![];
        let days = rule.days.as_deref().unwrap_or(&empty);

        let dates = expand_recurrence(
            event_start,
            range_start,
            effective_end,
            &rule.rule_type,
            rule.interval.max(1) as i64,
            days,
        );

        for date in dates {
            instances.push(make_instance(base, date, duration_secs));
        }
    }

    Ok(instances)
}

// ─── Task commands ────────────────────────────────────────────────────────────

/// Returns tasks optionally filtered by list and completion state.
#[tauri::command]
pub fn get_tasks(
    state: State<'_, PlannerDb>,
    list_id: Option<String>,
    include_completed: bool,
) -> Result<Vec<Task>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let sql = match (list_id.is_some(), include_completed) {
        (true, true) => {
            "SELECT id,title,description,due_date,due_time,priority,completed,completed_at,
                    list_id,sort_order,created_at,updated_at
             FROM tasks WHERE list_id=?1 ORDER BY sort_order ASC, created_at ASC"
        }
        (true, false) => {
            "SELECT id,title,description,due_date,due_time,priority,completed,completed_at,
                    list_id,sort_order,created_at,updated_at
             FROM tasks WHERE list_id=?1 AND completed=0 ORDER BY sort_order ASC, created_at ASC"
        }
        (false, true) => {
            "SELECT id,title,description,due_date,due_time,priority,completed,completed_at,
                    list_id,sort_order,created_at,updated_at
             FROM tasks ORDER BY sort_order ASC, created_at ASC"
        }
        (false, false) => {
            "SELECT id,title,description,due_date,due_time,priority,completed,completed_at,
                    list_id,sort_order,created_at,updated_at
             FROM tasks WHERE completed=0 ORDER BY sort_order ASC, created_at ASC"
        }
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let rows = if let Some(lid) = list_id {
        stmt.query_map(params![lid], row_to_task)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], row_to_task)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(rows)
}

#[tauri::command]
pub fn create_task(
    state: State<'_, PlannerDb>,
    task: TaskInput,
) -> Result<Task, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = new_id();
    let now = now_iso();
    let sort_order = task.sort_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO tasks
             (id,title,description,due_date,due_time,priority,completed,list_id,
              sort_order,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
        params![
            id,
            task.title,
            task.description,
            task.due_date,
            task.due_time,
            task.priority,
            task.completed as i32,
            task.list_id,
            sort_order,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Task {
        id,
        title: task.title,
        description: task.description,
        due_date: task.due_date,
        due_time: task.due_time,
        priority: task.priority,
        completed: task.completed,
        completed_at: None,
        list_id: task.list_id,
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_task(
    state: State<'_, PlannerDb>,
    id: String,
    task: TaskInput,
) -> Result<Task, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_iso();
    let sort_order = task.sort_order.unwrap_or(0);

    let rows = conn
        .execute(
            "UPDATE tasks
             SET title=?2, description=?3, due_date=?4, due_time=?5, priority=?6,
                 completed=?7, list_id=?8, sort_order=?9, updated_at=?10
             WHERE id=?1",
            params![
                id,
                task.title,
                task.description,
                task.due_date,
                task.due_time,
                task.priority,
                task.completed as i32,
                task.list_id,
                sort_order,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Task '{}' not found", id));
    }

    let (completed_at, created_at): (Option<String>, String) = conn
        .query_row(
            "SELECT completed_at, created_at FROM tasks WHERE id=?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(Task {
        id,
        title: task.title,
        description: task.description,
        due_date: task.due_date,
        due_time: task.due_time,
        priority: task.priority,
        completed: task.completed,
        completed_at,
        list_id: task.list_id,
        sort_order,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_task(state: State<'_, PlannerDb>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Updates sort_order for each task based on its position in `task_ids`.
#[tauri::command]
pub fn reorder_tasks(
    state: State<'_, PlannerDb>,
    task_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    for (i, id) in task_ids.iter().enumerate() {
        conn.execute(
            "UPDATE tasks SET sort_order=?2 WHERE id=?1",
            params![id, i as i32],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Flips the completed state of a task and sets/clears completed_at.
#[tauri::command]
pub fn toggle_task(state: State<'_, PlannerDb>, id: String) -> Result<Task, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = now_iso();

    let current_completed: i32 = conn
        .query_row(
            "SELECT completed FROM tasks WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let new_completed = if current_completed == 0 { 1i32 } else { 0i32 };
    let completed_at: Option<String> = if new_completed == 1 {
        Some(now.clone())
    } else {
        None
    };

    conn.execute(
        "UPDATE tasks SET completed=?2, completed_at=?3, updated_at=?4 WHERE id=?1",
        params![id, new_completed, completed_at, now],
    )
    .map_err(|e| e.to_string())?;

    let task = conn
        .query_row(
            "SELECT id,title,description,due_date,due_time,priority,completed,completed_at,
                    list_id,sort_order,created_at,updated_at
             FROM tasks WHERE id=?1",
            params![id],
            row_to_task,
        )
        .map_err(|e| e.to_string())?;

    Ok(task)
}

// ─── Task list commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn get_task_lists(state: State<'_, PlannerDb>) -> Result<Vec<TaskList>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, color, sort_order FROM task_lists ORDER BY sort_order ASC, name ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], row_to_task_list)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn create_task_list(
    state: State<'_, PlannerDb>,
    name: String,
    color: String,
) -> Result<TaskList, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = new_id();

    let sort_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM task_lists",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO task_lists (id, name, color, sort_order) VALUES (?1,?2,?3,?4)",
        params![id, name, color, sort_order],
    )
    .map_err(|e| e.to_string())?;

    Ok(TaskList { id, name, color, sort_order })
}

#[tauri::command]
pub fn update_task_list(
    state: State<'_, PlannerDb>,
    id: String,
    name: String,
    color: String,
) -> Result<TaskList, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE task_lists SET name=?2, color=?3 WHERE id=?1",
            params![id, name, color],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Task list '{}' not found", id));
    }

    let sort_order: i32 = conn
        .query_row(
            "SELECT sort_order FROM task_lists WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(TaskList { id, name, color, sort_order })
}

#[tauri::command]
pub fn delete_task_list(state: State<'_, PlannerDb>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    // Detach tasks from this list rather than deleting them.
    conn.execute("UPDATE tasks SET list_id=NULL WHERE list_id=?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM task_lists WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Time block commands ──────────────────────────────────────────────────────

/// Returns all time blocks whose start falls on `date` (YYYY-MM-DD).
#[tauri::command]
pub fn get_time_blocks(
    state: State<'_, PlannerDb>,
    date: String,
) -> Result<Vec<TimeBlock>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, start_time, end_time, color, event_id, task_id, created_at
             FROM   time_blocks
             WHERE  date(start_time) = ?1
             ORDER  BY start_time ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![date], row_to_time_block)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn create_time_block(
    state: State<'_, PlannerDb>,
    block: TimeBlockInput,
) -> Result<TimeBlock, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = new_id();
    let now = now_iso();
    let color = block.color.unwrap_or_else(|| "#8b5cf6".to_string());

    conn.execute(
        "INSERT INTO time_blocks
             (id, title, start_time, end_time, color, event_id, task_id, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            id,
            block.title,
            block.start_time,
            block.end_time,
            color,
            block.event_id,
            block.task_id,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(TimeBlock {
        id,
        title: block.title,
        start_time: block.start_time,
        end_time: block.end_time,
        color,
        event_id: block.event_id,
        task_id: block.task_id,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_time_block(
    state: State<'_, PlannerDb>,
    id: String,
    block: TimeBlockInput,
) -> Result<TimeBlock, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let color = block.color.unwrap_or_else(|| "#8b5cf6".to_string());

    let rows = conn
        .execute(
            "UPDATE time_blocks
             SET title=?2, start_time=?3, end_time=?4, color=?5, event_id=?6, task_id=?7
             WHERE id=?1",
            params![
                id,
                block.title,
                block.start_time,
                block.end_time,
                color,
                block.event_id,
                block.task_id,
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Time block '{}' not found", id));
    }

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM time_blocks WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(TimeBlock {
        id,
        title: block.title,
        start_time: block.start_time,
        end_time: block.end_time,
        color,
        event_id: block.event_id,
        task_id: block.task_id,
        created_at,
    })
}

#[tauri::command]
pub fn delete_time_block(state: State<'_, PlannerDb>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM time_blocks WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
