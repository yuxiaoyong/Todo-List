mod commands;
mod data;
pub mod db;
pub mod demo_seed;
mod lunar;
mod recurrence;
mod email_gateway;
mod error;
mod file_store;
mod minimal_dock;
mod notifications;
mod shortcuts;
mod window_opacity;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, LogicalSize, Listener, Manager, RunEvent, Size, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

const QUICK_CAPTURE_WIDTH: f64 = 520.0;
const QUICK_CAPTURE_HEIGHT: f64 = 130.0;
const MINIMAL_TODO_WIDTH: f64 = 420.0;
const MINIMAL_TODO_HEIGHT: f64 = 680.0;
const TASK_DETAIL_WIDTH: f64 = 720.0;
const TASK_DETAIL_HEIGHT: f64 = 820.0;
use parking_lot::Mutex;
use tauri_plugin_global_shortcut::Shortcut;

pub struct AppState {
    pub quick_capture_window: Option<tauri::WebviewWindow>,
    pub quick_capture_shortcut: Mutex<Option<Shortcut>>,
    pub toggle_main_shortcut: Mutex<Option<Shortcut>>,
    pub pending_notification_opens: Mutex<Vec<(i64, std::time::Instant)>>,
}

fn refresh_tray_count(app: &tauri::AppHandle) {
    let count = db::with_conn(db::repositories::incomplete_count).unwrap_or(0);
    let title = if count > 0 {
        format!("Todo List ({count})")
    } else {
        "Todo List".into()
    };
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(&title));
    }
}

fn focus_quick_capture(window: &tauri::WebviewWindow) {
    let _ = window.set_size(Size::Logical(LogicalSize::new(
        QUICK_CAPTURE_WIDTH,
        QUICK_CAPTURE_HEIGHT,
    )));
    let _ = window.show();
    let _ = window.set_focus();
    window_opacity::reapply_saved_to_window(window);
}

pub(crate) fn show_quick_capture(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("quick-capture") {
        focus_quick_capture(&window);
        return;
    }
    let _window = WebviewWindowBuilder::new(
        app,
        "quick-capture",
        WebviewUrl::App("index.html#/quick-capture".into()),
    )
    .title("快速添加")
    .inner_size(QUICK_CAPTURE_WIDTH, QUICK_CAPTURE_HEIGHT)
    .resizable(false)
    .always_on_top(true)
    .decorations(true)
    .build();
    if let Some(window) = app.get_webview_window("quick-capture") {
        focus_quick_capture(&window);
    }
}

fn set_window_skip_taskbar(window: &tauri::WebviewWindow, skip: bool) {
    let _ = window.set_skip_taskbar(skip);
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(minimal) = app.get_webview_window("minimal-todo") {
        set_window_skip_taskbar(&minimal, true);
        let _ = minimal.hide();
    }
    if let Some(window) = app.get_webview_window("main") {
        set_window_skip_taskbar(&window, false);
        let _ = window.show();
        let _ = window.set_focus();
        window_opacity::reapply_saved_to_window(&window);
    }
}

fn configure_minimal_todo_window(window: &tauri::WebviewWindow) {
    let _ = window.set_decorations(false);
    set_window_skip_taskbar(window, true);
}

fn focus_minimal_todo(window: &tauri::WebviewWindow) {
    configure_minimal_todo_window(window);
    let _ = window.show();
    minimal_dock::restore_docked_layout(window);
    minimal_dock::restore_if_hidden(window);
    let _ = window.set_focus();
    window_opacity::reapply_saved_to_window(window);
}

pub(crate) fn show_minimal_todo(app: &tauri::AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        set_window_skip_taskbar(&main, true);
        let _ = main.hide();
    }
    if let Some(window) = app.get_webview_window("minimal-todo") {
        focus_minimal_todo(&window);
        return;
    }
    let _window = WebviewWindowBuilder::new(
        app,
        "minimal-todo",
        WebviewUrl::App("index.html#/minimal".into()),
    )
    .title("Todo List")
    .inner_size(MINIMAL_TODO_WIDTH, MINIMAL_TODO_HEIGHT)
    .min_inner_size(320.0, 400.0)
    .resizable(true)
    .center()
    .decorations(false)
    .skip_taskbar(true)
    .build();
    if let Some(window) = app.get_webview_window("minimal-todo") {
        focus_minimal_todo(&window);
    }
}

pub(crate) fn open_task_detail_window(app: &tauri::AppHandle, id: i64) {
    let label = "task-detail";
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.emit("task-detail-navigate", id);
        let _ = window.show();
        let _ = window.set_focus();
        window_opacity::reapply_saved_to_window(&window);
        return;
    }

    let url = format!("index.html#/task-detail/{id}");
    match WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title("任务详情")
        .inner_size(TASK_DETAIL_WIDTH, TASK_DETAIL_HEIGHT)
        .min_inner_size(520.0, 480.0)
        .resizable(true)
        .center()
        .decorations(true)
        .build()
    {
        Ok(window) => {
            let _ = window.show();
            let _ = window.set_focus();
            window_opacity::reapply_saved_to_window(&window);
        }
        Err(err) => eprintln!("open task detail window failed: {err}"),
    }
}

