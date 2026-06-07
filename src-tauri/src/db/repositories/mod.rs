use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::db::{content_to_text, now_iso, tokenize};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub todo_count: i32,
    pub incomplete_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: String,
    pub todo_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanColumn {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub subtitle: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub todo_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoSummary {
    pub id: i64,
    pub title: String,
    pub content_text: String,
    pub completed: bool,
    pub priority: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub sort_order: i32,
    pub pinned: bool,
    pub assignee: String,
    pub kanban_column_id: Option<i64>,
    pub kanban_column_name: Option<String>,
    pub kanban_column_color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tag_ids: Vec<i64>,
    pub tag_names: Vec<String>,
    pub tag_colors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence_json: Option<RecurrenceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurrenceConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_recurrence_freq")]
    pub freq: String,
    #[serde(default = "default_recurrence_interval")]
    pub interval: u32,
    #[serde(default = "default_recurrence_anchor")]
    pub anchor: String,
    #[serde(default = "default_recurrence_calendar")]
    pub calendar: String,
    #[serde(default)]
    pub lunar_month: Option<u8>,
    #[serde(default)]
    pub lunar_day: Option<u8>,
    #[serde(default)]
    pub is_leap_month: Option<bool>,
    #[serde(default)]
    pub first_reminder_date: Option<String>,
    #[serde(default = "default_recurrence_time")]
    pub time: String,
    #[serde(default)]
    pub advance_minutes: u32,
    #[serde(default = "default_recurrence_on_complete")]
    pub on_complete: String,
}

fn default_recurrence_freq() -> String {
    "yearly".into()
}

fn default_recurrence_interval() -> u32 {
    1
}

fn default_recurrence_anchor() -> String {
    "dueDate".into()
}

fn default_recurrence_calendar() -> String {
    "solar".into()
}

fn default_recurrence_time() -> String {
    "09:00".into()
}

fn default_recurrence_on_complete() -> String {
    "reschedule".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtask {
    pub id: i64,
    pub todo_id: i64,
    pub title: String,
    pub completed: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoDetail {
    #[serde(flatten)]
    pub summary: TodoSummary,
    pub content_html: String,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryInput {
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryInput {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagInput {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKanbanColumnInput {
    pub name: String,
    pub color: Option<String>,
    pub subtitle: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKanbanColumnInput {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub subtitle: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoReorderItem {
    pub id: i64,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoListFilter {
    pub category_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
    pub completed: Option<bool>,
    pub priority: Option<String>,
    pub include_deleted: Option<bool>,
    pub search_query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTodoInput {
    pub title: String,
    pub category_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub content_html: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTodoInput {
    pub id: i64,
    pub title: String,
    pub content_html: String,
    pub completed: bool,
    pub priority: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub category_id: Option<i64>,
    pub tag_ids: Vec<i64>,
    pub sort_order: Option<i32>,
    pub pinned: Option<bool>,
    pub assignee: Option<String>,
    pub kanban_column_id: Option<i64>,
    pub quiet: Option<bool>,
    #[serde(default)]
    pub recurrence_json: Option<RecurrenceConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickCreateInput {
    pub title: String,
    pub category_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub id: i64,
    pub todo_id: i64,
    pub filename: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: i64,
    pub kind: String,
    pub url: String,
    pub created_at: String,
}

fn escape_fts_term(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn search_text_fields(title: &str, assignee: &str) -> (String, String) {
    (tokenize(title), tokenize(assignee))
}

fn append_search_filter(sql: &mut String, query: &str) {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return;
    }

    let tokens: Vec<String> = tokenize(trimmed)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if tokens.is_empty() {
        return;
    }

    let fts_query = tokens
        .iter()
        .map(|token| format!("\"{}\"", escape_fts_term(token)))
        .collect::<Vec<_>>()
        .join(" OR ")
        .replace('\'', "''");

    sql.push_str(&format!(
        " AND t.id IN (SELECT rowid FROM todos_fts WHERE todos_fts MATCH '{fts_query}')"
    ));
}

const FTS_TRIGGERS_SQL: &str = r#"
CREATE TRIGGER todos_ai AFTER INSERT ON todos BEGIN
  INSERT INTO todos_fts(rowid, title_text, content_text, tags_text, assignee_text)
  VALUES (new.id, new.title_text, new.content_text, new.tags_text, new.assignee_text);
END;

CREATE TRIGGER todos_ad AFTER DELETE ON todos BEGIN
  INSERT INTO todos_fts(todos_fts, rowid, title_text, content_text, tags_text, assignee_text)
  VALUES ('delete', old.id, old.title_text, old.content_text, old.tags_text, old.assignee_text);
END;

CREATE TRIGGER todos_au AFTER UPDATE ON todos BEGIN
  INSERT INTO todos_fts(todos_fts, rowid, title_text, content_text, tags_text, assignee_text)
  VALUES ('delete', old.id, old.title_text, old.content_text, old.tags_text, old.assignee_text);
  INSERT INTO todos_fts(rowid, title_text, content_text, tags_text, assignee_text)
  VALUES (new.id, new.title_text, new.content_text, new.tags_text, new.assignee_text);
END;
"#;

const FTS_TABLE_SQL: &str = r#"
DROP TRIGGER IF EXISTS todos_ai;
DROP TRIGGER IF EXISTS todos_ad;
DROP TRIGGER IF EXISTS todos_au;
DROP TABLE IF EXISTS todos_fts;

CREATE VIRTUAL TABLE todos_fts USING fts5(
  title_text,
  content_text,
  tags_text,
  assignee_text,
  content='todos',
  content_rowid='id',
  tokenize='porter unicode61'
);
"#;

fn drop_fts_triggers(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "DROP TRIGGER IF EXISTS todos_ai;
         DROP TRIGGER IF EXISTS todos_ad;
         DROP TRIGGER IF EXISTS todos_au;",
    )?;
    Ok(())
}

fn create_fts_triggers(conn: &Connection) -> AppResult<()> {
    drop_fts_triggers(conn)?;
    conn.execute_batch(FTS_TRIGGERS_SQL)?;
    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"),
        [column],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn fts_is_healthy(conn: &Connection) -> bool {
    conn.query_row("SELECT count(*) FROM todos_fts", [], |row| row.get::<_, i64>(0))
        .is_ok()
}

fn fts_uses_porter(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'todos_fts'",
        [],
        |row| row.get::<_, Option<String>>(0),
    )
    .ok()
    .flatten()
    .map(|sql| sql.contains("porter"))
    .unwrap_or(false)
}

fn recreate_fts_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(FTS_TABLE_SQL)?;
    create_fts_triggers(conn)?;
    Ok(())
}

fn backfill_search_text(conn: &Connection) -> AppResult<()> {
    drop_fts_triggers(conn)?;

    let mut stmt = conn.prepare(
        "SELECT id, title, COALESCE(NULLIF(TRIM(assignee), ''), '自己') AS assignee FROM todos",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let tx = conn.unchecked_transaction()?;
    for (id, title, assignee) in rows {
        let (title_text, assignee_text) = search_text_fields(&title, &assignee);
        tx.execute(
            "UPDATE todos SET title_text = ?1, assignee_text = ?2 WHERE id = ?3",
            params![title_text, assignee_text, id],
        )?;
    }
    tx.commit()?;
    create_fts_triggers(conn)?;
    Ok(())
}

fn rebuild_fts_index(conn: &Connection) -> AppResult<()> {
    conn.execute("INSERT INTO todos_fts(todos_fts) VALUES('rebuild')", [])?;
    Ok(())
}

pub fn ensure_search_index(conn: &Connection) -> AppResult<()> {
    if !has_column(conn, "todos", "title_text")? {
        return Ok(());
    }

    if !fts_is_healthy(conn) || !fts_uses_porter(conn) {
        recreate_fts_table(conn)?;
    }

    backfill_search_text(conn)?;
    rebuild_fts_index(conn)?;
    Ok(())
}

fn parse_recurrence_json(value: Option<String>) -> Option<RecurrenceConfig> {
    let raw = value?;
    if raw.trim().is_empty() {
        return None;
    }
    serde_json::from_str(&raw).ok()
}

fn serialize_recurrence_json(config: &Option<RecurrenceConfig>) -> Option<String> {
    let config = config.as_ref()?;
    serde_json::to_string(config).ok()
}

fn map_summary(row: &Row, tag_ids: Vec<i64>, tag_names: Vec<String>, tag_colors: Vec<String>) -> rusqlite::Result<TodoSummary> {
    Ok(TodoSummary {
        id: row.get("id")?,
        title: row.get("title")?,
        content_text: row.get("content_text")?,
        completed: row.get::<_, i32>("completed")? != 0,
        priority: row.get("priority")?,
        start_date: row.get("start_date")?,
        due_date: row.get("due_date")?,
        category_id: row.get("category_id")?,
        category_name: row.get("category_name")?,
        category_color: row.get("category_color")?,
        sort_order: row.get("sort_order")?,
        pinned: row.get::<_, i32>("pinned")? != 0,
        assignee: row
            .get::<_, Option<String>>("assignee")?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "自己".into()),
        kanban_column_id: row.get("kanban_column_id")?,
        kanban_column_name: row.get("kanban_column_name")?,
        kanban_column_color: row.get("kanban_column_color")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        tag_ids,
        tag_names,
        tag_colors,
        recurrence_json: parse_recurrence_json(row.get("recurrence_json")?),
    })
}

fn load_tags_for_todos(conn: &Connection, todo_ids: &[i64]) -> AppResult<std::collections::HashMap<i64, (Vec<i64>, Vec<String>, Vec<String>)>> {
    let mut map: std::collections::HashMap<i64, (Vec<i64>, Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();
    for id in todo_ids {
        map.insert(*id, (vec![], vec![], vec![]));
    }
    if todo_ids.is_empty() {
        return Ok(map);
    }
    let placeholders = todo_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT tt.todo_id, t.id, t.name, t.color FROM todo_tags tt
         JOIN tags t ON t.id = tt.tag_id
         WHERE tt.todo_id IN ({placeholders})
         ORDER BY t.name"
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<Box<dyn rusqlite::ToSql>> = todo_ids
        .iter()
        .map(|id| Box::new(*id) as Box<dyn rusqlite::ToSql>)
        .collect();
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (todo_id, tag_id, name, color) = row?;
        if let Some(entry) = map.get_mut(&todo_id) {
            entry.0.push(tag_id);
            entry.1.push(name);
            entry.2.push(color);
        }
    }
    Ok(map)
}

pub fn list_categories(conn: &Connection) -> AppResult<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.color, c.icon, c.sort_order, c.created_at,
                COUNT(t.id) AS todo_count,
                SUM(CASE WHEN t.completed = 0 AND t.deleted_at IS NULL THEN 1 ELSE 0 END) AS incomplete_count
         FROM categories c
         LEFT JOIN todos t ON t.category_id = c.id AND t.deleted_at IS NULL
         GROUP BY c.id
         ORDER BY c.sort_order, c.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            icon: row.get(3)?,
            sort_order: row.get(4)?,
            created_at: row.get(5)?,
            todo_count: row.get::<_, i32>(6)?,
            incomplete_count: row.get::<_, i32>(7)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_category(conn: &Connection, input: CreateCategoryInput) -> AppResult<Category> {
    let now = now_iso();
    let color = input.color.unwrap_or_else(|| "#409EFF".into());
    let max_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM categories",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO categories (name, color, icon, sort_order, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![input.name, color, input.icon, max_order + 1, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Category {
        id,
        name: input.name,
        color,
        icon: input.icon,
        sort_order: max_order + 1,
        created_at: now,
        todo_count: 0,
        incomplete_count: 0,
    })
}

pub fn update_category(conn: &Connection, input: UpdateCategoryInput) -> AppResult<Category> {
    conn.execute(
        "UPDATE categories SET name = ?1, color = ?2, icon = ?3 WHERE id = ?4",
        params![input.name, input.color, input.icon, input.id],
    )?;
    list_categories(conn)?
        .into_iter()
        .find(|c| c.id == input.id)
        .ok_or_else(|| AppError::msg("category not found"))
}

pub fn delete_category(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM categories WHERE id = ?1", [id])?;
    Ok(())
}

pub fn reorder_categories(conn: &Connection, ids: Vec<i64>) -> AppResult<()> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE categories SET sort_order = ?1 WHERE id = ?2",
            params![index as i32, id],
        )?;
    }
    Ok(())
}

pub fn list_kanban_columns(conn: &Connection) -> AppResult<Vec<KanbanColumn>> {
    let mut stmt = conn.prepare(
        "SELECT k.id, k.name, k.color, k.subtitle, k.sort_order, k.created_at,
                COUNT(t.id) AS todo_count
         FROM kanban_columns k
         LEFT JOIN todos t ON t.kanban_column_id = k.id AND t.deleted_at IS NULL
         GROUP BY k.id
         ORDER BY k.sort_order, k.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(KanbanColumn {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            subtitle: row.get(3)?,
            sort_order: row.get(4)?,
            created_at: row.get(5)?,
            todo_count: row.get::<_, i32>(6)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_kanban_column(conn: &Connection, input: CreateKanbanColumnInput) -> AppResult<KanbanColumn> {
    let now = now_iso();
    let color = input.color.unwrap_or_else(|| "#1677ff".into());
    let max_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM kanban_columns",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO kanban_columns (name, color, subtitle, sort_order, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![input.name, color, input.subtitle, max_order + 1, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(KanbanColumn {
        id,
        name: input.name,
        color,
        subtitle: input.subtitle,
        sort_order: max_order + 1,
        created_at: now,
        todo_count: 0,
    })
}

pub fn update_kanban_column(conn: &Connection, input: UpdateKanbanColumnInput) -> AppResult<KanbanColumn> {
    conn.execute(
        "UPDATE kanban_columns SET name = ?1, color = ?2, subtitle = ?3 WHERE id = ?4",
        params![input.name, input.color, input.subtitle, input.id],
    )?;
    list_kanban_columns(conn)?
        .into_iter()
        .find(|column| column.id == input.id)
        .ok_or_else(|| AppError::msg("kanban column not found"))
}

pub fn delete_kanban_column(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE todos SET kanban_column_id = NULL WHERE kanban_column_id = ?1",
        [id],
    )?;
    conn.execute("DELETE FROM kanban_columns WHERE id = ?1", [id])?;
    Ok(())
}

pub fn reorder_kanban_columns(conn: &Connection, ids: Vec<i64>) -> AppResult<()> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE kanban_columns SET sort_order = ?1 WHERE id = ?2",
            params![index as i32, id],
        )?;
    }
    Ok(())
}

pub fn list_tags(conn: &Connection) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.created_at,
                COUNT(tt.todo_id) AS todo_count
         FROM tags t
         LEFT JOIN todo_tags tt ON tt.tag_id = t.id
         LEFT JOIN todos td ON td.id = tt.todo_id AND td.deleted_at IS NULL
         GROUP BY t.id
         ORDER BY t.name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
            todo_count: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_tag(conn: &Connection, input: CreateTagInput) -> AppResult<Tag> {
    let now = now_iso();
    let color = input.color.unwrap_or_else(|| "#909399".into());
    conn.execute(
        "INSERT INTO tags (name, color, created_at) VALUES (?1, ?2, ?3)",
        params![input.name, color, now],
    )?;
    Ok(Tag {
        id: conn.last_insert_rowid(),
        name: input.name,
        color,
        created_at: now,
        todo_count: 0,
    })
}

pub fn update_tag(conn: &Connection, input: UpdateTagInput) -> AppResult<Tag> {
    conn.execute(
        "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3",
        params![input.name, input.color, input.id],
    )?;
    list_tags(conn)?
        .into_iter()
        .find(|t| t.id == input.id)
        .ok_or_else(|| AppError::msg("tag not found"))
}

pub fn delete_tag(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    Ok(())
}

fn sync_tags_text(conn: &Connection, todo_id: i64) -> AppResult<()> {
    let tag_names: Vec<String> = conn.prepare(
        "SELECT t.name FROM tags t
         JOIN todo_tags tt ON tt.tag_id = t.id
         WHERE tt.todo_id = ?1",
    )?
    .query_map([todo_id], |row| row.get(0))?
    .collect::<Result<Vec<_>, _>>()?;
    let tags_text = tokenize(&tag_names.join(" "));
    conn.execute(
        "UPDATE todos SET tags_text = ?1 WHERE id = ?2",
        params![tags_text, todo_id],
    )?;
    Ok(())
}

fn set_todo_tags(conn: &Connection, todo_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    conn.execute("DELETE FROM todo_tags WHERE todo_id = ?1", [todo_id])?;
    for tag_id in tag_ids {
        conn.execute(
            "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
            params![todo_id, tag_id],
        )?;
    }
    sync_tags_text(conn, todo_id)?;
    Ok(())
}

fn prepare_content(content: &str) -> (String, String) {
    let plain = content_to_text(content);
    let content_text = tokenize(&plain);
    (content.to_string(), content_text)
}

pub fn list_todos(conn: &Connection, filter: TodoListFilter) -> AppResult<Vec<TodoSummary>> {
    let mut sql = String::from(
        "SELECT t.id, t.title, t.content_text, t.completed, t.priority, t.start_date, t.due_date,
                t.category_id, c.name AS category_name, c.color AS category_color,
                t.sort_order, t.pinned, t.assignee,
                t.kanban_column_id, k.name AS kanban_column_name, k.color AS kanban_column_color,
                t.recurrence_json, t.created_at, t.updated_at
         FROM todos t
         LEFT JOIN categories c ON c.id = t.category_id
         LEFT JOIN kanban_columns k ON k.id = t.kanban_column_id
         WHERE 1=1",
    );

    if filter.include_deleted != Some(true) {
        sql.push_str(" AND t.deleted_at IS NULL");
    } else {
        sql.push_str(" AND t.deleted_at IS NOT NULL");
    }

    if let Some(category_id) = filter.category_id {
        sql.push_str(&format!(" AND t.category_id = {category_id}"));
    }
    if let Some(completed) = filter.completed {
        sql.push_str(&format!(" AND t.completed = {}", if completed { 1 } else { 0 }));
    }
    if let Some(ref priority) = filter.priority {
        sql.push_str(&format!(" AND t.priority = '{priority}'"));
    }
    if let Some(ref tag_ids) = filter.tag_ids {
        if !tag_ids.is_empty() {
            let ids = tag_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            sql.push_str(&format!(
                " AND t.id IN (SELECT todo_id FROM todo_tags WHERE tag_id IN ({ids}))"
            ));
        }
    }

    if let Some(ref query) = filter.search_query {
        append_search_filter(&mut sql, query);
    }

    sql.push_str(" ORDER BY t.pinned DESC, t.sort_order ASC, t.updated_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>("id")?,
            map_summary(row, vec![], vec![], vec![])?,
        ))
    })?;

    let mut summaries = Vec::new();
    let mut ids = Vec::new();
    for row in rows {
        let (id, summary) = row?;
        ids.push(id);
        summaries.push(summary);
    }

    let tag_map = load_tags_for_todos(conn, &ids)?;
    for summary in &mut summaries {
        if let Some((tag_ids, tag_names, tag_colors)) = tag_map.get(&summary.id) {
            summary.tag_ids = tag_ids.clone();
            summary.tag_names = tag_names.clone();
            summary.tag_colors = tag_colors.clone();
        }
    }
    Ok(summaries)
}

pub fn get_todo(conn: &Connection, id: i64) -> AppResult<TodoDetail> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.content_text, t.content_html, t.completed, t.priority, t.start_date, t.due_date,
                t.category_id, c.name AS category_name, c.color AS category_color,
                t.sort_order, t.pinned, t.assignee,
                t.kanban_column_id, k.name AS kanban_column_name, k.color AS kanban_column_color,
                t.recurrence_json, t.created_at, t.updated_at
         FROM todos t
         LEFT JOIN categories c ON c.id = t.category_id
         LEFT JOIN kanban_columns k ON k.id = t.kanban_column_id
         WHERE t.id = ?1",
    )?;
    let row = stmt.query_row([id], |row| {
        let tag_ids = vec![];
        let summary = map_summary(row, tag_ids, vec![], vec![])?;
        Ok((summary, row.get::<_, String>("content_html")?))
    })?;
    let (mut summary, content_html) = row;
    let tag_map = load_tags_for_todos(conn, &[id])?;
    if let Some((tag_ids, tag_names, tag_colors)) = tag_map.get(&id) {
        summary.tag_ids = tag_ids.clone();
        summary.tag_names = tag_names.clone();
        summary.tag_colors = tag_colors.clone();
    }
    let subtasks = list_subtasks(conn, id)?;
    Ok(TodoDetail {
        summary,
        content_html,
        subtasks,
    })
}

