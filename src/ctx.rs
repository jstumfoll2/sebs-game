//! `Ctx` ("context") bundles the things every game needs: input, font, sound, voice, confetti.
//! It gets passed into each game's `update` and `draw`.

use crate::{fx::Confetti, input::Input, sfx::Sfx, speech::Voice};
use macroquad::prelude::*;

const PRAISE: [&str; 8] = [
    "Great job!",
    "You did it!",
    "Awesome!",
    "Yay!",
    "Super!",
    "Way to go, Sebastian!",
    "High five!",
    "Wow!",
];

pub struct Ctx {
    pub input: Input,
    pub font: Option<Font>,
    pub sfx: Sfx,
    pub voice: Voice,
    pub confetti: Confetti,
    /// Stars earned this session (one per right answer).
    pub stars: u32,
    /// Makes the star counter "pop" when it goes up. Fades from 1 to 0.
    pub star_pop: f32,
    pub time: f32,
    pub dt: f32,
}

impl Ctx {
    pub fn font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    /// Celebrate a right answer: ding, confetti, a star, and some praise.
    pub fn correct(&mut self, at: Vec2, words: &str, leveled_up: bool) {
        self.sfx.ding();
        self.confetti.burst(at, if leveled_up { 140 } else { 60 });
        self.stars += 1;
        self.star_pop = 1.0;
        let praise = PRAISE[rand::gen_range(0, PRAISE.len())];
        self.voice.say(&format!("{praise} {words}"));
        if leveled_up {
            self.sfx.tada();
            self.voice.then("You're getting so good at this!");
        }
    }

    /// A gentle "uh-oh" for a wrong answer. Never scary, never a penalty.
    pub fn wrong(&mut self, words: &str) {
        self.sfx.oops();
        self.voice.say(words);
    }
}
