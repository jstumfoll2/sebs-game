//! Setting up a two-player match: pick two players, then how many stars wins.

use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::hud;
use crate::players::Players;
use macroquad::prelude::*;

/// The goals to choose from: first to this many stars wins.
pub const GOALS: [u32; 4] = [5, 10, 15, 20];

pub enum Setup {
    Nothing,
    Back,
    /// Start a match between these two players, first to `goal` stars.
    Start { first: usize, second: usize, goal: u32 },
}

#[derive(Default)]
pub struct VersusSetup {
    /// The players picked so far, in order (player 1, player 2).
    picked: Vec<usize>,
}

/// One card per player, in a centered grid (like "Who's playing?", without the "+").
fn cards(n: usize) -> Vec<Rect> {
    let (w, h) = (screen_width(), screen_height());
    let cols = n.clamp(1, 5);
    let rows = n.div_ceil(cols);
    let cell = (w * 0.84 / cols as f32).min(h * 0.4 / rows as f32).min(h * 0.3);
    let size = cell * 0.86;
    let x0 = (w - cell * cols as f32) / 2.0;
    (0..n)
        .map(|i| {
            let (col, row) = ((i % cols) as f32, (i / cols) as f32);
            let in_row = if i / cols == rows - 1 { n - (rows - 1) * cols } else { cols };
            let shift = (cols - in_row) as f32 * cell / 2.0;
            Rect::new(x0 + shift + col * cell + (cell - size) / 2.0, h * 0.24 + row * cell, size, size)
        })
        .collect()
}

fn goal_rects() -> Vec<Rect> {
    crate::games::row_of_cards(GOALS.len(), 0.6, 0.2, 0.8)
}

impl VersusSetup {
    pub fn start(&mut self, ctx: &mut Ctx) {
        self.picked.clear();
        ctx.voice.say("Two players! Pick who's playing.");
    }

    pub fn update(&mut self, players: &Players, ctx: &mut Ctx) -> Setup {
        if ctx.input.tapped(hud::home_rect()) {
            ctx.sfx.pop();
            return Setup::Back;
        }
        if !ctx.input.pressed {
            return Setup::Nothing;
        }
        let pos = ctx.input.pos;
        if let Some(i) = cards(players.players.len()).iter().position(|r| r.contains(pos)) {
            ctx.sfx.pop();
            if let Some(k) = self.picked.iter().position(|&p| p == i) {
                self.picked.remove(k); // tap again to un-pick
            } else if self.picked.len() < 2 {
                self.picked.push(i);
                let name = &players.players[i].name;
                if self.picked.len() == 1 {
                    ctx.voice.say(&format!("{name}! Who else?"));
                } else {
                    let first = &players.players[self.picked[0]].name;
                    ctx.voice.say(&format!("{first} and {name}! First to how many stars?"));
                }
            }
            return Setup::Nothing;
        }
        if self.picked.len() == 2 {
            if let Some(g) = goal_rects().iter().position(|r| r.contains(pos)) {
                ctx.sfx.pop();
                return Setup::Start { first: self.picked[0], second: self.picked[1], goal: GOALS[g] };
            }
        }
        Setup::Nothing
    }

    pub fn draw(&self, players: &Players, ctx: &Ctx) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        let heading = if self.picked.len() < 2 { "Pick two players" } else { "First to how many stars?" };
        art::text_center(font, heading, vec2(w / 2.0, h * 0.14), h * 0.08, art::INK);

        for (i, r) in cards(players.players.len()).iter().enumerate() {
            let p = &players.players[i];
            let order = self.picked.iter().position(|&x| x == i);
            let mut r = *r;
            if order.is_some() {
                r = art::scale_rect(r, 1.06);
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            let avatar = vec2(r.center().x, r.y + r.h * 0.4);
            draw_circle(avatar.x, avatar.y, r.w * 0.26, Paint::ALL[i % Paint::ALL.len()].color());
            let initial: String = p.name.chars().take(1).collect();
            art::text_center(font, &initial, avatar, r.w * 0.32, WHITE);
            art::text_center(font, &p.name, vec2(r.center().x, r.y + r.h * 0.82), r.h * 0.14, art::INK);
            // "1" and "2" badges show who goes first.
            if let Some(k) = order {
                art::badge(font, vec2(r.x + r.w * 0.88, r.y + r.w * 0.12), r.w * 0.13, &(k + 1).to_string());
            }
        }

        if self.picked.len() == 2 {
            for (r, goal) in goal_rects().iter().zip(GOALS) {
                let bob = (ctx.time * 2.5 + goal as f32).sin() * r.h * 0.03;
                let r = Rect::new(r.x, r.y + bob, r.w, r.h);
                art::card(r, WHITE);
                let c = vec2(r.center().x, r.y + r.h * 0.42);
                art::star(c, r.w * 0.34, art::GOLD);
                art::text_center(font, &goal.to_string(), c + vec2(0.0, r.w * 0.04), r.w * 0.26, art::INK);
                art::text_center(font, "stars", vec2(r.center().x, r.y + r.h * 0.85), r.h * 0.14, art::INK);
            }
        }
        hud::draw_home_button(ctx);
    }
}