fn map_subtask(row: &Row<'_>) -> rusqlite::Result<Subtask> {
    Ok(Subtask {
        id: row.get("id")?,
        todo_id: row.get("todo_id")?,
        title: row.get("title")?,
        completed: row.get::<_, i32>("completed")? != 0,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_subtasks(conn: &Connection, todo_id: i64) -> AppResult<Vec<Subtask>> {
    let mut stmt = conn.prepare(
        "SELECT id, todo_id, title, completed, sort_order, created_at, updated_at
         FROM subtasks WHERE todo_id = ?1 ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([todo_id], map_subtask)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)
}

pub fn create_subtask(conn: &Connection, todo_id: i64, title: String) -> AppResult<Subtask> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::msg("子任务标题不能为空"));
    }
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE id = ?1 AND deleted_at IS NULL",
        [todo_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::msg("任务不存在或已被删除"));
    }
    let now = now_iso();
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM subtasks WHERE todo_id = ?1",
            [todo_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "INSERT INTO subtasks (todo_id, title, completed, sort_order, created_at, updated_at)
         VALUES (?1, ?2, 0, ?3, ?4, ?5)",
        params![todo_id, trimmed, max_order + 1, now, now],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, todo_id, title, completed, sort_order, created_at, updated_at
         FROM subtasks WHERE id = ?1",
        [id],
        map_subtask,
    )
    .map_err(AppError::from)
}

