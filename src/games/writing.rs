//! Writing practice. Trace with your finger; the game watches how close you stay to the line.
//!
//! Levels:
//! 1-3. Dotted lines, curves and shapes to trace. A hand shows where to start and which way to go.
//! 4-5. Dotted letters. The hand draws each stroke in order (1, 2, 3...) before you try.
//! 6-7. A letter with part of it missing: finish it. The dots only appear if you need help.
//! 8.   A blank page and a small picture of the letter: write the whole thing, then tap the check.
//!
//! Feedback, while you draw: the ink is green where you're on the line and orange where you
//! drifted. When you lift your finger and it isn't quite right, the orange bits get a red glow,
//! the parts of the line you missed light up gold, and a voice says what to fix. Every miss
//! makes the path a little wider, and after two the hand shows the stroke again. Nothing is
//! ever taken away: you just try that stroke again.

use super::{celebration_over, pick, MiniGame, Phase, Progress};
use crate::alphabet::{self, capitalize};
use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::glyphs::{self, Glyph};
use crate::input::Input;
use crate::tracing;
use macroquad::prelude::*;
use std::cell::Cell;

const LEVELS: u32 = 8;
const LEVEL_NAMES: [&str; LEVELS as usize] =
    ["lines", "curves", "shapes", "easy ABC", "all ABC", "finish it", "half blank", "write it"];
/// The letters made of straight lines, for the first letter level.
const EASY_LETTERS: &str = "ILTHEFXZ";
/// How far from the line the finger can be and still count, as a fraction of the height of
/// what's being written. It gets a little stricter as the levels go up.
const TOLERANCE: [f32; LEVELS as usize] = [0.09, 0.09, 0.09, 0.085, 0.08, 0.075, 0.075, 0.12];
/// Each miss widens the path by this much (up to three misses), so it never gets frustrating.
const WIDEN_PER_MISS: f32 = 0.3;
const CELEBRATE_SECS: f32 = 1.4;
/// How long the "here's what went wrong" picture stays before the try is cleared.
const FEEDBACK_SECS: f32 = 3.5;
/// If nothing is touched for this long, the hand shows the stroke again.
const IDLE_REPLAY_SECS: f32 = 9.0;
/// Pause between strokes in the demo.
const DEMO_PAUSE: f32 = 0.45;
/// A finger path shorter than this (as a fraction of the height) was just a tap.
const TAP_LENGTH: f32 = 0.1;
/// Not green (right) or orange (off the line), so the colors don't mean two things.
const ROUND_COLORS: [Paint; 4] = [Paint::Red, Paint::Blue, Paint::Purple, Paint::Pink];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Style {
    /// Trace dotted lines, one stroke at a time, in order.
    Dotted,
    /// Some strokes are already there; draw the rest.
    FillIn,
    /// Write it all, on a blank page.
    Free,
}

fn style(level: u32) -> Style {
    match level {
        1..=5 => Style::Dotted,
        6 | 7 => Style::FillIn,
        _ => Style::Free,
    }
}

enum Subject {
    Shape { name: &'static str, say: &'static str },
    Letter(char),
}

/// One point of the finger's path, and whether it's off the line.
struct Ink {
    pos: Vec2,
    off: bool,
}

/// What went wrong with the last try, drawn on the page for a few seconds.
struct Feedback {
    secs: f32,
    /// Spots on the line the finger never reached (they glow gold).
    missed: Vec<Vec2>,
}

/// The hand drawing strokes `from` up to (not including) `to`.
struct DemoRun {
    from: usize,
    to: usize,
    t: f32,
}

pub struct Writing {
    progress: Progress,
    subject: Subject,
    /// What to write. For fill-in letters, a one-stroke letter is cut in two pieces here.
    glyph: Glyph,
    /// Strokes that are already drawn in (fill-in levels).
    given: Vec<bool>,
    /// Strokes the player has finished.
    done: Vec<bool>,
    /// Failed tries at each stroke (for hints and the widening path).
    misses: Vec<u32>,
    total_misses: u32,
    color: Color,
    /// The try in progress, or the last failed one while its feedback shows.
    ink: Vec<Ink>,
    /// Free-writing levels: the strokes drawn so far (before the check is tapped).
    finished: Vec<Vec<Ink>>,
    drawing: bool,
    feedback: Option<Feedback>,
    demo: Option<DemoRun>,
    /// Set by `prompt` (which only gets `&self`) to ask for the demo to play again.
    replay: Cell<bool>,
    idle: f32,
    told_slide: bool,
    /// The last thing we asked for, so it isn't picked twice in a row.
    last_key: String,
    phase: Phase,
}

