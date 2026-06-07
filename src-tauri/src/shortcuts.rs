use crate::db::{self, repositories};
use crate::error::{AppError, AppResult};
use crate::AppState;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub const QUICK_CAPTURE_SETTING_KEY: &str = "shortcut.quick_capture";
pub const TOGGLE_MAIN_SETTING_KEY: &str = "shortcut.toggle_main";
const LEGACY_MINIMIZE_TO_TRAY_SETTING_KEY: &str = "shortcut.minimize_to_tray";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBinding {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub code: String,
}

impl ShortcutBinding {
    pub fn quick_capture_default() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            meta: false,
            code: "KeyN".into(),
        }
    }

    pub fn toggle_main_default() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            meta: false,
            code: "KeyH".into(),
        }
    }
}

impl Default for ShortcutBinding {
    fn default() -> Self {
        Self::quick_capture_default()
    }
}

fn load_binding(key: &str, default: ShortcutBinding) -> AppResult<ShortcutBinding> {
    let saved = db::with_conn(|conn| repositories::get_setting(conn, key))?;
    let Some(raw) = saved else {
        return Ok(default);
    };
    let binding: ShortcutBinding = serde_json::from_str(&raw)?;
    Ok(binding)
}

fn save_binding(key: &str, binding: &ShortcutBinding) -> AppResult<()> {
    let raw = serde_json::to_string(binding)?;
    db::with_conn(|conn| repositories::set_setting(conn, key, &raw))
}

pub fn load_quick_capture_binding() -> AppResult<ShortcutBinding> {
    load_binding(QUICK_CAPTURE_SETTING_KEY, ShortcutBinding::quick_capture_default())
}

pub fn load_toggle_main_binding() -> AppResult<ShortcutBinding> {
    let default = ShortcutBinding::toggle_main_default();
    let saved = db::with_conn(|conn| repositories::get_setting(conn, TOGGLE_MAIN_SETTING_KEY))?;
    if saved.is_some() {
        return load_binding(TOGGLE_MAIN_SETTING_KEY, default);
    }
    load_binding(LEGACY_MINIMIZE_TO_TRAY_SETTING_KEY, default)
}

pub fn save_quick_capture_binding(binding: &ShortcutBinding) -> AppResult<()> {
    save_binding(QUICK_CAPTURE_SETTING_KEY, binding)
}

pub fn save_toggle_main_binding(binding: &ShortcutBinding) -> AppResult<()> {
    save_binding(TOGGLE_MAIN_SETTING_KEY, binding)
}

fn build_modifiers(binding: &ShortcutBinding) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    if binding.ctrl {
        modifiers |= Modifiers::CONTROL;
    }
    if binding.shift {
        modifiers |= Modifiers::SHIFT;
    }
    if binding.alt {
        modifiers |= Modifiers::ALT;
    }
    if binding.meta {
        modifiers |= Modifiers::SUPER;
    }
    modifiers
}

fn parse_code(code: &str) -> AppResult<Code> {
    if let Some(letter) = code.strip_prefix("Key") {
        if letter.len() == 1 {
            return letter_to_code(letter.chars().next().unwrap());
        }
    }

    if let Some(digit) = code.strip_prefix("Digit") {
        return match digit {
            "0" => Ok(Code::Digit0),
            "1" => Ok(Code::Digit1),
            "2" => Ok(Code::Digit2),
            "3" => Ok(Code::Digit3),
            "4" => Ok(Code::Digit4),
            "5" => Ok(Code::Digit5),
            "6" => Ok(Code::Digit6),
            "7" => Ok(Code::Digit7),
            "8" => Ok(Code::Digit8),
            "9" => Ok(Code::Digit9),
            _ => Err(AppError::msg(format!("不支持的按键: {code}"))),
        };
    }

    match code {
        "Space" => Ok(Code::Space),
        "Enter" => Ok(Code::Enter),
        "Escape" => Ok(Code::Escape),
        "Tab" => Ok(Code::Tab),
        "Backspace" => Ok(Code::Backspace),
        "Delete" => Ok(Code::Delete),
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        _ => Err(AppError::msg(format!("不支持的按键: {code}"))),
    }
}

fn letter_to_code(letter: char) -> AppResult<Code> {
    Ok(match letter.to_ascii_uppercase() {
        'A' => Code::KeyA,
        'B' => Code::KeyB,
        'C' => Code::KeyC,
        'D' => Code::KeyD,
        'E' => Code::KeyE,
        'F' => Code::KeyF,
        'G' => Code::KeyG,
        'H' => Code::KeyH,
        'I' => Code::KeyI,
        'J' => Code::KeyJ,
        'K' => Code::KeyK,
        'L' => Code::KeyL,
        'M' => Code::KeyM,
        'N' => Code::KeyN,
        'O' => Code::KeyO,
        'P' => Code::KeyP,
        'Q' => Code::KeyQ,
        'R' => Code::KeyR,
        'S' => Code::KeyS,
        'T' => Code::KeyT,
        'U' => Code::KeyU,
        'V' => Code::KeyV,
        'W' => Code::KeyW,
        'X' => Code::KeyX,
        'Y' => Code::KeyY,
        'Z' => Code::KeyZ,
        _ => return Err(AppError::msg(format!("不支持的按键: {letter}"))),
    })
}

pub fn binding_to_shortcut(binding: &ShortcutBinding) -> AppResult<Shortcut> {
    if !binding.ctrl && !binding.shift && !binding.alt && !binding.meta {
        return Err(AppError::msg("请至少包含一个修饰键（Ctrl / Shift / Alt / Win）"));
    }
    let modifiers = build_modifiers(binding);
    let code = parse_code(&binding.code)?;
    Ok(Shortcut::new(Some(modifiers), code))
}

fn register_shortcut<F>(
    app: &AppHandle,
    slot: &Mutex<Option<Shortcut>>,
    binding: &ShortcutBinding,
    on_pressed: F,
) -> AppResult<()>
where
    F: Fn(&AppHandle) + Send + Sync + 'static,
{
    let shortcut = binding_to_shortcut(binding)?;
    let mut current = slot.lock();
    if let Some(old) = current.take() {
        let _ = app.global_shortcut().unregister(old);
    }

    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                on_pressed(app);
            }
        })
        .map_err(|e| AppError::msg(e.to_string()))?;

    *current = Some(shortcut);
    Ok(())
}

pub fn register_quick_capture(
    app: &AppHandle,
    state: &AppState,
    binding: &ShortcutBinding,
) -> AppResult<()> {
    register_shortcut(app, &state.quick_capture_shortcut, binding, |app| {
        crate::show_quick_capture(app);
    })
}

pub fn register_toggle_main(
    app: &AppHandle,
    state: &AppState,
    binding: &ShortcutBinding,
) -> AppResult<()> {
    register_shortcut(app, &state.toggle_main_shortcut, binding, |app| {
        crate::toggle_main_minimal_window(app);
    })
}

pub fn init_global_shortcuts(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    register_quick_capture(app, &state, &load_quick_capture_binding()?)?;
    register_toggle_main(app, &state, &load_toggle_main_binding()?)?;
    Ok(())
}
