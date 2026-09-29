//! The voice menu: every installed voice as a card. Tap one to switch to it and hear it.

use crate::art::{self, Paint};
use crate::ctx::Ctx;
use crate::hud;
use crate::voice;
use macroquad::prelude::*;

pub enum VoicePick {
    Nothing,
    Back,
    /// Switch to this voice (e.g. "en_US-amy-medium").
    Choose(String),
}

#[derive(Default)]
pub struct VoicePicker {
    voices: Vec<String>,
}

fn cards(n: usize) -> Vec<Rect> {
    let (w, h) = (screen_width(), screen_height());
    let cols = n.clamp(1, 5);
    let rows = n.div_ceil(cols).max(1);
    let cell = (w * 0.86 / cols as f32).min(h * 0.55 / rows as f32).min(h * 0.32);
    let size = cell * 0.86;
    let x0 = (w - cell * cols as f32) / 2.0;
    (0..n)
        .map(|i| {
            let (col, row) = ((i % cols) as f32, (i / cols) as f32);
            let in_row = if i / cols == rows - 1 { n - (rows - 1) * cols } else { cols };
            let shift = (cols - in_row) as f32 * cell / 2.0;
            Rect::new(x0 + shift + col * cell + (cell - size) / 2.0, h * 0.3 + row * cell, size, size)
        })
        .collect()
}

impl VoicePicker {
    pub fn start(&mut self, ctx: &mut Ctx) {
        self.voices = voice::installed();
        if self.voices.is_empty() {
            ctx.voice.say("No voices are installed yet. Ask a grown-up to run the voice setup.");
        } else {
            ctx.voice.say("Pick a voice! Tap one to hear it.");
        }
    }

    pub fn update(&mut self, ctx: &mut Ctx) -> VoicePick {
        if ctx.input.tapped(hud::home_rect()) {
            ctx.sfx.pop();
            return VoicePick::Back;
        }
        match cards(self.voices.len()).iter().position(|r| ctx.input.tapped(*r)) {
            Some(i) => {
                ctx.sfx.pop();
                VoicePick::Choose(self.voices[i].clone())
            }
            None => VoicePick::Nothing,
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = ctx.font();
        let (w, h) = (screen_width(), screen_height());
        art::text_center(font, "Pick a voice", vec2(w / 2.0, h * 0.14), h * 0.09, art::INK);
        art::text_center(font, "Tap one to hear it", vec2(w / 2.0, h * 0.23), h * 0.045, art::INK);

        let current = ctx.voice.model();
        for (i, (r, model)) in cards(self.voices.len()).iter().zip(&self.voices).enumerate() {
            let chosen = current == Some(model.as_str());
            let mut r = *r;
            if chosen {
                r = art::scale_rect(r, 1.06);
                art::glow(r, ctx.time);
            }
            art::card(r, WHITE);
            let color = Paint::ALL[(i + 4) % Paint::ALL.len()].color();
            let c = vec2(r.center().x, r.y + r.h * 0.38);
            draw_circle(c.x, c.y, r.w * 0.26, color);
            art::speaker_icon(c, r.w * 0.22, WHITE);
            art::text_center(font, &voice::display_name(model), vec2(r.center().x, r.y + r.h * 0.8), r.h * 0.15, art::INK);
            if chosen {
                let b = vec2(r.x + r.w * 0.88, r.y + r.w * 0.12);
                draw_circle(b.x, b.y, r.w * 0.12, Paint::Green.color());
                let t = r.w * 0.025;
                draw_line(b.x - r.w * 0.05, b.y, b.x - r.w * 0.01, b.y + r.w * 0.045, t, WHITE);
                draw_line(b.x - r.w * 0.01, b.y + r.w * 0.045, b.x + r.w * 0.06, b.y - r.w * 0.04, t, WHITE);
            }
        }
        hud::draw_home_button(ctx);
    }
}
