//! Everything we draw: colors, cute objects, cards, buckets, text and the background.
//! All art is made from simple shapes, so there are no image files to manage.

use macroquad::prelude::*;
use std::f32::consts::PI;

pub const SKY: Color = Color::new(0.87, 0.95, 1.0, 1.0);
pub const INK: Color = Color::new(0.20, 0.18, 0.28, 1.0);
pub const SHADOW: Color = Color::new(0.72, 0.82, 0.90, 1.0);
pub const GOLD: Color = Color::new(1.0, 0.80, 0.10, 1.0);

/// The colors the game teaches. `Paint` (not `Color`) so it doesn't clash with macroquad's `Color`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Paint {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Brown,
}

impl Paint {
    pub const ALL: [Paint; 8] = [
        Paint::Red,
        Paint::Orange,
        Paint::Yellow,
        Paint::Green,
        Paint::Blue,
        Paint::Purple,
        Paint::Pink,
        Paint::Brown,
    ];

    pub fn color(self) -> Color {
        match self {
            Paint::Red => Color::from_rgba(214, 22, 22, 255),
            Paint::Orange => Color::from_rgba(251, 140, 0, 255),
            Paint::Yellow => Color::from_rgba(253, 216, 53, 255),
            Paint::Green => Color::from_rgba(67, 160, 71, 255),
            Paint::Blue => Color::from_rgba(30, 136, 229, 255),
            Paint::Purple => Color::from_rgba(142, 36, 170, 255),
            Paint::Pink => Color::from_rgba(240, 98, 146, 255),
            Paint::Brown => Color::from_rgba(121, 85, 72, 255),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Paint::Red => "red",
            Paint::Orange => "orange",
            Paint::Yellow => "yellow",
            Paint::Green => "green",
            Paint::Blue => "blue",
            Paint::Purple => "purple",
            Paint::Pink => "pink",
            Paint::Brown => "brown",
        }
    }
}

/// The objects we can draw.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Thing {
    Ball,
    Apple,
    Star,
    Heart,
    Balloon,
    Fish,
    Circle,
    Square,
    Triangle,
    Rectangle,
    Diamond,
    Oval,
}

impl Thing {
    pub const ALL: [Thing; 12] = [
        Thing::Ball,
        Thing::Apple,
        Thing::Star,
        Thing::Heart,
        Thing::Balloon,
        Thing::Fish,
        Thing::Circle,
        Thing::Square,
        Thing::Triangle,
        Thing::Rectangle,
        Thing::Diamond,
        Thing::Oval,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Thing::Ball => "ball",
            Thing::Apple => "apple",
            Thing::Star => "star",
            Thing::Heart => "heart",
            Thing::Balloon => "balloon",
            Thing::Fish => "fish",
            Thing::Circle => "circle",
            Thing::Square => "square",
            Thing::Triangle => "triangle",
            Thing::Rectangle => "rectangle",
            Thing::Diamond => "diamond",
            Thing::Oval => "oval",
        }
    }

    pub fn plural(self) -> &'static str {
        match self {
            Thing::Ball => "balls",
            Thing::Apple => "apples",
            Thing::Star => "stars",
            Thing::Heart => "hearts",
            Thing::Balloon => "balloons",
            Thing::Fish => "fish",
            Thing::Circle => "circles",
            Thing::Square => "squares",
            Thing::Triangle => "triangles",
            Thing::Rectangle => "rectangles",
            Thing::Diamond => "diamonds",
            Thing::Oval => "ovals",
        }
    }
}

// ---------- color helpers ----------

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub fn lighten(c: Color, t: f32) -> Color {
    mix(c, Color::new(1.0, 1.0, 1.0, c.a), t)
}

pub fn darken(c: Color, t: f32) -> Color {
    mix(c, Color::new(0.0, 0.0, 0.0, c.a), t)
}

/// A version of `c` that shows up on white: very light colors (like yellow) get darkened.
pub fn readable(c: Color) -> Color {
    let brightness = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
    if brightness > 0.7 {
        darken(c, 0.25)
    } else {
        c
    }
}

