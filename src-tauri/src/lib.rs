use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use tauri::{path::BaseDirectory, Manager};

const GITHUB_API_VERSION: &str = "2022-11-28";

fn parse_github_repo(repo: &str) -> Result<(String, String), String> {
    let trimmed = repo.trim();
    let parts: Vec<&str> = trimmed.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() != 2 {
        return Err("repo must be in the format owner/repo".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn sanitize_git_path_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let ok = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_');
        if ok {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed.to_string()
    }
}

fn compress_image_to_jpeg_under_limit(bytes: &[u8], max_bytes: usize) -> Result<Vec<u8>, String> {
    use image::imageops::FilterType;

    let mut img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;

    // Ensure largest dimension is within a reasonable bound.
    let (w, h) = img.dimensions();
    let max_dim: u32 = 1920;
    let largest = w.max(h);
    if largest > max_dim {
        let scale = max_dim as f32 / largest as f32;
        let nw = ((w as f32) * scale).round().max(1.0) as u32;
        let nh = ((h as f32) * scale).round().max(1.0) as u32;
        img = img.resize(nw, nh, FilterType::Lanczos3);
    }

    // Drop alpha by converting to RGB.
    let img = image::DynamicImage::ImageRgb8(img.to_rgb8());

    // Try decreasing quality.
    for quality in [85u8, 80, 75, 70, 65, 60, 55, 50, 45] {
        let mut out = Vec::new();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
        enc.encode_image(&img).map_err(|e| e.to_string())?;
        if out.len() <= max_bytes {
            return Ok(out);
        }
    }

    Err("unable to compress image under size limit".to_string())
}

#[derive(Serialize)]
struct GitHubPutContentReq {
    message: String,
    content: String,
    branch: String,
}

#[derive(Deserialize)]
struct GitHubPutContentResp {
    content: GitHubContentInfo,
}

#[derive(Deserialize)]
struct GitHubContentInfo {
    path: String,
}

#[derive(Deserialize)]
struct GitHubRepoResp {
    permissions: Option<GitHubRepoPermissions>,
}

#[derive(Deserialize)]
struct GitHubRepoPermissions {
    push: Option<bool>,
    admin: Option<bool>,
}

#[derive(Serialize)]
struct GitHubValidateRepoResult {
    push: bool,
    admin: bool,
}

#[tauri::command]
fn github_validate_repo(repo: String, token: String) -> Result<GitHubValidateRepoResult, String> {
    let (owner, name) = parse_github_repo(&repo)?;
    let client = reqwest::blocking::Client::builder()
        .user_agent("Carbo Markdown Editor")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.github.com/repos/{}/{}", owner, name);
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .send()
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("github validate failed: {} {}", status, body));
    }

    let parsed: GitHubRepoResp = resp.json().map_err(|e| e.to_string())?;
    let push = parsed
        .permissions
        .as_ref()
        .and_then(|p| p.push)
        .unwrap_or(false);
    let admin = parsed
        .permissions
        .as_ref()
        .and_then(|p| p.admin)
        .unwrap_or(false);

    Ok(GitHubValidateRepoResult { push, admin })
}

#[tauri::command]
fn github_upload_image_from_path(
    repo: String,
    branch: String,
    path_prefix: String,
    token: String,
    local_path: String,
    max_bytes: u64,
) -> Result<String, String> {
    let (owner, name) = parse_github_repo(&repo)?;

    let src = std::path::Path::new(&local_path);
    let meta = std::fs::metadata(src).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("not a file".to_string());
    }

    let original = std::fs::read(src).map_err(|e| e.to_string())?;
    let target = max_bytes as usize;

    let (bytes, ext) = if (original.len() as u64) <= max_bytes {
        let ext = src
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin")
            .to_ascii_lowercase();
        (original, ext)
    } else {
        // Best effort: compress to JPEG under the given limit.
        let compressed = compress_image_to_jpeg_under_limit(&original, target)?;
        (compressed, "jpg".to_string())
    };

    let file_stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let file_stem = sanitize_git_path_component(file_stem);

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let prefix = path_prefix.trim().trim_matches('/');
    let prefix = if prefix.is_empty() { "images" } else { prefix };
    let dst_path = format!("{}/{}-{}.{}", prefix, ts, file_stem, ext);

    let content_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    let branch = branch.trim();
    let branch = if branch.is_empty() { "master" } else { branch };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Carbo Markdown Editor")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner, name, dst_path
    );

    let req_body = GitHubPutContentReq {
        message: format!("chore(images): add {}", dst_path),
        content: content_base64,
        branch: branch.to_string(),
    };

    let resp = client
        .put(&url)
        .bearer_auth(token)
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .json(&req_body)
        .send()
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("github upload failed: {} {}", status, body));
    }

    let parsed: GitHubPutContentResp = resp.json().map_err(|e| e.to_string())?;
    let raw_url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        owner, name, branch, parsed.content.path
    );

    Ok(raw_url)
}

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

fn write_export_bytes(
    dir: &PathBuf,
    file_name: &str,
    bytes: &[u8],
    ts: u128,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}-{}", ts, file_name));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
fn save_export_bytes(
    app: tauri::AppHandle,
    file_name: String,
    bytes: Vec<u8>,
    file_path: Option<String>,
) -> Result<String, String> {
    let file_name = sanitize_file_name(&file_name);

    if let Some(file_path) = file_path {
        let p = std::path::PathBuf::from(&file_path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&p, &bytes).map_err(|e| e.to_string())?;
        return Ok(p.to_string_lossy().to_string());
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    if let Ok(downloads) = app.path().resolve("Carbo Exports", BaseDirectory::Download) {
        if let Ok(path) = write_export_bytes(&downloads, &file_name, &bytes, ts) {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    let app_dir = app
        .path()
        .resolve("carbo-assets/exports", BaseDirectory::AppData)
        .map_err(|e| e.to_string())?;
    let path = write_export_bytes(&app_dir, &file_name, &bytes, ts)?;

    Ok(path.to_string_lossy().to_string())
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            save_export_bytes,
            save_image_bytes,
            copy_image_to_app_data,
            github_validate_repo,
            github_upload_image_from_path,
            read_text_file
        ])
        .setup(|app| {
            // Set window icon
            if let Some(window) = app.get_webview_window("main") {
                // Load and decode icon PNG
                let icon_bytes = include_bytes!("../icons/32x32.png");
                if let Ok(img) = image::load_from_memory(icon_bytes) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
                    let _ = window.set_icon(icon);
                }

                #[cfg(debug_assertions)]
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
