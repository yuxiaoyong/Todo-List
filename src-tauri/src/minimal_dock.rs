use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent};

use crate::db::{self, repositories};

const SNAP_THRESHOLD_LOGICAL: f64 = 96.0;
const PEEK_WIDTH_LOGICAL: f64 = 10.0;
const MOVE_DEBOUNCE_MS: u64 = 280;
const BLUR_HIDE_DELAY_MS: u64 = 200;
const ANIMATION_DURATION_MS: u64 = 260;
const ANIMATION_STEPS: u32 = 20;
const SETTINGS_DOCK_SIDE: &str = "minimal.dock.side";
const SETTINGS_DOCK_Y: &str = "minimal.dock.y";
const WINDOWS_MINIMIZED_COORD: i32 = -31_000;
const RESTORE_REINFORCE_DELAY_MS: u64 = 80;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DockSide {
    None,
    Left,
    Right,
}

pub struct MinimalDockState {
    side: Mutex<DockSide>,
    hidden: Mutex<bool>,
    docked_y: Mutex<i32>,
    last_pos: Mutex<Option<PhysicalPosition<i32>>>,
    move_generation: AtomicU64,
    blur_generation: AtomicU64,
    anim_generation: AtomicU64,
    adjusting: Mutex<bool>,
}

impl MinimalDockState {
    pub fn new() -> Self {
        Self {
            side: Mutex::new(DockSide::None),
            hidden: Mutex::new(false),
            docked_y: Mutex::new(0),
            last_pos: Mutex::new(None),
            move_generation: AtomicU64::new(0),
            blur_generation: AtomicU64::new(0),
            anim_generation: AtomicU64::new(0),
            adjusting: Mutex::new(false),
        }
    }
}

pub fn handle_event(app: &AppHandle, event: &WindowEvent) {
    let Some(window) = app.get_webview_window("minimal-todo") else {
        return;
    };

    match event {
        WindowEvent::Moved(_) => schedule_snap(&window),
        WindowEvent::Focused(false) => on_blur(&window),
        WindowEvent::Focused(true) => {
            on_focus(&window);
        }
        _ => {}
    }
}

pub fn on_blur(window: &WebviewWindow) {
    schedule_blur_hide(window);
}

pub fn restore_if_hidden(window: &WebviewWindow) {
    let _ = expand_if_hidden(window);
}

pub fn restore_docked_layout(window: &WebviewWindow) {
    let _ = reposition_to_stored_dock(window, false);
}

fn on_focus(window: &WebviewWindow) {
    cancel_pending_snap(window);
    ensure_dock_state_loaded(window);
    let _ = expand_if_hidden(window);
    let _ = reposition_to_stored_dock(window, false);
    schedule_restore_reinforce(window);
}

fn schedule_snap(window: &WebviewWindow) {
    let state = window.state::<MinimalDockState>();
    if *state.adjusting.lock() || *state.hidden.lock() {
        return;
    }

    let generation = state.move_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let window = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(MOVE_DEBOUNCE_MS));
        if window.state::<MinimalDockState>().move_generation.load(Ordering::SeqCst) != generation {
            return;
        }
        let window_for_main = window.clone();
        let _ = window.app_handle().run_on_main_thread(move || {
            let _ = handle_moved(&window_for_main);
        });
    });
}

fn schedule_blur_hide(window: &WebviewWindow) {
    let state = window.state::<MinimalDockState>();
    if *state.adjusting.lock() || *state.hidden.lock() {
        return;
    }

    let generation = state.blur_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let window = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(BLUR_HIDE_DELAY_MS));
        let dock_state = window.state::<MinimalDockState>();
        if dock_state.blur_generation.load(Ordering::SeqCst) != generation {
            return;
        }
        let window_for_main = window.clone();
        let _ = window.app_handle().run_on_main_thread(move || {
            if is_window_active(&window_for_main) {
                return;
            }
            let _ = refresh_dock_side(&window_for_main, true);
            let _ = hide_to_edge(&window_for_main);
        });
    });
}

