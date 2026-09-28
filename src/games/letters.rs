//! Find the letter: a picture clue appears ("ball"), the voice says the letter and its sound
//! ("Find the letter B! Buh, buh, ball!"), and Sebastian drags the matching letter onto
//! the picture. Tapping a letter just says its name and sound.
//! Starts with a handful of letters and slowly adds more.
//!
//! Hints: after one miss the clue's word appears (its first letter is the answer!);
//! after two misses the right card glows.

use super::{celebration_over, fade, shuffle, CardDrag, Demo, DragEvent, MiniGame, Phase, Progress};
use crate::alphabet::{self, capitalize, Letter};
use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::pictures;
use macroquad::prelude::*;

/// The order letters get introduced. The first few are the ones in play at level 1.
/// Tip: put the letters he already knows first!
pub const LETTER_ORDER: &str = "SABMOTEDPCNIRLHFGKUWJYVZXQ";
/// How many letters from LETTER_ORDER are in play at each level.
const POOL_SIZE: [usize; 5] = [5, 8, 12, 17, 26];
/// How many letters to choose from at each level.
const CHOICES: [usize; 5] = [3, 3, 4, 4, 5];

pub struct Letters {
    progress: Progress,
    choices: Vec<char>,
    colors: Vec<Paint>,
    target: char,
    first_try: bool,
    misses: u32,
    shake: Vec<f32>,
    phase: Phase,
    /// Letters are dragged onto the picture; tapping one just says it.
    drag: CardDrag,
    demo: Demo,
}

impl Letters {
    pub fn new() -> Self {
        let mut game = Letters {
            progress: Progress::new(POOL_SIZE.len() as u32),
            choices: Vec::new(),
            colors: Vec::new(),
            target: 'S',
            first_try: true,
            misses: 0,
            shake: Vec::new(),
            phase: Phase::Playing,
            drag: CardDrag::default(),
            demo: Demo::default(),
        };
        game.new_round();
        game
    }

    fn target(&self) -> &'static Letter {
        alphabet::get(self.target).expect("letters are A-Z")
    }

    fn new_round(&mut self) {
        let lvl = (self.progress.level - 1) as usize;
        let pool: Vec<char> = LETTER_ORDER.chars().take(POOL_SIZE[lvl]).collect();

        // A new target letter (not the same as last time).
        let mut others: Vec<char> = pool.iter().copied().filter(|c| *c != self.target).collect();
        shuffle(&mut others);
        self.target = others.pop().unwrap_or('S');

        // Fill in the other choices.
        others.truncate(CHOICES[lvl] - 1);
        self.choices = others;
        self.choices.push(self.target);
        shuffle(&mut self.choices);

        // Every letter gets its own color. (No yellow: hard to see on white.)
        let mut paints: Vec<Paint> = Paint::ALL.into_iter().filter(|p| *p != Paint::Yellow).collect();
        shuffle(&mut paints);
        self.colors = paints;

        self.first_try = true;
        self.misses = 0;
        self.shake = vec![0.0; self.choices.len()];
        self.drag.reset();
        self.phase = Phase::Playing;
    }

    /// Where the right letter sits once it's been dropped on the picture.
    fn placed_rect() -> Rect {
        let clue = Self::clue_rect();
        let size = clue.w * 0.38;
        Rect::new(clue.x + clue.w - size * 0.6, clue.y + clue.h - size * 0.7, size, size)
    }

    fn rects(&self) -> Vec<Rect> {
        super::row_of_cards(self.choices.len(), 0.9, 0.3, 0.7)
    }

    /// The picture clue card at the top.
    fn clue_rect() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        let size = h * 0.34;
        Rect::new(w / 2.0 - size / 2.0, h * 0.08, size, size)
    }
}

