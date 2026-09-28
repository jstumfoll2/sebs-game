//! Color sorting: lots of objects in different colors, and a bucket for each color.
//! Drag every object into the bucket that matches its color. Tapping an object says what it
//! is ("red ball"); tapping a bucket says its color. The voice names colors on each drop, to
//! connect colors with their names.

use super::{celebration_over, fade, pick, shuffle, CardDrag, Demo, DragEvent, MiniGame, Phase, Progress};
use crate::alphabet::capitalize;
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
use macroquad::prelude::*;

/// Easy-to-tell-apart colors used for the first levels.
const EASY_COLORS: [Paint; 4] = [Paint::Red, Paint::Blue, Paint::Yellow, Paint::Green];
/// For each level: how many buckets, and how many objects to sort.
const LEVELS: [(usize, usize); 5] = [(2, 4), (3, 5), (4, 6), (5, 7), (6, 8)];
const CELEBRATE_SECS: f32 = 1.5;

/// One object to sort.
#[derive(Clone, Copy, Debug)]
struct Item {
    thing: Thing,
    paint: Paint,
    /// Where it sits (0..1 across the play area).
    spot: Vec2,
    /// Which bucket it's been sorted into, once it has.
    sorted: Option<usize>,
}

pub struct ColorSort {
    progress: Progress,
    buckets: Vec<Paint>,
    items: Vec<Item>,
    drag: CardDrag,
    demo: Demo,
    first_try: bool,
    misses: u32,
    /// Buckets wiggle for a wrong drop and hop when tapped or filled.
    shake: Vec<f32>,
    hop: Vec<f32>,
    /// Objects wiggle when dropped in the wrong bucket.
    item_shake: Vec<f32>,
    phase: Phase,
}

impl ColorSort {
    pub fn new() -> Self {
        let mut game = ColorSort {
            progress: Progress::new(LEVELS.len() as u32),
            buckets: Vec::new(),
            items: Vec::new(),
            drag: CardDrag::default(),
            demo: Demo::default(),
            first_try: true,
            misses: 0,
            shake: Vec::new(),
            hop: Vec::new(),
            item_shake: Vec::new(),
            phase: Phase::Playing,
        };
        game.new_round();
        game
    }

    fn new_round(&mut self) {
        let (count, n_items) = LEVELS[(self.progress.level - 1) as usize];
        let mut pool = if self.progress.level <= 2 { EASY_COLORS.to_vec() } else { Paint::ALL.to_vec() };
        shuffle(&mut pool);
        pool.truncate(count);
        self.buckets = pool;

        // Every bucket gets at least one object; the rest are random colors.
        let mut paints: Vec<Paint> = self.buckets.clone();
        while paints.len() < n_items {
            paints.push(pick(&self.buckets));
        }
        shuffle(&mut paints);

        // Scatter the objects on a loose grid so they don't overlap.
        let cols = n_items.div_ceil(2);
        let mut cells: Vec<usize> = (0..cols * 2).collect();
        shuffle(&mut cells);
        self.items = paints
            .into_iter()
            .zip(cells)
            .map(|(paint, cell)| {
                let jitter = || rand::gen_range(-0.12, 0.12);
                Item {
                    thing: pick(&Thing::ALL),
                    paint,
                    spot: vec2(
                        ((cell % cols) as f32 + 0.5 + jitter()) / cols as f32,
                        ((cell / cols) as f32 + 0.5 + jitter()) / 2.0,
                    ),
                    sorted: None,
                }
            })
            .collect();

        self.drag.reset();
        self.first_try = true;
        self.misses = 0;
        self.shake = vec![0.0; count];
        self.hop = vec![0.0; count];
        self.item_shake = vec![0.0; self.items.len()];
        self.phase = Phase::Playing;
    }

    /// Where the objects wait to be sorted.
    fn play_area() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        Rect::new(w * 0.08, h * 0.15, w * 0.84, h * 0.42)
    }

    fn item_size() -> f32 {
        screen_height() * 0.075
    }

    /// Each object's "home" spot as a square (sorted ones are parked out of reach).
    fn item_homes(&self) -> Vec<Rect> {
        let a = Self::play_area();
        let s = Self::item_size() * 2.4;
        self.items
            .iter()
            .map(|it| {
                if it.sorted.is_some() {
                    return Rect::new(-10_000.0, -10_000.0, s, s);
                }
                let c = vec2(a.x + it.spot.x * a.w, a.y + it.spot.y * a.h);
                Rect::new(c.x - s / 2.0, c.y - s / 2.0, s, s)
            })
            .collect()
    }

    fn name(item: &Item) -> String {
        format!("{} {}", item.paint.name(), item.thing.name())
    }

    fn remaining(&self) -> usize {
        self.items.iter().filter(|it| it.sorted.is_none()).count()
    }
}

