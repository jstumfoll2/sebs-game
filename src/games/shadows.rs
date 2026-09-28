//! Shadows: dark shadows along the top, colorful pictures along the bottom. Drag each
//! picture onto its shadow. Matching shapes like this is great pattern-matching practice.

use super::{celebration_over, fade, row_of_cards, shuffle, CardDrag, Demo, DragEvent, MiniGame, Progress};
use crate::alphabet::{self, Letter};
use crate::art;
use crate::ctx::Ctx;
use crate::render;
use macroquad::prelude::*;

/// How many pictures to match at each level.
const COUNTS: [usize; 4] = [2, 3, 4, 5];
/// The color of the shadows.
const SHADOW_INK: Color = Color::new(0.24, 0.24, 0.36, 1.0);

pub struct Shadows {
    progress: Progress,
    /// The pictures in this round, in the order their shadows appear on top.
    items: Vec<&'static Letter>,
    textures: Vec<RenderTarget>,
    material: Option<Material>,
    /// Which picture sits in each spot along the bottom row.
    bottom: Vec<usize>,
    matched: Vec<bool>,
    drag: CardDrag,
    demo: Demo,
    shake: Vec<f32>,
    pop: Vec<f32>,
    first_try: bool,
    done: Option<f32>,
}

impl Shadows {
    pub fn new() -> Self {
        let mut game = Shadows {
            progress: Progress::new(COUNTS.len() as u32),
            items: Vec::new(),
            textures: Vec::new(),
            material: None,
            bottom: Vec::new(),
            matched: Vec::new(),
            drag: CardDrag::default(),
            demo: Demo::default(),
            shake: Vec::new(),
            pop: Vec::new(),
            first_try: true,
            done: None,
        };
        game.new_round();
        game
    }

    fn new_round(&mut self) {
        let n = COUNTS[(self.progress.level - 1) as usize];
        let mut all: Vec<&'static Letter> = alphabet::LETTERS.iter().collect();
        shuffle(&mut all);
        self.items = all.into_iter().take(n).collect();
        self.textures.clear(); // drawn at the start of the next update
        self.bottom = (0..n).collect();
        // Make sure the pictures aren't lined up right under their own shadows.
        while n > 1 && self.bottom.iter().enumerate().any(|(i, &b)| i == b) {
            shuffle(&mut self.bottom);
        }
        self.matched = vec![false; n];
        self.drag.reset();
        self.shake = vec![0.0; n];
        self.pop = vec![0.0; n];
        self.first_try = true;
        self.done = None;
    }

    fn shadow_rects(&self) -> Vec<Rect> {
        row_of_cards(self.items.len(), 0.9, 0.3, 0.36)
    }

    /// Where the bottom-row pictures sit (matched ones are parked out of reach).
    fn picture_homes(&self) -> Vec<Rect> {
        let spots = row_of_cards(self.items.len(), 0.9, 0.3, 0.76);
        (0..self.items.len())
            .map(|item| {
                if self.matched[item] {
                    return Rect::new(-10_000.0, -10_000.0, spots[0].w, spots[0].h);
                }
                let spot = self.bottom.iter().position(|&b| b == item).unwrap_or(0);
                spots[spot]
            })
            .collect()
    }
}

impl MiniGame for Shadows {
    fn enter(&mut self, ctx: &mut Ctx) {
        self.new_round();
        self.demo.start(2.0);
        ctx.voice.then("Can you match the pictures to their shadows? Drag each picture onto its shadow!");
    }

    fn prompt(&self) -> String {
        "Drag each picture onto its shadow!".to_string()
    }

