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

        let (big, small) = game.level_label(level);
        let big_size = (r.h * 0.36).min(r.w * 1.5 / big.len().max(1) as f32);
        art::text_center(font, &big, vec2(r.center().x, r.y + r.h * 0.52), big_size, color);
        if !small.is_empty() {
            art::text_center(font, &small, vec2(r.center().x, r.y + r.h * 0.82), r.h * 0.13, art::INK);
        }
    }
}