/// Side-to-side wiggle used when a wrong answer is tapped. `amount` fades from 1 to 0.
pub fn shake_x(amount: f32, time: f32) -> f32 {
    (time * 45.0).sin() * amount * 14.0
}

// ---------- basic shapes ----------

pub fn rounded_rect(r: Rect, radius: f32, color: Color) {
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0);
    draw_rectangle(r.x + rad, r.y, r.w - 2.0 * rad, r.h, color);
    draw_rectangle(r.x, r.y + rad, rad, r.h - 2.0 * rad, color);
    draw_rectangle(r.x + r.w - rad, r.y + rad, rad, r.h - 2.0 * rad, color);
    for (cx, cy) in [
        (r.x + rad, r.y + rad),
        (r.x + r.w - rad, r.y + rad),
        (r.x + rad, r.y + r.h - rad),
        (r.x + r.w - rad, r.y + r.h - rad),
    ] {
        draw_circle(cx, cy, rad, color);
    }
}

/// A white (or colored) rounded card with a soft drop shadow.
pub fn card(r: Rect, fill: Color) {
    let rad = r.w.min(r.h) * 0.18;
    let off = (r.h * 0.05).max(4.0);
    rounded_rect(Rect::new(r.x, r.y + off, r.w, r.h), rad, SHADOW);
    rounded_rect(r, rad, fill);
}

/// A glowing outline behind a card, used for hints.
pub fn glow(r: Rect, time: f32) {
    let pad = r.w.min(r.h) * (0.07 + 0.03 * (time * 6.0).sin());
    let g = Rect::new(r.x - pad, r.y - pad, r.w + pad * 2.0, r.h + pad * 2.0);
    rounded_rect(g, g.w.min(g.h) * 0.2, GOLD);
}

/// Shrink or grow a rect around its center.
pub fn scale_rect(r: Rect, s: f32) -> Rect {
    let c = r.center();
    Rect::new(c.x - r.w * s / 2.0, c.y - r.h * s / 2.0, r.w * s, r.h * s)
}

/// A thick curved line (part of a circle). Angles in radians; screen y points down,
/// so going from 0.2π to 0.8π draws a smile.
pub fn arc(c: Vec2, r: f32, a0: f32, a1: f32, thick: f32, color: Color) {
    let steps = 14;
    let mut prev = c + vec2(a0.cos(), a0.sin()) * r;
    draw_circle(prev.x, prev.y, thick / 2.0, color);
    for i in 1..=steps {
        let a = a0 + (a1 - a0) * i as f32 / steps as f32;
        let p = c + vec2(a.cos(), a.sin()) * r;
        draw_line(prev.x, prev.y, p.x, p.y, thick, color);
        draw_circle(p.x, p.y, thick / 2.0, color);
        prev = p;
    }
}

pub fn star(c: Vec2, r_out: f32, color: Color) {
    let r_in = r_out * 0.48;
    let pts: [Vec2; 10] = std::array::from_fn(|i| {
        let a = -PI / 2.0 + i as f32 * PI / 5.0;
        let r = if i % 2 == 0 { r_out } else { r_in };
        c + vec2(a.cos(), a.sin()) * r
    });
    for i in 0..10 {
        draw_triangle(c, pts[i], pts[(i + 1) % 10], color);
    }
}

fn heart(c: Vec2, s: f32, color: Color) {
    draw_circle(c.x - 0.42 * s, c.y - 0.2 * s, 0.48 * s, color);
    draw_circle(c.x + 0.42 * s, c.y - 0.2 * s, 0.48 * s, color);
    draw_triangle(
        vec2(c.x - 0.88 * s, c.y - 0.05 * s),
        vec2(c.x + 0.88 * s, c.y - 0.05 * s),
        vec2(c.x, c.y + 0.85 * s),
        color,
    );
}

