use std::path::{Path, PathBuf};

use base64::Engine;
use tauri::AppHandle;

use crate::db::{attachments_dir, with_conn};
use crate::db::repositories::{delete_attachment_record, get_attachment, save_attachment_record, AttachmentInfo};
use crate::error::{AppError, AppResult};

pub fn todo_attachment_dir(app: &AppHandle, todo_id: i64) -> AppResult<PathBuf> {
    let dir = attachments_dir(app)?.join(todo_id.to_string());
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn validate_filename(filename: &str) -> AppResult<()> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::msg("invalid filename"));
    }
    Ok(())
}

pub fn attachment_file_path(app: &AppHandle, todo_id: i64, filename: &str) -> AppResult<PathBuf> {
    validate_filename(filename)?;
    Ok(todo_attachment_dir(app, todo_id)?.join(filename))
}

fn extension_from_name(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

fn extension_from_mime(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "application/pdf" => "pdf",
        "application/msword" => "doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "application/vnd.ms-excel" => "xls",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
        "text/plain" => "txt",
        _ => "bin",
    }
}

fn is_allowed_inline(ext: &str, mime_type: &str) -> bool {
    matches!(ext, "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
        || mime_type.starts_with("image/")
}

fn is_allowed_attachment(ext: &str, mime_type: &str) -> bool {
    if is_allowed_inline(ext, mime_type) {
        return true;
    }
    matches!(
        ext,
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "txt"
    ) || matches!(
        mime_type,
        "application/pdf"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "text/plain"
    )
}

fn validate_attachment_type(
    original_name: Option<&str>,
    mime_type: &str,
    kind: &str,
) -> AppResult<()> {
    let ext = original_name
        .and_then(|name| extension_from_name(name))
        .unwrap_or_else(|| extension_from_mime(mime_type).to_string());

    let allowed = if kind == "inline" {
        is_allowed_inline(&ext, mime_type)
    } else {
        is_allowed_attachment(&ext, mime_type)
    };

    if allowed {
        Ok(())
    } else {
        Err(AppError::msg("unsupported file type"))
    }
}

pub fn save_attachment(
    app: &AppHandle,
    todo_id: i64,
    data_base64: &str,
    original_name: Option<&str>,
    mime_type: &str,
    kind: &str,
) -> AppResult<AttachmentInfo> {
    validate_attachment_type(original_name, mime_type, kind)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64)
        .map_err(|e| AppError::msg(format!("invalid base64: {e}")))?;
    let ext = original_name
        .and_then(|n| extension_from_name(n))
        .unwrap_or_else(|| extension_from_mime(mime_type).to_string());
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    validate_filename(&filename)?;
    let path = attachment_file_path(app, todo_id, &filename)?;
    std::fs::write(&path, &bytes)?;
    with_conn(|conn| {
        save_attachment_record(
            conn,
            todo_id,
            &filename,
            original_name,
            mime_type,
            bytes.len() as i64,
            kind,
        )
    })
}

pub fn get_attachment_path(app: &AppHandle, todo_id: i64, filename: &str) -> AppResult<String> {
    let path = attachment_file_path(app, todo_id, filename)?;
    if !path.exists() {
        return Err(AppError::msg("attachment file not found"));
    }
    Ok(path.to_string_lossy().into_owned())
}

pub fn open_attachment(app: &AppHandle, todo_id: i64, filename: &str) -> AppResult<()> {
    let path = attachment_file_path(app, todo_id, filename)?;
    if !path.exists() {
        return Err(AppError::msg("attachment file not found"));
    }
    tauri_plugin_opener::open_path(path, None::<&str>)
        .map_err(|e| AppError::msg(format!("failed to open attachment: {e}")))
}

pub fn delete_attachment(app: &AppHandle, id: i64) -> AppResult<()> {
    let info = with_conn(|conn| delete_attachment_record(conn, id))?;
    let path = attachment_file_path(app, info.todo_id, &info.filename)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn read_attachment_bytes(app: &AppHandle, todo_id: i64, filename: &str) -> AppResult<Vec<u8>> {
    let path = attachment_file_path(app, todo_id, filename)?;
    Ok(std::fs::read(path)?)
}

pub fn delete_todo_attachments(app: &AppHandle, todo_id: i64) -> AppResult<()> {
    let dir = attachments_dir(app)?.join(todo_id.to_string());
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn get_attachment_by_id(_app: &AppHandle, id: i64) -> AppResult<AttachmentInfo> {
    with_conn(|conn| get_attachment(conn, id))
}