impl MiniGame for ColorSort {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        // Show how to play: a hand drags an object down toward the buckets.
        self.demo.start(2.0);
        ctx.voice.then("Sort them by color! Drag each one into the bucket with the same color.");
    }

    fn prompt(&self) -> String {
        "Drag each one into the bucket with the same color!".to_string()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.shake, ctx.dt, 2.5);
        fade(&mut self.hop, ctx.dt, 3.0);
        fade(&mut self.item_shake, ctx.dt, 2.5);
        self.demo.update(ctx);
        let rects = bucket_rects(self.buckets.len());

        if let Phase::Celebrating(t) = self.phase {
            let t = t - ctx.dt;
            if celebration_over(t, ctx) {
                self.new_round();
                ctx.voice.then("Here are some more! Sort them by color.");
            } else {
                self.phase = Phase::Celebrating(t);
            }
            return;
        }

        // Tapping a bucket says its color (answering is done by dragging).
        if self.drag.held().is_none() {
            if let Some(i) = rects.iter().position(|r| ctx.input.tapped(*r)) {
                self.hop[i] = 1.0;
                ctx.sfx.pop();
                ctx.voice.say(&format!("{}!", capitalize(self.buckets[i].name())));
                return;
            }
        }

        let homes = self.item_homes();
        match self.drag.update(&ctx.input, &homes, ctx.dt) {
            DragEvent::PickedUp(i) => {
                ctx.sfx.pop();
                ctx.voice.say(&format!("{}!", Self::name(&self.items[i])));
            }
            DragEvent::Dropped(i, at) => {
                let Some(b) = rects.iter().position(|r| art::scale_rect(*r, 1.1).contains(at)) else {
                    return; // dropped somewhere else: it just slides back
                };
                let item = self.items[i];
                if self.buckets[b] == item.paint {
                    self.items[i].sorted = Some(b);
                    self.hop[b] = 1.0;
                    ctx.sfx.ding();
                    if self.remaining() == 0 {
                        let leveled = self.progress.record(self.first_try);
                        ctx.correct(rects[b].center(), "You sorted them all!", leveled);
                        self.phase = Phase::Celebrating(CELEBRATE_SECS);
                    } else {
                        ctx.voice.say(&format!("{}!", capitalize(item.paint.name())));
                    }
                } else {
                    self.first_try = false;
                    self.misses += 1;
                    self.shake[b] = 1.0;
                    self.item_shake[i] = 1.0;
                    ctx.wrong(&format!(
                        "That's the {} bucket. The {} is {}!",
                        self.buckets[b].name(),
                        item.thing.name(),
                        item.paint.name()
                    ));
                }
            }
            DragEvent::Nothing => {}
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let rects = bucket_rects(self.buckets.len());
        let s = Self::item_size();
        let held = self.drag.held().map(|i| self.items[i].paint);

        for (b, (r, paint)) in rects.iter().zip(&self.buckets).enumerate() {
            let mut r = *r;
            r.x += art::shake_x(self.shake[b], ctx.time);
            r.y -= (self.hop[b] * std::f32::consts::PI).sin() * r.h * 0.1;
            // Hint: after two misses, the right bucket for the object being dragged bounces.
            if self.misses >= 2 && held == Some(*paint) {
                r.y -= (ctx.time * 8.0).sin().abs() * r.h * 0.08;
            }
            // Sorted objects peek out over the rim (drawn first, so the bucket covers them).
            let inside: Vec<&Item> = self.items.iter().filter(|it| it.sorted == Some(b)).collect();
            for (k, it) in inside.iter().enumerate() {
                let x = r.center().x + (k as f32 - (inside.len() as f32 - 1.0) / 2.0) * r.w * 0.22;
                art::draw_thing(it.thing, vec2(x, r.y + r.h * 0.08), s * 0.55, it.paint.color());
            }
            art::bucket(r, paint.color());
            let label_c = vec2(r.center().x, r.y + r.h * 0.8);
            art::word_label(ctx.font(), paint.name(), label_c, r.w * 0.14, r.w * 0.6, art::readable(paint.color()), Some(WHITE));
        }

        // The objects still to sort. The one being dragged is drawn last, on top.
        let homes = self.item_homes();
        let mut order: Vec<usize> =
            (0..self.items.len()).filter(|&i| self.items[i].sorted.is_none() && Some(i) != self.drag.held()).collect();
        order.extend(self.drag.held());
        for i in order {
            let it = &self.items[i];
            let r = self.drag.rect(i, homes[i]);
            let held = self.drag.held() == Some(i);
            let bob = if held { 0.0 } else { (ctx.time * 3.0 + i as f32).sin() * s * 0.06 };
            let c = r.center() + vec2(art::shake_x(self.item_shake[i], ctx.time), bob);
            art::draw_thing(it.thing, c, s * if held { 1.15 } else { 1.0 }, it.paint.color());
        }

        // How-to-play: a hand carries the first object down toward the buckets.
        if let Some(first) = homes.iter().find(|r| r.x > -1000.0) {
            let to = vec2(screen_width() / 2.0, rects[0].y);
            if let (Some(p), Some(it)) = (self.demo.carry(first.center(), to), self.items.iter().find(|it| it.sorted.is_none())) {
                art::draw_thing(it.thing, p, s, it.paint.color());
            }
            self.demo.draw(first.center(), to);
        }
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        let (colors, items) = LEVELS[(level - 1) as usize];
        format!("{colors} colors, {items} things")
    }
}

fn bucket_rects(n: usize) -> Vec<Rect> {
    let (w, h) = (screen_width(), screen_height());
    let slot = w * 0.9 / n as f32;
    let bw = (slot * 0.8).min(h * 0.28);
    let bh = h * 0.27;
    (0..n)
        .map(|i| {
            let cx = w * 0.05 + slot * (i as f32 + 0.5);
            Rect::new(cx - bw / 2.0, h * 0.66, bw, bh)
        })
        .collect()
}
