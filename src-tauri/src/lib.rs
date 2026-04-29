mod terminal;
mod files;
mod file_settings;
mod fonts;
mod planner_db;
mod planner_commands;
mod gantt_db;
mod gantt_commands;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

/// Global flag the frontend can flip via the `set_close_to_tray` command.
static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(true);

#[tauri::command]
fn set_close_to_tray(enabled: bool) {
    CLOSE_TO_TRAY.store(enabled, Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(terminal::TerminalState::default())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = planner_db::initialize(&data_dir.join("planner.db"))?;
            app.manage(planner_db::PlannerDb(std::sync::Mutex::new(conn)));
            let gconn = gantt_db::initialize(&data_dir.join("gantt.db"))?;
            app.manage(gantt_db::GanttDb(std::sync::Mutex::new(gconn)));

            // ── System tray ──────────────────────────────────────────────
            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&hide_item)
                .item(&separator)
                .item(&quit_item)
                .build()?;

            let tray_icon = Image::from_path("icons/tray-icon.png")
                .unwrap_or_else(|_| Image::from_path("icons/32x32.png").expect("fallback icon"));

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("Dev Toolkit")
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Intercept window close → hide to tray ────────────────────
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    if CLOSE_TO_TRAY.load(Ordering::Relaxed) {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_close_to_tray,

            terminal::spawn_terminal,
            terminal::write_to_terminal,
            terminal::resize_terminal,

            files::read_directory,
            files::delete_item,
            files::rename_item,
            files::create_directory,
            files::create_file,
            files::move_item,
            files::copy_item,
            files::read_file,
            files::write_file,
            files::read_file_base64,
            files::extract_video_thumbnail,
            files::generate_video_preview,
            files::get_directory_size,
            files::get_directory_sizes,
            files::get_available_drives,
            files::save_screenshot,
            files::get_playable_video,
            files::get_file_info,

            file_settings::get_thumbnail_cache_size,
            file_settings::clear_thumbnail_cache,
            file_settings::get_folder_size,
            file_settings::count_files_in_directory,
            file_settings::enforce_cache_limit,

            fonts::get_system_fonts,

            planner_commands::get_events,
            planner_commands::create_event,
            planner_commands::update_event,
            planner_commands::delete_event,
            planner_commands::get_recurring_event_instances,

            planner_commands::get_tasks,
            planner_commands::create_task,
            planner_commands::update_task,
            planner_commands::delete_task,
            planner_commands::reorder_tasks,
            planner_commands::toggle_task,

            planner_commands::get_task_lists,
            planner_commands::create_task_list,
            planner_commands::update_task_list,
            planner_commands::delete_task_list,

            planner_commands::get_time_blocks,
            planner_commands::create_time_block,
            planner_commands::update_time_block,
            planner_commands::delete_time_block,

            gantt_commands::get_gantt_projects,
            gantt_commands::get_gantt_project,
            gantt_commands::create_gantt_project,
            gantt_commands::update_gantt_project,
            gantt_commands::delete_gantt_project,

            gantt_commands::get_gantt_tasks,
            gantt_commands::create_gantt_task,
            gantt_commands::update_gantt_task,
            gantt_commands::delete_gantt_task,
            gantt_commands::reorder_gantt_tasks,
            gantt_commands::batch_update_gantt_tasks,

            gantt_commands::get_gantt_dependencies,
            gantt_commands::create_gantt_dependency,
            gantt_commands::delete_gantt_dependency,
            gantt_commands::validate_dependencies,

            gantt_commands::get_gantt_milestones,
            gantt_commands::create_gantt_milestone,
            gantt_commands::update_gantt_milestone,
            gantt_commands::delete_gantt_milestone,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
