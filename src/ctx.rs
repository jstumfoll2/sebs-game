//! `Ctx` ("context") bundles the things every game needs: input, font, sound, voice, confetti.
//! It gets passed into each game's `update` and `draw`.

use crate::{
    fx::{Confetti, Fireworks},
    input::Input,
    sfx::Sfx,
    voice::Voice,
};
use macroquad::prelude::*;

/// Things to say for a right answer. "NAME" is swapped for the player's name.
pub const PRAISE: [&str; 8] = [
    "Great job!",
    "You did it!",
    "Awesome!",
    "Yay!",
    "Super!",
    "Way to go, NAME!",
    "High five!",
    "Wow!",
];

pub struct Ctx {
    pub input: Input,
    pub font: Option<Font>,
    pub sfx: Sfx,
    pub voice: Voice,
    pub confetti: Confetti,
    pub fireworks: Fireworks,
    /// Who's playing (used in the title, greetings and praise).
    pub name: String,
    /// Stars earned (one per right answer), saved with the player.
    pub stars: u32,
    /// Makes the star counter "pop" when it goes up. Fades from 1 to 0.
    pub star_pop: f32,
    pub time: f32,
    pub dt: f32,
    /// Sound turned off with the speaker button.
    pub muted: bool,
    /// Makes the top buttons bounce when tapped: [home, say again, speaker]. Fades 1 -> 0.
    pub button_pop: [f32; 3],
    pub last_mute_toggle: f32,
}

impl Ctx {
    /// The speaker button: turn all sound off or back on.
    pub fn toggle_mute(&mut self) {
        // Ignore a second toggle right away, so one tap can never flip it twice.
        if self.time - self.last_mute_toggle < 0.5 {
            return;
        }
        self.last_mute_toggle = self.time;
        self.muted = !self.muted;
        self.sfx.muted = self.muted;
        self.voice.set_muted(self.muted);
        if !self.muted {
            self.sfx.pop();
            self.voice.say("Sound on!");
        }
    }

    pub fn font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    /// Celebrate a right answer: ding, confetti, a star, and some praise.
    pub fn correct(&mut self, at: Vec2, words: &str, leveled_up: bool) {
        self.sfx.ding();
        self.confetti.burst(at, if leveled_up { 140 } else { 60 });
        self.stars += 1;
        self.star_pop = 1.0;
        let praise = PRAISE[rand::gen_range(0, PRAISE.len())].replace("NAME", &self.name);
        self.voice.say(&format!("{praise} {words}"));
        if leveled_up {
            self.sfx.tada(); // the "Level 3 done!" banner says the rest
        }
    }

    /// A gentle "uh-oh" for a wrong answer. Never scary, never a penalty.
    pub fn wrong(&mut self, words: &str) {
        self.sfx.oops();
        self.voice.say(words);
    }
}