impl Writing {
    pub fn new() -> Self {
        let mut game = Writing {
            progress: Progress::new(LEVELS),
            subject: Subject::Letter('S'),
            glyph: Glyph { strokes: Vec::new(), aspect: 1.0, joined: None },
            given: Vec::new(),
            done: Vec::new(),
            misses: Vec::new(),
            total_misses: 0,
            color: Paint::Red.color(),
            ink: Vec::new(),
            finished: Vec::new(),
            drawing: false,
            feedback: None,
            demo: None,
            replay: Cell::new(false),
            idle: 0.0,
            told_slide: false,
            last_key: String::new(),
            phase: Phase::Playing,
        };
        game.new_round();
        game
    }

    fn level(&self) -> u32 {
        self.progress.level
    }

    fn style(&self) -> Style {
        style(self.level())
    }

    fn new_round(&mut self) {
        let level = self.level();
        let last = std::mem::take(&mut self.last_key);

        let (subject, mut glyph, key) = if level <= 3 {
            let mut items = glyphs::shapes(level);
            items.retain(|i| i.say != last);
            let item = items.swap_remove(macroquad::rand::gen_range(0, items.len()));
            (Subject::Shape { name: item.name, say: item.say }, item.glyph, item.say.to_string())
        } else {
            let pool = if level == 4 { EASY_LETTERS } else { super::letters::LETTER_ORDER };
            let choices: Vec<char> = pool.chars().filter(|c| c.to_string() != last).collect();
            let c = pick(&choices);
            (Subject::Letter(c), glyphs::letter(c).expect("letters are A-Z"), c.to_string())
        };

        // Fill-in levels: draw some of it in already. A letter that's one stroke (like S) is
        // cut in two, and the first piece is the one that's drawn in.
        let mut given = vec![false; glyph.strokes.len()];
        if style(level) == Style::FillIn {
            if glyph.strokes.len() == 1 {
                let cut = if level == 6 { 0.5 } else { 0.3 };
                let whole = glyph.strokes.remove(0);
                glyph.strokes = vec![tracing::trim(&whole, 0.0, cut), tracing::trim(&whole, cut, 1.0)];
                given = vec![true, false];
            } else {
                // Level 6 leaves only the last stroke to draw; level 7 all but the first.
                let n = given.len();
                let drawn_in = if level == 6 { n - 1 } else { 1 };
                given[..drawn_in].fill(true);
            }
        }

        self.color = pick(&ROUND_COLORS).color();
        self.done = vec![false; glyph.strokes.len()];
        self.misses = vec![0; glyph.strokes.len()];
        self.given = given;
        self.subject = subject;
        self.last_key = key;
        self.total_misses = 0;
        self.ink.clear();
        self.finished.clear();
        self.drawing = false;
        self.feedback = None;
        self.idle = 0.0;
        self.told_slide = false;
        self.phase = Phase::Playing;
        // Dotted levels start with the hand drawing the whole thing, in order.
        self.demo = (style(level) == Style::Dotted).then_some(DemoRun { from: 0, to: glyph.strokes.len(), t: 0.0 });
        self.glyph = glyph;
    }

    // ---------- where things are ----------

