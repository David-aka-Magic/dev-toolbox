use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Serialize;
use tauri::command;
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Debug)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
}

// --- HELPER: Handle Duplicates (e.g. "File (1).txt") ---
fn get_unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }

    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|s| s.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut i = 1;
    loop {
        // Generates: "filename (1).ext"
        let new_name = format!("{} ({}){}", file_stem, i, extension);
        let new_path = parent.join(new_name);
        
        if !new_path.exists() {
            return new_path;
        }
        i += 1;
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
                let is_dir = path_buf.is_dir();
                let path_str = path_buf.to_string_lossy().to_string();

                if !name.starts_with('.') {
                    entries.push(FileEntry { name, path: path_str, is_dir });
                }
            }
        }
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(entries)
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
pub fn create_directory(path: String, name: String) -> Result<String, String> {
    let base_path = Path::new(&path).join(name);
    let final_path = get_unique_path(base_path);
    
    fs::create_dir_all(&final_path).map_err(|e| e.to_string())?;
    
    Ok(final_path.file_name().unwrap().to_string_lossy().to_string())
}

#[command]
pub fn create_file(path: String, name: String) -> Result<String, String> {
    let base_path = Path::new(&path).join(name);
    let final_path = get_unique_path(base_path);
    
    fs::File::create(&final_path).map_err(|e| e.to_string())?;
    
    Ok(final_path.file_name().unwrap().to_string_lossy().to_string())
}

#[command]
pub fn move_item(source: String, destination: String) -> Result<(), String> {
    let src_path = Path::new(&source);
    let dest_folder = Path::new(&destination);
    
    if !src_path.exists() || !dest_folder.is_dir() {
        return Err("Invalid source or destination".to_string());
    }

    let file_name = src_path.file_name().ok_or("Invalid source filename")?;
    let dest_path = dest_folder.join(file_name);

    let final_dest = get_unique_path(dest_path);

    fs::rename(source, final_dest).map_err(|e| e.to_string())
}

#[command]
pub fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

#[command]
pub fn read_file_base64(path: String) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(&bytes))
}

fn find_ffmpeg() -> Result<String, String> {
    if Command::new("ffmpeg")
        .arg("-version")
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .is_ok()
    {
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

#[command]
pub fn extract_video_thumbnail(path: String) -> Result<String, String> {
    let ffmpeg_path = find_ffmpeg()?;
    
    let temp_dir = std::env::temp_dir().join("dev-toolkit-thumbs");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_output = temp_dir.join(format!("thumb_{}.png", uuid::Uuid::new_v4()));
    
    let output = Command::new(&ffmpeg_path)
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
        let _ = fs::remove_file(&temp_output);
        return Err("Failed to extract video frame".to_string());
    }

    let bytes = fs::read(&temp_output)
        .map_err(|e| format!("Failed to read thumbnail: {}", e))?;
    
    let _ = fs::remove_file(&temp_output);

    Ok(general_purpose::STANDARD.encode(&bytes))
}

fn detect_gpu_encoder(ffmpeg_path: &str) -> Option<String> {
    let nvenc_test = Command::new(ffmpeg_path)
        .args(["-hide_banner", "-encoders"])
        .output();
    
    if let Ok(output) = nvenc_test {
        let encoders = String::from_utf8_lossy(&output.stdout);
        
        if encoders.contains("h264_nvenc") {
            return Some("h264_nvenc".to_string());
        }
        if encoders.contains("h264_qsv") {
            return Some("h264_qsv".to_string());
        }
        if encoders.contains("h264_amf") {
            return Some("h264_amf".to_string());
        }
        if encoders.contains("h264_videotoolbox") {
            return Some("h264_videotoolbox".to_string());
        }
    }
    
    None
}

fn try_encode_video(
    ffmpeg_path: &str,
    input_path: &str,
    output_path: &Path,
    max_duration: u32,
    resolution: u32,
    fps: u32,
    encoder: &str,
    is_gpu: bool
) -> Result<(), String> {
    let mut args = vec![
        "-i".to_string(),
        input_path.to_string(),
        "-t".to_string(),
        max_duration.to_string(),
        "-vf".to_string(),
        format!("scale={}:{}:force_original_aspect_ratio=decrease,fps={}", resolution, resolution, fps),
        "-c:v".to_string(),
        encoder.to_string(),
    ];
    
    // Add encoder-specific settings
    if is_gpu {
        args.extend(["-preset".to_string(), "fast".to_string()]);
        if encoder.contains("nvenc") {
            args.extend([
                "-rc".to_string(), "vbr".to_string(),
                "-cq".to_string(), "28".to_string(),
            ]);
        }
    } else {
        args.extend([
            "-preset".to_string(), "ultrafast".to_string(),
            "-crf".to_string(), "28".to_string(),
        ]);
    }
    
    args.extend([
        "-an".to_string(),
        "-y".to_string(),
        output_path.to_str().unwrap().to_string(),
    ]);
    
    println!("   Trying encoder: {} ({})", encoder, if is_gpu { "GPU" } else { "CPU" });
    
    let output = Command::new(ffmpeg_path)
        .args(&args)
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("FFmpeg execution failed: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Encoder {} failed: {}", encoder, stderr));
    }
    
    Ok(())
}

#[command]
pub fn generate_video_preview(
    path: String,
    max_duration: u32,
    resolution: u32,
    fps: u32,
    use_hardware_accel: bool
) -> Result<String, String> {
    println!("🎬 generate_video_preview called");
    println!("   Input: {}", path);
    println!("   Settings: {}x{} @ {}fps, {}s max", resolution, resolution, fps, max_duration);
    
    let ffmpeg_path = find_ffmpeg()?;
    
    let temp_dir = std::env::temp_dir().join("dev-toolkit-previews");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let preview_output = temp_dir.join(format!("preview_{}.mp4", uuid::Uuid::new_v4()));
    
    // Try GPU encoding first if requested
    if use_hardware_accel {
        if let Some(gpu_encoder) = detect_gpu_encoder(&ffmpeg_path) {
            println!("   GPU encoder available: {}", gpu_encoder);
            
            // Try GPU encoding
            match try_encode_video(
                &ffmpeg_path,
                &path,
                &preview_output,
                max_duration,
                resolution,
                fps,
                &gpu_encoder,
                true
            ) {
                Ok(_) => {
                    println!("✅ Video preview generated successfully with GPU!");
                    return Ok(preview_output.to_str().unwrap().to_string());
                }
                Err(e) => {
                    println!("⚠️ GPU encoding failed: {}", e);
                    println!("   Falling back to CPU encoding...");
                    // Clean up failed output
                    let _ = fs::remove_file(&preview_output);
                }
            }
        } else {
            println!("   No GPU encoder detected, using CPU");
        }
    }
    
    // Fallback to software encoding
    println!("   Using software encoder: libx264");
    try_encode_video(
        &ffmpeg_path,
        &path,
        &preview_output,
        max_duration,
        resolution,
        fps,
        "libx264",
        false
    )?;
    
    println!("✅ Video preview generated successfully with CPU!");
    Ok(preview_output.to_str().unwrap().to_string())
}