pub fn update_subtask(conn: &Connection, id: i64, title: String) -> AppResult<Subtask> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::msg("子任务标题不能为空"));
    }
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE subtasks SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![trimmed, now, id],
    )?;
    if updated == 0 {
        return Err(AppError::msg("子任务不存在"));
    }
    conn.query_row(
        "SELECT id, todo_id, title, completed, sort_order, created_at, updated_at
         FROM subtasks WHERE id = ?1",
        [id],
        map_subtask,
    )
    .map_err(AppError::from)
}

pub fn toggle_subtask(conn: &Connection, id: i64) -> AppResult<Subtask> {
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE subtasks SET completed = CASE completed WHEN 1 THEN 0 ELSE 1 END,
         updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    if updated == 0 {
        return Err(AppError::msg("子任务不存在"));
    }
    conn.query_row(
        "SELECT id, todo_id, title, completed, sort_order, created_at, updated_at
         FROM subtasks WHERE id = ?1",
        [id],
        map_subtask,
    )
    .map_err(AppError::from)
}

pub fn delete_subtask(conn: &Connection, id: i64) -> AppResult<()> {
    let deleted = conn.execute("DELETE FROM subtasks WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(AppError::msg("子任务不存在"));
    }
    Ok(())
}

pub fn import_subtasks_for_todo(
    conn: &Connection,
    todo_id: i64,
    subtasks: &[Subtask],
) -> AppResult<()> {
    for subtask in subtasks {
        let created = create_subtask(conn, todo_id, subtask.title.clone())?;
        if subtask.completed {
            toggle_subtask(conn, created.id)?;
        }
    }
    Ok(())
}

