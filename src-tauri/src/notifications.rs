use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::AppState;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};

use crate::db::repositories::{
    due_notification_candidates, get_setting, recurrence_notification_candidates, set_setting,
    TodoSummary,
};
use crate::recurrence::{recurrence_occurrence_key, should_notify_recurrence};
use crate::db::with_conn;
use crate::error::AppResult;

const SENT_LOG_KEY: &str = "notification.sent_log.v3";
const ENABLED_SETTING_KEY: &str = "notification.enabled";
const SYSTEM_SETTING_KEY: &str = "notification.system";
const EMAIL_SETTING_KEY: &str = "notification.email";
const ADVANCE_HOURS_SETTING_KEY: &str = "notification.advanceHours";
const REPEAT_MINUTES_SETTING_KEY: &str = "notification.repeatMinutes";
const CHECK_INTERVAL_SECS: u64 = 60;
const SENT_LOG_RETENTION_SECS: i64 = 30 * 24 * 60 * 60;
const PENDING_OPEN_TTL: Duration = Duration::from_secs(30);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DueReminderPayload {
    pub todo_id: i64,
    pub title: String,
}

pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(CHECK_INTERVAL_SECS));
        let _ = check_and_notify(&app);
    });
}

pub fn check_and_notify(app: &AppHandle) -> AppResult<()> {
    let (enabled, advance_hours, repeat_minutes) = with_conn(|conn| {
        Ok((
            setting_bool(conn, ENABLED_SETTING_KEY, true)?,
            setting_u32(conn, ADVANCE_HOURS_SETTING_KEY, 0)?,
            setting_u32(conn, REPEAT_MINUTES_SETTING_KEY, 1440)?,
        ))
    })?;
    if !enabled {
        return Ok(());
    }

    let now = Local::now();
    let due_todos = with_conn(due_notification_candidates)?;
    let recurrence_todos = with_conn(recurrence_notification_candidates)?;
    let active_ids: std::collections::HashSet<String> = due_todos
        .iter()
        .chain(recurrence_todos.iter())
        .map(|todo| todo.id.to_string())
        .collect();
    let mut sent_log = load_sent_log()?;
    prune_sent_log(&mut sent_log, now.timestamp(), &active_ids);

    let mut changed = false;
    for todo in due_todos {
        if !should_notify_todo(&todo, now, advance_hours) {
            continue;
        }
        if !should_repeat(&sent_log, todo.id, now.timestamp(), repeat_minutes) {
            continue;
        }
        if emit_due_reminder(app, &todo).is_ok() {
            sent_log.insert(todo.id.to_string(), now.timestamp().to_string());
            changed = true;
        }
    }
    for todo in recurrence_todos {
        if !should_notify_recurrence(&todo, now, advance_hours) {
            continue;
        }
        let Some(log_key) = recurrence_occurrence_key(&todo, now) else {
            continue;
        };
        if !should_repeat_key(&sent_log, &log_key, now.timestamp(), repeat_minutes) {
            continue;
        }
        if emit_due_reminder(app, &todo).is_ok() {
            sent_log.insert(log_key, now.timestamp().to_string());
            changed = true;
        }
    }
    if changed {
        save_sent_log(&sent_log)?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn pre_register_windows() {
    windows_notify::pre_register();
}

#[cfg(not(windows))]
pub fn pre_register_windows() {}

pub fn init_platform(app: &AppHandle) {
    #[cfg(windows)]
    windows_notify::init(app);
}

fn emit_due_reminder(app: &AppHandle, todo: &TodoSummary) -> AppResult<()> {
    mark_notification_pending(app, todo.id);
    let system_enabled = with_conn(|conn| setting_bool(conn, SYSTEM_SETTING_KEY, true))?;
    if system_enabled {
        let _ = show_due_system_notification(app, todo);
    }
    let payload = DueReminderPayload {
        todo_id: todo.id,
        title: todo.title.clone(),
    };
    app.emit("todo-due-reminder", payload)
        .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
    let email_enabled = with_conn(|conn| setting_bool(conn, EMAIL_SETTING_KEY, false))?;
    if email_enabled {
        let _ = crate::email_gateway::try_send_due_reminder(todo);
    }
    Ok(())
}

fn show_due_system_notification(app: &AppHandle, todo: &TodoSummary) -> AppResult<()> {
    #[cfg(windows)]
    {
        windows_notify::show_toast(app, todo.id, "任务到期提醒", &todo.title)?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title("任务到期提醒")
            .body(&todo.title)
            .show()
            .map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        Ok(())
    }
}

pub fn mark_notification_pending(app: &AppHandle, todo_id: i64) {
    let state = app.state::<AppState>();
    state
        .pending_notification_opens
        .lock()
        .push((todo_id, Instant::now()));
}

pub fn parse_notification_todo_id(arg: &str) -> Option<i64> {
    arg.split('&')
        .find_map(|part| part.strip_prefix("todo_id=").and_then(|id| id.parse().ok()))
        .or_else(|| {
            if arg.starts_with("todo_id=") {
                arg.strip_prefix("todo_id=")?.parse().ok()
            } else {
                None
            }
        })
}

pub fn try_emit_open_task_on_focus(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let mut pending = state.pending_notification_opens.lock();
    let now = Instant::now();
    pending.retain(|(_, at)| now.duration_since(*at) <= PENDING_OPEN_TTL);
    let Some((todo_id, _)) = pending.pop() else {
        return false;
    };
    drop(pending);

    emit_open_task(app, todo_id);
    true
}

pub fn try_handle_notification_launch(app: &AppHandle, args: &[String]) -> bool {
    if let Some(todo_id) = args.iter().find_map(|arg| parse_notification_todo_id(arg)) {
        emit_open_task(app, todo_id);
        return true;
    }
    try_emit_open_task_on_focus(app)
}

pub fn emit_open_task(app: &AppHandle, todo_id: i64) {
    let state = app.state::<AppState>();
    state
        .pending_notification_opens
        .lock()
        .retain(|(id, _)| *id != todo_id);
    crate::open_task_detail_window(app, todo_id);
}

const SENT_LOG_V2_KEY: &str = "notification.sent_log.v2";

fn load_sent_log() -> AppResult<HashMap<String, String>> {
    with_conn(|conn| {
        let raw = match get_setting(conn, SENT_LOG_KEY)? {
            Some(value) => Some(value),
            None => get_setting(conn, SENT_LOG_V2_KEY)?,
        };
        let mut log: HashMap<String, String> = match raw {
            Some(value) => serde_json::from_str(&value).unwrap_or_default(),
            None => HashMap::new(),
        };
        migrate_sent_log(&mut log);
        Ok(log)
    })
}

fn save_sent_log(log: &HashMap<String, String>) -> AppResult<()> {
    let raw = serde_json::to_string(log)?;
    with_conn(|conn| set_setting(conn, SENT_LOG_KEY, &raw))
}

fn migrate_sent_log(log: &mut HashMap<String, String>) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let now_ts = Local::now().timestamp().to_string();
    log.retain(|_, value| {
        if value.parse::<i64>().is_ok() {
            return true;
        }
        if value.len() == 10 && value.chars().nth(4) == Some('-') {
            if value == &today {
                *value = now_ts.clone();
                return true;
            }
            return false;
        }
        false
    });
}

fn should_repeat(
    log: &HashMap<String, String>,
    todo_id: i64,
    now_ts: i64,
    repeat_minutes: u32,
) -> bool {
    should_repeat_key(log, &todo_id.to_string(), now_ts, repeat_minutes)
}

fn should_repeat_key(
    log: &HashMap<String, String>,
    key: &str,
    now_ts: i64,
    repeat_minutes: u32,
) -> bool {
    let Some(raw) = log.get(key) else {
        return true;
    };
    let Some(last_ts) = raw.parse::<i64>().ok() else {
        return true;
    };
    let interval_secs = i64::from(repeat_minutes.max(1)) * 60;
    now_ts.saturating_sub(last_ts) >= interval_secs
}

fn prune_sent_log(
    log: &mut HashMap<String, String>,
    now_ts: i64,
    active_ids: &std::collections::HashSet<String>,
) {
    log.retain(|key, value| {
        let todo_id = key
            .strip_prefix("recurrence:")
            .and_then(|rest| rest.split(':').next())
            .unwrap_or(key.as_str());
        if !active_ids.contains(todo_id) {
            return false;
        }
        let Some(last_ts) = value.parse::<i64>().ok() else {
            return false;
        };
        now_ts.saturating_sub(last_ts) <= SENT_LOG_RETENTION_SECS
    });
}

fn setting_bool(
    conn: &rusqlite::Connection,
    key: &str,
    default: bool,
) -> AppResult<bool> {
    let value = get_setting(conn, key)?;
    Ok(parse_bool(value.as_deref(), default))
}

fn setting_u32(conn: &rusqlite::Connection, key: &str, default: u32) -> AppResult<u32> {
    let value = get_setting(conn, key)?;
    Ok(value
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(default))
}

fn parse_bool(value: Option<&str>, default: bool) -> bool {
    match value {
        Some("false") | Some("0") => false,
        Some("true") | Some("1") => true,
        _ => default,
    }
}

fn should_notify_todo(todo: &TodoSummary, now: DateTime<Local>, advance_hours: u32) -> bool {
    let Some(due_raw) = todo.due_date.as_deref() else {
        return false;
    };
    let Some(due_moment) = parse_due_moment(due_raw) else {
        return false;
    };
    let window_start = due_moment - chrono::Duration::hours(i64::from(advance_hours));
    now >= window_start
}

fn parse_due_moment(due: &str) -> Option<DateTime<Local>> {
    if due.len() > 10 {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(due) {
            return Some(parsed.with_timezone(&Local));
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(due, "%Y-%m-%d %H:%M:%S") {
            return Local.from_local_datetime(&naive).single();
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(due, "%Y-%m-%dT%H:%M:%S") {
            return Local.from_local_datetime(&naive).single();
        }
    }

    let day_part = due.get(..10).unwrap_or(due);
    let date = NaiveDate::parse_from_str(day_part, "%Y-%m-%d").ok()?;
    let naive = date.and_hms_opt(0, 0, 0)?;
    Local.from_local_datetime(&naive).single()
}

#[cfg(windows)]
mod windows_notify {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    use once_cell::sync::OnceCell;
    use tauri::AppHandle;
    use winrt_toast_reborn::{Toast, ToastManager};
    use windows::core::{Interface, PCWSTR, PROPVARIANT};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
        IPersistFile,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SetCurrentProcessExplicitAppUserModelID};
    use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;

    use crate::error::{AppError, AppResult};
    use crate::notifications::{emit_open_task, mark_notification_pending};

    static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();
    static TOAST_MANAGER: OnceCell<SharedToastManager> = OnceCell::new();

    struct SharedToastManager(ToastManager);
    unsafe impl Send for SharedToastManager {}
    unsafe impl Sync for SharedToastManager {}

    pub fn pre_register() {
        let app_id = app_id();
        let icon = resolve_icon_path();
        if let Err(e) = winrt_toast_reborn::register(app_id, display_name(), icon.as_deref()) {
            eprintln!("注册 Windows 通知失败: {e}");
        }
        if let Err(e) = create_start_menu_shortcut(app_id, display_name(), icon.as_deref()) {
            eprintln!("创建开始菜单快捷方式失败: {e}");
        }
        let _ = set_process_app_id(app_id);
    }

    pub fn init(app: &AppHandle) {
        let _ = APP_HANDLE.set(app.clone());
        let app_id = app.config().identifier.clone();
        let manager = ToastManager::new(&app_id)
            .on_activated(None, |action| {
                let Some(action) = action else {
                    return;
                };
                let Some(todo_id) = super::parse_notification_todo_id(&action.arg) else {
                    return;
                };
                let Some(app) = APP_HANDLE.get() else {
                    return;
                };
                mark_notification_pending(app, todo_id);
                emit_open_task(app, todo_id);
            })
            .on_failed(|failed| {
                eprintln!("Windows 通知显示失败: {:?}", failed.error);
            });
        let _ = TOAST_MANAGER.set(SharedToastManager(manager));
    }

    pub fn show_toast(
        app: &AppHandle,
        todo_id: i64,
        title: &str,
        body: &str,
    ) -> AppResult<()> {
        let title = title.to_string();
        let body = body.to_string();
        let (tx, rx) = std::sync::mpsc::sync_channel::<AppResult<()>>(1);

        app.run_on_main_thread(move || {
            let result = show_toast_on_main_thread(todo_id, &title, &body);
            let _ = tx.send(result);
        })
        .map_err(|e| AppError::msg(format!("调度系统通知失败: {e}")))?;

        rx.recv()
            .unwrap_or_else(|_| Err(AppError::msg("系统通知未执行")))
    }

    fn show_toast_on_main_thread(todo_id: i64, title: &str, body: &str) -> AppResult<()> {
        let manager = TOAST_MANAGER
            .get()
            .ok_or_else(|| AppError::msg("Windows 通知未初始化"))?;

        let mut toast = Toast::new();
        toast
            .text1(title)
            .text2(body)
            .launch(format!("todo_id={todo_id}"));

        manager
            .0
            .show(&toast)
            .map_err(|e| AppError::msg(format!("发送系统通知失败: {e}")))?;
        Ok(())
    }

    fn create_start_menu_shortcut(
        app_id: &str,
        display_name: &str,
        icon_path: Option<&Path>,
    ) -> AppResult<()> {
        let exe_path = std::env::current_exe()
            .map_err(|e| AppError::msg(format!("获取程序路径失败: {e}")))?;
        let shortcut_dir = start_menu_programs_dir()?;
        std::fs::create_dir_all(&shortcut_dir)
            .map_err(|e| AppError::msg(format!("创建开始菜单目录失败: {e}")))?;
        let shortcut_path = shortcut_dir.join(format!("{display_name}.lnk"));

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let shell_link: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| AppError::msg(format!("创建 ShellLink 失败: {e}")))?;

            let exe_wide = exe_path.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>();
            shell_link
                .SetPath(PCWSTR(exe_wide.as_ptr()))
                .map_err(|e| AppError::msg(format!("设置快捷方式路径失败: {e}")))?;

            let desc_wide = display_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            shell_link
                .SetDescription(PCWSTR(desc_wide.as_ptr()))
                .map_err(|e| AppError::msg(format!("设置快捷方式描述失败: {e}")))?;

            shell_link
                .SetShowCmd(SHOW_WINDOW_CMD(1))
                .map_err(|e| AppError::msg(format!("设置快捷方式显示模式失败: {e}")))?;

            if let Some(icon) = icon_path {
                let icon_wide = icon.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>();
                shell_link
                    .SetIconLocation(PCWSTR(icon_wide.as_ptr()), 0)
                    .map_err(|e| AppError::msg(format!("设置快捷方式图标失败: {e}")))?;
            }

            let property_store: IPropertyStore = shell_link
                .cast()
                .map_err(|e| AppError::msg(format!("获取 IPropertyStore 失败: {e}")))?;
            let propvar = PROPVARIANT::from(app_id);
            property_store
                .SetValue(&PKEY_AppUserModel_ID, &propvar)
                .map_err(|e| AppError::msg(format!("设置 AppUserModelID 失败: {e}")))?;
            property_store
                .Commit()
                .map_err(|e| AppError::msg(format!("提交快捷方式属性失败: {e}")))?;

            let persist: IPersistFile = shell_link
                .cast()
                .map_err(|e| AppError::msg(format!("获取 IPersistFile 失败: {e}")))?;
            let shortcut_wide = shortcut_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>();
            persist
                .Save(PCWSTR(shortcut_wide.as_ptr()), BOOL(1))
                .map_err(|e| AppError::msg(format!("保存快捷方式失败: {e}")))?;
        }

        Ok(())
    }

    fn start_menu_programs_dir() -> AppResult<PathBuf> {
        let appdata = std::env::var("APPDATA")
            .map_err(|e| AppError::msg(format!("读取 APPDATA 失败: {e}")))?;
        Ok(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"))
    }

    fn app_id() -> &'static str {
        "com.tx.todo-list"
    }

    fn display_name() -> &'static str {
        "Todo List"
    }

    fn resolve_icon_path() -> Option<PathBuf> {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let bundled = dir.join("resources").join("icon.ico");
                if bundled.exists() {
                    return bundled.canonicalize().ok();
                }
            }
        }

        let dev_icon = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/icon.ico");
        if dev_icon.exists() {
            return dev_icon.canonicalize().ok();
        }

        None
    }

    fn set_process_app_id(app_id: &str) -> AppResult<()> {
        let wide: Vec<u16> = OsStr::new(app_id).encode_wide().chain(Some(0)).collect();
        unsafe {
            SetCurrentProcessExplicitAppUserModelID(PCWSTR(wide.as_ptr()))
                .map_err(|e| AppError::msg(format!("设置 AppUserModelID 失败: {e}")))?;
        }
        Ok(())
    }

}
