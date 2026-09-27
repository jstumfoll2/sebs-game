//! Buttons and counters drawn on top of every game: home, "say it again", and the star counter.

use crate::art;
use crate::ctx::Ctx;
use macroquad::prelude::*;

fn button_size() -> f32 {
    (screen_height() * 0.1).clamp(56.0, 110.0)
}

pub fn home_rect() -> Rect {
    let s = button_size();
    Rect::new(s * 0.25, s * 0.25, s, s)
}

pub fn repeat_rect() -> Rect {
    let s = button_size();
    Rect::new(s * 1.5, s * 0.25, s, s)
}

pub fn draw_game_buttons(ctx: &Ctx, level: u32) {
    let home = home_rect();
    art::round_button(home.center(), home.w / 2.0);
    art::home_icon(home.center(), home.w / 2.0);

    let rep = repeat_rect();
    art::round_button(rep.center(), rep.w / 2.0);
    art::speaker_icon(rep.center(), rep.w / 2.0, art::Paint::Blue.color());

    // Small level readout for grown-ups (arrow keys change it).
    let s = button_size();
    art::text_left(
        ctx.font(),
        &format!("level {level}"),
        s * 0.3,
        screen_height() - s * 0.3,
        s * 0.25,
        art::SHADOW,
    );
}

pub fn draw_stars(ctx: &Ctx) {
    let s = button_size();
    let c = vec2(screen_width() - s * 1.9, s * 0.75);
    let r = s * 0.45 * (1.0 + ctx.star_pop * 0.5);
    art::star(c, r * 1.1, art::darken(art::GOLD, 0.25));
    art::star(c, r, art::GOLD);
    art::text_left(ctx.font(), &ctx.stars.to_string(), c.x + s * 0.6, c.y, s * 0.7, art::INK);
}
