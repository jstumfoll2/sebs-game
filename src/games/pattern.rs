//! Patterns: find the missing piece of a repeating pattern.
//!
//! The pattern is a list of "cells" laid out in different ways, with one cell missing:
//! - a **row** (red, blue, red, blue, ?),
//! - a **necklace** of beads on a loop (the missing bead can be anywhere),
//! - a **grid** (like a checkerboard) with one square missing,
//! - a **growing** row (1 star, 2 stars, 3 stars, ?).
//!
//! Items can differ by color, shape, both, size (big/small), or direction (a turning arrow).
//! Drag the right choice into the "?". Afterwards the voice chants the pattern while each
//! item lights up.

use super::counting::number_word;
use super::{
    celebration_over, fade, pick, row_of_cards, shuffle, CardDrag, Demo, DragEvent, MiniGame, Phase,
    Progress, STEP_TIMEOUT,
};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

/// One item in a pattern. Which fields matter depends on what the pattern changes (`Vary`).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Item {
    thing: Thing,
    paint: Paint,
    big: bool,
    /// How many things on the card (for growing patterns).
    count: usize,
    /// Which way an arrow points: 0 up, 1 right, 2 down, 3 left.
    turn: usize,
}

impl Item {
    fn new(thing: Thing, paint: Paint) -> Item {
        Item { thing, paint, big: true, count: 1, turn: 0 }
    }
}

/// What changes from one item to the next.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Vary {
    Color,
    Shape,
    Both,
    Size,
    Turn,
    Grow,
}

/// How the pattern is laid out on screen.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Layout {
    Row,
    Necklace,
    Grid { cols: usize },
}

/// Repeating "units": each number is which item goes there. [0, 1] means A B A B...
const UNITS: [&[usize]; 5] = [&[0, 1], &[0, 0, 1], &[0, 1, 2], &[0, 1, 1], &[0, 0, 1, 1]];
/// Units with only two kinds of item (for big/small patterns).
const TWO_KIND_UNITS: [&[usize]; 4] = [&[0, 1], &[0, 0, 1], &[0, 1, 1], &[0, 0, 1, 1]];
/// An arrow turning all the way around: up, right, down, left.
const TURNING: &[usize] = &[0, 1, 2, 3];
const MAX_ROW: usize = 9;
const DIRECTIONS: [&str; 4] = ["up", "right", "down", "left"];
/// Minimum seconds per item while chanting (so it has a steady rhythm).
const CHANT_STEP: f32 = 0.35;

/// After a right answer we "chant" the pattern, one item at a time,
/// lighting up each item while the voice says it.
#[derive(Clone, Debug, Default)]
struct Chant {
    /// The cells to say, in order.
    order: Vec<usize>,
    next: usize,
    lit: Option<usize>,
    time: f32,
    done: bool,
}

pub struct Pattern {
    progress: Progress,
    vary: Vary,
    layout: Layout,
    /// Every cell of the pattern, including the missing one.
    cells: Vec<Item>,
    missing: usize,
    choices: Vec<Item>,
    first_try: bool,
    misses: u32,
    shake: Vec<f32>,
    phase: Phase,
    chant: Chant,
    /// Answers are dragged into the "?"; tapping one just says its name.
    drag: CardDrag,
    demo: Demo,
    /// The choice that was dropped in the slot (hidden from the choices while celebrating).
    placed: Option<usize>,
    /// Pattern cells hop when tapped.
    hop: Vec<f32>,
}

impl Pattern {
    pub fn new() -> Self {
        let mut game = Pattern {
            progress: Progress::new(7),
            vary: Vary::Color,
            layout: Layout::Row,
            cells: Vec::new(),
            missing: 0,
            choices: Vec::new(),
            first_try: true,
            misses: 0,
            shake: Vec::new(),
            phase: Phase::Playing,
            chant: Chant::default(),
            drag: CardDrag::default(),
            demo: Demo::default(),
            placed: None,
            hop: Vec::new(),
        };
        game.new_round();
        game
    }

    fn answer(&self) -> Item {
        self.cells[self.missing]
    }