    /// The big white page.
    fn board() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        Rect::new(w * 0.13, h * 0.17, w * 0.74, h * 0.79)
    }

    /// The box the letter or shape is written in (centered on the page).
    fn glyph_box(&self) -> Rect {
        let b = Self::board();
        let h = b.h * 0.8;
        let w = (h * self.glyph.aspect).min(b.w * 0.6);
        Rect::new(b.center().x - w / 2.0, b.center().y - h / 2.0, w, h)
    }

    /// Every stroke in screen pixels.
    fn strokes_px(&self) -> Vec<Vec<Vec2>> {
        let b = self.glyph_box();
        self.glyph
            .strokes
            .iter()
            .map(|s| s.iter().map(|p| vec2(b.x + p.x * b.w, b.y + p.y * b.h)).collect())
            .collect()
    }

    /// The stroke to draw next: the first one that isn't drawn in or finished yet.
    fn current(&self) -> Option<usize> {
        if self.style() == Style::Free {
            return None;
        }
        (0..self.done.len()).find(|&i| !self.given[i] && !self.done[i])
    }

    /// The number shown at the start of stroke `i`: its place among the strokes to draw.
    fn number_of(&self, i: usize) -> usize {
        1 + (0..i).filter(|&j| !self.given[j]).count()
    }

    /// How far off the line counts as off, in pixels. Widens after misses.
    fn tol(&self, stroke: Option<usize>) -> f32 {
        let misses = stroke.map_or(self.total_misses, |i| self.misses[i]);
        let base = self.glyph_box().h * TOLERANCE[self.level() as usize - 1];
        base * (1.0 + WIDEN_PER_MISS * misses.min(3) as f32)
    }

    fn ink_thickness(&self) -> f32 {
        self.glyph_box().h * 0.045
    }

    fn ref_rect() -> Rect {
        let b = Self::board();
        let s = b.h * 0.22;
        Rect::new(b.x + b.w * 0.03, b.y + b.h * 0.08, s, s)
    }

    fn done_rect() -> Rect {
        let b = Self::board();
        let s = b.h * 0.16;
        Rect::new(b.right() - s * 1.3, b.center().y - s * 1.1, s, s)
    }

    fn again_rect() -> Rect {
        let done = Self::done_rect();
        Rect::new(done.x, done.y + done.h * 1.2, done.w, done.h)
    }

    // ---------- the finger ----------

    /// Add a point to the try, filling in any gap so fast strokes still have a point every few pixels.
    fn push_ink(&mut self, p: Vec2) {
        const SPACING: f32 = 6.0;
        let new_points = match self.ink.last().map(|i| i.pos) {
            Some(last) if last.distance(p) < 1.5 => return,
            Some(last) if last.distance(p) > SPACING => {
                let n = (last.distance(p) / SPACING).ceil() as usize;
                (1..=n).map(|k| last.lerp(p, k as f32 / n as f32)).collect()
            }
            _ => vec![p],
        };
        let strokes = self.strokes_px();
        let current = self.current();
        let tol = self.tol(current);
        for pos in new_points {
            // Off the line means off the stroke we're drawing (or, freehand, off every stroke).
            let dist = match current {
                Some(i) => tracing::project(&strokes[i], pos).0,
                None => tracing::distance_to_any(&strokes, pos),
            };
            self.ink.push(Ink { pos, off: dist > tol });
        }
    }

    fn ink_points(ink: &[Ink]) -> Vec<Vec2> {
        ink.iter().map(|i| i.pos).collect()
    }

    /// The finger was lifted.
    fn finish_stroke(&mut self, ctx: &mut Ctx) {
        let points = Self::ink_points(&self.ink);
        if tracing::length(&points) < self.glyph_box().h * TAP_LENGTH {
            // Just a tap, not a stroke.
            self.ink.clear();
            ctx.sfx.pop();
            if self.style() != Style::Free && !self.told_slide {
                self.told_slide = true;
                ctx.voice.say("Slide your finger along the line!");
            }
            return;
        }
        match self.current() {
            Some(i) => self.judge_current(i, &points, ctx),
            None => {
                // Freehand: keep the stroke; if the whole thing is already right, we're done.
                self.finished.push(std::mem::take(&mut self.ink));
                let verdict = self.judge_all();
                if verdict.passed {
                    self.succeed(ctx);
                }
            }
        }
    }

    fn judge_all(&self) -> tracing::Verdict {
        let inks: Vec<Vec<Vec2>> = self.finished.iter().map(|s| Self::ink_points(s)).collect();
        tracing::judge_free(&self.strokes_px(), &inks, self.tol(None))
    }

    /// Did the finger follow stroke `i`?
    fn judge_current(&mut self, i: usize, points: &[Vec2], ctx: &mut Ctx) {
        let strokes = self.strokes_px();
        let tol = self.tol(Some(i));
        let verdict = tracing::judge_stroke(&strokes[i], points, tol);
        // N and M can also be written in one go. If the first stroke doesn't fit the first
        // line but does fit the one-stroke version, switch to that version.
        if !verdict.passed && self.style() == Style::Dotted && self.done.iter().all(|d| !d) {
            if let Some(joined) = self.glyph.joined.clone() {
                let b = self.glyph_box();
                let joined_px: Vec<Vec2> = joined.iter().map(|p| vec2(b.x + p.x * b.w, b.y + p.y * b.h)).collect();
                if tracing::judge_stroke(&joined_px, points, tol).passed {
                    self.glyph.strokes = vec![joined];
                    self.glyph.joined = None;
                    self.given = vec![false];
                    self.done = vec![false];
                    self.misses = vec![0];
                    self.done[0] = true;
                    self.ink.clear();
                    self.succeed(ctx);
                    return;
                }
            }
        }
        if verdict.passed {
            self.done[i] = true;
            self.ink.clear();
            ctx.confetti.burst(*strokes[i].last().unwrap(), 14);
            match self.current() {
                Some(next) => {
                    let praise = pick(&["Nice!", "Good!", "That's it!", "Yes!"]);
                    let words = if self.style() == Style::Dotted {
                        format!("{praise} Now number {}!", self.number_of(next))
                    } else {
                        format!("{praise} Now the next part!")
                    };
                    ctx.sfx.pop();
                    ctx.voice.say(&words);
                }
                None => self.succeed(ctx),
            }
            return;
        }

        self.misses[i] += 1;
        self.total_misses += 1;
        let number = self.number_of(i);
        let only_one = (0..self.done.len()).filter(|&j| !self.given[j]).count() == 1;
        let start = if only_one { "the dot".to_string() } else { format!("number {number}") };

        // Did they draw a different stroke of the letter instead of this one?
        let other = (0..strokes.len())
            .filter(|&j| j != i && !self.given[j] && !self.done[j])
            .find(|&j| tracing::coverage(&strokes[j], points, tol) >= 0.75);
        let words = if let (Some(j), false) = (other, verdict.start_ok) {
            format!("That's number {}! Start with number {number}.", self.number_of(j))
        } else if !verdict.start_ok {
            format!("Start at {start}! Put your finger right on it.")
        } else if !verdict.direction_ok {
            "Go this way! Follow the arrow.".to_string()
        } else if verdict.stray > 0.2 {
            "Oops, you went off the line! Stay on the dots.".to_string()
        } else {
            "Keep going! Follow the dots all the way to the end.".to_string()
        };
        ctx.wrong(&words);
        self.feedback = Some(Feedback { secs: FEEDBACK_SECS, missed: verdict.missed });
        // Second miss in a row: show how it's done again.
        if self.misses[i] == 2 {
            self.demo = Some(DemoRun { from: i, to: i + 1, t: 0.0 });
        }
    }

    /// The check button on the free-writing levels.
    fn check_free(&mut self, ctx: &mut Ctx) {
        if self.finished.is_empty() {
            ctx.voice.say("Write the letter first!");
            return;
        }
        let verdict = self.judge_all();
        if verdict.passed {
            self.succeed(ctx);
            return;
        }
        self.total_misses += 1;
        let words = if verdict.stray > tracing::MAX_STRAY_FREE && verdict.coverage < tracing::MIN_COVERAGE_FREE {
            "Almost! Some parts went off the line, and the glowing parts are missing."
        } else if verdict.stray > tracing::MAX_STRAY_FREE {
            "Almost! Some parts went off the line. Look at the red spots."
        } else {
            "Almost! The glowing parts are missing. Look at the dots to see where to go."
        };
        ctx.wrong(words);
        self.feedback = Some(Feedback { secs: FEEDBACK_SECS, missed: verdict.missed });
    }

    /// Everything is written!
    fn succeed(&mut self, ctx: &mut Ctx) {
        self.done.fill(true);
        self.ink.clear();
        self.finished.clear();
        self.feedback = None;
        self.demo = None;
        // Up to one retry still counts as getting it the first time.
        let leveled = self.progress.record(self.total_misses <= 1);
        let words = match &self.subject {
            Subject::Shape { name, .. } => format!("That's a nice {name}!"),
            Subject::Letter(c) => {
                let l = alphabet::get(*c).expect("letters are A-Z");
                format!("{}! {}", capitalize(l.name), l.teach(&ctx.name))
            }
        };
        ctx.correct(self.glyph_box().center(), &words, leveled);
        self.phase = Phase::Celebrating(CELEBRATE_SECS);
    }

    // ---------- the demo hand ----------

    /// Seconds the hand takes to draw each stroke from `from` up to `to`.
    fn demo_durations(&self, strokes: &[Vec<Vec2>], from: usize, to: usize) -> Vec<f32> {
        let h = self.glyph_box().h;
        (from..to).map(|i| (tracing::length(&strokes[i]) / h * 1.6 + 0.5).clamp(0.8, 2.8)).collect()
    }

    fn start_demo(&mut self) {
        let n = self.glyph.strokes.len();
        self.demo = match (self.style(), self.current()) {
            (Style::Dotted, _) if self.done.iter().all(|d| !d) => Some(DemoRun { from: 0, to: n, t: 0.0 }),
            (_, Some(i)) => Some(DemoRun { from: i, to: i + 1, t: 0.0 }),
            _ => None,
        };
    }

    /// Developer helper: `--fake-write` makes a pretend finger trace the first stroke and drift
    /// off the line halfway, so the feedback can be checked with `--snapshot` (no touch screen needed).
    fn fake_finger(&self, time: f32, dt: f32, input: Input) -> Input {
        let strokes = self.strokes_px();
        let cur = self.current();
        let Some(target) = strokes.get(cur.unwrap_or(0)) else { return input };
        if !(1.0..3.0).contains(&time) {
            return Input { pressed: false, down: false, ..input };
        }
        let f = (time - 1.0) / 2.0 * 0.75;
        let p = tracing::point_at(target, f);
        let dir = (tracing::point_at(target, f + 0.01) - p).normalize_or_zero();
        let bump = if (0.25..0.45).contains(&f) { self.tol(cur) * 2.2 * ((f - 0.25) / 0.2 * std::f32::consts::PI).sin() } else { 0.0 };
        Input { pos: p + dir.perp() * bump, pressed: time - dt < 1.0, down: true }
    }

    // ---------- the game loop ----------

    fn play(&mut self, ctx: &mut Ctx, input: Input) {
        let free = self.style() == Style::Free;

        // The feedback picture fades away by itself, taking the failed try with it.
        let expired = match &mut self.feedback {
            Some(fb) => {
                fb.secs -= ctx.dt;
                fb.secs <= 0.0
            }
            None => false,
        };
        if expired {
            self.feedback = None;
            self.ink.clear();
            self.finished.clear();
        }

        // The hand.
        if self.replay.replace(false) {
            self.start_demo();
        }
        if let Some(run) = &mut self.demo {
            run.t += ctx.dt;
        }
        if let Some(run) = &self.demo {
            let strokes = self.strokes_px();
            let total: f32 = self.demo_durations(&strokes, run.from, run.to).iter().map(|d| d + DEMO_PAUSE).sum();
            if run.t >= total {
                self.demo = None;
            }
        }
        self.idle = if input.down { 0.0 } else { self.idle + ctx.dt };
        if self.idle > IDLE_REPLAY_SECS && self.demo.is_none() && !free {
            self.idle = 0.0;
            self.start_demo();
        }

        if input.pressed {
            if free {
                if Self::done_rect().contains(input.pos) {
                    ctx.sfx.pop();
                    self.check_free(ctx);
                    return;
                }
                if Self::again_rect().contains(input.pos) {
                    ctx.sfx.pop();
                    self.ink.clear();
                    self.finished.clear();
                    self.feedback = None;
                    return;
                }
                if let (Subject::Letter(c), true) = (&self.subject, Self::ref_rect().contains(input.pos)) {
                    // Tapping the little letter says it, like in the Letters game.
                    ctx.sfx.pop();
                    ctx.voice.say(&alphabet::get(*c).expect("letters are A-Z").teach(&ctx.name));
                    return;
                }
            }
            if Self::board().contains(input.pos) {
                // A new try. (A failed try is wiped; strokes already written freehand stay.)
                self.demo = None;
                if self.feedback.take().is_some() {
                    self.finished.clear();
                }
                self.ink.clear();
                self.drawing = true;
            }
        }
        if self.drawing {
            if input.down {
                self.push_ink(input.pos);
            } else {
                self.drawing = false;
                self.finish_stroke(ctx);
            }
        }
    }
}

