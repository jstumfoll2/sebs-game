//! What to write: lines, curves and shapes for the early Writing levels, and all 26 capital
//! letters, each as a list of *strokes* in the order you'd draw them.
//!
//! A stroke is a list of points in a 1x1 box (x to the right, y *down*, like the screen).
//! Curves are built from little arcs, so a stroke is really a many-cornered line.

use macroquad::prelude::{vec2, Vec2};

/// Letters are taller than wide.
const LETTER_ASPECT: f32 = 0.7;

/// Something to write: one or more strokes, and how wide the box is compared to its height.
#[derive(Clone, Debug)]
pub struct Glyph {
    pub strokes: Vec<Vec<Vec2>>,
    pub aspect: f32,
    /// Another natural way to write it as one unbroken stroke (N and M), if there is one.
    pub joined: Option<Vec<Vec2>>,
}

/// A shape for the early levels, with what to say about it.
pub struct Item {
    /// The instruction ("Draw a line down!").
    pub say: &'static str,
    /// What it's called, for the praise ("That's a nice circle!").
    pub name: &'static str,
    pub glyph: Glyph,
}

// ---------- building strokes ----------

/// One piece of a stroke.
enum Seg {
    /// A straight line to this point.
    To(f32, f32),
    /// Part of an ellipse: center, radii, and start/end angle in degrees. Angles go clockwise
    /// on screen (0 = right, 90 = down, 270 = up); a smaller end angle goes the other way.
    Arc(f32, f32, f32, f32, f32, f32),
}

/// A stroke starting at `(x, y)` and made of `parts`.
fn stroke(x: f32, y: f32, parts: &[Seg]) -> Vec<Vec2> {
    let mut points = vec![vec2(x, y)];
    for part in parts {
        match *part {
            Seg::To(x, y) => points.push(vec2(x, y)),
            Seg::Arc(cx, cy, rx, ry, a0, a1) => {
                let steps = ((a1 - a0).abs() / 6.0).ceil().max(2.0) as usize;
                let on_arc = |i: usize| {
                    let a = (a0 + (a1 - a0) * i as f32 / steps as f32).to_radians();
                    vec2(cx + rx * a.cos(), cy + ry * a.sin())
                };
                // An arc that begins where the stroke already is doesn't repeat that point.
                let first = if points.last().is_some_and(|p| p.distance(on_arc(0)) < 0.005) { 1 } else { 0 };
                points.extend((first..=steps).map(on_arc));
            }
        }
    }
    points
}