    fn new_round(&mut self) {
        let level = self.progress.level;
        let mut things = Thing::ALL.to_vec();
        let mut paints = Paint::ALL.to_vec();
        shuffle(&mut things);
        shuffle(&mut paints);
        let base = Item::new(things[0], paints[0]);

        // Pick the layout, what changes, and the repeating unit for this level.
        let (layout, vary, unit): (Layout, Vary, &[usize]) = match level {
            1 => (Layout::Row, Vary::Color, UNITS[0]),
            2 => (Layout::Row, Vary::Both, UNITS[0]),
            3 => (Layout::Row, pick(&[Vary::Color, Vary::Shape, Vary::Both]), UNITS[rand::gen_range(0, 3)]),
            4 => {
                if rand::gen_range(0, 2) == 0 {
                    (Layout::Row, Vary::Size, pick(&TWO_KIND_UNITS))
                } else {
                    (Layout::Row, Vary::Turn, pick(&[UNITS[0], TURNING]))
                }
            }
            5 => {
                let vary = pick(&[Vary::Color, Vary::Shape, Vary::Both, Vary::Size]);
                let unit = if vary == Vary::Size { pick(&TWO_KIND_UNITS) } else { pick(&UNITS) };
                (Layout::Necklace, vary, unit)
            }
            6 => (Layout::Grid { cols: 4 }, pick(&[Vary::Color, Vary::Shape, Vary::Both]), UNITS[rand::gen_range(0, 3)]),
            _ => (Layout::Row, Vary::Grow, UNITS[0]),
        };
        self.layout = layout;
        self.vary = vary;

        // The different kinds of item (A, B, C...).
        let kinds = unit.iter().max().copied().unwrap_or(0) + 1;
        let elems: Vec<Item> = (0..kinds)
            .map(|i| match vary {
                Vary::Color => Item { paint: paints[i], ..base },
                Vary::Shape => Item { thing: things[i], ..base },
                Vary::Both => Item { thing: things[i], paint: paints[i], ..base },
                Vary::Size => Item { big: i == 0, ..base },
                Vary::Turn => Item { turn: if unit == TURNING { i } else { i * 2 }, ..base },
                Vary::Grow => base,
            })
            .collect();

        // Lay the pattern out and choose which cell is missing.
        let at = |i: usize| elems[unit[i % unit.len()]];
        match layout {
            Layout::Row if vary == Vary::Grow => {
                // Growing: 1, 2, 3, 4, ? (or starting at 2).
                let start = rand::gen_range(1, 3);
                self.cells = (0..5).map(|k| Item { count: start + k, ..base }).collect();
                self.missing = 4;
            }
            Layout::Row => {
                let len = (unit.len() * 2 + rand::gen_range(0, unit.len())).min(MAX_ROW);
                self.cells = (0..=len).map(at).collect();
                // Usually "what comes next", but on harder levels the gap can be in the middle.
                self.missing = if level >= 4 && rand::gen_range(0, 3) == 0 {
                    rand::gen_range(2, len)
                } else {
                    len
                };
            }
            Layout::Necklace => {
                // Beads all the way around, so the pattern joins up seamlessly.
                let repeats = (8 + unit.len() - 1) / unit.len();
                let n = (unit.len() * repeats).min(12);
                self.cells = (0..n).map(at).collect();
                self.missing = rand::gen_range(0, n);
            }
            Layout::Grid { cols } => {
                // Each row repeats the pattern, sometimes shifted one step (a checkerboard).
                let shift = rand::gen_range(0, 2);
                let rows = 3;
                self.cells = (0..rows * cols).map(|i| at(i % cols + (i / cols) * shift)).collect();
                self.missing = rand::gen_range(cols, rows * cols); // not in the first row
            }
        }

        // Choices: the answer, the other kinds, and extras of the same sort.
        let answer = self.answer();
        let want = if level == 1 || vary == Vary::Size { 2 } else { 3 };
        let mut choices = vec![answer];
        for e in &elems {
            if !choices.contains(e) && choices.len() < want {
                choices.push(*e);
            }
        }
        let mut tries = 0;
        while choices.len() < want && tries < 100 {
            tries += 1;
            let extra = match vary {
                Vary::Color => Item { paint: pick(&Paint::ALL), ..answer },
                Vary::Shape => Item { thing: pick(&Thing::ALL), ..answer },
                Vary::Both => Item { thing: pick(&Thing::ALL), paint: pick(&Paint::ALL), ..answer },
                Vary::Size => Item { big: !answer.big, ..answer },
                Vary::Turn => Item { turn: rand::gen_range(0, 4), ..answer },
                Vary::Grow => Item { count: (answer.count + rand::gen_range(0, 4)).saturating_sub(2).max(1), ..answer },
            };
            if !choices.contains(&extra) {
                choices.push(extra);
            }
        }
        shuffle(&mut choices);
        self.choices = choices;

        self.first_try = true;
        self.misses = 0;
        self.shake = vec![0.0; self.choices.len()];
        self.drag.reset();
        self.placed = None;
        self.hop = vec![0.0; self.cells.len()];
        self.phase = Phase::Playing;
    }

