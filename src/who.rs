//! "Who's playing?": pick a player, or add a new one by typing their name.

use crate::alphabet;
use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::hud;
use crate::players::{tidy_name, Players, MAX_NAME};
use macroquad::prelude::*;

// ---------- picking a player ----------

pub enum Pick {
    Nothing,
    Player(usize),
    AddNew,
}

/// One card per player plus a "+" card, in a centered grid.
fn player_cards(players: usize) -> Vec<Rect> {
    let (w, h) = (screen_width(), screen_height());
    let n = players + 1;
    let cols = n.min(4);
    let rows = n.div_ceil(cols);
    let cell = (w * 0.84 / cols as f32).min(h * 0.62 / rows as f32).min(h * 0.42);
    let size = cell * 0.86;
    let x0 = (w - cell * cols as f32) / 2.0;
    let y0 = h * 0.3;
    (0..n)
        .map(|i| {
            let (col, row) = ((i % cols) as f32, (i / cols) as f32);
            // Center a short last row.
            let in_row = if (i / cols) == rows - 1 { n - (rows - 1) * cols } else { cols };
            let shift = (cols - in_row) as f32 * cell / 2.0;
            Rect::new(x0 + shift + col * cell + (cell - size) / 2.0, y0 + row * cell, size, size)
        })
        .collect()
}

pub fn update_picker(players: &Players, ctx: &mut Ctx) -> Pick {
    let cards = player_cards(players.players.len());
    match cards.iter().position(|r| ctx.input.tapped(*r)) {
        Some(i) if i < players.players.len() => Pick::Player(i),
        Some(_) => Pick::AddNew,
        None => Pick::Nothing,
    }
}

pub fn draw_picker(players: &Players, ctx: &Ctx) {
    let font = ctx.font();
    let (w, h) = (screen_width(), screen_height());
    art::text_center(font, "Who's playing?", vec2(w / 2.0, h * 0.16), h * 0.1, art::INK);

    let cards = player_cards(players.players.len());
    for (i, r) in cards.iter().enumerate() {
        let bob = (ctx.time * 2.0 + i as f32).sin() * r.h * 0.02;
        let r = Rect::new(r.x, r.y + bob, r.w, r.h);
        art::card(r, WHITE);
        let avatar = vec2(r.center().x, r.y + r.h * 0.38);
        match players.players.get(i) {
            Some(p) => {
                // A colored circle with their first letter, their name, and their stars.
                let color = Paint::ALL[i % Paint::ALL.len()].color();
                draw_circle(avatar.x, avatar.y, r.w * 0.24, color);
                let initial: String = p.name.chars().take(1).collect();
                art::text_center(font, &initial, avatar, r.w * 0.3, WHITE);
                art::text_center(font, &p.name, vec2(r.center().x, r.y + r.h * 0.72), r.h * 0.13, art::INK);
                let star_c = vec2(r.center().x - r.w * 0.12, r.y + r.h * 0.88);
                art::star(star_c, r.w * 0.06, art::GOLD);
                art::text_left(font, &p.stars.to_string(), star_c.x + r.w * 0.09, star_c.y, r.h * 0.1, art::INK);
            }
            None => {
                // The "+" card: add a new player.
                draw_circle(avatar.x, avatar.y, r.w * 0.24, art::SHADOW);
                art::text_center(font, "+", avatar, r.w * 0.4, WHITE);
                art::text_center(font, "New", vec2(r.center().x, r.y + r.h * 0.76), r.h * 0.13, art::INK);
            }
        }
    }
}

// ---------- typing a new name ----------

const KEY_ROWS: [&str; 3] = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

#[derive(Clone, Copy, PartialEq)]
enum Key {
    Letter(char),
    Back,
    Done,
}

pub enum Typed {
    Nothing,
    Done(String),
    Cancel,
}

#[derive(Default)]
pub struct NameEntry {
    text: String,
    /// Keys flash when tapped.
    flash: Option<(Key, f32)>,
}

impl NameEntry {
    pub fn start(&mut self, ctx: &mut Ctx) {
        self.text.clear();
        ctx.voice.say("Hi there! What's your name? Ask a grown-up to help type it.");
    }

