//! Groups: early multiplication by counting in groups ("skip counting").
//! Plates each hold the same number of things. Tap each plate to count by that number
//! ("two, four, six"), then pick how many there are altogether. The strip of running
//! totals along the bottom (2 -> 4 -> 6) shows the pattern of multiples.

use super::counting::number_word;
use super::{celebration_over, fade, pick, row_of_cards, shuffle, MiniGame, Progress, STEP_TIMEOUT};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use macroquad::prelude::*;

/// For each level: the group sizes to use, and the fewest/most groups.
const LEVELS: [(&[usize], usize, usize); 6] = [
    (&[2], 2, 3),
    (&[2], 2, 5),
    (&[5], 2, 4),
    (&[10], 2, 5),
    (&[3], 2, 4),
    (&[2, 3, 5, 10], 2, 5),
];
/// Minimum seconds per number when counting together after a miss.
const RECOUNT_STEP: f32 = 0.6;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    Tapping,
    /// All plates counted: say "three groups of two make six", then ask.
    Pause(f32),
    Choosing,
    Celebrating(f32),
}

pub struct Groups {
    progress: Progress,
    thing: Thing,
    paint: Paint,
    /// Things on each plate.
    size: usize,
    plates: usize,
    /// When each plate was counted (1st, 2nd, ...), in tap order.
    order: Vec<Option<usize>>,
    counted: usize,
    pop: Vec<f32>,
    wiggle: Vec<f32>,
    stage: Stage,
    choices: Vec<usize>,
    shake: Vec<f32>,
    first_try: bool,
    /// Counting together after a miss: (plates counted so far, seconds on this one).
    recount: Option<(usize, f32)>,
}

impl Groups {
    pub fn new() -> Self {
        let mut game = Groups {
            progress: Progress::new(LEVELS.len() as u32),
            thing: Thing::Apple,
            paint: Paint::Red,
            size: 2,
            plates: 2,
            order: Vec::new(),
            counted: 0,
            pop: Vec::new(),
            wiggle: Vec::new(),
            stage: Stage::Tapping,
            choices: Vec::new(),
            shake: Vec::new(),
            first_try: true,
            recount: None,
        };
        game.new_round();
        game
    }

    fn total(&self) -> usize {
        self.size * self.plates
    }

    fn new_round(&mut self) {
        let (sizes, lo, hi) = LEVELS[(self.progress.level - 1) as usize];
        self.size = pick(sizes);
        self.plates = rand::gen_range(lo, hi + 1);
        self.thing = pick(&Thing::ALL);
        self.paint = if self.thing == Thing::Apple { Paint::Red } else { pick(&Paint::ALL) };

        // Choices are all multiples: the answer and its neighbors (e.g. 4, 6, 8).
        let total = self.total();
        let mut choices = vec![total, total + self.size];
        choices.push(if total > self.size { total - self.size } else { total + 2 * self.size });
        shuffle(&mut choices);
        self.choices = choices;

        self.order = vec![None; self.plates];
        self.counted = 0;
        self.pop = vec![0.0; self.plates];
        self.wiggle = vec![0.0; self.plates];
        self.shake = vec![0.0; self.choices.len()];
        self.stage = Stage::Tapping;
        self.first_try = true;
        self.recount = None;
    }

    fn plate_rects(&self) -> Vec<Rect> {
        row_of_cards(self.plates, 0.9, 0.3, 0.36)
    }

    fn choice_rects(&self) -> Vec<Rect> {
        row_of_cards(self.choices.len(), 0.6, 0.22, 0.8)
    }

    fn by_what(&self) -> String {
        match self.size {
            2 => "twos".to_string(),
            3 => "threes".to_string(),
            5 => "fives".to_string(),
            10 => "tens".to_string(),
            n => format!("{}s", number_word(n)),
        }
    }