    /// What the chant says for an item: just the part that changes ("red", "big", "up", "three").
    fn word(&self, item: Item) -> String {
        match self.vary {
            Vary::Color => item.paint.name().to_string(),
            Vary::Shape => item.thing.name().to_string(),
            Vary::Both => format!("{} {}", item.paint.name(), item.thing.name()),
            Vary::Size => if item.big { "big" } else { "small" }.to_string(),
            Vary::Turn => DIRECTIONS[item.turn].to_string(),
            Vary::Grow => number_word(item.count),
        }
    }

    /// The full name of an item, for when it's tapped: "red star", "big red star", "three stars".
    fn full_name(&self, item: Item) -> String {
        let plain = format!("{} {}", item.paint.name(), item.thing.name());
        match self.vary {
            Vary::Size => format!("{} {plain}", if item.big { "big" } else { "small" }),
            Vary::Turn => format!("an arrow pointing {}", DIRECTIONS[item.turn]),
            Vary::Grow if item.count == 1 => format!("one {}", item.thing.name()),
            Vary::Grow => format!("{} {}", number_word(item.count), item.thing.plural()),
            _ => plain,
        }
    }

    fn update_chant(&mut self, ctx: &mut Ctx) {
        self.chant.time += ctx.dt;
        let voice_done = !ctx.voice.busy() || self.chant.time > STEP_TIMEOUT;
        if self.chant.time < CHANT_STEP || !voice_done {
            return;
        }
        if let Some(&cell) = self.chant.order.get(self.chant.next) {
            let word = self.word(self.cells[cell]);
            ctx.voice.then(&word);
            self.chant.lit = Some(cell);
            self.chant.next += 1;
            self.chant.time = 0.0;
        } else {
            self.chant.lit = None;
            self.chant.done = true;
            self.phase = Phase::Celebrating(0.6); // short pause before the next round
        }
    }

    /// Which cells to chant: the whole pattern, or just the missing cell's row in a grid.
    fn chant_order(&self) -> Vec<usize> {
        match self.layout {
            Layout::Grid { cols } => {
                let row = self.missing / cols;
                (row * cols..(row + 1) * cols).collect()
            }
            _ => (0..self.cells.len()).collect(),
        }
    }

    /// Where each cell of the pattern goes on screen.
    fn cell_rects(&self) -> Vec<Rect> {
        let (w, h) = (screen_width(), screen_height());
        let n = self.cells.len();
        match self.layout {
            Layout::Row => row_of_cards(n, 0.94, 0.2, 0.34),
            Layout::Necklace => {
                let (c, rx, ry) = (vec2(w / 2.0, h * 0.37), w * 0.27, h * 0.2);
                let size = (h * 0.13).min(TAU * ry / n as f32 * 0.85);
                (0..n)
                    .map(|i| {
                        let a = -PI / 2.0 + i as f32 * TAU / n as f32;
                        let p = c + vec2(a.cos() * rx, a.sin() * ry);
                        Rect::new(p.x - size / 2.0, p.y - size / 2.0, size, size)
                    })
                    .collect()
            }
            Layout::Grid { cols } => {
                let rows = n.div_ceil(cols);
                let cell = (w * 0.6 / cols as f32).min(h * 0.46 / rows as f32);
                let size = cell * 0.9;
                let x0 = (w - cell * cols as f32) / 2.0;
                (0..n)
                    .map(|i| {
                        let (col, row) = ((i % cols) as f32, (i / cols) as f32);
                        Rect::new(x0 + col * cell, h * 0.13 + row * cell, size, size)
                    })
                    .collect()
            }
        }
    }

    fn choice_rects(&self) -> Vec<Rect> {
        match self.layout {
            Layout::Row => row_of_cards(self.choices.len(), 0.7, 0.26, 0.74),
            _ => row_of_cards(self.choices.len(), 0.6, 0.2, 0.8),
        }
    }

