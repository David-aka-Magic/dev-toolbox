use std::collections::BTreeSet;
use std::path::Path;

/// Scan a directory for font files and extract family names from filenames
fn collect_font_names_from_dir(dir: &Path, names: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_font_names_from_dir(&path, names);
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "woff" | "woff2") {
            continue;
        }

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let clean = stem
                .replace('-', " ")
                .replace('_', " ");

            // Strip common style suffixes to get the family name
            let family = clean
                .trim()
                .to_string();

            let family = strip_style_suffix(&family);

            if !family.is_empty() {
                names.insert(family);
            }
        }
    }
}

fn strip_style_suffix(name: &str) -> String {
    let suffixes = [
        "Bold Italic", "Bold Oblique", "Semi Bold Italic", "SemiBold Italic",
        "Extra Bold Italic", "ExtraBold Italic", "Ultra Bold Italic",
        "Light Italic", "Light Oblique", "Thin Italic", "Thin Oblique",
        "Medium Italic", "Medium Oblique", "Black Italic", "Black Oblique",
        "Heavy Italic", "Heavy Oblique",
        "Bold", "Italic", "Oblique", "Regular", "Medium", "Light",
        "Thin", "Black", "Heavy", "ExtraBold", "Extra Bold",
        "SemiBold", "Semi Bold", "DemiBold", "Demi Bold",
        "ExtraLight", "Extra Light", "UltraLight", "Ultra Light",
        "Book", "Condensed", "Narrow", "Wide", "Extended",
    ];

    let mut result = name.to_string();
    for suffix in &suffixes {
        if let Some(stripped) = result.strip_suffix(suffix) {
            result = stripped.trim().to_string();
        }
    }
    result
}

#[tauri::command]
pub async fn get_system_fonts() -> Result<Vec<String>, String> {
    let mut names = BTreeSet::new();

    #[cfg(target_os = "windows")]
    {
        if let Some(windir) = std::env::var_os("WINDIR") {
            let sys_fonts = Path::new(&windir).join("Fonts");
            collect_font_names_from_dir(&sys_fonts, &mut names);
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let user_fonts = Path::new(&local).join("Microsoft").join("Windows").join("Fonts");
            collect_font_names_from_dir(&user_fonts, &mut names);
        }
    }

    #[cfg(target_os = "macos")]
    {
        for dir in &["/System/Library/Fonts", "/Library/Fonts"] {
            collect_font_names_from_dir(Path::new(dir), &mut names);
        }
        if let Some(home) = std::env::var_os("HOME") {
            let user_fonts = Path::new(&home).join("Library/Fonts");
            collect_font_names_from_dir(&user_fonts, &mut names);
        }
    }

    #[cfg(target_os = "linux")]
    {
        for dir in &["/usr/share/fonts", "/usr/local/share/fonts"] {
            collect_font_names_from_dir(Path::new(dir), &mut names);
        }
        if let Some(home) = std::env::var_os("HOME") {
            let user_fonts = Path::new(&home).join(".local/share/fonts");
            collect_font_names_from_dir(&user_fonts, &mut names);
        }
    }

    Ok(names.into_iter().collect())
}