    /// "three groups of two"
    fn groups_of(&self) -> String {
        let groups = if self.plates == 1 { "group" } else { "groups" };
        format!("{} {groups} of {}", number_word(self.plates), number_word(self.size))
    }

    fn plate_counted_as(&self, n: usize) -> Option<usize> {
        self.order.iter().position(|o| *o == Some(n))
    }

    /// Counting together after a miss: light each plate as its running total is said.
    fn update_recount(&mut self, ctx: &mut Ctx) {
        let Some((n, t)) = self.recount else { return };
        let t = t + ctx.dt;
        let voice_done = !ctx.voice.busy() || t > STEP_TIMEOUT;
        if t >= RECOUNT_STEP && voice_done {
            if n < self.plates {
                let n = n + 1;
                ctx.voice.say(&number_word(n * self.size));
                if let Some(i) = self.plate_counted_as(n) {
                    self.pop[i] = 1.0;
                }
                self.recount = Some((n, 0.0));
            } else {
                self.recount = None;
                ctx.voice.say(&self.prompt());
            }
            return;
        }
        self.recount = Some((n, t));
    }
}

impl MiniGame for Groups {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        ctx.voice.then(&self.prompt());
    }

    fn prompt(&self) -> String {
        match self.stage {
            Stage::Choosing => format!("How many {} altogether?", self.thing.plural()),
            _ => format!("Let's count by {}! Tap each plate.", self.by_what()),
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.pop, ctx.dt, 3.0);
        fade(&mut self.wiggle, ctx.dt, 2.5);
        fade(&mut self.shake, ctx.dt, 2.5);
        if self.recount.is_some() {
            self.update_recount(ctx);
            return;
        }

        match self.stage {
            Stage::Tapping => {
                if let Some(i) = self.plate_rects().iter().position(|r| ctx.input.tapped(*r)) {
                    if self.order[i].is_none() {
                        self.counted += 1;
                        self.order[i] = Some(self.counted);
                        self.pop[i] = 1.0;
                        ctx.sfx.pop();
                        ctx.voice.say(&number_word(self.counted * self.size));
                        if self.counted == self.plates {
                            self.stage = Stage::Pause(1.0);
                        }
                    } else {
                        self.wiggle[i] = 1.0;
                        ctx.voice.say("We already counted that plate!");
                    }
                }
            }
            Stage::Pause(t) => {
                let t = t - ctx.dt;
                if t <= 0.0 && !ctx.voice.busy() {
                    self.stage = Stage::Choosing;
                    let summary = format!("{} make {}!", self.groups_of(), number_word(self.total()));
                    ctx.voice.say(&crate::alphabet::capitalize(&summary));
                    ctx.voice.then(&self.prompt());
                } else {
                    self.stage = Stage::Pause(t);
                }
            }
            Stage::Choosing => {
                let rects = self.choice_rects();
                if let Some(i) = rects.iter().position(|r| ctx.input.tapped(*r)) {
                    if self.choices[i] == self.total() {
                        let leveled = self.progress.record(self.first_try);
                        let words = format!("{} is {}!", self.groups_of(), number_word(self.total()));
                        ctx.correct(rects[i].center(), &crate::alphabet::capitalize(&words), leveled);
                        self.stage = Stage::Celebrating(1.5);
                    } else {
                        self.first_try = false;
                        self.shake[i] = 1.0;
                        ctx.wrong(&format!("Let's count by {} together!", self.by_what()));
                        self.recount = Some((0, 0.0));
                    }
                }
            }
            Stage::Celebrating(t) => {
                let t = t - ctx.dt;
                if celebration_over(t, ctx) {
                    self.new_round();
                    ctx.voice.then(&self.prompt());
                } else {
                    self.stage = Stage::Celebrating(t);
                }
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        let color = self.paint.color();

        // What we're doing, at the top: "COUNT BY TWOS".
        let heading = format!("count by {}", self.by_what());
        art::word_label(font, &heading, vec2(w / 2.0, h * 0.075), h * 0.045, w * 0.4, art::readable(color), Some(WHITE));

        let lit = self.recount.and_then(|(n, _)| self.plate_counted_as(n));
        for (i, r) in self.plate_rects().iter().enumerate() {
            let mut r = art::scale_rect(*r, 1.0 + self.pop[i] * 0.12);
            r.x += art::shake_x(self.wiggle[i], ctx.time);
            if self.order[i].is_none() && self.stage == Stage::Tapping {
                r.y += (ctx.time * 2.5 + i as f32).sin() * r.h * 0.02; // uncounted plates bob
            }
            if lit == Some(i) {
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            draw_group(self.thing, color, self.size, r);
            // Running total badge: 2, 4, 6...
            if let Some(n) = self.order[i] {
                let c = vec2(r.x + r.w * 0.88, r.y + r.h * 0.1);
                art::badge(font, c, r.w * 0.14, &(n * self.size).to_string());
            }
        }

        // The skip-counting strip: 2 -> 4 -> 6.
        if self.counted > 0 {
            let steps: Vec<String> = (1..=self.counted).map(|n| (n * self.size).to_string()).collect();
            let line = steps.join("  >  ");
            art::text_center(font, &line, vec2(w / 2.0, h * 0.6), h * 0.07, art::readable(color));
        }

        if matches!(self.stage, Stage::Choosing | Stage::Celebrating(_)) {
            for (i, (r, &k)) in self.choice_rects().iter().zip(&self.choices).enumerate() {
                let mut r = *r;
                r.x += art::shake_x(self.shake[i], ctx.time);
                if matches!(self.stage, Stage::Celebrating(_)) && k == self.total() {
                    r = art::scale_rect(r, 1.12 + 0.04 * (ctx.time * 8.0).sin());
                }
                art::card(r, WHITE);
                let c = art::readable(Paint::ALL[(k / self.size) % Paint::ALL.len()].color());
                art::text_center(font, &k.to_string(), r.center() - vec2(0.0, r.h * 0.08), r.h * 0.5, c);
                art::word_label(font, &number_word(k), vec2(r.center().x, r.y + r.h * 0.8), r.h * 0.12, r.w * 0.9, c, None);
            }
        }
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        let (sizes, lo, hi) = LEVELS[(level - 1) as usize];
        if sizes.len() == 1 {
            format!("{}s, {lo}-{hi} plates", sizes[0])
        } else {
            "mixed".to_string()
        }
    }
}

/// Neatly arrange `n` things on a plate: in rows, like dots on dice or a ten-frame.
fn draw_group(thing: Thing, color: Color, n: usize, r: Rect) {
    let cols = match n {
        0..=3 => n.max(1),
        4 => 2,
        5..=6 => 3,
        _ => 5,
    };
    let rows = n.div_ceil(cols);
    let cell = (r.w * 0.84 / cols as f32).min(r.h * 0.84 / rows as f32);
    let s = cell * 0.42;
    for i in 0..n {
        let (row, col) = (i / cols, i % cols);
        let in_row = (n - row * cols).min(cols);
        let x = r.center().x + (col as f32 - (in_row as f32 - 1.0) / 2.0) * cell;
        let y = r.center().y + (row as f32 - (rows as f32 - 1.0) / 2.0) * cell;
        art::draw_thing(thing, vec2(x, y), s, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_are_distinct_multiples_including_the_answer() {
        let mut game = Groups::new();
        for level in 1..=LEVELS.len() as u32 {
            game.progress.set_level(level);
            for _ in 0..200 {
                game.new_round();
                let total = game.total();
                assert!(game.choices.contains(&total));
                assert!(game.choices.iter().all(|&c| c > 0 && c % game.size == 0));
                let mut sorted = game.choices.clone();
                sorted.sort();
                sorted.dedup();
                assert_eq!(sorted.len(), game.choices.len(), "choices must be different");
            }
        }
    }
}