/// A little smiling face. Gives everything a friendly, kid-like look.
pub fn face(c: Vec2, s: f32) {
    let ink = Color::new(0.15, 0.12, 0.18, 1.0);
    for dx in [-0.32, 0.32] {
        let e = c + vec2(dx * s, -0.12 * s);
        draw_circle(e.x, e.y, 0.13 * s, ink);
        draw_circle(e.x + 0.04 * s, e.y - 0.04 * s, 0.045 * s, WHITE);
    }
    arc(c + vec2(0.0, -0.02 * s), 0.3 * s, 0.2 * PI, 0.8 * PI, 0.08 * s, ink);
    let blush = Color::new(1.0, 0.45, 0.55, 0.45);
    for dx in [-0.55, 0.55] {
        draw_circle(c.x + dx * s, c.y + 0.12 * s, 0.1 * s, blush);
    }
}

/// Draw one of our objects centered at `c`. `s` is roughly its radius.
pub fn draw_thing(thing: Thing, c: Vec2, s: f32, color: Color) {
    let dark = darken(color, 0.25);
    let shine = Color::new(1.0, 1.0, 1.0, 0.55);
    match thing {
        Thing::Ball => {
            draw_circle(c.x, c.y, 0.9 * s, dark);
            draw_circle(c.x, c.y, 0.82 * s, color);
            draw_circle(c.x - 0.35 * s, c.y - 0.35 * s, 0.18 * s, shine);
            face(c + vec2(0.0, 0.1 * s), 0.55 * s);
        }
        Thing::Apple => {
            let brown = Color::from_rgba(110, 70, 40, 255);
            draw_rectangle(c.x - 0.05 * s, c.y - 0.85 * s, 0.1 * s, 0.35 * s, brown);
            draw_circle(c.x + 0.22 * s, c.y - 0.68 * s, 0.17 * s, Paint::Green.color());
            for (dx, dy, r) in [(-0.3, 0.05, 0.62), (0.3, 0.05, 0.62), (0.0, 0.2, 0.6)] {
                draw_circle(c.x + dx * s, c.y + dy * s, r * s, color);
            }
            draw_circle(c.x - 0.4 * s, c.y - 0.2 * s, 0.14 * s, shine);
            face(c + vec2(0.0, 0.15 * s), 0.5 * s);
        }
        Thing::Star => {
            star(c, s, dark);
            star(c, s * 0.86, color);
            face(c + vec2(0.0, 0.1 * s), 0.38 * s);
        }
        Thing::Heart => {
            heart(c, s, dark);
            heart(c + vec2(0.0, -0.02 * s), s * 0.88, color);
            face(c + vec2(0.0, -0.05 * s), 0.45 * s);
        }
        Thing::Balloon => {
            draw_line(c.x, c.y + 0.65 * s, c.x + 0.12 * s, c.y + 1.05 * s, 2.0, INK);
            draw_triangle(
                vec2(c.x, c.y + 0.5 * s),
                vec2(c.x - 0.12 * s, c.y + 0.7 * s),
                vec2(c.x + 0.12 * s, c.y + 0.7 * s),
                dark,
            );
            draw_circle(c.x, c.y - 0.15 * s, 0.72 * s, color);
            draw_circle(c.x - 0.3 * s, c.y - 0.45 * s, 0.14 * s, shine);
            face(c + vec2(0.0, -0.1 * s), 0.5 * s);
        }
        // The basic shapes: a darker outline, the color on top, and a friendly face.
        Thing::Circle => {
            draw_circle(c.x, c.y, 0.85 * s, dark);
            draw_circle(c.x, c.y, 0.77 * s, color);
            face(c + vec2(0.0, 0.08 * s), 0.5 * s);
        }
        Thing::Square => {
            let o = 0.05 * s;
            rounded_rect(Rect::new(c.x - 0.78 * s, c.y - 0.78 * s, 1.56 * s, 1.56 * s), 0.12 * s, dark);
            rounded_rect(Rect::new(c.x - 0.78 * s + o, c.y - 0.78 * s + o, 1.56 * s - 2.0 * o, 1.56 * s - 2.0 * o), 0.1 * s, color);
            face(c + vec2(0.0, 0.08 * s), 0.5 * s);
        }
        Thing::Triangle => {
            let tri = |k: f32, col: Color| {
                draw_triangle(
                    c + vec2(0.0, -0.95 * s) * k,
                    c + vec2(-0.95 * s, 0.75 * s) * k,
                    c + vec2(0.95 * s, 0.75 * s) * k,
                    col,
                )
            };
            tri(1.0, dark);
            tri(0.88, color);
            face(c + vec2(0.0, 0.25 * s), 0.4 * s);
        }
        Thing::Rectangle => {
            let o = 0.05 * s;
            rounded_rect(Rect::new(c.x - 0.98 * s, c.y - 0.58 * s, 1.96 * s, 1.16 * s), 0.12 * s, dark);
            rounded_rect(Rect::new(c.x - 0.98 * s + o, c.y - 0.58 * s + o, 1.96 * s - 2.0 * o, 1.16 * s - 2.0 * o), 0.1 * s, color);
            face(c + vec2(0.0, 0.06 * s), 0.45 * s);
        }
        Thing::Diamond => {
            let diamond = |k: f32, col: Color| {
                let (top, bot) = (c + vec2(0.0, -0.95 * s * k), c + vec2(0.0, 0.95 * s * k));
                let (left, right) = (c + vec2(-0.7 * s * k, 0.0), c + vec2(0.7 * s * k, 0.0));
                draw_triangle(top, left, right, col);
                draw_triangle(bot, left, right, col);
            };
            diamond(1.0, dark);
            diamond(0.88, color);
            face(c + vec2(0.0, 0.08 * s), 0.4 * s);
        }
        Thing::Oval => {
            draw_ellipse(c.x, c.y, 0.98 * s, 0.66 * s, 0.0, dark);
            draw_ellipse(c.x, c.y, 0.9 * s, 0.58 * s, 0.0, color);
            face(c + vec2(0.0, 0.06 * s), 0.45 * s);
        }
        Thing::Fish => {
            draw_triangle(
                vec2(c.x - 0.35 * s, c.y),
                vec2(c.x - 0.95 * s, c.y - 0.5 * s),
                vec2(c.x - 0.95 * s, c.y + 0.5 * s),
                dark,
            );
            draw_circle(c.x + 0.15 * s, c.y, 0.62 * s, color);
            draw_circle(c.x + 0.42 * s, c.y - 0.15 * s, 0.15 * s, WHITE);
            draw_circle(c.x + 0.45 * s, c.y - 0.15 * s, 0.08 * s, INK);
            arc(c + vec2(0.4 * s, 0.05 * s), 0.14 * s, 0.2 * PI, 0.8 * PI, 0.05 * s, INK);
        }
    }
}