pub fn create_todo(conn: &Connection, input: CreateTodoInput) -> AppResult<TodoDetail> {
    let now = now_iso();
    let priority = input.priority.unwrap_or_else(|| "medium".into());
    let content_html = input.content_html.unwrap_or_default();
    let (_, content_text) = prepare_content(&content_html);
    let max_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM todos WHERE deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?;

    let assignee = "自己";
    let (title_text, assignee_text) = search_text_fields(&input.title, assignee);

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO todos (title, title_text, content_html, content_text, priority, due_date, category_id, sort_order, assignee, assignee_text, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            input.title,
            title_text,
            content_html,
            content_text,
            priority,
            input.due_date,
            input.category_id,
            max_order + 1,
            assignee,
            assignee_text,
            now,
            now
        ],
    )?;
    let id = tx.last_insert_rowid();
    if let Some(tag_ids) = input.tag_ids {
        set_todo_tags(&tx, id, &tag_ids)?;
    }
    tx.commit()?;
    get_todo(conn, id)
}

pub fn update_todo(conn: &Connection, input: UpdateTodoInput) -> AppResult<TodoDetail> {
    let now = now_iso();
    let (_, content_text) = prepare_content(&input.content_html);
    let assignee = input
        .assignee
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| {
            conn.query_row(
                "SELECT COALESCE(NULLIF(TRIM(assignee), ''), '自己') FROM todos WHERE id = ?1",
                [input.id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "自己".into())
        });
    let (title_text, assignee_text) = search_text_fields(&input.title, &assignee);

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE todos SET title = ?1, title_text = ?2, content_html = ?3, content_text = ?4, completed = ?5,
         priority = ?6, start_date = ?7, due_date = ?8, category_id = ?9, updated_at = ?10, pinned = COALESCE(?11, pinned),
         sort_order = COALESCE(?12, sort_order), assignee = ?13, assignee_text = ?14,
         kanban_column_id = ?15, recurrence_json = ?16
         WHERE id = ?17",
        params![
            input.title,
            title_text,
            input.content_html,
            content_text,
            input.completed as i32,
            input.priority,
            input.start_date,
            input.due_date,
            input.category_id,
            now,
            input.pinned.map(|p| p as i32),
            input.sort_order,
            assignee,
            assignee_text,
            input.kanban_column_id,
            serialize_recurrence_json(&input.recurrence_json),
            input.id
        ],
    )?;
    set_todo_tags(&tx, input.id, &input.tag_ids)?;
    tx.commit()?;
    get_todo(conn, input.id)
}

