// This file is part of PisiWorks.
// Copyright (C) 2026 Murat SAPMAZ
//
// PisiWorks is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// PisiWorks is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with PisiWorks. If
// not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turns a dense polyline back into a path of few Bezier curves.
//!
//! The path operations, the Knife and Freeform work on the straight pieces a curve
//! is drawn as. Fitting curves to what they return gives the user a curved path with
//! a handful of points again, instead of hundreds of tiny segments.
//!
//! The fitting is the method of Philip Schneider, "An Algorithm for Automatically
//! Fitting Digitized Curves" (Graphics Gems, 1990): least squares for the handle
//! lengths, a few rounds of moving each sample's parameter, and a split at the worst
//! point when that is not enough.

use crate::model::{v2, Handles, Vec2};

/// How far, in document units, a fitted curve may stray from the outline it replaces.
pub const TOLERANCE: f32 = 0.35;
/// A turn sharper than this, in degrees, is kept as a corner.
const CORNER_DEGREES: f32 = 28.0;
/// Rounds of reparameterization before a run is split.
const REFINE_ROUNDS: usize = 4;

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    v2(a.x - b.x, a.y - b.y)
}

fn add(a: Vec2, b: Vec2) -> Vec2 {
    v2(a.x + b.x, a.y + b.y)
}

fn scale(a: Vec2, k: f32) -> Vec2 {
    v2(a.x * k, a.y * k)
}

fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn length(a: Vec2) -> f32 {
    dot(a, a).sqrt()
}

fn unit(a: Vec2) -> Vec2 {
    let len = length(a);
    if len > 1e-6 { scale(a, 1.0 / len) } else { Vec2::default() }
}

fn bezier(c: &[Vec2; 4], t: f32) -> Vec2 {
    let u = 1.0 - t;
    let (a, b, d, e) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    v2(a * c[0].x + b * c[1].x + d * c[2].x + e * c[3].x, a * c[0].y + b * c[1].y + d * c[2].y + e * c[3].y)
}

/// Chord-length parameters of `points`, from 0 to 1.
fn chord_parameters(points: &[Vec2]) -> Vec<f32> {
    let mut u = vec![0.0; points.len()];
    for i in 1..points.len() {
        u[i] = u[i - 1] + length(sub(points[i], points[i - 1]));
    }
    let total = u[points.len() - 1];
    if total > 0.0 {
        u.iter_mut().for_each(|v| *v /= total);
    }
    u
}

/// The cubic through the ends of `points` with end tangents `t1` and `t2` that is
/// closest to the samples at parameters `u`.
fn least_squares(points: &[Vec2], u: &[f32], t1: Vec2, t2: Vec2) -> [Vec2; 4] {
    let (first, last) = (points[0], points[points.len() - 1]);
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (p, &t) in points.iter().zip(u) {
        let s = 1.0 - t;
        let (b0, b1, b2, b3) = (s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t);
        let (a0, a1) = (scale(t1, b1), scale(t2, b2));
        c00 += dot(a0, a0);
        c01 += dot(a0, a1);
        c11 += dot(a1, a1);
        let rest = sub(*p, add(scale(first, b0 + b1), scale(last, b2 + b3)));
        x0 += dot(a0, rest);
        x1 += dot(a1, rest);
    }
    let det = c00 * c11 - c01 * c01;
    let (mut left, mut right) = if det.abs() > 1e-9 { ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det) } else { (0.0, 0.0) };
    // Handles that point backward or are vanishingly short make loops; a third of the chord is the safe guess.
    let chord = length(sub(last, first));
    if left < chord * 1e-4 || right < chord * 1e-4 {
        left = chord / 3.0;
        right = chord / 3.0;
    }
    [first, add(first, scale(t1, left)), add(last, scale(t2, right)), last]
}

/// The square of the largest gap between the samples and the curve, and the sample it is at.
fn worst_gap(points: &[Vec2], u: &[f32], curve: &[Vec2; 4]) -> (f32, usize) {
    let (mut worst, mut at) = (0.0, points.len() / 2);
    for i in 1..points.len() - 1 {
        let gap = sub(bezier(curve, u[i]), points[i]);
        let squared = dot(gap, gap);
        if squared >= worst {
            (worst, at) = (squared, i);
        }
    }
    (worst, at)
}

/// Moves each parameter a Newton step toward the point of the curve nearest its sample.
fn reparameterize(points: &[Vec2], u: &mut [f32], curve: &[Vec2; 4]) {
    let d1: [Vec2; 3] = [scale(sub(curve[1], curve[0]), 3.0), scale(sub(curve[2], curve[1]), 3.0), scale(sub(curve[3], curve[2]), 3.0)];
    let d2: [Vec2; 2] = [scale(sub(d1[1], d1[0]), 2.0), scale(sub(d1[2], d1[1]), 2.0)];
    for (p, t) in points.iter().zip(u.iter_mut()) {
        let s = 1.0 - *t;
        let at = bezier(curve, *t);
        let first = add(add(scale(d1[0], s * s), scale(d1[1], 2.0 * s * *t)), scale(d1[2], *t * *t));
        let second = add(scale(d2[0], s), scale(d2[1], *t));
        let gap = sub(at, *p);
        let below = dot(first, first) + dot(gap, second);
        if below.abs() > 1e-9 {
            *t = (*t - dot(gap, first) / below).clamp(0.0, 1.0);
        }
    }
}