impl MiniGame for Writing {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        ctx.voice.then(&self.words());
    }

    fn prompt(&self) -> String {
        // Saying the question again also shows the hand again.
        self.replay.set(true);
        self.words()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        match self.phase {
            Phase::Celebrating(t) => {
                let t = t - ctx.dt;
                if celebration_over(t, ctx) {
                    self.new_round();
                    ctx.voice.then(&self.words());
                } else {
                    self.phase = Phase::Celebrating(t);
                }
            }
            Phase::Playing => {
                let mut input = ctx.input;
                if std::env::args().any(|a| a == "--fake-write") {
                    input = self.fake_finger(ctx.time, ctx.dt, input);
                }
                self.play(ctx, input);
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let board = Self::board();
        // (Not `art::card`: its corners get very round on a card this big.)
        let radius = board.h * 0.05;
        art::rounded_rect(Rect::new(board.x, board.y + 8.0, board.w, board.h), radius, art::SHADOW);
        art::rounded_rect(board, radius, WHITE);
        let gb = self.glyph_box();
        let strokes = self.strokes_px();
        let style = self.style();
        let thick = self.ink_thickness();
        let celebrating = matches!(self.phase, Phase::Celebrating(_));
        let current = self.current();

        // Lined paper for letters: top line, dashed middle line, and a baseline.
        if matches!(self.subject, Subject::Letter(_)) {
            let line_color = art::lighten(Paint::Blue.color(), 0.7);
            let (x0, x1) = (board.x + board.w * 0.2, board.right() - board.w * 0.2);
            for y in [gb.y, gb.bottom()] {
                draw_line(x0, y, x1, y, 3.0, line_color);
            }
            let mut x = x0;
            while x < x1 {
                draw_line(x, gb.center().y, (x + 14.0).min(x1), gb.center().y, 2.0, line_color);
                x += 28.0;
            }
        }

        // Fill-in level 6: a soft yellow window where the missing part goes.
        if style == Style::FillIn && self.level() == 6 && !celebrating {
            let missing: Vec<Vec2> = (0..strokes.len()).filter(|&i| !self.given[i]).flat_map(|i| strokes[i].clone()).collect();
            let pad = self.tol(None) * 1.4;
            let min = missing.iter().fold(vec2(f32::MAX, f32::MAX), |a, p| a.min(*p));
            let max = missing.iter().fold(vec2(f32::MIN, f32::MIN), |a, p| a.max(*p));
            let zone = Rect::new(min.x - pad, min.y - pad, max.x - min.x + pad * 2.0, max.y - min.y + pad * 2.0);
            art::rounded_rect(zone, pad, Color::new(1.0, 0.96, 0.78, 1.0));
        }

        // Strokes that are drawn in (given, or already finished) are solid.
        for (i, s) in strokes.iter().enumerate() {
            if self.given[i] || self.done[i] {
                polyline(s, thick, self.color);
            }
        }

        // The dotted guide, on a soft path that gets wider after misses.
        for (i, s) in strokes.iter().enumerate() {
            let shown = !self.given[i] && !self.done[i] && self.guide_shown(i, current);
            if shown && !celebrating {
                let tol = self.tol((style != Style::Free).then_some(i));
                polyline(s, tol * 1.1, art::lighten(Paint::Blue.color(), 0.82));
                for p in tracing::resample(s, gb.h * 0.045) {
                    draw_circle(p.x, p.y, gb.h * 0.011, art::darken(art::SHADOW, 0.35));
                }
            }
        }

        // What went wrong: the parts of the line that were missed glow gold and pulse...
        if let Some(fb) = &self.feedback {
            let pulse = 1.0 + 0.18 * (ctx.time * 8.0).sin();
            for p in &fb.missed {
                draw_circle(p.x, p.y, thick * 0.95 * pulse, Color::new(1.0, 0.8, 0.1, 0.9));
            }
            // ...and the parts of the try that strayed get a red glow.
            for (k, ink) in self.ink.iter().chain(self.finished.iter().flatten()).enumerate() {
                if ink.off && k % 2 == 0 {
                    draw_circle(ink.pos.x, ink.pos.y, thick * 1.05, Color::new(1.0, 0.2, 0.2, 0.3));
                }
            }
        }

        // The finger's ink: green on the line, orange off it.
        for stroke in self.finished.iter().chain(std::iter::once(&self.ink)) {
            draw_ink(stroke, thick);
        }

        // Where to start: numbers, a pulsing ring and an arrow for the next stroke. Drawn last
        // number first, because strokes often share a start (the stem and top of an F), and
        // the lowest number should be the one on top.
        if !celebrating {
            for i in (0..strokes.len()).rev() {
                if self.given[i] || self.done[i] || !self.start_shown(i) {
                    continue;
                }
                let start = strokes[i][0];
                if current == Some(i) && !self.drawing {
                    let ring = gb.h * 0.06 * (1.0 + 0.2 * (ctx.time * 5.0).sin());
                    draw_circle_lines(start.x, start.y, ring, 4.0, art::GOLD);
                    let ahead = tracing::point_at(&strokes[i], (gb.h * 0.1 / tracing::length(&strokes[i]).max(1.0)).min(0.5));
                    let dir = (ahead - start).normalize_or_zero();
                    arrow(start + dir * gb.h * 0.11, dir, gb.h * 0.03, art::darken(art::SHADOW, 0.45));
                }
                art::badge(font, start, gb.h * 0.032, &self.number_of(i).to_string());
            }
        }

        // The hand showing how to draw.
        if let Some(run) = &self.demo {
            let durations = self.demo_durations(&strokes, run.from, run.to);
            let mut start = 0.0;
            for (k, i) in (run.from..run.to).enumerate() {
                if run.t < start {
                    break;
                }
                let e = ((run.t - start) / durations[k]).clamp(0.0, 1.0);
                let e = e * e * (3.0 - 2.0 * e);
                let trail = tracing::trim(&strokes[i], 0.0, e);
                if trail.len() >= 2 {
                    polyline(&trail, thick * 0.9, art::lighten(self.color, 0.3));
                }
                if run.t < start + durations[k] + DEMO_PAUSE {
                    art::hand(tracing::point_at(&strokes[i], e), screen_height() * 0.07, e < 1.0);
                    break;
                }
                start += durations[k] + DEMO_PAUSE;
            }
        }

        // Free writing: a small picture of the letter, and the "done" and "try again" buttons.
        if style == Style::Free {
            self.draw_free_controls();
        }
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        LEVEL_NAMES[level as usize - 1].to_string()
    }
}

impl Writing {
    /// The spoken instruction.
    fn words(&self) -> String {
        match &self.subject {
            Subject::Shape { say, .. } => say.to_string(),
            Subject::Letter(c) => {
                let name = alphabet::get(*c).expect("letters are A-Z").name;
                match self.style() {
                    Style::Dotted => format!("Let's write the letter {name}! Watch how, then trace it with your finger."),
                    Style::FillIn => format!("Finish the letter {name}! Draw the missing part."),
                    Style::Free => format!("Write the letter {name}! Tap the green check when you're done."),
                }
            }
        }
    }

    /// Show the dotted guide for stroke `i` (which isn't drawn yet)?
    fn guide_shown(&self, i: usize, current: Option<usize>) -> bool {
        match self.style() {
            Style::Dotted => true,
            // The fill-in and free levels only help after a miss.
            Style::FillIn => current == Some(i) && self.misses[i] >= 1,
            Style::Free => self.total_misses >= 1,
        }
    }

    /// Show the number (and ring and arrow) where stroke `i` starts?
    fn start_shown(&self, i: usize) -> bool {
        match self.style() {
            Style::Dotted => true,
            Style::FillIn => self.level() == 6 || self.misses[i] >= 1,
            Style::Free => false,
        }
    }

    fn draw_free_controls(&self) {
        // The letter to copy, small, on its own card.
        let r = Self::ref_rect();
        art::card(r, WHITE);
        let box_h = r.h * 0.68;
        let box_w = (box_h * self.glyph.aspect).min(r.w * 0.8);
        for s in &self.glyph.strokes {
            let pts: Vec<Vec2> = s
                .iter()
                .map(|p| vec2(r.center().x - box_w / 2.0 + p.x * box_w, r.center().y - box_h / 2.0 + p.y * box_h))
                .collect();
            polyline(&pts, r.h * 0.05, self.color);
        }

        // Done: a green check. Try again: a blue circular arrow.
        let done = Self::done_rect();
        art::round_button(done.center(), done.w / 2.0);
        draw_circle(done.center().x, done.center().y, done.w * 0.4, Paint::Green.color());
        let (c, t) = (done.center(), done.w * 0.09);
        draw_line(c.x - done.w * 0.2, c.y, c.x - done.w * 0.05, c.y + done.w * 0.16, t, WHITE);
        draw_line(c.x - done.w * 0.05, c.y + done.w * 0.16, c.x + done.w * 0.22, c.y - done.w * 0.15, t, WHITE);

        let again = Self::again_rect();
        art::round_button(again.center(), again.w / 2.0);
        let (c, r) = (again.center(), again.w * 0.22);
        art::arc(c, r, 0.3 * std::f32::consts::PI, 1.8 * std::f32::consts::PI, again.w * 0.08, Paint::Blue.color());
        let tip = c + vec2((0.3 * std::f32::consts::PI).cos(), (0.3 * std::f32::consts::PI).sin()) * r;
        arrow(tip + vec2(again.w * 0.03, again.w * 0.02), vec2(-0.3, 1.0).normalize(), again.w * 0.11, Paint::Blue.color());
    }
}

// ---------- drawing helpers ----------

/// A thick line through the points, with round joins.
fn polyline(points: &[Vec2], thick: f32, color: Color) {
    for w in points.windows(2) {
        draw_line(w[0].x, w[0].y, w[1].x, w[1].y, thick, color);
    }
    for p in points {
        draw_circle(p.x, p.y, thick / 2.0, color);
    }
}

/// The finger's path: green while it's on the line, orange where it wandered off.
fn draw_ink(ink: &[Ink], thick: f32) {
    let (good, bad) = (Paint::Green.color(), Paint::Orange.color());
    let color = |i: &Ink| if i.off { bad } else { good };
    for w in ink.windows(2) {
        draw_line(w[0].pos.x, w[0].pos.y, w[1].pos.x, w[1].pos.y, thick, color(&w[1]));
    }
    for i in ink {
        draw_circle(i.pos.x, i.pos.y, thick / 2.0, color(i));
    }
}

/// A filled arrowhead with its tip at `tip`, pointing along `dir`.
fn arrow(tip: Vec2, dir: Vec2, size: f32, color: Color) {
    let base = tip - dir * size * 1.6;
    let side = dir.perp() * size;
    draw_triangle(tip, base + side, base - side, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Every level makes rounds that can be played: each stroke has its flags, fill-in levels
    /// leave something to draw and something drawn in, and pieces of a cut letter join up.
    #[test]
    fn every_level_makes_a_playable_round() {
        let mut game = Writing::new();
        for level in 1..=LEVELS {
            game.progress.set_level(level);
            for _ in 0..60 {
                game.new_round();
                let n = game.glyph.strokes.len();
                assert!(n >= 1);
                assert_eq!((game.given.len(), game.done.len(), game.misses.len()), (n, n, n));
                let to_draw = game.given.iter().filter(|g| !**g).count();
                match style(level) {
                    Style::Free => assert_eq!((game.current(), to_draw), (None, n)),
                    _ => assert_eq!(game.current(), Some(game.given.iter().position(|g| !g).unwrap())),
                }
                if style(level) == Style::FillIn {
                    assert!(to_draw >= 1 && game.given.iter().any(|g| *g), "level {level}: {:?}", game.given);
                    // The drawn-in strokes come first, so the numbers start at 1.
                    let first = game.given.iter().position(|g| !g).unwrap();
                    assert!(game.given[..first].iter().all(|g| *g) && game.given[first..].iter().all(|g| !*g));
                    assert_eq!(game.number_of(first), 1);
                }
                assert_eq!(game.demo.is_some(), style(level) == Style::Dotted);
            }
        }
    }

    #[test]
    fn a_cut_letter_joins_up() {
        let mut game = Writing::new();
        game.progress.set_level(6);
        // Keep making rounds until an S (one stroke, so it gets cut) comes up.
        for _ in 0..500 {
            game.new_round();
            if matches!(game.subject, Subject::Letter('S')) {
                let (a, b) = (&game.glyph.strokes[0], &game.glyph.strokes[1]);
                assert!(a.last().unwrap().distance(b[0]) < 0.001, "the halves should meet");
                assert_eq!(game.given, vec![true, false]);
                return;
            }
        }
        panic!("never got an S");
    }

    #[test]
    fn never_the_same_thing_twice() {
        let mut game = Writing::new();
        for level in [2, 5] {
            game.progress.set_level(level);
            let mut last = String::new();
            for _ in 0..100 {
                game.new_round();
                assert_ne!(game.last_key, last);
                last = game.last_key.clone();
            }
        }
    }
}