pub fn quick_create(conn: &Connection, input: QuickCreateInput) -> AppResult<TodoDetail> {
    create_todo(
        conn,
        CreateTodoInput {
            title: input.title,
            category_id: input.category_id,
            tag_ids: input.tag_ids,
            priority: input.priority,
            due_date: None,
            content_html: None,
        },
    )
}

pub fn toggle_complete(conn: &Connection, id: i64) -> AppResult<TodoDetail> {
    conn.execute(
        "UPDATE todos SET completed = CASE completed WHEN 1 THEN 0 ELSE 1 END, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    get_todo(conn, id)
}

pub fn toggle_pin(conn: &Connection, id: i64) -> AppResult<TodoDetail> {
    conn.execute(
        "UPDATE todos SET pinned = CASE pinned WHEN 1 THEN 0 ELSE 1 END, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    get_todo(conn, id)
}

pub fn soft_delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE todos SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    Ok(())
}

pub fn restore_todo(conn: &Connection, id: i64) -> AppResult<TodoDetail> {
    conn.execute(
        "UPDATE todos SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), id],
    )?;
    get_todo(conn, id)
}

pub fn permanent_delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list_deleted_todo_ids(conn: &Connection) -> AppResult<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT id FROM todos WHERE deleted_at IS NOT NULL")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

pub fn empty_trash(conn: &Connection) -> AppResult<u64> {
    let count = conn.execute("DELETE FROM todos WHERE deleted_at IS NOT NULL", [])?;
    Ok(count as u64)
}

pub fn set_todo_kanban_column(
    conn: &Connection,
    id: i64,
    kanban_column_id: Option<i64>,
) -> AppResult<TodoDetail> {
    if let Some(column_id) = kanban_column_id {
        let exists: Option<i64> = conn
            .query_row(
                "SELECT id FROM kanban_columns WHERE id = ?1",
                [column_id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(AppError::msg("kanban column not found"));
        }
    }
    conn.execute(
        "UPDATE todos SET kanban_column_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![kanban_column_id, now_iso(), id],
    )?;
    get_todo(conn, id)
}

pub fn reorder_todos(conn: &Connection, ids: Vec<i64>) -> AppResult<()> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE todos SET sort_order = ?1 WHERE id = ?2",
            params![index as i32, id],
        )?;
    }
    Ok(())
}

