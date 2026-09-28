//! Count & tap: tap each object to count it (the voice counts along and a number badge appears),
//! then pick how many there are. The last badge matches the answer, which teaches the key idea
//! that "the last number you say is how many there are".

use super::{celebration_over, fade, pick, shuffle, MiniGame, Progress, STEP_TIMEOUT};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use macroquad::prelude::*;

pub const NUMBER_WORDS: [&str; 21] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
    "nineteen", "twenty",
];
/// Smallest and largest count for each level.
/// Any number from 0 to 99 in words: 7 -> "seven", 45 -> "forty-five".
pub fn number_word(n: usize) -> String {
    const TENS: [&str; 10] = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    match n {
        0..=20 => NUMBER_WORDS[n].to_string(),
        _ if n < 100 && n % 10 == 0 => TENS[n / 10].to_string(),
        _ if n < 100 => format!("{}-{}", TENS[n / 10], NUMBER_WORDS[n % 10]),
        _ => n.to_string(),
    }
}

/// Levels whose range starts at 0 sometimes show an empty basket: zero means none!
const RANGES: [(usize, usize); 7] = [(0, 3), (0, 5), (4, 7), (5, 10), (7, 10), (10, 15), (11, 20)];
/// Biggest number the game uses.
const MAX_NUMBER: usize = 20;
/// Minimum seconds per number when counting together after a miss.
const RECOUNT_STEP: f32 = 0.5;

/// Counting together after a miss: which number we're on (0 = still saying
/// "Let's count them together!") and how long we've been on it.
#[derive(Clone, Copy, Debug)]
struct Recount {
    number: usize,
    time: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    /// Tapping objects to count them.
    Tapping,
    /// All counted; short pause before asking "how many?".
    Pause(f32),
    /// Picking the number.
    Choosing,
    Celebrating(f32),
}

pub struct Counting {
    progress: Progress,
    thing: Thing,
    paint: Paint,
    /// Object positions, from 0..1 inside the play area.
    spots: Vec<Vec2>,
    /// Grid size used to place the objects (also sets how big they are).
    grid: (usize, usize),
    /// The count number shown on each object once tapped.
    order: Vec<Option<usize>>,
    counted: usize,
    pop: Vec<f32>,
    wiggle: Vec<f32>,
    stage: Stage,
    choices: Vec<usize>,
    shake: Vec<f32>,
    first_try: bool,
    recount: Option<Recount>,
}

