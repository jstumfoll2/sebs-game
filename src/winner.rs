//! The winner screen: fireworks, a crown, and the stars side by side.
//! In a two-player match it compares the two scores ("10 is more than 7!"), a little
//! lesson about bigger and smaller numbers. Solo players see it every 50 stars.

use crate::art::{self, Paint};
use crate::ctx::{Ctx, Win};
use crate::games::counting::number_word;
use crate::pictures::{self, Picture};
use macroquad::prelude::*;

/// Can't be tapped away until this many seconds (so it isn't skipped by accident).
const MIN_SECS: f32 = 3.0;

pub struct Winner {
    win: Win,
    age: f32,
}

impl Winner {
    /// Start the celebration: fireworks, a fanfare, and the voice.
    pub fn new(win: Win, ctx: &mut Ctx) -> Self {
        ctx.fireworks.show(8.0);
        ctx.sfx.tada();
        let words = match &win {
            Win::Solo { name, stars } => {
                format!("Hooray, {name}! You got {} stars! You're a superstar!", number_word(*stars as usize))
            }
            Win::Versus { winner, winner_stars, other, other_stars, .. } => {
                let (w, o) = (number_word(*winner_stars as usize), number_word(*other_stars as usize));
                format!(
                    "{winner} wins! {winner} has {w} stars. {other} has {o} stars. \
                     {} is more than {o}! Great game, both of you!",
                    crate::alphabet::capitalize(&w)
                )
            }
        };
        ctx.voice.say(&words);
        Winner { win, age: 0.0 }
    }

    pub fn is_versus(&self) -> bool {
        matches!(self.win, Win::Versus { .. })
    }

    /// Returns true when it's time to leave the winner screen (a tap, after a few seconds).
    pub fn update(&mut self, ctx: &mut Ctx) -> bool {
        self.age += ctx.dt;
        if self.age > MIN_SECS && ctx.input.pressed {
            ctx.fireworks.stop();
            return true;
        }
        false
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        // A darker sky so the fireworks shine.
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.08, 0.1, 0.25, 0.75));

        let title = match &self.win {
            Win::Solo { name, stars } => format!("{name}: {stars} stars!"),
            Win::Versus { winner, .. } => format!("{winner} wins!"),
        };
        rainbow_title(ctx, &title, vec2(w / 2.0, h * 0.13), h * 0.11);

        match &self.win {
            Win::Solo { stars, .. } => self.draw_solo(ctx, *stars),
            Win::Versus { winner, winner_stars, other, other_stars, colors } => {
                let rows = [(winner, *winner_stars, colors[0], true), (other, *other_stars, colors[1], false)];
                for (k, (name, stars, color, crowned)) in rows.iter().enumerate() {
                    draw_player_row(ctx, name, *stars, *color, *crowned, h * (0.33 + k as f32 * 0.2));
                }
                // The lesson: which number is bigger?
                let cmp = format!("{winner_stars}  >  {other_stars}");
                let pulse = 1.0 + 0.05 * (ctx.time * 4.0).sin();
                art::text_center_zoomed(font, &cmp, vec2(w / 2.0, h * 0.76), h * 0.12, pulse, art::GOLD);
                let words = format!("{winner_stars} is more than {other_stars}!");
                art::text_center(font, &words, vec2(w / 2.0, h * 0.86), h * 0.055, WHITE);
            }
        }

        if self.age > MIN_SECS {
            let blink = 0.6 + 0.4 * (ctx.time * 3.0).sin();
            art::text_center(font, "Tap to keep playing", vec2(w / 2.0, h * 0.95), h * 0.035, Color::new(1.0, 1.0, 1.0, blink));
        }
    }

    fn draw_solo(&self, ctx: &Ctx, stars: u32) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        // A big spinning-ish star with the number.
        let c = vec2(w / 2.0, h * 0.4);
        let r = h * 0.17 * (1.0 + 0.04 * (ctx.time * 3.0).sin());
        art::star(c, r * 1.08, art::darken(art::GOLD, 0.3));
        art::star(c, r, art::GOLD);
        art::text_center(font, &stars.to_string(), c + vec2(0.0, r * 0.12), r * 0.6, art::INK);
        // All the stars, in rows of ten (a gap after each five), so you can see how many.
        let shown = stars.min(100) as usize;
        let cell = (w * 0.6 / 10.5).min(h * 0.26 / shown.div_ceil(10).max(1) as f32);
        let top = h * 0.63;
        for i in 0..shown {
            let (row, col) = (i / 10, i % 10);
            let gap = if col >= 5 { cell * 0.5 } else { 0.0 };
            let x = w / 2.0 - cell * 5.25 + cell * (col as f32 + 0.5) + gap;
            let y = top + cell * (row as f32 + 0.5);
            art::star(vec2(x, y), cell * 0.42, art::GOLD);
        }
    }
}

/// One player's row: avatar (with a crown for the winner), name, their stars, and the number.
fn draw_player_row(ctx: &Ctx, name: &str, stars: u32, color: usize, crowned: bool, y: f32) {
    let font = ctx.font();
    let (w, h) = (screen_width(), screen_height());
    let avatar = vec2(w * 0.14, y);
    let rad = h * 0.07;
    draw_circle(avatar.x, avatar.y, rad, Paint::ALL[color % Paint::ALL.len()].color());
    let initial: String = name.chars().take(1).collect();
    art::text_center(font, &initial, avatar, rad * 1.2, WHITE);
    if crowned {
        let bob = (ctx.time * 3.0).sin() * rad * 0.08;
        pictures::draw(Picture::Crown, avatar - vec2(0.0, rad * 1.35 + bob), rad * 0.75);
    }
    art::text_left(font, name, w * 0.22, y - h * 0.045, h * 0.05, WHITE);

    // The stars, in groups of five.
    let cell = (w * 0.5 / 21.0).min(h * 0.05);
    for i in 0..stars.min(40) as usize {
        let gap = (i / 5) as f32 * cell * 0.5;
        let x = w * 0.22 + cell * (i as f32 + 0.5) + gap;
        art::star(vec2(x, y + h * 0.035), cell * 0.45, art::GOLD);
    }
    art::text_center(font, &stars.to_string(), vec2(w * 0.88, y), h * 0.09, art::GOLD);
}

/// Big bouncing rainbow letters.
fn rainbow_title(ctx: &Ctx, text: &str, c: Vec2, size: f32) {
    let font = ctx.font();
    let widths: Vec<f32> = text.chars().map(|ch| art::measure(font, &ch.to_string(), size).width).collect();
    let total: f32 = widths.iter().sum();
    let mut x = c.x - total / 2.0;
    for (i, (ch, cw)) in text.chars().zip(&widths).enumerate() {
        let hop = (ctx.time * 5.0 - i as f32 * 0.5).sin().max(0.0) * size * 0.15;
        let color = Paint::ALL[i % Paint::ALL.len()].color();
        let pos = vec2(x + cw / 2.0, c.y - hop);
        art::text_center(font, &ch.to_string(), pos + vec2(0.0, size * 0.05), size, art::darken(color, 0.5));
        art::text_center(font, &ch.to_string(), pos, size, color);
        x += cw;
    }
}
