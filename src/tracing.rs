//! The math behind the Writing game: how close is a finger stroke to the line it should follow?
//!
//! Lines are `&[Vec2]`: a list of points joined by straight segments (a curve is just lots of
//! little segments). Everything here is plain geometry with no drawing, so it can be tested.

use macroquad::prelude::Vec2;

/// How far along a line something is, as a fraction from 0 (start) to 1 (end).
pub type Frac = f32;

/// Total length of a line.
pub fn length(poly: &[Vec2]) -> f32 {
    poly.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// The point `frac` of the way along the line.
pub fn point_at(poly: &[Vec2], frac: Frac) -> Vec2 {
    let mut left = length(poly) * frac.clamp(0.0, 1.0);
    for w in poly.windows(2) {
        let seg = w[0].distance(w[1]);
        if left <= seg && seg > 0.0 {
            return w[0].lerp(w[1], left / seg);
        }
        left -= seg;
    }
    poly.last().copied().unwrap_or(Vec2::ZERO)
}

/// The part of the line between two fractions (`trim(p, 0.0, 0.5)` is the first half).
pub fn trim(poly: &[Vec2], from: Frac, to: Frac) -> Vec<Vec2> {
    let total = length(poly);
    let (from, to) = (from.clamp(0.0, 1.0) * total, to.clamp(0.0, 1.0) * total);
    let mut out = Vec::new();
    let mut at = 0.0;
    for w in poly.windows(2) {
        let seg = w[0].distance(w[1]);
        let (a, b) = (at, at + seg);
        at = b;
        if seg <= 0.0 || b < from || a > to {
            continue;
        }
        let p0 = w[0].lerp(w[1], ((from - a) / seg).clamp(0.0, 1.0));
        let p1 = w[0].lerp(w[1], ((to - a) / seg).clamp(0.0, 1.0));
        if out.is_empty() {
            out.push(p0);
        }
        out.push(p1);
    }
    out
}

/// Evenly spaced points along the line, `step` apart (always includes both ends).
pub fn resample(poly: &[Vec2], step: f32) -> Vec<Vec2> {
    let total = length(poly);
    if total <= 0.0 || step <= 0.0 {
        return poly.first().copied().into_iter().collect();
    }
    let n = (total / step).ceil().max(1.0) as usize;
    (0..=n).map(|i| point_at(poly, i as f32 / n as f32)).collect()
}

/// How far `p` is from the line, and how far along the line (in pixels) the nearest spot is.
pub fn project(poly: &[Vec2], p: Vec2) -> (f32, f32) {
    let mut best = (f32::MAX, 0.0);
    let mut before = 0.0;
    for w in poly.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = b - a;
        let len2 = seg.length_squared();
        let t = if len2 > 0.0 { ((p - a).dot(seg) / len2).clamp(0.0, 1.0) } else { 0.0 };
        let d = p.distance(a + seg * t);
        if d < best.0 {
            best = (d, before + seg.length() * t);
        }
        before += seg.length();
    }
    if poly.len() == 1 {
        best = (p.distance(poly[0]), 0.0);
    }
    best
}

/// Distance from `p` to the nearest of several lines.
pub fn distance_to_any(polys: &[Vec<Vec2>], p: Vec2) -> f32 {
    polys.iter().map(|l| project(l, p).0).fold(f32::MAX, f32::min)
}

/// How a try went. Holds enough detail to *show* what went wrong, not just that it did.
#[derive(Debug)]
pub struct Verdict {
    pub passed: bool,
    /// Fraction of the target line the finger came close to (1 = all of it).
    pub coverage: f32,
    /// Fraction of the finger's path that wandered too far from the line.
    pub stray: f32,
    /// Did the stroke begin near the start of the line?
    pub start_ok: bool,
    /// Was it drawn the right way round (matters for loops, which start where they end)?
    pub direction_ok: bool,
    /// Spots along the target that the finger never got near.
    pub missed: Vec<Vec2>,
}

/// At least this much of the line has to be covered...
const MIN_COVERAGE: f32 = 0.85;
/// ...and no more than this much of the finger's path may wander off it.
const MAX_STRAY_ONE: f32 = 0.20;
/// Freehand (no letter shown, so it's judged on overall shape) is much looser than tracing.
pub const MIN_COVERAGE_FREE: f32 = 0.7;
pub const MAX_STRAY_FREE: f32 = 0.4;
/// Freehand: every line of the letter must be at least this covered, so a short stroke
/// (the tail of a Q, the bar of an A) can't be skipped just because the rest is perfect.
const MIN_EACH_STROKE: f32 = 0.45;
/// The start can be this many tolerances away from the exact start point.
const START_SLACK: f32 = 1.8;

