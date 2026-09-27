//! Find the letter: the voice asks for a letter, Sebastian taps it.
//! Starts with a handful of letters and slowly adds more.

use super::{celebration_over, fade, shuffle, MiniGame, Phase, Progress};
use crate::art::{self, Paint, Thing};
use crate::ctx::Ctx;
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
        };
        game.new_round();
        game
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
        self.phase = Phase::Playing;
    }

    fn rects(&self) -> Vec<Rect> {
        super::row_of_cards(self.choices.len(), 0.9, 0.36, 0.58)
    }
}

impl MiniGame for Letters {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        ctx.voice.then(&self.prompt());
    }

    fn prompt(&self) -> String {
        format!("Find the letter {}!", letter_name(self.target))
    }

    fn update(&mut self, ctx: &mut Ctx) {
        fade(&mut self.shake, ctx.dt, 2.5);
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
                let rects = self.rects();
                if let Some(i) = rects.iter().position(|r| ctx.input.tapped(*r)) {
                    let tapped = self.choices[i];
                    if tapped == self.target {
                        let leveled = self.progress.record(self.first_try);
                        let name = letter_name(tapped);
                        let (word, _) = word_for(tapped);
                        ctx.correct(rects[i].center(), &format!("{name}! {name} is for {word}!"), leveled);
                        self.phase = Phase::Celebrating(2.6);
                    } else {
                        self.first_try = false;
                        self.misses += 1;
                        self.shake[i] = 1.0;
                        ctx.wrong(&format!(
                            "That's {}. Can you find {}?",
                            letter_name(tapped),
                            letter_name(self.target)
                        ));
                    }
                }
            }
        }
    }

    fn draw(&self, ctx: &Ctx) {
        let celebrating = matches!(self.phase, Phase::Celebrating(_));
        for (i, (r, letter)) in self.rects().iter().zip(&self.choices).enumerate() {
            let is_target = *letter == self.target;
            let mut r = *r;
            r.x += art::shake_x(self.shake[i], ctx.time);
            if celebrating && is_target {
                r = art::scale_rect(r, 1.12 + 0.04 * (ctx.time * 8.0).sin());
            }
            if self.misses >= 2 && is_target && !celebrating {
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            let color = self.colors[i % self.colors.len()].color();
            art::text_center(ctx.font(), &letter.to_string(), r.center(), r.h * 0.75, color);

            // Show a picture for the letter's word, if we can draw one (A = apple, B = ball...).
            if celebrating && is_target {
                if let (_, Some(thing)) = word_for(*letter) {
                    let c = vec2(r.center().x, screen_height() * 0.2);
                    art::draw_thing(thing, c, screen_height() * 0.08, color);
                }
            }
        }
    }

    fn progress(&mut self) -> &mut Progress {
        &mut self.progress
    }
}

/// How to say a letter's name so the computer voice pronounces it right.
pub fn letter_name(c: char) -> &'static str {
    match c.to_ascii_uppercase() {
        'A' => "ay",
        'B' => "bee",
        'C' => "see",
        'D' => "dee",
        'E' => "ee",
        'F' => "eff",
        'G' => "gee",
        'H' => "aitch",
        'I' => "eye",
        'J' => "jay",
        'K' => "kay",
        'L' => "ell",
        'M' => "em",
        'N' => "en",
        'O' => "oh",
        'P' => "pee",
        'Q' => "cue",
        'R' => "ar",
        'S' => "ess",
        'T' => "tee",
        'U' => "you",
        'V' => "vee",
        'W' => "double you",
        'X' => "ex",
        'Y' => "why",
        'Z' => "zee",
        _ => "",
    }
}

/// A word that starts with the letter, plus a picture if we have one.
fn word_for(c: char) -> (&'static str, Option<Thing>) {
    match c {
        'A' => ("apple", Some(Thing::Apple)),
        'B' => ("ball", Some(Thing::Ball)),
        'C' => ("cat", None),
        'D' => ("dog", None),
        'E' => ("elephant", None),
        'F' => ("fish", Some(Thing::Fish)),
        'G' => ("goat", None),
        'H' => ("heart", Some(Thing::Heart)),
        'I' => ("igloo", None),
        'J' => ("jellyfish", None),
        'K' => ("kite", None),
        'L' => ("lion", None),
        'M' => ("moon", None),
        'N' => ("nest", None),
        'O' => ("octopus", None),
        'P' => ("pig", None),
        'Q' => ("queen", None),
        'R' => ("rainbow", None),
        'S' => ("Sebastian, and star", Some(Thing::Star)),
        'T' => ("tiger", None),
        'U' => ("umbrella", None),
        'V' => ("van", None),
        'W' => ("whale", None),
        'X' => ("xylophone", None),
        'Y' => ("yo-yo", None),
        'Z' => ("zebra", None),
        _ => ("", None),
    }
}
