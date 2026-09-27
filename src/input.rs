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
        // ...otherwise use the mouse (Windows also turns screen taps into mouse clicks).
        let (x, y) = mouse_position();
        Input {
            pos: vec2(x, y),
            pressed: is_mouse_button_pressed(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
        }
    }

    /// Did the finger just touch down inside `r`?
    pub fn tapped(&self, r: Rect) -> bool {
        self.pressed && r.contains(self.pos)
    }
}