/// A friendly bucket for the color sorting game. `r` is the whole bucket area.
pub fn bucket(r: Rect, color: Color) {
    let cx = r.x + r.w / 2.0;
    let top = r.y + r.h * 0.12;
    let bot = r.y + r.h;
    let dark = darken(color, 0.25);
    let o = r.w * 0.03;
    trapezoid(cx, top - o, bot + o, r.w * 0.5 + o, r.w * 0.36 + o, dark);
    trapezoid(cx, top, bot, r.w * 0.5, r.w * 0.36, color);
    rounded_rect(
        Rect::new(r.x - r.w * 0.02, r.y + r.h * 0.03, r.w * 1.04, r.h * 0.15),
        r.h * 0.07,
        dark,
    );
    rounded_rect(
        Rect::new(r.x + r.w * 0.01, r.y + r.h * 0.055, r.w * 0.98, r.h * 0.095),
        r.h * 0.045,
        lighten(color, 0.3),
    );
    face(vec2(cx, r.y + r.h * 0.48), r.w * 0.26);
}

fn trapezoid(cx: f32, top: f32, bot: f32, half_top: f32, half_bot: f32, color: Color) {
    let a = vec2(cx - half_top, top);
    let b = vec2(cx + half_top, top);
    let c = vec2(cx + half_bot, bot);
    let d = vec2(cx - half_bot, bot);
    draw_triangle(a, b, c, color);
    draw_triangle(a, c, d, color);
}

