//! The level picker shown after choosing a game: one big card per level.
//! The current level glows, so Sebastian can just tap the glowing one.

use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::games::{row_of_cards, MiniGame};
use macroquad::prelude::*;

fn rects(levels: u32) -> Vec<Rect> {
    row_of_cards(levels as usize, 0.92, 0.32, 0.58)
}

/// Returns the level that was tapped, if any.
pub fn update(game: &dyn MiniGame, ctx: &Ctx) -> Option<u32> {
    rects(game.progress().max)
        .iter()
        .position(|r| ctx.input.tapped(*r))
        .map(|i| i as u32 + 1)
}

pub fn draw(game: &dyn MiniGame, name: &str, ctx: &Ctx) {
    let font = ctx.font();
    let (w, h) = (screen_width(), screen_height());
    art::text_center(font, name, vec2(w / 2.0, h * 0.2), h * 0.11, art::INK);

    let current = game.progress().level;
    for (i, r) in rects(game.progress().max).iter().enumerate() {
        let level = i as u32 + 1;
        let paint = Paint::ALL[i % Paint::ALL.len()];
        let color = if paint == Paint::Yellow {
            art::darken(paint.color(), 0.2) // plain yellow is hard to read on white
        } else {
            paint.color()
        };
        let mut r = *r;
        if level == current {
            r = art::scale_rect(r, 1.06 + 0.02 * (ctx.time * 5.0).sin());
            art::glow(r, ctx.time);
        }
        art::card(r, WHITE);

        // One little star per level.
        let star_r = r.w * 0.06;
        for s in 0..level {
            let x = r.center().x + (s as f32 - (level as f32 - 1.0) / 2.0) * star_r * 2.3;
            art::star(vec2(x, r.y + r.h * 0.16), star_r, art::GOLD);
        }

        // The level number, big, so it matches "Level 3!" and the corner readout...
        art::text_center(font, &level.to_string(), vec2(r.center().x, r.y + r.h * 0.5), r.h * 0.4, color);
        // ...and what the level means underneath ("3 colors", "5-10"), shrunk to fit.
        let caption = game.level_label(level);
        let mut size = r.h * 0.13;
        let width = measure_text(&caption, font, size as u16, 1.0).width;
        if width > r.w * 0.9 {
            size *= r.w * 0.9 / width;
        }
        art::text_center(font, &caption, vec2(r.center().x, r.y + r.h * 0.83), size, art::INK);

        // A green check on levels already finished.
        if game.progress().completed.contains(&level) {
            let c = vec2(r.x + r.w * 0.9, r.y + r.w * 0.1);
            let rad = r.w * 0.14;
            draw_circle(c.x, c.y + rad * 0.1, rad, art::SHADOW);
            draw_circle(c.x, c.y, rad, Paint::Green.color());
            let t = rad * 0.22;
            draw_line(c.x - rad * 0.45, c.y, c.x - rad * 0.1, c.y + rad * 0.38, t, WHITE);
            draw_line(c.x - rad * 0.1, c.y + rad * 0.38, c.x + rad * 0.5, c.y - rad * 0.35, t, WHITE);
        }
    }
    let done = game.progress().completed.len();
    if done > 0 {
        let note = format!("Finished {done} of {} levels", game.progress().max);
        art::text_center(font, &note, vec2(w / 2.0, h * 0.86), h * 0.045, art::INK);
    }
}