/// Fits one or more cubics to `points`, which run between two corners or split points.
fn fit_run(points: &[Vec2], t1: Vec2, t2: Vec2, tolerance: f32, out: &mut Vec<[Vec2; 4]>) {
    let (first, last) = (points[0], points[points.len() - 1]);
    if points.len() == 2 {
        let third = length(sub(last, first)) / 3.0;
        out.push([first, add(first, scale(t1, third)), add(last, scale(t2, third)), last]);
        return;
    }
    let limit = tolerance * tolerance;
    let mut u = chord_parameters(points);
    let mut curve = least_squares(points, &u, t1, t2);
    let (mut worst, mut split) = worst_gap(points, &u, &curve);
    // Close enough to be worth nudging the parameters before giving up on one curve.
    if worst >= limit && worst < limit * 16.0 {
        for _ in 0..REFINE_ROUNDS {
            reparameterize(points, &mut u, &curve);
            curve = least_squares(points, &u, t1, t2);
            (worst, split) = worst_gap(points, &u, &curve);
            if worst < limit {
                break;
            }
        }
    }
    if worst < limit {
        out.push(curve);
        return;
    }
    let split = split.clamp(1, points.len() - 2);
    let center = unit(sub(points[split - 1], points[split + 1]));
    fit_run(&points[..=split], t1, center, tolerance, out);
    fit_run(&points[split..], scale(center, -1.0), t2, tolerance, out);
}

/// Whether every point of the run lies within `tolerance` of the line between its ends.
fn is_straight(points: &[Vec2], tolerance: f32) -> bool {
    let (first, last) = (points[0], points[points.len() - 1]);
    let along = sub(last, first);
    let len = length(along);
    points.iter().all(|p| {
        let rel = sub(*p, first);
        let gap = if len > 1e-6 { (rel.x * along.y - rel.y * along.x).abs() / len } else { length(rel) };
        gap <= tolerance
    })
}

