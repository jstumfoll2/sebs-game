//! Pattern finish: red, blue, red, blue, ... what comes next?
//! Starts with simple AB color patterns and grows to ABC, AAB, ABB and AABB.

use super::{
    celebration_over, fade, pick, row_of_cards, shuffle, MiniGame, Phase, Progress, STEP_TIMEOUT,
};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use macroquad::prelude::*;

/// One item in a pattern.
type Item = (Thing, Paint);

/// Pattern "shapes": each number is which item goes there. [0, 1] means A B A B...
const UNITS: [&[usize]; 5] = [&[0, 1], &[0, 0, 1], &[0, 1, 2], &[0, 1, 1], &[0, 0, 1, 1]];
const MAX_SHOWN: usize = 9;

/// What changes between items in the pattern.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Vary {
    Color,
    Shape,
    Both,
}

pub struct Pattern {
    progress: Progress,
    vary: Vary,
    shown: Vec<Item>,
    answer: Item,
    choices: Vec<Item>,
    first_try: bool,
    misses: u32,
    shake: Vec<f32>,
    phase: Phase,
    chant: Chant,
}

/// After a right answer we "chant" the whole pattern, one item at a time,
/// lighting up each item while the voice says it.
#[derive(Clone, Copy, Debug, Default)]
struct Chant {
    /// Next item to say (the answer slot is index `shown.len()`).
    next: usize,
    /// Item currently lit up.
    lit: Option<usize>,
    /// Seconds on the current item.
    time: f32,
    done: bool,
}

/// Minimum seconds per item while chanting (so it has a steady rhythm).
const CHANT_STEP: f32 = 0.35;

impl Pattern {
    pub fn new() -> Self {
        let mut game = Pattern {
            progress: Progress::new(4),
            vary: Vary::Color,
            shown: Vec::new(),
            answer: (Thing::Ball, Paint::Red),
            choices: Vec::new(),
            first_try: true,
            misses: 0,
            shake: Vec::new(),
            phase: Phase::Playing,
            chant: Chant::default(),
        };
        game.new_round();
        game
    }

    fn new_round(&mut self) {
        let level = self.progress.level;
        let unit_count = match level {
            1 | 2 => 1, // just A B
            3 => 3,     // + A A B, A B C
            _ => UNITS.len(),
        };
        let unit = UNITS[rand::gen_range(0, unit_count)];
        self.vary = match level {
            1 => Vary::Color,
            2 => Vary::Both,
            _ => pick(&[Vary::Color, Vary::Shape, Vary::Both]),
        };

        // Make the distinct items (A, B, C...).
        let kinds = unit.iter().max().unwrap() + 1;
        let mut things = Thing::ALL.to_vec();
        let mut paints = Paint::ALL.to_vec();
        shuffle(&mut things);
        shuffle(&mut paints);
        let items: Vec<Item> = (0..kinds)
            .map(|i| match self.vary {
                Vary::Color => (things[0], paints[i]),
                Vary::Shape => (things[i], paints[0]),
                Vary::Both => (things[i], paints[i]),
            })
            .collect();

        // Show at least two full repeats, then ask for the next one.
        let len = (unit.len() * 2 + rand::gen_range(0, unit.len())).min(MAX_SHOWN);
        self.shown = (0..len).map(|i| items[unit[i % unit.len()]]).collect();
        self.answer = items[unit[len % unit.len()]];

        // Choices: every item in the pattern, plus an extra "wrong" one on harder levels.
        let want = if level == 1 { 2 } else { 3 };
        self.choices = items.clone();
        while self.choices.len() < want {
            let extra = match self.vary {
                Vary::Color => (things[0], pick(&Paint::ALL)),
                Vary::Shape => (pick(&Thing::ALL), paints[0]),
                Vary::Both => (pick(&Thing::ALL), pick(&Paint::ALL)),
            };
            if !self.choices.contains(&extra) {
                self.choices.push(extra);
            }
        }
        shuffle(&mut self.choices);

        self.first_try = true;
        self.misses = 0;
        self.shake = vec![0.0; self.choices.len()];
        self.phase = Phase::Playing;
    }

    fn word(&self, item: Item) -> String {
        match self.vary {
            Vary::Color => item.1.name().to_string(),
            Vary::Shape => item.0.name().to_string(),
            Vary::Both => format!("{} {}", item.1.name(), item.0.name()),
        }
    }

    /// Item `i` of the finished pattern (the last one is the answer).
    fn item(&self, i: usize) -> Item {
        self.shown.get(i).copied().unwrap_or(self.answer)
    }