fn handle_moved(window: &WebviewWindow) -> Result<(), String> {
    let Some((work, scale, pos, size)) = window_geometry(window)? else {
        return Ok(());
    };

    if is_minimized_position(pos) {
        cancel_pending_snap(window);
        return Ok(());
    }

    let state = window.state::<MinimalDockState>();
    ensure_dock_state_loaded(window);
    let stored_side = *state.side.lock();
    let prev_pos = state.last_pos.lock().clone();
    *state.last_pos.lock() = Some(pos);

    let win_w = size.width as i32;
    if stored_side != DockSide::None && is_undocking(&work, scale, pos, win_w) {
        clear_dock_state(window);
        return Ok(());
    }

    if stored_side != DockSide::None
        && should_restore_stored_dock(stored_side, &work, scale, pos, &size, prev_pos)
    {
        cancel_pending_snap(window);
        return reposition_to_stored_dock(window, false);
    }

    snap_to_edge(window, &work, scale, pos, size, prev_pos)
}

fn is_undocking(
    work: &tauri::Monitor,
    scale: f64,
    pos: PhysicalPosition<i32>,
    win_w: i32,
) -> bool {
    detect_side(work, scale, pos, win_w) == DockSide::None
}

fn clear_dock_state(window: &WebviewWindow) {
    let state = window.state::<MinimalDockState>();
    *state.side.lock() = DockSide::None;
    *state.hidden.lock() = false;
    clear_persisted_dock_state(window);
}

fn should_restore_stored_dock(
    stored: DockSide,
    work: &tauri::Monitor,
    scale: f64,
    pos: PhysicalPosition<i32>,
    size: &tauri::PhysicalSize<u32>,
    prev_pos: Option<PhysicalPosition<i32>>,
) -> bool {
    let win_w = size.width as i32;
    let detected = detect_side(work, scale, pos, win_w);

    // User dragged away from edges or toward the opposite edge — don't snap back.
    if detected == DockSide::None || detected != stored {
        return false;
    }

    let work_w = work.work_area().size.width as i32;
    if prev_pos.is_some_and(|prev| is_suspicious_teleport(prev, pos, work_w)) {
        return true;
    }

    is_at_wrong_dock_x(work, scale, pos, win_w, stored)
}

fn snap_to_edge(
    window: &WebviewWindow,
    work: &tauri::Monitor,
    scale: f64,
    pos: PhysicalPosition<i32>,
    size: tauri::PhysicalSize<u32>,
    prev_pos: Option<PhysicalPosition<i32>>,
) -> Result<(), String> {
    let win_w = size.width as i32;
    let detected = detect_side(work, scale, pos, win_w);
    let state = window.state::<MinimalDockState>();
    let stored = *state.side.lock();
    let work_w = work.work_area().size.width as i32;
    let side = resolve_dock_side(stored, detected, prev_pos, pos, work_w);
    if side == DockSide::None {
        *state.side.lock() = DockSide::None;
        clear_persisted_dock_state(window);
        return Ok(());
    }

    let win_h = size.height as i32;
    let area = work.work_area();
    let y = clamp_y(pos.y, area.position.y, area.size.height as i32, win_h);
    apply_dock_side(window, side, y);
    *state.hidden.lock() = false;

    let target_x = visible_x(side, work, win_w);
    animate_x(
        window.clone(),
        pos.x,
        target_x,
        y,
        AnimationEase::OutCubic,
        DockAnimPhase::Snap,
        |_| {},
    );
    Ok(())
}

fn refresh_dock_side(window: &WebviewWindow, allow_loose: bool) -> Result<(), String> {
    let Some((work, scale, pos, size)) = window_geometry(window)? else {
        return Ok(());
    };

    if is_minimized_position(pos) {
        return Ok(());
    }

    let state = window.state::<MinimalDockState>();
    ensure_dock_state_loaded(window);
    let stored = *state.side.lock();
    let win_h = size.height as i32;
    let area = work.work_area();
    let y = clamp_y(pos.y, area.position.y, area.size.height as i32, win_h);

    if stored != DockSide::None {
        *state.docked_y.lock() = y;
        persist_dock_state(window, stored, y);
        return Ok(());
    }

    let side = if allow_loose {
        detect_side_loose(&work, scale, pos, size.width as i32)
    } else {
        detect_side(&work, scale, pos, size.width as i32)
    };

    if side == DockSide::None {
        return Ok(());
    }

    apply_dock_side(window, side, y);
    Ok(())
}

