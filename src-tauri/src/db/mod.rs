use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub mod repositories;

static DB: OnceCell<Mutex<Option<Connection>>> = OnceCell::new();

const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("migrations/001_init.sql")),
    (2, include_str!("migrations/002_attachments.sql")),
    (3, include_str!("migrations/003_fts.sql")),
    (4, include_str!("migrations/004_settings.sql")),
    (5, include_str!("migrations/005_assignee.sql")),
    (6, include_str!("migrations/006_fts_chinese.sql")),
    (7, include_str!("migrations/007_fix_fts_porter.sql")),
    (8, include_str!("migrations/008_quadrant.sql")),
    (9, include_str!("migrations/009_kanban_columns.sql")),
    (10, include_str!("migrations/010_subtasks.sql")),
    (11, include_str!("migrations/011_start_date.sql")),
    (12, include_str!("migrations/012_tag_sort_order.sql")),
];

pub fn open_standalone(db_path: &std::path::Path) -> AppResult<Connection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    open_connection(&db_path.to_path_buf())
}

/// 初始化数据库。返回 `true` 表示本次创建了新的 `todos.db`（首次安装）。
pub fn init(app: &AppHandle) -> AppResult<bool> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::msg(e.to_string()))?;
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("todos.db");
    let is_new_db = !db_path.exists();
    let conn = open_connection(&db_path)?;
    DB.set(Mutex::new(Some(conn)))
        .map_err(|_| AppError::msg("database already initialized"))?;
    Ok(is_new_db)
}

fn open_connection(db_path: &PathBuf) -> AppResult<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    repositories::ensure_search_index(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    for (version, sql) in MIGRATIONS {
        let applied: Option<i32> = conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [*version],
                |row| row.get(0),
            )
            .optional()?;

        if applied.is_none() {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![version, Utc::now().to_rfc3339()],
            )?;
        }
    }
    Ok(())
}

pub fn with_conn<T, F>(f: F) -> AppResult<T>
where
    F: FnOnce(&Connection) -> AppResult<T>,
{
    let db = DB.get().ok_or_else(|| AppError::msg("database not initialized"))?;
    let guard = db.lock();
    let conn = guard
        .as_ref()
        .ok_or_else(|| AppError::msg("database not connected"))?;
    f(conn)
}

pub fn app_data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::msg(e.to_string()))
}

pub fn attachments_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app_data_dir(app)?.join("attachments");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn db_file_path(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app_data_dir(app)?.join("todos.db"))
}

pub fn checkpoint() -> AppResult<()> {
    with_conn(|conn| {
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    })
}

pub fn disconnect() -> AppResult<()> {
    if let Some(db) = DB.get() {
        if let Some(conn) = db.lock().as_ref() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
        *db.lock() = None;
    }
    Ok(())
}

pub fn connect(app: &AppHandle) -> AppResult<()> {
    let db_path = db_file_path(app)?;
    let conn = open_connection(&db_path)?;
    let db = DB
        .get()
        .ok_or_else(|| AppError::msg("database not initialized"))?;
    *db.lock() = Some(conn);
    Ok(())
}

pub fn remove_db_sidecars(db_path: &PathBuf) -> AppResult<()> {
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", db_path.to_string_lossy()));
        if sidecar.exists() {
            fs::remove_file(&sidecar)?;
        }
    }
    Ok(())
}

pub fn reload_connection(app: &AppHandle) -> AppResult<()> {
    disconnect()?;
    connect(app)
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn tokenize(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    static JIEBA: OnceCell<jieba_rs::Jieba> = OnceCell::new();
    let jieba = JIEBA.get_or_init(jieba_rs::Jieba::new);
    jieba
        .cut(text, false)
        .into_iter()
        .filter(|t| !t.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn html_to_text(html: &str) -> String {
    let without_tags = regex::Regex::new(r"<[^>]+>")
        .unwrap()
        .replace_all(html, " ")
        .to_string();
    html_escape::decode_html_entities(&without_tags)
        .unwrap_or(without_tags)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn markdown_to_text(md: &str) -> String {
    use pulldown_cmark::{Event, Options, Parser};

    let parser = Parser::new_ext(md, Options::empty());
    let mut out = String::new();
    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => out.push_str(&text),
            Event::SoftBreak | Event::HardBreak => out.push(' '),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn content_to_text(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('<') && (trimmed.contains("</") || trimmed.contains("/>")) {
        html_to_text(content)
    } else {
        markdown_to_text(content)
    }
}

// Simple HTML entity decode without extra crate - use manual approach
mod html_escape {
    pub fn decode_html_entities(s: &str) -> Option<String> {
        Some(
            s.replace("&nbsp;", " ")
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\""),
        )
    }
}
