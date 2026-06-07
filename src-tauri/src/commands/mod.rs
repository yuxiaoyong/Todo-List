use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use base64::Engine;
use crate::db::repositories::*;
use crate::db::with_conn;
use crate::error::AppResult;
use crate::file_store::{
    delete_attachment, delete_todo_attachments, get_attachment_path, open_attachment,
    read_attachment_bytes, save_attachment,
};
use crate::AppState;

#[tauri::command]
pub fn category_list() -> AppResult<Vec<Category>> {
    with_conn(list_categories)
}

#[tauri::command]
pub fn category_create(input: CreateCategoryInput) -> AppResult<Category> {
    with_conn(|conn| create_category(conn, input))
}

#[tauri::command]
pub fn category_update(input: UpdateCategoryInput) -> AppResult<Category> {
    with_conn(|conn| update_category(conn, input))
}

#[tauri::command]
pub fn category_delete(id: i64) -> AppResult<()> {
    with_conn(|conn| delete_category(conn, id))
}

#[tauri::command]
pub fn category_reorder(ids: Vec<i64>) -> AppResult<()> {
    with_conn(|conn| reorder_categories(conn, ids))
}

#[tauri::command]
pub fn kanban_column_list() -> AppResult<Vec<KanbanColumn>> {
    with_conn(list_kanban_columns)
}

#[tauri::command]
pub fn kanban_column_create(input: CreateKanbanColumnInput) -> AppResult<KanbanColumn> {
    with_conn(|conn| create_kanban_column(conn, input))
}

#[tauri::command]
pub fn kanban_column_update(input: UpdateKanbanColumnInput) -> AppResult<KanbanColumn> {
    with_conn(|conn| update_kanban_column(conn, input))
}

#[tauri::command]
pub fn kanban_column_delete(id: i64) -> AppResult<()> {
    with_conn(|conn| delete_kanban_column(conn, id))
}

#[tauri::command]
pub fn kanban_column_reorder(ids: Vec<i64>) -> AppResult<()> {
    with_conn(|conn| reorder_kanban_columns(conn, ids))
}

#[tauri::command]
pub fn tag_list() -> AppResult<Vec<Tag>> {
    with_conn(list_tags)
}

#[tauri::command]
pub fn tag_create(input: CreateTagInput) -> AppResult<Tag> {
    with_conn(|conn| create_tag(conn, input))
}

#[tauri::command]
pub fn tag_update(input: UpdateTagInput) -> AppResult<Tag> {
    with_conn(|conn| update_tag(conn, input))
}

#[tauri::command]
pub fn tag_delete(id: i64) -> AppResult<()> {
    with_conn(|conn| delete_tag(conn, id))
}

#[tauri::command]
pub fn todo_list(filter: TodoListFilter) -> AppResult<Vec<TodoSummary>> {
    with_conn(|conn| list_todos(conn, filter))
}

#[tauri::command]
pub fn todo_get(id: i64) -> AppResult<TodoDetail> {
    with_conn(|conn| get_todo(conn, id))
}

#[tauri::command]
pub fn todo_create(app: AppHandle, input: CreateTodoInput) -> AppResult<TodoDetail> {
    let detail = with_conn(|conn| create_todo(conn, input))?;
    let _ = app.emit("todo-changed", ());
    Ok(detail)
}

#[tauri::command]
pub fn todo_update(app: AppHandle, input: UpdateTodoInput) -> AppResult<TodoDetail> {
    let quiet = input.quiet.unwrap_or(false);
    let detail = with_conn(|conn| update_todo(conn, input))?;
    if !quiet {
        let _ = app.emit("todo-changed", ());
    }
    Ok(detail)
}

#[tauri::command]
pub fn todo_quick_create(app: AppHandle, input: QuickCreateInput) -> AppResult<TodoDetail> {
    let detail = with_conn(|conn| quick_create(conn, input))?;
    let _ = app.emit("todo-changed", ());
    Ok(detail)
}