pub fn reorder_todos_positions(conn: &Connection, items: Vec<TodoReorderItem>) -> AppResult<()> {
    for item in items {
        conn.execute(
            "UPDATE todos SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![item.sort_order, now_iso(), item.id],
        )?;
    }
    Ok(())
}

pub fn incomplete_count(conn: &Connection) -> AppResult<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE completed = 0 AND deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn due_today_todos(conn: &Connection) -> AppResult<Vec<TodoSummary>> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    list_todos(
        conn,
        TodoListFilter {
            category_id: None,
            tag_ids: None,
            completed: Some(false),
            priority: None,
            include_deleted: Some(false),
            search_query: None,
        },
    )
    .map(|todos| {
        todos
            .into_iter()
            .filter(|t| t.due_date.as_deref() == Some(today.as_str()))
            .collect()
    })
}

pub fn save_attachment_record(
    conn: &Connection,
    todo_id: i64,
    filename: &str,
    original_name: Option<&str>,
    mime_type: &str,
    file_size: i64,
    kind: &str,
) -> AppResult<AttachmentInfo> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO attachments (todo_id, filename, original_name, mime_type, file_size, kind, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![todo_id, filename, original_name, mime_type, file_size, kind, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(AttachmentInfo {
        id,
        todo_id,
        filename: filename.into(),
        original_name: original_name.map(|s| s.into()),
        mime_type: Some(mime_type.into()),
        file_size,
        kind: kind.into(),
        url: format!("local://attachment/{todo_id}/{filename}"),
        created_at: now,
    })
}

pub fn list_attachments(conn: &Connection, todo_id: i64) -> AppResult<Vec<AttachmentInfo>> {
    let mut stmt = conn.prepare(
        "SELECT id, todo_id, filename, original_name, mime_type, file_size, kind, created_at
         FROM attachments WHERE todo_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map([todo_id], |row| {
        let todo_id: i64 = row.get(1)?;
        let filename: String = row.get(2)?;
        Ok(AttachmentInfo {
            id: row.get(0)?,
            todo_id,
            filename: filename.clone(),
            original_name: row.get(3)?,
            mime_type: row.get(4)?,
            file_size: row.get(5)?,
            kind: row.get(6)?,
            url: format!("local://attachment/{todo_id}/{filename}"),
            created_at: row.get(7)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_attachment(conn: &Connection, id: i64) -> AppResult<AttachmentInfo> {
    conn.query_row(
        "SELECT id, todo_id, filename, original_name, mime_type, file_size, kind, created_at
         FROM attachments WHERE id = ?1",
        [id],
        |row| {
            let todo_id: i64 = row.get(1)?;
            let filename: String = row.get(2)?;
            Ok(AttachmentInfo {
                id: row.get(0)?,
                todo_id,
                filename: filename.clone(),
                original_name: row.get(3)?,
                mime_type: row.get(4)?,
                file_size: row.get(5)?,
                kind: row.get(6)?,
                url: format!("local://attachment/{todo_id}/{filename}"),
                created_at: row.get(7)?,
            })
        },
    )
    .map_err(Into::into)
}

pub fn delete_attachment_record(conn: &Connection, id: i64) -> AppResult<AttachmentInfo> {
    let info = get_attachment(conn, id)?;
    conn.execute("DELETE FROM attachments WHERE id = ?1", [id])?;
    Ok(info)
}

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAttachmentMeta {
    pub filename: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: i64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTodoItem {
    #[serde(flatten)]
    pub detail: TodoDetail,
    pub attachments: Vec<ExportAttachmentMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataExportSnapshot {
    #[serde(default = "default_export_version")]
    pub version: i32,
    #[serde(default)]
    pub exported_at: String,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub kanban_columns: Vec<KanbanColumn>,
    #[serde(default)]
    pub todos: Vec<ExportTodoItem>,
    #[serde(default)]
    pub settings: std::collections::HashMap<String, String>,
}

fn default_export_version() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataImportResult {
    pub categories_created: i32,
    pub categories_reused: i32,
    pub tags_created: i32,
    pub tags_reused: i32,
    pub kanban_columns_created: i32,
    pub kanban_columns_reused: i32,
    pub todos_imported: i32,
    pub todos_skipped: i32,
}

pub fn export_all_data(conn: &Connection) -> AppResult<DataExportSnapshot> {
    let categories = list_categories(conn)?;
    let tags = list_tags(conn)?;
    let kanban_columns = list_kanban_columns(conn)?;
    let settings = get_all_settings(conn)?;
    let summaries = list_todos(
        conn,
        TodoListFilter {
            category_id: None,
            tag_ids: None,
            completed: None,
            priority: None,
            include_deleted: Some(false),
            search_query: None,
        },
    )?;

    let mut todos = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let detail = get_todo(conn, summary.id)?;
        let attachments = list_attachments(conn, summary.id)?
            .into_iter()
            .map(|item| ExportAttachmentMeta {
                filename: item.filename,
                original_name: item.original_name,
                mime_type: item.mime_type,
                file_size: item.file_size,
                kind: item.kind,
            })
            .collect();
        todos.push(ExportTodoItem {
            detail,
            attachments,
        });
    }

    Ok(DataExportSnapshot {
        version: 1,
        exported_at: now_iso(),
        categories,
        tags,
        kanban_columns,
        todos,
        settings,
    })
}

pub fn import_all_data(conn: &Connection, snapshot: DataExportSnapshot) -> AppResult<DataImportResult> {
    if snapshot.version < 1 {
        return Err(AppError::msg("不支持的 JSON 版本"));
    }

    let mut result = DataImportResult {
        categories_created: 0,
        categories_reused: 0,
        tags_created: 0,
        tags_reused: 0,
        kanban_columns_created: 0,
        kanban_columns_reused: 0,
        todos_imported: 0,
        todos_skipped: 0,
    };

    let existing_categories = list_categories(conn)?;
    let mut category_map: HashMap<i64, i64> = HashMap::new();
    for category in &snapshot.categories {
        if let Some(existing) = existing_categories
            .iter()
            .find(|item| item.name == category.name)
        {
            category_map.insert(category.id, existing.id);
            result.categories_reused += 1;
        } else {
            let created = create_category(
                conn,
                CreateCategoryInput {
                    name: category.name.clone(),
                    color: Some(category.color.clone()),
                    icon: category.icon.clone(),
                },
            )?;
            category_map.insert(category.id, created.id);
            result.categories_created += 1;
        }
    }

    let existing_tags = list_tags(conn)?;
    let mut tag_map: HashMap<i64, i64> = HashMap::new();
    for tag in &snapshot.tags {
        if let Some(existing) = existing_tags.iter().find(|item| item.name == tag.name) {
            tag_map.insert(tag.id, existing.id);
            result.tags_reused += 1;
        } else {
            let created = create_tag(
                conn,
                CreateTagInput {
                    name: tag.name.clone(),
                    color: Some(tag.color.clone()),
                },
            )?;
            tag_map.insert(tag.id, created.id);
            result.tags_created += 1;
        }
    }

    let existing_kanban_columns = list_kanban_columns(conn)?;
    let mut kanban_map: HashMap<i64, i64> = HashMap::new();
    for column in &snapshot.kanban_columns {
        if let Some(existing) = existing_kanban_columns
            .iter()
            .find(|item| item.name == column.name)
        {
            kanban_map.insert(column.id, existing.id);
            result.kanban_columns_reused += 1;
        } else {
            let created = create_kanban_column(
                conn,
                CreateKanbanColumnInput {
                    name: column.name.clone(),
                    color: Some(column.color.clone()),
                    subtitle: column.subtitle.clone(),
                },
            )?;
            kanban_map.insert(column.id, created.id);
            result.kanban_columns_created += 1;
        }
    }

    let existing_todos = list_todos(
        conn,
        TodoListFilter {
            category_id: None,
            tag_ids: None,
            completed: None,
            priority: None,
            include_deleted: Some(false),
            search_query: None,
        },
    )?;
    let mut existing_keys: HashSet<(String, Option<i64>)> = existing_todos
        .iter()
        .map(|todo| (todo.title.trim().to_string(), todo.category_id))
        .collect();

    for item in &snapshot.todos {
        let summary = &item.detail.summary;
        let category_id = summary
            .category_id
            .and_then(|id| category_map.get(&id).copied());
        let dedupe_key = (summary.title.trim().to_string(), category_id);
        if existing_keys.contains(&dedupe_key) {
            result.todos_skipped += 1;
            continue;
        }

        let tag_ids: Vec<i64> = summary
            .tag_ids
            .iter()
            .filter_map(|id| tag_map.get(id).copied())
            .collect();
        let created = create_todo(
            conn,
            CreateTodoInput {
                title: summary.title.clone(),
                category_id,
                tag_ids: Some(tag_ids.clone()),
                priority: Some(summary.priority.clone()),
                due_date: summary.due_date.clone(),
                content_html: Some(item.detail.content_html.clone()),
            },
        )?;

        let kanban_column_id = summary
            .kanban_column_id
            .and_then(|id| kanban_map.get(&id).copied());
        update_todo(
            conn,
            UpdateTodoInput {
                id: created.summary.id,
                title: summary.title.clone(),
                content_html: item.detail.content_html.clone(),
                completed: summary.completed,
                priority: summary.priority.clone(),
                start_date: summary.start_date.clone(),
                due_date: summary.due_date.clone(),
                category_id,
                tag_ids,
                sort_order: Some(summary.sort_order),
                pinned: Some(summary.pinned),
                assignee: Some(summary.assignee.clone()),
                kanban_column_id,
                quiet: Some(true),
                recurrence_json: summary.recurrence_json.clone(),
            },
        )?;

        if !item.detail.subtasks.is_empty() {
            import_subtasks_for_todo(conn, created.summary.id, &item.detail.subtasks)?;
        }

        existing_keys.insert(dedupe_key);
        result.todos_imported += 1;
    }

    ensure_search_index(conn)?;
    Ok(result)
}

pub fn due_notification_candidates(conn: &Connection) -> AppResult<Vec<TodoSummary>> {
    list_todos(
        conn,
        TodoListFilter {
            category_id: None,
            tag_ids: None,
            completed: Some(false),
            priority: None,
            include_deleted: Some(false),
            search_query: None,
        },
    )
    .map(|todos| {
        todos
            .into_iter()
            .filter(|todo| {
                todo.due_date.is_some()
                    && !todo
                        .recurrence_json
                        .as_ref()
                        .map(|config| config.enabled)
                        .unwrap_or(false)
            })
            .collect()
    })
}

pub fn recurrence_notification_candidates(conn: &Connection) -> AppResult<Vec<TodoSummary>> {
    list_todos(
        conn,
        TodoListFilter {
            category_id: None,
            tag_ids: None,
            completed: Some(false),
            priority: None,
            include_deleted: Some(false),
            search_query: None,
        },
    )
    .map(|todos| {
        todos
            .into_iter()
            .filter(|todo| {
                todo.recurrence_json
                    .as_ref()
                    .map(|config| config.enabled)
                    .unwrap_or(false)
            })
            .collect()
    })
}

pub fn count_active_todos(conn: &Connection) -> AppResult<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn count_attachments(conn: &Connection) -> AppResult<i32> {
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM attachments", [], |row| row.get(0))?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn.execute_batch(include_str!("../migrations/001_init.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/003_fts.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/005_assignee.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/006_fts_chinese.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/007_fix_fts_porter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/008_quadrant.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/009_kanban_columns.sql"))
            .unwrap();
        ensure_search_index(&conn).unwrap();
        conn
    }

    #[test]
    #[ignore = "manual inspection against local app database"]
    fn inspect_local_database() {
        let path = std::env::var("TODO_DB_PATH")
            .unwrap_or_else(|_| r"C:\Users\TX\AppData\Roaming\com.tx.todo-list\todos.db".into());
        let conn = Connection::open(&path).unwrap();
        let migrations: Vec<i32> = conn
            .prepare("SELECT version FROM schema_migrations ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        println!("migrations: {migrations:?}");
        let has_title_text: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('todos') WHERE name = 'title_text'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        println!("has_title_text: {has_title_text}");
        if has_title_text > 0 {
            ensure_search_index(&conn).unwrap();
            let rows = list_todos(
                &conn,
                TodoListFilter {
                    category_id: None,
                    tag_ids: None,
                    completed: None,
                    priority: None,
                    include_deleted: Some(false),
                    search_query: Some("测试".into()),
                },
            )
            .unwrap();
            println!("search 测试 -> {} rows", rows.len());
            if let Some(item) = rows.first() {
                update_todo(
                    &conn,
                    UpdateTodoInput {
                        id: item.id,
                        title: item.title.clone(),
                        content_html: String::new(),
                        completed: item.completed,
                        priority: item.priority.clone(),
                        start_date: item.start_date.clone(),
                        due_date: item.due_date.clone(),
                        category_id: item.category_id,
                        tag_ids: item.tag_ids.clone(),
                        sort_order: None,
                        pinned: None,
                        assignee: Some(item.assignee.clone()),
                        kanban_column_id: item.kanban_column_id,
                        quiet: None,
                    },
                )
                .unwrap();
                println!("update_todo ok for id {}", item.id);
            }
        }
    }

    #[test]
    fn update_todo_persists_with_fts_columns() {
        let conn = setup_db();
        let detail = create_todo(
            &conn,
            CreateTodoInput {
                title: "测试一下".into(),
                category_id: None,
                tag_ids: None,
                priority: None,
                due_date: None,
                content_html: Some("<p>中文内容</p>".into()),
            },
        )
        .unwrap();

        update_todo(
            &conn,
            UpdateTodoInput {
                id: detail.summary.id,
                title: "测试保存".into(),
                content_html: "<p>更新后的内容</p>".into(),
                completed: false,
                priority: "medium".into(),
                start_date: None,
                due_date: None,
                category_id: None,
                tag_ids: vec![],
                sort_order: None,
                pinned: None,
                assignee: Some("自己".into()),
                kanban_column_id: None,
                quiet: None,
            },
        )
        .unwrap();

        let rows = list_todos(
            &conn,
            TodoListFilter {
                category_id: None,
                tag_ids: None,
                completed: None,
                priority: None,
                include_deleted: Some(false),
                search_query: Some("测试".into()),
            },
        )
        .unwrap();

        assert!(rows.iter().any(|item| item.id == detail.summary.id));
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> AppResult<std::collections::HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
    Ok(rows.collect::<Result<std::collections::HashMap<_, _>, _>>()?)
}
