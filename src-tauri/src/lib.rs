use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{path::BaseDirectory, Manager};

fn is_allowed_text_extension(path: &std::path::Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(ext.as_str(), "md" | "markdown" | "txt")
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);

    if !is_allowed_text_extension(p) {
        return Err("unsupported file type".to_string());
    }

    let meta = std::fs::metadata(p).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("not a file".to_string());
    }

    // Prevent loading extremely large files into memory.
    const MAX_BYTES: u64 = 5 * 1024 * 1024;
    if meta.len() > MAX_BYTES {
        return Err("file too large".to_string());
    }

    std::fs::read_to_string(p).map_err(|e| e.to_string())
}

fn sanitize_file_name(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let ok = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ' ');
        out.push(if ok { ch } else { '_' });
    }
    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed
    }
}

fn is_allowed_image_extension(path: &std::path::Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico"
    )
}

#[tauri::command]
fn save_image_bytes(
    app: tauri::AppHandle,
    file_name: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let file_name = sanitize_file_name(&file_name);

    let dir = app
        .path()
        .resolve("carbo-assets/images", BaseDirectory::AppData)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(format!("{}-{}", ts, file_name));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn copy_image_to_app_data(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let src = std::path::Path::new(&path);
    if !is_allowed_image_extension(src) {
        return Err("unsupported image type".to_string());
    }

    let meta = std::fs::metadata(src).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("not a file".to_string());
    }

    // Prevent loading extremely large files into memory.
    const MAX_BYTES: u64 = 20 * 1024 * 1024;
    if meta.len() > MAX_BYTES {
        return Err("file too large".to_string());
    }

    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let file_name = src.file_name().and_then(|s| s.to_str()).unwrap_or("image");
    let file_name = sanitize_file_name(file_name);

    let dir = app
        .path()
        .resolve("carbo-assets/images", BaseDirectory::AppData)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let dst = dir.join(format!("{}-{}", ts, file_name));
    std::fs::write(&dst, bytes).map_err(|e| e.to_string())?;

    Ok(dst.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            save_image_bytes,
            copy_image_to_app_data,
            read_text_file
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