fn hide_to_edge(window: &WebviewWindow) -> Result<(), String> {
    let state = window.state::<MinimalDockState>();
    let side = *state.side.lock();
    if side == DockSide::None || *state.hidden.lock() {
        return Ok(());
    }

    let Some((work, scale, pos, size)) = window_geometry(window)? else {
        return Ok(());
    };

    let win_w = size.width as i32;
    let y = *state.docked_y.lock();
    let peek = peek_width(scale);
    let area = work.work_area();
    let target_x = match side {
        DockSide::Left => area.position.x - win_w + peek,
        DockSide::Right => area.position.x + area.size.width as i32 - peek,
        DockSide::None => return Ok(()),
    };

    let window = window.clone();
    animate_x(
        window.clone(),
        pos.x,
        target_x,
        y,
        AnimationEase::InCubic,
        DockAnimPhase::Hide,
        move |completed| {
            if completed {
                *window.state::<MinimalDockState>().hidden.lock() = true;
            }
        },
    );
    Ok(())
}

fn reposition_to_stored_dock(window: &WebviewWindow, animate: bool) -> Result<(), String> {
    cancel_pending_snap(window);
    ensure_dock_state_loaded(window);

    let state = window.state::<MinimalDockState>();
    let side = *state.side.lock();
    if side == DockSide::None {
        return Ok(());
    }

    let Some((work, scale, pos, size)) = window_geometry(window)? else {
        return Ok(());
    };

    let win_w = size.width as i32;
    let y = *state.docked_y.lock();
    let hidden = *state.hidden.lock();
    let target_x = if hidden {
        let peek = peek_width(scale);
        let area = work.work_area();
        match side {
            DockSide::Left => area.position.x - win_w + peek,
            DockSide::Right => area.position.x + area.size.width as i32 - peek,
            DockSide::None => return Ok(()),
        }
    } else {
        visible_x(side, &work, win_w)
    };

    if pos.x == target_x && pos.y == y {
        return Ok(());
    }

    if animate {
        let ease = if hidden {
            AnimationEase::InCubic
        } else {
            AnimationEase::OutCubic
        };
        let phase = if hidden {
            DockAnimPhase::Hide
        } else {
            DockAnimPhase::Show
        };
        animate_x(window.clone(), pos.x, target_x, y, ease, phase, |_| {});
    } else {
        let _ = window.set_position(PhysicalPosition::new(target_x, y));
        record_position(window, target_x, y);
    }
    Ok(())
}

fn cancel_pending_snap(window: &WebviewWindow) {
    window
        .state::<MinimalDockState>()
        .move_generation
        .fetch_add(1, Ordering::SeqCst);
}

fn schedule_restore_reinforce(window: &WebviewWindow) {
    let window = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(RESTORE_REINFORCE_DELAY_MS));
        let window_for_main = window.clone();
        let _ = window.app_handle().run_on_main_thread(move || {
            let _ = reposition_to_stored_dock(&window_for_main, false);
        });
    });
}

fn ensure_dock_state_loaded(window: &WebviewWindow) {
    let state = window.state::<MinimalDockState>();
    if *state.side.lock() != DockSide::None {
        return;
    }

    let Ok(side_value) = db::with_conn(|conn| repositories::get_setting(conn, SETTINGS_DOCK_SIDE)) else {
        return;
    };
    let Some(side_value) = side_value.filter(|value| !value.is_empty()) else {
        return;
    };

    let side = match side_value.as_str() {
        "left" => DockSide::Left,
        "right" => DockSide::Right,
        _ => return,
    };

    let y = db::with_conn(|conn| repositories::get_setting(conn, SETTINGS_DOCK_Y))
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);

    *state.side.lock() = side;
    *state.docked_y.lock() = y;
}

fn apply_dock_side(window: &WebviewWindow, side: DockSide, y: i32) {
    let state = window.state::<MinimalDockState>();
    *state.side.lock() = side;
    *state.docked_y.lock() = y;
    persist_dock_state(window, side, y);
}

fn persist_dock_state(_window: &WebviewWindow, side: DockSide, y: i32) {
    let side_value = match side {
        DockSide::Left => "left",
        DockSide::Right => "right",
        DockSide::None => return,
    };
    let _ = db::with_conn(|conn| {
        repositories::set_setting(conn, SETTINGS_DOCK_SIDE, side_value)?;
        repositories::set_setting(conn, SETTINGS_DOCK_Y, &y.to_string())?;
        Ok(())
    });
}

fn clear_persisted_dock_state(_window: &WebviewWindow) {
    let _ = db::with_conn(|conn| {
        repositories::set_setting(conn, SETTINGS_DOCK_SIDE, "")?;
        Ok(())
    });
}

