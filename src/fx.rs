//! Confetti bursts for celebrating right answers.

use crate::art::Paint;
use macroquad::prelude::*;
use std::f32::consts::TAU;

struct Bit {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    rot: f32,
    spin: f32,
    life: f32,
    size: f32,
}

#[derive(Default)]
pub struct Confetti {
    bits: Vec<Bit>,
}

impl Confetti {
    pub fn burst(&mut self, at: Vec2, count: usize) {
        let s = screen_height();
        for _ in 0..count {
            let angle = rand::gen_range(0.0, TAU);
            let speed = rand::gen_range(0.3, 1.0) * s * 0.9;
            let paint = Paint::ALL[rand::gen_range(0, Paint::ALL.len())];
            self.bits.push(Bit {
                pos: at,
                vel: vec2(angle.cos() * speed, angle.sin() * speed - s * 0.6),
                color: paint.color(),
                rot: rand::gen_range(0.0, TAU),
                spin: rand::gen_range(-10.0, 10.0),
                life: rand::gen_range(1.5, 2.5),
                size: rand::gen_range(0.012, 0.024) * s,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        let gravity = screen_height() * 1.6;
        for b in &mut self.bits {
            b.vel.y += gravity * dt;
            b.vel *= 1.0 - 0.8 * dt; // air resistance
            b.pos += b.vel * dt;
            b.rot += b.spin * dt;
            b.life -= dt;
        }
        self.bits.retain(|b| b.life > 0.0);
    }

    pub fn draw(&self) {
        for b in &self.bits {
            let mut color = b.color;
            color.a = b.life.min(1.0);
            draw_rectangle_ex(
                b.pos.x,
                b.pos.y,
                b.size,
                b.size * 0.6,
                DrawRectangleParams {
                    offset: vec2(0.5, 0.5),
                    rotation: b.rot,
                    color,
                },
            );
        }
    }
}
