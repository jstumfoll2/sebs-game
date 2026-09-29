//! Celebrations: confetti bursts for right answers, and fireworks for big moments.

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

// ---------- fireworks ----------

struct Rocket {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    /// Seconds until it bursts.
    fuse: f32,
}

struct Spark {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    life: f32,
}

/// Rockets that shoot up from the bottom of the screen and burst into colorful sparks.
#[derive(Default)]
pub struct Fireworks {
    rockets: Vec<Rocket>,
    sparks: Vec<Spark>,
    /// Keep launching rockets for this many more seconds.
    show_left: f32,
    next_launch: f32,
}

impl Fireworks {
    /// Launch rockets for `seconds`.
    pub fn show(&mut self, seconds: f32) {
        self.show_left = self.show_left.max(seconds);
        self.next_launch = 0.0;
    }

    pub fn stop(&mut self) {
        self.show_left = 0.0;
    }

    /// How many sparks are flying (for the speed log).
    pub fn sparks(&self) -> usize {
        self.sparks.len()
    }

    fn launch(&mut self) {
        let (w, h) = (screen_width(), screen_height());
        let paint = Paint::ALL[rand::gen_range(0, Paint::ALL.len())];
        self.rockets.push(Rocket {
            pos: vec2(rand::gen_range(w * 0.1, w * 0.9), h),
            vel: vec2(rand::gen_range(-w * 0.05, w * 0.05), -h * rand::gen_range(0.75, 1.0)),
            color: paint.color(),
            fuse: rand::gen_range(0.7, 1.0),
        });
    }

    /// A burst of sparks right here.
    pub fn burst(&mut self, at: Vec2, color: Color) {
        let s = screen_height();
        for _ in 0..70 {
            let angle = rand::gen_range(0.0, TAU);
            let speed = rand::gen_range(0.15, 0.4) * s;
            // Mostly the rocket's color, with a few white twinkles.
            let c = if rand::gen_range(0, 6) == 0 { WHITE } else { color };
            self.sparks.push(Spark {
                pos: at,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                color: c,
                life: rand::gen_range(0.9, 1.6),
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        let h = screen_height();
        if self.show_left > 0.0 {
            self.show_left -= dt;
            self.next_launch -= dt;
            if self.next_launch <= 0.0 {
                self.launch();
                self.next_launch = rand::gen_range(0.2, 0.45);
            }
        }
        let mut bursts = Vec::new();
        for r in &mut self.rockets {
            r.vel.y += h * 0.5 * dt;
            r.pos += r.vel * dt;
            r.fuse -= dt;
            if r.fuse <= 0.0 {
                bursts.push((r.pos, r.color));
            }
        }
        self.rockets.retain(|r| r.fuse > 0.0);
        for (at, color) in bursts {
            self.burst(at, color);
        }
        for s in &mut self.sparks {
            s.vel.y += h * 0.35 * dt;
            s.vel *= 1.0 - 1.5 * dt;
            s.pos += s.vel * dt;
            s.life -= dt;
        }
        self.sparks.retain(|s| s.life > 0.0);
    }

    pub fn draw(&self) {
        let h = screen_height();
        for r in &self.rockets {
            // A little glowing trail behind each rocket.
            for k in 0..5 {
                let p = r.pos - r.vel * (k as f32 * 0.012);
                let mut c = r.color;
                c.a = 1.0 - k as f32 * 0.18;
                draw_circle(p.x, p.y, h * 0.006 * (1.0 - k as f32 * 0.12), c);
            }
        }
        for s in &self.sparks {
            let fade = s.life.min(1.0);
            let mut glow = s.color;
            glow.a = 0.25 * fade;
            draw_circle(s.pos.x, s.pos.y, h * 0.012, glow);
            let mut c = s.color;
            c.a = fade;
            draw_circle(s.pos.x, s.pos.y, h * 0.005, c);
        }
    }
}