fn record_position(window: &WebviewWindow, x: i32, y: i32) {
    *window
        .state::<MinimalDockState>()
        .last_pos
        .lock() = Some(PhysicalPosition::new(x, y));
}

fn is_minimized_position(pos: PhysicalPosition<i32>) -> bool {
    pos.x <= WINDOWS_MINIMIZED_COORD || pos.y <= WINDOWS_MINIMIZED_COORD
}

fn is_at_wrong_dock_x(
    work: &tauri::Monitor,
    scale: f64,
    pos: PhysicalPosition<i32>,
    win_w: i32,
    stored: DockSide,
) -> bool {
    let expected_x = visible_x(stored, work, win_w);
    (pos.x - expected_x).abs() > snap_threshold(scale) * 2
}

fn resolve_dock_side(
    stored: DockSide,
    detected: DockSide,
    prev_pos: Option<PhysicalPosition<i32>>,
    pos: PhysicalPosition<i32>,
    work_w: i32,
) -> DockSide {
    if detected == DockSide::None {
        return DockSide::None;
    }
    if stored == DockSide::None || detected == stored {
        return detected;
    }
    if prev_pos.is_some_and(|prev| !is_suspicious_teleport(prev, pos, work_w)) {
        return detected;
    }
    stored
}

fn is_suspicious_teleport(
    prev: PhysicalPosition<i32>,
    cur: PhysicalPosition<i32>,
    work_w: i32,
) -> bool {
    let jump = (cur.x - prev.x).abs();
    jump > work_w / 3
}

fn expand_if_hidden(window: &WebviewWindow) -> Result<(), String> {
    let state = window.state::<MinimalDockState>();
    let side = *state.side.lock();
    if side == DockSide::None || !*state.hidden.lock() {
        return Ok(());
    }

    let Some((work, pos, size)) = window_geometry(window)?.map(|(work, _, pos, size)| (work, pos, size)) else {
        return Ok(());
    };

    let win_w = size.width as i32;
    let y = *state.docked_y.lock();
    let target_x = visible_x(side, &work, win_w);

    *state.hidden.lock() = false;

    let window = window.clone();
    animate_x(
        window,
        pos.x,
        target_x,
        y,
        AnimationEase::OutCubic,
        DockAnimPhase::Show,
        |_| {},
    );
    Ok(())
}

fn begin_animation(window: &WebviewWindow) -> u64 {
    cancel_pending_snap(window);
    let state = window.state::<MinimalDockState>();
    *state.adjusting.lock() = true;
    state.anim_generation.fetch_add(1, Ordering::SeqCst) + 1
}

