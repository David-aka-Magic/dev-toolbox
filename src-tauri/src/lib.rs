mod terminal;
mod files;
mod file_settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(terminal::TerminalState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
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
            
            file_settings::get_thumbnail_cache_size,
            file_settings::clear_thumbnail_cache,
            file_settings::get_folder_size,
            file_settings::count_files_in_directory,
            file_settings::enforce_cache_limit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}