/// Small white circle with a number in it, e.g. "3" on the third counted apple.
pub fn badge(font: Option<&Font>, c: Vec2, r: f32, text: &str) {
    draw_circle(c.x, c.y, r * 1.1, INK);
    draw_circle(c.x, c.y, r, WHITE);
    text_center(font, text, c, r * 1.4, INK);
}

// ---------- text ----------

/// Try a font dropped into `assets/font.ttf`, then friendly Windows fonts, else macroquad's default.
pub fn load_font() -> Option<Font> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let mut paths = vec![crate::assets::dir().join("font.ttf").to_string_lossy().into_owned()];
    for name in ["comicbd.ttf", "comic.ttf", "segoeuib.ttf", "arialbd.ttf"] {
        paths.push(format!("{windir}\\Fonts\\{name}"));
    }
    paths
        .iter()
        .filter_map(|p| std::fs::read(p).ok())
        .find_map(|bytes| load_ttf_font_from_bytes(&bytes).ok())
}

/// The font has to draw every letter again for each new text size, which is slow (a
/// tenth of a second for a big title). Text that grows or pulses would do that every
/// frame. So text is only ever drawn at a few sizes, 10% apart, and stretched a little
/// to the exact size. Returns (size to draw at, how much to stretch it).
fn font_size(size: f32) -> (u16, f32) {
    let size = size.max(1.0);
    let step = (size.ln() / 1.1f32.ln()).round();
    let fs = 1.1f32.powf(step).round().max(1.0);
    (fs as u16, size / fs)
}

/// How big this text will be at `size` (use this instead of macroquad's `measure_text`).
pub fn measure(font: Option<&Font>, text: &str, size: f32) -> TextDimensions {
    let (fs, scale) = font_size(size);
    measure_text(text, font, fs, scale)
}

/// Draw text with its baseline at (x, y).
pub fn text_at(font: Option<&Font>, text: &str, x: f32, y: f32, size: f32, color: Color) {
    let (font_size, font_scale) = font_size(size);
    draw_text_ex(text, x, y, TextParams { font, font_size, font_scale, color, ..Default::default() });
}

/// Draw text centered on a point.
pub fn text_center(font: Option<&Font>, text: &str, c: Vec2, size: f32, color: Color) {
    text_center_zoomed(font, text, c, size, 1.0, color);
}

/// Text that grows, shrinks or pulses: the letters are drawn once at `size`, then stretched
/// by `zoom`, so animating `zoom` costs nothing.
pub fn text_center_zoomed(font: Option<&Font>, text: &str, c: Vec2, size: f32, zoom: f32, color: Color) {
    let (font_size, stretch) = font_size(size);
    let font_scale = stretch * zoom.max(0.01);
    let d = measure_text(text, font, font_size, font_scale);
    let (x, y) = (c.x - d.width / 2.0, c.y + d.offset_y / 2.0);
    draw_text_ex(text, x, y, TextParams { font, font_size, font_scale, color, ..Default::default() });
}

/// A word label like "BALL", in capitals to match the Letters game, with the first letter
/// bigger and in `first_color`, so the word's first letter stands out. `bg` draws a pill
/// behind it (use it when the label sits on something colorful). The label shrinks if
/// needed to fit in `max_w` pixels.
pub fn word_label(
    font: Option<&Font>,
    word: &str,
    c: Vec2,
    size: f32,
    max_w: f32,
    first_color: Color,
    bg: Option<Color>,
) {
    let word = word.to_uppercase();
    let mut chars = word.chars();
    let Some(first) = chars.next() else { return };
    let first = first.to_string();
    let rest: String = chars.collect();

    let widths = |size: f32| {
        let w_first = measure(font, &first, size * 1.35).width;
        let w_rest = measure(font, &rest, size).width;
        (w_first, w_rest, size * 0.06)
    };
    let (w_first, w_rest, gap) = widths(size);
    let size = if w_first + gap + w_rest > max_w {
        size * max_w / (w_first + gap + w_rest)
    } else {
        size
    };
    let (w_first, w_rest, gap) = widths(size);
    let total = w_first + gap + w_rest;
    let x = c.x - total / 2.0;
    let baseline = c.y + size * 0.36;

    if let Some(bg) = bg {
        let pad = size * 0.35;
        rounded_rect(
            Rect::new(x - pad, c.y - size * 0.75, total + pad * 2.0, size * 1.5),
            size * 0.5,
            bg,
        );
    }
    text_at(font, &first, x, baseline, size * 1.35, first_color);
    text_at(font, &rest, x + w_first + gap, baseline, size, INK);
}