    fn keys() -> Vec<(Key, Rect)> {
        let (w, h) = (screen_width(), screen_height());
        let k = (w * 0.86 / 10.0).min(h * 0.11);
        let gap = k * 0.1;
        let mut keys = Vec::new();
        for (row, letters) in KEY_ROWS.iter().enumerate() {
            let count = letters.len() as f32 + if row == 2 { 1.6 } else { 0.0 };
            let x0 = (w - count * k) / 2.0;
            let y = h * 0.42 + row as f32 * k;
            for (col, c) in letters.chars().enumerate() {
                keys.push((Key::Letter(c), Rect::new(x0 + col as f32 * k + gap / 2.0, y, k - gap, k - gap)));
            }
            if row == 2 {
                let x = x0 + letters.len() as f32 * k + gap / 2.0;
                keys.push((Key::Back, Rect::new(x, y, k * 1.6 - gap, k - gap)));
            }
        }
        let done_w = k * 3.5;
        keys.push((Key::Done, Rect::new((w - done_w) / 2.0, h * 0.42 + 3.0 * k + gap * 2.0, done_w, k * 0.95)));
        keys
    }

    fn press(&mut self, key: Key, ctx: &mut Ctx) -> Typed {
        self.flash = Some((key, 1.0));
        match key {
            Key::Letter(c) => {
                if self.text.chars().count() < MAX_NAME {
                    self.text.push(c.to_ascii_lowercase());
                    ctx.sfx.pop();
                    if let Some(l) = alphabet::get(c) {
                        ctx.voice.say(l.name);
                    }
                }
            }
            Key::Back => {
                self.text.pop();
                ctx.sfx.pop();
            }
            Key::Done => {
                let name = tidy_name(&self.text);
                if !name.is_empty() {
                    return Typed::Done(name);
                }
                ctx.voice.say("Type a name first!");
            }
        }
        Typed::Nothing
    }

    /// `can_cancel`: show the home button (there are other players to go back to).
    pub fn update(&mut self, ctx: &mut Ctx, can_cancel: bool) -> Typed {
        if let Some((_, t)) = &mut self.flash {
            *t -= ctx.dt * 4.0;
            if *t <= 0.0 {
                self.flash = None;
            }
        }
        if can_cancel && ctx.input.tapped(hud::home_rect()) {
            return Typed::Cancel;
        }

        // The laptop's keyboard works too.
        while let Some(c) = get_char_pressed() {
            if c.is_ascii_alphabetic() {
                if let Typed::Done(n) = self.press(Key::Letter(c.to_ascii_uppercase()), ctx) {
                    return Typed::Done(n);
                }
            } else if (c == ' ' || c == '-' || c == '\'') && self.text.chars().count() < MAX_NAME {
                self.text.push(c);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.press(Key::Back, ctx);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            return self.press(Key::Done, ctx);
        }

        if ctx.input.pressed {
            if let Some((key, _)) = Self::keys().into_iter().find(|(_, r)| r.contains(ctx.input.pos)) {
                return self.press(key, ctx);
            }
        }
        Typed::Nothing
    }

    pub fn draw(&self, ctx: &Ctx, can_cancel: bool) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        art::text_center(font, "What's your name?", vec2(w / 2.0, h * 0.13), h * 0.08, art::INK);

        // The name so far, in rainbow letters.
        let box_r = Rect::new(w * 0.2, h * 0.2, w * 0.6, h * 0.16);
        art::card(box_r, WHITE);
        let shown = tidy_name(&self.text);
        let size = box_r.h * 0.65;
        let total: f32 = shown.chars().map(|c| measure_text(&c.to_string(), font, size as u16, 1.0).width).sum();
        let mut x = box_r.center().x - total / 2.0;
        for (i, c) in shown.chars().enumerate() {
            let cw = measure_text(&c.to_string(), font, size as u16, 1.0).width;
            let color = art::readable(Paint::ALL[i % Paint::ALL.len()].color());
            art::text_center(font, &c.to_string(), vec2(x + cw / 2.0, box_r.center().y), size, color);
            x += cw;
        }
        // Blinking cursor.
        if (ctx.time * 2.0) as i32 % 2 == 0 {
            draw_rectangle(x + size * 0.05, box_r.center().y - size * 0.4, size * 0.06, size * 0.8, art::INK);
        }

        for (key, r) in Self::keys() {
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
                    let ready = !tidy_name(&self.text).is_empty();
                    let fill = if ready { Paint::Green.color() } else { art::SHADOW };
                    art::card(r, fill);
                    art::text_center(font, "Done", r.center(), r.h * 0.55, WHITE);
                }
            }
        }

        if can_cancel {
            hud::draw_home_button(ctx);
        }
    }
}

/// A "delete" arrow pointing left.
fn backspace_icon(c: Vec2, s: f32) {
    let red = Paint::Red.color();
    draw_triangle(vec2(c.x - s * 1.2, c.y), vec2(c.x - s * 0.4, c.y - s * 0.7), vec2(c.x - s * 0.4, c.y + s * 0.7), red);
    draw_rectangle(c.x - s * 0.45, c.y - s * 0.3, s * 1.5, s * 0.6, red);
}
