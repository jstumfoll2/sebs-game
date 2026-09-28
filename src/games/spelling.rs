//! Spelling: a picture and its word appear ("Let's spell cat! C, A, T."), and the player
//! types it on the big keyboard. Each letter is said as it's typed. A wrong spelling is read
//! back as typed ("That spells C-T-A"), then it's time to try again.

use super::{celebration_over, MiniGame, Progress};
use crate::alphabet::{self, capitalize};
use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::keyboard::{Key, Keyboard};
use crate::pictures::{self, Picture};
use macroquad::prelude::*;

/// For each level: word lengths to use, and whether the letters are shown faintly as a guide.
const LEVELS: [(usize, usize, bool); 6] = [
    (3, 3, true),
    (3, 3, false),
    (4, 4, true),
    (4, 4, false),
    (5, 5, true),
    (6, 9, true),
];

pub struct Spelling {
    progress: Progress,
    word: &'static str,
    picture: Picture,
    typed: String,
    keyboard: Keyboard,
    first_try: bool,
    misses: u32,
    /// Seconds left of celebrating a right answer.
    done: Option<f32>,
    /// The boxes wiggle after a wrong try.
    shake: f32,
    /// The word is complete: check it once the last letter has been said.
    checking: bool,
}

/// All the words we have pictures for (from the alphabet), e.g. ("cat", Cat).
fn words() -> Vec<(&'static str, Picture)> {
    alphabet::LETTERS
        .iter()
        .filter(|l| l.word.chars().all(|c| c.is_ascii_lowercase()))
        .map(|l| (l.word, l.picture))
        .collect()
}

/// "C, A, T" (in the voice's markup, so each letter name is said right).
fn spelled_out(word: &str) -> String {
    word.chars()
        .filter_map(alphabet::get)
        .map(|l| l.name)
        .collect::<Vec<_>>()
        .join(", ")
}

impl Spelling {
    pub fn new() -> Self {
        let mut game = Spelling {
            progress: Progress::new(LEVELS.len() as u32),
            word: "cat",
            picture: Picture::Cat,
            typed: String::new(),
            keyboard: Keyboard::new(0.55, "Say it"),
            first_try: true,
            misses: 0,
            done: None,
            shake: 0.0,
            checking: false,
        };
        game.new_round();
        game
    }

    fn guide(&self) -> bool {
        LEVELS[(self.progress.level - 1) as usize].2 || self.misses > 0
    }

    fn new_round(&mut self) {
        let (lo, hi, _) = LEVELS[(self.progress.level - 1) as usize];
        let choices: Vec<_> = words()
            .into_iter()
            .filter(|(w, _)| (lo..=hi).contains(&w.len()) && *w != self.word)
            .collect();
        if !choices.is_empty() {
            let (w, p) = choices[rand::gen_range(0, choices.len())];
            self.word = w;
            self.picture = p;
        }
        self.typed.clear();
        self.first_try = true;
        self.misses = 0;
        self.done = None;
        self.checking = false;
    }

    /// The letter boxes under the picture.
    fn boxes(&self) -> Vec<Rect> {
        let (w, h) = (screen_width(), screen_height());
        let n = self.word.len();
        let size = (w * 0.7 / n as f32).min(h * 0.13);
        let gap = size * 0.12;
        let x0 = (w - n as f32 * size) / 2.0;
        (0..n).map(|i| Rect::new(x0 + i as f32 * size + gap / 2.0, h * 0.37, size - gap, size - gap)).collect()
    }

    fn picture_rect() -> Rect {
        let (w, h) = (screen_width(), screen_height());
        let size = h * 0.2;
        Rect::new(w / 2.0 - size / 2.0, h * 0.1, size, size)
    }

    fn check(&mut self, ctx: &mut Ctx) {
        if self.typed == self.word {
            let leveled = self.progress.record(self.first_try);
            let words = format!("{}! You spelled {}!", capitalize(self.word), self.word);
            ctx.correct(Self::picture_rect().center(), &words, leveled);
            self.done = Some(1.2);
        } else {
            // Read back what they typed, then show them the right way.
            let typed_letters = spelled_out(&self.typed);
            let words = format!(
                "That spells {typed_letters}. Let's try again! {} is spelled {}.",
                capitalize(self.word),
                spelled_out(self.word)
            );
            ctx.wrong(&words);
            self.first_try = false;
            self.misses += 1;
            self.shake = 1.0;
            self.typed.clear();
        }
    }
}

