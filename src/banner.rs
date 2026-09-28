//! The "Level 3 done! On to level 4!" banner that pops up (with fireworks) when a player
//! finishes a level.

use crate::art::{self, Paint};
use crate::ctx::Ctx;
use macroquad::prelude::*;

/// How long the banner stays up.
const SHOW_SECS: f32 = 3.5;

pub struct LevelBanner {
    done: u32,
    /// Was that the game's last level?
    last: bool,
    left: f32,
}

impl LevelBanner {
    /// Show the banner, start the fireworks, and say it.
    pub fn new(done: u32, max: u32, ctx: &mut Ctx) -> Self {
        let last = done >= max;
        ctx.fireworks.show(2.0);
        if last {
            ctx.voice.then(&format!("Amazing! You finished level {done}, the very last level!"));
        } else {
            ctx.voice.then(&format!("Wow! You finished level {done}! Now let's try level {}!", done + 1));
        }
        LevelBanner { done, last, left: SHOW_SECS }
    }

    /// Returns false once the banner is done.
    pub fn update(&mut self, dt: f32) -> bool {
        self.left -= dt;
        self.left > 0.0
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        // Pop in quickly, shrink away at the end.
        let age = SHOW_SECS - self.left;
        let scale = (age * 5.0).min(1.0).min(self.left * 4.0).max(0.0);
        let wobble = 1.0 + 0.03 * (ctx.time * 6.0).sin();
        let r = art::scale_rect(Rect::new(w * 0.22, h * 0.3, w * 0.56, h * 0.3), scale * wobble);
        if r.w < 2.0 {
            return;
        }
        art::glow(r, ctx.time);
        art::card(r, WHITE);

        // "Level 3 done!" in rainbow letters.
        let title = format!("Level {} done!", self.done);
        let size = r.h * 0.3;
        let widths: Vec<f32> = title.chars().map(|c| measure_text(&c.to_string(), font, size as u16, 1.0).width).collect();
        let total: f32 = widths.iter().sum();
        let mut x = r.center().x - total / 2.0;
        for (i, (c, cw)) in title.chars().zip(&widths).enumerate() {
            let color = art::readable(Paint::ALL[i % Paint::ALL.len()].color());
            let hop = (ctx.time * 5.0 - i as f32 * 0.4).sin().max(0.0) * size * 0.08;
            art::text_center(font, &c.to_string(), vec2(x + cw / 2.0, r.y + r.h * 0.38 - hop), size, color);
            x += cw;
        }
        let next = if self.last { "You finished them all!".to_string() } else { format!("On to level {}!", self.done + 1) };
        art::text_center(font, &next, vec2(r.center().x, r.y + r.h * 0.74), r.h * 0.16, art::INK);

        // A star on each side.
        for side in [-1.0, 1.0] {
            let c = vec2(r.center().x + side * r.w * 0.44, r.center().y);
            art::star(c, r.h * 0.16, art::darken(art::GOLD, 0.25));
            art::star(c, r.h * 0.14, art::GOLD);
        }
    }
}
