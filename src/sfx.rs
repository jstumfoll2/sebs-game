//! Sound effects, synthesized in code (no sound files needed).
//! We build tiny WAV files in memory from a list of notes, then hand them to macroquad.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use std::f32::consts::TAU;

const RATE: u32 = 44_100;

/// One note: starts at `at` seconds, slides from `f0` to `f1` Hz over `len` seconds.
struct Note {
    at: f32,
    f0: f32,
    f1: f32,
    len: f32,
    vol: f32,
}

const fn note(at: f32, freq: f32, len: f32) -> Note {
    Note { at, f0: freq, f1: freq, len, vol: 0.5 }
}

pub struct Sfx {
    pop: Option<Sound>,
    ding: Option<Sound>,
    oops: Option<Sound>,
    tada: Option<Sound>,
}

impl Sfx {
    pub async fn load() -> Self {
        // Note frequencies: C5=523, E5=659, G5=784, C6=1047, E6=1319, G6=1568
        Sfx {
            pop: make(&[Note { at: 0.0, f0: 500.0, f1: 1100.0, len: 0.09, vol: 0.5 }]).await,
            ding: make(&[note(0.0, 1047.0, 0.3), note(0.08, 1319.0, 0.3), note(0.16, 1568.0, 0.45)]).await,
            oops: make(&[note(0.0, 392.0, 0.18), note(0.16, 330.0, 0.28)]).await,
            tada: make(&[
                note(0.0, 523.0, 0.2),
                note(0.12, 659.0, 0.2),
                note(0.24, 784.0, 0.2),
                note(0.36, 1047.0, 0.7),
            ])
            .await,
        }
    }

    pub fn pop(&self) {
        play(&self.pop, 0.5);
    }
    pub fn ding(&self) {
        play(&self.ding, 0.5);
    }
    pub fn oops(&self) {
        play(&self.oops, 0.4);
    }
    pub fn tada(&self) {
        play(&self.tada, 0.6);
    }
}

async fn make(notes: &[Note]) -> Option<Sound> {
    load_sound_from_bytes(&wav(notes)).await.ok()
}

fn play(sound: &Option<Sound>, volume: f32) {
    if let Some(s) = sound {
        play_sound(s, PlaySoundParams { looped: false, volume });
    }
}

/// Render notes into a 16-bit mono WAV file (as bytes).
fn wav(notes: &[Note]) -> Vec<u8> {
    let total = notes.iter().map(|n| n.at + n.len).fold(0.0, f32::max) + 0.05;
    let count = (total * RATE as f32) as usize;
    let mut buf = vec![0.0f32; count];

    for n in notes {
        let start = (n.at * RATE as f32) as usize;
        let len = (n.len * RATE as f32) as usize;
        let mut phase = 0.0f32;
        for i in 0..len {
            let t = i as f32 / len as f32;
            let freq = n.f0 + (n.f1 - n.f0) * t;
            phase += TAU * freq / RATE as f32;
            let attack = (i as f32 / (0.005 * RATE as f32)).min(1.0);
            let envelope = attack * (1.0 - t).powi(2);
            // A sine wave plus a little overtone sounds brighter, like a toy xylophone.
            let sample = phase.sin() * 0.8 + (phase * 2.0).sin() * 0.2;
            if let Some(b) = buf.get_mut(start + i) {
                *b += sample * envelope * n.vol;
            }
        }
    }

    let data_len = (count * 2) as u32;
    let mut out = Vec::with_capacity(44 + count * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // size of this chunk
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes()); // bytes per second
    out.extend_from_slice(&2u16.to_le_bytes()); // bytes per sample
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in buf {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}
