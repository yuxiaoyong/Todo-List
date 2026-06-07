use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use walkdir::WalkDir;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::db::repositories::{
    export_all_data, import_all_data, DataExportSnapshot, DataImportResult,
};
use crate::db::{
    app_data_dir, attachments_dir, checkpoint, connect, db_file_path, disconnect, remove_db_sidecars,
    with_conn,
};
use crate::error::{AppError, AppResult};

const BACKUP_FORMAT: &str = "todo-list-backup";
const BACKUP_VERSION: i32 = 1;

fn zip_error(err: zip::result::ZipError) -> AppError {
    AppError::msg(err.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    version: i32,
    format: String,
    app_version: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataInfo {
    pub app_data_dir: String,
    pub db_size_bytes: u64,
    pub attachment_count: i32,
    pub todo_count: i32,
}

pub fn get_data_info(app: &AppHandle) -> AppResult<DataInfo> {
    let data_dir = app_data_dir(app)?;
    let db_path = db_file_path(app)?;
    let db_size_bytes = fs::metadata(&db_path).map(|meta| meta.len()).unwrap_or(0);
    let (attachment_count, todo_count) = with_conn(|conn| {
        Ok((
            crate::db::repositories::count_attachments(conn)?,
            crate::db::repositories::count_active_todos(conn)?,
        ))
    })?;

    Ok(DataInfo {
        app_data_dir: data_dir.to_string_lossy().to_string(),
        db_size_bytes,
        attachment_count,
        todo_count,
    })
}

pub fn create_backup(app: &AppHandle, dest_path: &Path) -> AppResult<String> {
    checkpoint()?;
    let db_path = db_file_path(app)?;
    let attachments = attachments_dir(app)?;
    let manifest = BackupManifest {
        version: BACKUP_VERSION,
        format: BACKUP_FORMAT.into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        created_at: crate::db::now_iso(),
    };

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(dest_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    zip.start_file("manifest.json", options)
        .map_err(zip_error)?;
    zip.write_all(manifest_json.as_bytes())?;

    zip_add_file(&mut zip, "todos.db", &db_path)?;
    zip_add_dir(&mut zip, &attachments, "attachments/")?;
    zip.finish().map_err(zip_error)?;

    Ok(dest_path.to_string_lossy().to_string())
}

fn normalize_zip_entry_name(name: &str) -> AppResult<String> {
    let normalized = name.replace('\\', "/");
    if normalized.contains("..") {
        return Err(AppError::msg(format!("非法压缩项: {name}")));
    }
    let trimmed = normalized.trim_start_matches('/');
    if trimmed.is_empty() || trimmed.ends_with('/') {
        return Err(AppError::msg(format!("非法压缩项: {name}")));
    }
    Ok(trimmed.to_string())
}

pub fn restore_backup(app: &AppHandle, source_path: &Path) -> AppResult<()> {
    let file = File::open(source_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let temp_dir = app_data_dir(app)?.join(format!("restore-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)?;

    let restore_result = (|| -> AppResult<()> {
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(zip_error)?;
            let raw_name = entry.name().replace('\\', "/");
            if raw_name.ends_with('/') {
                continue;
            }
            let name = normalize_zip_entry_name(entry.name())?;
            let out_path = temp_dir.join(&name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }

        let manifest_path = temp_dir.join("manifest.json");
        let manifest_raw = fs::read_to_string(&manifest_path)
            .map_err(|_| AppError::msg("备份文件缺少 manifest.json"))?;
        let manifest: BackupManifest = serde_json::from_str(&manifest_raw)
            .map_err(|_| AppError::msg("备份 manifest 格式无效"))?;
        if manifest.format != BACKUP_FORMAT {
            return Err(AppError::msg("无效的备份文件格式"));
        }

        let extracted_db = temp_dir.join("todos.db");
        if !extracted_db.exists() {
            return Err(AppError::msg("备份文件缺少数据库"));
        }

        checkpoint()?;
        disconnect()?;

        let db_path = db_file_path(app)?;
        remove_db_sidecars(&db_path)?;
        fs::copy(&extracted_db, &db_path).map_err(|err| {
            AppError::msg(format!("无法替换数据库文件: {err}"))
        })?;
        remove_db_sidecars(&db_path)?;

        let attachments_target = attachments_dir(app)?;
        if attachments_target.exists() {
            fs::remove_dir_all(&attachments_target).map_err(|err| {
                AppError::msg(format!("无法清理附件目录: {err}"))
            })?;
        }
        fs::create_dir_all(&attachments_target)?;

        let extracted_attachments = temp_dir.join("attachments");
        if extracted_attachments.exists() {
            copy_dir_all(&extracted_attachments, &attachments_target)?;
        }

        connect(app)?;
        Ok(())
    })();

    let _ = fs::remove_dir_all(&temp_dir);

    if restore_result.is_err() {
        let _ = connect(app);
    }

    restore_result
}

pub fn export_json(_app: &AppHandle, dest_path: &Path) -> AppResult<String> {
    let snapshot: DataExportSnapshot = with_conn(export_all_data)?;
    let json = serde_json::to_string_pretty(&snapshot)?;
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest_path, json)?;
    Ok(dest_path.to_string_lossy().to_string())
}

pub fn import_json(_app: &AppHandle, source_path: &Path) -> AppResult<DataImportResult> {
    let raw = fs::read_to_string(source_path)?;
    let snapshot: DataExportSnapshot = serde_json::from_str(&raw)
        .map_err(|_| AppError::msg("JSON 格式无效，请使用本应用导出的文件"))?;
    with_conn(|conn| import_all_data(conn, snapshot))
}

fn zip_add_file<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    name: &str,
    path: &Path,
) -> AppResult<()> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    writer
        .start_file(name, SimpleFileOptions::default())
        .map_err(zip_error)?;
    writer.write_all(&buffer)?;
    Ok(())
}

fn zip_add_dir<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    dir: &Path,
    prefix: &str,
) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(dir)
            .map_err(|e| AppError::msg(e.to_string()))?;
        let name = format!(
            "{prefix}{}",
            rel.to_string_lossy().replace('\\', "/")
        );
        zip_add_file(writer, &name, path)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> AppResult<()> {
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path
            .strip_prefix(src)
            .map_err(|e| AppError::msg(e.to_string()))?;
        let target = dst.join(rel);
        if path.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target)?;
        }
    }
    Ok(())
}