    /// "red... blue... red... blue!" Say each item once the voice has finished the last one,
    /// so the lit-up item always matches the word being spoken.
    fn update_chant(&mut self, ctx: &mut Ctx) {
        let total = self.shown.len() + 1;
        self.chant.time += ctx.dt;
        let voice_done = !ctx.voice.busy() || self.chant.time > STEP_TIMEOUT;
        if self.chant.time < CHANT_STEP || !voice_done {
            return;
        }
        if self.chant.next < total {
            let word = self.word(self.item(self.chant.next));
            ctx.voice.then(&word);
            self.chant.lit = Some(self.chant.next);
            self.chant.next += 1;
            self.chant.time = 0.0;
        } else {
            self.chant.lit = None;
            self.chant.done = true;
            self.phase = Phase::Celebrating(0.6); // short pause before the next round
        }
    }

    /// An item on its card, with its label underneath ("RED", "STAR" or "RED STAR").
    fn draw_item(&self, ctx: &Ctx, r: Rect, item: Item, bob: f32) {
        let pic = vec2(r.center().x, r.y + r.h * 0.4 + bob);
        art::draw_thing(item.0, pic, r.w * 0.27, item.1.color());
        let label = vec2(r.center().x, r.y + r.h * 0.84);
        let first = art::readable(item.1.color());
        art::word_label(ctx.font(), &self.word(item), label, r.h * 0.12, r.w * 0.9, first, None);
    }

    fn slot_rects(&self) -> Vec<Rect> {
        row_of_cards(self.shown.len() + 1, 0.94, 0.2, 0.34)
    }

    fn choice_rects(&self) -> Vec<Rect> {
        row_of_cards(self.choices.len(), 0.7, 0.26, 0.74)
    }
}

impl MiniGame for Pattern {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        ctx.voice.then(&self.prompt());
    }

    fn prompt(&self) -> String {
        "What comes next?".to_string()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.shake, ctx.dt, 2.5);
        match self.phase {
            Phase::Celebrating(_) if !self.chant.done => self.update_chant(ctx),
            Phase::Celebrating(t) => {
                let t = t - ctx.dt;
                if celebration_over(t, ctx) {
                    self.new_round();
                    ctx.voice.then(&self.prompt());
                } else {
                    self.phase = Phase::Celebrating(t);
                }
            }
            Phase::Playing => {
                let rects = self.choice_rects();
                if let Some(i) = rects.iter().position(|r| ctx.input.tapped(*r)) {
                    if self.choices[i] == self.answer {
                        let leveled = self.progress.record(self.first_try);
                        let slot = *self.slot_rects().last().unwrap();
                        ctx.correct(slot.center(), "Let's say it together!", leveled);
                        self.phase = Phase::Celebrating(0.0);
                        self.chant = Chant::default();
                    } else {
                        self.first_try = false;
                        self.misses += 1;
                        self.shake[i] = 1.0;
                        ctx.wrong("Hmm, not that one. Try again!");
                    }
                }
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let slots = self.slot_rects();

        let playing = self.phase == Phase::Playing;

        // The pattern. While playing, a little wave runs through it to show the rhythm;
        // while chanting, the item being said lights up and hops.
        for (i, (r, item)) in slots.iter().zip(&self.shown).enumerate() {
            let mut r = *r;
            if playing {
                r.y -= (ctx.time * 4.0 - i as f32 * 0.8).sin().max(0.0) * r.h * 0.06;
            } else if self.chant.lit == Some(i) {
                r = art::scale_rect(r, 1.15);
                r.y -= r.h * 0.08;
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            self.draw_item(ctx, r, *item, 0.0);
        }

        // The "?" slot, or the answer once found.
        let mut q = *slots.last().unwrap();
        if playing {
            q = art::scale_rect(q, 1.0 + 0.05 * (ctx.time * 5.0).sin());
            art::card(q, Color::from_rgba(255, 244, 200, 255));
            art::text_center(font, "?", q.center(), q.h * 0.7, art::INK);
        } else {
            if self.chant.lit == Some(self.shown.len()) {
                q = art::scale_rect(q, 1.15);
                q.y -= q.h * 0.08;
                art::glow(q, ctx.time);
            }
            art::card(q, Color::from_rgba(255, 244, 200, 255));
            self.draw_item(ctx, q, self.answer, 0.0);
        }

        // Answer choices.
        for (i, (r, item)) in self.choice_rects().iter().zip(&self.choices).enumerate() {
            let mut r = *r;
            r.x += art::shake_x(self.shake[i], ctx.time);
            if self.misses >= 2 && *item == self.answer && self.phase == Phase::Playing {
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            let bob = (ctx.time * 2.5 + i as f32).sin() * r.h * 0.03;
            self.draw_item(ctx, r, *item, bob);
        }
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> (String, String) {
        let (big, small) = match level {
            1 => ("AB", "colors"),
            2 => ("AB", "shapes"),
            3 => ("ABC", "AAB"),
            _ => ("AABB", "ABB"),
        };
        (big.to_string(), small.to_string())
    }
}
