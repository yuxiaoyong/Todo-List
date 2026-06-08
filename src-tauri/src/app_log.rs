use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;
use serde::Serialize;
use tauri::AppHandle;

use crate::db::repositories::{get_setting, set_setting};
use crate::db::{app_data_dir, with_conn};
use crate::error::AppResult;

const HEALTH_PROBE_KEY: &str = "__health_probe__";
const LOG_FILE: &str = "app.log";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub ok: bool,
    pub app_data_dir: String,
    pub db_path: String,
    pub message: Option<String>,
}

pub fn append_log(app: &AppHandle, level: &str, message: &str) -> AppResult<()> {
    let dir = app_data_dir(app)?;
    fs::create_dir_all(&dir)?;
    let path = dir.join(LOG_FILE);
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("{timestamp} [{level}] {message}\n");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

fn probe_directory_writable(dir: &PathBuf) -> AppResult<()> {
    fs::create_dir_all(dir)?;
    let probe = dir.join(".write_probe");
    fs::write(&probe, b"ok")?;
    fs::remove_file(probe)?;
    Ok(())
}

pub fn health_check(app: &AppHandle) -> HealthCheckResult {
    let data_dir = match app_data_dir(app) {
        Ok(dir) => dir,
        Err(err) => {
            return HealthCheckResult {
                ok: false,
                app_data_dir: String::new(),
                db_path: String::new(),
                message: Some(format!("无法解析数据目录: {err}")),
            };
        }
    };

    let db_path = data_dir.join("todos.db");
    let app_data_dir = data_dir.to_string_lossy().into_owned();
    let db_path_str = db_path.to_string_lossy().into_owned();

    if let Err(err) = probe_directory_writable(&data_dir) {
        return HealthCheckResult {
            ok: false,
            app_data_dir,
            db_path: db_path_str,
            message: Some(format!("数据目录不可写: {err}")),
        };
    }

    if let Err(err) = with_conn(|conn| {
        set_setting(conn, HEALTH_PROBE_KEY, "ok")?;
        let _ = get_setting(conn, HEALTH_PROBE_KEY)?;
        Ok(())
    }) {
        return HealthCheckResult {
            ok: false,
            app_data_dir,
            db_path: db_path_str,
            message: Some(format!("数据库读写失败: {err}")),
        };
    }

    HealthCheckResult {
        ok: true,
        app_data_dir,
        db_path: db_path_str,
        message: None,
    }
}