impl Counting {
    pub fn new() -> Self {
        let mut game = Counting {
            progress: Progress::new(RANGES.len() as u32),
            thing: Thing::Apple,
            paint: Paint::Red,
            spots: Vec::new(),
            grid: (1, 1),
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

    fn n(&self) -> usize {
        self.spots.len()
    }

    fn new_round(&mut self) {
        let (lo, hi) = RANGES[(self.progress.level - 1) as usize];
        // Zero shows up now and then (not every other round!) on levels that include it.
        let n = if lo == 0 && rand::gen_range(0, 5) == 0 {
            0
        } else {
            rand::gen_range(lo.max(1), hi + 1)
        };

        self.thing = pick(&Thing::ALL);
        self.paint = if self.thing == Thing::Apple {
            Paint::Red
        } else {
            pick(&Paint::ALL)
        };

        // Scatter objects on a loose grid so they never overlap.
        // (The play area is wide, so use more columns than rows.)
        let cols = ((n as f32 * 2.4).sqrt().ceil() as usize).max(1);
        let rows = ((n + cols - 1) / cols).max(1);
        let mut cells: Vec<usize> = (0..cols * rows).collect();
        shuffle(&mut cells);
        self.spots = cells[..n]
            .iter()
            .map(|&cell| {
                let jitter = || rand::gen_range(-0.15, 0.15);
                vec2(
                    ((cell % cols) as f32 + 0.5 + jitter()) / cols as f32,
                    ((cell / cols) as f32 + 0.5 + jitter()) / rows as f32,
                )
            })
            .collect();
        self.grid = (cols, rows);

        // Choices: the answer plus two nearby numbers.
        let mut choices = vec![n];
        let mut near: Vec<usize> = [n.wrapping_sub(2), n.wrapping_sub(1), n + 1, n + 2]
            .into_iter()
            .filter(|&k| k <= MAX_NUMBER)
            .collect();
        shuffle(&mut near);
        choices.extend(near.into_iter().take(2));
        shuffle(&mut choices);
        self.choices = choices;

        self.order = vec![None; n];
        self.counted = 0;
        self.pop = vec![0.0; n];
        self.wiggle = vec![0.0; n];
        self.shake = vec![0.0; self.choices.len()];
        // With nothing to tap, go straight to asking "how many?".
        self.stage = if n == 0 { Stage::Pause(0.5) } else { Stage::Tapping };
        self.first_try = true;
        self.recount = None;
    }

    fn area() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        Rect::new(w * 0.08, h * 0.15, w * 0.84, h * 0.46)
    }

    fn spot_px(&self, i: usize) -> Vec2 {
        let a = Self::area();
        vec2(a.x + self.spots[i].x * a.w, a.y + self.spots[i].y * a.h)
    }

    fn obj_size(&self) -> f32 {
        let a = Self::area();
        let cell = (a.w / self.grid.0 as f32).min(a.h / self.grid.1 as f32);
        (cell * 0.32).min(screen_height() * 0.11)
    }

    fn choice_rects(&self) -> Vec<Rect> {
        super::row_of_cards(self.choices.len(), 0.6, 0.24, 0.76)
    }

    fn name(&self) -> String {
        let noun = if self.n() == 1 { self.thing.name() } else { self.thing.plural() };
        format!("{} {}", self.paint.name(), noun)
    }

    /// Which object is lit up while counting together after a miss.
    fn recount_highlight(&self) -> Option<usize> {
        let number = self.recount?.number;
        self.order.iter().position(|o| *o == Some(number))
    }

    /// Step through counting together: each number is highlighted *while* it's being said,
    /// and we only move on once the voice has finished saying it.
    fn update_recount(&mut self, ctx: &mut Ctx) {
        let Some(mut rc) = self.recount else { return };
        rc.time += ctx.dt;
        let voice_done = !ctx.voice.busy() || rc.time > STEP_TIMEOUT;
        if rc.time >= RECOUNT_STEP && voice_done {
            if rc.number < self.n() {
                rc = Recount { number: rc.number + 1, time: 0.0 };
                ctx.voice.say(NUMBER_WORDS[rc.number]);
                // (Find the index first: Rust won't let us read `self` inside `self.pop[...] = `.)
                let target = self.recount_target(rc.number);
                self.pop[target] = 1.0;
            } else {
                self.recount = None;
                ctx.voice.say(&self.prompt());
                return;
            }
        }
        self.recount = Some(rc);
    }

    fn recount_target(&self, number: usize) -> usize {
        self.order.iter().position(|o| *o == Some(number)).unwrap_or(0)
    }
}

impl MiniGame for Counting {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        ctx.voice.then(&self.prompt());
    }

