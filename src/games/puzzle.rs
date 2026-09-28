//! Puzzle: a picture is cut into pieces; drag each piece onto the board where it belongs.
//! Pieces snap into place when they're close. Early levels show a faint copy of the picture
//! on the board as a guide.

use super::{celebration_over, fade, shuffle, CardDrag, Demo, DragEvent, MiniGame, Progress};
use crate::alphabet::{self, Letter};
use crate::art;
use crate::ctx::Ctx;
use crate::render;
use macroquad::prelude::*;

/// Pieces across and down for each level.
const GRIDS: [(usize, usize); 5] = [(2, 1), (2, 2), (3, 2), (3, 3), (4, 3)];
/// Levels up to this one show a faint picture on the board.
const GUIDE_UNTIL_LEVEL: u32 = 3;

pub struct Puzzle {
    progress: Progress,
    /// The picture (and its word) we're building.
    letter: &'static Letter,
    tex: Option<RenderTarget>,
    cols: usize,
    rows: usize,
    /// Which pieces are in place on the board.
    placed: Vec<bool>,
    /// Where each piece waits in the tray (its "slot"), shuffled.
    tray_slot: Vec<usize>,
    drag: CardDrag,
    demo: Demo,
    wiggle: Vec<f32>,
    snap_pop: Vec<f32>,
    first_try: bool,
    /// Seconds left of celebrating a finished puzzle.
    done: Option<f32>,
}

impl Puzzle {
    pub fn new() -> Self {
        let mut game = Puzzle {
            progress: Progress::new(GRIDS.len() as u32),
            letter: &alphabet::LETTERS[0],
            tex: None,
            cols: 2,
            rows: 1,
            placed: Vec::new(),
            tray_slot: Vec::new(),
            drag: CardDrag::default(),
            demo: Demo::default(),
            wiggle: Vec::new(),
            snap_pop: Vec::new(),
            first_try: true,
            done: None,
        };
        game.new_round();
        game
    }

    fn pieces(&self) -> usize {
        self.cols * self.rows
    }

    fn new_round(&mut self) {
        let (cols, rows) = GRIDS[(self.progress.level - 1) as usize];
        self.cols = cols;
        self.rows = rows;
        let previous = self.letter.letter;
        loop {
            self.letter = &alphabet::LETTERS[rand::gen_range(0, alphabet::LETTERS.len())];
            if self.letter.letter != previous {
                break;
            }
        }
        self.tex = None; // drawn at the start of the next update
        let n = self.pieces();
        self.placed = vec![false; n];
        self.tray_slot = (0..n).collect();
        shuffle(&mut self.tray_slot);
        self.drag.reset();
        self.wiggle = vec![0.0; n];
        self.snap_pop = vec![0.0; n];
        self.first_try = true;
        self.done = None;
    }

    /// The square board on the left.
    fn board() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        let side = (h * 0.66).min(w * 0.46);
        Rect::new(w * 0.05, h * 0.2, side, side)
    }

    /// Where piece `i` belongs on the board.
    fn cell(&self, i: usize) -> Rect {
        let b = Self::board();
        let (cw, ch) = (b.w / self.cols as f32, b.h / self.rows as f32);
        Rect::new(b.x + (i % self.cols) as f32 * cw, b.y + (i / self.cols) as f32 * ch, cw, ch)
    }

    /// Where each piece waits before it's placed (the tray on the right).
    fn tray_homes(&self) -> Vec<Rect> {
        let (w, h) = (screen_width(), screen_height());
        let b = Self::board();
        let tray = Rect::new(b.x + b.w + w * 0.05, h * 0.2, w * 0.95 - (b.x + b.w + w * 0.05), h * 0.72);
        // Pieces wait in the tray at a smaller size if they wouldn't all fit full size.
        let full = self.cell(0);
        let mut scale: f32 = 1.0;
        let (per_row, rows) = loop {
            let (sw, sh) = (full.w * scale * 1.12, full.h * scale * 1.12);
            let per_row = ((tray.w / sw).floor() as usize).max(1);
            let rows = self.pieces().div_ceil(per_row);
            if rows as f32 * sh <= tray.h || scale < 0.3 {
                break (per_row, rows);
            }
            scale -= 0.05;
        };
        let cell = Rect::new(0.0, 0.0, full.w * scale, full.h * scale);
        let slot_w = cell.w * 1.12;
        let slot_h = cell.h * 1.12;
        let top = tray.y + (tray.h - rows as f32 * slot_h).max(0.0) / 2.0;
        (0..self.pieces())
            .map(|i| {
                if self.placed[i] {
                    // Placed pieces can't be picked up again: park them far away.
                    return Rect::new(-10_000.0, -10_000.0, cell.w, cell.h);
                }
                let slot = self.tray_slot[i];
                let (col, row) = (slot % per_row, slot / per_row);
                let in_row = (self.pieces() - row * per_row).min(per_row);
                let left = tray.x + (tray.w - in_row as f32 * slot_w) / 2.0;
                Rect::new(
                    left + col as f32 * slot_w + (slot_w - cell.w) / 2.0,
                    top + row as f32 * slot_h + (slot_h - cell.h) / 2.0,
                    cell.w,
                    cell.h,
                )
            })
            .collect()
    }

    /// The part of the picture that piece `i` shows (0..1 units).
    fn source(&self, i: usize) -> Rect {
        let (cw, ch) = (1.0 / self.cols as f32, 1.0 / self.rows as f32);
        Rect::new((i % self.cols) as f32 * cw, (i / self.cols) as f32 * ch, cw, ch)
    }

    fn draw_piece(&self, tex: &Texture2D, i: usize, r: Rect) {
        art::rounded_rect(Rect::new(r.x + 3.0, r.y + 5.0, r.w, r.h), 6.0, art::SHADOW);
        draw_rectangle(r.x, r.y, r.w, r.h, WHITE);
        render::draw_picture(tex, r, self.source(i), WHITE);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, art::darken(art::SHADOW, 0.2));
    }
}

