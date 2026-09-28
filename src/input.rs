//! One simple "finger" input that works the same for touch screens and mice.

use macroquad::prelude::*;

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    /// Where the finger (or mouse) is.
    pub pos: Vec2,
    /// True only on the frame the finger touches down.
    pub pressed: bool,
    /// True while the finger is held down.
    pub down: bool,
}

impl Input {
    pub fn read() -> Self {
        // Prefer real touch events when the platform gives them to us...
        if let Some(t) = touches().first() {
            return Input {
                pos: t.position,
                pressed: matches!(t.phase, TouchPhase::Started),
                down: matches!(
                    t.phase,
                    TouchPhase::Started | TouchPhase::Moved | TouchPhase::Stationary
                ),
            };
        }
        // ...otherwise use the mouse (Windows turns screen taps into mouse clicks).
        let (x, y) = mouse_position();
        if is_mouse_button_released(MouseButton::Left) {
            crate::log::line(&format!("release at {:?}", pointer_position().unwrap_or(vec2(x, y))));
        }
        Input {
            pos: pointer_position().unwrap_or(vec2(x, y)),
            pressed: is_mouse_button_pressed(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
        }
    }

    /// Did the finger just touch down inside `r`?
    pub fn tapped(&self, r: Rect) -> bool {
        self.pressed && r.contains(self.pos)
    }
}

/// Windows sometimes reports one tap twice, a split second apart at the exact same spot.
/// For most buttons that's harmless, but it would flip the speaker button off and straight
/// back on. This filter drops a second tap that's too fast and too close to the last one.
#[derive(Default)]
pub struct TapFilter {
    last: Option<(f64, Vec2)>,
}

impl TapFilter {
    const MIN_GAP_SECS: f64 = 0.25;
    const SAME_SPOT: f32 = 12.0;

    pub fn read(&mut self) -> Input {
        let mut input = Input::read();
        if input.pressed {
            let now = get_time();
            if let Some((t, pos)) = self.last {
                if now - t < Self::MIN_GAP_SECS && pos.distance(input.pos) < Self::SAME_SPOT {
                    crate::log::line(&format!("ignored double tap at {:?}", input.pos));
                    input.pressed = false;
                    return input;
                }
            }
            self.last = Some((now, input.pos));
        }
        input
    }
}

/// Ask Windows directly where the pointer is, in game units.
///
/// Why: when a finger taps the screen, Windows sends a "button down" without always
/// moving the mouse there first, and macroquad reports the click at the *previous*
/// pointer spot. Asking Windows ourselves gives the real tap location.
fn pointer_position() -> Option<Vec2> {
    let (p, size) = pointer_raw()?;
    // Windows measures in real pixels; scale to the game's own units, which can differ
    // with display scaling (e.g. 150%).
    Some(vec2(p.x * screen_width() / size.x, p.y * screen_height() / size.y))
}

/// For the debug log: what Windows reports, before any scaling.
pub fn debug_pointer() -> String {
    match pointer_raw() {
        Some((p, size)) => format!("windows says {:.0},{:.0} in a {:.0}x{:.0} window", p.x, p.y, size.x, size.y),
        None => "windows pointer unavailable".to_string(),
    }
}

/// The pointer's position inside our window and the window's size, both in real pixels.
#[cfg(windows)]
fn pointer_raw() -> Option<(Vec2, Vec2)> {
    // FFI: declaring functions that live in Windows' user32.dll so Rust can call them.
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> isize;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetCursorPos(p: *mut Point) -> i32;
        fn ScreenToClient(hwnd: isize, p: *mut Point) -> i32;
        fn GetClientRect(hwnd: isize, r: *mut WinRect) -> i32;
    }

    // `unsafe`: Rust can't check what code in another language does, so we promise it's OK.
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if hwnd == 0 || pid != std::process::id() {
            return None; // our window isn't in front
        }
        let mut p = Point { x: 0, y: 0 };
        let mut r = WinRect { left: 0, top: 0, right: 0, bottom: 0 };
        if GetCursorPos(&mut p) == 0 || ScreenToClient(hwnd, &mut p) == 0 || GetClientRect(hwnd, &mut r) == 0 {
            return None;
        }
        let (w, h) = ((r.right - r.left) as f32, (r.bottom - r.top) as f32);
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        Some((vec2(p.x as f32, p.y as f32), vec2(w, h)))
    }
}

#[cfg(not(windows))]
fn pointer_raw() -> Option<(Vec2, Vec2)> {
    None
}
