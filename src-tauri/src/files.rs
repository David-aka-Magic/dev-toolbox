use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;
use serde::Serialize;
use tauri::command;
use base64::{Engine as _, engine::general_purpose};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Serialize, Debug)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct DriveInfo {
    letter: String,
    path: String,
    label: Option<String>,
    drive_type: String,
}

const MAX_FILES_FOR_SIZE_CALC: usize = 1000;

fn calculate_dir_size(path: &Path) -> Option<u64> {
    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;
    
    fn calc_recursive(path: &Path, total_size: &mut u64, file_count: &mut usize, max_files: usize) -> bool {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                *file_count += 1;
                if *file_count > max_files {
                    return false;
                }
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if !calc_recursive(&entry_path, total_size, file_count, max_files) {
                        return false;
                    }
                } else if let Ok(metadata) = entry.metadata() {
                    *total_size += metadata.len();
                }
            }
        }
        true
    }
    
    if calc_recursive(path, &mut total_size, &mut file_count, MAX_FILES_FOR_SIZE_CALC) {
        Some(total_size)
    } else {
        None
    }
}

fn get_unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }

    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|s| s.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut i = 1;
    loop {
        let new_name = format!("{} ({}){}", file_stem, i, extension);
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        i += 1;
    }
}

fn find_ffmpeg() -> Result<String, String> {
    let mut check_cmd = Command::new("ffmpeg");
    check_cmd.arg("-version")
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null());
    
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        check_cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    if check_cmd.status().is_ok() {
        return Ok("ffmpeg".to_string());
    }
    
    let common_paths = vec![
        "C:\\ffmpeg\\ffmpeg.exe",
        "C:\\ffmpeg\\bin\\ffmpeg.exe",
        "C:\\Program Files\\ffmpeg\\bin\\ffmpeg.exe",
        "C:\\Program Files (x86)\\ffmpeg\\bin\\ffmpeg.exe",
    ];
    
    for path in common_paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    Err("FFmpeg not found. Please ensure FFmpeg is installed and in your PATH.".to_string())
}

fn create_ffmpeg_command(ffmpeg_path: &str) -> Command {
    let mut cmd = Command::new(ffmpeg_path);
    
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    cmd
}

#[command]
pub fn get_available_drives() -> Result<Vec<DriveInfo>, String> {
    let mut drives = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive_letter = (letter as char).to_string();
            let drive_path = format!("{}:\\", drive_letter);
            
            if Path::new(&drive_path).exists() {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "vol", &drive_path])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null());
                
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
                
                let label = if let Ok(output) = cmd.output() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.lines()
                        .find(|line| line.contains("Volume in drive"))
                        .and_then(|line| {
                            line.split("is").nth(1).map(|s| s.trim().to_string())
                        })
                        .filter(|s| !s.is_empty())
                } else {
                    None
                };
                
                let drive_type = get_drive_type(&drive_path);
                
                drives.push(DriveInfo {
                    letter: drive_letter,
                    path: drive_path,
                    label,
                    drive_type,
                });
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        drives.push(DriveInfo {
            letter: "/".to_string(),
            path: "/".to_string(),
            label: Some("Root".to_string()),
            drive_type: "fixed".to_string(),
        });
    }
    
    Ok(drives)
}

