// src-tauri/src/file_settings.rs
// Commands for file manager settings functionality

use std::fs;
use std::path::PathBuf;

/// Get the thumbnail cache directory path using app data directory
fn get_cache_dir() -> Result<PathBuf, String> {
    // Use platform-specific cache directory
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| "Failed to get cache directory".to_string())?;
    
    let app_cache = cache_dir.join("com.devtoolkit.app").join("thumbnails");
    
    // Create if it doesn't exist
    if !app_cache.exists() {
        fs::create_dir_all(&app_cache)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;
    }
    
    Ok(app_cache)
}

/// Get the video preview cache directory path
fn get_preview_cache_dir() -> Result<PathBuf, String> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| "Failed to get cache directory".to_string())?;
    
    let preview_cache = cache_dir.join("com.devtoolkit.app").join("video_previews");
    
    if !preview_cache.exists() {
        fs::create_dir_all(&preview_cache)
            .map_err(|e| format!("Failed to create preview cache dir: {}", e))?;
    }
    
    Ok(preview_cache)
}

/// Calculate total size of thumbnail cache in bytes
#[tauri::command]
pub async fn get_thumbnail_cache_size() -> Result<u64, String> {
    let thumbnail_dir = get_cache_dir()?;
    let preview_dir = get_preview_cache_dir()?;
    
    let thumbnail_size = calculate_dir_size(&thumbnail_dir)?;
    let preview_size = calculate_dir_size(&preview_dir)?;
    
    Ok(thumbnail_size + preview_size)
}

/// Calculate directory size recursively
fn calculate_dir_size(path: &PathBuf) -> Result<u64, String> {
    let mut total_size: u64 = 0;
    
    if !path.exists() {
        return Ok(0);
    }
    
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let metadata = entry.metadata()
            .map_err(|e| format!("Failed to get metadata: {}", e))?;
        
        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_dir_size(&entry.path())?;
        }
    }
    
    Ok(total_size)
}

/// Clear all thumbnail and preview cache files
#[tauri::command]
pub async fn clear_thumbnail_cache() -> Result<(), String> {
    let thumbnail_dir = get_cache_dir()?;
    let preview_dir = get_preview_cache_dir()?;
    
    // Clear thumbnail cache
    if thumbnail_dir.exists() {
        clear_directory(&thumbnail_dir)?;
    }
    
    // Clear video preview cache
    if preview_dir.exists() {
        clear_directory(&preview_dir)?;
    }
    
    Ok(())
}

/// Clear all files in a directory (but keep the directory itself)
fn clear_directory(path: &PathBuf) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();
        
        if entry_path.is_file() {
            fs::remove_file(&entry_path)
                .map_err(|e| format!("Failed to remove file: {}", e))?;
        } else if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)
                .map_err(|e| format!("Failed to remove directory: {}", e))?;
        }
    }
    
    Ok(())
}

/// Get total size of a folder (for folder size display feature)
#[tauri::command]
pub async fn get_folder_size(path: String) -> Result<u64, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }
    
    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    calculate_dir_size(&path)
}

/// Count files in a directory (non-recursive, for threshold check)
#[tauri::command]
pub async fn count_files_in_directory(path: String) -> Result<u64, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }
    
    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let count = entries.count() as u64;
    Ok(count)
}

/// Enforce cache size limit by removing oldest files
#[tauri::command]
pub async fn enforce_cache_limit(max_size_mb: u64) -> Result<(), String> {
    let max_size_bytes = max_size_mb * 1024 * 1024;
    let thumbnail_dir = get_cache_dir()?;
    let preview_dir = get_preview_cache_dir()?;
    
    // Get current size
    let current_size = calculate_dir_size(&thumbnail_dir)? + calculate_dir_size(&preview_dir)?;
    
    if current_size <= max_size_bytes {
        return Ok(());
    }
    
    // Collect all cache files with their metadata
    let mut files: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
    
    collect_files_with_metadata(&thumbnail_dir, &mut files)?;
    collect_files_with_metadata(&preview_dir, &mut files)?;
    
    // Sort by modified time (oldest first)
    files.sort_by(|a, b| a.2.cmp(&b.2));
    
    // Remove oldest files until we're under the limit
    let mut removed_size: u64 = 0;
    let target_removal = current_size - max_size_bytes;
    
    for (path, size, _) in files {
        if removed_size >= target_removal {
            break;
        }
        
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to remove cache file {:?}: {}", path, e);
            continue;
        }
        
        removed_size += size;
    }
    
    Ok(())
}

/// Helper to collect files with their size and modified time
fn collect_files_with_metadata(
    dir: &PathBuf, 
    files: &mut Vec<(PathBuf, u64, std::time::SystemTime)>
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            let metadata = entry.metadata()
                .map_err(|e| format!("Failed to get metadata: {}", e))?;
            
            let modified = metadata.modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            
            files.push((path, metadata.len(), modified));
        } else if path.is_dir() {
            collect_files_with_metadata(&path, files)?;
        }
    }
    
    Ok(())
}