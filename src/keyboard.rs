//! A big on-screen keyboard for little fingers (the laptop's keys work too).
//! Used for typing names and in the Spelling game.

use crate::art::{self, Paint};
use crate::ctx::Ctx;
use macroquad::prelude::*;

const KEY_ROWS: [&str; 3] = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Key {
    /// A capital letter A-Z.
    Letter(char),
    /// Space, dash or apostrophe typed on the laptop keyboard (handy in names).
    Other(char),
    Back,
    /// The big green button at the bottom ("Done", "Say it", ...).
    Done,
}

pub struct Keyboard {
    /// How far down the screen the first row starts (0 = top, 1 = bottom).
    top: f32,
    done_label: &'static str,
    /// Keys shrink for a moment when pressed.
    flash: Option<(Key, f32)>,
}

impl Keyboard {
    pub fn new(top: f32, done_label: &'static str) -> Self {
        Keyboard { top, done_label, flash: None }
    }

    fn keys(&self) -> Vec<(Key, Rect)> {
        let (w, h) = (screen_width(), screen_height());
        let k = (w * 0.86 / 10.0).min(h * 0.105);
        let gap = k * 0.1;
        let mut keys = Vec::new();
        for (row, letters) in KEY_ROWS.iter().enumerate() {
            let count = letters.len() as f32 + if row == 2 { 1.6 } else { 0.0 };
            let x0 = (w - count * k) / 2.0;
            let y = h * self.top + row as f32 * k;
            for (col, c) in letters.chars().enumerate() {
                keys.push((Key::Letter(c), Rect::new(x0 + col as f32 * k + gap / 2.0, y, k - gap, k - gap)));
            }
            if row == 2 {
                let x = x0 + letters.len() as f32 * k + gap / 2.0;
                keys.push((Key::Back, Rect::new(x, y, k * 1.6 - gap, k - gap)));
            }
        }
        let done_w = k * 3.5;
        let done_y = h * self.top + 3.0 * k + gap * 2.0;
        keys.push((Key::Done, Rect::new((w - done_w) / 2.0, done_y, done_w, k * 0.95)));
        keys
    }

    /// The key pressed this frame, if any (on screen or on the laptop's keyboard).
    pub fn update(&mut self, ctx: &Ctx) -> Option<Key> {
        if let Some((_, t)) = &mut self.flash {
            *t -= ctx.dt * 4.0;
            if *t <= 0.0 {
                self.flash = None;
            }
        }
        let mut key = None;
        while let Some(c) = get_char_pressed() {
            if c.is_ascii_alphabetic() {
                key = Some(Key::Letter(c.to_ascii_uppercase()));
            } else if c == ' ' || c == '-' || c == '\'' {
                key = Some(Key::Other(c));
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            key = Some(Key::Back);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            key = Some(Key::Done);
        }
        if ctx.input.pressed {
            if let Some((k, _)) = self.keys().into_iter().find(|(_, r)| r.contains(ctx.input.pos)) {
                key = Some(k);
            }
        }
        if let Some(k) = key {
            self.flash = Some((k, 1.0));
        }
        key
    }

    /// `ready`: whether the big button is lit up (green) or greyed out.
    pub fn draw(&self, ctx: &Ctx, ready: bool) {
        let font = ctx.font();
        for (key, r) in self.keys() {
            let flash = match self.flash {
                Some((k, t)) if k == key => t,
                _ => 0.0,
            };
            let r = art::scale_rect(r, 1.0 - 0.1 * flash);
            match key {
                Key::Letter(c) => {
                    art::card(r, WHITE);
                    art::text_center(font, &c.to_string(), r.center(), r.h * 0.6, art::INK);
                }
                Key::Back => {
                    art::card(r, Color::from_rgba(255, 228, 228, 255));
                    backspace_icon(r.center(), r.h * 0.3);
                }
                Key::Done => {
                    let fill = if ready { Paint::Green.color() } else { art::SHADOW };
                    art::card(r, fill);
                    art::text_center(font, self.done_label, r.center(), r.h * 0.55, WHITE);
                }
                Key::Other(_) => {}
            }
        }
    }
}

/// A "delete" arrow pointing left.
fn backspace_icon(c: Vec2, s: f32) {
    let red = Paint::Red.color();
    draw_triangle(vec2(c.x - s * 1.2, c.y), vec2(c.x - s * 0.4, c.y - s * 0.7), vec2(c.x - s * 0.4, c.y + s * 0.7), red);
    draw_rectangle(c.x - s * 0.45, c.y - s * 0.3, s * 1.5, s * 0.6, red);
}