/// A straight stroke between two points.
fn line(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Vec2> {
    stroke(x0, y0, &[Seg::To(x1, y1)])
}

/// A stroke that is just an arc (its first point is on the arc).
fn arc(cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) -> Vec<Vec2> {
    let start = vec2(cx + rx * a0.to_radians().cos(), cy + ry * a0.to_radians().sin());
    stroke(start.x, start.y, &[Seg::Arc(cx, cy, rx, ry, a0, a1)])
}

/// A stroke joining these points with straight lines.
fn poly(points: &[(f32, f32)]) -> Vec<Vec2> {
    points.iter().map(|&(x, y)| vec2(x, y)).collect()
}

// ---------- shapes ----------

/// The things to trace at level 1 (lines), 2 (curves) or 3 (shapes).
pub fn shapes(level: u32) -> Vec<Item> {
    let item = |say, name, strokes| Item { say, name, glyph: Glyph { strokes, aspect: 1.0, joined: None } };
    match level {
        1 => vec![
            item("Trace the line! Follow the dots, from the top to the bottom.", "line", vec![line(0.5, 0.05, 0.5, 0.95)]),
            item("Trace the line! Follow the dots, from the left to the right.", "line", vec![line(0.05, 0.5, 0.95, 0.5)]),
            item("Trace the slanted line! Follow the dots, down to the right.", "line", vec![line(0.1, 0.1, 0.9, 0.9)]),
            item("Trace the slanted line! Follow the dots, down to the left.", "line", vec![line(0.9, 0.1, 0.1, 0.9)]),
        ],
        2 => {
            let wave: Vec<Vec2> = (0..=48)
                .map(|i| {
                    let t = i as f32 / 48.0;
                    vec2(0.05 + 0.9 * t, 0.5 - 0.28 * (t * std::f32::consts::TAU * 2.0).sin())
                })
                .collect();
            vec![
                item("Trace the hill! Follow the dots up and over.", "hill", vec![arc(0.5, 0.85, 0.45, 0.7, 180.0, 360.0)]),
                item("Trace the smile! Follow the dots down and up.", "smile", vec![arc(0.5, 0.15, 0.45, 0.7, 180.0, 0.0)]),
                item("Trace the wave! Follow the dots.", "wave", vec![wave]),
                item(
                    "Trace the zigzag! Follow the dots.",
                    "zigzag",
                    vec![poly(&[(0.05, 0.8), (0.275, 0.2), (0.5, 0.8), (0.725, 0.2), (0.95, 0.8)])],
                ),
            ]
        }
        _ => vec![
            item("Trace the circle! Start at the number one and go round.", "circle", vec![arc(0.5, 0.5, 0.45, 0.45, -90.0, -450.0)]),
            item(
                "Trace the square! Start at the number one and go round.",
                "square",
                vec![poly(&[(0.1, 0.1), (0.9, 0.1), (0.9, 0.9), (0.1, 0.9), (0.1, 0.1)])],
            ),
            item(
                "Trace the triangle! Start at the number one and go round.",
                "triangle",
                vec![poly(&[(0.5, 0.08), (0.1, 0.9), (0.9, 0.9), (0.5, 0.08)])],
            ),
            item(
                "Trace the plus! Do the number one line, then number two.",
                "plus",
                vec![line(0.5, 0.05, 0.5, 0.95), line(0.05, 0.5, 0.95, 0.5)],
            ),
            item(
                "Trace the cross! Do the number one line, then number two.",
                "cross",
                vec![line(0.1, 0.1, 0.9, 0.9), line(0.9, 0.1, 0.1, 0.9)],
            ),
        ],
    }
}

// ---------- letters ----------

/// A capital letter, drawn the way a teacher would show it: tops first, left to right,
/// down before across. Returns None for anything that isn't A-Z.
pub fn letter(c: char) -> Option<Glyph> {
    use Seg::*;
    let strokes = match c.to_ascii_uppercase() {
        'A' => vec![line(0.5, 0.0, 0.05, 1.0), line(0.5, 0.0, 0.95, 1.0), line(0.21, 0.65, 0.79, 0.65)],
        'B' => vec![
            line(0.1, 0.0, 0.1, 1.0),
            stroke(0.1, 0.0, &[To(0.5, 0.0), Arc(0.5, 0.25, 0.3, 0.25, -90.0, 90.0), To(0.1, 0.5)]),
            stroke(0.1, 0.5, &[To(0.55, 0.5), Arc(0.55, 0.75, 0.35, 0.25, -90.0, 90.0), To(0.1, 1.0)]),
        ],
        'C' => vec![arc(0.5, 0.5, 0.5, 0.5, -45.0, -315.0)],
        'D' => vec![line(0.1, 0.0, 0.1, 1.0), stroke(0.1, 0.0, &[To(0.4, 0.0), Arc(0.4, 0.5, 0.5, 0.5, -90.0, 90.0), To(0.1, 1.0)])],
        'E' => vec![line(0.1, 0.0, 0.1, 1.0), line(0.1, 0.0, 0.9, 0.0), line(0.1, 0.5, 0.75, 0.5), line(0.1, 1.0, 0.9, 1.0)],
        'F' => vec![line(0.1, 0.0, 0.1, 1.0), line(0.1, 0.0, 0.9, 0.0), line(0.1, 0.5, 0.75, 0.5)],
        'G' => vec![stroke(0.8536, 0.1464, &[Arc(0.5, 0.5, 0.5, 0.5, -45.0, -360.0), To(0.55, 0.5)])],
        'H' => vec![line(0.1, 0.0, 0.1, 1.0), line(0.9, 0.0, 0.9, 1.0), line(0.1, 0.5, 0.9, 0.5)],
        'I' => vec![line(0.5, 0.0, 0.5, 1.0), line(0.25, 0.0, 0.75, 0.0), line(0.25, 1.0, 0.75, 1.0)],
        'J' => vec![stroke(0.7, 0.0, &[To(0.7, 0.7), Arc(0.4, 0.7, 0.3, 0.3, 0.0, 180.0)])],
        'K' => vec![line(0.1, 0.0, 0.1, 1.0), line(0.9, 0.0, 0.1, 0.6), line(0.35, 0.41, 0.95, 1.0)],
        'L' => vec![line(0.1, 0.0, 0.1, 1.0), line(0.1, 1.0, 0.85, 1.0)],
        'M' => vec![line(0.1, 1.0, 0.1, 0.0), poly(&[(0.1, 0.0), (0.5, 0.65), (0.9, 0.0)]), line(0.9, 0.0, 0.9, 1.0)],
        'N' => vec![line(0.1, 1.0, 0.1, 0.0), line(0.1, 0.0, 0.9, 1.0), line(0.9, 1.0, 0.9, 0.0)],
        'O' => vec![arc(0.5, 0.5, 0.5, 0.5, -90.0, -450.0)],
        'P' => vec![line(0.1, 0.0, 0.1, 1.0), stroke(0.1, 0.0, &[To(0.5, 0.0), Arc(0.5, 0.27, 0.35, 0.27, -90.0, 90.0), To(0.1, 0.54)])],
        'Q' => vec![arc(0.5, 0.45, 0.5, 0.45, -90.0, -450.0), line(0.6, 0.7, 0.95, 1.0)],
        'R' => vec![
            line(0.1, 0.0, 0.1, 1.0),
            stroke(0.1, 0.0, &[To(0.5, 0.0), Arc(0.5, 0.27, 0.35, 0.27, -90.0, 90.0), To(0.1, 0.54)]),
            line(0.45, 0.54, 0.9, 1.0),
        ],
        'S' => vec![stroke(0.8464, 0.125, &[Arc(0.5, 0.25, 0.4, 0.25, -30.0, -270.0), Arc(0.5, 0.75, 0.4, 0.25, -90.0, 150.0)])],
        'T' => vec![line(0.05, 0.0, 0.95, 0.0), line(0.5, 0.0, 0.5, 1.0)],
        'U' => vec![stroke(0.1, 0.0, &[To(0.1, 0.6), Arc(0.5, 0.6, 0.4, 0.4, 180.0, 0.0), To(0.9, 0.0)])],
        'V' => vec![poly(&[(0.05, 0.0), (0.5, 1.0), (0.95, 0.0)])],
        'W' => vec![poly(&[(0.0, 0.0), (0.25, 1.0), (0.5, 0.25), (0.75, 1.0), (1.0, 0.0)])],
        'X' => vec![line(0.1, 0.0, 0.9, 1.0), line(0.9, 0.0, 0.1, 1.0)],
        'Y' => vec![line(0.1, 0.0, 0.5, 0.5), poly(&[(0.9, 0.0), (0.5, 0.5), (0.5, 1.0)])],
        'Z' => vec![poly(&[(0.1, 0.0), (0.9, 0.0), (0.1, 1.0), (0.9, 1.0)])],
        _ => return None,
    };
    let wide = matches!(c.to_ascii_uppercase(), 'M' | 'W');
    let joined = match c.to_ascii_uppercase() {
        'N' => Some(poly(&[(0.1, 1.0), (0.1, 0.0), (0.9, 1.0), (0.9, 0.0)])),
        'M' => Some(poly(&[(0.1, 1.0), (0.1, 0.0), (0.5, 0.65), (0.9, 0.0), (0.9, 1.0)])),
        _ => None,
    };
    Some(Glyph { strokes, aspect: if wide { 0.9 } else { LETTER_ASPECT }, joined })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::length;

    fn check(glyph: &Glyph, what: &str) {
        assert!(!glyph.strokes.is_empty(), "{what} has no strokes");
        for (i, s) in glyph.strokes.iter().enumerate() {
            assert!(s.len() >= 2 && length(s) > 0.2, "{what} stroke {i} is too short");
            for p in s {
                assert!((-0.001..=1.001).contains(&p.x) && (-0.001..=1.001).contains(&p.y), "{what} stroke {i} leaves the box: {p}");
            }
            // No repeated points, which would make a zero-length piece.
            assert!(s.windows(2).all(|w| w[0].distance(w[1]) > 0.0001), "{what} stroke {i} repeats a point");
        }
    }

    #[test]
    fn every_letter_is_writable() {
        for c in 'A'..='Z' {
            check(&letter(c).expect("a glyph"), &c.to_string());
        }
        assert!(letter('?').is_none());
        assert!(letter('b').is_some(), "either case");
    }

    #[test]
    fn every_shape_is_writable() {
        for level in 1..=3 {
            let items = shapes(level);
            assert!(items.len() >= 4);
            for item in &items {
                check(&item.glyph, item.name);
            }
        }
    }

    #[test]
    fn curves_start_where_you_expect() {
        // A circle starts at the top; an S starts at the upper right.
        let o = letter('O').unwrap();
        assert!((o.strokes[0][0] - vec2(0.5, 0.0)).length() < 0.01);
        let s = letter('S').unwrap();
        assert!(s.strokes[0][0].x > 0.8 && s.strokes[0][0].y < 0.2);
        // Letters with a curve joined to a line are one unbroken stroke.
        assert_eq!(letter('U').unwrap().strokes.len(), 1);
        assert_eq!(letter('B').unwrap().strokes.len(), 3);
        let n = letter('N').unwrap();
        assert_eq!(n.strokes.len(), 3);
        check(&Glyph { strokes: vec![n.joined.unwrap()], aspect: 1.0, joined: None }, "one-stroke N");
    }
}