impl MiniGame for Letters {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        // Show how to play: a hand drags from the letters up onto the picture.
        self.demo.start(2.0);
        let intro = format!("{} Drag the letter onto the picture!", self.prompt());
        ctx.voice.then(&intro);
    }

    fn prompt(&self) -> String {
        let l = self.target();
        let s = l.sound();
        if l.letter == 'X' {
            format!("Find the letter X! {s}, like the end of box!")
        } else {
            format!("Find the letter {}! {s}, {s}, {}!", l.name, l.word)
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.shake, ctx.dt, 2.5);
        self.demo.update(ctx);
        match self.phase {
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
                // Tapping the picture says its letter and sound: "Bee says buh. Buh, buh, ball!"
                if self.drag.held().is_none() && ctx.input.tapped(Self::clue_rect()) {
                    ctx.sfx.pop();
                    ctx.voice.say(&self.target().teach(&ctx.name));
                    return;
                }
                let rects = self.rects();
                match self.drag.update(&ctx.input, &rects, ctx.dt) {
                    DragEvent::PickedUp(i) => {
                        // Touching a letter says its name and sound: "Bee! Buh."
                        let l = alphabet::get(self.choices[i]).expect("letters are A-Z");
                        ctx.sfx.pop();
                        ctx.voice.say(&format!("{}! {}.", capitalize(l.name), l.sound()));
                    }
                    DragEvent::Dropped(i, at) => {
                        // Only a letter dropped on the picture counts as an answer.
                        if art::scale_rect(Self::clue_rect(), 1.2).contains(at) {
                            let target = self.target();
                            if self.choices[i] == self.target {
                                let leveled = self.progress.record(self.first_try);
                                let words = format!("{}! {}", capitalize(target.name), target.teach(&ctx.name));
                                ctx.correct(Self::placed_rect().center(), &words, leveled);
                                self.phase = Phase::Celebrating(1.0);
                            } else {
                                let tapped = alphabet::get(self.choices[i]).expect("letters are A-Z");
                                self.first_try = false;
                                self.misses += 1;
                                self.shake[i] = 1.0;
                                ctx.wrong(&format!(
                                    "That's {}. {} says {}. Can you find {}?",
                                    tapped.name,
                                    capitalize(tapped.name),
                                    tapped.sound(),
                                    target.name
                                ));
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
        let celebrating = matches!(self.phase, Phase::Celebrating(_));
        let target = self.target();
        let target_color = self
            .choices
            .iter()
            .position(|c| *c == self.target)
            .map(|i| self.colors[i % self.colors.len()].color())
            .unwrap_or(art::INK);

        // The picture clue. Its word shows up as a hint after a miss, and when it's solved.
        let clue = Self::clue_rect();
        let bob = (ctx.time * 2.0).sin() * clue.h * 0.02;
        let clue = Rect::new(clue.x, clue.y + bob, clue.w, clue.h);
        art::card(clue, WHITE);
        let show_word = celebrating || self.misses >= 1;
        let pic_c = vec2(clue.center().x, clue.y + clue.h * if show_word { 0.42 } else { 0.5 });
        pictures::draw(target.picture, pic_c, clue.h * if show_word { 0.3 } else { 0.36 });
        if show_word {
            let c = vec2(clue.center().x, clue.y + clue.h * 0.86);
            art::word_label(font, target.word, c, clue.h * 0.12, clue.w * 0.9, target_color, None);
        }

        // The letter choices. The one being dragged is drawn last so it's on top.
        let homes = self.rects();
        let mut order: Vec<usize> = (0..self.choices.len()).filter(|&i| Some(i) != self.drag.held()).collect();
        order.extend(self.drag.held());
        for i in order {
            let letter = self.choices[i];
            let is_target = letter == self.target;
            let mut r = self.drag.rect(i, homes[i]);
            if celebrating && is_target {
                // The right letter sticks to the corner of the picture.
                r = art::scale_rect(Self::placed_rect(), 1.0 + 0.06 * (ctx.time * 8.0).sin());
            }
            r.x += art::shake_x(self.shake[i], ctx.time);
            if self.misses >= 2 && is_target && !celebrating {
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            let color = self.colors[i % self.colors.len()].color();
            art::text_center(font, &letter.to_string(), r.center(), r.h * 0.75, color);
        }

        // How-to-play: a hand drags from the middle of the letters up onto the picture.
        let from = homes.iter().map(|r| r.center()).sum::<Vec2>() / homes.len().max(1) as f32;
        self.demo.draw_with_card(from, Self::clue_rect().center(), homes[0].w * 0.8);
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        format!("{} letters", POOL_SIZE[(level - 1) as usize])
    }
}
