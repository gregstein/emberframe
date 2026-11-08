//! Parametric shapes. Each keeps its settings and rebuilds its outline from its bounds.

use std::f32::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::model::{bridge, v2, Rect, Vec2};

/// Segments used per full turn of a curve.
const CURVE_SEGMENTS: usize = 64;
const MAX_COUNT: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoKind {
    Arrow,
    Chamfer,
    Star,
    Polygon,
    Doughnut,
    Pie,
    Bubble,
    Gear,
    Frame,
    Cross,
    LShape,
    Spiral,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoParams {
    pub a: f32,
    pub b: f32,
    pub count: u32,
}

/// What a parameter means for a given shape, with its allowed range.
#[derive(Clone, Copy, Debug)]
pub struct ParamInfo {
    pub label: &'static str,
    pub min: f32,
    pub max: f32,
}

const fn p(label: &'static str, min: f32, max: f32) -> Option<ParamInfo> {
    Some(ParamInfo { label, min, max })
}

impl AutoKind {
    pub const ALL: [AutoKind; 12] = [
        AutoKind::Arrow,
        AutoKind::Chamfer,
        AutoKind::Star,
        AutoKind::Polygon,
        AutoKind::Doughnut,
        AutoKind::Pie,
        AutoKind::Bubble,
        AutoKind::Gear,
        AutoKind::Frame,
        AutoKind::Cross,
        AutoKind::LShape,
        AutoKind::Spiral,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AutoKind::Arrow => "Arrow",
            AutoKind::Chamfer => "Chamfer Rectangle",
            AutoKind::Star => "Star",
            AutoKind::Polygon => "Polygon",
            AutoKind::Doughnut => "Doughnut",
            AutoKind::Pie => "Pie",
            AutoKind::Bubble => "Speech Bubble",
            AutoKind::Gear => "Gear",
            AutoKind::Frame => "Frame",
            AutoKind::Cross => "Cross",
            AutoKind::LShape => "L-Shape",
            AutoKind::Spiral => "Spiral",
        }
    }

    pub fn defaults(self) -> AutoParams {
        let (a, b, count) = match self {
            AutoKind::Arrow => (0.4, 0.5, 0),
            AutoKind::Chamfer => (0.2, 0.0, 0),
            AutoKind::Star => (0.45, 0.0, 5),
            AutoKind::Polygon => (0.0, 0.0, 6),
            AutoKind::Doughnut => (0.55, 0.0, 0),
            AutoKind::Pie => (270.0, -90.0, 0),
            AutoKind::Bubble => (0.15, 0.25, 0),
            AutoKind::Gear => (0.2, 0.0, 10),
            AutoKind::Frame => (0.15, 0.0, 0),
            AutoKind::Cross => (0.33, 0.0, 0),
            AutoKind::LShape => (0.35, 0.0, 0),
            AutoKind::Spiral => (0.0, 0.0, 4),
        };
        AutoParams { a, b, count }
    }

    /// Editable settings as `[a, b, count]`.
    pub fn params(self) -> [Option<ParamInfo>; 3] {
        match self {
            AutoKind::Arrow => [p("Head length", 0.05, 1.0), p("Shaft width", 0.05, 1.0), None],
            AutoKind::Chamfer => [p("Corner", 0.0, 0.5), None, None],
            AutoKind::Star => [p("Inner radius", 0.05, 1.0), None, p("Points", 3.0, 64.0)],
            AutoKind::Polygon => [None, None, p("Sides", 3.0, 64.0)],
            AutoKind::Doughnut => [p("Hole size", 0.05, 0.95), None, None],
            AutoKind::Pie => [p("Sweep", 1.0, 360.0), p("Start angle", -360.0, 360.0), None],
            AutoKind::Bubble => [p("Corner", 0.0, 0.5), p("Tail position", 0.1, 0.9), None],
            AutoKind::Gear => [p("Tooth depth", 0.05, 0.6), None, p("Teeth", 3.0, 64.0)],
            AutoKind::Frame => [p("Thickness", 0.02, 0.49), None, None],
            AutoKind::Cross | AutoKind::LShape => [p("Thickness", 0.05, 0.95), None, None],
            AutoKind::Spiral => [None, None, p("Turns", 1.0, 32.0)],
        }
    }

    pub fn is_closed(self) -> bool {
        self != AutoKind::Spiral
    }
}