/// Rewrites a polyline as the vertices and handles of a path that follows it within
/// `tolerance`. Sharp turns stay corners, straight runs stay straight, and a vertex
/// that the outline passes through twice, as the seams that join holes do, is never
/// moved or smoothed.
pub fn fit_curves(points: &[Vec2], closed: bool, tolerance: f32) -> (Vec<Vec2>, Vec<Handles>) {
    let count = points.len();
    if count < 3 {
        return (points.to_vec(), Vec::new());
    }
    let limit = CORNER_DEGREES.to_radians().cos();
    let twice = |i: usize| points.iter().enumerate().any(|(j, p)| j != i && *p == points[i]);
    let is_corner = |i: usize| {
        if !closed && (i == 0 || i == count - 1) {
            return true;
        }
        let (before, after) = (points[(i + count - 1) % count], points[(i + 1) % count]);
        let (a, b) = (unit(sub(points[i], before)), unit(sub(after, points[i])));
        dot(a, b) < limit || twice(i)
    };
    let corners: Vec<usize> = (0..count).filter(|i| is_corner(*i)).collect();

    // The runs to fit: from each corner to the next, or once around a shape with no corner at all.
    let smooth_loop = corners.is_empty();
    let starts: Vec<usize> = if smooth_loop { vec![0] } else { corners };
    let runs = if closed { starts.len() } else { starts.len() - 1 };
    let mut out_points: Vec<Vec2> = Vec::new();
    let mut out_handles: Vec<Handles> = Vec::new();
    for run in 0..runs {
        let from = starts[run];
        let to = starts[(run + 1) % starts.len()];
        let span = if to > from { to - from } else { to + count - from };
        let samples: Vec<Vec2> = (0..=span).map(|k| points[(from + k) % count]).collect();
        if out_points.is_empty() {
            out_points.push(samples[0]);
            out_handles.push(Handles::default());
        }
        if is_straight(&samples, tolerance / 2.0) {
            out_points.push(samples[span]);
            out_handles.push(Handles::default());
            continue;
        }
        // Where a smooth loop is cut open to be fitted, both ends share one tangent so the join stays smooth.
        let (t1, t2) = if smooth_loop {
            let through = unit(sub(samples[1], samples[span - 1]));
            (through, scale(through, -1.0))
        } else {
            (unit(sub(samples[1], samples[0])), unit(sub(samples[span - 1], samples[span])))
        };
        let mut curves = Vec::new();
        fit_run(&samples, t1, t2, tolerance, &mut curves);
        for curve in curves {
            if let Some(last) = out_handles.last_mut() {
                last.after = sub(curve[1], curve[0]);
            }
            out_points.push(curve[3]);
            out_handles.push(Handles { before: sub(curve[2], curve[3]), after: Vec2::default() });
        }
    }
    // A closed path does not repeat its first vertex: the last one fitted is the first again.
    if closed && out_points.len() > 1 {
        let closing = out_handles.pop().unwrap_or_default();
        out_points.pop();
        if let Some(first) = out_handles.first_mut() {
            first.before = closing.before;
        }
    }
    if out_handles.iter().all(|h| h.is_corner()) {
        out_handles.clear();
    }
    (out_points, out_handles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::flatten_curve;

    /// The largest distance from any of `samples` to the polyline `path`.
    fn deviation(samples: &[Vec2], path: &[Vec2], closed: bool) -> f32 {
        let segments = if closed { path.len() } else { path.len() - 1 };
        samples
            .iter()
            .map(|p| {
                (0..segments)
                    .map(|i| {
                        let (a, b) = (path[i], path[(i + 1) % path.len()]);
                        let ab = sub(b, a);
                        let t = if dot(ab, ab) > 0.0 { (dot(sub(*p, a), ab) / dot(ab, ab)).clamp(0.0, 1.0) } else { 0.0 };
                        length(sub(*p, add(a, scale(ab, t))))
                    })
                    .fold(f32::MAX, f32::min)
            })
            .fold(0.0, f32::max)
    }

    fn circle(radius: f32, steps: usize) -> Vec<Vec2> {
        (0..steps).map(|i| std::f32::consts::TAU * i as f32 / steps as f32).map(|a| v2(100.0 + radius * a.cos(), 100.0 + radius * a.sin())).collect()
    }

    #[test]
    fn a_polygon_keeps_its_corners_and_gets_no_handles() {
        let square = vec![v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)];
        assert_eq!(fit_curves(&square, true, 0.35), (square.clone(), Vec::new()));
        let open = vec![v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0)];
        assert_eq!(fit_curves(&open, false, 0.35), (open.clone(), Vec::new()));
    }

    #[test]
    fn points_along_a_straight_side_are_dropped() {
        let side: Vec<Vec2> = (0..=20).map(|i| v2(i as f32 * 5.0, 0.0)).chain([v2(100.0, 40.0)]).collect();
        let (points, handles) = fit_curves(&side, false, 0.35);
        assert_eq!(points, vec![v2(0.0, 0.0), v2(100.0, 0.0), v2(100.0, 40.0)]);
        assert!(handles.is_empty());
    }

    #[test]
    fn a_circle_becomes_a_few_curves_that_stay_on_it() {
        let polygon = circle(50.0, 96);
        let (points, handles) = fit_curves(&polygon, true, 0.35);
        assert!((3..=8).contains(&points.len()), "{} points", points.len());
        assert_eq!(points.len(), handles.len());
        assert!(handles.iter().all(|h| !h.is_corner()), "a circle has no corners");
        let fitted = flatten_curve(&points, &handles, true);
        assert!(deviation(&polygon, &fitted, true) < 0.5, "off by {}", deviation(&polygon, &fitted, true));
        for p in fitted {
            let radius = length(sub(p, v2(100.0, 100.0)));
            assert!((radius - 50.0).abs() < 0.6, "radius was {radius}");
        }
    }

    #[test]
    fn a_half_disc_has_two_corners_one_straight_side_and_a_curved_one() {
        // Half of a circle, closed along its diameter.
        let arc: Vec<Vec2> = circle(50.0, 96)[..=48].to_vec();
        let (points, handles) = fit_curves(&arc, true, 0.35);
        assert_eq!(points[0], arc[0]);
        assert!(points.contains(&arc[48]), "the far end of the diameter is a vertex");
        let fitted = flatten_curve(&points, &handles, true);
        assert!(deviation(&arc, &fitted, true) < 0.5);
        // The diameter is one straight segment: the vertex before the first has no handle toward it.
        let far = points.iter().position(|p| *p == arc[48]).unwrap();
        assert_eq!(handles[far].after, Vec2::default());
        assert_eq!(handles[0].before, Vec2::default());
        assert_ne!(handles[0].after, Vec2::default());
    }

    #[test]
    fn seam_vertices_stay_exactly_where_they_are() {
        // A disc with a round hole, joined by a seam from one ring to the other and back.
        let outer = circle(50.0, 96);
        let hole: Vec<Vec2> = circle(20.0, 48).into_iter().rev().collect();
        let joined = crate::model::bridge(outer.clone(), vec![hole.clone()]);
        let (points, handles) = fit_curves(&joined, true, 0.35);
        let fitted = flatten_curve(&points, &handles, true);
        let rings = crate::model::contours(&fitted);
        assert_eq!(rings.len(), 2, "the seam is still walked both ways, so the hole is still a hole");
        assert!(points.len() < 30, "{} points", points.len());
        assert!(deviation(&outer, &fitted, true) < 0.5 && deviation(&hole, &fitted, true) < 0.5);
    }
}
