//! Color sorting: drag the object into the bucket that matches its color.
//! Tapping a bucket just says its color; only dragging answers. The voice names colors on
//! pick-up, on success, and on a miss, to connect colors with their names.

use super::{celebration_over, fade, pick, shuffle, Demo, MiniGame, Phase, Progress};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use macroquad::prelude::*;

/// Easy-to-tell-apart colors used for the first levels.
const EASY_COLORS: [Paint; 4] = [Paint::Red, Paint::Blue, Paint::Yellow, Paint::Green];
const CELEBRATE_SECS: f32 = 1.8;

pub struct ColorSort {
    progress: Progress,
    buckets: Vec<Paint>,
    target: Paint,
    thing: Thing,
    item_pos: Vec2,
    dragging: bool,
    grab_offset: Vec2,
    first_try: bool,
    misses: u32,
    shake: Vec<f32>,
    phase: Phase,
    chosen: usize,
    /// Buckets hop when tapped.
    hop: Vec<f32>,
    demo: Demo,
}

impl ColorSort {
    pub fn new() -> Self {
        let mut game = ColorSort {
            progress: Progress::new(5),
            buckets: Vec::new(),
            target: Paint::Red,
            thing: Thing::Ball,
            item_pos: Vec2::ZERO,
            dragging: false,
            grab_offset: Vec2::ZERO,
            first_try: true,
            misses: 0,
            shake: Vec::new(),
            phase: Phase::Playing,
            chosen: 0,
            hop: Vec::new(),
            demo: Demo::default(),
        };
        game.new_round();
        game
    }

    fn new_round(&mut self) {
        // Level 1 = 2 buckets, level 2 = 3 buckets, ... level 5 = 6 buckets.
        let count = (self.progress.level + 1) as usize;
        let mut pool = if self.progress.level <= 2 {
            EASY_COLORS.to_vec()
        } else {
            Paint::ALL.to_vec()
        };
        shuffle(&mut pool);
        pool.truncate(count);
        self.target = pick(&pool);
        self.buckets = pool;
        self.thing = pick(&Thing::ALL);
        self.item_pos = home();
        self.dragging = false;
        self.first_try = true;
        self.misses = 0;
        self.shake = vec![0.0; count];
        self.hop = vec![0.0; count];
        self.phase = Phase::Playing;
    }

    fn choose(&mut self, i: usize, ctx: &mut Ctx) {
        let rects = bucket_rects(self.buckets.len());
        if self.buckets[i] == self.target {
            let leveled = self.progress.record(self.first_try);
            let words = format!("{}!", capitalize(self.target.name()));
            ctx.correct(rects[i].center(), &words, leveled);
            self.chosen = i;
            self.dragging = false;
            self.phase = Phase::Celebrating(CELEBRATE_SECS);
        } else {
            self.first_try = false;
            self.misses += 1;
            self.shake[i] = 1.0;
            ctx.wrong(&format!("That one is {}. Try again!", self.buckets[i].name()));
        }
    }
}

