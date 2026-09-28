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
    /// A grown-up confirmed removing this player.
    Remove(usize),
}

/// The "Who's playing?" screen. A grown-up can tap "Edit" to remove players.
#[derive(Default)]
pub struct Picker {
    editing: bool,
    /// Asking "Remove Maya?" about this player.
    confirming: Option<usize>,
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

/// The small "Edit" button in the bottom-right corner (for grown-ups).
fn edit_rect() -> Rect {
    let (w, h) = (screen_width(), screen_height());
    Rect::new(w - h * 0.2, h - h * 0.11, h * 0.17, h * 0.08)
}

/// The red X on a player's card in edit mode.
fn remove_spot(card: Rect) -> Vec2 {
    vec2(card.x + card.w * 0.9, card.y + card.w * 0.1)
}

/// "Remove Maya?" dialog: the box, then the Remove and Keep buttons.
fn confirm_rects() -> (Rect, Rect, Rect) {
    let (w, h) = (screen_width(), screen_height());
    let dialog = Rect::new(w * 0.25, h * 0.3, w * 0.5, h * 0.4);
    let bw = dialog.w * 0.38;
    let by = dialog.y + dialog.h * 0.58;
    let remove = Rect::new(dialog.x + dialog.w * 0.08, by, bw, dialog.h * 0.3);
    let keep = Rect::new(dialog.x + dialog.w * 0.54, by, bw, dialog.h * 0.3);
    (dialog, remove, keep)
}

impl Picker {
    /// Developer helper: open edit mode (`--edit-players`).
    pub fn start_editing(&mut self) {
        self.editing = true;
    }

    pub fn update(&mut self, players: &Players, ctx: &mut Ctx) -> Pick {
        if !ctx.input.pressed {
            return Pick::Nothing;
        }
        // The "Remove Maya?" question takes over the screen until it's answered.
        if let Some(i) = self.confirming {
            let (_, remove, keep) = confirm_rects();
            if remove.contains(ctx.input.pos) {
                self.confirming = None;
                self.editing = false;
                ctx.sfx.pop();
                return Pick::Remove(i);
            }
            if keep.contains(ctx.input.pos) {
                self.confirming = None;
                ctx.sfx.pop();
            }
            return Pick::Nothing;
        }
        if edit_rect().contains(ctx.input.pos) {
            self.editing = !self.editing;
            ctx.sfx.pop();
            return Pick::Nothing;
        }
        let cards = player_cards(players.players.len());
        if self.editing {
            for (i, card) in cards.iter().enumerate().take(players.players.len()) {
                if remove_spot(*card).distance(ctx.input.pos) < card.w * 0.16 {
                    self.confirming = Some(i);
                    ctx.sfx.pop();
                    return Pick::Nothing;
                }
            }
            return Pick::Nothing; // while editing, cards can't be picked
        }
        match cards.iter().position(|r| r.contains(ctx.input.pos)) {
            Some(i) if i < players.players.len() => Pick::Player(i),
            Some(_) => Pick::AddNew,
            None => Pick::Nothing,
        }
    }

    pub fn draw(&self, players: &Players, ctx: &Ctx) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        art::text_center(font, "Who's playing?", vec2(w / 2.0, h * 0.16), h * 0.1, art::INK);

