//! Buttons and counters drawn on top of the screens: home, "say it again", the speaker
//! (sound on/off), and the star counter.

use crate::art;
use crate::ctx::Ctx;
use macroquad::prelude::*;

/// Which top button, as an index into `ctx.button_pop`.
pub const HOME: usize = 0;
pub const REPEAT: usize = 1;
pub const MUTE: usize = 2;

fn button_size() -> f32 {
    (screen_height() * 0.1).clamp(56.0, 110.0)
}

pub fn home_rect() -> Rect {
    let s = button_size();
    Rect::new(s * 0.25, s * 0.25, s, s)
}

/// "Say the question again" (speech bubble), next to home.
pub fn repeat_rect() -> Rect {
    let s = button_size();
    Rect::new(s * 1.5, s * 0.25, s, s)
}

/// Sound on/off (speaker), in the top-right corner.
pub fn mute_rect() -> Rect {
    let s = button_size();
    Rect::new(screen_width() - s * 1.25, s * 0.25, s, s)
}

/// A round white button that bounces for a moment after being tapped.
fn button(r: Rect, pop: f32) -> (Vec2, f32) {
    let radius = r.w / 2.0 * (1.0 + 0.25 * (pop * std::f32::consts::PI).sin());
    art::round_button(r.center(), radius);
    (r.center(), radius)
}

pub fn draw_home_button(ctx: &Ctx) {
    let (c, r) = button(home_rect(), ctx.button_pop[HOME]);
    art::home_icon(c, r);
}

/// On the menu, the top-left button switches player instead of going home.
pub fn draw_players_button(ctx: &Ctx) {
    let (c, r) = button(home_rect(), ctx.button_pop[HOME]);
    art::people_icon(c, r);
}

/// `level` is shown small in the corner for grown-ups (None for games without levels).
pub fn draw_game_buttons(ctx: &Ctx, level: Option<u32>) {
    draw_home_button(ctx);

    let (c, r) = button(repeat_rect(), ctx.button_pop[REPEAT]);
    art::question_bubble_icon(ctx.font(), c, r, art::Paint::Blue.color());

    // Small level readout for grown-ups (arrow keys change it).
    if let Some(level) = level {
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
}

/// The speaker button shows on every screen.
pub fn draw_mute_button(ctx: &Ctx) {
    let (c, r) = button(mute_rect(), ctx.button_pop[MUTE]);
    if ctx.muted {
        art::muted_speaker_icon(c, r, art::INK);
    } else {
        art::speaker_icon(c, r, art::Paint::Blue.color());
    }
}

fn star_center() -> Vec2 {
    let s = button_size();
    vec2(screen_width() - s * 3.6, s * 0.75)
}

/// The tappable area around the star counter (star plus number).
pub fn star_rect() -> Rect {
    let (s, c) = (button_size(), star_center());
    Rect::new(c.x - s * 0.7, 0.0, s * 2.3, s * 1.5)
}

/// In a two-player match: both players' scores. Whoever's turn it is glows and is bigger.
pub fn draw_scores(ctx: &Ctx, m: &crate::ctx::Match) {
    let font = ctx.font();
    let (w, s) = (screen_width(), button_size());
    for k in 0..2 {
        let mine = m.turn == k;
        let c = vec2(w * 0.78 + k as f32 * w * 0.09, s * 0.75);
        let scale = if mine { 1.1 + 0.04 * (ctx.time * 4.0).sin() } else { 0.9 };
        let pill = art::scale_rect(Rect::new(c.x - s * 0.7, c.y - s * 0.42, s * 1.4, s * 0.84), scale);
        if mine {
            art::glow(pill, ctx.time);
        }
        art::card(pill, WHITE);
        let a = vec2(pill.x + pill.h * 0.5, pill.center().y);
        draw_circle(a.x, a.y, pill.h * 0.36, art::Paint::ALL[m.colors[k] % art::Paint::ALL.len()].color());
        let initial: String = m.names[k].chars().take(1).collect();
        art::text_center(font, &initial, a, pill.h * 0.45, WHITE);
        art::star(vec2(pill.x + pill.w * 0.58, pill.center().y), pill.h * 0.2, art::GOLD);
        art::text_center(font, &m.scores[k].to_string(), vec2(pill.x + pill.w * 0.82, pill.center().y), pill.h * 0.5, art::INK);
    }
}

pub fn draw_stars(ctx: &Ctx) {
    let s = button_size();
    let c = star_center();
    let r = s * 0.45 * (1.0 + ctx.star_pop * 0.5);
    art::star(c, r * 1.1, art::darken(art::GOLD, 0.25));
    art::star(c, r, art::GOLD);
    art::text_left(ctx.font(), &ctx.stars.to_string(), c.x + s * 0.6, c.y, s * 0.7, art::INK);
}
