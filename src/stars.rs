//! The star panel: tap the star counter to see every star earned, and count them together.
//! Small piles are counted by ones (or sometimes twos); bigger ones are counted in groups of
//! 2, 3, 5 or 10 ("five, ten, fifteen..."), picked at random each time and finishing with
//! ones, which is a nice taste of skip counting.

use crate::alphabet::capitalize;
use crate::art;
use crate::ctx::Ctx;
use crate::games::counting::number_word;
use crate::hud;
use macroquad::prelude::*;

/// Seconds (at least) between numbers while counting out loud.
const STEP: f32 = 0.35;
/// Most stars we'll draw and count (after that the number says it all).
const MAX_SHOWN: usize = 100;

#[derive(Default)]
pub struct StarPanel {
    is_open: bool,
    /// How many stars are showing so far (they pop in as we count).
    shown: usize,
    /// How many stars appeared in the last step (1, or a whole group).
    last_step: usize,
    /// Counting by this many at a time (1 = by ones). Chosen when the panel opens.
    group: usize,
    counting: bool,
    step_time: f32,
    /// Makes the newest stars pop.
    pop: f32,
}

impl StarPanel {
    /// Returns true if the panel is open (so the game underneath should ignore input).
    pub fn update(&mut self, ctx: &mut Ctx) -> bool {
        if !self.is_open {
            if ctx.input.tapped(hud::star_rect()) {
                self.open(ctx);
                return true;
            }
            return false;
        }

        let total = (ctx.stars as usize).min(MAX_SHOWN);
        self.pop = (self.pop - ctx.dt * 3.0).max(0.0);
        if self.counting {
            self.step_time += ctx.dt;
            if self.step_time >= STEP && !ctx.voice.busy() {
                self.step_time = 0.0;
                if self.shown < total {
                    // Jump by a group while a whole group fits, then go by ones.
                    let g = self.group;
                    self.last_step = if g > 1 && self.shown.is_multiple_of(g) && self.shown + g <= total { g } else { 1 };
                    self.shown += self.last_step;
                    self.pop = 1.0;
                    ctx.sfx.pop();
                    ctx.voice.say(&number_word(self.shown));
                } else {
                    self.counting = false;
                    ctx.voice.say(&format!("{}! Great job!", capitalize(&stars_phrase(ctx.stars as usize))));
                }
            }
        }

        // Any tap closes the panel.
        if ctx.input.pressed {
            self.is_open = false;
            ctx.sfx.pop();
            ctx.voice.say("");
        }
        true
    }

    /// Open the panel and start counting.
    pub fn open(&mut self, ctx: &mut Ctx) {
        let total = ctx.stars as usize;
        ctx.sfx.pop();
        self.is_open = true;
        self.step_time = 0.0;
        self.last_step = 0;
        if total == 0 {
            self.shown = 0;
            self.counting = false;
            ctx.voice.say("No stars yet! Play a game to earn stars.");
        } else if total <= MAX_SHOWN {
            self.shown = 0;
            self.counting = true;
            self.group = choose_group(total, self.group);
            let how = match self.group {
                2 => " Let's count by twos!",
                3 => " Let's count by threes!",
                5 => " Let's count by fives!",
                10 => " Let's count by tens!",
                _ => "",
            };
            ctx.voice.say(&format!("Let's count your stars!{how}"));
        } else {
            // So many! Just show them and say the number.
            self.shown = MAX_SHOWN;
            self.counting = false;
            ctx.voice.say(&format!("You have {total} stars! Wow!"));
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        if !self.is_open {
            return;
        }
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.1, 0.1, 0.2, 0.45));

        let panel = Rect::new(w * 0.08, h * 0.1, w * 0.84, h * 0.8);
        art::card(panel, WHITE);

        // Title: the number, big.
        let n = if self.counting { self.shown } else { ctx.stars as usize };
        let title = if n == 1 { "1 star".to_string() } else { format!("{n} stars") };
        art::text_center(font, &title, vec2(panel.center().x, panel.y + panel.h * 0.13), panel.h * 0.13, art::INK);

        // The stars, in rows of ten with a gap after every five. The layout is based on
        // the final total, so stars don't jump around while they're being counted.
        let total = (ctx.stars as usize).min(MAX_SHOWN);
        let rows = total.div_ceil(10).max(1);
        let area = Rect::new(panel.x + panel.w * 0.05, panel.y + panel.h * 0.26, panel.w * 0.9, panel.h * 0.68);
        let cell = (area.w / 10.5).min(area.h / rows as f32);
        let top = area.y + (area.h - cell * rows as f32) / 2.0;
        for i in 0..self.shown.min(total) {
            let (row, col) = (i / 10, i % 10);
            let extra_gap = if col >= 5 { cell * 0.5 } else { 0.0 };
            let x = area.x + (area.w - cell * 10.5) / 2.0 + cell * (col as f32 + 0.5) + extra_gap;
            let y = top + cell * (row as f32 + 0.5);
            // The stars that just appeared (one, or a group of five) pop together.
            let newest = self.counting && i + self.last_step >= self.shown;
            let r = cell * 0.42 * if newest { 1.0 + self.pop * 0.5 } else { 1.0 };
            art::star(vec2(x, y), r * 1.1, art::darken(art::GOLD, 0.25));
            art::star(vec2(x, y), r, art::GOLD);
        }
    }
}

/// The group sizes that suit a pile of `total` stars: at least two whole groups, and bigger
/// piles get bigger groups (by ones only for the smallest piles).
fn group_options(total: usize) -> Vec<usize> {
    match total {
        0..=3 => vec![1],
        4..=10 => vec![1, 1, 2],
        11..=19 => vec![2, 5, 2, 5],
        20..=29 => vec![2, 5, 10, 5],
        30..=40 => vec![2, 3, 5, 10, 5, 10],
        _ => vec![5, 10, 5, 10, 2],
    }
}

/// Pick how to count, different from `last` time when there's a choice.
fn choose_group(total: usize, last: usize) -> usize {
    let mut options = group_options(total);
    let fresh: Vec<usize> = options.iter().copied().filter(|g| *g != last).collect();
    if !fresh.is_empty() {
        options = fresh;
    }
    options[macroquad::rand::gen_range(0, options.len())]
}

/// "one star", "twenty-three stars"
fn stars_phrase(n: usize) -> String {
    if n == 1 {
        "one star".to_string()
    } else {
        format!("{} stars", number_word(n))
    }
}

#[cfg(test)]
mod tests {
    use crate::games::counting::number_word;

    #[test]
    fn groups_fit_the_pile() {
        for total in 1..=100 {
            let options = super::group_options(total);
            assert!(!options.is_empty());
            // Every choice makes at least two whole groups (or is by ones).
            assert!(options.iter().all(|&g| g == 1 || g * 2 <= total), "{total}: {options:?}");
            // Never the same twice in a row when there's a choice.
            for last in 1..=10 {
                let g = super::choose_group(total, last);
                assert!(options.contains(&g) && (g != last || options.iter().all(|&o| o == last)));
            }
        }
    }

    #[test]
    fn number_words() {
        assert_eq!(number_word(7), "seven");
        assert_eq!(number_word(20), "twenty");
        assert_eq!(number_word(45), "forty-five");
        assert_eq!(number_word(90), "ninety");
    }
}