    /// An item on its card, with a label underneath when there's room ("RED", "BIG", "UP").
    fn draw_item(&self, ctx: &Ctx, r: Rect, item: Item, bob: f32) {
        let labelled = r.h > screen_height() * 0.14;
        let center = vec2(r.center().x, r.y + r.h * if labelled { 0.4 } else { 0.5 } + bob);
        let s = r.w * if labelled { 0.27 } else { 0.32 };
        match self.vary {
            Vary::Turn => draw_arrow(center, s, item.turn, item.paint.color()),
            Vary::Grow => draw_count(item.thing, item.paint.color(), item.count, center, s * 2.2),
            _ => art::draw_thing(item.thing, center, s * if item.big { 1.0 } else { 0.55 }, item.paint.color()),
        }
        if labelled {
            let label = vec2(r.center().x, r.y + r.h * 0.84);
            let first = art::readable(item.paint.color());
            art::word_label(ctx.font(), &self.word(item), label, r.h * 0.12, r.w * 0.9, first, None);
        }
    }
}

/// An arrow pointing up (0), right (1), down (2) or left (3).
fn draw_arrow(c: Vec2, s: f32, turn: usize, color: Color) {
    let a = turn as f32 * PI / 2.0;
    let rot = |x: f32, y: f32| c + vec2(x * a.cos() - y * a.sin(), x * a.sin() + y * a.cos()) * s;
    let dark = art::darken(color, 0.25);
    for (grow, col) in [(1.12, dark), (1.0, color)] {
        let g = |x: f32, y: f32| rot(x * grow, y * grow);
        // Head (a triangle pointing "up" before turning) and a thick shaft.
        draw_triangle(g(0.0, -0.95), g(-0.75, -0.1), g(0.75, -0.1), col);
        draw_triangle(g(-0.3, -0.15), g(0.3, -0.15), g(0.3, 0.85), col);
        draw_triangle(g(-0.3, -0.15), g(0.3, 0.85), g(-0.3, 0.85), col);
    }
}

/// `n` small things neatly arranged in a square area `size` wide.
fn draw_count(thing: Thing, color: Color, n: usize, c: Vec2, size: f32) {
    // Up to 2 across, then more rows: keeps each thing a good size.
    let cols = if n <= 2 { n.max(1) } else if n <= 4 { 2 } else { 3 };
    let rows = n.div_ceil(cols);
    let cell = size / cols.max(rows) as f32 * 1.15;
    for i in 0..n {
        let (row, col) = (i / cols, i % cols);
        let in_row = (n - row * cols).min(cols);
        let x = c.x + (col as f32 - (in_row as f32 - 1.0) / 2.0) * cell;
        let y = c.y + (row as f32 - (rows as f32 - 1.0) / 2.0) * cell;
        art::draw_thing(thing, vec2(x, y), cell * 0.4, color);
    }
}

