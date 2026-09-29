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
    /// When a voice was tapped, while we wait for it to be ready to talk.
    loading_since: Option<f32>,
}

/// Don't flash the loading bar for clips that are ready almost at once.
const SHOW_LOADING_AFTER: f32 = 0.3;

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

    /// A voice was just picked: show the loading bar until it starts talking.
    pub fn chose(&mut self, time: f32) {
        self.loading_since = Some(time);
    }

    /// Seconds we've been waiting for the voice, once that's long enough to show.
    fn loading(&self, ctx: &Ctx) -> Option<f32> {
        let waited = ctx.time - self.loading_since?;
        (ctx.voice.waiting() && waited >= SHOW_LOADING_AFTER).then_some(waited)
    }

    pub fn update(&mut self, ctx: &mut Ctx) -> VoicePick {
        if self.loading_since.is_some() && !ctx.voice.waiting() {
            self.loading_since = None; // it's talking
        }
        if ctx.input.tapped(hud::home_rect()) {
            ctx.sfx.pop();
            return VoicePick::Back;
        }
        match cards(self.voices.len()).iter().position(|r| ctx.input.tapped(*r)) {
            Some(i) => {
                ctx.sfx.pop();
                self.chose(ctx.time);
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
        let loading = self.loading(ctx);
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
            match loading.filter(|_| chosen) {
                Some(_) => spinner(c, r.w * 0.15, ctx.time),
                None => art::speaker_icon(c, r.w * 0.22, WHITE),
            }
            art::text_center(font, &voice::display_name(model), vec2(r.center().x, r.y + r.h * 0.8), r.h * 0.15, art::INK);
            if chosen {
                let b = vec2(r.x + r.w * 0.88, r.y + r.w * 0.12);
                draw_circle(b.x, b.y, r.w * 0.12, Paint::Green.color());
                let t = r.w * 0.025;
                draw_line(b.x - r.w * 0.05, b.y, b.x - r.w * 0.01, b.y + r.w * 0.045, t, WHITE);
                draw_line(b.x - r.w * 0.01, b.y + r.w * 0.045, b.x + r.w * 0.06, b.y - r.w * 0.04, t, WHITE);
            }
        }
        // The loading bar under the voice that's getting ready.
        if let (Some(waited), Some(i)) = (loading, self.voices.iter().position(|m| current == Some(m.as_str()))) {
            let card = cards(self.voices.len())[i];
            let bar = Rect::new(w * 0.3, card.y + card.h * 1.25, w * 0.4, h * 0.035);
            // We can't know how long Piper will take, so the bar fills quickly at first and then
            // slows down (it never quite reaches the end, and vanishes when the voice talks).
            let fill = 0.95 * (1.0 - (-waited / 3.0).exp());
            art::rounded_rect(bar, bar.h / 2.0, art::SHADOW);
            let done = Rect::new(bar.x, bar.y, (bar.w * fill).max(bar.h), bar.h);
            art::rounded_rect(done, bar.h / 2.0, Paint::Green.color());
            // A shine sliding along the filled part, so it always looks busy.
            let shine = done.x + ((ctx.time * 0.8) % 1.0) * done.w;
            draw_circle(shine, done.center().y, bar.h * 0.3, Color::new(1.0, 1.0, 1.0, 0.6));
            let name = voice::display_name(&self.voices[i]);
            let words = format!("Getting {name} ready...");
            art::text_center(font, &words, vec2(w / 2.0, bar.y + bar.h * 2.6), h * 0.045, art::INK);
        }
        hud::draw_home_button(ctx);
    }
}

/// Eight dots going round, getting bigger toward the "front" one.
fn spinner(c: Vec2, radius: f32, time: f32) {
    for k in 0..8 {
        let angle = k as f32 / 8.0 * std::f32::consts::TAU;
        let behind = ((time * 1.5 - k as f32 / 8.0).rem_euclid(1.0) * 8.0).floor() / 8.0;
        let p = c + vec2(angle.sin(), -angle.cos()) * radius;
        draw_circle(p.x, p.y, radius * (0.12 + 0.14 * (1.0 - behind)), Color::new(1.0, 1.0, 1.0, 1.0 - behind * 0.7));
    }
}
