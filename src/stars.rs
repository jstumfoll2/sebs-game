//! The star panel: tap the star counter to see every star earned, and count them together.

use crate::alphabet::capitalize;
use crate::art;
use crate::ctx::Ctx;
use crate::games::counting::NUMBER_WORDS;
use crate::hud;
use macroquad::prelude::*;

/// Seconds (at least) between stars while counting them out loud.
const STEP: f32 = 0.35;
/// Most stars we'll count out loud; above this we just show them all.
const COUNT_ALOUD_MAX: usize = 20;
/// Most stars we'll draw (after that the number says it all).
const MAX_SHOWN: usize = 100;

#[derive(Default)]
pub struct StarPanel {
    is_open: bool,
    /// How many stars are showing so far (they pop in one at a time while counting).
    shown: usize,
    counting: bool,
    step_time: f32,
    /// Makes the newest star pop.
    pop: f32,
}

impl StarPanel {
    /// Returns true if the panel is open (so the game underneath should ignore input).
    pub fn update(&mut self, ctx: &mut Ctx) -> bool {
        if !self.is_open {
            if ctx.input.tapped(hud::star_rect()) {
                self.open(ctx);
                return true;
            }
            return false;
        }

        let total = ctx.stars as usize;
        self.pop = (self.pop - ctx.dt * 3.0).max(0.0);
        if self.counting {
            self.step_time += ctx.dt;
            if self.step_time >= STEP && !ctx.voice.busy() {
                self.step_time = 0.0;
                if self.shown < total {
                    self.shown += 1;
                    self.pop = 1.0;
                    ctx.sfx.pop();
                    ctx.voice.say(NUMBER_WORDS[self.shown]);
                } else {
                    self.counting = false;
                    let word = if total == 1 { "star" } else { "stars" };
                    ctx.voice.say(&format!("{} {word}! Great job!", capitalize(NUMBER_WORDS[total])));
                }
            }
        }

        // Any tap closes the panel.
        if ctx.input.pressed {
            self.is_open = false;
            ctx.sfx.pop();
            ctx.voice.say("");
        }
        true
    }

    /// Open the panel and start counting (or just say the total).
    pub fn open(&mut self, ctx: &mut Ctx) {
        let total = ctx.stars as usize;
        ctx.sfx.pop();
        self.is_open = true;
        self.step_time = 0.0;
        if total == 0 {
            self.shown = 0;
            self.counting = false;
            ctx.voice.say("No stars yet! Play a game to earn stars.");
        } else if total <= COUNT_ALOUD_MAX {
            self.shown = 0;
            self.counting = true;
            ctx.voice.say("Let's count your stars!");
        } else {
            self.shown = total;
            self.counting = false;
            ctx.voice.say(&format!("You have {total} stars! Wow!"));
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        if !self.is_open {
            return;
        }
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.1, 0.1, 0.2, 0.45));

        let panel = Rect::new(w * 0.08, h * 0.1, w * 0.84, h * 0.8);
        art::card(panel, WHITE);

        // Title: the number, big.
        let n = if self.counting { self.shown } else { ctx.stars as usize };
        let title = if n == 1 { "1 star".to_string() } else { format!("{n} stars") };
        art::text_center(font, &title, vec2(panel.center().x, panel.y + panel.h * 0.13), panel.h * 0.13, art::INK);

        // The stars, in rows of ten (with a little gap after every five).
        let shown = self.shown.min(MAX_SHOWN);
        let rows = shown.div_ceil(10).max(1);
        let area = Rect::new(panel.x + panel.w * 0.05, panel.y + panel.h * 0.26, panel.w * 0.9, panel.h * 0.68);
        let cell = (area.w / 10.5).min(area.h / rows as f32);
        let top = area.y + (area.h - cell * rows as f32) / 2.0;
        for i in 0..shown {
            let (row, col) = (i / 10, i % 10);
            let extra_gap = if col >= 5 { cell * 0.5 } else { 0.0 };
            let x = area.x + (area.w - cell * 10.5) / 2.0 + cell * (col as f32 + 0.5) + extra_gap;
            let y = top + cell * (row as f32 + 0.5);
            let newest = i + 1 == self.shown && self.counting;
            let r = cell * 0.42 * if newest { 1.0 + self.pop * 0.5 } else { 1.0 };
            art::star(vec2(x, y), r * 1.1, art::darken(art::GOLD, 0.25));
            art::star(vec2(x, y), r, art::GOLD);
        }
    }
}
