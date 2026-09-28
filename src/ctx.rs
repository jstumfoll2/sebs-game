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

/// A two-player match: first to `goal` stars wins. Turns switch after each star.
/// Scores here are just for this match (they don't change anyone's saved stars).
pub struct Match {
    pub names: [String; 2],
    /// Each player's color (their spot in the players list), for their avatar.
    pub colors: [usize; 2],
    pub scores: [u32; 2],
    /// Whose turn it is (0 or 1).
    pub turn: usize,
    pub goal: u32,
}

/// Somebody won! Shown on the winner screen.
#[derive(Clone, Debug)]
pub enum Win {
    /// One player reached a star milestone (50, 100, ...).
    Solo { name: String, stars: u32 },
    /// A two-player match: the winner and the other player, with their stars.
    Versus { winner: String, winner_stars: u32, other: String, other_stars: u32, colors: [usize; 2] },
}

/// Solo players get the winner screen every this many stars.
pub const SOLO_GOAL: u32 = 50;

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
    /// A two-player match, if one is going.
    pub versus: Option<Match>,
    /// Set when someone wins; the app then shows the winner screen.
    pub win: Option<Win>,
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
        self.star_pop = 1.0;
        let praise = PRAISE[rand::gen_range(0, PRAISE.len())].replace("NAME", &self.name);
        self.voice.say(&format!("{praise} {words}"));
        if leveled_up {
            self.sfx.tada(); // the "Level 3 done!" banner says the rest
        }

        if let Some(m) = &mut self.versus {
            // Two players: the star goes to whoever's turn it is, then it's the other's turn.
            let t = m.turn;
            m.scores[t] += 1;
            if m.scores[t] >= m.goal {
                self.win = Some(Win::Versus {
                    winner: m.names[t].clone(),
                    winner_stars: m.scores[t],
                    other: m.names[1 - t].clone(),
                    other_stars: m.scores[1 - t],
                    colors: [m.colors[t], m.colors[1 - t]],
                });
            } else {
                m.turn = 1 - t;
                self.name = m.names[m.turn].clone();
                self.voice.then(&format!("Now it's {}'s turn!", self.name));
            }
        } else {
            self.stars += 1;
            if self.stars % SOLO_GOAL == 0 {
                self.win = Some(Win::Solo { name: self.name.clone(), stars: self.stars });
            }
        }
    }

    /// A gentle "uh-oh" for a wrong answer. Never scary, never a penalty.
    pub fn wrong(&mut self, words: &str) {
        self.sfx.oops();
        self.voice.say(words);
    }
}