impl MiniGame for Pattern {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        // Show how to play: a hand drags from the choices up into the "?".
        self.demo.start(2.0);
        let intro = format!("{} Drag it into the question mark!", self.prompt());
        ctx.voice.then(&intro);
    }

    fn prompt(&self) -> String {
        match (self.layout, self.vary) {
            (Layout::Necklace, _) => "Find the missing bead!",
            (Layout::Grid { .. }, _) => "Find the missing square!",
            (_, Vary::Grow) => "It's growing! What comes next?",
            (Layout::Row, _) if self.missing + 1 < self.cells.len() => "Find the missing one!",
            _ => "What comes next?",
        }
        .to_string()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.shake, ctx.dt, 2.5);
        fade(&mut self.hop, ctx.dt, 3.0);
        self.demo.update(ctx);
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
                let cells = self.cell_rects();
                // Tapping a cell says what it is; tapping the "?" asks the question.
                if self.drag.held().is_none() {
                    if let Some(i) = cells.iter().position(|r| ctx.input.tapped(*r)) {
                        ctx.sfx.pop();
                        self.hop[i] = 1.0;
                        if i == self.missing {
                            ctx.voice.say(&self.prompt());
                        } else {
                            ctx.voice.say(&self.full_name(self.cells[i]));
                        }
                        return;
                    }
                }
                let rects = self.choice_rects();
                match self.drag.update(&ctx.input, &rects, ctx.dt) {
                    DragEvent::PickedUp(i) => {
                        // Touching a choice says what it is.
                        ctx.sfx.pop();
                        ctx.voice.say(&self.full_name(self.choices[i]));
                    }
                    DragEvent::Dropped(i, at) => {
                        // Only a choice dropped on the "?" counts as an answer.
                        let slot = cells[self.missing];
                        if art::scale_rect(slot, 1.6).contains(at) {
                            if self.choices[i] == self.answer() {
                                let leveled = self.progress.record(self.first_try);
                                ctx.correct(slot.center(), "Let's say it together!", leveled);
                                self.phase = Phase::Celebrating(0.0);
                                self.chant = Chant { order: self.chant_order(), ..Default::default() };
                                self.placed = Some(i);
                            } else {
                                self.first_try = false;
                                self.misses += 1;
                                self.shake[i] = 1.0;
                                ctx.wrong("Hmm, not that one. Try again!");
                            }
                        }
                    }
                    DragEvent::Nothing => {}
                }
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let cells = self.cell_rects();
        let playing = self.phase == Phase::Playing;

        // A necklace hangs on a string.
        if self.layout == Layout::Necklace {
            let (w, h) = (screen_width(), screen_height());
            draw_ellipse_lines(w / 2.0, h * 0.37, w * 0.27, h * 0.2, 0.0, 4.0, art::darken(art::SHADOW, 0.2));
        }

        // The pattern. While playing, a little wave runs through it to show the rhythm;
        // while chanting, the item being said lights up and hops.
        for (i, r) in cells.iter().enumerate() {
            let mut r = *r;
            let is_missing = i == self.missing;
            if playing {
                if self.layout == Layout::Row {
                    r.y -= (ctx.time * 4.0 - i as f32 * 0.8).sin().max(0.0) * r.h * 0.06;
                }
                r.y -= (self.hop[i] * PI).sin() * r.h * 0.15;
                if is_missing {
                    r = art::scale_rect(r, 1.0 + 0.05 * (ctx.time * 5.0).sin());
                }
            } else if self.chant.lit == Some(i) {
                r = art::scale_rect(r, 1.15);
                r.y -= r.h * 0.08;
                art::glow(r, ctx.time);
            }
            if is_missing {
                art::card(r, Color::from_rgba(255, 244, 200, 255));
                if playing {
                    art::text_center(font, "?", r.center(), r.h * 0.7, art::INK);
                } else {
                    self.draw_item(ctx, r, self.cells[i], 0.0);
                }
            } else {
                art::card(r, WHITE);
                self.draw_item(ctx, r, self.cells[i], 0.0);
            }
        }

        // Answer choices. The one being dragged is drawn last so it's on top.
        let homes = self.choice_rects();
        let mut order: Vec<usize> = (0..self.choices.len()).filter(|&i| Some(i) != self.drag.held()).collect();
        order.extend(self.drag.held());
        for i in order {
            if !playing && self.placed == Some(i) {
                continue; // it's in the "?" now
            }
            let item = self.choices[i];
            let mut r = self.drag.rect(i, homes[i]);
            r.x += art::shake_x(self.shake[i], ctx.time);
            if self.misses >= 2 && item == self.answer() && playing {
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            let bob = (ctx.time * 2.5 + i as f32).sin() * r.h * 0.03;
            self.draw_item(ctx, r, item, bob);
        }

        // How-to-play: a hand drags from the middle of the choices up to the "?".
        let from = homes.iter().map(|r| r.center()).sum::<Vec2>() / homes.len().max(1) as f32;
        self.demo.draw_with_card(from, cells[self.missing].center(), homes[0].w * 0.8);
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        match level {
            1 => "AB colors",
            2 => "AB shapes",
            3 => "ABC, AAB",
            4 => "big/small, arrows",
            5 => "necklace",
            6 => "grid",
            _ => "growing",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_makes_a_solvable_round() {
        let mut game = Pattern::new();
        for level in 1..=7 {
            game.progress.set_level(level);
            for _ in 0..300 {
                game.new_round();
                assert!(game.missing < game.cells.len());
                let answer = game.answer();
                assert_eq!(game.choices.iter().filter(|c| **c == answer).count(), 1, "exactly one right choice");
                assert!(game.choices.len() >= 2);
            }
        }
    }
}