#[cfg(target_os = "windows")]
fn get_drive_type(path: &str) -> String {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;
    
    let path_wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    unsafe {
        match windows_sys::Win32::Storage::FileSystem::GetDriveTypeW(path_wide.as_ptr()) {
            2 => "removable".to_string(),
            3 => "fixed".to_string(),
            4 => "network".to_string(),
            5 => "cdrom".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

#[command]
pub fn read_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();
    let paths = fs::read_dir(&path).map_err(|e| e.to_string())?;

    for path_result in paths {
        if let Ok(entry) = path_result {
            let path_buf = entry.path();
            if let Some(name_os) = path_buf.file_name() {
                let name = name_os.to_string_lossy().to_string();
                
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path_buf.is_dir();
                let path_str = path_buf.to_string_lossy().to_string();

                let (size, modified) = match fs::metadata(&path_buf) {
                    Ok(meta) => {
                        let size = if is_dir {
                            None
                        } else {
                            Some(meta.len())
                        };
                        
                        let modified = meta.modified().ok().and_then(|time| {
                            time.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
                        });
                        
                        (size, modified)
                    }
                    Err(_) => (None, None),
                };

                entries.push(FileEntry { 
                    name, 
                    path: path_str, 
                    is_dir,
                    size,
                    modified,
                });
            }
        }
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(entries)
}

#[command]
pub fn get_directory_size(path: String) -> Result<Option<u64>, String> {
    let p = Path::new(&path);
    if !p.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    Ok(calculate_dir_size(p))
}

#[command]
pub fn get_directory_sizes(paths: Vec<String>) -> Result<Vec<(String, Option<u64>)>, String> {
    let results: Vec<(String, Option<u64>)> = paths
        .into_iter()
        .map(|path| {
            let p = Path::new(&path);
            let size = if p.is_dir() {
                calculate_dir_size(p)
            } else {
                None
            };
            (path, size)
        })
        .collect();
    Ok(results)
}

#[command]
pub fn delete_item(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        fs::remove_file(path).map_err(|e| e.to_string())
    }
}

#[command]
pub fn rename_item(path: String, new_name: String) -> Result<(), String> {
    let p = Path::new(&path);
    let parent = p.parent().ok_or("Cannot rename root directory or invalid path")?;
    let new_path = parent.join(new_name);
    fs::rename(path, new_path).map_err(|e| e.to_string())
}

#[command]
pub fn create_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

#[command]
pub fn create_file(path: String) -> Result<(), String> {
    if Path::new(&path).exists() {
        return Err("File already exists".to_string());
    }
    fs::write(path, "").map_err(|e| e.to_string())
}

#[command]
pub fn move_item(src: String, dest: String) -> Result<(), String> {
    let src_path = Path::new(&src);
    let dest_folder = Path::new(&dest);
    
    if !src_path.exists() || !dest_folder.is_dir() {
        return Err("Invalid source or destination".to_string());
    }

    let file_name = src_path.file_name().ok_or("Invalid source filename")?;
    let dest_path = dest_folder.join(file_name);
    let final_dest = get_unique_path(dest_path);

    fs::rename(src, final_dest).map_err(|e| e.to_string())
}

#[command]
pub fn copy_item(src: String, dest: String, new_name: Option<String>) -> Result<(), String> {
    let src_path = Path::new(&src);
    let dest_folder = Path::new(&dest);
    
    if !src_path.exists() || !dest_folder.is_dir() {
        return Err("Invalid source or destination".to_string());
    }

    let file_name = match &new_name {
        Some(name) => name.as_str(),
        None => src_path.file_name().and_then(|n| n.to_str()).ok_or("Invalid source filename")?
    };
    
    let dest_path = dest_folder.join(file_name);
    let final_dest = if new_name.is_some() { dest_path } else { get_unique_path(dest_path) };

    if src_path.is_dir() {
        copy_dir_recursive(src_path, &final_dest)?;
    } else {
        fs::copy(src_path, final_dest).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[command]
pub fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[command]
pub fn read_file_base64(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(&bytes))
}

#[command]
pub fn extract_video_thumbnail(path: String) -> Result<String, String> {
    let ffmpeg_path = find_ffmpeg()?;
    
    let temp_dir = std::env::temp_dir().join("dev-toolkit-thumbs");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_output = temp_dir.join(format!("thumb_{}.png", uuid::Uuid::new_v4()));
    
    let output = create_ffmpeg_command(&ffmpeg_path)
        .args([
            "-i", &path,
            "-vframes", "1",
            "-vf", "scale=128:128:force_original_aspect_ratio=decrease,pad=128:128:(ow-iw)/2:(oh-ih)/2",
            "-y",
            temp_output.to_str().unwrap()
        ])
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("FFmpeg execution failed: {}", e))?;

    if !output.status.success() {
        return Err("FFmpeg failed to extract thumbnail".to_string());
    }

    let bytes = fs::read(&temp_output).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&temp_output);
    
    Ok(general_purpose::STANDARD.encode(&bytes))
}

#[command]
pub fn generate_video_preview(
    path: String, 
    max_duration: u32, 
    resolution: u32, 
    fps: u32,
    use_hardware_accel: bool
) -> Result<String, String> {
    let ffmpeg_path = find_ffmpeg()?;
    
    let temp_dir = std::env::temp_dir().join("dev-toolkit-previews");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let output_path = temp_dir.join(format!("preview_{}.webm", uuid::Uuid::new_v4()));
    
    let mut args = vec![
        "-i".to_string(), path,
        "-t".to_string(), max_duration.to_string(),
        "-vf".to_string(), format!("scale={}:-1,fps={}", resolution, fps),
        "-an".to_string(),
        "-c:v".to_string(), "libvpx-vp9".to_string(),
        "-b:v".to_string(), "200k".to_string(),
        "-y".to_string(),
        output_path.to_string_lossy().to_string()
    ];
    
    if use_hardware_accel {
        args.insert(0, "-hwaccel".to_string());
        args.insert(1, "auto".to_string());
    }
    
    let output = create_ffmpeg_command(&ffmpeg_path)
        .args(&args)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("FFmpeg execution failed: {}", e))?;

    if !output.status.success() {
        return Err("FFmpeg failed to generate preview".to_string());
    }

    Ok(output_path.to_string_lossy().to_string())
}