    fn prompt(&self) -> String {
        match self.stage {
            Stage::Choosing => format!("How many {}?", self.thing.plural()),
            _ if self.n() == 0 => format!("Let's count the {}! Hmm... where are they?", self.name()),
            _ => format!("Let's count the {}! Tap each one.", self.name()),
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.pop, ctx.dt, 3.0);
        fade(&mut self.wiggle, ctx.dt, 2.5);
        fade(&mut self.shake, ctx.dt, 2.5);
        if self.recount.is_some() {
            // Wait for counting together to finish before accepting answers.
            self.update_recount(ctx);
            return;
        }

        let input = ctx.input;
        match self.stage {
            Stage::Tapping => {
                if !input.pressed {
                    return;
                }
                let reach = self.obj_size() * 1.3;
                if let Some(i) = (0..self.n()).find(|&i| self.spot_px(i).distance(input.pos) < reach) {
                    if self.order[i].is_none() {
                        self.counted += 1;
                        self.order[i] = Some(self.counted);
                        self.pop[i] = 1.0;
                        ctx.sfx.pop();
                        ctx.voice.say(NUMBER_WORDS[self.counted]);
                        if self.counted == self.n() {
                            self.stage = Stage::Pause(1.0);
                        }
                    } else {
                        self.wiggle[i] = 1.0;
                        ctx.voice.say("We already counted that one!");
                    }
                }
            }
            Stage::Pause(t) => {
                let t = t - ctx.dt;
                if t <= 0.0 {
                    self.stage = Stage::Choosing;
                    ctx.voice.then(&self.prompt());
                } else {
                    self.stage = Stage::Pause(t);
                }
            }
            Stage::Choosing => {
                let rects = self.choice_rects();
                if let Some(i) = rects.iter().position(|r| input.tapped(*r)) {
                    let n = self.n();
                    if self.choices[i] == n {
                        let leveled = self.progress.record(self.first_try);
                        let mut words = format!("{} {}!", NUMBER_WORDS[n], self.name());
                        if n == 0 {
                            words += " Zero means none!";
                        }
                        ctx.correct(rects[i].center(), &words, leveled);
                        self.stage = Stage::Celebrating(2.2);
                    } else if n == 0 {
                        self.first_try = false;
                        self.shake[i] = 1.0;
                        ctx.wrong("Look, the basket is empty! There are none. None is zero!");
                        ctx.voice.then(&self.prompt());
                    } else {
                        self.first_try = false;
                        self.shake[i] = 1.0;
                        ctx.wrong("Let's count them together!");
                        self.recount = Some(Recount { number: 0, time: 0.0 });
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
        let s = self.obj_size();
        let lit = self.recount_highlight();
        let (w, h) = (screen_width(), screen_height());

        // What we're counting, written at the top: "APPLES".
        let color = art::readable(self.paint.color());
        let top = vec2(w / 2.0, h * 0.075);
        art::word_label(font, self.thing.plural(), top, h * 0.05, w * 0.4, color, Some(WHITE));

        if self.n() == 0 {
            // An empty basket: there's nothing here!
            let a = Self::area();
            let size = a.h * 0.7;
            let r = Rect::new(a.center().x - size * 0.6, a.center().y - size / 2.0, size * 1.2, size);
            art::bucket(r, Paint::Brown.color());
        }

        for i in 0..self.n() {
            let mut c = self.spot_px(i);
            c.x += art::shake_x(self.wiggle[i], ctx.time);
            if self.order[i].is_none() {
                // Uncounted objects bob gently, inviting a tap.
                c.y += (ctx.time * 3.0 + i as f32).sin() * s * 0.06;
            }
            if lit == Some(i) {
                draw_circle(c.x, c.y, s * 1.35, Color::new(1.0, 0.85, 0.2, 0.55));
            }
            let scale = 1.0 + self.pop[i] * 0.3;
            art::draw_thing(self.thing, c, s * scale, self.paint.color());
            if let Some(k) = self.order[i] {
                art::badge(font, c + vec2(s * 0.8, -s * 0.8), s * 0.38, &k.to_string());
            }
        }

        if matches!(self.stage, Stage::Choosing | Stage::Celebrating(_)) {
            for (i, (r, &k)) in self.choice_rects().iter().zip(&self.choices).enumerate() {
                let mut r = *r;
                r.x += art::shake_x(self.shake[i], ctx.time);
                if matches!(self.stage, Stage::Celebrating(_)) && k == self.n() {
                    r = art::scale_rect(r, 1.12 + 0.04 * (ctx.time * 8.0).sin());
                }
                art::card(r, WHITE);
                let color = art::readable(Paint::ALL[k % Paint::ALL.len()].color());
                art::text_center(font, &k.to_string(), r.center() - vec2(0.0, r.h * 0.15), r.h * 0.5, color);
                number_dots(r, k);
                // The number's word under the card: "FIVE".
                let label = vec2(r.center().x, r.y + r.h * 1.16);
                art::word_label(font, NUMBER_WORDS[k], label, r.h * 0.14, r.w * 1.1, color, Some(WHITE));
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
        let (lo, hi) = RANGES[(level - 1) as usize];
        format!("{lo}-{hi}")
    }
}

/// Little dots under the number (rows of five) so the number has a "size" you can see.
fn number_dots(r: Rect, k: usize) {
    let rows = k.div_ceil(5).max(1);
    // Squeeze the rows together when there are lots of them (up to 4 rows for 20).
    let gap = (r.w * 0.14).min(r.h * 0.34 / rows as f32);
    let rad = gap * 0.32;
    let top = r.y + r.h * 0.62 + gap / 2.0;
    for i in 0..k {
        let row = i / 5;
        let col = i % 5;
        let in_row = (k - row * 5).min(5);
        let x = r.center().x + (col as f32 - (in_row as f32 - 1.0) / 2.0) * gap;
        let y = top + row as f32 * gap;
        draw_circle(x, y, rad, art::INK);
    }
}