    fn update(&mut self, ctx: &mut Ctx) {
        if self.material.is_none() {
            self.material = render::shadow_material();
        }
        if self.textures.is_empty() {
            self.textures = self.items.iter().map(|l| render::picture_texture(l.picture, 384)).collect();
        }
        fade(&mut self.shake, ctx.dt, 2.5);
        fade(&mut self.pop, ctx.dt, 3.0);
        self.demo.update(ctx);

        if let Some(t) = self.done {
            let t = t - ctx.dt;
            if celebration_over(t, ctx) {
                self.new_round();
                ctx.voice.then("Here are some new shadows!");
            } else {
                self.done = Some(t);
            }
            return;
        }

        // Tapping a shadow asks the question.
        let shadows = self.shadow_rects();
        if self.drag.held().is_none() {
            if let Some(i) = shadows.iter().position(|r| ctx.input.tapped(*r)) {
                if !self.matched[i] {
                    ctx.sfx.pop();
                    ctx.voice.say("Find the picture that makes this shadow!");
                    return;
                }
            }
        }

        let homes = self.picture_homes();
        match self.drag.update(&ctx.input, &homes, ctx.dt) {
            DragEvent::PickedUp(i) => {
                // Touching a picture says what it is.
                ctx.sfx.pop();
                ctx.voice.say(&format!("{}!", alphabet::capitalize(self.items[i].word)));
            }
            DragEvent::Dropped(i, at) => {
                if let Some(target) = shadows.iter().position(|r| art::scale_rect(*r, 1.15).contains(at)) {
                    if target == i {
                        self.matched[i] = true;
                        self.pop[i] = 1.0;
                        ctx.sfx.ding();
                        if self.matched.iter().all(|m| *m) {
                            let leveled = self.progress.record(self.first_try);
                            let at = shadows[i].center();
                            ctx.correct(at, "You matched all the shadows!", leveled);
                            self.done = Some(1.5);
                        } else {
                            ctx.voice.say(&format!("Yes! That's the {}'s shadow!", self.items[i].word));
                        }
                    } else if !self.matched[target] {
                        self.first_try = false;
                        self.shake[i] = 1.0;
                        ctx.wrong(&format!("Hmm, that's not the {}'s shadow. Try another one!", self.items[i].word));
                    }
                }
            }
            DragEvent::Nothing => {}
        }
    }

    fn draw(&self, ctx: &Ctx) {
        if self.textures.len() != self.items.len() {
            return;
        }
        let whole = Rect::new(0.0, 0.0, 1.0, 1.0);

        // The shadows (or the picture, once it's been matched).
        for (i, r) in self.shadow_rects().iter().enumerate() {
            let r = art::scale_rect(*r, 1.0 + 0.1 * (self.pop[i] * std::f32::consts::PI).sin());
            art::card(r, WHITE);
            if self.matched[i] {
                render::draw_picture(&self.textures[i].texture, r, whole, WHITE);
            } else if let Some(m) = &self.material {
                render::draw_with_material(m, &self.textures[i].texture, r, SHADOW_INK);
            }
        }

        // The pictures to drag. The one being dragged is drawn last, on top.
        let homes = self.picture_homes();
        let mut order: Vec<usize> =
            (0..self.items.len()).filter(|&i| !self.matched[i] && Some(i) != self.drag.held()).collect();
        order.extend(self.drag.held());
        for i in order {
            let mut r = self.drag.rect(i, homes[i]);
            r.x += art::shake_x(self.shake[i], ctx.time);
            art::card(r, WHITE);
            render::draw_picture(&self.textures[i].texture, r, whole, WHITE);
        }

        // How-to-play: a hand carries a card from the pictures up to the shadows.
        let spots = row_of_cards(self.items.len(), 0.9, 0.3, 0.76);
        let from = vec2(screen_width() / 2.0, spots[0].center().y);
        let to = vec2(screen_width() / 2.0, self.shadow_rects()[0].center().y);
        self.demo.draw_with_card(from, to, spots[0].w);
    }

    fn progress(&self) -> &Progress {
        &self.progress
    }

    fn progress_mut(&mut self) -> &mut Progress {
        &mut self.progress
    }

    fn level_label(&self, level: u32) -> String {
        format!("{} shadows", COUNTS[(level - 1) as usize])
    }
}
