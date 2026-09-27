//! The mini-games. Each one implements the `MiniGame` trait so `main` can treat them all alike.

pub mod color_sort;
pub mod counting;
pub mod letters;
pub mod pattern;

use crate::ctx::Ctx;
use macroquad::rand;

/// What every mini-game must be able to do.
pub trait MiniGame {
    /// Called when the game is picked from the menu. Starts a fresh round.
    fn enter(&mut self, ctx: &mut Ctx);
    /// The spoken question for the current round ("Find the letter bee!").
    fn prompt(&self) -> String;
    fn update(&mut self, ctx: &mut Ctx);
    fn draw(&self, ctx: &Ctx);
    fn progress(&mut self) -> &mut Progress;
}

/// Right answers in a row (on the first try) needed to move up a level.
const STREAK_TO_LEVEL_UP: u32 = 4;

/// Tracks difficulty. Levels only ever go up on their own; a parent can use the
/// arrow keys to move them up or down.
pub struct Progress {
    pub level: u32,
    pub max: u32,
    streak: u32,
}

impl Progress {
    pub fn new(max: u32) -> Self {
        Progress { level: 1, max, streak: 0 }
    }

    /// Record a right answer. Returns true if that earned a level up.
    pub fn record(&mut self, first_try: bool) -> bool {
        if !first_try {
            self.streak = 0;
            return false;
        }
        self.streak += 1;
        if self.streak >= STREAK_TO_LEVEL_UP && self.level < self.max {
            self.level += 1;
            self.streak = 0;
            true
        } else {
            false
        }
    }

    pub fn set_level(&mut self, level: u32) {
        self.level = level.clamp(1, self.max);
        self.streak = 0;
    }
}

/// Is the game waiting for an answer, or celebrating one (with seconds left)?
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Playing,
    Celebrating(f32),
}

/// Longest we'll wait for the voice after a celebration's minimum time (safety net).
const MAX_VOICE_WAIT: f32 = 8.0;
/// Longest we'll wait for the voice to say one word when stepping through words
/// (chanting a pattern, counting together) before moving on anyway.
pub const STEP_TIMEOUT: f32 = 4.0;

/// A celebration ends once its minimum time `t` has run out AND the voice has finished
/// its praise, so the next round never appears while we're still talking about the last one.
/// `t` keeps counting down below zero while we wait.
pub fn celebration_over(t: f32, ctx: &Ctx) -> bool {
    t <= 0.0 && (!ctx.voice.busy() || t < -MAX_VOICE_WAIT)
}

/// Pick a random item from a list.
pub fn pick<T: Copy>(items: &[T]) -> T {
    items[rand::gen_range(0, items.len())]
}

/// Shuffle a list in place (Fisher-Yates).
pub fn shuffle<T>(items: &mut [T]) {
    for i in (1..items.len()).rev() {
        let j = rand::gen_range(0, i + 1);
        items.swap(i, j);
    }
}

/// Fade each value toward zero, e.g. wiggle timers.
pub fn fade(values: &mut [f32], dt: f32, speed: f32) {
    for v in values {
        *v = (*v - dt * speed).max(0.0);
    }
}

/// Lay out `n` equal cards in a centered row. `max_w` is the fraction of screen width to use,
/// `max_h` caps the card size as a fraction of screen height, `cy` is the row's center (fraction of height).
pub fn row_of_cards(n: usize, max_w: f32, max_h: f32, cy: f32) -> Vec<macroquad::prelude::Rect> {
    use macroquad::prelude::*;
    let (w, h) = (screen_width(), screen_height());
    let cell = (w * max_w / n as f32).min(h * max_h);
    let size = cell * 0.86;
    let x0 = (w - cell * n as f32) / 2.0;
    (0..n)
        .map(|i| {
            Rect::new(
                x0 + cell * i as f32 + (cell - size) / 2.0,
                h * cy - size / 2.0,
                size,
                size,
            )
        })
        .collect()
}