/// Fraction of `target` that `ink` came within `tol` of, plus the spots that were missed.
fn cover(targets: &[&[Vec2]], ink: &[Vec2], tol: f32) -> (f32, Vec<Vec2>) {
    let samples: Vec<Vec2> = targets.iter().flat_map(|t| resample(t, tol * 0.5)).collect();
    let missed: Vec<Vec2> = samples.iter().copied().filter(|s| !ink.iter().any(|p| p.distance(*s) <= tol)).collect();
    let coverage = 1.0 - missed.len() as f32 / samples.len().max(1) as f32;
    (coverage, missed)
}

/// Fraction of coverage only (used to tell whether a stroke matched a different line).
pub fn coverage(target: &[Vec2], ink: &[Vec2], tol: f32) -> f32 {
    cover(&[target], ink, tol).0
}

/// Judge one finger stroke against one line to trace. `tol` is how far off the line is OK.
pub fn judge_stroke(target: &[Vec2], ink: &[Vec2], tol: f32) -> Verdict {
    let (coverage, missed) = cover(&[target], ink, tol);
    let off = ink.iter().filter(|p| project(target, **p).0 > tol).count();
    let stray = off as f32 / ink.len().max(1) as f32;
    let start_ok = ink.first().is_some_and(|p| p.distance(target[0]) <= tol * START_SLACK);

    // A quarter of the way along the finger's path, we should be a bit of the way along the
    // line. (This tells a circle drawn backwards from a right one: both start in the same spot.)
    let ink_len = length(ink);
    let quarter = point_at(ink, 0.25);
    let along = project(target, quarter).1 / length(target).max(1.0);
    let direction_ok = ink_len <= 0.0 || (0.03..=0.6).contains(&along);

    Verdict {
        passed: coverage >= MIN_COVERAGE && stray <= MAX_STRAY_ONE && start_ok && direction_ok,
        coverage,
        stray,
        start_ok,
        direction_ok,
        missed,
    }
}