fn ring(b: Rect, scale: f32, reversed: bool) -> Vec<Vec2> {
    let c = b.center();
    let mut out: Vec<Vec2> = (0..CURVE_SEGMENTS)
        .map(|i| {
            let t = TAU * i as f32 / CURVE_SEGMENTS as f32;
            v2(c.x + b.w / 2.0 * scale * t.cos(), c.y + b.h / 2.0 * scale * t.sin())
        })
        .collect();
    if reversed {
        out.reverse();
    }
    out
}

/// Vertices on the bounds' inscribed ellipse, alternating between the two radii.
fn alternating(b: Rect, count: u32, inner: f32) -> Vec<Vec2> {
    let c = b.center();
    let steps = count * 2;
    (0..steps)
        .map(|i| {
            let t = TAU * i as f32 / steps as f32 - PI / 2.0;
            let r = if i % 2 == 0 { 1.0 } else { inner };
            v2(c.x + b.w / 2.0 * r * t.cos(), c.y + b.h / 2.0 * r * t.sin())
        })
        .collect()
}

fn gear(b: Rect, teeth: u32, depth: f32) -> Vec<Vec2> {
    let c = b.center();
    let inner = 1.0 - depth;
    let pitch = TAU / teeth as f32;
    let mut out = Vec::with_capacity(teeth as usize * 4);
    for i in 0..teeth {
        let start = i as f32 * pitch - PI / 2.0;
        for (fraction, r) in [(0.0, inner), (0.15, 1.0), (0.45, 1.0), (0.6, inner)] {
            let t = start + pitch * fraction;
            out.push(v2(c.x + b.w / 2.0 * r * t.cos(), c.y + b.h / 2.0 * r * t.sin()));
        }
    }
    out
}

fn spiral(b: Rect, turns: u32) -> Vec<Vec2> {
    let c = b.center();
    let steps = turns as usize * CURVE_SEGMENTS;
    (0..=steps)
        .map(|i| {
            let f = i as f32 / steps as f32;
            let t = f * turns as f32 * TAU;
            v2(c.x + b.w / 2.0 * f * t.cos(), c.y + b.h / 2.0 * f * t.sin())
        })
        .collect()
}

fn pie(b: Rect, sweep: f32, start: f32) -> Vec<Vec2> {
    let c = b.center();
    let steps = ((sweep / 360.0 * CURVE_SEGMENTS as f32).ceil() as usize).max(1);
    let mut out = if sweep >= 360.0 { Vec::new() } else { vec![c] };
    for i in 0..=steps {
        let t = (start + sweep * i as f32 / steps as f32).to_radians();
        out.push(v2(c.x + b.w / 2.0 * t.cos(), c.y + b.h / 2.0 * t.sin()));
    }
    out
}

fn bubble(b: Rect, corner: f32, tail: f32) -> Vec<Vec2> {
    let body = b.h * 0.78;
    let r = (corner * b.w.min(body)).min(b.w / 2.0).min(body / 2.0).max(0.0);
    let bottom = b.y + body;
    let tip_x = b.x + b.w * tail;
    let half_base = (b.w * 0.08).min((b.w - 2.0 * r) / 2.0).max(0.0);
    let base_x = tip_x.clamp(b.x + r + half_base, b.right() - r - half_base);
    let mut out = Vec::new();
    let mut arc = |cx: f32, cy: f32, from: f32| {
        for i in 0..=8 {
            let t = (from + 90.0 * i as f32 / 8.0).to_radians();
            out.push(v2(cx + r * t.cos(), cy + r * t.sin()));
        }
    };
    arc(b.right() - r, b.y + r, -90.0);
    arc(b.right() - r, bottom - r, 0.0);
    out.push(v2(base_x + half_base, bottom));
    out.push(v2(tip_x, b.bottom()));
    out.push(v2(base_x - half_base, bottom));
    let mut arc = |cx: f32, cy: f32, from: f32| {
        for i in 0..=8 {
            let t = (from + 90.0 * i as f32 / 8.0).to_radians();
            out.push(v2(cx + r * t.cos(), cy + r * t.sin()));
        }
    };
    arc(b.x + r, bottom - r, 90.0);
    arc(b.x + r, b.y + r, 180.0);
    out
}

