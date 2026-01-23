mod terminal;
mod files;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(terminal::TerminalState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Terminal Commands
            terminal::spawn_terminal,
            terminal::write_to_terminal,
            terminal::resize_terminal,
            
            // File Commands
            files::read_directory,
            files::delete_item,
            files::rename_item,
            files::create_directory,
            files::create_file,
            files::move_item,
            files::copy_item,  // Added for clipboard paste
            files::read_file,
            files::write_file,
            files::read_file_base64,
            files::extract_video_thumbnail,
            files::generate_video_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}