        let cards = player_cards(players.players.len());
        for (i, r) in cards.iter().enumerate() {
            let bob = if self.editing { 0.0 } else { (ctx.time * 2.0 + i as f32).sin() * r.h * 0.02 };
            let r = Rect::new(r.x, r.y + bob, r.w, r.h);
            if self.editing && i >= players.players.len() {
                continue; // no "+" card while editing
            }
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
                    if self.editing {
                        let x = remove_spot(r);
                        draw_circle(x.x, x.y, r.w * 0.12, Paint::Red.color());
                        let s = r.w * 0.05;
                        draw_line(x.x - s, x.y - s, x.x + s, x.y + s, r.w * 0.03, WHITE);
                        draw_line(x.x - s, x.y + s, x.x + s, x.y - s, r.w * 0.03, WHITE);
                    }
                }
                None => {
                    // The "+" card: add a new player.
                    draw_circle(avatar.x, avatar.y, r.w * 0.24, art::SHADOW);
                    art::text_center(font, "+", avatar, r.w * 0.4, WHITE);
                    art::text_center(font, "New", vec2(r.center().x, r.y + r.h * 0.76), r.h * 0.13, art::INK);
                }
            }
        }

        // The grown-up "Edit" / "Done" button.
        if !players.players.is_empty() {
            let e = edit_rect();
            art::card(e, if self.editing { Paint::Green.color() } else { WHITE });
            let (label, color) = if self.editing { ("Done", WHITE) } else { ("Edit", art::INK) };
            art::text_center(font, label, e.center(), e.h * 0.5, color);
            if self.editing {
                art::text_center(font, "Tap a red X to remove a player", vec2(w / 2.0, h * 0.93), h * 0.04, art::INK);
            }
        }

        if let Some(i) = self.confirming {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.1, 0.1, 0.2, 0.45));
            let (dialog, remove, keep) = confirm_rects();
            art::card(dialog, WHITE);
            let name = players.players.get(i).map(|p| p.name.as_str()).unwrap_or("");
            let question = format!("Remove {name}?");
            art::text_center(font, &question, vec2(dialog.center().x, dialog.y + dialog.h * 0.22), dialog.h * 0.15, art::INK);
            let note = "Their stars will be gone too.";
            art::text_center(font, note, vec2(dialog.center().x, dialog.y + dialog.h * 0.4), dialog.h * 0.08, art::INK);
            art::card(remove, Paint::Red.color());
            art::text_center(font, "Remove", remove.center(), remove.h * 0.4, WHITE);
            art::card(keep, Paint::Green.color());
            art::text_center(font, "Keep", keep.center(), keep.h * 0.4, WHITE);
        }
    }
}

// ---------- keep your stars, or start over? ----------

pub enum StarChoice {
    Nothing,
    Keep,
    StartOver,
}

fn star_choice_rects() -> (Rect, Rect) {
    let (w, h) = (screen_width(), screen_height());
    let size = (h * 0.4).min(w * 0.3);
    let y = h * 0.4;
    (
        Rect::new(w / 2.0 - size * 1.1, y, size, size),
        Rect::new(w / 2.0 + size * 0.1, y, size, size),
    )
}

pub fn update_star_choice(ctx: &mut Ctx) -> StarChoice {
    let (keep, start_over) = star_choice_rects();
    if ctx.input.tapped(keep) {
        StarChoice::Keep
    } else if ctx.input.tapped(start_over) {
        StarChoice::StartOver
    } else {
        StarChoice::Nothing
    }
}

pub fn draw_star_choice(name: &str, stars: u32, ctx: &Ctx) {
    let font = ctx.font();
    let (w, h) = (screen_width(), screen_height());
    art::text_center(font, &format!("Hi {name}!"), vec2(w / 2.0, h * 0.14), h * 0.1, art::INK);
    let line = format!("You have {stars} stars. Keep them, or start over?");
    art::text_center(font, &line, vec2(w / 2.0, h * 0.28), h * 0.05, art::INK);

    let (keep, start_over) = star_choice_rects();
    for (i, r) in [keep, start_over].iter().enumerate() {
        let bob = (ctx.time * 2.0 + i as f32 * 1.5).sin() * r.h * 0.02;
        let r = Rect::new(r.x, r.y + bob, r.w, r.h);
        art::card(r, WHITE);
        let c = vec2(r.center().x, r.y + r.h * 0.4);
        let count = if i == 0 { stars } else { 0 };
        art::star(c, r.w * 0.28, if i == 0 { art::GOLD } else { art::SHADOW });
        art::text_center(font, &count.to_string(), c + vec2(0.0, r.w * 0.03), r.w * 0.2, art::INK);
        let label = if i == 0 { "Keep" } else { "Start at 0" };
        art::text_center(font, label, vec2(r.center().x, r.y + r.h * 0.82), r.h * 0.13, art::INK);
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