/// Builds the outline. Shapes with a hole are one path joined by a seam, see [`bridge`].
pub fn outline(kind: AutoKind, params: AutoParams, b: Rect) -> Vec<Vec2> {
    let info = kind.params();
    let clamp = |value: f32, slot: usize| info[slot].map_or(value, |p| value.clamp(p.min, p.max));
    let a = clamp(params.a, 0);
    let second = clamp(params.b, 1);
    let count = params.count.clamp(info[2].map_or(0, |p| p.min as u32), MAX_COUNT);
    let (x0, y0, x1, y1) = (b.x, b.y, b.right(), b.bottom());
    let at = |fx: f32, fy: f32| v2(x0 + b.w * fx, y0 + b.h * fy);
    match kind {
        AutoKind::Arrow => {
            let neck = 1.0 - a;
            let (top, bottom) = (0.5 - second / 2.0, 0.5 + second / 2.0);
            vec![at(0.0, top), at(neck, top), at(neck, 0.0), at(1.0, 0.5), at(neck, 1.0), at(neck, bottom), at(0.0, bottom)]
        }
        AutoKind::Chamfer => {
            let c = a * b.w.min(b.h);
            vec![
                v2(x0 + c, y0),
                v2(x1 - c, y0),
                v2(x1, y0 + c),
                v2(x1, y1 - c),
                v2(x1 - c, y1),
                v2(x0 + c, y1),
                v2(x0, y1 - c),
                v2(x0, y0 + c),
            ]
        }
        AutoKind::Star => alternating(b, count, a),
        AutoKind::Polygon => {
            let c = b.center();
            (0..count)
                .map(|i| {
                    let t = TAU * i as f32 / count as f32 - PI / 2.0;
                    v2(c.x + b.w / 2.0 * t.cos(), c.y + b.h / 2.0 * t.sin())
                })
                .collect()
        }
        AutoKind::Doughnut => bridge(ring(b, 1.0, false), vec![ring(b, a, true)]),
        AutoKind::Pie => pie(b, a, second),
        AutoKind::Bubble => bubble(b, a, second),
        AutoKind::Gear => gear(b, count, a),
        AutoKind::Frame => {
            let t = a * b.w.min(b.h);
            let outer = vec![v2(x0, y0), v2(x1, y0), v2(x1, y1), v2(x0, y1)];
            let hole = vec![v2(x0 + t, y0 + t), v2(x0 + t, y1 - t), v2(x1 - t, y1 - t), v2(x1 - t, y0 + t)];
            bridge(outer, vec![hole])
        }
        AutoKind::Cross => {
            let (l, h) = (0.5 - a / 2.0, 0.5 + a / 2.0);
            vec![
                at(l, 0.0),
                at(h, 0.0),
                at(h, l),
                at(1.0, l),
                at(1.0, h),
                at(h, h),
                at(h, 1.0),
                at(l, 1.0),
                at(l, h),
                at(0.0, h),
                at(0.0, l),
                at(l, l),
            ]
        }
        AutoKind::LShape => vec![at(0.0, 0.0), at(a, 0.0), at(a, 1.0 - a), at(1.0, 1.0 - a), at(1.0, 1.0), at(0.0, 1.0)],
        AutoKind::Spiral => spiral(b, count),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::contours;

    const BOX: Rect = Rect { x: 10.0, y: 20.0, w: 100.0, h: 80.0 };

    #[test]
    fn every_shape_stays_inside_its_bounds() {
        for kind in AutoKind::ALL {
            let points = outline(kind, kind.defaults(), BOX);
            assert!(points.len() >= 3, "{kind:?}");
            for p in points {
                assert!(p.x >= 9.99 && p.x <= 110.01 && p.y >= 19.99 && p.y <= 100.01, "{kind:?} {p:?}");
            }
        }
    }

    #[test]
    fn star_has_two_vertices_per_point() {
        let params = AutoParams { a: 0.5, b: 0.0, count: 7 };
        assert_eq!(outline(AutoKind::Star, params, BOX).len(), 14);
    }

    #[test]
    fn doughnut_and_frame_separate_into_two_rings() {
        for kind in [AutoKind::Doughnut, AutoKind::Frame] {
            assert_eq!(contours(&outline(kind, kind.defaults(), BOX)).len(), 2, "{kind:?}");
        }
    }

    #[test]
    fn out_of_range_settings_are_clamped() {
        let params = AutoParams { a: 99.0, b: -5.0, count: 1 };
        assert_eq!(outline(AutoKind::Polygon, params, BOX).len(), 3);
        assert!(!outline(AutoKind::Star, AutoParams { a: 0.0, b: 0.0, count: 9999 }, BOX).is_empty());
    }

    #[test]
    fn spiral_is_the_only_open_shape() {
        assert_eq!(AutoKind::ALL.iter().filter(|k| !k.is_closed()).count(), 1);
    }
}