/// Judge a whole letter written freehand: any strokes, in any order, are fine as long as
/// together they follow all the lines and nothing else.
pub fn judge_free(targets: &[Vec<Vec2>], inks: &[Vec<Vec2>], tol: f32) -> Verdict {
    let ink: Vec<Vec2> = inks.iter().flatten().copied().collect();
    let target_refs: Vec<&[Vec2]> = targets.iter().map(|t| t.as_slice()).collect();
    let (coverage, missed) = cover(&target_refs, &ink, tol);
    let off = ink.iter().filter(|p| distance_to_any(targets, **p) > tol).count();
    let stray = off as f32 / ink.len().max(1) as f32;
    let every_stroke = target_refs.iter().all(|t| cover(&[t], &ink, tol).0 >= MIN_EACH_STROKE);
    Verdict {
        passed: coverage >= MIN_COVERAGE_FREE && every_stroke && stray <= MAX_STRAY_FREE,
        coverage,
        stray,
        start_ok: true,
        direction_ok: true,
        missed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec2;

    /// A finger path along `line`, wobbling sideways by `wobble` pixels, from `from` to `to`
    /// (fractions of the line), with a point about every 5 pixels.
    fn finger(line: &[Vec2], from: f32, to: f32, wobble: f32) -> Vec<Vec2> {
        let n = (length(line) * (to - from) / 5.0).ceil() as usize;
        (0..=n)
            .map(|i| {
                let f = from + (to - from) * i as f32 / n as f32;
                point_at(line, f) + vec2(0.0, wobble * (i as f32 * 0.7).sin())
            })
            .collect()
    }

    fn across() -> Vec<Vec2> {
        vec![vec2(100.0, 200.0), vec2(500.0, 200.0)]
    }

    fn circle() -> Vec<Vec2> {
        (0..=60)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 - i as f32 / 60.0 * std::f32::consts::TAU;
                vec2(300.0, 300.0) + vec2(a.cos(), a.sin()) * 150.0
            })
            .collect()
    }

    #[test]
    fn lengths_and_points() {
        let l = across();
        assert_eq!(length(&l), 400.0);
        assert_eq!(point_at(&l, 0.5), vec2(300.0, 200.0));
        assert_eq!(point_at(&l, 2.0), vec2(500.0, 200.0), "clamped to the end");
        let half = trim(&l, 0.0, 0.5);
        assert_eq!((half.first().copied(), half.last().copied()), (Some(vec2(100.0, 200.0)), Some(vec2(300.0, 200.0))));
        let bent = vec![vec2(0.0, 0.0), vec2(100.0, 0.0), vec2(100.0, 100.0)];
        assert!((length(&trim(&bent, 0.25, 0.75)) - 100.0).abs() < 0.01);
        let dots = resample(&l, 100.0);
        assert_eq!(dots.len(), 5);
    }

    #[test]
    fn projecting_onto_a_line() {
        let (d, along) = project(&across(), vec2(250.0, 230.0));
        assert!((d - 30.0).abs() < 0.01 && (along - 150.0).abs() < 0.01);
        // Beyond the end, the distance is to the end point.
        let (d, _) = project(&across(), vec2(600.0, 200.0));
        assert!((d - 100.0).abs() < 0.01);
    }

    #[test]
    fn a_good_trace_passes() {
        let v = judge_stroke(&across(), &finger(&across(), 0.0, 1.0, 8.0), 40.0);
        assert!(v.passed, "{v:?}");
    }

    #[test]
    fn stopping_early_misses_the_end() {
        let v = judge_stroke(&across(), &finger(&across(), 0.0, 0.6, 5.0), 40.0);
        assert!(!v.passed && v.coverage < 0.85 && v.stray < 0.05, "{v:?}");
        assert!(v.missed.iter().all(|m| m.x > 300.0), "the missed spots are at the far end");
    }

    #[test]
    fn drifting_away_is_stray() {
        // Wobbling 90 px off a line that allows 40.
        let v = judge_stroke(&across(), &finger(&across(), 0.0, 1.0, 90.0), 40.0);
        assert!(!v.passed && v.stray > 0.3, "{v:?}");
    }

    #[test]
    fn starting_in_the_wrong_place() {
        let v = judge_stroke(&across(), &finger(&across(), 0.5, 1.0, 0.0), 40.0);
        assert!(!v.start_ok && !v.passed);
        // Drawn from the far end back toward the start.
        let mut back = finger(&across(), 0.0, 1.0, 0.0);
        back.reverse();
        assert!(!judge_stroke(&across(), &back, 40.0).passed);
    }

    #[test]
    fn a_circle_drawn_backwards_fails() {
        let right = circle();
        assert!(judge_stroke(&right, &finger(&right, 0.0, 1.0, 5.0), 40.0).passed);
        // Same start point, but going the other way round.
        let wrong: Vec<Vec2> = finger(&right, 0.0, 1.0, 0.0).iter().enumerate().map(|(i, _)| {
            let n = finger(&right, 0.0, 1.0, 0.0).len();
            let f = if i == 0 { 0.0 } else { 1.0 - i as f32 / (n - 1) as f32 };
            point_at(&right, f)
        }).collect();
        let v = judge_stroke(&right, &wrong, 40.0);
        assert!(!v.direction_ok && !v.passed, "{v:?}");
    }

    #[test]
    fn freehand_cannot_skip_a_short_stroke() {
        // A big loop with a short tail, like Q. The loop alone is most of the total.
        let tail = vec![vec2(280.0, 480.0), vec2(360.0, 560.0)];
        let targets = vec![circle(), tail.clone()];
        let ring = finger(&circle(), 0.0, 1.0, 5.0);
        let v = judge_free(&targets, &[ring.clone()], 40.0);
        assert!(v.coverage >= MIN_COVERAGE_FREE && !v.passed, "{v:?}");
        assert!(judge_free(&targets, &[ring, finger(&tail, 0.0, 1.0, 3.0)], 40.0).passed);
    }

    #[test]
    fn freehand_accepts_an_n_in_one_stroke() {
        // Three lines of an N, written as one zigzag with a wobbly, rather loose hand.
        let n: Vec<Vec<Vec2>> = vec![
            vec![vec2(100.0, 500.0), vec2(100.0, 100.0)],
            vec![vec2(100.0, 100.0), vec2(400.0, 500.0)],
            vec![vec2(400.0, 500.0), vec2(400.0, 100.0)],
        ];
        let zigzag = vec![vec2(100.0, 500.0), vec2(100.0, 100.0), vec2(400.0, 500.0), vec2(400.0, 100.0)];
        let ink = finger(&zigzag, 0.0, 1.0, 25.0);
        assert!(judge_free(&n, &[ink], 60.0).passed);
    }

    #[test]
    fn freehand_needs_every_line() {
        let cross = vec![vec![vec2(300.0, 100.0), vec2(300.0, 500.0)], vec![vec2(100.0, 300.0), vec2(500.0, 300.0)]];
        let down = finger(&cross[0], 0.0, 1.0, 5.0);
        let over = finger(&cross[1], 0.0, 1.0, 5.0);
        assert!(!judge_free(&cross, &[down.clone()], 40.0).passed, "one line of two");
        assert!(judge_free(&cross, &[over.clone(), down.clone()], 40.0).passed, "any order");
        let scribble = vec![vec2(50.0, 50.0), vec2(550.0, 80.0), vec2(60.0, 560.0)];
        assert!(!judge_free(&cross, &[down, over, resample(&scribble, 5.0)], 40.0).passed, "extra scribbles");
    }
}