#[tauri::command]
pub fn todo_toggle_complete(app: AppHandle, id: i64) -> AppResult<TodoDetail> {
    let detail = with_conn(|conn| toggle_complete(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(detail)
}

#[tauri::command]
pub fn todo_toggle_pin(app: AppHandle, id: i64) -> AppResult<TodoDetail> {
    let detail = with_conn(|conn| toggle_pin(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(detail)
}

#[tauri::command]
pub fn subtask_create(app: AppHandle, todo_id: i64, title: String) -> AppResult<Subtask> {
    let subtask = with_conn(|conn| create_subtask(conn, todo_id, title))?;
    let _ = app.emit("todo-changed", ());
    Ok(subtask)
}

#[tauri::command]
pub fn subtask_update(app: AppHandle, id: i64, title: String) -> AppResult<Subtask> {
    let subtask = with_conn(|conn| update_subtask(conn, id, title))?;
    let _ = app.emit("todo-changed", ());
    Ok(subtask)
}

#[tauri::command]
pub fn subtask_toggle(app: AppHandle, id: i64) -> AppResult<Subtask> {
    let subtask = with_conn(|conn| toggle_subtask(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(subtask)
}

#[tauri::command]
pub fn subtask_delete(app: AppHandle, id: i64) -> AppResult<()> {
    with_conn(|conn| delete_subtask(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(())
}

#[tauri::command]
pub fn todo_delete(app: AppHandle, id: i64) -> AppResult<()> {
    with_conn(|conn| soft_delete(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(())
}

#[tauri::command]
pub fn todo_restore(app: AppHandle, id: i64) -> AppResult<TodoDetail> {
    let detail = with_conn(|conn| restore_todo(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(detail)
}

#[tauri::command]
pub fn todo_permanent_delete(app: AppHandle, id: i64) -> AppResult<()> {
    delete_todo_attachments(&app, id)?;
    with_conn(|conn| permanent_delete(conn, id))?;
    let _ = app.emit("todo-changed", ());
    Ok(())
}

#[tauri::command]
pub fn todo_empty_trash(app: AppHandle) -> AppResult<u64> {
    let ids = with_conn(list_deleted_todo_ids)?;
    for id in ids {
        delete_todo_attachments(&app, id)?;
    }
    let count = with_conn(empty_trash)?;
    let _ = app.emit("todo-changed", ());
    Ok(count)
}

#[tauri::command]
pub fn todo_reorder(ids: Vec<i64>) -> AppResult<()> {
    with_conn(|conn| reorder_todos(conn, ids))
}

#[tauri::command]
pub fn todo_reorder_positions(app: AppHandle, items: Vec<TodoReorderItem>) -> AppResult<()> {
    with_conn(|conn| reorder_todos_positions(conn, items))?;
    let _ = app.emit("todo-changed", ());
    Ok(())
}

#[tauri::command]
pub fn todo_set_kanban_column(
    app: AppHandle,
    id: i64,
    kanban_column_id: Option<i64>,
) -> AppResult<TodoDetail> {
    let detail = with_conn(|conn| set_todo_kanban_column(conn, id, kanban_column_id))?;
    let _ = app.emit("todo-changed", ());
    Ok(detail)
}

#[tauri::command]
pub fn todo_incomplete_count() -> AppResult<i32> {
    with_conn(incomplete_count)
}

#[tauri::command]
pub fn todo_due_today() -> AppResult<Vec<TodoSummary>> {
    with_conn(due_today_todos)
}

#[tauri::command]
pub fn attachment_save(
    app: AppHandle,
    todo_id: i64,
    data_base64: String,
    original_name: Option<String>,
    mime_type: String,
    kind: Option<String>,
) -> AppResult<AttachmentInfo> {
    save_attachment(
        &app,
        todo_id,
        &data_base64,
        original_name.as_deref(),
        &mime_type,
        kind.as_deref().unwrap_or("attachment"),
    )
}

#[tauri::command]
pub fn attachment_delete(app: AppHandle, id: i64) -> AppResult<()> {
    delete_attachment(&app, id)
}

#[tauri::command]
pub fn attachment_list(todo_id: i64) -> AppResult<Vec<AttachmentInfo>> {
    with_conn(|conn| list_attachments(conn, todo_id))
}

#[tauri::command]
pub fn attachment_read(
    app: AppHandle,
    todo_id: i64,
    filename: String,
) -> AppResult<String> {
    let bytes = read_attachment_bytes(&app, todo_id, &filename)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[tauri::command]
pub fn attachment_get_path(
    app: AppHandle,
    todo_id: i64,
    filename: String,
) -> AppResult<String> {
    get_attachment_path(&app, todo_id, &filename)
}

#[tauri::command]
pub fn attachment_open(
    app: AppHandle,
    todo_id: i64,
    filename: String,
) -> AppResult<()> {
    open_attachment(&app, todo_id, &filename)
}

#[tauri::command]
pub fn settings_get(key: String) -> AppResult<Option<String>> {
    with_conn(|conn| get_setting(conn, &key))
}

#[tauri::command]
pub fn settings_set(key: String, value: String) -> AppResult<()> {
    with_conn(|conn| set_setting(conn, &key, &value))
}

#[tauri::command]
pub fn settings_get_all() -> AppResult<std::collections::HashMap<String, String>> {
    with_conn(get_all_settings)
}

#[tauri::command]
pub fn shortcut_get_quick_capture() -> AppResult<crate::shortcuts::ShortcutBinding> {
    crate::shortcuts::load_quick_capture_binding()
}

#[tauri::command]
pub fn shortcut_set_quick_capture(
    app: AppHandle,
    state: State<'_, AppState>,
    binding: crate::shortcuts::ShortcutBinding,
) -> AppResult<crate::shortcuts::ShortcutBinding> {
    crate::shortcuts::save_quick_capture_binding(&binding)?;
    crate::shortcuts::register_quick_capture(&app, &state, &binding)?;
    Ok(binding)
}

#[tauri::command]
pub fn shortcut_get_toggle_main() -> AppResult<crate::shortcuts::ShortcutBinding> {
    crate::shortcuts::load_toggle_main_binding()
}

#[tauri::command]
pub fn shortcut_set_toggle_main(
    app: AppHandle,
    state: State<'_, AppState>,
    binding: crate::shortcuts::ShortcutBinding,
) -> AppResult<crate::shortcuts::ShortcutBinding> {
    crate::shortcuts::save_toggle_main_binding(&binding)?;
    crate::shortcuts::register_toggle_main(&app, &state, &binding)?;
    Ok(binding)
}

#[tauri::command]
pub fn show_quick_capture(state: State<'_, AppState>) -> AppResult<()> {
    if let Some(window) = &state.quick_capture_window {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub fn window_show_main(app: AppHandle) -> AppResult<()> {
    crate::show_main_window(&app);
    Ok(())
}

#[tauri::command]
pub fn window_open_task_detail(app: AppHandle, id: i64) -> AppResult<()> {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::open_task_detail_window(&app, id);
    });
    Ok(())
}

#[tauri::command]
pub fn window_set_opacity(app: AppHandle, opacity: f64) -> AppResult<()> {
    crate::window_opacity::apply_to_all_windows(&app, opacity);
    Ok(())
}

#[tauri::command]
pub fn minimal_dock_on_blur(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("minimal-todo") {
        crate::minimal_dock::on_blur(&window);
    }
    Ok(())
}

#[tauri::command]
pub fn data_get_info(app: AppHandle) -> AppResult<crate::data::DataInfo> {
    crate::data::get_data_info(&app)
}

#[tauri::command]
pub fn data_open_app_data_dir(app: AppHandle) -> AppResult<()> {
    let dir = crate::db::app_data_dir(&app)?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn data_create_backup(app: AppHandle) -> AppResult<Option<String>> {
    let default_name = format!(
        "todo-list-backup-{}.zip",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let picked = rfd::AsyncFileDialog::new()
        .set_file_name(&default_name)
        .add_filter("Zip 备份", &["zip"])
        .save_file()
        .await;
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.path().to_path_buf();
    let saved = crate::data::create_backup(&app, &path)?;
    Ok(Some(saved))
}

#[tauri::command]
pub async fn data_restore_backup(app: AppHandle) -> AppResult<bool> {
    let picked = rfd::AsyncFileDialog::new()
        .add_filter("Zip 备份", &["zip"])
        .pick_file()
        .await;
    let Some(file) = picked else {
        return Ok(false);
    };
    crate::data::restore_backup(&app, file.path())?;
    let _ = app.emit("todo-changed", ());
    Ok(true)
}

#[tauri::command]
pub async fn data_export_json(app: AppHandle) -> AppResult<Option<String>> {
    let default_name = format!(
        "todo-list-export-{}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let picked = rfd::AsyncFileDialog::new()
        .set_file_name(&default_name)
        .add_filter("JSON 数据", &["json"])
        .save_file()
        .await;
    let Some(file) = picked else {
        return Ok(None);
    };
    let path: PathBuf = file.path().to_path_buf();
    let saved = crate::data::export_json(&app, &path)?;
    Ok(Some(saved))
}

#[tauri::command]
pub fn email_gateway_get_config() -> AppResult<crate::email_gateway::EmailGatewayPublicConfig> {
    crate::email_gateway::get_public_config()
}

#[tauri::command]
pub fn email_gateway_save_config(
    config: crate::email_gateway::EmailGatewaySaveInput,
) -> AppResult<crate::email_gateway::EmailGatewayPublicConfig> {
    crate::email_gateway::save_config(config)
}

#[tauri::command]
pub fn email_gateway_send_test() -> AppResult<()> {
    crate::email_gateway::send_test_email()
}

#[tauri::command]
pub async fn data_import_json(app: AppHandle) -> AppResult<Option<crate::db::repositories::DataImportResult>> {
    let picked = rfd::AsyncFileDialog::new()
        .add_filter("JSON 数据", &["json"])
        .pick_file()
        .await;
    let Some(file) = picked else {
        return Ok(None);
    };
    let result = crate::data::import_json(&app, file.path())?;
    let _ = app.emit("todo-changed", ());
    Ok(Some(result))
}