pub(crate) fn toggle_main_minimal_window(app: &tauri::AppHandle) {
    let main = app.get_webview_window("main");
    let minimal = app.get_webview_window("minimal-todo");

    let main_visible = main
        .as_ref()
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let minimal_visible = minimal
        .as_ref()
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);

    if main_visible {
        show_minimal_todo(app);
        return;
    }
    if minimal_visible {
        show_main_window(app);
        return;
    }
    show_main_window(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    notifications::pre_register_windows();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !notifications::try_handle_notification_launch(app, &args) {
                show_main_window(app);
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_denylist(&["task-detail", "quick-capture", "minimal-todo"])
                .build(),
        )
        .manage(AppState {
            quick_capture_window: None,
            quick_capture_shortcut: Mutex::new(None),
            toggle_main_shortcut: Mutex::new(None),
            pending_notification_opens: Mutex::new(Vec::new()),
        })
        .manage(minimal_dock::MinimalDockState::new())
        .setup(|app| {
            db::init(app.handle())?;
            notifications::init_platform(app.handle());

            let quick_item = MenuItem::with_id(app, "quick-add", "快速添加任务", true, None::<&str>)?;
            let show_item = MenuItem::with_id(app, "show-main", "显示主窗口", true, None::<&str>)?;
            let quit_item = PredefinedMenuItem::quit(app, Some("退出"))?;
            let tray_menu = Menu::with_items(app, &[&quick_item, &show_item, &quit_item])?;

            let icon = app.default_window_icon().cloned();
            let mut tray_builder = TrayIconBuilder::with_id("main-tray").menu(&tray_menu);
            if let Some(icon) = icon {
                tray_builder = tray_builder.icon(icon);
            }
            tray_builder
                .tooltip("Todo List")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quick-add" => show_quick_capture(app),
                    "show-main" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            refresh_tray_count(app.handle());

            shortcuts::init_global_shortcuts(app.handle())?;

            let app_handle = app.handle().clone();
            app.handle().listen("todo-changed", move |_event| {
                refresh_tray_count(&app_handle);
            });

            notifications::start_scheduler(app.handle().clone());

            if let Some(main) = app.get_webview_window("main") {
                window_opacity::reapply_saved_to_window(&main);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::category_list,
            commands::category_create,
            commands::category_update,
            commands::category_delete,
            commands::category_reorder,
            commands::kanban_column_list,
            commands::kanban_column_create,
            commands::kanban_column_update,
            commands::kanban_column_delete,
            commands::kanban_column_reorder,
            commands::tag_list,
            commands::tag_create,
            commands::tag_update,
            commands::tag_delete,
            commands::todo_list,
            commands::todo_get,
            commands::todo_create,
            commands::todo_update,
            commands::todo_quick_create,
            commands::todo_toggle_complete,
            commands::todo_toggle_pin,
            commands::subtask_create,
            commands::subtask_update,
            commands::subtask_toggle,
            commands::subtask_delete,
            commands::todo_delete,
            commands::todo_restore,
            commands::todo_permanent_delete,
            commands::todo_empty_trash,
            commands::todo_reorder,
            commands::todo_reorder_positions,
            commands::todo_set_kanban_column,
            commands::todo_incomplete_count,
            commands::todo_due_today,
            commands::attachment_save,
            commands::attachment_delete,
            commands::attachment_list,
            commands::attachment_read,
            commands::attachment_get_path,
            commands::attachment_open,
            commands::settings_get,
            commands::settings_set,
            commands::settings_get_all,
            commands::shortcut_get_quick_capture,
            commands::shortcut_set_quick_capture,
            commands::shortcut_get_toggle_main,
            commands::shortcut_set_toggle_main,
            commands::show_quick_capture,
            commands::window_show_main,
            commands::window_open_task_detail,
            commands::window_set_opacity,
            commands::minimal_dock_on_blur,
            commands::data_get_info,
            commands::data_open_app_data_dir,
            commands::data_create_backup,
            commands::data_restore_backup,
            commands::data_export_json,
            commands::data_import_json,
            commands::data_reset_demo,
            commands::email_gateway_get_config,
            commands::email_gateway_save_config,
            commands::email_gateway_send_test,
        ])
        .on_window_event(|window, event| {
            if window.label() == "minimal-todo" {
                minimal_dock::handle_event(window.app_handle(), event);
            }

            match event {
                WindowEvent::CloseRequested { api, .. }
                    if window.label() == "main" || window.label() == "minimal-todo" =>
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
                WindowEvent::Focused(true) if window.label() == "main" => {
                    notifications::try_emit_open_task_on_focus(window.app_handle());
                    if let Some(main) = window.app_handle().get_webview_window("main") {
                        window_opacity::reapply_saved_to_window(&main);
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Ready = event {
                refresh_tray_count(app_handle);
                let _ = notifications::check_and_notify(app_handle);
            }
        });
}
