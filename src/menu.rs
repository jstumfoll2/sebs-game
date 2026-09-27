//! The home screen: a bouncing rainbow title (tap a letter to hear it!) and four big game tiles.

use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use crate::games::letters::letter_name;
use macroquad::prelude::*;

const TITLE: &str = "Sebastian's Game";
const LABELS: [&str; 4] = ["Colors", "Patterns", "Letters", "Counting"];
const TILE_COLORS: [(u8, u8, u8); 4] = [(255, 228, 236), (232, 224, 255), (220, 245, 228), (255, 236, 214)];

pub struct Menu {
    /// Per-letter hop when a title letter is tapped.
    bounce: Vec<f32>,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            bounce: vec![0.0; TITLE.chars().count()],
        }
    }

    /// Returns the index of the game that was tapped, if any.
    pub fn update(&mut self, ctx: &mut Ctx) -> Option<usize> {
        crate::games::fade(&mut self.bounce, ctx.dt, 2.0);
        if !ctx.input.pressed {
            return None;
        }
        let p = ctx.input.pos;

        for (i, r) in tiles().iter().enumerate() {
            if r.contains(p) {
                ctx.voice.say(&format!("{}!", LABELS[i]));
                return Some(i);
            }
        }

        let (size, letters) = title_layout(ctx.font());
        for (i, (ch, c, w)) in letters.iter().enumerate() {
            let hit = Rect::new(c.x - w / 2.0, c.y - size * 0.6, *w, size * 1.2);
            if ch.is_alphabetic() && hit.contains(p) {
                self.bounce[i] = 1.0;
                ctx.sfx.pop();
                ctx.voice.say(letter_name(*ch));
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();

        // Rainbow title; each letter gently waves and hops when tapped.
        let (size, letters) = title_layout(font);
        for (i, (ch, c, _)) in letters.iter().enumerate() {
            let hop = (self.bounce[i] * std::f32::consts::PI).sin() * size * 0.35;
            let wave = (ctx.time * 2.5 + i as f32 * 0.5).sin() * size * 0.05;
            let pos = *c - vec2(0.0, hop + wave);
            let color = Paint::ALL[i % Paint::ALL.len()].color();
            let text = ch.to_string();
            art::text_center(font, &text, pos + vec2(0.0, size * 0.05), size, art::darken(color, 0.4));
            art::text_center(font, &text, pos, size, color);
        }

        for (i, r) in tiles().iter().enumerate() {
            let bob = (ctx.time * 2.0 + i as f32 * 1.3).sin() * r.h * 0.015;
            let r = Rect::new(r.x, r.y + bob, r.w, r.h);
            let (cr, cg, cb) = TILE_COLORS[i];
            art::card(r, Color::from_rgba(cr, cg, cb, 255));
            draw_tile_icon(i, r, ctx);
            let label_c = vec2(r.center().x, r.y + r.h * 0.84);
            art::text_center(font, LABELS[i], label_c, r.h * 0.15, art::INK);
        }
    }
}

/// Four big tiles in a 2x2 grid.
pub fn tiles() -> [Rect; 4] {
    let (w, h) = (screen_width(), screen_height());
    let area = Rect::new(w * 0.08, h * 0.27, w * 0.84, h * 0.68);
    let gap = area.h * 0.06;
    let tw = (area.w - gap) / 2.0;
    let th = (area.h - gap) / 2.0;
    std::array::from_fn(|i| {
        let col = (i % 2) as f32;
        let row = (i / 2) as f32;
        Rect::new(area.x + col * (tw + gap), area.y + row * (th + gap), tw, th)
    })
}

/// Font size plus (letter, center, width) for each title character.
fn title_layout(font: Option<&Font>) -> (f32, Vec<(char, Vec2, f32)>) {
    let (w, h) = (screen_width(), screen_height());
    let size = (h * 0.12).min(w * 0.066);
    let widths: Vec<f32> = TITLE
        .chars()
        .map(|ch| {
            if ch == ' ' {
                size * 0.35
            } else {
                measure_text(&ch.to_string(), font, size as u16, 1.0).width + size * 0.04
            }
        })
        .collect();
    let total: f32 = widths.iter().sum();
    let mut x = (w - total) / 2.0;
    let y = h * 0.14;
    let letters = TITLE
        .chars()
        .zip(&widths)
        .map(|(ch, &cw)| {
            let c = vec2(x + cw / 2.0, y);
            x += cw;
            (ch, c, cw)
        })
        .collect();
    (size, letters)
}

fn draw_tile_icon(i: usize, r: Rect, ctx: &Ctx) {
    let font = ctx.font();
    let s = r.h * 0.17;
    let c = vec2(r.center().x, r.y + r.h * 0.4);
    let spread = s * 2.4;
    match i {
        0 => {
            for (k, p) in [Paint::Red, Paint::Blue, Paint::Yellow].iter().enumerate() {
                let x = c.x + (k as f32 - 1.0) * spread;
                art::draw_thing(Thing::Ball, vec2(x, c.y), s, p.color());
            }
        }
        1 => {
            let items = [Paint::Red, Paint::Blue, Paint::Red];
            for (k, p) in items.iter().enumerate() {
                let x = c.x + (k as f32 - 1.5) * spread * 0.9;
                art::draw_thing(Thing::Star, vec2(x, c.y), s * 0.9, p.color());
            }
            art::text_center(font, "?", vec2(c.x + 1.5 * spread * 0.9, c.y), s * 2.2, art::INK);
        }
        2 => {
            for (k, (letter, p)) in [("A", Paint::Red), ("B", Paint::Blue), ("C", Paint::Green)].iter().enumerate() {
                let x = c.x + (k as f32 - 1.0) * spread;
                art::text_center(font, letter, vec2(x, c.y), s * 2.4, p.color());
            }
        }
        _ => {
            for k in 0..3 {
                let x = c.x + (k as f32 - 1.0) * spread;
                art::draw_thing(Thing::Apple, vec2(x, c.y), s * 0.9, Paint::Red.color());
                art::badge(font, vec2(x + s * 0.75, c.y - s * 0.75), s * 0.32, &(k + 1).to_string());
            }
        }
    }
}