/// Draw text starting at `x`, vertically centered on `cy`.
pub fn text_left(font: Option<&Font>, text: &str, x: f32, cy: f32, size: f32, color: Color) {
    let d = measure(font, text, size);
    text_at(font, text, x, cy + d.offset_y / 2.0, size, color);
}

// ---------- background ----------

pub fn background(time: f32) {
    let (w, h) = (screen_width(), screen_height());
    // A full-screen rectangle rather than clear_background(): clearing throws away drawing
    // that's still queued up this frame, like pictures being drawn into puzzle images.
    draw_rectangle(0.0, 0.0, w, h, SKY);

    // Drifting clouds.
    for i in 0..4 {
        let speed = 0.010 + i as f32 * 0.004;
        let x = ((i as f32 * 0.29 + time * speed) % 1.3 - 0.15) * w;
        let y = h * (0.09 + 0.07 * (i % 3) as f32);
        cloud(vec2(x, y), h * 0.045);
    }

    // Soft rolling hills.
    // (draw_poly with many sides: plain draw_circle looks blocky at this size)
    draw_poly(w * 0.15, h * 1.35, 96, h * 0.6, 0.0, Color::from_rgba(205, 238, 205, 255));
    draw_poly(w * 0.62, h * 1.48, 96, h * 0.72, 0.0, Color::from_rgba(190, 230, 195, 255));
    draw_poly(w * 1.0, h * 1.32, 96, h * 0.55, 0.0, Color::from_rgba(210, 240, 210, 255));
}

fn cloud(c: Vec2, s: f32) {
    for (dx, dy, r) in [(0.0, 0.0, 1.0), (-1.1, 0.3, 0.75), (1.1, 0.3, 0.8), (0.5, -0.4, 0.7)] {
        draw_circle(c.x + dx * s, c.y + dy * s, r * s, WHITE);
    }
}

// ---------- buttons ----------

pub fn round_button(c: Vec2, r: f32) {
    draw_circle(c.x, c.y + r * 0.1, r, SHADOW);
    draw_circle(c.x, c.y, r, WHITE);
}

pub fn home_icon(c: Vec2, r: f32) {
    draw_rectangle(c.x - 0.4 * r, c.y - 0.08 * r, 0.8 * r, 0.55 * r, Paint::Yellow.color());
    draw_triangle(
        vec2(c.x - 0.6 * r, c.y - 0.02 * r),
        vec2(c.x + 0.6 * r, c.y - 0.02 * r),
        vec2(c.x, c.y - 0.58 * r),
        Paint::Red.color(),
    );
    draw_rectangle(c.x - 0.12 * r, c.y + 0.15 * r, 0.24 * r, 0.32 * r, Paint::Brown.color());
}

