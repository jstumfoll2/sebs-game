//! The home screen: a bouncing rainbow title (tap a letter to hear it!) and four big game tiles.

use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use crate::alphabet::{self, capitalize};
use macroquad::prelude::*;

/// "Sebastian's Game", or "James' Game" for a name ending in s.
fn title(ctx: &Ctx) -> String {
    let name = &ctx.name;
    if ctx.versus.is_some() {
        return format!("{name}'s turn!");
    }
    if name.ends_with('s') {
        format!("{name}' Game")
    } else {
        format!("{name}'s Game")
    }
}
pub const LABELS: [&str; 10] = [
    "Colors", "Patterns", "Shadows", "Puzzles", "Letters", "Spelling", "Counting", "Groups", "Drawing", "Writing",
];
const TILE_COLORS: [(u8, u8, u8); 10] = [
    (255, 228, 236),
    (232, 224, 255),
    (230, 230, 240),
    (255, 243, 205),
    (220, 245, 228),
    (236, 246, 214),
    (255, 236, 214),
    (222, 238, 255),
    (250, 230, 255),
    (255, 240, 220),
];

pub struct Menu {
    /// Per-letter hop when a title letter is tapped.
    bounce: Vec<f32>,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            bounce: Vec::new(),
        }
    }

    /// Returns the index of the game that was tapped, if any.
    pub fn update(&mut self, ctx: &mut Ctx) -> Option<usize> {
        let title = title(ctx);
        self.bounce.resize(title.chars().count(), 0.0);
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

        let (size, letters) = title_layout(ctx.font(), &title, title_room(ctx));
        for (i, (ch, c, w)) in letters.iter().enumerate() {
            let hit = Rect::new(c.x - w / 2.0, c.y - size * 0.6, *w, size * 1.2);
            if ch.is_alphabetic() && hit.contains(p) {
                self.bounce[i] = 1.0;
                ctx.sfx.pop();
                if let Some(l) = alphabet::get(*ch) {
                    // "Ess! Ess says sss."
                    ctx.voice.say(&format!("{}! {} says {}.", capitalize(l.name), capitalize(l.name), l.sound()));
                }
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();

        // Rainbow title; each letter gently waves and hops when tapped.
        let (size, letters) = title_layout(font, &title(ctx), title_room(ctx));
        for (i, (ch, c, _)) in letters.iter().enumerate() {
            let bounce = self.bounce.get(i).copied().unwrap_or(0.0);
            let hop = (bounce * std::f32::consts::PI).sin() * size * 0.35;
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

/// Big game tiles: three on top, the rest centered underneath.
pub fn tiles() -> Vec<Rect> {
    let (w, h) = (screen_width(), screen_height());
    let area = Rect::new(w * 0.05, h * 0.27, w * 0.9, h * 0.68);
    let n = LABELS.len();
    let cols = if n > 8 {
        5
    } else if n > 6 {
        4
    } else {
        3
    };
    let rows = n.div_ceil(cols);
    let gap = area.h * 0.05;
    let tw = (area.w - gap * (cols - 1) as f32) / cols as f32;
    let th = (area.h - gap * (rows - 1) as f32) / rows as f32;
    (0..n)
        .map(|i| {
            let (col, row) = (i % cols, i / cols);
            let in_row = (n - row * cols).min(cols);
            let shift = (cols - in_row) as f32 * (tw + gap) / 2.0;
            Rect::new(area.x + shift + col as f32 * (tw + gap), area.y + row as f32 * (th + gap), tw, th)
        })
        .collect()
}

/// Font size plus (letter, center, width) for each title character.
/// How much of the screen width the title may use (less in a match, to leave room for
/// both players' scores).
fn title_room(ctx: &Ctx) -> f32 {
    if ctx.versus.is_some() {
        0.38
    } else {
        0.46
    }
}

fn title_layout(font: Option<&Font>, title: &str, room: f32) -> (f32, Vec<(char, Vec2, f32)>) {
    let (w, h) = (screen_width(), screen_height());
    let measure = |size: f32| -> Vec<f32> {
        title
            .chars()
            .map(|ch| {
                if ch == ' ' {
                    size * 0.35
                } else {
                    art::measure(font, &ch.to_string(), size).width + size * 0.04
                }
            })
            .collect()
    };
    // Start big, then shrink long names so the title fits between the top buttons.
    let mut size = (h * 0.12).min(w * 0.066);
    let natural: f32 = measure(size).iter().sum();
    if natural > w * room {
        size *= w * room / natural;
    }
    let widths = measure(size);
    let total: f32 = widths.iter().sum();
    let mut x = (w - total) / 2.0;
    let y = h * 0.14;
    let letters = title
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
    // Size icons to fit narrow tiles too (about 7 icon-widths across).
    let s = (r.h * 0.17).min(r.w * 0.13);
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
            // A colorful fish next to its shadow.
            let dark = Color::new(0.24, 0.24, 0.36, 1.0);
            art::draw_thing(Thing::Fish, vec2(c.x - spread * 0.6, c.y), s * 1.2, Paint::Orange.color());
            art::draw_thing(Thing::Fish, vec2(c.x + spread * 0.6, c.y), s * 1.2, dark);
        }
        3 => {
            // Four puzzle pieces, one lifted out.
            let p = s * 1.1;
            let colors = [Paint::Red, Paint::Blue, Paint::Green, Paint::Yellow];
            for (k, paint) in colors.iter().enumerate() {
                let (col, row) = ((k % 2) as f32, (k / 2) as f32);
                let lift = if k == 3 { vec2(p * 0.5, p * 0.35) } else { Vec2::ZERO };
                let r = Rect::new(c.x - p + col * p + lift.x, c.y - p + row * p + lift.y, p * 0.94, p * 0.94);
                art::rounded_rect(r, p * 0.15, paint.color());
            }
        }
        4 => {
            for (k, (letter, p)) in [("A", Paint::Red), ("B", Paint::Blue), ("C", Paint::Green)].iter().enumerate() {
                let x = c.x + (k as f32 - 1.0) * spread;
                art::text_center(font, letter, vec2(x, c.y), s * 2.4, p.color());
            }
        }
        5 => {
            // C A T in letter boxes.
            for (k, (letter, p)) in [("C", Paint::Red), ("A", Paint::Blue), ("T", Paint::Green)].iter().enumerate() {
                let x = c.x + (k as f32 - 1.0) * spread;
                let bx = Rect::new(x - s * 1.05, c.y - s * 1.05, s * 2.1, s * 2.1);
                art::card(bx, WHITE);
                art::text_center(font, letter, bx.center(), s * 1.6, p.color());
            }
        }
        6 => {
            for k in 0..3 {
                let x = c.x + (k as f32 - 1.0) * spread;
                art::draw_thing(Thing::Apple, vec2(x, c.y), s * 0.9, Paint::Red.color());
                art::badge(font, vec2(x + s * 0.75, c.y - s * 0.75), s * 0.32, &(k + 1).to_string());
            }
        }
        7 => {
            // Two plates of two apples: "2, 4".
            for k in 0..2 {
                let px = c.x + (k as f32 - 0.5) * spread * 1.4;
                let plate = Rect::new(px - s * 1.5, c.y - s * 1.1, s * 3.0, s * 2.2);
                art::card(plate, WHITE);
                for a in 0..2 {
                    let ax = px + (a as f32 - 0.5) * s * 1.3;
                    art::draw_thing(Thing::Apple, vec2(ax, c.y), s * 0.55, Paint::Red.color());
                }
                art::badge(font, vec2(plate.x + plate.w, plate.y), s * 0.36, &((k + 1) * 2).to_string());
            }
        }
        8 => {
            // A painter's palette with a rainbow brush stroke.
            let pal = c + vec2(-spread * 0.45, 0.0);
            draw_ellipse(pal.x, pal.y, s * 1.6, s * 1.15, 0.0, Color::from_rgba(214, 170, 120, 255));
            for (k, p) in [Paint::Red, Paint::Yellow, Paint::Green, Paint::Blue].iter().enumerate() {
                let a = -2.4 + k as f32 * 0.8;
                draw_circle(pal.x + a.cos() * s * 1.05, pal.y + a.sin() * s * 0.7, s * 0.28, p.color());
            }
            for (k, p) in [Paint::Red, Paint::Orange, Paint::Yellow, Paint::Green, Paint::Blue].iter().enumerate() {
                let r = s * (1.4 - k as f32 * 0.15);
                art::arc(c + vec2(spread * 0.75, s * 0.9), r, 1.1 * std::f32::consts::PI, 1.9 * std::f32::consts::PI, s * 0.16, p.color());
            }
        }
        _ => {
            // A dotted squiggle being traced by a pencil.
            let squiggle = |t: f32| c + vec2(-spread * 0.3 + (t * std::f32::consts::TAU).sin() * s * 1.1, (t - 0.5) * s * 3.2);
            for k in 0..=24 {
                let t = k as f32 / 24.0;
                let p = squiggle(t);
                draw_circle(p.x, p.y, s * 0.15, if t < 0.6 { Paint::Blue.color() } else { art::SHADOW });
            }
            // The pencil: dark tip, yellow body and pink eraser, where the traced part ends.
            let tip = squiggle(0.6);
            let dir = vec2(0.55, -0.83);
            let side = dir.perp() * s * 0.38;
            let (a, b) = (tip + dir * s * 1.0, tip + dir * s * 3.4);
            draw_triangle(tip, a + side, a - side, Color::from_rgba(90, 60, 40, 255));
            draw_triangle(a + side, a - side, b - side, Paint::Yellow.color());
            draw_triangle(a + side, b - side, b + side, Paint::Yellow.color());
            let e = b + dir * s * 0.6;
            draw_triangle(b + side, b - side, e - side, Paint::Pink.color());
            draw_triangle(b + side, e - side, e + side, Paint::Pink.color());
        }
    }
}
