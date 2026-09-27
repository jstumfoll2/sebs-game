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

/// Ask Windows directly where the pointer is.
///
/// Why: when a finger taps the screen, Windows sends a "button down" without always
/// moving the mouse there first, and macroquad reports the click at the *previous*
/// pointer spot. Asking Windows ourselves gives the real tap location.
#[cfg(windows)]
fn pointer_position() -> Option<Vec2> {
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
        // Windows gives real pixels; the game may use scaled units (display scaling).
        Some(vec2(p.x as f32 * screen_width() / w, p.y as f32 * screen_height() / h))
    }
}

#[cfg(not(windows))]
fn pointer_position() -> Option<Vec2> {
    None
}
