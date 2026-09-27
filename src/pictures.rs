//! Pictures for the alphabet: one friendly object per letter (A = apple, B = ball, ...).
//! Like the rest of the art, they're built from simple shapes. Each is drawn centered on
//! `c` and fits in roughly a circle of radius `s`. They're designed to sit on white cards.

use crate::art::{self, face, Paint, Thing};
use macroquad::prelude::*;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Picture {
    Apple,
    Ball,
    Cat,
    Dog,
    Egg,
    Fish,
    Grapes,
    Heart,
    Igloo,
    Jellyfish,
    Kite,
    Lollipop,
    Moon,
    Nest,
    Octopus,
    Pig,
    Crown,
    Rainbow,
    Sun,
    Tree,
    Umbrella,
    Van,
    Whale,
    Box,
    YoYo,
    Zebra,
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

pub fn draw(pic: Picture, c: Vec2, s: f32) {
    let ink = art::INK;
    match pic {
        // These reuse the objects from the other games.
        Picture::Apple => art::draw_thing(Thing::Apple, c, s * 0.9, Paint::Red.color()),
        Picture::Ball => art::draw_thing(Thing::Ball, c, s * 0.95, Paint::Blue.color()),
        Picture::Fish => art::draw_thing(Thing::Fish, c, s, Paint::Orange.color()),
        Picture::Heart => art::draw_thing(Thing::Heart, c, s * 0.95, Paint::Pink.color()),

        Picture::Cat => {
            let fur = Paint::Orange.color();
            let pink = rgb(255, 170, 190);
            for side in [-1.0, 1.0] {
                let ear = |k: f32| {
                    (
                        vec2(c.x + side * 0.75 * s * k, c.y - 0.3 * s),
                        vec2(c.x + side * 0.55 * s, c.y - 0.95 * s * k),
                        vec2(c.x + side * 0.12 * s, c.y - 0.6 * s),
                    )
                };
                let (a, b, d) = ear(1.0);
                draw_triangle(a, b, d, fur);
                let (a, b, d) = ear(0.8);
                draw_triangle(a, b, d, pink);
            }
            draw_circle(c.x, c.y, 0.72 * s, fur);
            face(c + vec2(0.0, 0.05 * s), 0.55 * s);
            draw_triangle(
                vec2(c.x - 0.08 * s, c.y + 0.02 * s),
                vec2(c.x + 0.08 * s, c.y + 0.02 * s),
                vec2(c.x, c.y + 0.1 * s),
                pink,
            );
            for side in [-1.0, 1.0] {
                for dy in [0.05, 0.22] {
                    draw_line(
                        c.x + side * 0.3 * s,
                        c.y + 0.15 * s,
                        c.x + side * 0.95 * s,
                        c.y + dy * s,
                        2.0,
                        ink,
                    );
                }
            }
        }

        Picture::Dog => {
            let fur = rgb(196, 140, 90);
            let dark = rgb(120, 80, 50);
            for side in [-1.0, 1.0] {
                draw_ellipse(c.x + side * 0.62 * s, c.y + 0.05 * s, 0.22 * s, 0.5 * s, side * 20.0, dark);
            }
            draw_circle(c.x, c.y, 0.68 * s, fur);
            draw_ellipse(c.x, c.y + 0.3 * s, 0.34 * s, 0.25 * s, 0.0, rgb(240, 215, 185));
            draw_circle(c.x, c.y + 0.4 * s, 0.1 * s, rgb(255, 120, 140)); // tongue
            draw_ellipse(c.x, c.y + 0.2 * s, 0.12 * s, 0.08 * s, 0.0, ink); // nose
            eyes(c + vec2(0.0, -0.15 * s), 0.25 * s, 0.09 * s);
        }

        Picture::Egg => {
            draw_ellipse(c.x, c.y, 0.66 * s, 0.86 * s, 0.0, rgb(225, 205, 170));
            draw_ellipse(c.x, c.y, 0.6 * s, 0.8 * s, 0.0, rgb(255, 245, 225));
            face(c + vec2(0.0, 0.15 * s), 0.45 * s);
        }

        Picture::Grapes => {
            let purple = Paint::Purple.color();
            draw_line(c.x, c.y - 0.55 * s, c.x + 0.1 * s, c.y - 0.95 * s, 4.0, Paint::Brown.color());
            draw_ellipse(c.x + 0.3 * s, c.y - 0.75 * s, 0.25 * s, 0.12 * s, -20.0, Paint::Green.color());
            let r = 0.22 * s;
            for (row, count) in [(0, 4), (1, 3), (2, 2), (3, 1)] {
                for i in 0..count {
                    let x = c.x + (i as f32 - (count as f32 - 1.0) / 2.0) * r * 1.8;
                    let y = c.y - 0.4 * s + row as f32 * r * 1.6;
                    draw_circle(x, y, r, art::darken(purple, 0.2));
                    draw_circle(x, y, r * 0.85, purple);
                    draw_circle(x - r * 0.3, y - r * 0.3, r * 0.2, Color::new(1.0, 1.0, 1.0, 0.5));
                }
            }
        }

        Picture::Igloo => {
            let ice = rgb(225, 240, 255);
            let line = rgb(150, 190, 225);
            let base = c + vec2(0.0, 0.45 * s);
            half_disc(base, 0.9 * s, line);
            half_disc(base, 0.85 * s, ice);
            for k in [0.3, 0.6] {
                let y = base.y - 0.85 * s * k;
                let hw = 0.85 * s * (1.0 - k * k).sqrt();
                draw_line(base.x - hw, y, base.x + hw, y, 2.0, line);
            }
            half_disc(base, 0.3 * s, rgb(70, 90, 120)); // door
            draw_line(base.x - 0.95 * s, base.y, base.x + 0.95 * s, base.y, 3.0, line);
        }

        Picture::Jellyfish => {
            let pink = Paint::Pink.color();
            for i in 0..5 {
                let x0 = c.x + (i as f32 - 2.0) * 0.22 * s;
                let mut prev = vec2(x0, c.y);
                for k in 1..=10 {
                    let t = k as f32 / 10.0;
                    let p = vec2(x0 + (t * 10.0 + i as f32).sin() * 0.07 * s, c.y + t * 0.85 * s);
                    draw_line(prev.x, prev.y, p.x, p.y, 4.0, art::lighten(pink, 0.3));
                    prev = p;
                }
            }
            half_disc(c + vec2(0.0, 0.05 * s), 0.72 * s, pink);
            face(c + vec2(0.0, -0.25 * s), 0.45 * s);
        }

        Picture::Kite => {
            let top = vec2(c.x, c.y - 0.9 * s);
            let left = vec2(c.x - 0.55 * s, c.y - 0.2 * s);
            let right = vec2(c.x + 0.55 * s, c.y - 0.2 * s);
            let bottom = vec2(c.x, c.y + 0.6 * s);
            let mid = vec2(c.x, c.y - 0.2 * s);
            draw_triangle(top, left, mid, Paint::Red.color());
            draw_triangle(top, right, mid, Paint::Yellow.color());
            draw_triangle(bottom, left, mid, Paint::Blue.color());
            draw_triangle(bottom, right, mid, Paint::Green.color());
            // Tail with little bows.
            let mut prev = bottom;
            for k in 1..=4 {
                let p = vec2(c.x + (k as f32 * 1.3).sin() * 0.2 * s, bottom.y + k as f32 * 0.12 * s);
                draw_line(prev.x, prev.y, p.x, p.y, 2.0, ink);
                draw_circle(p.x, p.y, 0.05 * s, Paint::Pink.color());
                prev = p;
            }
        }

        Picture::Lollipop => {
            draw_rectangle(c.x - 0.05 * s, c.y, 0.1 * s, 0.95 * s, rgb(235, 225, 210));
            let colors = [Paint::Pink.color(), WHITE, Paint::Purple.color(), WHITE, Paint::Pink.color()];
            draw_circle(c.x, c.y - 0.3 * s, 0.62 * s, art::darken(Paint::Pink.color(), 0.2));
            for (i, col) in colors.iter().enumerate() {
                draw_circle(c.x, c.y - 0.3 * s, (0.58 - i as f32 * 0.12) * s, *col);
            }
        }

        Picture::Moon => {
            crescent(c, 0.8 * s, vec2(0.38 * s, -0.2 * s), 0.68 * s, Paint::Yellow.color());
            eyes(c + vec2(-0.45 * s, -0.1 * s), 0.0, 0.08 * s);
        }

        Picture::Nest => {
            for (dx, dy) in [(-0.3, -0.05), (0.0, -0.15), (0.3, -0.05)] {
                draw_ellipse(c.x + dx * s, c.y + dy * s, 0.2 * s, 0.26 * s, 0.0, rgb(150, 200, 240));
            }
            let brown = rgb(140, 95, 55);
            draw_ellipse(c.x, c.y + 0.25 * s, 0.85 * s, 0.4 * s, 0.0, art::darken(brown, 0.2));
            draw_ellipse(c.x, c.y + 0.2 * s, 0.8 * s, 0.33 * s, 0.0, brown);
            for i in 0..6 {
                let y = c.y + (0.05 + i as f32 * 0.06) * s;
                let tilt = if i % 2 == 0 { 0.08 } else { -0.08 } * s;
                draw_line(c.x - 0.7 * s, y - tilt, c.x + 0.7 * s, y + tilt, 2.0, art::darken(brown, 0.35));
            }
        }

        Picture::Octopus => {
            let purple = Paint::Purple.color();
            for i in 0..6 {
                let side = i as f32 - 2.5;
                let mut p = vec2(c.x + side * 0.16 * s, c.y);
                for k in 0..8 {
                    let t = k as f32 / 7.0;
                    p += vec2(side * 0.05 * s, 0.1 * s);
                    let wiggle = (t * 6.0 + i as f32).sin() * 0.04 * s;
                    draw_circle(p.x + wiggle, p.y, (0.1 - 0.04 * t) * s, purple);
                }
            }
            draw_circle(c.x, c.y - 0.25 * s, 0.6 * s, purple);
            face(c + vec2(0.0, -0.2 * s), 0.5 * s);
        }

        Picture::Pig => {
            let pink = rgb(255, 170, 185);
            let dark = rgb(230, 120, 140);
            for side in [-1.0, 1.0] {
                draw_triangle(
                    vec2(c.x + side * 0.7 * s, c.y - 0.35 * s),
                    vec2(c.x + side * 0.6 * s, c.y - 0.85 * s),
                    vec2(c.x + side * 0.2 * s, c.y - 0.6 * s),
                    dark,
                );
            }
            draw_circle(c.x, c.y, 0.72 * s, pink);
            draw_ellipse(c.x, c.y + 0.2 * s, 0.28 * s, 0.2 * s, 0.0, dark);
            for side in [-1.0, 1.0] {
                draw_ellipse(c.x + side * 0.1 * s, c.y + 0.2 * s, 0.05 * s, 0.08 * s, 0.0, art::darken(dark, 0.4));
            }
            eyes(c + vec2(0.0, -0.2 * s), 0.28 * s, 0.08 * s);
        }

        Picture::Crown => {
            let gold = art::GOLD;
            let base_y = c.y + 0.3 * s;
            draw_rectangle(c.x - 0.75 * s, base_y - 0.1 * s, 1.5 * s, 0.4 * s, art::darken(gold, 0.1));
            for i in 0..3 {
                let x = c.x + (i as f32 - 1.0) * 0.55 * s;
                draw_triangle(vec2(x - 0.3 * s, base_y), vec2(x + 0.3 * s, base_y), vec2(x, c.y - 0.6 * s), gold);
                draw_circle(x, c.y - 0.6 * s, 0.09 * s, gold);
            }
            for (i, p) in [Paint::Red, Paint::Blue, Paint::Green].iter().enumerate() {
                draw_circle(c.x + (i as f32 - 1.0) * 0.5 * s, base_y + 0.1 * s, 0.1 * s, p.color());
            }
        }

        Picture::Rainbow => {
            let base = c + vec2(0.0, 0.35 * s);
            let bands = [Paint::Red, Paint::Orange, Paint::Yellow, Paint::Green, Paint::Blue, Paint::Purple];
            for (i, p) in bands.iter().enumerate() {
                half_disc(base, (0.95 - i as f32 * 0.1) * s, p.color());
            }
            half_disc(base, 0.35 * s, WHITE);
            for side in [-1.0, 1.0] {
                let cl = base + vec2(side * 0.65 * s, 0.0);
                for (dx, r) in [(-0.15, 0.14), (0.0, 0.2), (0.15, 0.14)] {
                    draw_circle(cl.x + dx * s, cl.y, r * s, rgb(215, 228, 245));
                }
            }
        }

        Picture::Sun => {
            let yellow = Paint::Yellow.color();
            for i in 0..10 {
                let a = i as f32 * PI / 5.0;
                let dir = vec2(a.cos(), a.sin());
                let side = vec2(-dir.y, dir.x);
                draw_triangle(
                    c + dir * 0.95 * s,
                    c + dir * 0.55 * s + side * 0.14 * s,
                    c + dir * 0.55 * s - side * 0.14 * s,
                    Paint::Orange.color(),
                );
            }
            draw_circle(c.x, c.y, 0.62 * s, yellow);
            face(c + vec2(0.0, 0.05 * s), 0.5 * s);
        }

        Picture::Tree => {
            draw_rectangle(c.x - 0.13 * s, c.y + 0.1 * s, 0.26 * s, 0.85 * s, Paint::Brown.color());
            let green = Paint::Green.color();
            for (dx, dy, r) in [(-0.35, -0.05, 0.45), (0.35, -0.05, 0.45), (0.0, -0.4, 0.55), (0.0, 0.05, 0.45)] {
                draw_circle(c.x + dx * s, c.y + dy * s, r * s, green);
            }
            for (dx, dy) in [(-0.35, -0.1), (0.25, -0.4), (0.3, 0.1)] {
                draw_circle(c.x + dx * s, c.y + dy * s, 0.08 * s, Paint::Red.color());
            }
        }

        Picture::Umbrella => {
            let red = Paint::Red.color();
            draw_line(c.x, c.y - 0.1 * s, c.x, c.y + 0.7 * s, 4.0, ink);
            art::arc(vec2(c.x - 0.15 * s, c.y + 0.7 * s), 0.15 * s, 0.0, PI, 4.0, ink);
            let base = c + vec2(0.0, -0.05 * s);
            half_disc(base, 0.9 * s, red);
            for i in 0..4 {
                let x = base.x - 0.675 * s + i as f32 * 0.45 * s;
                // Scalloped edge: little white bites out of the bottom.
                draw_circle(x, base.y + 0.02 * s, 0.225 * s, WHITE);
            }
            draw_circle(c.x, c.y - 0.95 * s, 0.06 * s, ink);
        }

        Picture::Van => {
            let blue = Paint::Blue.color();
            let body = Rect::new(c.x - 0.9 * s, c.y - 0.5 * s, 1.8 * s, 0.85 * s);
            art::rounded_rect(body, 0.2 * s, blue);
            let glass = rgb(200, 230, 255);
            art::rounded_rect(Rect::new(c.x - 0.75 * s, c.y - 0.38 * s, 0.6 * s, 0.35 * s), 0.06 * s, glass);
            art::rounded_rect(Rect::new(c.x - 0.05 * s, c.y - 0.38 * s, 0.4 * s, 0.35 * s), 0.06 * s, glass);
            art::rounded_rect(Rect::new(c.x + 0.45 * s, c.y - 0.38 * s, 0.35 * s, 0.35 * s), 0.06 * s, glass);
            for dx in [-0.5, 0.5] {
                draw_circle(c.x + dx * s, c.y + 0.38 * s, 0.22 * s, ink);
                draw_circle(c.x + dx * s, c.y + 0.38 * s, 0.1 * s, rgb(200, 200, 210));
            }
            draw_circle(c.x + 0.85 * s, c.y + 0.1 * s, 0.07 * s, Paint::Yellow.color());
        }

        Picture::Whale => {
            let blue = rgb(70, 130, 200);
            let water = rgb(120, 190, 240);
            for dx in [-0.15, 0.0, 0.15] {
                draw_line(c.x - 0.1 * s, c.y - 0.45 * s, c.x - 0.1 * s + dx * s * 2.0, c.y - 0.9 * s, 4.0, water);
            }
            draw_triangle(
                vec2(c.x + 0.55 * s, c.y),
                vec2(c.x + 1.0 * s, c.y - 0.45 * s),
                vec2(c.x + 1.0 * s, c.y + 0.2 * s),
                blue,
            );
            draw_ellipse(c.x - 0.1 * s, c.y + 0.1 * s, 0.75 * s, 0.5 * s, 0.0, blue);
            draw_ellipse(c.x - 0.15 * s, c.y + 0.35 * s, 0.55 * s, 0.2 * s, 0.0, art::lighten(blue, 0.5));
            eyes(c + vec2(-0.45 * s, -0.05 * s), 0.0, 0.08 * s);
            art::arc(c + vec2(-0.45 * s, 0.05 * s), 0.15 * s, 0.2 * PI, 0.8 * PI, 3.0, ink);
        }

        Picture::Box => {
            let brown = rgb(200, 150, 95);
            let dark = art::darken(brown, 0.2);
            draw_triangle(vec2(c.x - 0.7 * s, c.y - 0.4 * s), vec2(c.x - 0.2 * s, c.y - 0.4 * s), vec2(c.x - 0.85 * s, c.y - 0.85 * s), dark);
            draw_triangle(vec2(c.x + 0.7 * s, c.y - 0.4 * s), vec2(c.x + 0.2 * s, c.y - 0.4 * s), vec2(c.x + 0.85 * s, c.y - 0.85 * s), dark);
            draw_rectangle(c.x - 0.7 * s, c.y - 0.4 * s, 1.4 * s, 1.2 * s, brown);
            draw_rectangle(c.x - 0.1 * s, c.y - 0.4 * s, 0.2 * s, 0.5 * s, rgb(230, 200, 150)); // tape
            face(c + vec2(0.0, 0.3 * s), 0.5 * s);
        }

        Picture::YoYo => {
            draw_line(c.x, c.y, c.x + 0.1 * s, c.y - 0.95 * s, 2.0, ink);
            draw_circle(c.x + 0.1 * s, c.y - 0.95 * s, 0.07 * s, ink);
            let red = Paint::Red.color();
            draw_circle(c.x, c.y + 0.15 * s, 0.7 * s, art::darken(red, 0.25));
            draw_circle(c.x, c.y + 0.15 * s, 0.6 * s, red);
            draw_circle(c.x, c.y + 0.15 * s, 0.25 * s, art::lighten(red, 0.4));
            draw_circle(c.x, c.y + 0.15 * s, 0.08 * s, ink);
        }

        Picture::Zebra => {
            let (hx, hy, a, b) = (c.x, c.y, 0.5 * s, 0.8 * s);
            for side in [-1.0, 1.0] {
                draw_triangle(
                    vec2(hx + side * 0.2 * s, hy - 0.6 * s),
                    vec2(hx + side * 0.5 * s, hy - 0.95 * s),
                    vec2(hx + side * 0.45 * s, hy - 0.45 * s),
                    ink,
                );
            }
            draw_rectangle(hx - 0.12 * s, hy - 0.95 * s, 0.24 * s, 0.35 * s, ink); // mane
            draw_ellipse(hx, hy, a, b, 0.0, WHITE);
            draw_ellipse(hx, hy, a, b, 0.0, rgb(245, 245, 245));
            // Stripes from each side, following the head's curve.
            for k in [-0.55, -0.3, -0.05, 0.2] {
                let y = hy + k * b;
                let hw = a * (1.0 - k * k).sqrt();
                for side in [-1.0, 1.0] {
                    draw_line(hx + side * hw * 0.98, y, hx + side * hw * 0.35, y + 0.05 * s, 0.1 * s, ink);
                }
            }
            draw_ellipse(hx, hy + 0.5 * s, 0.38 * s, 0.28 * s, 0.0, rgb(120, 120, 130));
            for side in [-1.0, 1.0] {
                draw_circle(hx + side * 0.13 * s, hy + 0.5 * s, 0.05 * s, ink);
            }
            eyes(vec2(hx, hy - 0.2 * s), 0.22 * s, 0.08 * s);
        }
    }
}

/// Two cute eyes, `spread` apart from the center.
fn eyes(c: Vec2, spread: f32, r: f32) {
    for dx in [-spread, spread] {
        draw_circle(c.x + dx, c.y, r, art::INK);
        draw_circle(c.x + dx + r * 0.3, c.y - r * 0.3, r * 0.35, WHITE);
        if spread == 0.0 {
            break; // a single eye (side view)
        }
    }
}

/// The top half of a circle, sitting on `base`.
fn half_disc(base: Vec2, r: f32, color: Color) {
    let steps = 32;
    for i in 0..steps {
        let a0 = PI + PI * i as f32 / steps as f32;
        let a1 = PI + PI * (i + 1) as f32 / steps as f32;
        draw_triangle(
            base,
            base + vec2(a0.cos(), a0.sin()) * r,
            base + vec2(a1.cos(), a1.sin()) * r,
            color,
        );
    }
}

/// A crescent: a circle with a bite taken out by a second circle (offset by `bite_offset`).
fn crescent(c: Vec2, r: f32, bite_offset: Vec2, bite_r: f32, color: Color) {
    // Fill it in thin horizontal strips: in each strip, color from the left edge of the
    // circle to the left edge of the bite.
    let bite = c + bite_offset;
    let steps = 48;
    let x_range = |y: f32| {
        let dy = y - c.y;
        let half = (r * r - dy * dy).max(0.0).sqrt();
        let left = c.x - half;
        let mut right = c.x + half;
        let bdy = y - bite.y;
        if bdy.abs() < bite_r {
            let bite_left = bite.x - (bite_r * bite_r - bdy * bdy).sqrt();
            right = right.min(bite_left);
        }
        (left, right.max(left))
    };
    for i in 0..steps {
        let y0 = c.y - r + 2.0 * r * i as f32 / steps as f32;
        let y1 = c.y - r + 2.0 * r * (i + 1) as f32 / steps as f32;
        let (l0, r0) = x_range(y0);
        let (l1, r1) = x_range(y1);
        draw_triangle(vec2(l0, y0), vec2(r0, y0), vec2(r1, y1), color);
        draw_triangle(vec2(l0, y0), vec2(r1, y1), vec2(l1, y1), color);
    }
}