/// A cartoon pointing hand, with the fingertip at `tip`. `pressing` squashes it a little,
/// like it's pushing down on the screen.
pub fn hand(tip: Vec2, s: f32, pressing: bool) {
    let skin = Color::from_rgba(255, 214, 180, 255);
    let line = Color::from_rgba(200, 140, 110, 255);
    let squash = if pressing { 0.9 } else { 1.0 };
    let t = tip + vec2(s * 0.12, 0.0); // tip sits on the point; the hand trails down-right
    let finger = Rect::new(t.x - 0.13 * s, t.y, 0.26 * s, 0.75 * s * squash);
    let palm = Rect::new(t.x - 0.2 * s, t.y + 0.5 * s * squash, 0.75 * s, 0.7 * s);
    // Outline first (slightly bigger, darker), then the skin on top.
    for (r, col, grow) in [(finger, line, 0.04 * s), (palm, line, 0.04 * s), (finger, skin, 0.0), (palm, skin, 0.0)] {
        let r = Rect::new(r.x - grow, r.y - grow, r.w + grow * 2.0, r.h + grow * 2.0);
        rounded_rect(r, r.w.min(r.h) * 0.45, col);
    }
    // Folded fingers.
    for i in 0..3 {
        let c = vec2(t.x + (0.2 + i as f32 * 0.16) * s, t.y + 0.62 * s * squash);
        draw_circle(c.x, c.y, 0.1 * s, line);
        draw_circle(c.x, c.y, 0.08 * s, skin);
    }
    // Fingernail.
    rounded_rect(Rect::new(t.x - 0.08 * s, t.y + 0.05 * s, 0.16 * s, 0.14 * s), 0.05 * s, Color::from_rgba(255, 235, 220, 255));
}

/// Two little people: "who's playing?"
pub fn people_icon(c: Vec2, r: f32) {
    for (dx, dy, s, paint) in [(0.25, 0.05, 0.85, Paint::Green), (-0.18, -0.02, 1.0, Paint::Blue)] {
        let p = c + vec2(dx * r, dy * r);
        let color = paint.color();
        draw_circle(p.x, p.y - 0.25 * r * s, 0.17 * r * s, color);
        // Shoulders: the top half of a circle.
        let base = p + vec2(0.0, 0.35 * r * s);
        let rad = 0.3 * r * s;
        for i in 0..16 {
            let a0 = PI + PI * i as f32 / 16.0;
            let a1 = PI + PI * (i + 1) as f32 / 16.0;
            draw_triangle(base, base + vec2(a0.cos(), a0.sin()) * rad, base + vec2(a1.cos(), a1.sin()) * rad, color);
        }
    }
}

/// A speech bubble with a question mark: "say the question again".
pub fn question_bubble_icon(font: Option<&Font>, c: Vec2, r: f32, color: Color) {
    let body = Rect::new(c.x - 0.6 * r, c.y - 0.5 * r, 1.2 * r, 0.85 * r);
    rounded_rect(body, 0.3 * r, color);
    draw_triangle(
        vec2(c.x - 0.35 * r, c.y + 0.3 * r),
        vec2(c.x - 0.05 * r, c.y + 0.3 * r),
        vec2(c.x - 0.45 * r, c.y + 0.6 * r),
        color,
    );
    text_center(font, "?", vec2(c.x, c.y - 0.08 * r), 0.8 * r, WHITE);
}

/// The speaker with a red X: sound is off.
pub fn muted_speaker_icon(c: Vec2, r: f32, color: Color) {
    speaker_body(c, r, color);
    let red = Paint::Red.color();
    let (x0, x1, y0, y1) = (c.x + 0.15 * r, c.x + 0.55 * r, c.y - 0.2 * r, c.y + 0.2 * r);
    draw_line(x0, y0, x1, y1, 0.1 * r, red);
    draw_line(x0, y1, x1, y0, 0.1 * r, red);
}

fn speaker_body(c: Vec2, r: f32, color: Color) {
    draw_rectangle(c.x - 0.55 * r, c.y - 0.18 * r, 0.25 * r, 0.36 * r, color);
    let a = vec2(c.x - 0.35 * r, c.y - 0.18 * r);
    let b = vec2(c.x + 0.05 * r, c.y - 0.45 * r);
    let cc = vec2(c.x + 0.05 * r, c.y + 0.45 * r);
    let d = vec2(c.x - 0.35 * r, c.y + 0.18 * r);
    draw_triangle(a, b, cc, color);
    draw_triangle(a, cc, d, color);
}

pub fn speaker_icon(c: Vec2, r: f32, color: Color) {
    speaker_body(c, r, color);
    for rad in [0.25, 0.45] {
        arc(vec2(c.x + 0.05 * r, c.y), rad * r, -0.3 * PI, 0.3 * PI, 0.08 * r, color);
    }
}
