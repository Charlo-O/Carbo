use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
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

#[tauri::command]
fn github_validate_repo(repo: String, token: String) -> Result<(), String> {
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

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        Err(format!("github validate failed: {} {}", status, body))
    }
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
    if meta.len() > max_bytes {
        return Err(format!(
            "file too large: {} bytes (limit {})",
            meta.len(),
            max_bytes
        ));
    }

    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;

    let ext = src
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin")
        .to_ascii_lowercase();

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
            github_validate_repo,
            github_upload_image_from_path,
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
