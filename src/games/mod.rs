//! The mini-games. Each one implements the `MiniGame` trait so `main` can treat them all alike.

pub mod color_sort;
pub mod counting;
pub mod groups;
pub mod letters;
pub mod pattern;
pub mod puzzle;
pub mod shadows;

use crate::ctx::Ctx;
use macroquad::prelude::{Rect, Vec2};
use macroquad::rand;

/// What every mini-game must be able to do.
pub trait MiniGame {
    /// Called when the game is picked from the menu. Starts a fresh round.
    fn enter(&mut self, ctx: &mut Ctx);
    /// The spoken question for the current round ("Find the letter bee!").
    fn prompt(&self) -> String;
    fn update(&mut self, ctx: &mut Ctx);
    fn draw(&self, ctx: &Ctx);
    fn progress(&self) -> &Progress;
    fn progress_mut(&mut self) -> &mut Progress;
    /// What a level means, shown under its number in the level picker,
    /// e.g. "5-10" for counting or "3 colors" for color sorting.
    fn level_label(&self, level: u32) -> String;
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

// ---------- dragging answer cards ----------

/// What happened with a card drag this frame.
pub enum DragEvent {
    Nothing,
    /// A card was touched (say its name!).
    PickedUp(usize),
    /// A card was let go, with its center at this point (did it land on the target?).
    Dropped(usize, Vec2),
}

/// Lets the player drag one of a row of cards. Cards that are let go slide back home.
#[derive(Default)]
pub struct CardDrag {
    held: Option<usize>,
    grab: Vec2,
    /// How far each card has been moved away from its home spot.
    offsets: Vec<Vec2>,
}

impl CardDrag {
    pub fn reset(&mut self) {
        self.held = None;
        self.offsets.clear();
    }

    pub fn held(&self) -> Option<usize> {
        self.held
    }

    /// `homes` are where the cards normally sit.
    pub fn update(&mut self, input: &crate::input::Input, homes: &[Rect], dt: f32) -> DragEvent {
        use macroquad::prelude::*;
        if self.offsets.len() != homes.len() {
            self.offsets = vec![Vec2::ZERO; homes.len()];
            self.held = None;
        }
        // Let-go cards glide back home.
        for (i, o) in self.offsets.iter_mut().enumerate() {
            if self.held != Some(i) {
                *o = o.lerp(Vec2::ZERO, (dt * 10.0).min(1.0));
            }
        }
        if input.pressed && self.held.is_none() {
            if let Some(i) = (0..homes.len()).find(|&i| self.rect(i, homes[i]).contains(input.pos)) {
                self.held = Some(i);
                self.grab = homes[i].center() + self.offsets[i] - input.pos;
                return DragEvent::PickedUp(i);
            }
        }
        if let Some(i) = self.held {
            if input.down {
                self.offsets[i] = input.pos + self.grab - homes[i].center();
            } else {
                self.held = None;
                return DragEvent::Dropped(i, homes[i].center() + self.offsets[i]);
            }
        }
        DragEvent::Nothing
    }

    /// Where card `i` is right now (a held card is drawn a bit bigger).
    pub fn rect(&self, i: usize, home: Rect) -> Rect {
        let off = self.offsets.get(i).copied().unwrap_or_default();
        let r = Rect::new(home.x + off.x, home.y + off.y, home.w, home.h);
        if self.held == Some(i) {
            crate::art::scale_rect(r, 1.1)
        } else {
            r
        }
    }
}

// ---------- the "how to play" demo ----------

/// Seconds for one run of the demo motion.
const DEMO_CYCLE: f32 = 1.8;

/// At the start of a game, a cartoon hand shows how to drag. Any touch stops it.
#[derive(Default)]
pub struct Demo {
    total: f32,
    left: f32,
}

impl Demo {
    pub fn start(&mut self, times: f32) {
        self.total = DEMO_CYCLE * times;
        self.left = self.total;
    }

    pub fn update(&mut self, ctx: &Ctx) {
        self.left = if ctx.input.pressed { 0.0 } else { (self.left - ctx.dt).max(0.0) };
    }

    pub fn active(&self) -> bool {
        self.left > 0.0
    }

    /// How far through the current run we are (0 to 1).
    fn phase(&self) -> f32 {
        ((self.total - self.left) % DEMO_CYCLE) / DEMO_CYCLE
    }

    /// Where something being carried by the hand is (None when the hand isn't carrying).
    pub fn carry(&self, from: Vec2, to: Vec2) -> Option<Vec2> {
        if !self.active() {
            return None;
        }
        let p = self.phase();
        if !(0.1..0.85).contains(&p) {
            return None;
        }
        // Ease in and out so the motion looks natural.
        let t = ((p - 0.15) / 0.55).clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        Some(from.lerp(to, eased))
    }

    /// Like `draw`, but the hand carries a blank card of the given size (so it looks like
    /// dragging a card, without giving away which one is right).
    pub fn draw_with_card(&self, from: Vec2, to: Vec2, size: f32) {
        if let Some(c) = self.carry(from, to) {
            let r = Rect::new(c.x - size / 2.0, c.y - size / 2.0, size, size);
            crate::art::card(r, macroquad::prelude::WHITE);
        }
        self.draw(from, to);
    }

    pub fn draw(&self, from: Vec2, to: Vec2) {
        if !self.active() {
            return;
        }
        let p = self.phase();
        let (pos, pressing) = match self.carry(from, to) {
            Some(pos) => (pos, true),
            None if p < 0.1 => (from, false),
            None => return, // resting between runs
        };
        crate::art::hand(pos, macroquad::prelude::screen_height() * 0.07, pressing);
    }
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