impl MiniGame for ColorSort {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        // Show how to play: a hand drags the object down toward the buckets.
        self.demo.start(2.0);
        ctx.voice.then(&format!(
            "Drag the {} {} into the bucket with the same color!",
            self.target.name(),
            self.thing.name()
        ));
    }

    fn prompt(&self) -> String {
        format!("Where does the {} {} go?", self.target.name(), self.thing.name())
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.shake, ctx.dt, 2.5);
        fade(&mut self.hop, ctx.dt, 3.0);
        self.demo.update(ctx);
        let rects = bucket_rects(self.buckets.len());

        match self.phase {
            Phase::Celebrating(t) => {
                // Fly the object into its bucket.
                let r = rects[self.chosen];
                let into = vec2(r.center().x, r.y + r.h * 0.2);
                self.item_pos = self.item_pos.lerp(into, (ctx.dt * 8.0).min(1.0));
                let t = t - ctx.dt;
                if celebration_over(t, ctx) {
                    self.new_round();
                    ctx.voice.then(&self.prompt());
                } else {
                    self.phase = Phase::Celebrating(t);
                }
            }
            Phase::Playing => {
                let input = ctx.input;
                if input.pressed {
                    if input.pos.distance(self.item_pos) < item_size() * 1.3 {
                        self.dragging = true;
                        self.grab_offset = self.item_pos - input.pos;
                        ctx.sfx.pop();
                        ctx.voice.say(&format!("{} {}!", self.target.name(), self.thing.name()));
                    } else if let Some(i) = rects.iter().position(|r| r.contains(input.pos)) {
                        // Tapping a bucket just says its color. Answering means dragging.
                        self.hop[i] = 1.0;
                        ctx.sfx.pop();
                        ctx.voice.say(&format!("{}!", capitalize(self.buckets[i].name())));
                    }
                }
                if self.dragging {
                    if input.down {
                        self.item_pos = input.pos + self.grab_offset;
                    } else {
                        self.dragging = false;
                        if let Some(i) = rects.iter().position(|r| r.contains(self.item_pos)) {
                            self.choose(i, ctx);
                        }
                    }
                }
                if !self.dragging && self.phase == Phase::Playing {
                    // Spring back home when let go.
                    self.item_pos = self.item_pos.lerp(home(), (ctx.dt * 10.0).min(1.0));
                }
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let rects = bucket_rects(self.buckets.len());
        for (i, (r, paint)) in rects.iter().zip(&self.buckets).enumerate() {
            let mut r = *r;
            r.x += art::shake_x(self.shake[i], ctx.time);
            // Hint: after two misses the right bucket starts bouncing.
            if self.misses >= 2 && *paint == self.target && self.phase == Phase::Playing {
                r.y -= (ctx.time * 8.0).sin().abs() * r.h * 0.08;
            }
            r.y -= (self.hop[i] * std::f32::consts::PI).sin() * r.h * 0.1;
            art::bucket(r, paint.color());
            // The color's name on the bucket, e.g. "RED".
            let label_c = vec2(r.center().x, r.y + r.h * 0.8);
            let color = art::readable(paint.color());
            art::word_label(ctx.font(), paint.name(), label_c, r.w * 0.14, r.w * 0.6, color, Some(WHITE));
        }

        let s = item_size();
        let (bob, scale) = match self.phase {
            Phase::Celebrating(t) => (0.0, 0.5 + 0.5 * (t / CELEBRATE_SECS).max(0.0)),
            Phase::Playing if self.dragging => (0.0, 1.15),
            Phase::Playing => ((ctx.time * 3.0).sin() * s * 0.08, 1.0),
        };
        // During the how-to-play demo, the hand carries the object toward the buckets.
        let demo_to = vec2(screen_width() / 2.0, screen_height() * 0.6);
        let pos = self.demo.carry(home(), demo_to).unwrap_or(self.item_pos + vec2(0.0, bob));
        art::draw_thing(self.thing, pos, s * scale, self.target.color());
        // The object's name underneath, e.g. "BALL".
        if self.phase == Phase::Playing {
            let color = art::readable(self.target.color());
            let label_c = pos + vec2(0.0, s * scale * 1.35);
            art::word_label(ctx.font(), self.thing.name(), label_c, s * 0.38, s * 3.0, color, Some(WHITE));
        }
        self.demo.draw(home(), demo_to);
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> (String, String) {
        ((level + 1).to_string(), "colors".to_string())
    }
}

fn item_size() -> f32 {
    screen_height() * 0.11
}

/// Where the object waits to be sorted.
fn home() -> Vec2 {
    vec2(screen_width() / 2.0, screen_height() * 0.36)
}

fn bucket_rects(n: usize) -> Vec<Rect> {
    let (w, h) = (screen_width(), screen_height());
    let slot = w * 0.9 / n as f32;
    let bw = (slot * 0.8).min(h * 0.3);
    let bh = h * 0.27;
    (0..n)
        .map(|i| {
            let cx = w * 0.05 + slot * (i as f32 + 0.5);
            Rect::new(cx - bw / 2.0, h * 0.64, bw, bh)
        })
        .collect()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