fn finish_animation(window: &WebviewWindow, generation: u64) {
    let state = window.state::<MinimalDockState>();
    if state.anim_generation.load(Ordering::SeqCst) == generation {
        *state.adjusting.lock() = false;
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum DockAnimPhase {
    Hide,
    Show,
    Snap,
}

#[derive(Clone, Copy, Serialize)]
struct DockAnimatingPayload {
    active: bool,
    phase: DockAnimPhase,
}

#[derive(Clone, Copy)]
enum AnimationEase {
    InCubic,
    OutCubic,
}

impl AnimationEase {
    fn apply(self, t: f64) -> f64 {
        match self {
            Self::InCubic => t * t * t,
            Self::OutCubic => 1.0 - (1.0 - t).powi(3),
        }
    }
}

fn emit_animating(window: &WebviewWindow, active: bool, phase: DockAnimPhase) {
    let _ = window.emit(
        "minimal-dock-animating",
        DockAnimatingPayload { active, phase },
    );
}

fn animate_x(
    window: WebviewWindow,
    from_x: i32,
    to_x: i32,
    y: i32,
    ease: AnimationEase,
    phase: DockAnimPhase,
    on_complete: impl FnOnce(bool) + Send + 'static,
) {
    if from_x == to_x {
        on_complete(true);
        return;
    }

    emit_animating(&window, true, phase);
    let generation = begin_animation(&window);
    let window_for_emit = window.clone();
    std::thread::spawn(move || {
        let step_ms = (ANIMATION_DURATION_MS / ANIMATION_STEPS as u64).max(8);
        let mut completed = false;

        for step in 0..=ANIMATION_STEPS {
            let state = window.state::<MinimalDockState>();
            if state.anim_generation.load(Ordering::SeqCst) != generation {
                finish_animation(&window, generation);
                on_complete(false);
                return;
            }

            let progress = step as f64 / ANIMATION_STEPS as f64;
            let eased = ease.apply(progress);
            let x = from_x + ((to_x - from_x) as f64 * eased).round() as i32;
            let is_last = step == ANIMATION_STEPS;

            if !set_position_on_main(&window, x, y) {
                finish_animation(&window, generation);
                on_complete(false);
                return;
            }

            if !is_last {
                std::thread::sleep(Duration::from_millis(step_ms));
            } else {
                completed = true;
            }
        }

        finish_animation(&window, generation);
        emit_animating(&window_for_emit, false, phase);
        if completed {
            record_position(&window, to_x, y);
        }
        on_complete(completed);
    });
}

fn set_position_on_main(window: &WebviewWindow, x: i32, y: i32) -> bool {
    let (tx, rx) = mpsc::sync_channel(1);
    let window_for_main = window.clone();
    if window
        .app_handle()
        .run_on_main_thread(move || {
            let result = window_for_main.set_position(PhysicalPosition::new(x, y));
            let _ = tx.send(result.is_ok());
        })
        .is_err()
    {
        return false;
    }

    rx.recv_timeout(Duration::from_millis(120))
        .unwrap_or(false)
}

fn window_geometry(
    window: &WebviewWindow,
) -> Result<Option<(tauri::Monitor, f64, PhysicalPosition<i32>, tauri::PhysicalSize<u32>)>, String>
{
    let Some(monitor) = window
        .current_monitor()
        .map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };

    let pos = window
        .outer_position()
        .map_err(|error| error.to_string())?;
    let size = window.outer_size().map_err(|error| error.to_string())?;
    let scale = monitor.scale_factor();
    Ok(Some((monitor, scale, pos, size)))
}

fn snap_threshold(scale: f64) -> i32 {
    (SNAP_THRESHOLD_LOGICAL * scale).round().max(48.0) as i32
}

fn peek_width(scale: f64) -> i32 {
    (PEEK_WIDTH_LOGICAL * scale).round().max(6.0) as i32
}

fn detect_side(
    work: &tauri::Monitor,
    scale: f64,
    pos: PhysicalPosition<i32>,
    win_w: i32,
) -> DockSide {
    let threshold = snap_threshold(scale);
    let wa_left = work.work_area().position.x;
    let wa_right = work.work_area().position.x + work.work_area().size.width as i32;
    let dist_left = pos.x - wa_left;
    let dist_right = wa_right - (pos.x + win_w);

    if dist_left <= threshold && dist_left <= dist_right {
        DockSide::Left
    } else if dist_right <= threshold {
        DockSide::Right
    } else {
        DockSide::None
    }
}

fn detect_side_loose(
    work: &tauri::Monitor,
    scale: f64,
    pos: PhysicalPosition<i32>,
    win_w: i32,
) -> DockSide {
    let strict = detect_side(work, scale, pos, win_w);
    if strict != DockSide::None {
        return strict;
    }

    let wa_left = work.work_area().position.x;
    let wa_right = work.work_area().position.x + work.work_area().size.width as i32;
    let work_w = work.work_area().size.width as i32;
    let center_x = pos.x + win_w / 2;
    let left_zone = wa_left + work_w / 4;
    let right_zone = wa_right - work_w / 4;

    if center_x <= left_zone {
        DockSide::Left
    } else if center_x >= right_zone {
        DockSide::Right
    } else {
        DockSide::None
    }
}

fn visible_x(side: DockSide, work: &tauri::Monitor, win_w: i32) -> i32 {
    let wa_left = work.work_area().position.x;
    let wa_right = work.work_area().position.x + work.work_area().size.width as i32;
    match side {
        DockSide::Left => wa_left,
        DockSide::Right => wa_right - win_w,
        DockSide::None => wa_left,
    }
}

fn clamp_y(y: i32, wa_top: i32, wa_height: i32, win_h: i32) -> i32 {
    let min_y = wa_top;
    let max_y = wa_top + wa_height - win_h;
    if max_y < min_y {
        return wa_top;
    }
    y.clamp(min_y, max_y)
}

fn is_window_active(window: &WebviewWindow) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        if let Ok(hwnd) = window.hwnd() {
            let foreground = unsafe { GetForegroundWindow() };
            if foreground.0 == hwnd.0 {
                return true;
            }
        }
        return false;
    }

    #[cfg(not(target_os = "windows"))]
    {
        window.is_focused().unwrap_or(false)
    }
}