impl MiniGame for Spelling {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        ctx.voice.then(&self.prompt());
    }

    fn prompt(&self) -> String {
        if self.guide() {
            format!("Let's spell {}! {}. {}!", self.word, spelled_out(self.word), capitalize(self.word))
        } else {
            format!("Can you spell {}?", self.word)
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        self.shake = (self.shake - ctx.dt * 2.5).max(0.0);
        if let Some(t) = self.done {
            let t = t - ctx.dt;
            if celebration_over(t, ctx) {
                self.new_round();
                ctx.voice.then(&self.prompt());
            } else {
                self.done = Some(t);
            }
            return;
        }

        if self.checking {
            if !ctx.voice.busy() {
                self.checking = false;
                self.check(ctx);
            }
            return;
        }

        // Tapping the picture says the word.
        if ctx.input.tapped(Self::picture_rect()) {
            ctx.sfx.pop();
            ctx.voice.say(&format!("{}!", capitalize(self.word)));
            return;
        }

        match self.keyboard.update(ctx) {
            Some(Key::Letter(c)) if self.typed.len() < self.word.len() => {
                self.typed.push(c.to_ascii_lowercase());
                ctx.sfx.pop();
                if let Some(l) = alphabet::get(c) {
                    ctx.voice.say(l.name);
                }
                if self.typed.len() == self.word.len() {
                    // Let the last letter be heard, then check.
                    self.checking = true;
                }
            }
            Some(Key::Back) => {
                self.typed.pop();
                ctx.sfx.pop();
            }
            Some(Key::Done) => {
                // "Say it": read out whatever has been typed so far.
                if self.typed.is_empty() {
                    ctx.voice.say(&self.prompt());
                } else {
                    ctx.voice.say(&format!("{}. {}", spelled_out(&self.typed), self.typed));
                }
            }
            _ => {}
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let pic = Self::picture_rect();
        let bob = (ctx.time * 2.0).sin() * pic.h * 0.02;
        let pic = Rect::new(pic.x, pic.y + bob, pic.w, pic.h);
        art::card(pic, WHITE);
        pictures::draw(self.picture, pic.center(), pic.h * 0.36);

        let celebrating = self.done.is_some();
        let dx = art::shake_x(self.shake, ctx.time);
        for (i, r) in self.boxes().iter().enumerate() {
            let mut r = *r;
            r.x += dx;
            if celebrating {
                r.y -= ((ctx.time * 6.0 - i as f32 * 0.6).sin().max(0.0)) * r.h * 0.12;
            }
            art::card(r, WHITE);
            let color = art::readable(Paint::ALL[i % Paint::ALL.len()].color());
            let want = self.word[i..i + 1].to_uppercase();
            match self.typed.get(i..i + 1) {
                Some(c) => art::text_center(font, &c.to_uppercase(), r.center(), r.h * 0.7, color),
                None if self.guide() => art::text_center(font, &want, r.center(), r.h * 0.7, art::SHADOW),
                None => {}
            }
            // A little line under the next box to fill.
            if i == self.typed.len() && !celebrating {
                draw_rectangle(r.x + r.w * 0.2, r.y + r.h * 0.88, r.w * 0.6, r.h * 0.05, art::INK);
            }
        }

        self.keyboard.draw(ctx, !self.typed.is_empty());
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        let (lo, hi, guide) = LEVELS[(level - 1) as usize];
        let len = if lo == hi { format!("{lo} letters") } else { format!("{lo}+ letters") };
        if guide { len } else { format!("{len}, no help") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_has_words() {
        for (lo, hi, _) in LEVELS {
            let n = words().iter().filter(|(w, _)| (lo..=hi).contains(&w.len())).count();
            assert!(n >= 2, "level with {lo}-{hi} letters needs at least two words, has {n}");
        }
    }
}