impl MiniGame for Puzzle {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        // Show how to play: a hand drags from the pieces over to the board.
        self.demo.start(2.0);
        ctx.voice.then("Let's make a puzzle! Drag each piece to its spot.");
    }

    fn prompt(&self) -> String {
        "Drag each piece to its spot in the picture.".to_string()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        if self.tex.is_none() {
            self.tex = Some(render::picture_texture_ex(self.letter.picture, 512, true));
        }
        // Developer check: `--solve` puts every piece in its place (the picture should look right).
        if std::env::args().any(|a| a == "--solve") {
            self.placed.fill(true);
        }
        fade(&mut self.wiggle, ctx.dt, 2.5);
        fade(&mut self.snap_pop, ctx.dt, 3.0);
        self.demo.update(ctx);

        if let Some(t) = self.done {
            let t = t - ctx.dt;
            if celebration_over(t, ctx) {
                self.new_round();
                ctx.voice.then("Here's a new puzzle!");
            } else {
                self.done = Some(t);
            }
            return;
        }

        let homes = self.tray_homes();
        match self.drag.update(&ctx.input, &homes, ctx.dt) {
            DragEvent::PickedUp(_) => ctx.sfx.pop(),
            DragEvent::Dropped(i, at) => {
                let target = self.cell(i);
                let close = target.center().distance(at) < target.w.min(target.h) * 0.45;
                if close {
                    self.placed[i] = true;
                    self.snap_pop[i] = 1.0;
                    ctx.sfx.pop();
                    if self.placed.iter().all(|p| *p) {
                        let leveled = self.progress.record(self.first_try);
                        let words = format!("You made {}!", with_article(self.letter.word));
                        ctx.correct(Self::board().center(), &words, leveled);
                        self.done = Some(1.5);
                    }
                } else if Self::board().contains(at) {
                    // On the board but in the wrong spot: wiggle and slide back.
                    self.first_try = false;
                    self.wiggle[i] = 1.0;
                    ctx.sfx.oops();
                }
            }
            DragEvent::Nothing => {}
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let b = Self::board();
        let Some(target) = &self.tex else { return };
        let tex = &target.texture;

        // The board: a white frame, a faint guide picture on early levels, and cell lines.
        let frame = art::scale_rect(b, 1.04);
        art::card(frame, WHITE);
        if self.progress.level <= GUIDE_UNTIL_LEVEL {
            render::draw_picture(tex, b, Rect::new(0.0, 0.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0, 0.25));
        }
        for i in 0..self.pieces() {
            let c = self.cell(i);
            draw_rectangle_lines(c.x, c.y, c.w, c.h, 2.0, art::SHADOW);
        }

        // Pieces already in place.
        for i in (0..self.pieces()).filter(|&i| self.placed[i]) {
            let r = art::scale_rect(self.cell(i), 1.0 + 0.08 * (self.snap_pop[i] * std::f32::consts::PI).sin());
            render::draw_picture(tex, r, self.source(i), WHITE);
        }
        if self.done.is_some() {
            // The finished picture's word underneath: "FISH".
            let c = vec2(b.center().x, b.y + b.h + screen_height() * 0.06);
            art::word_label(ctx.font(), self.letter.word, c, screen_height() * 0.05, b.w, art::INK, Some(WHITE));
        }

        // Pieces waiting in the tray; the one being dragged is drawn last, on top.
        let homes = self.tray_homes();
        let mut order: Vec<usize> =
            (0..self.pieces()).filter(|&i| !self.placed[i] && Some(i) != self.drag.held()).collect();
        order.extend(self.drag.held());
        for i in order {
            let mut r = self.drag.rect(i, homes[i]);
            r.x += art::shake_x(self.wiggle[i], ctx.time);
            self.draw_piece(tex, i, r);
        }

        // How-to-play: a hand carries a piece-sized card from the tray to the board.
        let visible: Vec<Vec2> = homes.iter().filter(|r| r.x > -1000.0).map(|r| r.center()).collect();
        if !visible.is_empty() {
            let from = visible.iter().copied().sum::<Vec2>() / visible.len() as f32;
            self.demo.draw_with_card(from, b.center(), self.cell(0).w.min(self.cell(0).h));
        }
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        let (c, r) = GRIDS[(level - 1) as usize];
        format!("{} pieces", c * r)
    }
}

/// "a fish", "an apple", "an igloo"
fn with_article(word: &str) -> String {
    let vowel = word.starts_with(['a', 'e', 'i', 'o', 'u']);
    format!("{} {word}", if vowel { "an" } else { "a" })
}
