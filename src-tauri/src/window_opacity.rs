use tauri::{AppHandle, Manager, WebviewWindow};

use crate::db::repositories::get_setting;
use crate::db::with_conn;

pub const OPACITY_SETTING_KEY: &str = "windowOpacity";
const MIN_OPACITY: f64 = 0.5;
const MAX_OPACITY: f64 = 1.0;

const WINDOW_LABELS: &[&str] = &["main", "minimal-todo", "quick-capture", "task-detail"];

pub fn clamp_opacity(opacity: f64) -> f64 {
    opacity.clamp(MIN_OPACITY, MAX_OPACITY)
}

pub fn load_saved_opacity() -> f64 {
    with_conn(|conn| get_setting(conn, OPACITY_SETTING_KEY))
        .ok()
        .flatten()
        .and_then(|value| value.parse::<f64>().ok())
        .map(clamp_opacity)
        .unwrap_or(MAX_OPACITY)
}

pub fn apply_to_window(window: &WebviewWindow, opacity: f64) -> Result<(), String> {
    let opacity = clamp_opacity(opacity);
    #[cfg(target_os = "windows")]
    {
        let hwnd = window
            .hwnd()
            .map_err(|error| format!("get window handle failed: {error}"))?;
        apply_windows_opacity(hwnd.0 as isize, opacity);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        let _ = opacity;
    }
    Ok(())
}

pub fn apply_to_all_windows(app: &AppHandle, opacity: f64) {
    let opacity = clamp_opacity(opacity);
    for label in WINDOW_LABELS {
        if let Some(window) = app.get_webview_window(label) {
            let _ = apply_to_window(&window, opacity);
        }
    }
}

pub fn reapply_saved_to_window(window: &WebviewWindow) {
    let _ = apply_to_window(window, load_saved_opacity());
}

#[cfg(target_os = "windows")]
fn apply_windows_opacity(hwnd: isize, opacity: f64) {
    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_LAYERED: isize = 0x00080000;
    const LWA_ALPHA: u32 = 0x2;

    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if opacity >= MAX_OPACITY {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style & !WS_EX_LAYERED);
            return;
        }

        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED);
        let alpha = (opacity * 255.0).round().clamp(0.0, 255.0) as u8;
        let _ = SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
    }
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn GetWindowLongPtrW(hwnd: isize, nindex: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: isize, nindex: i32, dwnewlong: isize) -> isize;
    fn SetLayeredWindowAttributes(hwnd: isize, crkey: u32, balpha: u8, dwflags: u32) -> i32;
}
