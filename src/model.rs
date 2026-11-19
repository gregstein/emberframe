// This file is part of PisiWorks.
// Copyright (C) 2026 Murat SAPMAZ
// Portions copyright (c) 2026 Med Amine Belbachir, from Emberframe, under the MIT license:
// see LICENSES/MIT.txt and NOTICE.
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

//! Document model and editing operations. Free of UI types so it can be unit tested.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::blend::BlendMode;
use crate::i18n::{t, tc, tf};
use crate::autoshape::{self, AutoKind, AutoParams};
use crate::bitmap::{Bitmap, Filter, Mask};
use crate::optimize::ExportSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Parses `#RGB` or `#RRGGBB`, with or without the leading `#`.
    pub fn parse_hex(text: &str) -> Option<Self> {
        let t = text.trim().trim_start_matches('#');
        if !t.is_ascii() {
            return None;
        }
        let digit = |s: &str| u8::from_str_radix(s, 16).ok();
        match t.len() {
            3 => {
                let c = |i: usize| digit(&t[i..i + 1]).map(|v| v * 17);
                Some(Self::rgb(c(0)?, c(1)?, c(2)?))
            }
            6 => Some(Self::rgb(digit(&t[0..2])?, digit(&t[2..4])?, digit(&t[4..6])?)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub const fn v2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

/// The two Bezier control handles of a path vertex, as offsets from the vertex.
/// A zero handle leaves that side of the vertex straight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Handles {
    /// Shapes the segment that arrives at the vertex.
    pub before: Vec2,
    /// Shapes the segment that leaves it.
    pub after: Vec2,
}

impl Handles {
    /// A smooth vertex: both handles lie on one line, the same distance out.
    pub fn smooth(after: Vec2) -> Self {
        Self { before: v2(-after.x, -after.y), after }
    }

    pub fn is_corner(self) -> bool {
        self == Self::default()
    }

    fn map(self, f: impl Fn(Vec2) -> Vec2) -> Self {
        Self { before: f(self.before), after: f(self.after) }
    }
}

/// A projective map of the plane. Turning, slanting and pulling one corner of a box are
/// all cases of it, so one type carries every free transform.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warp {
    /// A 3x3 matrix, row by row.
    m: [f32; 9],
}

/// How a text or symbol object that has been freely transformed is drawn. The object
/// itself is unchanged, so text stays text and a symbol stays linked to its library entry.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    /// The box the object is laid out in, as it was before any transform.
    pub frame: Rect,
    /// Carries that box, and everything drawn in it, to where it shows in the document.
    pub warp: Warp,
}

impl Placement {
    /// The upright box around the transformed frame, which is the object's `bounds`.
    pub fn bounds(&self) -> Rect {
        let f = self.frame;
        Rect::of_points(&[v2(f.x, f.y), v2(f.right(), f.y), v2(f.right(), f.bottom()), v2(f.x, f.bottom())].map(|p| self.warp.apply(p)))
    }
}

impl Warp {
    pub fn identity() -> Self {
        Self { m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] }
    }

    pub fn translation(dx: f32, dy: f32) -> Self {
        Self { m: [1.0, 0.0, dx, 0.0, 1.0, dy, 0.0, 0.0, 1.0] }
    }

    /// Moves and stretches the rectangle `from` onto `to`. A side of no length is left unscaled.
    pub fn fit(from: Rect, to: Rect) -> Self {
        let sx = if from.w > 0.0 { to.w / from.w } else { 1.0 };
        let sy = if from.h > 0.0 { to.h / from.h } else { 1.0 };
        Self { m: [sx, 0.0, to.x - from.x * sx, 0.0, sy, to.y - from.y * sy, 0.0, 0.0, 1.0] }
    }

    /// An exact quarter turn around `center`, with none of the rounding a turn by an angle has.
    pub fn quarter_turn(center: Vec2, clockwise: bool) -> Self {
        let (cx, cy) = (center.x, center.y);
        if clockwise {
            Self { m: [0.0, -1.0, cx + cy, 1.0, 0.0, cy - cx, 0.0, 0.0, 1.0] }
        } else {
            Self { m: [0.0, 1.0, cx - cy, -1.0, 0.0, cy + cx, 0.0, 0.0, 1.0] }
        }
    }

    /// Mirrors left to right, or top to bottom, through `center`.
    pub fn mirror(center: Vec2, horizontal: bool) -> Self {
        if horizontal {
            Self { m: [-1.0, 0.0, 2.0 * center.x, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] }
        } else {
            Self { m: [1.0, 0.0, 0.0, 0.0, -1.0, 2.0 * center.y, 0.0, 0.0, 1.0] }
        }
    }

    /// Lays what stands on the level line `foot` down behind it, as a cast shadow lies:
    /// leaning over by `lean` for each pixel of height, and `squash` times as tall.
    pub fn cast(foot: f32, lean: f32, squash: f32) -> Self {
        Self { m: [1.0, -lean, lean * foot, 0.0, squash, foot * (1.0 - squash), 0.0, 0.0, 1.0] }
    }

    /// The map as the six numbers a browser's canvas takes for a transform. What the map does beyond turning, scaling and slanting is left out.
    pub fn affine(&self) -> [f32; 6] {
        [self.m[0], self.m[3], self.m[1], self.m[4], self.m[2], self.m[5]]
    }

    /// How far the map turns what it is given, in radians clockwise: the angle of a rotation, or of the turn in a freer transform.
    pub fn turn(&self) -> f32 {
        self.m[3].atan2(self.m[0])
    }

    /// This map followed by `next`.
    pub fn then(&self, next: &Warp) -> Self {
        let (a, b) = (&next.m, &self.m);
        let mut m = [0.0; 9];
        for row in 0..3 {
            for column in 0..3 {
                m[row * 3 + column] = (0..3).map(|k| a[row * 3 + k] * b[k * 3 + column]).sum();
            }
        }
        Self { m }
    }

    /// Turns around `center`. Positive angles turn clockwise on screen, where y grows downward.
    pub fn rotation(center: Vec2, radians: f32) -> Self {
        let (s, c) = radians.sin_cos();
        Self { m: [c, -s, center.x - c * center.x + s * center.y, s, c, center.y - s * center.x - c * center.y, 0.0, 0.0, 1.0] }
    }

    /// Slides points sideways by `slope` for every unit they are below the line y = `fixed_y`.
    pub fn shear_x(fixed_y: f32, slope: f32) -> Self {
        Self { m: [1.0, slope, -slope * fixed_y, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] }
    }

    /// Slides points up or down by `slope` for every unit they are right of the line x = `fixed_x`.
    pub fn shear_y(fixed_x: f32, slope: f32) -> Self {
        Self { m: [1.0, 0.0, 0.0, slope, 1.0, -slope * fixed_x, 0.0, 0.0, 1.0] }
    }

    /// Maps the rectangle `from` onto the quadrilateral `to`, whose corners run clockwise
    /// from the top-left. Straight lines stay straight. Returns `None` when `from` is empty
    /// or `to` is not convex, because the map would fold over itself.
    pub fn quad(from: Rect, to: [Vec2; 4]) -> Option<Self> {
        if from.w <= 0.0 || from.h <= 0.0 {
            return None;
        }
        let turn = |i: usize| {
            let (a, b, c) = (to[i], to[(i + 1) % 4], to[(i + 2) % 4]);
            (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
        };
        let turns = [turn(0), turn(1), turn(2), turn(3)];
        if !(turns.iter().all(|t| *t > 1e-3) || turns.iter().all(|t| *t < -1e-3)) {
            return None;
        }
        let [p0, p1, p2, p3] = to;
        let (dx1, dx2, sx) = (p1.x - p2.x, p3.x - p2.x, p0.x - p1.x + p2.x - p3.x);
        let (dy1, dy2, sy) = (p1.y - p2.y, p3.y - p2.y, p0.y - p1.y + p2.y - p3.y);
        let det = dx1 * dy2 - dx2 * dy1;
        if det.abs() < 1e-6 {
            return None;
        }
        let g = (sx * dy2 - dx2 * sy) / det;
        let h = (dx1 * sy - sx * dy1) / det;
        // The unit square onto the quadrilateral.
        let unit = [p1.x - p0.x + g * p1.x, p3.x - p0.x + h * p3.x, p0.x, p1.y - p0.y + g * p1.y, p3.y - p0.y + h * p3.y, p0.y, g, h, 1.0];
        // Then `from` onto the unit square first.
        let mut m = [0.0; 9];
        for row in 0..3 {
            let (a, b, c) = (unit[row * 3], unit[row * 3 + 1], unit[row * 3 + 2]);
            m[row * 3] = a / from.w;
            m[row * 3 + 1] = b / from.h;
            m[row * 3 + 2] = c - a * from.x / from.w - b * from.y / from.h;
        }
        Some(Self { m })
    }

    pub fn apply(&self, p: Vec2) -> Vec2 {
        let m = &self.m;
        let w = m[6] * p.x + m[7] * p.y + m[8];
        if w.abs() < 1e-6 {
            return p;
        }
        v2((m[0] * p.x + m[1] * p.y + m[2]) / w, (m[3] * p.x + m[4] * p.y + m[5]) / w)
    }

    /// The map that undoes this one, used to find where each pixel of a warped bitmap came from.
    pub fn inverse(&self) -> Option<Self> {
        let m = &self.m;
        let cofactor = |a: usize, b: usize, c: usize, d: usize| m[a] * m[d] - m[b] * m[c];
        let inv = [
            cofactor(4, 5, 7, 8),
            -cofactor(1, 2, 7, 8),
            cofactor(1, 2, 4, 5),
            -cofactor(3, 5, 6, 8),
            cofactor(0, 2, 6, 8),
            -cofactor(0, 2, 3, 5),
            cofactor(3, 4, 6, 7),
            -cofactor(0, 1, 6, 7),
            cofactor(0, 1, 3, 4),
        ];
        let det = m[0] * inv[0] + m[1] * inv[3] + m[2] * inv[6];
        (det.abs() > 1e-9).then(|| Self { m: inv.map(|v| v / det) })
    }
}

/// How far a handle reaches along each axis to draw a quarter of a circle of radius one.
const CIRCLE_HANDLE: f32 = 0.552_284_8;

/// Curve segments are cut into pieces about this long, in document units.
const CURVE_STEP: f32 = 3.0;
const CURVE_MIN_STEPS: usize = 8;
const CURVE_MAX_STEPS: usize = 64;

/// The point `t` of the way along a cubic Bezier curve.
fn cubic(controls: [Vec2; 4], t: f32) -> Vec2 {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    let [p0, p1, p2, p3] = controls;
    v2(a * p0.x + b * p1.x + c * p2.x + d * p3.x, a * p0.y + b * p1.y + c * p2.y + d * p3.y)
}

fn distance(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Turns vertices and their handles into the polyline that is drawn, filled and hit-tested.
/// Every vertex is kept, with the points of each curved segment after it.
pub fn flatten_curve(points: &[Vec2], handles: &[Handles], closed: bool) -> Vec<Vec2> {
    let count = points.len();
    if handles.iter().all(|h| h.is_corner()) || count < 2 {
        return points.to_vec();
    }
    let handle = |i: usize| handles.get(i).copied().unwrap_or_default();
    let segments = if closed { count } else { count - 1 };
    let mut out = Vec::with_capacity(count * CURVE_MIN_STEPS);
    for i in 0..segments {
        let j = (i + 1) % count;
        let (p0, p3) = (points[i], points[j]);
        let (out_handle, in_handle) = (handle(i).after, handle(j).before);
        out.push(p0);
        if out_handle == Vec2::default() && in_handle == Vec2::default() {
            continue;
        }
        let p1 = v2(p0.x + out_handle.x, p0.y + out_handle.y);
        let p2 = v2(p3.x + in_handle.x, p3.y + in_handle.y);
        let length = distance(p0, p1) + distance(p1, p2) + distance(p2, p3);
        let steps = ((length / CURVE_STEP).ceil() as usize).clamp(CURVE_MIN_STEPS, CURVE_MAX_STEPS);
        for step in 1..steps {
            out.push(cubic([p0, p1, p2, p3], step as f32 / steps as f32));
        }
    }
    if !closed {
        out.push(points[count - 1]);
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn from_corners(a: Vec2, b: Vec2) -> Self {
        Self {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            w: (a.x - b.x).abs(),
            h: (a.y - b.y).abs(),
        }
    }

    pub fn right(self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(self) -> f32 {
        self.y + self.h
    }

    pub fn center(self) -> Vec2 {
        v2(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn contains(self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }

    pub fn intersects(self, o: Rect) -> bool {
        self.x <= o.right() && o.x <= self.right() && self.y <= o.bottom() && o.y <= self.bottom()
    }

    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let (x0, y0) = (self.x.max(other.x), self.y.max(other.y));
        let (x1, y1) = (self.right().min(other.right()), self.bottom().min(other.bottom()));
        (x1 > x0 && y1 > y0).then_some(Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 })
    }

    pub fn union(self, o: Rect) -> Rect {
        let x = self.x.min(o.x);
        let y = self.y.min(o.y);
        Rect {
            x,
            y,
            w: self.right().max(o.right()) - x,
            h: self.bottom().max(o.bottom()) - y,
        }
    }

    pub fn of_points(points: &[Vec2]) -> Rect {
        let Some(first) = points.first() else {
            return Rect::default();
        };
        let (mut a, mut b) = (*first, *first);
        for p in points {
            a = v2(a.x.min(p.x), a.y.min(p.y));
            b = v2(b.x.max(p.x), b.y.max(p.y));
        }
        Rect::from_corners(a, b)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

/// How the edge of a fill is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillEdge {
    /// Every pixel is filled or not.
    Hard,
    #[default]
    AntiAlias,
    /// The edge fades out over a number of pixels.
    Feather,
}

impl FillEdge {
    pub const ALL: [FillEdge; 3] = [FillEdge::Hard, FillEdge::AntiAlias, FillEdge::Feather];

    pub fn label(self) -> &'static str {
        match self {
            FillEdge::Hard => t("Hard"),
            FillEdge::AntiAlias => t("Anti-Alias"),
            FillEdge::Feather => t("Feather"),
        }
    }
}

/// The edge of a fill and the texture laid into it. The default is an anti-aliased edge and no texture.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FillLook {
    #[serde(default)]
    pub edge: FillEdge,
    /// How many pixels a feathered edge fades over.
    #[serde(default = "feather_start")]
    pub feather: f32,
    /// Which texture, counted from 1; 0 is none.
    #[serde(default)]
    pub texture: u8,
    /// How strongly the texture shows, in percent.
    #[serde(default = "texture_start")]
    pub texture_amount: f32,
}

fn feather_start() -> f32 {
    10.0
}

fn texture_start() -> f32 {
    50.0
}

impl Default for FillLook {
    fn default() -> Self {
        Self { edge: FillEdge::AntiAlias, feather: feather_start(), texture: 0, texture_amount: texture_start() }
    }
}

impl FillLook {
    /// Whether the fill is drawn the ordinary way: an anti-aliased edge and no texture.
    pub fn is_plain(&self) -> bool {
        self.edge == FillEdge::AntiAlias && (self.texture == 0 || self.texture_amount <= 0.0)
    }
}

/// What an object can be filled with besides one color: Fireworks' gradients, and a pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillKind {
    /// Two colors pixel by pixel in a checkerboard, which the eye mixes into one a browser may not have.
    WebDither,
    Linear,
    Radial,
    Ellipse,
    Rectangle,
    Cone,
    Contour,
    Satin,
    Starburst,
    Folds,
    Bars,
    Ripples,
    Waves,
    Pattern,
}

impl FillKind {
    /// The gradients in the order of Fireworks' fill list, then the pattern.
    pub const ALL: [FillKind; 14] = [
        FillKind::WebDither,
        FillKind::Linear,
        FillKind::Radial,
        FillKind::Ellipse,
        FillKind::Rectangle,
        FillKind::Cone,
        FillKind::Contour,
        FillKind::Satin,
        FillKind::Starburst,
        FillKind::Folds,
        FillKind::Bars,
        FillKind::Ripples,
        FillKind::Waves,
        FillKind::Pattern,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FillKind::WebDither => t("Web Dither"),
            FillKind::Linear => t("Linear"),
            FillKind::Radial => t("Radial"),
            FillKind::Ellipse => t("Ellipse"),
            FillKind::Rectangle => t("Rectangle"),
            FillKind::Cone => t("Cone"),
            FillKind::Contour => t("Contour"),
            FillKind::Satin => t("Satin"),
            FillKind::Starburst => t("Starburst"),
            FillKind::Folds => t("Folds"),
            FillKind::Bars => t("Bars"),
            FillKind::Ripples => t("Ripples"),
            FillKind::Waves => t("Waves"),
            FillKind::Pattern => t("Pattern"),
        }
    }

    /// Where the two handles of a new fill of this kind stand in the object's box,
    /// which runs from 0 to 1 each way: a linear one crosses it, the others start in its middle.
    pub fn start_handles(self) -> (Vec2, Vec2) {
        match self {
            FillKind::Linear | FillKind::Bars | FillKind::Waves | FillKind::Folds | FillKind::Satin => (v2(0.0, 0.5), v2(1.0, 0.5)),
            _ => (v2(0.5, 0.5), v2(1.0, 0.5)),
        }
    }
}

/// A gradient or pattern fill.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
    pub kind: FillKind,
    /// The colors of a gradient: where each is, from 0 to 1, and the color, whose alpha is how opaque the fill is there.
    #[serde(default)]
    pub stops: Vec<(f32, Color)>,
    /// The two handles, in the object's box from 0 to 1 each way: where the gradient starts, and where it ends.
    pub start: Vec2,
    pub end: Vec2,
    /// Which pattern, and how large it is drawn, in percent.
    #[serde(default)]
    pub pattern: u8,
    #[serde(default = "full_scale")]
    pub scale: f32,
    /// The opacity stops of the gradient: only the alpha of each counts. None means opaque throughout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opacity_stops: Vec<(f32, Color)>,
    /// How wide an Ellipse or Rectangle gradient is across the line between its handles, as a part of that line's length; none is the usual width of the kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
}

fn full_scale() -> f32 {
    100.0
}

impl FillStyle {
    /// A fill of a kind running between two colors, with its handles where they start out.
    pub fn new(kind: FillKind, from: Color, to: Color) -> Self {
        let (start, end) = kind.start_handles();
        Self { kind, stops: vec![(0.0, from), (1.0, to)], start, end, pattern: 0, scale: 100.0, opacity_stops: Vec::new(), width: None }
    }

    /// The handles on the page, for an object with this box.
    pub fn handles(&self, bounds: Rect) -> (Vec2, Vec2) {
        let place = |p: Vec2| v2(bounds.x + p.x * bounds.w, bounds.y + p.y * bounds.h);
        (place(self.start), place(self.end))
    }

    /// How wide the gradient is across its line, as a part of the line's length. Only Ellipse and Rectangle have a width.
    pub fn across(&self) -> Option<f32> {
        match self.kind {
            FillKind::Ellipse => Some(self.width.unwrap_or(0.5).max(0.02)),
            FillKind::Rectangle => Some(self.width.unwrap_or(1.0).max(0.02)),
            _ => None,
        }
    }

    /// The third handle of an Ellipse or Rectangle gradient on the page: square to its line at the start, as far out as it is wide.
    pub fn width_handle(&self, bounds: Rect) -> Option<Vec2> {
        let width = self.across()?;
        let (start, end) = self.handles(bounds);
        Some(v2(start.x - (end.y - start.y) * width, start.y + (end.x - start.x) * width))
    }

    /// Sets the width from where the third handle is dragged to.
    pub fn set_width_handle(&mut self, at: Vec2, bounds: Rect) {
        let (start, end) = self.handles(bounds);
        let length = (end.x - start.x).hypot(end.y - start.y).max(0.001);
        self.width = Some(((at.x - start.x).hypot(at.y - start.y) / length).clamp(0.02, 20.0));
    }

    /// Puts a handle at a place on the page.
    pub fn set_handle(&mut self, end: bool, at: Vec2, bounds: Rect) {
        let inside = v2((at.x - bounds.x) / bounds.w.max(0.001), (at.y - bounds.y) / bounds.h.max(0.001));
        if end { self.end = inside } else { self.start = inside }
    }

    /// How far along the gradient a point of the page is, from 0 to 1, for an
    /// object with this box. `depth` is how deep inside the object the point lies,
    /// from 0 at its edge to 1 at its deepest, which only a contour asks for.
    pub fn place(&self, p: Vec2, bounds: Rect, depth: f32) -> f32 {
        use std::f32::consts::TAU;
        let (start, end) = self.handles(bounds);
        let (dx, dy) = (end.x - start.x, end.y - start.y);
        let length = (dx * dx + dy * dy).sqrt().max(0.001);
        // Along the line between the handles, and across it, both in lengths of that line.
        let (px, py) = (p.x - start.x, p.y - start.y);
        let (along, across) = ((px * dx + py * dy) / (length * length), (-px * dy + py * dx) / (length * length));
        let turn = (across.atan2(along) / TAU).rem_euclid(1.0);
        let far = along.hypot(across);
        // Up to 1 and back down to 0, once for each whole number.
        let wave = |v: f32| 1.0 - (v.rem_euclid(1.0) * 2.0 - 1.0).abs();
        match self.kind {
            FillKind::Linear | FillKind::Pattern | FillKind::WebDither => along,
            FillKind::Radial => far,
            FillKind::Ellipse => along.hypot(across / self.across().unwrap_or(0.5)),
            FillKind::Rectangle => along.abs().max(across.abs() / self.across().unwrap_or(1.0)),
            FillKind::Cone => turn,
            FillKind::Contour => 1.0 - depth,
            FillKind::Starburst => wave(turn * 8.0),
            FillKind::Ripples => wave(far * 3.0),
            FillKind::Bars => wave(along * 3.0),
            FillKind::Waves => 0.5 + 0.5 * (along * TAU * 2.0 + (across * TAU).sin()).sin(),
            FillKind::Folds => wave(along * 2.0).powi(2),
            FillKind::Satin => 0.5 + 0.5 * ((along + across) * TAU).sin() * ((along - across) * TAU * 0.5).cos(),
        }
        .clamp(0.0, 1.0)
    }
}

/// Where the lines of a text stand in its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    /// Every line but the last is spread from edge to edge, by widening the spaces between its words.
    Justify,
    /// Every line is stretched from edge to edge, letters and all.
    Stretch,
}

/// How a text is set: its font and what is done to it. The default is plain text in the program's own font.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// The font family; empty for the program's own font.
    #[serde(default)]
    pub font: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub align: TextAlign,
    /// For text attached to a path: how its letters stand on the path, and how far along the path it starts.
    #[serde(default)]
    pub on_path: PathOrientation,
    #[serde(default)]
    pub path_offset: f32,
    /// Which way the text runs.
    #[serde(default)]
    pub orientation: TextOrientation,
}

/// Which way a text runs, as Fireworks' text orientation button offers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextOrientation {
    /// Across, from the left.
    #[default]
    Horizontal,
    /// Across, from the right: the first letter of a line is its right-most.
    HorizontalRtl,
    /// Down, in columns that follow one another to the right.
    VerticalLtr,
    /// Down, in columns that follow one another to the left.
    VerticalRtl,
}

impl TextOrientation {
    pub const ALL: [TextOrientation; 4] = [TextOrientation::Horizontal, TextOrientation::HorizontalRtl, TextOrientation::VerticalLtr, TextOrientation::VerticalRtl];

    pub fn label(self) -> &'static str {
        match self {
            TextOrientation::Horizontal => t("Horizontal left to right"),
            TextOrientation::HorizontalRtl => t("Horizontal right to left"),
            TextOrientation::VerticalLtr => t("Vertical left to right"),
            TextOrientation::VerticalRtl => t("Vertical right to left"),
        }
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, TextOrientation::VerticalLtr | TextOrientation::VerticalRtl)
    }
}

/// How the letters of text on a path stand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathOrientation {
    /// Each letter is turned to stand on the path.
    #[default]
    Rotate,
    /// Each letter stays upright.
    Vertical,
    /// Each letter leans up and down with the path, its strokes staying upright.
    SkewVertical,
    /// Each letter leans sideways with the path, its baseline staying level.
    SkewHorizontal,
}

impl PathOrientation {
    pub const ALL: [PathOrientation; 4] = [PathOrientation::Rotate, PathOrientation::Vertical, PathOrientation::SkewVertical, PathOrientation::SkewHorizontal];

    pub fn label(self) -> &'static str {
        match self {
            PathOrientation::Rotate => t("Rotate Around Path"),
            PathOrientation::Vertical => t("Vertical"),
            PathOrientation::SkewVertical => t("Skew Vertical"),
            PathOrientation::SkewHorizontal => t("Skew Horizontal"),
        }
    }
}

impl TextStyle {
    pub fn is_plain(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Rect { radius: f32 },
    Ellipse,
    Line,
    Path { closed: bool },
    Text {
        content: String,
        size: f32,
        #[serde(default, skip_serializing_if = "TextStyle::is_plain")]
        style: TextStyle,
    },
    /// Pixels are shared copy-on-write, so undo snapshots stay cheap.
    Image { bitmap: Arc<Bitmap> },
    /// A web area drawn over the artwork and left out of flattened images.
    Web {
        kind: WebKind,
        url: String,
        /// The outline of the area inside its box: the whole box, a circle, or a polygon.
        #[serde(default, skip_serializing_if = "WebForm::is_rectangle")]
        form: WebForm,
    },
    /// A parametric shape whose outline is rebuilt from its bounds.
    Auto { kind: AutoKind, a: f32, b: f32, count: u32 },
    /// A placed copy of a library symbol. `objects` is rebuilt from the library on load.
    Instance {
        symbol: u64,
        #[serde(skip)]
        objects: Arc<Vec<Object>>,
    },
    /// Objects that select and move as one. Positions are relative to their combined bounds.
    Group { objects: Arc<Vec<Object>> },
}

/// The outline of a hotspot or slice.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum WebForm {
    #[default]
    Rectangle,
    Circle,
    /// The corners, inside the object's box, which runs from 0 to 1 each way.
    Polygon(Vec<Vec2>),
}

impl WebForm {
    pub fn is_rectangle(&self) -> bool {
        *self == WebForm::Rectangle
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebKind {
    Hotspot,
    Slice,
}

/// How a mask decides what shows of the object it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskKind {
    /// Everything inside the outline of the mask shows.
    PathOutline,
    /// The lighter the mask is drawn, the more shows through it.
    Grayscale,
    /// The more opaque the mask is, the more shows through it.
    Alpha,
}

/// A mask on an object: another object, placed in the document, that hides part of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectMask {
    pub object: Object,
    pub kind: MaskKind,
    /// With a path outline, whether the mask's own fill and stroke are drawn too.
    #[serde(default)]
    pub show_fill_stroke: bool,
    /// A disabled mask is kept but hides nothing.
    #[serde(default = "enabled")]
    pub enabled: bool,
    /// A linked mask moves and transforms with its object. Unlinked, the object moves under it.
    #[serde(default = "enabled")]
    pub linked: bool,
}

fn enabled() -> bool {
    true
}

impl ObjectMask {
    /// A mask from `object`, working the way Fireworks picks for it: a bitmap by its
    /// grayscale appearance, a vector by its outline unless `appearance` asks otherwise.
    pub fn new(object: Object, appearance: bool) -> Self {
        let kind = if object.is_image() || appearance { MaskKind::Grayscale } else { MaskKind::PathOutline };
        Self { object, kind, show_fill_stroke: false, enabled: true, linked: true }
    }
}

/// How a stroke changes the pixels under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelStroke {
    Paint(Color),
    Erase,
    Blur,
    /// Copies pixels from this document-space offset.
    Clone(Vec2),
    Sharpen,
    /// Lightens: the exposure from 0 to 1, and the tones worked on most.
    Dodge(f32, crate::bitmap::ToneRange),
    Burn(f32, crate::bitmap::ToneRange),
    /// Smudges: the pressure from 0 to 1, and the color the brush starts loaded with, if any.
    Smudge(f32, Option<Color>),
    /// Replaces the first color, within the tolerance, with the second.
    Replace([u8; 4], Color, u8),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: u64,
    pub name: String,
    pub shape: Shape,
    pub bounds: Rect,
    /// Absolute vertices; only used by `Line` and `Path`.
    pub points: Vec<Vec2>,
    /// Curve handles, one per vertex. Empty when every segment is straight, which
    /// is also how files written before curves existed read.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handles: Vec<Handles>,
    /// Set on text and symbols that have been rotated, skewed or distorted. `bounds`
    /// is then the upright box around the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<Placement>,
    /// The point a motion tween turns the object around, as Flash's transformation point is: where it lies
    /// in the object's own frame (see `anchor`), so that it follows the object however it is moved or turned.
    /// None is the middle of the object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<Vec2>,
    /// The ready-made motions the object has been given in PisiAnimasyon: see [`crate::behavior`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub behaviors: Vec<crate::behavior::Behavior>,
    /// The name of the object this one is attached to in PisiAnimasyon: in the published film it goes wherever that one goes.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub attached: String,
    /// How the object's colors mix with the artwork under it.
    #[serde(default, skip_serializing_if = "BlendMode::is_normal")]
    pub blend: BlendMode,
    /// What hides part of the object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<Arc<ObjectMask>>,
    /// Set on the stand-in for a mask while the mask is being edited: the id of the
    /// object it masks. The stand-in is an ordinary object every tool can work on,
    /// it is not drawn, and it never reaches a file.
    #[serde(skip)]
    pub mask_of: Option<u64>,
    /// A gradient or pattern the object is filled with in place of its fill color.
    /// The fill color stays set, so everything that only asks whether there is a fill still works.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_style: Option<Arc<FillStyle>>,
    /// How heavy the stroke is at each point of a path, as a part of its width: 1 is
    /// the full width. Set by the Path Scrubber tools. It counts only while there
    /// is one number for each point and the path has no curve handles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stroke_profile: Vec<f32>,
    /// The edge and the texture of the fill.
    #[serde(default, skip_serializing_if = "FillLook::is_plain")]
    pub fill_look: FillLook,
    /// Where a click on a button goes once the page is exported; empty for no link.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub link: String,
    /// Live filters, applied in this order each time the object is drawn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<crate::effects::LiveFilter>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
    /// 0.0 to 1.0.
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
}

const ELLIPSE_SEGMENTS: usize = 96;
const CORNER_SEGMENTS: usize = 12;

impl Object {
    pub fn is_closed(&self) -> bool {
        match self.shape {
            Shape::Rect { .. } | Shape::Ellipse | Shape::Image { .. } | Shape::Web { .. } | Shape::Instance { .. } | Shape::Group { .. } => true,
            Shape::Path { closed } => closed,
            Shape::Auto { kind, .. } => kind.is_closed(),
            Shape::Line | Shape::Text { .. } => false,
        }
    }

    pub fn new_bitmap(id: u64, name: impl Into<String>, bitmap: Bitmap, x: f32, y: f32) -> Self {
        Self {
            id,
            name: name.into(),
            bounds: Rect { x, y, w: bitmap.width() as f32, h: bitmap.height() as f32 },
            shape: Shape::Image { bitmap: Arc::new(bitmap) },
            points: Vec::new(),
            handles: Vec::new(),
            placement: None,
            pivot: None,

            behaviors: Vec::new(),
            attached: String::new(),
            blend: crate::blend::BlendMode::Normal,
            mask: None,
            mask_of: None,
            fill_style: None,
            stroke_profile: Vec::new(),
            fill_look: Default::default(),
            link: String::new(),
            filters: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    pub fn new_web(id: u64, kind: WebKind, bounds: Rect) -> Self {
        Self {
            id,
            name: match kind {
                WebKind::Hotspot => t("Hotspot").into(),
                WebKind::Slice => t("Slice").into(),
            },
            bounds,
            shape: Shape::Web { kind, url: String::new(), form: WebForm::Rectangle },
            points: Vec::new(),
            handles: Vec::new(),
            placement: None,
            pivot: None,

            behaviors: Vec::new(),
            attached: String::new(),
            blend: crate::blend::BlendMode::Normal,
            mask: None,
            mask_of: None,
            fill_style: None,
            stroke_profile: Vec::new(),
            fill_look: Default::default(),
            link: String::new(),
            filters: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    /// A hotspot or slice with straight sides through the given corners, in the order they were clicked.
    pub fn new_web_polygon(id: u64, kind: WebKind, corners: &[Vec2]) -> Self {
        let bounds = Rect::of_points(corners);
        let inside = corners.iter().map(|p| v2((p.x - bounds.x) / bounds.w.max(0.001), (p.y - bounds.y) / bounds.h.max(0.001))).collect();
        let mut area = Self::new_web(id, kind, bounds);
        if let Shape::Web { form, .. } = &mut area.shape {
            *form = WebForm::Polygon(inside);
        }
        area
    }

    /// A round hotspot inside a box.
    pub fn new_web_circle(id: u64, bounds: Rect) -> Self {
        let mut area = Self::new_web(id, WebKind::Hotspot, bounds);
        if let Shape::Web { form, .. } = &mut area.shape {
            *form = WebForm::Circle;
        }
        area
    }

    /// The outline of a round or many-sided web area on the page. A rectangular one has none of its own.
    pub fn web_outline(&self) -> Option<Vec<Vec2>> {
        let Shape::Web { form, .. } = &self.shape else { return None };
        let b = self.bounds;
        match form {
            WebForm::Rectangle => None,
            // Always a circle, as HTML can only describe one: the largest that fits the box, in its middle.
            WebForm::Circle => {
                let (center, radius) = (b.center(), b.w.min(b.h) / 2.0);
                Some((0..48).map(|step| step as f32 / 48.0 * std::f32::consts::TAU).map(|turn| v2(center.x + radius * turn.cos(), center.y + radius * turn.sin())).collect())
            }
            WebForm::Polygon(corners) => Some(corners.iter().map(|p| v2(b.x + p.x * b.w, b.y + p.y * b.h)).collect()),
        }
    }

    /// The weight of the stroke at each point, if the path has one that is in step with its points.
    pub fn stroke_weights(&self) -> Option<&[f32]> {
        let usable = !self.stroke_profile.is_empty() && self.stroke_profile.len() == self.points.len() && self.handles.is_empty() && self.placement.is_none();
        (usable && matches!(self.shape, Shape::Path { .. } | Shape::Line)).then_some(&self.stroke_profile[..])
    }

    /// Readies a path for the Path Scrubber: it becomes many short straight pieces, each point with a weight of its own.
    pub fn prepare_scrub(&mut self) -> bool {
        if self.stroke.is_none() || !self.to_path() {
            return false;
        }
        if self.stroke_weights().is_none() {
            self.flatten();
            self.points = subdivide(&self.points, self.is_closed(), 3.0);
            self.stroke_profile = vec![1.0; self.points.len()];
        }
        true
    }

    /// The Path Scrubber: the stroke gets heavier, or with a negative amount lighter,
    /// at the points within `radius` of a place, most at the place itself.
    pub fn scrub(&mut self, at: Vec2, radius: f32, amount: f32) {
        if self.stroke_weights().is_none() {
            return;
        }
        for (point, weight) in self.points.iter().zip(&mut self.stroke_profile) {
            let far = (point.x - at.x).hypot(point.y - at.y);
            if far < radius {
                let part = 0.5 + 0.5 * (std::f32::consts::PI * far / radius).cos();
                *weight = (*weight + amount * part).clamp(0.0, 3.0);
            }
        }
    }

    /// Fireworks' 9-slice scaling: the object is resized from one box to another,
    /// but only its middle stretches. `guides` are the two upright and the two level
    /// cuts, as parts of the old box from 0 to 1; what lies outside them keeps its size.
    pub fn nine_slice(&mut self, from: Rect, to: Rect, guides: [f32; 4]) {
        let map = |p: Vec2| nine_slice_point(p, from, to, guides);
        if self.reshapable() && self.to_path() {
            // Handles are measured from their vertex, so each curve keeps its shape where its vertex goes.
            let moved: Vec<Vec2> = self.points.iter().map(|p| map(*p)).collect();
            self.points = moved;
            self.refit_to_points();
            return;
        }
        let (a, b) = (map(v2(self.bounds.x, self.bounds.y)), map(v2(self.bounds.right(), self.bounds.bottom())));
        let target = Rect::from_corners(a, b);
        match &self.shape {
            // A bitmap is redrawn pixel by pixel: each pixel of the new image is the one it came from.
            Shape::Image { bitmap } if self.placement.is_none() => {
                let (sx, sy) = self.pixel_scale().unwrap_or((1.0, 1.0));
                let (width, height) = ((target.w * sx).round().max(1.0) as u32, (target.h * sy).round().max(1.0) as u32);
                let old = self.bounds;
                let mut rgba = vec![0u8; width as usize * height as usize * 4];
                for (at, px) in rgba.chunks_exact_mut(4).enumerate() {
                    let on_page = v2(target.x + ((at as u32 % width) as f32 + 0.5) / sx, target.y + ((at as u32 / width) as f32 + 0.5) / sy);
                    let source = nine_slice_source(on_page, from, to, guides);
                    let (x, y) = (((source.x - old.x) * sx).floor(), ((source.y - old.y) * sy).floor());
                    if x >= 0.0 && y >= 0.0 {
                        if let Some(color) = bitmap.pixel(x as u32, y as u32) {
                            px.copy_from_slice(&color);
                        }
                    }
                }
                if let Some(redrawn) = Bitmap::from_rgba(width, height, rgba) {
                    self.shape = Shape::Image { bitmap: Arc::new(redrawn) };
                    self.bounds = target;
                }
            }
            // A group is scaled member by member, so each keeps its own corners.
            Shape::Group { .. } => {
                let own = self.opacity;
                let mut members = self.instance_parts();
                for member in &mut members {
                    // The members came out with the group's opacity on them; it stays the group's.
                    if own > 0.0 {
                        member.opacity /= own;
                    }
                    member.nine_slice(from, to, guides);
                }
                if let Some(bounds) = members.iter().map(|o| o.bounds).reduce(Rect::union) {
                    self.shape = Shape::Group { objects: Arc::new(members) };
                    self.placement = None;
                    self.bounds = bounds;
                }
            }
            // Text keeps its size and goes where its middle goes.
            Shape::Text { .. } => {
                let (center, moved) = (self.bounds.center(), map(self.bounds.center()));
                self.translate(moved.x - center.x, moved.y - center.y);
            }
            // What is left is moved and sized by where its own corners go.
            _ => self.set_bounds(target),
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(self.shape, Shape::Image { .. })
    }

    pub fn is_web(&self) -> bool {
        matches!(self.shape, Shape::Web { .. })
    }

    /// Whether fill and stroke settings apply to this object.
    pub fn takes_color(&self) -> bool {
        !self.is_image() && !self.is_web() && !self.is_instance()
    }

    /// Whether the object is made of other objects: a symbol copy or a group.
    pub fn is_instance(&self) -> bool {
        matches!(self.shape, Shape::Instance { .. } | Shape::Group { .. })
    }

    pub fn reshapable(&self) -> bool {
        matches!(self.shape, Shape::Rect { .. } | Shape::Ellipse | Shape::Line | Shape::Path { .. } | Shape::Auto { .. })
    }

    /// The objects of a placed symbol, fitted to this instance's bounds.
    pub fn instance_parts(&self) -> Vec<Object> {
        let (Shape::Instance { objects, .. } | Shape::Group { objects }) = &self.shape else { return Vec::new() };
        let Some(natural) = objects.iter().map(|o| o.bounds).reduce(Rect::union) else { return Vec::new() };
        // A transformed symbol is fitted to its untouched frame, then its parts are transformed.
        let frame = self.placement.map_or(self.bounds, |p| p.frame);
        let sx = if natural.w > 0.0 { frame.w / natural.w } else { 1.0 };
        let sy = if natural.h > 0.0 { frame.h / natural.h } else { 1.0 };
        objects
            .iter()
            .map(|part| {
                let mut part = part.clone();
                let b = part.bounds;
                // Text that carries a placement is scaled through it, not through its size.
                let plain_text = part.placement.is_none();
                part.set_bounds(Rect {
                    x: frame.x + (b.x - natural.x) * sx,
                    y: frame.y + (b.y - natural.y) * sy,
                    w: b.w * sx,
                    h: b.h * sy,
                });
                part.opacity *= self.opacity;
                if let Shape::Text { size, .. } = &mut part.shape {
                    if plain_text {
                        *size *= sy;
                    }
                }
                if let Some(stroke) = &mut part.stroke {
                    stroke.width *= (sx + sy) / 2.0;
                }
                if let Some(placement) = &self.placement {
                    part.warp(&placement.warp);
                }
                part
            })
            .collect()
    }

    /// Every drawable object inside this one, or just this object if it is not a symbol.
    pub fn leaves(&self) -> Vec<Object> {
        if self.is_instance() {
            self.instance_parts().iter().flat_map(Object::leaves).collect()
        } else {
            vec![self.clone()]
        }
    }

    /// The lines to stroke, leaving out the seams that join holes to their outer edge.
    pub fn stroke_runs(&self) -> Vec<(Vec<Vec2>, bool)> {
        let outline = self.outline();
        if !self.is_closed() {
            return vec![(outline, false)];
        }
        contours(&outline).into_iter().map(|ring| (ring, true)).collect()
    }

    /// Turns a shape into an editable path. Returns false for text, bitmaps and web areas.
    /// A rectangle or an ellipse becomes the few curve points that draw it, and a
    /// shape that was rotated, skewed or distorted keeps the look it had.
    pub fn to_path(&mut self) -> bool {
        if !self.reshapable() {
            return false;
        }
        if matches!(self.shape, Shape::Path { .. }) {
            return true;
        }
        // The path is built upright in the shape's frame, then carried to where the shape shows.
        let placement = self.placement.take();
        if let Some(placement) = &placement {
            self.bounds = placement.frame;
        }
        if !self.to_bezier_path() {
            let closed = self.is_closed();
            self.points = self.outline();
            self.handles.clear();
            self.shape = Shape::Path { closed };
        }
        if let Some(placement) = &placement {
            self.map_points(&placement.warp);
        }
        self.refit_to_points();
        true
    }

    /// The vertices Subselection and the Pen show for this object: its own if it is a
    /// path, and those it would have as one if it is a shape that can become a path.
    pub fn path_points(&self) -> Vec<Vec2> {
        if matches!(self.shape, Shape::Line | Shape::Path { .. }) {
            return self.points.clone();
        }
        let mut path = self.clone();
        if path.reshapable() && path.to_path() { path.points } else { Vec::new() }
    }

    /// Moves every vertex, and the handles on it, through `warp`.
    fn map_points(&mut self, warp: &Warp) {
        for index in 0..self.points.len() {
            let from = self.points[index];
            let to = warp.apply(from);
            if let Some(handles) = self.handles.get_mut(index) {
                // A handle moves with the point of the plane it sits on.
                *handles = handles.map(|offset| {
                    if offset == Vec2::default() {
                        return offset;
                    }
                    let end = warp.apply(v2(from.x + offset.x, from.y + offset.y));
                    v2(end.x - to.x, end.y - to.y)
                });
            }
            self.points[index] = to;
        }
    }

    /// Sizes `bounds` to what a transformed object shows: the box around a shape's
    /// outline, or around the frame of text and symbols.
    pub fn refit_placed(&mut self) {
        let Some(placement) = self.placement else { return };
        self.bounds = match self.shape {
            Shape::Text { .. } | Shape::Instance { .. } => placement.bounds(),
            _ => Rect::of_points(&self.outline()),
        };
    }

    /// Rewrites an upright rectangle or ellipse as the Bezier path that draws it.
    /// Returns false for every other shape.
    fn to_bezier_path(&mut self) -> bool {
        let b = self.bounds;
        let (points, handles) = match self.shape {
            Shape::Ellipse => {
                let (c, rx, ry) = (b.center(), b.w / 2.0, b.h / 2.0);
                let (kx, ky) = (rx * CIRCLE_HANDLE, ry * CIRCLE_HANDLE);
                (
                    vec![v2(c.x + rx, c.y), v2(c.x, c.y + ry), v2(c.x - rx, c.y), v2(c.x, c.y - ry)],
                    vec![
                        Handles { before: v2(0.0, -ky), after: v2(0.0, ky) },
                        Handles { before: v2(kx, 0.0), after: v2(-kx, 0.0) },
                        Handles { before: v2(0.0, ky), after: v2(0.0, -ky) },
                        Handles { before: v2(-kx, 0.0), after: v2(kx, 0.0) },
                    ],
                )
            }
            Shape::Rect { radius } if radius.min(b.w / 2.0).min(b.h / 2.0) > 0.0 => {
                let r = radius.min(b.w / 2.0).min(b.h / 2.0);
                let k = r * CIRCLE_HANDLE;
                let (x0, y0, x1, y1) = (b.x, b.y, b.right(), b.bottom());
                let zero = Vec2::default();
                (
                    vec![
                        v2(x0 + r, y0),
                        v2(x1 - r, y0),
                        v2(x1, y0 + r),
                        v2(x1, y1 - r),
                        v2(x1 - r, y1),
                        v2(x0 + r, y1),
                        v2(x0, y1 - r),
                        v2(x0, y0 + r),
                    ],
                    vec![
                        Handles { before: v2(-k, 0.0), after: zero },
                        Handles { before: zero, after: v2(k, 0.0) },
                        Handles { before: v2(0.0, -k), after: zero },
                        Handles { before: zero, after: v2(0.0, k) },
                        Handles { before: v2(k, 0.0), after: zero },
                        Handles { before: zero, after: v2(-k, 0.0) },
                        Handles { before: v2(0.0, k), after: zero },
                        Handles { before: zero, after: v2(0.0, -k) },
                    ],
                )
            }
            _ => return false,
        };
        self.points = points;
        self.handles = handles;
        self.shape = Shape::Path { closed: true };
        true
    }

    /// Replaces text with a group of paths, one for each character, so it can be
    /// reshaped like any other artwork. Returns false if the text draws nothing.
    pub fn text_to_paths(&mut self) -> bool {
        let Shape::Text { content, size, style } = &self.shape else { return false };
        let frame = self.placement.map_or(self.bounds, |p| p.frame);
        // Text attached to a path is laid out along it; its points are that path.
        let glyphs = match self.points.len() {
            0 | 1 => crate::export::glyph_outlines(content, *size, v2(frame.x, frame.y), style),
            _ => crate::export::glyph_outlines_on_path(content, *size, style, &self.points),
        };
        let heavy = crate::export::made_up_bold(style);
        let parts: Vec<Object> = glyphs
            .into_iter()
            .filter_map(|mut rings| {
                if rings.is_empty() {
                    return None;
                }
                let outer = rings.remove(0);
                let points = bridge(outer, rings);
                Some(Object {
                    id: 0,
                    name: t("Path").into(),
                    shape: Shape::Path { closed: true },
                    bounds: Rect::of_points(&points),
                    points,
                    handles: Vec::new(),
                    placement: None,
                    pivot: None,

                    behaviors: Vec::new(),
                    attached: String::new(),
                    blend: crate::blend::BlendMode::Normal,
                    mask: None,
                    mask_of: None,
                    fill_style: None,
                    stroke_profile: Vec::new(),
                    fill_look: Default::default(),
                    link: String::new(),
                    filters: Vec::new(),
                    fill: self.fill,
                    // A bold the font does not have is made by thickening each letter.
                    stroke: self.fill.filter(|_| heavy).map(|color| Stroke { color, width: (*size / 14.0).max(0.6) }),
                    opacity: 1.0,
                    visible: true,
                    locked: false,
                })
            })
            .collect();
        let Some(bounds) = parts.iter().map(|part| part.bounds).reduce(Rect::union) else { return false };
        self.bounds = bounds;
        self.shape = Shape::Group { objects: Arc::new(parts) };
        // The letters were laid out upright, so a transform the text carried is applied to them now.
        match self.placement.take() {
            Some(placement) => self.warp_own(&placement.warp),
            None => true,
        }
    }

    /// Transformed text as the paths that draw it, or `None` for text that is still upright.
    pub fn placed_text_paths(&self) -> Option<Object> {
        let Shape::Text { style, .. } = &self.shape else { return None };
        // Text that is turned, or set in a way a plain line of type cannot show, is drawn from its outlines.
        if self.placement.is_none() && !crate::export::needs_outlines(style) && self.points.len() < 2 {
            return None;
        }
        let mut paths = self.clone();
        paths.text_to_paths().then_some(paths)
    }

    /// Sizes a text object to its content, after the text or its size has changed.
    pub fn refit_text(&mut self) {
        let Shape::Text { content, size, style } = &self.shape else { return };
        // On a path, the box is the one around the letters where the path puts them.
        if self.points.len() >= 2 {
            let letters = crate::export::glyph_outlines_on_path(content, *size, style, &self.points);
            let all: Vec<Vec2> = letters.into_iter().flatten().flatten().collect();
            self.bounds = if all.is_empty() { Rect::of_points(&self.points) } else { Rect::of_points(&all) };
            return;
        }
        let (w, h) = crate::export::measure_styled(content, *size, style);
        match &mut self.placement {
            Some(placement) => {
                (placement.frame.w, placement.frame.h) = (w, h);
                self.bounds = placement.bounds();
            }
            None => (self.bounds.w, self.bounds.h) = (w, h),
        }
    }

    /// Undoes every rotation, skew and distortion of a text or symbol object,
    /// leaving it upright around the same center.
    pub fn remove_transform(&mut self) -> bool {
        let Some(placement) = self.placement.take() else { return false };
        let (center, frame) = (self.bounds.center(), placement.frame);
        self.bounds = Rect { x: center.x - frame.w / 2.0, y: center.y - frame.h / 2.0, ..frame };
        true
    }

    /// Whether a free transform can reshape this object. Web areas cannot take one.
    pub fn warpable(&self) -> bool {
        match &self.shape {
            Shape::Web { .. } => false,
            Shape::Group { objects } => objects.iter().all(Object::warpable),
            _ => true,
        }
    }

    /// Applies a free transform: a rotation, a skew or a distortion. Paths are
    /// reshaped and bitmaps are redrawn. Shapes, text and symbols only record the
    /// transform and stay what they are. Returns false, leaving the object as it
    /// was, when it cannot be transformed.
    pub fn warp(&mut self, warp: &Warp) -> bool {
        let done = self.warp_own(warp);
        if done {
            self.with_mask(|mask| {
                mask.warp(warp);
            });
        }
        done
    }

    /// Runs `change` on the object's mask, if it has one, so the mask follows the object.
    fn with_mask(&mut self, change: impl FnOnce(&mut Object)) {
        if let Some(mask) = self.mask.as_mut().filter(|mask| mask.linked) {
            change(&mut Arc::make_mut(mask).object);
        }
    }

    fn warp_own(&mut self, warp: &Warp) -> bool {
        if !self.warpable() {
            return false;
        }
        match &self.shape {
            // These only record the transform, so a rectangle keeps its roundness,
            // an auto shape its settings, text its words and a symbol its link.
            Shape::Text { .. } | Shape::Instance { .. } | Shape::Rect { .. } | Shape::Ellipse | Shape::Auto { .. } => {
                let mut placement = self.placement.unwrap_or(Placement { frame: self.bounds, warp: Warp::identity() });
                placement.warp = placement.warp.then(warp);
                self.placement = Some(placement);
                self.refit_placed();
                return true;
            }
            Shape::Group { .. } => {
                let mut parts = self.instance_parts();
                if !parts.iter_mut().all(|part| part.warp(warp)) {
                    return false;
                }
                self.bounds = parts.iter().map(|part| part.bounds).reduce(Rect::union).unwrap_or(self.bounds);
                self.opacity = 1.0;
                self.shape = Shape::Group { objects: Arc::new(parts) };
                return true;
            }
            Shape::Image { bitmap } => {
                let Some((bitmap, bounds)) = bitmap.warped(self.bounds, warp) else { return false };
                self.shape = Shape::Image { bitmap: Arc::new(bitmap) };
                self.bounds = bounds;
                return true;
            }
            _ => {}
        }
        if !self.to_path() {
            return false;
        }
        self.map_points(warp);
        self.refit_to_points();
        true
    }

    pub fn is_curved(&self) -> bool {
        self.handles.iter().any(|h| !h.is_corner())
    }

    /// The handles of vertex `index`, which are zero for a corner.
    pub fn handles_at(&self, index: usize) -> Handles {
        self.handles.get(index).copied().unwrap_or_default()
    }

    /// Sets the handles of one vertex, keeping `handles` empty while the path is all corners.
    pub fn set_handles(&mut self, index: usize, handles: Handles) {
        if index >= self.points.len() {
            return;
        }
        self.handles.resize(self.points.len(), Handles::default());
        self.handles[index] = handles;
        if !self.is_curved() {
            self.handles.clear();
        }
    }

    /// How many segments the path has: one more than its gaps when it is closed.
    fn segment_count(&self) -> usize {
        match self.points.len() {
            0 | 1 => 0,
            count if self.is_closed() => count,
            count => count - 1,
        }
    }

    /// The four Bezier control points of the segment that leaves vertex `index`.
    fn segment(&self, index: usize) -> [Vec2; 4] {
        let next = (index + 1) % self.points.len();
        let (from, to) = (self.points[index], self.points[next]);
        let (out_handle, in_handle) = (self.handles_at(index).after, self.handles_at(next).before);
        [from, v2(from.x + out_handle.x, from.y + out_handle.y), v2(to.x + in_handle.x, to.y + in_handle.y), to]
    }

    /// The segment of a path that passes within `reach` of `p`, and how far along it.
    pub fn nearest_segment(&self, p: Vec2, reach: f32) -> Option<(usize, f32)> {
        const STEPS: usize = 24;
        let mut best: Option<(f32, usize, f32)> = None;
        for index in 0..self.segment_count() {
            let controls = self.segment(index);
            // A straight segment is measured evenly along its length, as insert_point splits it.
            let straight = controls[0] == controls[1] && controls[2] == controls[3];
            let mut from = controls[0];
            for step in 1..=STEPS {
                let t = step as f32 / STEPS as f32;
                let to = if straight {
                    v2(controls[0].x + (controls[3].x - controls[0].x) * t, controls[0].y + (controls[3].y - controls[0].y) * t)
                } else {
                    cubic(controls, t)
                };
                // Nearest point of this short piece, so a long straight segment is not missed between samples.
                let (dx, dy) = (to.x - from.x, to.y - from.y);
                let length = dx * dx + dy * dy;
                let along = if length > 0.0 { (((p.x - from.x) * dx + (p.y - from.y) * dy) / length).clamp(0.0, 1.0) } else { 0.0 };
                let gap = distance(p, v2(from.x + dx * along, from.y + dy * along));
                if best.is_none_or(|(least, ..)| gap < least) {
                    best = Some((gap, index, (step as f32 - 1.0 + along) / STEPS as f32));
                }
                from = to;
            }
        }
        best.filter(|(gap, ..)| *gap <= reach).map(|(_, index, t)| (index, t.clamp(0.02, 0.98)))
    }

    /// Adds a vertex on segment `index`, `t` of the way along, without changing how
    /// the path looks. Returns the new vertex's index.
    pub fn insert_point(&mut self, index: usize, t: f32) -> usize {
        let count = self.points.len();
        let next = (index + 1) % count;
        let controls = self.segment(index);
        let lerp = |a: Vec2, b: Vec2| v2(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
        let straight = self.handles_at(index).after == Vec2::default() && self.handles_at(next).before == Vec2::default();
        // De Casteljau: the two halves of the curve are curves with these control points.
        let (a, b, c) = (lerp(controls[0], controls[1]), lerp(controls[1], controls[2]), lerp(controls[2], controls[3]));
        let (d, e) = (lerp(a, b), lerp(b, c));
        let at = if straight { lerp(controls[0], controls[3]) } else { lerp(d, e) };
        let mut handles = Handles::default();
        if !straight {
            self.handles.resize(count, Handles::default());
            self.handles[index].after = v2(a.x - controls[0].x, a.y - controls[0].y);
            self.handles[next].before = v2(c.x - controls[3].x, c.y - controls[3].y);
            handles = Handles { before: v2(d.x - at.x, d.y - at.y), after: v2(e.x - at.x, e.y - at.y) };
        }
        self.points.insert(index + 1, at);
        if !self.handles.is_empty() {
            self.handles.insert(index + 1, handles);
        }
        index + 1
    }

    /// Removes a vertex. Returns false when that would leave less than a line.
    pub fn remove_point(&mut self, index: usize) -> bool {
        if index >= self.points.len() || self.points.len() <= 2 {
            return false;
        }
        self.points.remove(index);
        if index < self.handles.len() {
            self.handles.remove(index);
        }
        if !self.is_curved() {
            self.handles.clear();
        }
        self.refit_to_points();
        true
    }

    /// Replaces curves with the straight pieces they are drawn as, for tools that
    /// work on plain vertices.
    pub fn flatten(&mut self) {
        if self.is_curved() {
            self.points = self.outline();
        }
        self.handles.clear();
    }

    fn distance_to(&self, p: Vec2) -> f32 {
        let outline = self.outline();
        let count = outline.len();
        let segments = if self.is_closed() { count } else { count.saturating_sub(1) };
        (0..segments).map(|i| dist_to_segment(p, outline[i], outline[(i + 1) % count])).fold(f32::MAX, f32::min)
    }

    pub fn bitmap(&self) -> Option<&Bitmap> {
        match &self.shape {
            Shape::Image { bitmap } => Some(bitmap),
            _ => None,
        }
    }

    /// Returns unique pixels to edit, copying them if a history entry shares them.
    pub fn bitmap_mut(&mut self) -> Option<&mut Bitmap> {
        match &mut self.shape {
            Shape::Image { bitmap } => Some(Arc::make_mut(bitmap)),
            _ => None,
        }
    }

    /// Maps a document point to the bitmap pixel under it.
    pub fn pixel_at(&self, p: Vec2) -> Option<(u32, u32)> {
        let bitmap = self.bitmap()?;
        let b = self.bounds;
        if !b.contains(p) || b.w <= 0.0 || b.h <= 0.0 {
            return None;
        }
        let x = ((p.x - b.x) * bitmap.width() as f32 / b.w) as u32;
        let y = ((p.y - b.y) * bitmap.height() as f32 / b.h) as u32;
        Some((x.min(bitmap.width() - 1), y.min(bitmap.height() - 1)))
    }

    /// Pixels per document unit on each axis.
    pub fn pixel_scale(&self) -> Option<(f32, f32)> {
        let bitmap = self.bitmap()?;
        let b = self.bounds;
        (b.w > 0.0 && b.h > 0.0).then(|| (bitmap.width() as f32 / b.w, bitmap.height() as f32 / b.h))
    }

    /// Maps a document point to bitmap coordinates, which may lie outside the image.
    pub fn doc_to_pixel(&self, p: Vec2) -> Option<Vec2> {
        let (sx, sy) = self.pixel_scale()?;
        Some(v2((p.x - self.bounds.x) * sx, (p.y - self.bounds.y) * sy))
    }

    pub fn pixel_to_doc(&self, p: Vec2) -> Option<Vec2> {
        let (sx, sy) = self.pixel_scale()?;
        Some(v2(self.bounds.x + p.x / sx, self.bounds.y + p.y / sy))
    }

    /// Runs a pixel edit, keeping anything outside `mask` unchanged.
    pub fn edit_pixels(&mut self, mask: Option<&Mask>, edit: impl FnOnce(&mut Bitmap)) {
        let before = mask.and_then(|_| self.bitmap().cloned());
        let Some(bitmap) = self.bitmap_mut() else { return };
        edit(bitmap);
        if let (Some(before), Some(mask)) = (before, mask) {
            bitmap.restore_outside(&before, mask);
        }
    }

    /// Applies a document-space stroke to the pixels.
    pub fn stroke_bitmap(&mut self, points: &[Vec2], width: f32, op: PixelStroke, mask: Option<&Mask>) {
        let Some((sx, sy)) = self.pixel_scale() else { return };
        let mapped: Vec<Vec2> = points.iter().filter_map(|p| self.doc_to_pixel(*p)).collect();
        let pixel_width = width * (sx + sy) / 2.0;
        self.edit_pixels(mask, |bitmap| match op {
            PixelStroke::Paint(color) => bitmap.paint_stroke(&mapped, pixel_width, color),
            PixelStroke::Erase => bitmap.erase_stroke(&mapped, pixel_width),
            PixelStroke::Blur => bitmap.blur_stroke(&mapped, pixel_width),
            PixelStroke::Clone(offset) => bitmap.clone_stroke(&mapped, pixel_width, v2(offset.x * sx, offset.y * sy)),
            PixelStroke::Sharpen => bitmap.sharpen_stroke(&mapped, pixel_width),
            PixelStroke::Dodge(exposure, range) => bitmap.tone_stroke(&mapped, pixel_width, true, exposure, range),
            PixelStroke::Burn(exposure, range) => bitmap.tone_stroke(&mapped, pixel_width, false, exposure, range),
            PixelStroke::Smudge(pressure, color) => bitmap.smudge_stroke(&mapped, pixel_width, pressure, color),
            PixelStroke::Replace(from, to, tolerance) => bitmap.replace_color_stroke(&mapped, pixel_width, from, to, tolerance),
        });
    }

    /// Flattened outline used by canvas drawing, PNG export and hit testing.
    /// A frame of the object's own that follows whatever is done to it: a point and two arms from it.
    /// A shape that records its transform has the middle of its box and the box's half sides, carried by
    /// the transform. A path or a line keeps no record, so it has the middle of its points and the arm to
    /// its first point, which turn as it turns. Anything else has its upright box.
    fn anchor(&self) -> (Vec2, Vec2, Vec2) {
        let sound = |arm: Vec2| arm.x.hypot(arm.y) > 1e-3;
        match &self.shape {
            Shape::Text { .. } | Shape::Instance { .. } | Shape::Rect { .. } | Shape::Ellipse | Shape::Auto { .. } => {
                let frame = self.placement.map_or(self.bounds, |p| p.frame);
                let carry = |p: Vec2| self.placement.map_or(p, |placement| placement.warp.apply(p));
                let (o, x, y) = (carry(frame.center()), carry(v2(frame.right(), frame.center().y)), carry(v2(frame.center().x, frame.bottom())));
                let (x, y) = (v2(x.x - o.x, x.y - o.y), v2(y.x - o.x, y.y - o.y));
                if sound(x) && sound(y) {
                    return (o, x, y);
                }
            }
            Shape::Line | Shape::Path { .. } if self.points.len() >= 2 => {
                let count = self.points.len() as f32;
                let o = v2(self.points.iter().map(|p| p.x).sum::<f32>() / count, self.points.iter().map(|p| p.y).sum::<f32>() / count);
                if let Some(x) = self.points.iter().map(|p| v2(p.x - o.x, p.y - o.y)).find(|arm| sound(*arm)) {
                    return (o, x, v2(-x.y, x.x));
                }
            }
            _ => {}
        }
        (self.bounds.center(), v2((self.bounds.w / 2.0).max(1.0), 0.0), v2(0.0, (self.bounds.h / 2.0).max(1.0)))
    }

    /// Where a point of the object's own frame is in the document.
    fn anchored(&self, at: Vec2) -> Vec2 {
        let (o, x, y) = self.anchor();
        v2(o.x + at.x * x.x + at.y * y.x, o.y + at.x * x.y + at.y * y.y)
    }

    /// The object as its behaviors have it after `tick` frames of playing at `fps` frames a second: moved,
    /// and turned and resized around its transformation point.
    pub fn behaved(&self, tick: usize, fps: f32, stage: (f32, f32)) -> Object {
        let moved = crate::behavior::moved(&self.behaviors, tick, fps, self.bounds, stage);
        let pivot = self.pivot_point().unwrap_or(self.bounds.center());
        let mut object = self.clone();
        if (moved.scale - 1.0).abs() > 1e-4 {
            let size = moved.scale.max(0.01);
            object.warp(&Warp::fit(Rect { x: pivot.x - 1.0, y: pivot.y - 1.0, w: 2.0, h: 2.0 }, Rect { x: pivot.x - size, y: pivot.y - size, w: 2.0 * size, h: 2.0 * size }));
        }
        if moved.angle.abs() > 1e-4 {
            object.warp(&Warp::rotation(pivot, moved.angle));
        }
        if moved.dx != 0.0 || moved.dy != 0.0 {
            object.translate(moved.dx, moved.dy);
        }
        object.opacity *= moved.alpha;
        object
    }

    /// Where the object's transformation point is in the document, if it has been taken off the middle.
    pub fn pivot_point(&self) -> Option<Vec2> {
        self.pivot.map(|at| self.anchored(at))
    }

    /// Puts the transformation point on a point of the document, or, with None, back in the middle.
    pub fn set_pivot_point(&mut self, point: Option<Vec2>) {
        self.pivot = point.and_then(|p| {
            let (o, x, y) = self.anchor();
            let det = x.x * y.y - x.y * y.x;
            let (dx, dy) = (p.x - o.x, p.y - o.y);
            (det.abs() > 1e-6).then(|| v2((dx * y.y - dy * y.x) / det, (dy * x.x - dx * x.y) / det))
        });
    }

    pub fn outline(&self) -> Vec<Vec2> {
        let upright = self.upright_outline();
        match &self.placement {
            Some(placement) => upright.into_iter().map(|p| placement.warp.apply(p)).collect(),
            None => upright,
        }
    }

    /// The outline in the object's own frame, before any transform it carries.
    fn upright_outline(&self) -> Vec<Vec2> {
        let b = self.placement.map_or(self.bounds, |p| p.frame);
        match &self.shape {
            Shape::Rect { radius } => {
                let r = radius.min(b.w / 2.0).min(b.h / 2.0).max(0.0);
                if r <= 0.0 {
                    return vec![
                        v2(b.x, b.y),
                        v2(b.right(), b.y),
                        v2(b.right(), b.bottom()),
                        v2(b.x, b.bottom()),
                    ];
                }
                let centers = [
                    (v2(b.right() - r, b.y + r), -90.0_f32),
                    (v2(b.right() - r, b.bottom() - r), 0.0),
                    (v2(b.x + r, b.bottom() - r), 90.0),
                    (v2(b.x + r, b.y + r), 180.0),
                ];
                let mut out = Vec::with_capacity(4 * (CORNER_SEGMENTS + 1));
                for (c, start) in centers {
                    for i in 0..=CORNER_SEGMENTS {
                        let a = (start + 90.0 * i as f32 / CORNER_SEGMENTS as f32).to_radians();
                        out.push(v2(c.x + r * a.cos(), c.y + r * a.sin()));
                    }
                }
                out
            }
            Shape::Ellipse => {
                let c = b.center();
                (0..ELLIPSE_SEGMENTS)
                    .map(|i| {
                        let a = std::f32::consts::TAU * i as f32 / ELLIPSE_SEGMENTS as f32;
                        v2(c.x + b.w / 2.0 * a.cos(), c.y + b.h / 2.0 * a.sin())
                    })
                    .collect()
            }
            Shape::Line | Shape::Path { .. } => flatten_curve(&self.points, &self.handles, self.is_closed()),
            Shape::Auto { kind, a, b: second, count } => {
                autoshape::outline(*kind, AutoParams { a: *a, b: *second, count: *count }, b)
            }
            Shape::Text { .. } | Shape::Image { .. } | Shape::Web { .. } | Shape::Instance { .. } | Shape::Group { .. } => vec![
                v2(b.x, b.y),
                v2(b.right(), b.y),
                v2(b.right(), b.bottom()),
                v2(b.x, b.bottom()),
            ],
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.translate_own(dx, dy);
        self.with_mask(|mask| mask.translate(dx, dy));
    }

    fn translate_own(&mut self, dx: f32, dy: f32) {
        self.bounds.x += dx;
        self.bounds.y += dy;
        for p in &mut self.points {
            p.x += dx;
            p.y += dy;
        }
        if let Some(placement) = &mut self.placement {
            placement.warp = placement.warp.then(&Warp::translation(dx, dy));
        }
    }

    /// Moves or scales the object into `target`, carrying its vertices along.
    pub fn set_bounds(&mut self, target: Rect) {
        let old = self.bounds;
        self.set_bounds_own(target);
        // The mask keeps its place within the object as the object moves and stretches.
        let fit = Warp::fit(old, target);
        self.with_mask(|mask| {
            let b = mask.bounds;
            let (corner, far) = (fit.apply(v2(b.x, b.y)), fit.apply(v2(b.right(), b.bottom())));
            mask.set_bounds(Rect { x: corner.x, y: corner.y, w: far.x - corner.x, h: far.y - corner.y });
        });
    }

    fn set_bounds_own(&mut self, target: Rect) {
        let old = self.bounds;
        let sx = if old.w > 0.0 { target.w / old.w } else { 1.0 };
        let sy = if old.h > 0.0 { target.h / old.h } else { 1.0 };
        for p in &mut self.points {
            p.x = target.x + (p.x - old.x) * sx;
            p.y = target.y + (p.y - old.y) * sy;
        }
        for h in &mut self.handles {
            *h = h.map(|v| v2(v.x * sx, v.y * sy));
        }
        if let Some(placement) = &mut self.placement {
            placement.warp = placement.warp.then(&Warp::fit(old, target));
        }
        self.bounds = target;
    }

    pub fn flip(&mut self, horizontal: bool) {
        let b = self.bounds;
        self.flip_own(horizontal);
        self.with_mask(|mask| {
            let m = mask.bounds;
            if horizontal {
                mask.translate(b.x + b.right() - m.right() - m.x, 0.0);
            } else {
                mask.translate(0.0, b.y + b.bottom() - m.bottom() - m.y);
            }
            mask.flip(horizontal);
        });
    }

    fn flip_own(&mut self, horizontal: bool) {
        let b = self.bounds;
        match &self.shape {
            // Mirrored through their placement, so they stay editable.
            Shape::Text { .. } | Shape::Instance { .. } | Shape::Auto { .. } => {
                self.warp(&Warp::mirror(b.center(), horizontal));
                return;
            }
            Shape::Rect { .. } | Shape::Ellipse if self.placement.is_some() => {
                self.warp(&Warp::mirror(b.center(), horizontal));
                return;
            }
            // Each part swaps sides within the group and is mirrored itself.
            Shape::Group { .. } => {
                let mut parts = self.instance_parts();
                for part in &mut parts {
                    let p = part.bounds;
                    if horizontal {
                        part.translate(b.x + b.right() - p.right() - p.x, 0.0);
                    } else {
                        part.translate(0.0, b.y + b.bottom() - p.bottom() - p.y);
                    }
                    part.flip(horizontal);
                }
                self.opacity = 1.0;
                self.shape = Shape::Group { objects: Arc::new(parts) };
                return;
            }
            _ => {}
        }
        if let Some(bitmap) = self.bitmap_mut() {
            bitmap.flip(horizontal);
        }
        let b = self.bounds;
        for p in &mut self.points {
            if horizontal {
                p.x = b.x + b.right() - p.x;
            } else {
                p.y = b.y + b.bottom() - p.y;
            }
        }
        for h in &mut self.handles {
            *h = h.map(|v| if horizontal { v2(-v.x, v.y) } else { v2(v.x, -v.y) });
        }
    }

    /// Turns the object a quarter turn around `center`. Returns false if it cannot turn.
    pub fn rotate90(&mut self, clockwise: bool, center: Vec2) -> bool {
        let done = self.rotate90_own(clockwise, center);
        if done {
            self.with_mask(|mask| {
                mask.rotate90(clockwise, center);
            });
        }
        done
    }

    fn rotate90_own(&mut self, clockwise: bool, center: Vec2) -> bool {
        let turn = |p: Vec2| {
            let (dx, dy) = (p.x - center.x, p.y - center.y);
            if clockwise { v2(center.x - dy, center.y + dx) } else { v2(center.x + dy, center.y - dx) }
        };
        if self.placement.is_some() {
            return self.warp(&Warp::quarter_turn(center, clockwise));
        }
        match self.shape {
            Shape::Text { .. } | Shape::Instance { .. } => return self.warp(&Warp::quarter_turn(center, clockwise)),
            Shape::Group { .. } => {
                let mut parts = self.instance_parts();
                if !parts.iter_mut().all(|part| part.rotate90(clockwise, center)) {
                    return false;
                }
                self.bounds = parts.iter().map(|part| part.bounds).reduce(Rect::union).unwrap_or(self.bounds);
                self.opacity = 1.0;
                self.shape = Shape::Group { objects: Arc::new(parts) };
                return true;
            }
            Shape::Auto { .. } => {
                self.to_path();
            }
            _ => {}
        }
        if let Some(bitmap) = self.bitmap_mut() {
            bitmap.rotate90(clockwise);
        }
        let middle = turn(self.bounds.center());
        let (w, h) = (self.bounds.h, self.bounds.w);
        self.bounds = Rect { x: middle.x - w / 2.0, y: middle.y - h / 2.0, w, h };
        for point in &mut self.points {
            *point = turn(*point);
        }
        for h in &mut self.handles {
            *h = h.map(|v| if clockwise { v2(-v.y, v.x) } else { v2(v.y, -v.x) });
        }
        true
    }

    /// Recomputes `bounds` from the vertices after a vertex edit.
    pub fn refit_to_points(&mut self) {
        if !self.points.is_empty() {
            self.bounds = Rect::of_points(&self.outline());
        }
    }

    pub fn hit(&self, p: Vec2, tolerance: f32) -> bool {
        if !self.visible || self.locked {
            return false;
        }
        if matches!(self.shape, Shape::Text { .. } | Shape::Image { .. } | Shape::Web { .. } | Shape::Instance { .. } | Shape::Group { .. }) {
            return self.bounds.contains(p);
        }
        let outline = self.outline();
        if self.fill.is_some() && point_in_polygon(p, &outline) {
            return true;
        }
        let reach = tolerance + self.stroke.map_or(0.0, |s| s.width / 2.0);
        let count = outline.len();
        let segments = if self.is_closed() { count } else { count.saturating_sub(1) };
        (0..segments).any(|i| dist_to_segment(p, outline[i], outline[(i + 1) % count]) <= reach)
    }
}

/// Joins holes or separate islands onto one outline with zero-width seams, so a single
/// even-odd filled path can hold them. Use [`contours`] to take the rings apart again.
pub fn bridge(outer: Vec<Vec2>, holes: Vec<Vec<Vec2>>) -> Vec<Vec2> {
    let mut path = outer;
    for hole in holes.into_iter().filter(|h| !h.is_empty()) {
        if path.is_empty() {
            path = hole;
            continue;
        }
        let (mut best, mut at, mut from) = (f32::MAX, 0, 0);
        for (i, a) in path.iter().enumerate() {
            for (j, b) in hole.iter().enumerate() {
                let distance = (a.x - b.x).powi(2) + (a.y - b.y).powi(2);
                if distance < best {
                    (best, at, from) = (distance, i, j);
                }
            }
        }
        let mut spliced = Vec::with_capacity(path.len() + hole.len() + 2);
        spliced.extend_from_slice(&path[..=at]);
        spliced.extend(hole.iter().cycle().skip(from).take(hole.len() + 1).copied());
        spliced.extend_from_slice(&path[at..]);
        path = spliced;
    }
    path
}

/// Splits a closed outline into its rings by removing edges walked in both directions.
pub fn contours(points: &[Vec2]) -> Vec<Vec<Vec2>> {
    let count = points.len();
    if count < 3 {
        return vec![points.to_vec()];
    }
    let key = |p: Vec2| (p.x.to_bits(), p.y.to_bits());
    let ends = |i: usize| (key(points[i]), key(points[(i + 1) % count]));
    let mut edges: HashMap<_, Vec<usize>> = HashMap::new();
    for i in 0..count {
        edges.entry(ends(i)).or_default().push(i);
    }
    let mut seam = vec![false; count];
    for i in 0..count {
        if seam[i] {
            continue;
        }
        let (start, end) = ends(i);
        let twin = edges.get(&(end, start)).and_then(|list| list.iter().copied().find(|j| *j != i && !seam[*j]));
        if let Some(twin) = twin {
            seam[i] = true;
            seam[twin] = true;
        }
    }
    if !seam.contains(&true) {
        return vec![points.to_vec()];
    }
    let mut outgoing: HashMap<_, Vec<usize>> = HashMap::new();
    for i in (0..count).filter(|i| !seam[*i]) {
        outgoing.entry(key(points[i])).or_default().push(i);
    }
    let mut used = vec![false; count];
    let mut rings = Vec::new();
    for first in 0..count {
        if seam[first] || used[first] {
            continue;
        }
        let mut ring = Vec::new();
        let mut current = first;
        loop {
            used[current] = true;
            ring.push(points[current]);
            let next = outgoing.get(&ends(current).1).and_then(|list| list.iter().copied().find(|e| !used[*e]));
            match next {
                Some(edge) => current = edge,
                None => break,
            }
        }
        rings.push(ring);
    }
    rings
}

pub fn point_in_polygon(p: Vec2, poly: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = poly.len().wrapping_sub(1);
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Adds vertices so that no segment is longer than `max_len`.
pub fn subdivide(points: &[Vec2], closed: bool, max_len: f32) -> Vec<Vec2> {
    let count = points.len();
    let segments = if closed { count } else { count.saturating_sub(1) };
    let mut out = Vec::with_capacity(count);
    for i in 0..segments {
        let (a, b) = (points[i], points[(i + 1) % count]);
        let len = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        let parts = (len / max_len).ceil().max(1.0) as usize;
        for n in 0..parts {
            let t = n as f32 / parts as f32;
            out.push(v2(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
        }
    }
    if !closed {
        out.extend(points.last());
    }
    out
}

/// Where a point came from before its box was resized by 9-slice scaling: the way back from [`nine_slice_point`].
pub fn nine_slice_source(p: Vec2, from: Rect, to: Rect, guides: [f32; 4]) -> Vec2 {
    // The cuts keep their distance from the edges, so on the new box they are at other parts of its length.
    let back = |near: f32, far: f32, old: f32, new: f32| {
        let (low, high) = (near.min(far), near.max(far));
        if new - low * old - (1.0 - high) * old <= 0.0 || new <= 0.0 { (near, far) } else { (low * old / new, 1.0 - (1.0 - high) * old / new) }
    };
    let (x0, x1) = back(guides[0], guides[1], from.w, to.w);
    let (y0, y1) = back(guides[2], guides[3], from.h, to.h);
    nine_slice_point(p, to, from, [x0, x1, y0, y1])
}

/// Where a point goes when its box is resized by 9-slice scaling: outside the cuts
/// it keeps its distance from the nearer edge, between them it is stretched.
pub fn nine_slice_point(p: Vec2, from: Rect, to: Rect, guides: [f32; 4]) -> Vec2 {
    // One direction at a time: the old edges, the new edges, and the two cuts as parts of the old length.
    fn along(v: f32, from: (f32, f32), to: (f32, f32), cuts: (f32, f32)) -> f32 {
        let length = from.1 - from.0;
        let (near, far) = (length * cuts.0.min(cuts.1).clamp(0.0, 1.0), length * (1.0 - cuts.0.max(cuts.1).clamp(0.0, 1.0)));
        let (middle, new_middle) = (length - near - far, (to.1 - to.0) - near - far);
        // Too small for both ends to keep their size: everything scales alike.
        if new_middle <= 0.0 || middle <= 0.0 {
            return to.0 + (v - from.0) / length.max(0.001) * (to.1 - to.0);
        }
        if v <= from.0 + near {
            to.0 + (v - from.0)
        } else if v >= from.1 - far {
            to.1 - (from.1 - v)
        } else {
            to.0 + near + (v - from.0 - near) / middle * new_middle
        }
    }
    v2(
        along(p.x, (from.x, from.right()), (to.x, to.right()), (guides[0], guides[1])),
        along(p.y, (from.y, from.bottom()), (to.y, to.bottom()), (guides[2], guides[3])),
    )
}

/// Fireworks' Redraw Path: the part of a path between where a freehand stroke
/// starts and where it ends is replaced by the stroke. Both ends must come within
/// `reach` of the path; on an open path a stroke that leaves the path replaces it
/// from where it started to the nearer end.
pub fn redraw_path(points: &[Vec2], closed: bool, stroke: &[Vec2], reach: f32) -> Option<Vec<Vec2>> {
    let (first, last) = (*stroke.first()?, *stroke.last()?);
    if points.len() < 2 || stroke.len() < 2 {
        return None;
    }
    let nearest = |p: Vec2| points.iter().enumerate().map(|(index, q)| (index, (q.x - p.x).hypot(q.y - p.y))).min_by(|a, b| a.1.total_cmp(&b.1));
    let (start, start_far) = nearest(first)?;
    let (end, end_far) = nearest(last)?;
    if start_far > reach {
        return None;
    }
    let count = points.len();
    let mut out = Vec::new();
    if end_far > reach {
        if closed {
            return None;
        }
        // Drawn off the path: the nearer end of it is what gets redrawn.
        if start < count / 2 {
            out.extend(stroke.iter().rev());
            out.extend(&points[start + 1..]);
        } else {
            out.extend(&points[..start]);
            out.extend(stroke);
        }
        return Some(out);
    }
    if start == end {
        return None;
    }
    let (low, high, ordered): (usize, usize, Vec<Vec2>) = if start < end { (start, end, stroke.to_vec()) } else { (end, start, stroke.iter().rev().copied().collect()) };
    // On a closed path the shorter way round is the part meant.
    if closed && high - low > count / 2 {
        out.extend(&points[low + 1..high]);
        out.extend(ordered.iter().rev());
    } else {
        out.extend(&points[..low]);
        out.extend(&ordered);
        out.extend(&points[high + 1..]);
    }
    Some(out)
}

/// Drags vertices near `start` by `delta`, fading smoothly to nothing at `radius`.
pub fn pull_points(base: &[Vec2], start: Vec2, delta: Vec2, radius: f32) -> Vec<Vec2> {
    base.iter()
        .map(|p| {
            let distance = ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt();
            if distance >= radius {
                return *p;
            }
            let weight = 0.5 + 0.5 * (std::f32::consts::PI * distance / radius).cos();
            v2(p.x + delta.x * weight, p.y + delta.y * weight)
        })
        .collect()
}

/// Cuts a polyline along segment `a`-`b`. Returns `None` if the segment misses it.
pub fn split_path(points: &[Vec2], closed: bool, a: Vec2, b: Vec2) -> Option<Vec<Vec<Vec2>>> {
    let cross = |u: Vec2, v: Vec2| u.x * v.y - u.y * v.x;
    let sub = |u: Vec2, v: Vec2| v2(u.x - v.x, u.y - v.y);
    let count = points.len();
    let segments = if closed { count } else { count.saturating_sub(1) };
    let cut_dir = sub(b, a);
    // Each entry is the segment index and the cut point on it, in path order.
    let mut cuts: Vec<(usize, Vec2)> = Vec::new();
    for i in 0..segments {
        let (p, q) = (points[i], points[(i + 1) % count]);
        let dir = sub(q, p);
        let denominator = cross(dir, cut_dir);
        if denominator.abs() < 1e-6 {
            continue;
        }
        let t = cross(sub(a, p), cut_dir) / denominator;
        let u = cross(sub(a, p), dir) / denominator;
        if (0.0..1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            cuts.push((i, v2(p.x + dir.x * t, p.y + dir.y * t)));
        }
    }
    if cuts.is_empty() {
        return None;
    }
    let mut pieces = Vec::new();
    if closed {
        for (n, (segment, point)) in cuts.iter().enumerate() {
            let (next_segment, next_point) = cuts[(n + 1) % cuts.len()];
            // A lone cut wraps all the way around the ring back to itself.
            let steps = match (next_segment + count - segment) % count {
                0 => count,
                steps => steps,
            };
            let mut piece = vec![*point];
            piece.extend((1..=steps).map(|k| points[(segment + k) % count]));
            piece.push(next_point);
            pieces.push(piece);
        }
    } else {
        let mut piece = points[..=cuts[0].0].to_vec();
        for (n, (segment, point)) in cuts.iter().enumerate() {
            piece.push(*point);
            pieces.push(std::mem::replace(&mut piece, vec![*point]));
            let end = cuts.get(n + 1).map_or(count - 1, |c| c.0);
            piece.extend_from_slice(&points[segment + 1..=end]);
        }
        pieces.push(piece);
    }
    pieces.retain(|piece| {
        let extent = Rect::of_points(piece);
        extent.w > 0.0 || extent.h > 0.0
    });
    Some(pieces)
}

pub fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx * dx + dy * dy;
    let t = if len > 0.0 {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / len).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p.x - a.x - t * dx).powi(2) + (p.y - a.y - t * dy).powi(2)).sqrt()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    /// A shared layer shows the same objects in every state.
    #[serde(default)]
    pub shared: bool,
    /// Bottom-most object first.
    pub objects: Vec<Object>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), visible: true, locked: false, shared: false, objects: Vec::new() }
    }
}

pub const DEFAULT_STATE_DELAY: u32 = 500;

/// One frame of a page. The active state's objects live in the layers, and every
/// other state keeps one object list per layer here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub name: String,
    pub delay_ms: u32,
    /// Whether animated export and playback include this state.
    pub exported: bool,
    pub objects: Vec<Vec<Object>>,
}

impl State {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), delay_ms: DEFAULT_STATE_DELAY, exported: true, objects: Vec::new() }
    }
}

fn default_states() -> Vec<State> {
    vec![State::new(tf("State {}", &[&1]))]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// `None` is a transparent canvas.
    pub canvas: Option<Color>,
    /// Bottom-most layer first.
    pub layers: Vec<Layer>,
    pub active_layer: usize,
    #[serde(default = "default_states")]
    pub states: Vec<State>,
    #[serde(default)]
    pub active_state: usize,
    #[serde(default)]
    pub export: ExportSettings,
    /// The part of the page a JPEG keeps at its selective quality, a cell for each pixel of the page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jpeg_mask: Option<Arc<crate::bitmap::Mask>>,
    /// The color the JPEG mask is tinted with on the canvas; `None` is the usual pink.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jpeg_mask_color: Option<Color>,
    /// The symbols that are buttons. Filled in only on the copy of a page handed to
    /// the exporter, which has the page but not the document's library.
    #[serde(skip)]
    pub button_symbols: Vec<u64>,
    /// The guides dragged out of the rulers.
    #[serde(default, skip_serializing_if = "Guides::is_empty")]
    pub guides: Guides,
    /// The groups opened with Subselect, whose members lie loose on the page until
    /// the selection leaves them. Never written to a file: groups are closed first.
    #[serde(skip)]
    pub open_groups: Vec<OpenGroup>,
    /// The looks of the button symbols that have more than one. Filled in, like
    /// `button_symbols`, only on the copy of a page handed to the exporter.
    #[serde(skip)]
    pub button_looks: Vec<ButtonLooks>,
    /// The timeline, in a document of the animation program; the page is then a scene of the animation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<Timeline>,
    /// Set on the page that holds the timeline of a movie clip: the number of the clip's symbol.
    /// Such a page is not a scene of the animation, and is shown only while the clip is edited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clip_of: Option<u64>,
    /// Where on the clip's page the top left of the clip's first frame lies: the point its copies are placed by.
    #[serde(default)]
    pub clip_origin: Vec2,
    /// How many times an animated GIF made from the page plays: 0 is for ever, 1 is once through.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub gif_loops: u16,
    /// The symbol being edited in place. Never written to a file: the edit is ended first.
    #[serde(skip)]
    pub symbol_edit: Option<SymbolEdit>,
    /// The animation symbols on the page.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<Animation>,
}

/// A group whose members are being worked on one by one.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenGroup {
    /// The group without its members: its name, mask, filters and blend mode are put back when it closes.
    pub shell: Object,
    pub members: Vec<u64>,
}

/// The guides of a page: lines to line things up with, which are never part of the image.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Guides {
    /// The heights of the guides that run across the page.
    #[serde(default)]
    pub horizontal: Vec<f32>,
    /// The places of the guides that run down the page.
    #[serde(default)]
    pub vertical: Vec<f32>,
}

impl Guides {
    pub fn is_empty(&self) -> bool {
        self.horizontal.is_empty() && self.vertical.is_empty()
    }

    fn list(&self, horizontal: bool) -> &Vec<f32> {
        if horizontal { &self.horizontal } else { &self.vertical }
    }

    pub fn list_mut(&mut self, horizontal: bool) -> &mut Vec<f32> {
        if horizontal { &mut self.horizontal } else { &mut self.vertical }
    }

    /// The guide within `reach` of a point: whether it runs across, and which one it is. The nearest wins.
    pub fn near(&self, p: Vec2, reach: f32) -> Option<(bool, usize)> {
        let nearest = |horizontal: bool, at: f32| {
            self.list(horizontal).iter().enumerate().map(|(index, guide)| ((guide - at).abs(), horizontal, index)).filter(|(far, _, _)| *far <= reach).min_by(|a, b| a.0.total_cmp(&b.0))
        };
        [nearest(true, p.y), nearest(false, p.x)].into_iter().flatten().min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, horizontal, index)| (horizontal, index))
    }
}

/// How far to move so that one of `values` lands on one of `targets`, if any is within `reach`; the shortest move wins.
pub fn snap_offset(values: &[f32], targets: &[f32], reach: f32) -> Option<f32> {
    values.iter().flat_map(|value| targets.iter().map(move |target| target - value)).filter(|offset| offset.abs() <= reach).min_by(|a, b| a.abs().total_cmp(&b.abs()))
}

/// The three places of a box that line up with others, along each direction: its near edge, its middle and its far edge.
pub fn snap_places(bounds: Rect) -> ([f32; 3], [f32; 3]) {
    ([bounds.x, bounds.x + bounds.w / 2.0, bounds.right()], [bounds.y, bounds.y + bounds.h / 2.0, bounds.bottom()])
}

impl Page {
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            canvas: Some(Color::WHITE),
            layers: vec![Layer::new(tf("Layer {}", &[&1]))],
            active_layer: 0,
            states: default_states(),
            active_state: 0,
            export: ExportSettings::default(),
            jpeg_mask: None,
            jpeg_mask_color: None,
            button_symbols: Vec::new(),
            guides: Guides::default(),
            open_groups: Vec::new(),
            symbol_edit: None,
            clip_of: None,
            clip_origin: Vec2::default(),
            timeline: None,
            gif_loops: 0,
            animations: Vec::new(),
            button_looks: Vec::new(),
        }
    }

    /// Whether a group that is open has something of its own to draw with: a mask, filters or a blend mode.
    pub fn open_groups_need_drawing(&self) -> bool {
        self.open_groups.iter().any(|open| open.shell.mask.is_some() || !open.shell.filters.is_empty() || !open.shell.blend.is_normal())
    }

    /// The page as it is drawn while groups are open: each open group is put back
    /// together, so that its own mask, filters and blend mode show while its members are worked on.
    pub fn with_groups_closed(&self) -> Page {
        let mut page = self.clone();
        for OpenGroup { mut shell, members } in std::mem::take(&mut page.open_groups) {
            let mut found = Vec::new();
            let mut spot = None;
            for (layer_index, layer) in page.layers.iter_mut().enumerate() {
                let mut at = 0;
                while at < layer.objects.len() {
                    if members.contains(&layer.objects[at].id) {
                        spot.get_or_insert((layer_index, at));
                        found.push(layer.objects.remove(at));
                    } else {
                        at += 1;
                    }
                }
            }
            let (Some((layer, at)), Some(bounds)) = (spot, found.iter().map(|o| o.bounds).reduce(Rect::union)) else { continue };
            shell.bounds = bounds;
            shell.shape = Shape::Group { objects: Arc::new(found) };
            let at = at.min(page.layers[layer].objects.len());
            page.layers[layer].objects.insert(at, shell);
        }
        page
    }

    /// Gives the page a timeline, if it has none: each layer starts with one keyframe holding what it shows.
    pub fn start_timeline(&mut self, fps: f32) {
        if self.timeline.is_none() {
            let layers = self.layers.iter().map(|layer| vec![Span { start: 0, length: 1, objects: layer.objects.clone(), ..Default::default() }]).collect();
            self.timeline = Some(Timeline { fps, current: 0, layers, kinds: Vec::new(), bound: Vec::new() });
        }
    }

    /// Writes what the layers show into the timeline, as work done on the frame on show.
    /// On a frame of a plain keyframe that work belongs to the keyframe. On a frame inside
    /// a tween it makes a keyframe of the frame, as Flash does, if anything was changed there.
    pub fn store_frame(&mut self) {
        let Some(mut timeline) = self.timeline.take() else { return };
        timeline.layers.resize(self.layers.len(), Vec::new());
        let frame = timeline.current;
        for (index, layer) in self.layers.iter().enumerate() {
            if timeline.layers[index].is_empty() {
                timeline.layers[index].push(Span { start: 0, length: frame + 1, objects: Vec::new(), ..Default::default() });
            }
            let Some(at) = timeline.span_at(index, frame) else {
                // Past the end of the layer: drawing there makes it reach the frame, with a keyframe of what was drawn.
                if !layer.objects.is_empty() {
                    timeline.insert_keyframe(index, frame, true);
                    if let Some(at) = timeline.span_at(index, frame) {
                        timeline.layers[index][at].objects = layer.objects.clone();
                    }
                }
                continue;
            };
            let span = &timeline.layers[index][at];
            if span.tween.is_none() || span.start == frame {
                timeline.layers[index][at].objects = layer.objects.clone();
            } else if timeline.objects_at(index, frame) != layer.objects {
                timeline.insert_keyframe(index, frame, true);
                if let Some(at) = timeline.span_at(index, frame) {
                    let tween = timeline.layers[index][at - 1].tween.clone();
                    (timeline.layers[index][at].objects, timeline.layers[index][at].tween) = (layer.objects.clone(), tween);
                }
            }
        }
        self.timeline = Some(timeline);
    }

    /// Puts into the layers what the timeline has for the frame on show.
    pub fn load_frame(&mut self) {
        let Some(timeline) = &self.timeline else { return };
        let shown: Vec<Vec<Object>> = (0..self.layers.len()).map(|layer| timeline.objects_at(layer, timeline.current)).collect();
        for (layer, objects) in self.layers.iter_mut().zip(shown) {
            layer.objects = objects;
        }
    }

    /// Shows another frame: the work on this one is kept first.
    pub fn goto_frame(&mut self, frame: usize) {
        self.store_frame();
        if let Some(timeline) = &mut self.timeline {
            timeline.current = frame.min(MAX_FRAMES - 1);
        }
        self.load_frame();
    }

    /// A copy of the page showing another frame, as playing and exporting need it.
    pub fn at_frame(&self, frame: usize) -> Page {
        let mut page = self.clone();
        page.goto_frame(frame);
        page
    }

    /// The page as the finished animation shows the frame: guide layers are left out,
    /// and a mask layer lets the layer beneath it show only where the mask has something
    /// drawn. The copy has no timeline, so nothing done to it can find its way back.
    pub fn played(&self) -> Page {
        self.composed(true)
    }

    /// Whether a mask is to show on the stage: as in Flash, when the mask layer and every layer tied to it are locked.
    fn mask_shows_locked(&self, timeline: &Timeline, mask: usize) -> bool {
        let tied: Vec<usize> = (0..mask).filter(|layer| timeline.master(*layer) == Some((mask, LayerKind::Mask))).collect();
        self.layers[mask].locked && !tied.is_empty() && tied.iter().all(|layer| self.layers[*layer].locked)
    }

    /// Whether any mask is to show on the stage while editing.
    pub fn shows_locked_masks(&self) -> bool {
        self.timeline.as_ref().is_some_and(|timeline| (0..self.layers.len()).any(|layer| timeline.kind(layer) == LayerKind::Mask && self.mask_shows_locked(timeline, layer)))
    }

    /// The page as the stage shows it while editing, when locked masks are to show: those masks at work, everything else as it lies.
    pub fn with_locked_masks(&self) -> Page {
        self.composed(false)
    }

    fn composed(&self, playing: bool) -> Page {
        let mut page = self.clone();
        let Some(timeline) = page.timeline.take() else { return page };
        for index in 0..page.layers.len() {
            match timeline.kind(index) {
                LayerKind::Normal => {}
                LayerKind::Guide => {
                    if playing {
                        page.layers[index].objects.clear();
                    }
                }
                LayerKind::Mask => {
                    if !playing && !self.mask_shows_locked(&timeline, index) {
                        continue;
                    }
                    let shapes = std::mem::take(&mut page.layers[index].objects);
                    if !page.layers[index].visible {
                        continue;
                    }
                    let bundle = |objects: Vec<Object>, id: u64| {
                        let bounds = objects.iter().map(|o| o.bounds).reduce(Rect::union)?;
                        let mut group = Object::new_web(id, WebKind::Slice, bounds);
                        group.shape = Shape::Group { objects: Arc::new(objects) };
                        Some(group)
                    };
                    // Every layer tied to the mask shows only where the mask has something; with nothing in the mask, nothing of them shows.
                    let mask = bundle(shapes, u64::MAX - index as u64 * 2 - 1);
                    for below in (0..index).filter(|layer| timeline.master(*layer) == Some((index, LayerKind::Mask))) {
                        let masked = std::mem::take(&mut page.layers[below].objects);
                        let (Some(mut content), Some(mask)) = (bundle(masked, u64::MAX / 2 - below as u64), mask.clone()) else { continue };
                        content.mask = Some(Arc::new(ObjectMask { object: mask, kind: MaskKind::Alpha, show_fill_stroke: false, enabled: true, linked: true }));
                        page.layers[below].objects = vec![content];
                    }
                }
            }
        }
        page
    }

    /// Changes the timeline with the work on the frame on show kept, and shows the frame afresh afterwards.
    /// Whether anything on the page has been given a behavior.
    pub fn has_behaviors(&self) -> bool {
        self.objects().any(|o| !o.behaviors.is_empty())
    }

    /// The page with its behaviors at work, as they have things after `tick` frames of playing.
    pub fn behaved(&self, tick: usize) -> Page {
        let fps = self.timeline.as_ref().map_or(24.0, |timeline| timeline.fps);
        let stage = (self.width as f32, self.height as f32);
        let mut page = self.clone();
        for object in page.objects_mut().filter(|o| !o.behaviors.is_empty()) {
            *object = object.behaved(tick, fps, stage);
        }
        page
    }

    pub fn edit_timeline<T>(&mut self, change: impl FnOnce(&mut Timeline) -> T) -> Option<T> {
        self.store_frame();
        let done = change(self.timeline.as_mut()?);
        self.load_frame();
        Some(done)
    }

    /// The buttons that change under the mouse: the copies, lying loose on the page, of button symbols with more than one look.
    pub fn buttons(&self) -> Vec<&Object> {
        self.layers
            .iter()
            .filter(|l| l.visible)
            .flat_map(|l| l.objects.iter())
            .filter(|o| o.visible && matches!(o.shape, Shape::Instance { symbol, .. } if self.button_looks.iter().any(|b| b.symbol == symbol)))
            .collect()
    }

    /// The page with one of its buttons showing another look: 1 is Over, 2 Down, 3 Over
    /// While Down. A look the button does not have leaves it as it is.
    pub fn with_button_look(&self, id: u64, look: usize) -> Page {
        let mut page = self.clone();
        let looks = std::mem::take(&mut page.button_looks);
        if let Some(object) = page.objects_mut().find(|o| o.id == id) {
            if let Shape::Instance { symbol, objects } = &mut object.shape {
                let entry = looks.iter().find(|b| b.symbol == *symbol);
                let parts = entry.and_then(|b| b.looks.get(look.wrapping_sub(1))).filter(|parts| !parts.is_empty());
                if let (Some(entry), Some(parts)) = (entry, parts) {
                    let own = parts.iter().map(|o| o.bounds).reduce(Rect::union).unwrap_or_default();
                    let (natural, frame) = (entry.natural, object.placement.map_or(object.bounds, |p| p.frame));
                    let sx = if natural.w > 0.0 { frame.w / natural.w } else { 1.0 };
                    let sy = if natural.h > 0.0 { frame.h / natural.h } else { 1.0 };
                    let fitted = Rect { x: frame.x + (own.x - natural.x) * sx, y: frame.y + (own.y - natural.y) * sy, w: own.w * sx, h: own.h * sy };
                    *objects = parts.clone();
                    match &mut object.placement {
                        Some(placement) => {
                            placement.frame = fitted;
                            object.bounds = placement.bounds();
                        }
                        None => object.bounds = fitted,
                    }
                }
            }
        }
        page.button_looks = looks;
        page
    }

    /// The part of the page a button takes up with any of its looks, in whole pixels.
    pub fn button_area(&self, id: u64) -> Option<Rect> {
        let area = (0..4).filter_map(|look| self.with_button_look(id, look).objects().find(|o| o.id == id).map(|o| o.bounds)).reduce(Rect::union)?;
        let area = self.rect().intersect(area)?;
        let (left, top) = (area.x.floor(), area.y.floor());
        Some(Rect { x: left, y: top, w: (area.right().ceil() - left).max(1.0), h: (area.bottom().ceil() - top).max(1.0) })
    }

    /// The objects a layer has in a state, whether or not the state is the one on show.
    fn state_layer_mut(&mut self, state: usize, layer: usize) -> Option<&mut Vec<Object>> {
        if state == self.active_state || self.layers.get(layer).is_some_and(|l| l.shared) {
            self.layers.get_mut(layer).map(|l| &mut l.objects)
        } else {
            self.states.get_mut(state)?.objects.get_mut(layer)
        }
    }

    /// Stores the current objects in their state and loads another state's objects.
    pub fn goto_state(&mut self, index: usize) {
        if index == self.active_state || index >= self.states.len() {
            return;
        }
        let stored = self.layers.iter_mut().map(|l| if l.shared { Vec::new() } else { std::mem::take(&mut l.objects) }).collect();
        self.states[self.active_state].objects = stored;
        let loaded = std::mem::take(&mut self.states[index].objects);
        for (layer, objects) in self.layers.iter_mut().zip(loaded) {
            if !layer.shared {
                layer.objects = objects;
            }
        }
        self.active_state = index;
    }

    /// A copy of the page showing another state.
    pub fn at_state(&self, index: usize) -> Page {
        let mut page = self.clone();
        page.goto_state(index);
        page
    }

    fn insert_state(&mut self, state: State) {
        let at = self.active_state + 1;
        self.states.insert(at, state);
        self.goto_state(at);
    }

    /// Adds an empty state after the current one and moves to it.
    pub fn add_state(&mut self) {
        let mut state = State::new(tf("State {}", &[&(self.states.len() + 1)]));
        state.objects = vec![Vec::new(); self.layers.len()];
        self.insert_state(state);
    }

    /// Copies the current state after itself and moves to the copy.
    pub fn duplicate_state(&mut self) {
        let mut state = State::new(tf("State {}", &[&(self.states.len() + 1)]));
        state.delay_ms = self.states[self.active_state].delay_ms;
        state.objects = self.layers.iter().map(|l| if l.shared { Vec::new() } else { l.objects.clone() }).collect();
        self.insert_state(state);
    }

    /// Removes the current state. The last remaining state is kept.
    pub fn delete_state(&mut self) -> bool {
        if self.states.len() < 2 {
            return false;
        }
        let removing = self.active_state;
        self.goto_state(if removing + 1 < self.states.len() { removing + 1 } else { removing - 1 });
        self.states.remove(removing);
        if self.active_state > removing {
            self.active_state -= 1;
        }
        true
    }

    /// Objects drawn by another state, for onion skinning. Shared layers are left out.
    pub fn state_objects(&self, index: usize) -> impl Iterator<Item = &Object> {
        let slots = self.states.get(index).map_or(&[][..], |s| s.objects.as_slice());
        self.layers.iter().zip(slots).filter(|(l, _)| l.visible && !l.shared).flat_map(|(_, objects)| objects.iter())
    }

    fn inactive_states_mut(&mut self) -> impl Iterator<Item = &mut State> {
        let current = self.active_state;
        self.states.iter_mut().enumerate().filter(move |(i, _)| *i != current).map(|(_, state)| state)
    }

    /// Switches sharing for a layer. Sharing keeps what the current state shows.
    pub fn set_layer_shared(&mut self, index: usize, shared: bool) {
        if self.layers[index].shared == shared {
            return;
        }
        let objects = self.layers[index].objects.clone();
        for state in self.inactive_states_mut() {
            if let Some(slot) = state.objects.get_mut(index) {
                *slot = if shared { Vec::new() } else { objects.clone() };
            }
        }
        self.layers[index].shared = shared;
    }

    /// Every object on the page, including those of inactive states.
    pub fn all_objects_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        let inactive = self.states.iter_mut().flat_map(|s| s.objects.iter_mut().flatten());
        self.layers.iter_mut().flat_map(|l| l.objects.iter_mut()).chain(inactive)
    }

    fn retain_all(&mut self, keep: impl Fn(&Object) -> bool) {
        for layer in &mut self.layers {
            layer.objects.retain(&keep);
        }
        for slot in self.states.iter_mut().flat_map(|s| s.objects.iter_mut()) {
            slot.retain(&keep);
        }
    }

    /// What smart guides line a moving selection up with: the edges and middles of
    /// the other objects that show, and of the page. Places across, then places down.
    pub fn smart_targets(&self, moving: &[u64]) -> (Vec<f32>, Vec<f32>) {
        let (mut across, mut down) = (Vec::new(), Vec::new());
        let boxes = self.layers.iter().filter(|l| l.visible).flat_map(|l| &l.objects).filter(|o| o.visible && !moving.contains(&o.id)).map(|o| o.bounds);
        for bounds in boxes.chain([self.rect()]) {
            let (xs, ys) = snap_places(bounds);
            across.extend(xs);
            down.extend(ys);
        }
        (across, down)
    }

    pub fn rect(&self) -> Rect {
        Rect { x: 0.0, y: 0.0, w: self.width as f32, h: self.height as f32 }
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        self.layers.iter().flat_map(|l| l.objects.iter())
    }

    fn objects_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.layers.iter_mut().flat_map(|l| l.objects.iter_mut())
    }

    fn editable_top_down(&self) -> impl Iterator<Item = &Object> {
        self.layers
            .iter()
            .rev()
            .filter(|l| l.visible && !l.locked)
            .flat_map(|l| l.objects.iter().rev())
            .filter(|o| o.visible && !o.locked)
    }

    /// Finds the top-most artwork under `p`, looking through web areas.
    pub fn art_at(&self, p: Vec2, tolerance: f32) -> Option<u64> {
        self.editable_top_down().find(|o| !o.is_web() && o.hit(p, tolerance)).map(|o| o.id)
    }

    /// Finds the closest vector outline within `reach`, preferring selected objects.
    pub fn nearest_path(&self, p: Vec2, reach: f32, selection: &[u64]) -> Option<u64> {
        let closest = |only_selected: bool| {
            self.editable_top_down()
                .filter(|o| o.reshapable() && (!only_selected || selection.contains(&o.id)))
                .map(|o| (o.id, o.distance_to(p)))
                .filter(|(_, distance)| *distance <= reach)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(id, _)| id)
        };
        closest(true).or_else(|| closest(false))
    }

    /// Finds the editable bitmap under `p`, preferring a selected one over the top-most.
    pub fn bitmap_at(&self, p: Vec2, selection: &[u64]) -> Option<u64> {
        let mut under = self
            .layers
            .iter()
            .rev()
            .filter(|l| l.visible && !l.locked)
            .flat_map(|l| l.objects.iter().rev())
            .filter(|o| o.is_image() && o.hit(p, 0.0))
            .map(|o| o.id);
        let top = under.next()?;
        Some(if selection.contains(&top) { top } else { under.find(|id| selection.contains(id)).unwrap_or(top) })
    }

    /// Finds the top-most editable object under `p`.
    /// Every object under a point, topmost first.
    pub fn hits_at(&self, p: Vec2, tolerance: f32) -> Vec<u64> {
        self.layers.iter().rev().filter(|l| l.visible && !l.locked).flat_map(|l| l.objects.iter().rev()).filter(|o| o.hit(p, tolerance)).map(|o| o.id).collect()
    }

    pub fn hit_test(&self, p: Vec2, tolerance: f32) -> Option<u64> {
        self.layers
            .iter()
            .rev()
            .filter(|l| l.visible && !l.locked)
            .flat_map(|l| l.objects.iter().rev())
            .find(|o| o.hit(p, tolerance))
            .map(|o| o.id)
    }
}

/// A reusable set of fill, stroke and opacity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub name: String,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
    pub opacity: f32,
    /// The live filters the style gives an object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<crate::effects::LiveFilter>,
    /// The gradient or pattern the style fills an object with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_style: Option<Arc<FillStyle>>,
}

pub fn default_styles() -> Vec<Style> {
    let style = |name: &str, fill: Option<Color>, stroke: Option<(Color, f32)>, opacity: f32| Style {
        name: name.into(),
        fill,
        stroke: stroke.map(|(color, width)| Stroke { color, width }),
        opacity,
        filters: Vec::new(),
        fill_style: None,
    };
    let ink = Color::rgb(0x22, 0x22, 0x22);
    let blue = Color::rgb(0x1e, 0x78, 0xd2);
    let red = Color::rgb(0xd2, 0x3c, 0x32);
    let green = Color::rgb(0x2e, 0xa0, 0x5a);
    let amber = Color::rgb(0xf0, 0xa8, 0x20);
    let gray = Color::rgb(0xcc, 0xcc, 0xcc);
    vec![
        style(t("Solid Ink"), Some(ink), None, 1.0),
        style(t("Solid Blue"), Some(blue), None, 1.0),
        style(t("Solid Red"), Some(red), None, 1.0),
        style(t("Solid Green"), Some(green), None, 1.0),
        style(t("Solid Amber"), Some(amber), None, 1.0),
        style(t("Paper"), Some(Color::WHITE), Some((gray, 1.0)), 1.0),
        style(t("Outline Ink"), None, Some((ink, 2.0)), 1.0),
        style(t("Outline Blue"), None, Some((blue, 2.0)), 1.0),
        style(t("Heavy Outline"), None, Some((ink, 6.0)), 1.0),
        style(t("Tint Blue"), Some(blue), None, 0.35),
        style(t("Tint Red"), Some(red), None, 0.35),
        style(t("Blue Panel"), Some(Color::rgb(0xdc, 0xeb, 0xfa)), Some((blue, 1.0)), 1.0),
    ]
}

/// A reusable group of objects, stored relative to the top-left of its content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub id: u64,
    pub name: String,
    pub objects: Vec<Object>,
    #[serde(default)]
    pub kind: SymbolKind,
    /// The other looks of a button, after the Up look in `objects`: Over, Down and Over
    /// While Down. They are placed against the Up look, so a look may start off its corner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub looks: Vec<Vec<Object>>,
}

/// The four looks of a button, as Fireworks names them.
pub const BUTTON_LOOKS: [&str; 4] = ["Up", "Over", "Down", "Over While Down"];

/// The name of a look of a button, in the language of the interface.
pub fn button_look_name(look: usize) -> &'static str {
    crate::i18n::tc("button", BUTTON_LOOKS[look.min(3)])
}

fn is_zero(count: &u16) -> bool {
    *count == 0
}

/// The numbers of plays the looping list offers; 0 stands for "for ever".
pub const GIF_LOOPS: [u16; 9] = [0, 1, 2, 3, 4, 5, 10, 20, 50];

/// How a number of plays is named in the looping list.
pub fn gif_loop_name(loops: u16) -> String {
    match loops {
        0 => t("Forever").to_string(),
        1 => t("No Looping").to_string(),
        times => tf("{} times", &[&times]),
    }
}

/// The other looks of a button symbol, as the exporter and the preview need them.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonLooks {
    pub symbol: u64,
    /// The box of the Up look, which the other looks are placed against.
    pub natural: Rect,
    /// Over, Down and Over While Down.
    pub looks: Vec<Arc<Vec<Object>>>,
}

/// A symbol being edited in place: its objects lie loose on the page, and everything
/// drawn there until the edit ends becomes part of the symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolEdit {
    pub symbol: u64,
    /// The copy of the symbol that was opened, which goes back when the edit ends.
    pub shell: Option<Object>,
    /// The layer and the place in it where the symbol's objects were put.
    pub spot: (usize, usize),
    /// A symbol made empty gets its first copy on the page when the edit ends.
    pub place: bool,
    /// The objects of the page that are not the symbol's.
    pub outside: Vec<u64>,
    /// Those of them that were locked for the edit, and are unlocked after it.
    pub locked: Vec<u64>,
    /// Where the symbol lay when the edit began.
    pub frame: Rect,
    /// Which look of a button is on the page.
    pub look: usize,
    /// The looks that are not on the page now.
    pub looks: [Vec<Object>; 4],
}

/// The masks Auto Vector Mask offers: a name, the gradient, and where on the object it
/// starts and ends. The object shows where the gradient starts and fades toward where it ends.
pub const AUTO_MASKS: [(&str, FillKind, (f32, f32), (f32, f32)); 7] = [
    ("Linear, fading downward", FillKind::Linear, (0.5, 0.0), (0.5, 1.0)),
    ("Linear, fading upward", FillKind::Linear, (0.5, 1.0), (0.5, 0.0)),
    ("Linear, fading to the right", FillKind::Linear, (0.0, 0.5), (1.0, 0.5)),
    ("Linear, fading to the left", FillKind::Linear, (1.0, 0.5), (0.0, 0.5)),
    ("Ellipse", FillKind::Ellipse, (0.5, 0.5), (1.0, 0.5)),
    ("Rectangle", FillKind::Rectangle, (0.5, 0.5), (1.0, 0.5)),
    ("Starburst", FillKind::Starburst, (0.5, 0.5), (1.0, 0.5)),
];

/// What Twist and Fade asks: how many copies, and how each differs from the one before it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwistSettings {
    pub steps: u32,
    /// How far each copy is from the one before, in pixels, and which way, in degrees: 0 is to the right, 90 is up.
    pub spacing: f32,
    pub direction: f32,
    /// How far each copy is turned from the one before, in degrees clockwise.
    pub rotation: f32,
    /// The size of each copy, in percent of the one before.
    pub scale: f32,
    /// The opacity the last copy ends at, in percent; the copies fade evenly toward it.
    pub fade_to: f32,
}

impl Default for TwistSettings {
    fn default() -> Self {
        Self { steps: 8, spacing: 15.0, direction: 0.0, rotation: 15.0, scale: 95.0, fade_to: 10.0 }
    }
}

/// A motion tween: how the objects of a keyframe go over to those of the next one.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tween {
    /// From -100, which starts slowly, to 100, which ends slowly; 0 goes evenly.
    #[serde(default)]
    pub ease: f32,
    /// Whole turns made on the way, on top of the turn from one keyframe to the next; clockwise if positive.
    #[serde(default)]
    pub turns: i32,
    /// A shape tween: the outlines and colors of the keyframe's shapes change into those of the next keyframe's.
    #[serde(default)]
    pub shape: bool,
    /// Along a guide, whether the object turns to face the way the path runs.
    #[serde(default)]
    pub orient: bool,
    /// Shape hints: places on the outline of the keyframe's first shape, each with the
    /// place on the next keyframe's first shape that it is to end up at.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<(Vec2, Vec2)>,
}

/// What a layer of a timeline is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerKind {
    #[default]
    Normal,
    /// Holds a path that the tweened objects of the layer beneath it travel along. It is not part of the finished animation.
    Guide,
    /// The layer beneath it shows only where this layer has something drawn.
    Mask,
}

impl Tween {
    /// How far along the way a frame is, once the easing has had its say, for a part of the span from 0 to 1.
    pub fn eased(&self, part: f32) -> f32 {
        let strength = (self.ease.abs() / 100.0).min(1.0);
        let curved = if self.ease > 0.0 { 1.0 - (1.0 - part) * (1.0 - part) } else { part * part };
        part + (curved - part) * strength
    }
}

/// What a frame of the timeline, or a button, does when the finished animation reaches it or it is clicked.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Stop,
    Play,
    /// Goes to a frame of the scene, counted from 1, and plays on from there.
    GotoPlay(usize),
    /// Goes to a frame of the scene, counted from 1, and stops there.
    GotoStop(usize),
    /// Goes to the first frame of a scene, counted from 1, and plays.
    Scene(usize),
    /// Opens an address.
    Url(String),
}

impl Action {
    /// Reads an action as it is kept in a document. Anything that is not one of the others is an address.
    pub fn parse(text: &str) -> Option<Action> {
        let text = text.trim();
        let number = |rest: &str| rest.parse::<usize>().ok().filter(|n| *n >= 1);
        Some(match text {
            "" => return None,
            "action:stop" => Action::Stop,
            "action:play" => Action::Play,
            _ => {
                if let Some(rest) = text.strip_prefix("action:goto-play:") {
                    Action::GotoPlay(number(rest)?)
                } else if let Some(rest) = text.strip_prefix("action:goto-stop:") {
                    Action::GotoStop(number(rest)?)
                } else if let Some(rest) = text.strip_prefix("action:scene:") {
                    Action::Scene(number(rest)?)
                } else if text.starts_with("action:") {
                    return None;
                } else {
                    Action::Url(text.to_string())
                }
            }
        })
    }

    /// The action as it is kept in a document.
    pub fn text(&self) -> String {
        match self {
            Action::Stop => "action:stop".into(),
            Action::Play => "action:play".into(),
            Action::GotoPlay(frame) => format!("action:goto-play:{frame}"),
            Action::GotoStop(frame) => format!("action:goto-stop:{frame}"),
            Action::Scene(scene) => format!("action:scene:{scene}"),
            Action::Url(address) => address.clone(),
        }
    }

    /// Which entry of the list of actions this is: 0 none, then in the order the list shows them.
    pub fn choice(action: &Option<Action>) -> usize {
        match action {
            None => 0,
            Some(Action::Stop) => 1,
            Some(Action::Play) => 2,
            Some(Action::GotoPlay(_)) => 3,
            Some(Action::GotoStop(_)) => 4,
            Some(Action::Scene(_)) => 5,
            Some(Action::Url(_)) => 6,
        }
    }

    /// The action an entry of the list stands for, keeping the number or address of the one before where it fits.
    pub fn chosen(choice: usize, before: &Option<Action>) -> Option<Action> {
        let number = match before {
            Some(Action::GotoPlay(n) | Action::GotoStop(n) | Action::Scene(n)) => *n,
            _ => 1,
        };
        match choice {
            1 => Some(Action::Stop),
            2 => Some(Action::Play),
            3 => Some(Action::GotoPlay(number)),
            4 => Some(Action::GotoStop(number)),
            5 => Some(Action::Scene(number)),
            6 => Some(Action::Url(match before {
                Some(Action::Url(address)) => address.clone(),
                _ => "https://".into(),
            })),
            _ => None,
        }
    }
}

/// The names of the actions, in the order of [`Action::choice`].
pub const ACTION_NAMES: [&str; 7] = ["None", "Stop", "Play", "Go to Frame and Play", "Go to Frame and Stop", "Go to Scene", "Go to Address"];

/// A sound kept in the document: a file as it was imported.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sound {
    pub id: u64,
    pub name: String,
    /// What kind of sound file it is, as a browser is told: audio/mpeg, audio/ogg, audio/wav.
    pub mime: String,
    /// The file, in base64, as it goes into a published page.
    pub data: Arc<String>,
}

/// A keyframe and the frames that follow it until the next keyframe.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Span {
    /// The frame the keyframe is on, counted from 0.
    pub start: usize,
    /// How many frames it lasts, the keyframe included.
    pub length: usize,
    /// What the keyframe holds, bottom-most first; nothing for a blank keyframe.
    pub objects: Vec<Object>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tween: Option<Tween>,
    /// What the finished animation does on reaching the keyframe, written as [`Action::text`] writes it; empty for nothing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub action: String,
    /// The sound that starts on the keyframe, by its number in the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<u64>,
}

/// The timeline of a scene: for each layer of the page, its keyframes one after
/// another from frame 0. The objects of the frame on show lie in the page's layers,
/// where every tool works on them; the timeline has the other frames.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    /// Frames a second.
    pub fps: f32,
    /// The frame on show.
    pub current: usize,
    /// For each layer of the page, bottom-most first.
    pub layers: Vec<Vec<Span>>,
    /// What each layer is for; a layer not listed is an ordinary one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<LayerKind>,
    /// Which layers are tied to the guide or mask above them; a layer not listed is not.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bound: Vec<bool>,
}

/// How many points an outline is given for a shape tween.
const MORPH_POINTS: usize = 64;

/// An outline as evenly spaced points: `count` of them, the same distance apart along it.
fn resampled(outline: &[Vec2], closed: bool, count: usize) -> Vec<Vec2> {
    let mut line = outline.to_vec();
    if closed {
        line.extend(outline.first().copied());
    }
    if line.len() < 2 {
        return vec![line.first().copied().unwrap_or_default(); count];
    }
    let lengths: Vec<f32> = line.windows(2).map(|pair| (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y)).collect();
    let total: f32 = lengths.iter().sum();
    let steps = if closed { count } else { count - 1 };
    let mut points = Vec::with_capacity(count);
    let (mut piece, mut before) = (0, 0.0);
    for index in 0..count {
        let at = total * index as f32 / steps.max(1) as f32;
        while piece + 1 < lengths.len() && before + lengths[piece] < at {
            before += lengths[piece];
            piece += 1;
        }
        let part = if lengths[piece] > 0.0 { ((at - before) / lengths[piece]).clamp(0.0, 1.0) } else { 0.0 };
        points.push(v2(line[piece].x + (line[piece + 1].x - line[piece].x) * part, line[piece].y + (line[piece + 1].y - line[piece].y) * part));
    }
    points
}

/// A closed outline as `count` evenly spaced points, for laying something along it.
pub fn outline_points(outline: &[Vec2], count: usize) -> Vec<Vec2> {
    if outline.len() < 2 {
        return outline.to_vec();
    }
    resampled(outline, true, count)
}

fn mixed_color(from: Color, to: Color, part: f32) -> Color {
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * part).round().clamp(0.0, 255.0) as u8;
    Color { r: mix(from.r, to.r), g: mix(from.g, to.g), b: mix(from.b, to.b), a: mix(from.a, to.a) }
}

/// A shape on its way to becoming another: its outline, colors and stroke part of the way there.
fn morphed(from: &Object, to: &Object, part: f32, hints: &[(Vec2, Vec2)]) -> Object {
    let drawn = |object: &Object| matches!(object.shape, Shape::Rect { .. } | Shape::Ellipse | Shape::Line | Shape::Path { .. } | Shape::Auto { .. });
    if !drawn(from) || !drawn(to) {
        // What has no outline of its own to change, such as text or a copy of a symbol, moves as in a motion tween.
        return between(from, to, part, 0);
    }
    let closed = from.is_closed() && to.is_closed();
    let a = resampled(&from.outline(), from.is_closed(), MORPH_POINTS);
    let mut b = resampled(&to.outline(), to.is_closed(), MORPH_POINTS);
    // The index of the outline's point nearest a place.
    let nearest = |points: &[Vec2], place: Vec2| (0..points.len()).min_by(|x, y| (points[*x].x - place.x).hypot(points[*x].y - place.y).total_cmp(&(points[*y].x - place.x).hypot(points[*y].y - place.y))).unwrap_or(0);
    if closed && !hints.is_empty() {
        // With hints the second outline is begun where the first hint says, and gone round the way that
        // brings the other hints nearest their places; without a second hint, the way of the shortest journeys.
        let (start_a, start_b) = (nearest(&a, hints[0].0), nearest(&b, hints[0].1));
        let mut a_turned = a.clone();
        a_turned.rotate_left(start_a);
        let turned = |b: &[Vec2], backward: bool| {
            let mut b = b.to_vec();
            if backward {
                b.reverse();
            }
            let at = nearest(&b, hints[0].1);
            b.rotate_left(at);
            b
        };
        let _ = start_b;
        let cost = |b: &[Vec2]| -> f32 {
            if hints.len() > 1 {
                hints[1..].iter().map(|(on_a, on_b)| {
                    let (i, j) = (nearest(&a_turned, *on_a) as f32, nearest(b, *on_b) as f32);
                    (i - j).abs()
                }).sum()
            } else {
                (0..MORPH_POINTS).map(|i| (a_turned[i].x - b[i].x).powi(2) + (a_turned[i].y - b[i].y).powi(2)).sum()
            }
        };
        let (forth, back) = (turned(&b, false), turned(&b, true));
        b = if cost(&back) < cost(&forth) { back } else { forth };
        let points: Vec<Vec2> = a_turned.iter().zip(&b).map(|(p, q)| v2(p.x + (q.x - p.x) * part, p.y + (q.y - p.y) * part)).collect();
        return finish_morph(from, to, points, closed, part);
    }
    if closed {
        // The second outline is begun at the point, and gone round the way, that keeps every point's journey shortest.
        let journey = |b: &[Vec2], shift: usize| -> f32 { (0..MORPH_POINTS).map(|i| (a[i].x - b[(i + shift) % MORPH_POINTS].x).powi(2) + (a[i].y - b[(i + shift) % MORPH_POINTS].y).powi(2)).sum() };
        let mut backward = b.clone();
        backward.reverse();
        let best = |b: &[Vec2]| (0..MORPH_POINTS).map(|shift| (journey(b, shift), shift)).min_by(|x, y| x.0.total_cmp(&y.0)).unwrap_or((0.0, 0));
        let (forth, back) = (best(&b), best(&backward));
        if back.0 < forth.0 {
            b = backward;
        }
        b.rotate_left(if back.0 < forth.0 { back.1 } else { forth.1 });
    }
    let points: Vec<Vec2> = a.iter().zip(&b).map(|(p, q)| v2(p.x + (q.x - p.x) * part, p.y + (q.y - p.y) * part)).collect();
    finish_morph(from, to, points, closed, part)
}

/// The path a shape tween shows for an outline part of the way, with colors and stroke part of the way too.
fn finish_morph(from: &Object, to: &Object, points: Vec<Vec2>, closed: bool, part: f32) -> Object {
    let nearer = if part < 0.5 { from } else { to };
    let mut object = from.clone();
    (object.placement, object.handles, object.stroke_profile) = (None, Vec::new(), Vec::new());
    object.shape = Shape::Path { closed };
    object.bounds = Rect::of_points(&points);
    object.points = points;
    object.fill = match (from.fill, to.fill) {
        (Some(x), Some(y)) => Some(mixed_color(x, y, part)),
        _ => nearer.fill,
    };
    object.stroke = match (from.stroke, to.stroke) {
        (Some(x), Some(y)) => Some(Stroke { color: mixed_color(x.color, y.color, part), width: x.width + (y.width - x.width) * part }),
        _ => nearer.stroke,
    };
    object.fill_style = nearer.fill_style.clone();
    object.opacity = from.opacity + (to.opacity - from.opacity) * part;
    object
}

/// Where a place lies along a path, as a distance from its start: the nearest point of the path to it.
fn along_path(path: &[Vec2], place: Vec2) -> f32 {
    let (mut best, mut at, mut before) = (f32::MAX, 0.0, 0.0);
    for pair in path.windows(2) {
        let (dx, dy) = (pair[1].x - pair[0].x, pair[1].y - pair[0].y);
        let length = dx.hypot(dy);
        let part = if length > 0.0 { (((place.x - pair[0].x) * dx + (place.y - pair[0].y) * dy) / (length * length)).clamp(0.0, 1.0) } else { 0.0 };
        let distance = (pair[0].x + dx * part - place.x).hypot(pair[0].y + dy * part - place.y);
        if distance < best {
            (best, at) = (distance, before + length * part);
        }
        before += length;
    }
    at
}

/// Which way a path runs a distance along it, in radians clockwise from pointing right.
fn direction_along(path: &[Vec2], distance: f32) -> f32 {
    let mut before = 0.0;
    let mut last = 0.0;
    for pair in path.windows(2) {
        let (dx, dy) = (pair[1].x - pair[0].x, pair[1].y - pair[0].y);
        let length = dx.hypot(dy);
        if length > 0.0 {
            last = dy.atan2(dx);
            if before + length >= distance {
                return last;
            }
        }
        before += length;
    }
    last
}

/// The point of a path a distance along it.
fn point_along(path: &[Vec2], distance: f32) -> Vec2 {
    let mut before = 0.0;
    for pair in path.windows(2) {
        let length = (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
        if before + length >= distance && length > 0.0 {
            let part = ((distance - before) / length).clamp(0.0, 1.0);
            return v2(pair[0].x + (pair[1].x - pair[0].x) * part, pair[0].y + (pair[1].y - pair[0].y) * part);
        }
        before += length;
    }
    path.last().copied().unwrap_or_default()
}

/// The most frames a timeline may have.
pub const MAX_FRAMES: usize = 20_000;

/// How one outline has been turned, resized and moved to give another, if it has been
/// and nothing else: the turn in radians clockwise, the change of size, and the middle
/// of each. A shape turned by a command that redraws it as a path (Rotate 90°, a flip of
/// a symmetrical shape) carries no record of the turn; its outline still tells it.
fn turned_between(from: &[Vec2], to: &[Vec2]) -> Option<(f32, f32, Vec2, Vec2)> {
    // Two points, a line's, are enough to tell a turn.
    let count = from.len();
    if count < 2 || count != to.len() {
        return None;
    }
    let middle = |points: &[Vec2]| v2(points.iter().map(|p| p.x).sum::<f32>() / count as f32, points.iter().map(|p| p.y).sum::<f32>() / count as f32);
    let (ca, cb) = (middle(from), middle(to));
    let spread = |points: &[Vec2], c: Vec2| points.iter().map(|p| (p.x - c.x).powi(2) + (p.y - c.y).powi(2)).sum::<f32>();
    let (sa, sb) = (spread(from, ca), spread(to, cb));
    if sa <= 0.0 || sb <= 0.0 {
        return None;
    }
    let scale = (sb / sa).sqrt();
    // The corners of a redrawn shape may be numbered from another corner: each start is tried, and the best fit kept.
    let shifts = if count <= 256 { count } else { 1 };
    let mut best: Option<(f32, f32)> = None;
    for shift in 0..shifts {
        let (mut along, mut across) = (0.0, 0.0);
        for (index, p) in from.iter().enumerate() {
            let q = to[(index + shift) % count];
            let (ax, ay, bx, by) = (p.x - ca.x, p.y - ca.y, q.x - cb.x, q.y - cb.y);
            along += ax * bx + ay * by;
            across += ax * by - ay * bx;
        }
        let angle = across.atan2(along);
        // How far the points still are from their partners once turned and resized, against the size of the shape.
        let (sin, cos) = angle.sin_cos();
        let off: f32 = from
            .iter()
            .enumerate()
            .map(|(index, p)| {
                let q = to[(index + shift) % count];
                let (ax, ay) = ((p.x - ca.x) * scale, (p.y - ca.y) * scale);
                (ax * cos - ay * sin - (q.x - cb.x)).powi(2) + (ax * sin + ay * cos - (q.y - cb.y)).powi(2)
            })
            .sum::<f32>()
            / sb;
        // Point for point as they are numbered is the truth when it fits: a shape with many like sides, such as a
        // gear, fits from other starts as well, with a turn it never made.
        if shift == 0 && off < 1e-3 {
            return Some((angle, scale, ca, cb));
        }
        if best.is_none_or(|(least, _)| off < least - 1e-4) {
            best = Some((off, angle));
        }
    }
    best.filter(|(off, _)| *off < 1e-3).map(|(_, angle)| (angle, scale, ca, cb))
}

/// An object on its way from how one keyframe has it to how the next has it. It turns around its
/// transformation point, as in Flash: that point goes straight from where the first keyframe has it to where
/// the next has it, and the object turns and changes size around it. So an arm whose point has been put on
/// the shoulder stays on the shoulder.
fn between(from: &Object, to: &Object, part: f32, turns: i32) -> Object {
    let mut object = between_middles(from, to, part, turns);
    if let Some(at) = from.pivot {
        let (a, b, now) = (from.anchored(at), to.anchored(at), object.anchored(at));
        let (dx, dy) = (a.x + (b.x - a.x) * part - now.x, a.y + (b.y - a.y) * part - now.y);
        if dx.abs() > 1e-4 || dy.abs() > 1e-4 {
            object.translate(dx, dy);
        }
    }
    object
}

/// The same with the middle of the object going straight from keyframe to keyframe.
fn between_middles(from: &Object, to: &Object, part: f32, turns: i32) -> Object {
    // A shape that was turned between the keyframes without a record of it, or is asked to make whole turns, is
    // carried round by what its outline shows: turned, resized and moved a part of the way.
    if from.placement.is_none() && to.placement.is_none() && !matches!(from.shape, Shape::Text { .. } | Shape::Instance { .. } | Shape::Group { .. } | Shape::Image { .. }) {
        if let Some((angle, scale, ca, cb)) = turned_between(&from.outline(), &to.outline()) {
            if angle.abs() > 0.001 {
                let sweep = angle + turns as f32 * std::f32::consts::TAU;
                let center = v2(ca.x + (cb.x - ca.x) * part, ca.y + (cb.y - ca.y) * part);
                let size = 1.0 + (scale - 1.0) * part;
                let carry = Warp::fit(Rect { x: ca.x - 1.0, y: ca.y - 1.0, w: 2.0, h: 2.0 }, Rect { x: center.x - size, y: center.y - size, w: 2.0 * size, h: 2.0 * size }).then(&Warp::rotation(center, sweep * part));
                let mut object = from.clone();
                if object.warp(&carry) {
                    object.opacity = from.opacity + (to.opacity - from.opacity) * part;
                    return object;
                }
            }
        }
    }
    let pose = |object: &Object| {
        let frame = object.placement.map_or(object.bounds, |p| p.frame);
        (object.bounds.center(), frame.w, frame.h, object.placement.map_or(0.0, |p| p.warp.turn()), object.opacity)
    };
    let (a, b) = (pose(from), pose(to));
    let mut sweep = b.3 - a.3;
    if sweep > std::f32::consts::PI {
        sweep -= std::f32::consts::TAU;
    } else if sweep < -std::f32::consts::PI {
        sweep += std::f32::consts::TAU;
    }
    sweep += turns as f32 * std::f32::consts::TAU;
    let mix = |x: f32, y: f32| x + (y - x) * part;
    let center = v2(mix(a.0.x, b.0.x), mix(a.0.y, b.0.y));
    let (w, h) = (mix(a.1, b.1), mix(a.2, b.2));
    let mut object = from.clone();
    object.placement = None;
    // What was turned is laid straight again before it is given its new size and turn.
    if let Some(placement) = from.placement {
        object.bounds = placement.frame;
    }
    object.set_bounds(Rect { x: center.x - w / 2.0, y: center.y - h / 2.0, w, h });
    let angle = a.3 + sweep * part;
    if angle.abs() > 0.0001 {
        object.warp(&Warp::rotation(center, angle));
    }
    object.opacity = mix(a.4, b.4);
    object
}

impl Timeline {
    /// How many frames the scene has: as many as its longest layer.
    pub fn frame_count(&self) -> usize {
        self.layers.iter().filter_map(|spans| spans.last().map(|span| span.start + span.length)).max().unwrap_or(0)
    }

    /// Which span of a layer a frame falls in.
    pub fn span_at(&self, layer: usize, frame: usize) -> Option<usize> {
        self.layers.get(layer)?.iter().position(|span| (span.start..span.start + span.length).contains(&frame))
    }

    /// Whether a frame of a layer is a keyframe.
    pub fn is_keyframe(&self, layer: usize, frame: usize) -> bool {
        self.layers.get(layer).is_some_and(|spans| spans.iter().any(|span| span.start == frame))
    }

    /// What a layer shows on a frame: its keyframe's objects, or, in a tween, objects on their way to the next keyframe's.
    pub fn objects_at(&self, layer: usize, frame: usize) -> Vec<Object> {
        let Some(index) = self.span_at(layer, frame) else { return Vec::new() };
        let spans = &self.layers[layer];
        let span = &spans[index];
        let (Some(tween), Some(next)) = (span.tween.as_ref(), spans.get(index + 1)) else { return span.objects.clone() };
        if frame == span.start {
            return span.objects.clone();
        }
        let part = tween.eased((frame - span.start) as f32 / span.length as f32);
        if tween.shape {
            // Shapes change into those of the next keyframe one for one, in the order they lie in.
            // A shape with no partner fades away, and one that appears only in the next keyframe fades in.
            let mut shown: Vec<Object> = span
                .objects
                .iter()
                .enumerate()
                .map(|(index, object)| match next.objects.get(index) {
                    // The hints belong to the first shape of the keyframe.
                    Some(other) => morphed(object, other, part, if index == 0 { &tween.hints } else { &[] }),
                    None => Object { opacity: object.opacity * (1.0 - part), ..object.clone() },
                })
                .collect();
            shown.extend(next.objects.iter().skip(span.objects.len()).map(|object| Object { opacity: object.opacity * part, ..object.clone() }));
            return shown;
        }
        // The path of a guide layer lying on this one, if there is one: the first thing drawn there.
        let guide: Option<Vec<Vec2>> = self
            .master(layer)
            .filter(|(_, kind)| *kind == LayerKind::Guide)
            .and_then(|(above, _)| self.objects_at(above, frame).into_iter().map(|object| object.outline()).find(|line| line.len() >= 2));
        // An object goes over to the one with its number in the next keyframe; one that has none there stays as it is.
        span.objects
            .iter()
            .map(|object| {
                let Some(other) = next.objects.iter().find(|other| other.id == object.id) else { return object.clone() };
                let mut moved = between(object, other, part, tween.turns);
                if let Some(path) = &guide {
                    // Along a guide the object keeps to the path, from the point nearest where it starts to the one nearest where it ends.
                    let (start, end) = (along_path(path, object.bounds.center()), along_path(path, other.bounds.center()));
                    let at = start + (end - start) * part;
                    let place = point_along(path, at);
                    let center = moved.bounds.center();
                    moved.translate(place.x - center.x, place.y - center.y);
                    if tween.orient {
                        // Facing the way it travels: turned as far as the path is turned there, going back if it runs backward.
                        let heading = direction_along(path, at) + if end < start { std::f32::consts::PI } else { 0.0 };
                        moved.warp(&Warp::rotation(place, heading));
                    }
                }
                moved
            })
            .collect()
    }

    /// What a layer is for.
    pub fn kind(&self, layer: usize) -> LayerKind {
        self.kinds.get(layer).copied().unwrap_or_default()
    }

    /// Makes a layer an ordinary one, a guide or a mask. A guide or mask takes the layer
    /// beneath it under itself, as in Flash; made ordinary again, it lets go of the layers it had.
    pub fn set_kind(&mut self, layer: usize, kind: LayerKind) {
        if layer >= self.layers.len() {
            return;
        }
        if self.kinds.len() <= layer {
            self.kinds.resize(layer + 1, LayerKind::Normal);
        }
        self.kinds[layer] = kind;
        if kind == LayerKind::Normal {
            let mut below = layer;
            while below > 0 && self.is_bound(below - 1) {
                below -= 1;
                self.set_bound(below, false);
            }
        } else if layer > 0 {
            self.set_bound(layer, false);
            self.set_bound(layer - 1, true);
        }
    }

    /// Whether a layer is tied to the guide or mask above it.
    pub fn is_bound(&self, layer: usize) -> bool {
        self.bound.get(layer).copied().unwrap_or(false)
    }

    /// Ties a layer to the guide or mask above it, or lets it go. Only an ordinary layer
    /// that lies right under a guide, a mask, or a layer already tied to one can be tied.
    pub fn set_bound(&mut self, layer: usize, bound: bool) -> bool {
        if layer >= self.layers.len() || (bound && (self.kind(layer) != LayerKind::Normal || (self.kind(layer + 1) == LayerKind::Normal && !self.is_bound(layer + 1)) || layer + 1 >= self.layers.len())) {
            return false;
        }
        if self.bound.len() <= layer {
            self.bound.resize(layer + 1, false);
        }
        self.bound[layer] = bound;
        // Let go, a layer takes the ones tied through it along.
        if !bound {
            let mut below = layer;
            while below > 0 && self.is_bound(below - 1) && self.kind(below - 1) == LayerKind::Normal {
                below -= 1;
                self.bound[below] = false;
            }
        }
        true
    }

    /// The guide or mask a layer is tied to: its place and which of the two it is.
    pub fn master(&self, layer: usize) -> Option<(usize, LayerKind)> {
        if !self.is_bound(layer) || self.kind(layer) != LayerKind::Normal {
            return None;
        }
        let mut above = layer + 1;
        while above < self.layers.len() && self.kind(above) == LayerKind::Normal && self.is_bound(above) {
            above += 1;
        }
        (above < self.layers.len() && self.kind(above) != LayerKind::Normal).then(|| (above, self.kind(above)))
    }

    /// Puts the starts of a layer's spans in line with their lengths again.
    fn renumber(spans: &mut [Span]) {
        let mut at = 0;
        for span in spans {
            span.start = at;
            at += span.length;
        }
    }

    /// Makes a layer reach a frame, by letting its last keyframe last longer.
    fn reach(&mut self, layer: usize, frame: usize) {
        let Some(spans) = self.layers.get_mut(layer) else { return };
        match spans.last_mut() {
            Some(last) if last.start + last.length <= frame => last.length = frame + 1 - last.start,
            Some(_) => {}
            None => spans.push(Span { start: 0, length: frame + 1, objects: Vec::new(), ..Default::default() }),
        }
    }

    /// Flash's Insert Frame: the keyframe a frame belongs to lasts longer by `count`
    /// frames, and what follows moves later. Past the end, the layer is made to reach the frame.
    pub fn insert_frames(&mut self, layer: usize, frame: usize, count: usize) {
        if self.frame_count() + count > MAX_FRAMES || frame >= MAX_FRAMES {
            return;
        }
        match self.span_at(layer, frame) {
            Some(index) => {
                self.layers[layer][index].length += count;
                Self::renumber(&mut self.layers[layer]);
            }
            None => self.reach(layer, frame),
        }
    }

    /// Flash's Remove Frames: the layer loses a frame. A keyframe that has only its own frame goes with it, unless it is the layer's last.
    pub fn remove_frame(&mut self, layer: usize, frame: usize) {
        let Some(index) = self.span_at(layer, frame) else { return };
        let spans = &mut self.layers[layer];
        if spans[index].length > 1 {
            spans[index].length -= 1;
        } else if spans.len() > 1 {
            spans.remove(index);
        }
        Self::renumber(spans);
    }

    /// Flash's Insert Keyframe and Insert Blank Keyframe: a keyframe on a frame that has none.
    /// It starts with what the layer shows there, or, blank, with nothing.
    pub fn insert_keyframe(&mut self, layer: usize, frame: usize, blank: bool) -> bool {
        if frame >= MAX_FRAMES || layer >= self.layers.len() || self.is_keyframe(layer, frame) {
            return false;
        }
        self.reach(layer, frame);
        let objects = if blank { Vec::new() } else { self.objects_at(layer, frame) };
        let Some(index) = self.span_at(layer, frame) else { return false };
        let spans = &mut self.layers[layer];
        let rest = spans[index].start + spans[index].length - frame;
        spans[index].length -= rest;
        // A tween goes on to the new keyframe, and from it to where it went before.
        let tween = spans[index].tween.clone().filter(|_| !blank);
        spans.insert(index + 1, Span { start: frame, length: rest, objects, tween, ..Default::default() });
        true
    }

    /// Flash's Clear Keyframe: the keyframe becomes ordinary frames of the one before it. The first keyframe stays.
    pub fn clear_keyframe(&mut self, layer: usize, frame: usize) -> bool {
        let Some(index) = self.span_at(layer, frame).filter(|index| *index > 0 && self.layers[layer][*index].start == frame) else { return false };
        let spans = &mut self.layers[layer];
        let removed = spans.remove(index);
        spans[index - 1].length += removed.length;
        true
    }

    /// Sets or removes the tween of the keyframe a frame belongs to.
    pub fn set_tween(&mut self, layer: usize, frame: usize, tween: Option<Tween>) -> bool {
        let Some(index) = self.span_at(layer, frame) else { return false };
        self.layers[layer][index].tween = tween;
        true
    }
}

/// What Fireworks' Animate dialog asks.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSettings {
    pub states: u32,
    /// How far the object travels, in pixels.
    pub distance: f32,
    /// The way it travels, in degrees: 0 is to the right, 90 is up.
    pub direction: f32,
    /// The size it ends at, in percent of the size it starts at.
    pub scale: f32,
    pub opacity_from: f32,
    pub opacity_to: f32,
    /// How far it turns on the way, in degrees.
    pub rotation: f32,
    pub clockwise: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self { states: 5, distance: 72.0, direction: 0.0, scale: 100.0, opacity_from: 100.0, opacity_to: 100.0, rotation: 0.0, clockwise: true }
    }
}

impl AnimationSettings {
    /// Where the middle of the object is after `step` of the states, starting from `start`.
    pub fn center(&self, start: Vec2, step: usize) -> Vec2 {
        let along = self.along(step) * self.distance;
        let angle = self.direction.to_radians();
        v2(start.x + along * angle.cos(), start.y - along * angle.sin())
    }

    /// How much of the way is behind the object at a state: 0 at the first, 1 at the last.
    fn along(&self, step: usize) -> f32 {
        if self.states < 2 { 0.0 } else { step as f32 / (self.states - 1) as f32 }
    }

    /// The symbol as a state shows it: moved, resized, turned and faded from how it starts.
    fn pose(&self, start: &Object, step: usize) -> Object {
        let mut object = start.clone();
        let along = self.along(step);
        let size = 1.0 + (self.scale / 100.0 - 1.0) * along;
        let center = self.center(start.bounds.center(), step);
        let (w, h) = (start.bounds.w * size, start.bounds.h * size);
        object.set_bounds(Rect { x: center.x - w / 2.0, y: center.y - h / 2.0, w, h });
        let turn = self.rotation.to_radians() * along * if self.clockwise { 1.0 } else { -1.0 };
        if turn != 0.0 {
            object.warp(&Warp::rotation(center, turn));
        }
        object.opacity = ((self.opacity_from + (self.opacity_to - self.opacity_from) * along) / 100.0).clamp(0.0, 1.0);
        object
    }
}

/// An animation symbol on a page: one copy of the symbol in each of a run of states.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Animation {
    pub symbol: u64,
    pub settings: AnimationSettings,
    /// The state the animation starts in.
    pub first_state: usize,
    pub layer: usize,
    /// The copies, the first state's first.
    pub objects: Vec<u64>,
}

/// What a symbol is for, as Fireworks sorts them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    #[default]
    Graphic,
    Button,
    Animation,
}

impl SymbolKind {
    pub const ALL: [SymbolKind; 3] = [SymbolKind::Graphic, SymbolKind::Animation, SymbolKind::Button];

    pub fn label(self) -> &'static str {
        match self {
            SymbolKind::Graphic => t("Graphic"),
            SymbolKind::Button => t("Button"),
            // In the animation program this kind is a movie clip, with a timeline of its own.
            SymbolKind::Animation if crate::theme::is_sahne() => t("Movie Clip"),
            SymbolKind::Animation => t("Animation"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub pages: Vec<Page>,
    pub active_page: usize,
    pub next_id: u64,
    #[serde(default = "default_styles")]
    pub styles: Vec<Style>,
    #[serde(default)]
    pub symbols: Vec<Symbol>,
    #[serde(default)]
    pub urls: Vec<String>,
    /// The sounds of an animation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sounds: Vec<Sound>,
    /// What makes an animation a game: see [`crate::rules`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<crate::rules::Rule>,
    /// The numbers the rules keep, such as a score.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<crate::rules::Variable>,
    /// The commands defined in the written language: steps under a name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<crate::rules::Command>,
    /// While a movie clip is edited, the page to go back to. Never written to a file.
    #[serde(skip)]
    pub clip_return: Option<usize>,
}

const MAX_SYMBOL_DEPTH: usize = 16;

fn resolve_symbol(symbols: &[Symbol], id: u64, depth: usize) -> Arc<Vec<Object>> {
    let Some(symbol) = symbols.iter().find(|s| s.id == id).filter(|_| depth < MAX_SYMBOL_DEPTH) else {
        return Arc::default();
    };
    let mut parts = symbol.objects.clone();
    for part in &mut parts {
        link_object(symbols, part, depth + 1);
    }
    Arc::new(parts)
}

/// Fills in the contents of a symbol copy, including copies inside groups.
fn link_object(symbols: &[Symbol], object: &mut Object, depth: usize) {
    match &mut object.shape {
        Shape::Instance { symbol, objects } => *objects = resolve_symbol(symbols, *symbol, depth),
        Shape::Group { objects } => {
            for part in Arc::make_mut(objects) {
                link_object(symbols, part, depth);
            }
        }
        _ => {}
    }
}

fn object_uses(symbols: &[Symbol], object: &Object, target: u64, depth: usize) -> bool {
    match &object.shape {
        Shape::Instance { symbol, .. } => symbol_uses(symbols, *symbol, target, depth),
        Shape::Group { objects } => objects.iter().any(|part| object_uses(symbols, part, target, depth)),
        _ => false,
    }
}

/// Removes copies of a symbol from inside a group.
fn strip_symbol(object: &mut Object, id: u64) {
    if let Shape::Group { objects } = &mut object.shape {
        let parts = Arc::make_mut(objects);
        parts.retain(|part| !matches!(part.shape, Shape::Instance { symbol, .. } if symbol == id));
        parts.iter_mut().for_each(|part| strip_symbol(part, id));
    }
}

fn symbol_uses(symbols: &[Symbol], id: u64, target: u64, depth: usize) -> bool {
    id == target
        || (depth < MAX_SYMBOL_DEPTH
            && symbols.iter().find(|s| s.id == id).is_some_and(|symbol| {
                symbol.objects.iter().any(|o| object_uses(symbols, o, target, depth + 1))
            }))
}

/// Copies objects with their combined top-left moved to the origin.
fn normalized(objects: &[Object]) -> Vec<Object> {
    let origin = objects.iter().map(|o| o.bounds).reduce(Rect::union).unwrap_or_default();
    objects
        .iter()
        .map(|o| {
            let mut object = o.clone();
            object.translate(-origin.x, -origin.y);
            object
        })
        .collect()
}

impl Document {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pages: vec![Page::new(tf("Page {}", &[&1]), width, height)],
            active_page: 0,
            next_id: 1,
            styles: default_styles(),
            symbols: Vec::new(),
            urls: Vec::new(),
            sounds: Vec::new(),
            rules: Vec::new(),
            variables: Vec::new(),
            commands: Vec::new(),
            clip_return: None,
        }
    }

    /// The names objects have been given, anywhere in the document: what a rule can be about.
    pub fn object_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .pages
            .iter()
            .flat_map(|page| {
                let kept = page.timeline.iter().flat_map(|timeline| timeline.layers.iter().flatten().flat_map(|span| span.objects.iter()));
                page.objects().chain(kept).filter(|o| !o.is_web() && !o.name.trim().is_empty()).map(|o| o.name.clone())
            })
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// The names worth offering in a list: those of [`Self::object_names`], less the texts that are only
    /// called what they say. A text gets its words for a name by itself; one that was given a name is kept.
    pub fn chosen_names(&self) -> Vec<String> {
        let own_words = |o: &Object| matches!(&o.shape, Shape::Text { content, .. } if content.trim() == o.name.trim());
        let all = |keep: &dyn Fn(&Object) -> bool| -> Vec<String> {
            self.pages
                .iter()
                .flat_map(|page| {
                    let kept = page.timeline.iter().flat_map(|timeline| timeline.layers.iter().flatten().flat_map(|span| span.objects.iter()));
                    page.objects().chain(kept).filter(|o| !o.is_web() && !o.name.trim().is_empty() && keep(o)).map(|o| o.name.clone()).collect::<Vec<_>>()
                })
                .collect()
        };
        // A name that something other than such a text has is kept.
        let mut names = all(&|o| !own_words(o));
        names.sort();
        names.dedup();
        names
    }

    /// The pages that are scenes of the animation, in order: every page that is not the timeline of a movie clip.
    pub fn scenes(&self) -> Vec<usize> {
        (0..self.pages.len()).filter(|index| self.pages[*index].clip_of.is_none()).collect()
    }

    /// The page that holds the timeline of a movie clip.
    pub fn clip_page(&self, symbol: u64) -> Option<usize> {
        self.pages.iter().position(|page| page.clip_of == Some(symbol))
    }

    /// A document sized to an image, with the image as its only object.
    /// A document from the pictures of an animation: each becomes a state with the
    /// time it stays on show, in the middle of a canvas the size of the largest.
    pub fn from_frames(name: &str, frames: Vec<(Bitmap, u32)>, loops: u16) -> Option<Self> {
        let width = frames.iter().map(|(bitmap, _)| bitmap.width()).max()?;
        let height = frames.iter().map(|(bitmap, _)| bitmap.height()).max()?;
        let mut doc = Self::new(width, height);
        let page = &mut doc.pages[0];
        (page.canvas, page.gif_loops, page.states) = (None, loops, Vec::new());
        page.layers[0].name = t("Background").into();
        let mut id = 1;
        for (index, (bitmap, delay)) in frames.into_iter().enumerate() {
            // A smaller picture is put in the middle of the canvas.
            let (x, y) = (((width - bitmap.width()) / 2) as f32, ((height - bitmap.height()) / 2) as f32);
            let object = Object::new_bitmap(id, name, bitmap, x, y);
            id += 1;
            let mut state = State::new(tf("State {}", &[&(index + 1)]));
            state.delay_ms = delay.clamp(10, 600_000);
            if index == 0 {
                // The first state is the one on show: its objects lie in the layer.
                page.layers[0].objects.push(object);
            } else {
                state.objects = vec![vec![object]];
            }
            page.states.push(state);
        }
        page.export.format = crate::optimize::ExportFormat::AnimatedGif;
        doc.next_id = id;
        Some(doc)
    }

    pub fn from_bitmap(name: impl Into<String>, bitmap: Bitmap) -> Self {
        let mut doc = Self::new(bitmap.width(), bitmap.height());
        doc.pages[0].canvas = None;
        doc.pages[0].layers[0].name = t("Background").into();
        doc.pages[0].layers[0].objects.push(Object::new_bitmap(1, name, bitmap, 0.0, 0.0));
        doc.pages[0].layers.push(Layer::new(tf("Layer {}", &[&1])));
        doc.next_id = 2;
        doc
    }

    pub fn page(&self) -> &Page {
        &self.pages[self.active_page]
    }

    pub fn page_mut(&mut self) -> &mut Page {
        &mut self.pages[self.active_page]
    }

    /// Points every placed symbol at the library's current contents.
    pub fn relink_symbols(&mut self) {
        let symbols = std::mem::take(&mut self.symbols);
        let mut resolved: HashMap<u64, Arc<Vec<Object>>> = HashMap::new();
        for object in self.pages.iter_mut().flat_map(Page::all_objects_mut) {
            match &mut object.shape {
                Shape::Instance { symbol, objects } => {
                    *objects = resolved.entry(*symbol).or_insert_with(|| resolve_symbol(&symbols, *symbol, 0)).clone();
                }
                Shape::Group { .. } => link_object(&symbols, object, 0),
                _ => {}
            }
        }
        self.symbols = symbols;
    }
}

#[derive(Serialize, Deserialize)]
struct FileEnvelope {
    format: String,
    version: u32,
    document: Document,
}

const FILE_FORMAT: &str = "emberframe";
const FILE_VERSION: u32 = 1;

pub fn document_to_json(doc: &Document) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&FileEnvelope {
        format: FILE_FORMAT.into(),
        version: FILE_VERSION,
        document: doc.clone(),
    })?)
}

pub fn document_from_json(text: &str) -> anyhow::Result<Document> {
    let file: FileEnvelope = anyhow::Context::context(serde_json::from_str(text), t("not an Emberframe document"))?;
    anyhow::ensure!(file.format == FILE_FORMAT, t("not an Emberframe document"));
    anyhow::ensure!(file.version <= FILE_VERSION, t("file was saved by a newer version"));
    anyhow::ensure!(!file.document.pages.is_empty(), t("document has no pages"));
    let mut doc = file.document;
    doc.active_page = doc.active_page.min(doc.pages.len() - 1);
    for page in &mut doc.pages {
        anyhow::ensure!(!page.layers.is_empty(), t("page has no layers"));
        page.active_layer = page.active_layer.min(page.layers.len() - 1);
        if page.states.is_empty() {
            page.states = default_states();
        }
        page.active_state = page.active_state.min(page.states.len() - 1);
        let (active, layers) = (page.active_state, page.layers.len());
        for (i, state) in page.states.iter_mut().enumerate() {
            state.objects.resize(if i == active { 0 } else { layers }, Vec::new());
        }
    }
    doc.relink_symbols();
    Ok(doc)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrange {
    Front,
    Forward,
    Backward,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    CenterX,
    Right,
    Top,
    CenterY,
    Bottom,
}

/// How many steps Undo keeps unless Preferences says otherwise.
const HISTORY_LIMIT: usize = 200;

/// A document plus its selection, undo history and clipboard.
pub struct Editor {
    pub doc: Document,
    pub selection: Vec<u64>,
    pub history: Vec<(String, Document)>,
    /// How many steps are kept for Undo.
    pub history_limit: usize,
    pub history_pos: usize,
    /// Changes whenever the document's content or the view of it changes.
    pub revision: u64,
    saved_doc: Document,
    clipboard: Vec<Object>,
}

impl Editor {
    pub fn new(doc: Document) -> Self {
        Self {
            history: vec![(t("New Document").into(), doc.clone())],
            history_pos: 0,
            history_limit: HISTORY_LIMIT,
            revision: 0,
            saved_doc: doc.clone(),
            selection: Vec::new(),
            clipboard: Vec::new(),
            doc,
        }
    }

    /// A second editor on a copy of the document, with the same selection, for trying a thing two ways.
    #[cfg(test)]
    fn clone_for_test(&self) -> Editor {
        let mut copy = Editor::new(self.doc.clone());
        copy.selection = self.selection.clone();
        copy
    }

    pub fn is_dirty(&self) -> bool {
        self.doc != self.saved_doc
    }

    pub fn mark_saved(&mut self) {
        self.saved_doc = self.doc.clone();
    }

    /// Records the current document as a new undoable step.
    pub fn commit(&mut self, label: &str) {
        if self.doc == self.history[self.history_pos].1 {
            return;
        }
        self.history.truncate(self.history_pos + 1);
        self.history.push((label.to_string(), self.doc.clone()));
        while self.history.len() > self.history_limit.max(2) {
            self.history.remove(0);
        }
        self.history_pos = self.history.len() - 1;
        self.revision += 1;
    }

    /// Applies a view change, such as switching page, without adding an undo step.
    fn navigate(&mut self, apply: impl Fn(&mut Document)) {
        let saved = !self.is_dirty();
        apply(&mut self.doc);
        apply(&mut self.history[self.history_pos].1);
        if saved {
            apply(&mut self.saved_doc);
        }
        self.revision += 1;
    }

    /// Discards uncommitted changes, such as a cancelled drag.
    pub fn rollback(&mut self) {
        self.doc = self.history[self.history_pos].1.clone();
        self.prune_selection();
    }

    pub fn can_undo(&self) -> bool {
        self.history_pos > 0
    }

    pub fn can_redo(&self) -> bool {
        self.history_pos + 1 < self.history.len()
    }

    pub fn undo(&mut self) {
        if self.can_undo() {
            self.goto_history(self.history_pos - 1);
        }
    }

    pub fn redo(&mut self) {
        if self.can_redo() {
            self.goto_history(self.history_pos + 1);
        }
    }

    pub fn goto_history(&mut self, pos: usize) {
        if pos < self.history.len() {
            self.history_pos = pos;
            self.rollback();
            self.revision += 1;
        }
    }

    fn prune_selection(&mut self) {
        let page = &self.doc.pages[self.doc.active_page];
        self.selection.retain(|id| page.objects().any(|o| o.id == *id));
    }

    pub fn alloc_id(&mut self) -> u64 {
        let id = self.doc.next_id;
        self.doc.next_id += 1;
        id
    }

    pub fn is_selected(&self, id: u64) -> bool {
        self.selection.contains(&id)
    }

    pub fn object(&self, id: u64) -> Option<&Object> {
        self.doc.page().objects().find(|o| o.id == id)
    }

    /// The current page as the exporter needs it, knowing which of its symbols are buttons.
    /// Whether the page has anything the screen cannot draw object by object, so that
    /// it has to be shown as the exporter renders it.
    pub fn needs_compositing(&self) -> bool {
        fn special(object: &Object) -> bool {
            !object.blend.is_normal() || object.mask.is_some() || !object.filters.is_empty() || object.fill_style.is_some() || !object.fill_look.is_plain() || !object.stroke_profile.is_empty() || matches!(&object.shape, Shape::Group { objects } if objects.iter().any(special))
        }
        self.doc.page().objects().any(|o| o.visible && special(o)) || self.doc.page().open_groups_need_drawing() || self.doc.page().shows_locked_masks()
    }

    pub fn export_page(&self) -> Page {
        self.export_page_at(self.doc.active_page)
    }

    /// Any page of the document as the exporter and the printer need it.
    pub fn export_page_at(&self, index: usize) -> Page {
        let mut page = self.doc.pages.get(index).unwrap_or(self.doc.page()).clone();
        page.button_symbols = self.doc.symbols.iter().filter(|s| s.kind == SymbolKind::Button).map(|s| s.id).collect();
        page.button_looks = self.button_looks();
        // A scene of an animation goes out as the finished animation shows it: guides left out, masks at work.
        if page.timeline.is_some() {
            page = page.played();
        }
        page
    }

    /// The looks of every button symbol that has more than the Up look.
    pub fn button_looks(&self) -> Vec<ButtonLooks> {
        let symbols = &self.doc.symbols;
        symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Button && s.looks.iter().any(|look| !look.is_empty()))
            .map(|s| {
                let looks = s
                    .looks
                    .iter()
                    .map(|look| {
                        let mut parts = look.clone();
                        for part in &mut parts {
                            link_object(symbols, part, 1);
                        }
                        Arc::new(parts)
                    })
                    .collect();
                ButtonLooks { symbol: s.id, natural: s.objects.iter().map(|o| o.bounds).reduce(Rect::union).unwrap_or_default(), looks }
            })
            .collect()
    }

    pub fn object_mut(&mut self, id: u64) -> Option<&mut Object> {
        self.doc.page_mut().objects_mut().find(|o| o.id == id)
    }

    pub fn selected(&self) -> Vec<&Object> {
        self.doc.page().objects().filter(|o| self.is_selected(o.id)).collect()
    }

    pub fn selection_bounds(&self) -> Option<Rect> {
        self.selected().into_iter().map(|o| o.bounds).reduce(Rect::union)
    }

    /// Applies `f` to each selected object.
    pub fn for_selected(&mut self, mut f: impl FnMut(&mut Object)) {
        let ids = self.selection.clone();
        for o in self.doc.page_mut().objects_mut() {
            if ids.contains(&o.id) {
                f(o);
            }
        }
    }

    /// Adds to the active layer, or the top-most unlocked layer if it is locked.
    pub fn add_object(&mut self, object: Object) -> bool {
        let page = self.doc.page_mut();
        let index = if page.layers[page.active_layer].locked {
            match page.layers.iter().rposition(|l| !l.locked) {
                Some(i) => i,
                None => return false,
            }
        } else {
            page.active_layer
        };
        let id = object.id;
        page.layers[index].objects.push(object);
        self.selection = vec![id];
        true
    }

    pub fn select_all(&mut self) {
        self.selection = self
            .doc
            .page()
            .layers
            .iter()
            .filter(|l| l.visible && !l.locked)
            .flat_map(|l| l.objects.iter())
            .filter(|o| o.visible && !o.locked)
            .map(|o| o.id)
            .collect();
    }

    pub fn select_in_rect(&mut self, area: Rect) {
        self.selection = self
            .doc
            .page()
            .layers
            .iter()
            .filter(|l| l.visible && !l.locked)
            .flat_map(|l| l.objects.iter())
            .filter(|o| o.visible && !o.locked && o.bounds.intersects(area))
            .map(|o| o.id)
            .collect();
    }

    pub fn delete_selected(&mut self) {
        let ids = std::mem::take(&mut self.selection);
        for layer in &mut self.doc.page_mut().layers {
            layer.objects.retain(|o| !ids.contains(&o.id));
        }
    }

    pub fn copy(&mut self) {
        self.clipboard = self.selected().into_iter().cloned().collect();
    }

    pub fn has_clipboard(&self) -> bool {
        !self.clipboard.is_empty()
    }

    /// Inserts copies of the clipboard, offset by `offset`, and selects them.
    pub fn paste(&mut self, offset: f32) {
        let items = self.clipboard.clone();
        self.insert_copies(items, offset);
    }

    pub fn duplicate(&mut self, offset: f32) {
        let items: Vec<Object> = self.selected().into_iter().cloned().collect();
        self.insert_copies(items, offset);
    }

    fn insert_copies(&mut self, items: Vec<Object>, offset: f32) {
        let mut added = Vec::with_capacity(items.len());
        for mut object in items {
            object.id = self.alloc_id();
            object.translate(offset, offset);
            added.push(object.id);
            self.add_object(object);
        }
        self.selection = added;
    }

    /// Fireworks' Paste Attributes: the selected objects take the fill, stroke,
    /// opacity, blend mode and live filters of what was copied.
    pub fn paste_attributes(&mut self) -> bool {
        let Some(source) = self.clipboard.first().cloned() else { return false };
        self.for_selected(|o| {
            if o.takes_color() && source.takes_color() {
                (o.fill, o.stroke) = (source.fill, source.stroke);
                o.fill_style = source.fill_style.clone();
            }
            (o.opacity, o.blend, o.filters) = (source.opacity, source.blend, source.filters.clone());
            // From text to text the font, its size and what is done to the letters go along too.
            if let (Shape::Text { size, style, .. }, Shape::Text { size: from_size, style: from_style, .. }) = (&mut o.shape, &source.shape) {
                let on_path = (style.on_path, style.path_offset);
                (*size, *style) = (*from_size, from_style.clone());
                (style.on_path, style.path_offset) = on_path;
                o.refit_text();
            }
        });
        !self.selection.is_empty()
    }

    /// Fireworks' Insert Hotspot and Insert Rectangular Slice: one web object over
    /// each selected object, or with `single` one over all of them.
    pub fn insert_web(&mut self, kind: WebKind, single: bool) -> bool {
        let areas: Vec<Rect> = self.selected().iter().filter(|o| !o.is_web()).map(|o| o.bounds).collect();
        let areas = if single { areas.into_iter().reduce(Rect::union).into_iter().collect() } else { areas };
        let mut added = Vec::new();
        for area in areas {
            let object = Object::new_web(self.alloc_id(), kind, area);
            added.push(object.id);
            if !self.add_object(object) {
                return false;
            }
        }
        if added.is_empty() {
            return false;
        }
        self.selection = added;
        true
    }

    /// Fireworks' Insert Empty Bitmap: a clear bitmap the size of the canvas, to paint on.
    pub fn insert_empty_bitmap(&mut self) -> bool {
        let (width, height) = (self.doc.page().width, self.doc.page().height);
        let Some(bitmap) = Bitmap::from_rgba(width, height, vec![0; width as usize * height as usize * 4]) else { return false };
        let object = Object::new_bitmap(self.alloc_id(), t("Bitmap"), bitmap, 0.0, 0.0);
        self.add_object(object)
    }

    /// Fireworks' Trim Canvas: the canvas is cut back to the artwork. It never grows.
    pub fn trim_canvas(&mut self) -> bool {
        let page = self.doc.page();
        let Some(art) = page.objects().filter(|o| !o.is_web() && o.visible).map(|o| o.bounds).reduce(Rect::union) else { return false };
        let edges = Rect::from_corners(v2(art.x.floor(), art.y.floor()), v2(art.right().ceil(), art.bottom().ceil()));
        match page.rect().intersect(edges) {
            Some(area) if area != page.rect() => self.crop_canvas(area),
            _ => false,
        }
    }

    /// Where the selected artwork is on the page: layer and place in it, lowest first.
    fn selected_places(&self) -> Vec<(usize, usize)> {
        let page = self.doc.page();
        let mut places = Vec::new();
        for (layer_index, layer) in page.layers.iter().enumerate() {
            for (index, object) in layer.objects.iter().enumerate() {
                if self.selection.contains(&object.id) && !object.is_web() {
                    places.push((layer_index, index));
                }
            }
        }
        places
    }

    /// Turns the objects at `places`, lowest first, into one bitmap where the topmost of them was.
    fn flatten_places(&mut self, places: &[(usize, usize)]) -> bool {
        let Some(&(top_layer, top_index)) = places.last() else { return false };
        let page = self.doc.page();
        let objects: Vec<Object> = places.iter().map(|(layer, index)| page.layers[*layer].objects[*index].clone()).collect();
        if page.layers[top_layer].locked {
            return false;
        }
        let Some((bitmap, area)) = crate::export::render_objects(&objects) else { return false };
        let flat = Object::new_bitmap(self.alloc_id(), t("Bitmap"), bitmap, area.x, area.y);
        let id = flat.id;
        let page = self.doc.page_mut();
        page.layers[top_layer].objects[top_index] = flat;
        // From the top down, so the places of those still to go do not shift.
        for (layer, index) in places[..places.len() - 1].iter().rev() {
            page.layers[*layer].objects.remove(*index);
        }
        self.selection = vec![id];
        true
    }

    /// Fireworks' Flatten Selection: the selected objects become one bitmap.
    pub fn flatten_selection(&mut self) -> bool {
        let places = self.selected_places();
        self.flatten_places(&places)
    }

    /// The bitmap right under the lowest selected object, which Merge Down merges into.
    fn merge_target(&self) -> Option<(usize, usize)> {
        let &(layer, index) = self.selected_places().first()?;
        let below = self.doc.page().layers[layer].objects.get(index.checked_sub(1)?)?;
        (below.is_image() && !self.selection.contains(&below.id)).then_some((layer, index - 1))
    }

    pub fn can_merge_down(&self) -> bool {
        self.merge_target().is_some()
    }

    /// Fireworks' Merge Down: the selected objects are flattened into the bitmap right under them.
    pub fn merge_down(&mut self) -> bool {
        let Some(target) = self.merge_target() else { return false };
        let mut places = self.selected_places();
        places.insert(0, target);
        self.flatten_places(&places)
    }

    /// Fireworks' Flatten Layers: everything on the layers that show becomes one
    /// bitmap on one layer. Hidden layers go; slices and hotspots stay as they are.
    pub fn flatten_layers(&mut self) -> bool {
        self.leave_symbol();
        self.close_groups();
        let page = self.doc.page().clone();
        // What a state shows on its visible layers, as one bitmap, with its web objects after it.
        let flattened = |view: &Page| {
            let shown: Vec<Object> = view.layers.iter().filter(|l| l.visible).flat_map(|l| &l.objects).filter(|o| !o.is_web() && o.visible).cloned().collect();
            let web: Vec<Object> = view.layers.iter().flat_map(|l| &l.objects).filter(|o| o.is_web()).cloned().collect();
            (crate::export::render_objects(&shown), web)
        };
        let (Some((bitmap, area)), web) = flattened(&page) else { return false };
        let flat = Object::new_bitmap(self.alloc_id(), t("Bitmap"), bitmap, area.x, area.y);
        let id = flat.id;
        // Every other state is flattened the same way, as Fireworks does: one bitmap for each.
        let mut others = Vec::new();
        for index in (0..page.states.len()).filter(|index| *index != page.active_state) {
            let (drawn, web) = flattened(&page.at_state(index));
            let flat = drawn.map(|(bitmap, area)| Object::new_bitmap(self.alloc_id(), t("Bitmap"), bitmap, area.x, area.y));
            others.push((index, flat.into_iter().chain(web).collect::<Vec<Object>>()));
        }
        let page = self.doc.page_mut();
        let mut layer = Layer::new(tf("Layer {}", &[&1]));
        layer.objects = std::iter::once(flat).chain(web).collect();
        page.layers = vec![layer];
        page.active_layer = 0;
        for (index, objects) in others {
            page.states[index].objects = vec![objects];
        }
        page.animations.clear();
        self.selection = vec![id];
        true
    }

    pub fn arrange(&mut self, op: Arrange) {
        let ids = self.selection.clone();
        for layer in &mut self.doc.page_mut().layers {
            let objects = &mut layer.objects;
            match op {
                Arrange::Front => objects.sort_by_key(|o| ids.contains(&o.id)),
                Arrange::Back => objects.sort_by_key(|o| !ids.contains(&o.id)),
                Arrange::Forward => {
                    for i in (0..objects.len().saturating_sub(1)).rev() {
                        if ids.contains(&objects[i].id) && !ids.contains(&objects[i + 1].id) {
                            objects.swap(i, i + 1);
                        }
                    }
                }
                Arrange::Backward => {
                    for i in 1..objects.len() {
                        if ids.contains(&objects[i].id) && !ids.contains(&objects[i - 1].id) {
                            objects.swap(i, i - 1);
                        }
                    }
                }
            }
        }
    }

    /// Aligns to the selection bounds; a single object aligns to the canvas.
    pub fn align(&mut self, op: Align) {
        let page = self.doc.page();
        let page_rect = Rect { x: 0.0, y: 0.0, w: page.width as f32, h: page.height as f32 };
        let area = if self.selection.len() > 1 {
            self.selection_bounds().unwrap_or(page_rect)
        } else {
            page_rect
        };
        self.for_selected(|o| {
            let b = o.bounds;
            let (dx, dy) = match op {
                Align::Left => (area.x - b.x, 0.0),
                Align::CenterX => (area.center().x - b.center().x, 0.0),
                Align::Right => (area.right() - b.right(), 0.0),
                Align::Top => (0.0, area.y - b.y),
                Align::CenterY => (0.0, area.center().y - b.center().y),
                Align::Bottom => (0.0, area.bottom() - b.bottom()),
            };
            o.translate(dx, dy);
        });
    }

    /// Filters selected bitmaps. `pixels` confines the filter on one of them to a mask.
    pub fn apply_filter(&mut self, filter: Filter, pixels: Option<(u64, &Mask)>) {
        self.for_selected(|o| {
            let mask = pixels.filter(|(id, _)| *id == o.id).map(|(_, mask)| mask);
            o.edit_pixels(mask, |bitmap| bitmap.apply_filter(filter));
        });
    }

    /// Puts the selected pixels of a bitmap on the clipboard as a new bitmap.
    pub fn copy_pixels(&mut self, id: u64, mask: &Mask) {
        let Some(object) = self.object(id) else { return };
        let Some((piece, x, y)) = object.bitmap().and_then(|b| b.extract(mask)) else { return };
        let (Some(corner), Some(far)) = (
            object.pixel_to_doc(v2(x as f32, y as f32)),
            object.pixel_to_doc(v2((x + piece.width()) as f32, (y + piece.height()) as f32)),
        ) else {
            return;
        };
        let mut copy = Object::new_bitmap(0, object.name.clone(), piece, corner.x, corner.y);
        copy.set_bounds(Rect::from_corners(corner, far));
        self.clipboard = vec![copy];
    }

    pub fn clear_pixels(&mut self, id: u64, mask: &Mask) {
        if let Some(bitmap) = self.object_mut(id).and_then(Object::bitmap_mut) {
            bitmap.clear(mask);
        }
    }

    pub fn web_visible(&self) -> bool {
        self.doc.page().objects().any(|o| o.is_web() && o.visible)
    }

    pub fn set_web_visible(&mut self, visible: bool) {
        let mut hidden = Vec::new();
        for object in self.doc.page_mut().all_objects_mut().filter(|o| o.is_web()) {
            object.visible = visible;
            hidden.push(object.id);
        }
        if !visible {
            self.selection.retain(|id| !hidden.contains(id));
        }
    }

    /// Cuts vector objects along a line, replacing each with its pieces and selecting them.
    /// Only selected objects are cut, or everything when nothing is selected.
    /// Returns the number of objects cut.
    pub fn knife(&mut self, a: Vec2, b: Vec2) -> usize {
        let selection = self.selection.clone();
        let mut next_id = self.doc.next_id;
        let mut created = Vec::new();
        let mut cut = 0;
        for layer in self.doc.page_mut().layers.iter_mut().filter(|l| l.visible && !l.locked) {
            let mut kept = Vec::with_capacity(layer.objects.len());
            for object in std::mem::take(&mut layer.objects) {
                let eligible = object.visible
                    && !object.locked
                    && object.reshapable()
                    && (selection.is_empty() || selection.contains(&object.id));
                let mut path = object.clone();
                let pieces = if eligible && path.to_path() {
                    path.flatten();
                    split_path(&path.points, path.is_closed(), a, b).filter(|pieces| !pieces.is_empty())
                } else {
                    None
                };
                let Some(pieces) = pieces else {
                    kept.push(object);
                    continue;
                };
                cut += 1;
                for points in pieces {
                    let mut piece = path.clone();
                    piece.id = next_id;
                    next_id += 1;
                    piece.shape = Shape::Path { closed: false };
                    piece.bounds = Rect::of_points(&points);
                    // The cut was made on straight pieces, and each piece gets its curves back.
                    (piece.points, piece.handles) = crate::fit::fit_curves(&points, false, crate::fit::TOLERANCE);
                    created.push(piece.id);
                    kept.push(piece);
                }
            }
            layer.objects = kept;
        }
        self.doc.next_id = next_id;
        if cut > 0 {
            self.selection = created;
        }
        cut
    }

    pub fn has_selected_bitmap(&self) -> bool {
        self.selected().iter().any(|o| o.is_image())
    }

    /// Trims the canvas to `area`, shifting objects so its corner becomes the origin.
    pub fn crop_canvas(&mut self, area: Rect) -> bool {
        let (x, y) = (area.x.round(), area.y.round());
        let (w, h) = (area.w.round(), area.h.round());
        if w < 1.0 || h < 1.0 {
            return false;
        }
        let page = self.doc.page_mut();
        page.width = w as u32;
        page.height = h as u32;
        for object in page.all_objects_mut() {
            object.translate(-x, -y);
        }
        true
    }

    pub fn add_layer(&mut self) {
        let page = self.doc.page_mut();
        let name = tf("Layer {}", &[&(page.layers.len() + 1)]);
        let at = page.active_layer + 1;
        page.layers.insert(at, Layer::new(name));
        page.active_layer = at;
        if let Some(timeline) = &mut page.timeline {
            // A new layer of an animation is empty for as long as the scene lasts.
            let frames = timeline.frame_count().max(1);
            timeline.layers.insert(at.min(timeline.layers.len()), vec![Span { start: 0, length: frames, objects: Vec::new(), ..Default::default() }]);
            if at < timeline.kinds.len() {
                timeline.kinds.insert(at, LayerKind::Normal);
            }
            if at < timeline.bound.len() {
                timeline.bound.insert(at, false);
            }
        }
        for state in page.inactive_states_mut() {
            state.objects.insert(at, Vec::new());
        }
    }

    /// Fireworks' Add Shadow: under each selected object, a dark copy of it lying on the ground behind it.
    pub fn add_shadows(&mut self) -> usize {
        let ids = self.selection.clone();
        let mut added = 0;
        for id in ids {
            let Some(original) = self.object(id).filter(|o| !o.is_web()).cloned() else { continue };
            let mut shadow = original.clone();
            shadow.id = self.alloc_id();
            shadow.name = t("Shadow").into();
            (shadow.filters, shadow.mask, shadow.fill_style, shadow.opacity) = (Vec::new(), None, None, 0.4);
            if shadow.takes_color() {
                shadow.fill = shadow.fill.map(|_| Color::BLACK);
                shadow.stroke = shadow.stroke.map(|stroke| Stroke { color: Color::BLACK, ..stroke });
            }
            if !shadow.warp(&Warp::cast(original.bounds.bottom(), 0.5, 0.5)) {
                continue;
            }
            for layer in &mut self.doc.page_mut().layers {
                if let Some(at) = layer.objects.iter().position(|o| o.id == id) {
                    layer.objects.insert(at, shadow);
                    added += 1;
                    break;
                }
            }
        }
        added
    }

    /// Fireworks' Auto Vector Mask: each selected object gets a mask of its own size
    /// filled with a gradient from white to black, so that it fades as the gradient darkens.
    pub fn auto_vector_mask(&mut self, which: usize) -> bool {
        let Some((_, kind, start, end)) = AUTO_MASKS.get(which).copied() else { return false };
        let ids: Vec<u64> = self.selected().iter().filter(|o| !o.is_web()).map(|o| o.id).collect();
        for id in &ids {
            let Some(bounds) = self.object(*id).map(|o| o.bounds) else { continue };
            let mut mask = Object::new_web(self.alloc_id(), WebKind::Slice, bounds);
            mask.name = t("Mask").into();
            (mask.shape, mask.fill, mask.stroke, mask.opacity) = (Shape::Rect { radius: 0.0 }, Some(Color::WHITE), None, 1.0);
            let mut style = FillStyle::new(kind, Color::WHITE, Color::BLACK);
            (style.start, style.end) = (v2(start.0, start.1), v2(end.0, end.1));
            mask.fill_style = Some(Arc::new(style));
            if let Some(object) = self.object_mut(*id) {
                object.mask = Some(Arc::new(ObjectMask::new(mask, true)));
            }
        }
        !ids.is_empty()
    }

    /// Fireworks' Twist and Fade: copies of the selection, each a step further on,
    /// turned, resized and fainter than the one before it, laid under the original.
    pub fn twist_and_fade(&mut self, settings: TwistSettings) -> usize {
        let ids = self.selection.clone();
        let steps = settings.steps.clamp(1, 200);
        let angle = settings.direction.to_radians();
        let mut made = 0;
        for id in ids {
            let Some(original) = self.object(id).filter(|o| !o.is_web()).cloned() else { continue };
            let center = original.bounds.center();
            let mut copies = Vec::new();
            for step in 1..=steps {
                let n = step as f32;
                let mut copy = original.clone();
                copy.id = self.alloc_id();
                let size = (settings.scale / 100.0).max(0.01).powf(n);
                let middle = v2(center.x + angle.cos() * settings.spacing * n, center.y - angle.sin() * settings.spacing * n);
                let (w, h) = (original.bounds.w * size, original.bounds.h * size);
                copy.set_bounds(Rect { x: middle.x - w / 2.0, y: middle.y - h / 2.0, w, h });
                if settings.rotation != 0.0 {
                    copy.warp(&Warp::rotation(middle, (settings.rotation * n).to_radians()));
                }
                let faded = 1.0 + (settings.fade_to / 100.0 - 1.0) * n / steps as f32;
                copy.opacity = (original.opacity * faded).clamp(0.0, 1.0);
                // The furthest copy lies lowest.
                copies.insert(0, copy);
            }
            made += copies.len();
            for layer in &mut self.doc.page_mut().layers {
                if let Some(at) = layer.objects.iter().position(|o| o.id == id) {
                    layer.objects.splice(at..at, copies);
                    break;
                }
            }
        }
        made
    }

    /// Fireworks' Distribute to Layers: each selected object goes to a new layer of its own, named after it.
    pub fn distribute_to_layers(&mut self) -> usize {
        let objects = self.selected_in_order();
        if objects.len() < 2 {
            return 0;
        }
        let ids: Vec<u64> = objects.iter().map(|o| o.id).collect();
        let page = self.doc.page_mut();
        for layer in &mut page.layers {
            layer.objects.retain(|o| !ids.contains(&o.id));
        }
        let count = objects.len();
        for object in objects {
            let at = page.layers.len();
            let mut layer = Layer::new(object.name.clone());
            layer.objects.push(object);
            page.layers.push(layer);
            page.active_layer = at;
            for state in page.inactive_states_mut() {
                state.objects.insert(at.min(state.objects.len()), Vec::new());
            }
        }
        count
    }

    /// Hides, or locks, every layer but the active one.
    pub fn set_other_layers(&mut self, hide: bool) -> bool {
        let page = self.doc.page_mut();
        let active = page.active_layer;
        let mut changed = false;
        for (index, layer) in page.layers.iter_mut().enumerate() {
            if index != active {
                let flag = if hide { &mut layer.visible } else { &mut layer.locked };
                changed |= *flag != !hide;
                *flag = !hide;
            }
        }
        changed
    }

    /// Reverses the order of the states from one to another, both counted from 0 and both included.
    pub fn reverse_states(&mut self, from: usize, to: usize) -> bool {
        self.leave_symbol();
        self.close_groups();
        let page = self.doc.page_mut();
        let last = page.states.len() - 1;
        let (from, to) = (from.min(to).min(last), from.max(to).min(last));
        if from == to {
            return false;
        }
        // The state on show keeps its objects in the layers: they go into the state for the turn, and come back out.
        let active = page.active_state;
        page.states[active].objects = page.layers.iter_mut().map(|l| if l.shared { Vec::new() } else { std::mem::take(&mut l.objects) }).collect();
        page.states[from..=to].reverse();
        let active = if (from..=to).contains(&active) { from + to - active } else { active };
        let loaded = std::mem::take(&mut page.states[active].objects);
        for (layer, objects) in page.layers.iter_mut().zip(loaded) {
            if !layer.shared {
                layer.objects = objects;
            }
        }
        page.active_state = active;
        self.selection.clear();
        true
    }

    /// Removes the active layer. The last remaining layer is kept.
    pub fn delete_layer(&mut self) -> bool {
        let page = self.doc.page_mut();
        if page.layers.len() < 2 {
            return false;
        }
        let at = page.active_layer;
        page.layers.remove(at);
        if let Some(timeline) = page.timeline.as_mut().filter(|timeline| at < timeline.layers.len()) {
            if at < timeline.kinds.len() {
                timeline.kinds.remove(at);
            }
            if at < timeline.bound.len() {
                timeline.bound.remove(at);
            }
            timeline.layers.remove(at);
        }
        for state in page.inactive_states_mut() {
            state.objects.remove(at);
        }
        page.active_layer = page.active_layer.min(page.layers.len() - 1);
        self.prune_selection();
        true
    }

    pub fn add_page(&mut self) {
        let (w, h, canvas) = {
            let p = self.doc.page();
            (p.width, p.height, p.canvas)
        };
        self.leave_clip();
        // In an animation a new page is a new scene, with a timeline of its own.
        let fps = self.doc.page().timeline.as_ref().map(|timeline| timeline.fps);
        let number = self.doc.scenes().len() + 1;
        let mut page = Page::new(if fps.is_some() { tf("Scene {}", &[&number]) } else { tf("Page {}", &[&number]) }, w, h);
        page.canvas = canvas;
        if let Some(fps) = fps {
            page.start_timeline(fps);
        }
        self.doc.pages.push(page);
        self.doc.active_page = self.doc.pages.len() - 1;
        self.selection.clear();
    }

    pub fn delete_page(&mut self) -> bool {
        // The last scene stays, and the page of a movie clip goes only with its symbol.
        if self.doc.scenes().len() < 2 || self.doc.page().clip_of.is_some() {
            return false;
        }
        self.doc.clip_return = None;
        self.doc.pages.remove(self.doc.active_page);
        self.doc.active_page = self.doc.active_page.min(self.doc.pages.len() - 1);
        self.selection.clear();
        true
    }

    pub fn set_active_page(&mut self, index: usize) {
        if index < self.doc.pages.len() {
            // Going elsewhere from a movie clip's page ends the edit of the clip.
            if self.editing_clip().is_some() {
                self.sync_clip();
                self.doc.clip_return = None;
                self.commit(t("Edit Symbol"));
            }
            self.leave_symbol_for_good();
            self.close_groups();
            self.navigate(|doc| doc.active_page = index);
            self.selection.clear();
        }
    }

    pub fn set_active_layer(&mut self, index: usize) {
        if index < self.doc.page().layers.len() {
            self.navigate(|doc| doc.page_mut().active_layer = index);
        }
    }

    pub fn set_active_state(&mut self, index: usize) {
        if index < self.doc.page().states.len() && index != self.doc.page().active_state {
            self.leave_symbol_for_good();
            self.close_groups();
            self.navigate(|doc| doc.page_mut().goto_state(index));
            self.selection.clear();
        }
    }

    /// Fireworks' Attach to Path: with a text and a path selected, the text is set
    /// along the path, which then belongs to the text.
    pub fn attach_text_to_path(&mut self) -> bool {
        let selected = self.selected();
        let (Some(text), Some(path)) = (
            selected.iter().find(|o| matches!(o.shape, Shape::Text { .. })).map(|o| o.id),
            selected.iter().find(|o| o.reshapable() && !matches!(o.shape, Shape::Text { .. })).map(|o| (o.id, o.outline(), o.is_closed())),
        ) else {
            return false;
        };
        let (path_id, mut line, closed) = path;
        // A closed shape is gone round once, clockwise from its leftmost point, so the text
        // starts at the left and runs over the top, standing on the outside.
        if closed && line.len() >= 3 {
            let turn: f32 = (0..line.len()).map(|i| (line[i].x * line[(i + 1) % line.len()].y) - (line[(i + 1) % line.len()].x * line[i].y)).sum();
            if turn < 0.0 {
                line.reverse();
            }
            let leftmost = (0..line.len()).min_by(|a, b| line[*a].x.total_cmp(&line[*b].x).then(line[*b].y.total_cmp(&line[*a].y))).unwrap_or(0);
            line.rotate_left(leftmost);
            line.extend(line.first().copied());
        }
        let page = self.doc.page_mut();
        for layer in &mut page.layers {
            layer.objects.retain(|o| o.id != path_id);
        }
        if let Some(object) = self.object_mut(text) {
            object.placement = None;
            object.points = line;
            object.refit_text();
        }
        self.selection = vec![text];
        true
    }

    /// Whether a selected text is attached to a path.
    pub fn has_text_on_path(&self) -> bool {
        self.selected().iter().any(|o| matches!(o.shape, Shape::Text { .. }) && o.points.len() >= 2)
    }

    /// Fireworks' Detach from Path: the text stands on its own again, and its path is a path again.
    pub fn detach_text_from_path(&mut self) -> bool {
        let ids: Vec<u64> = self.selected().iter().filter(|o| matches!(o.shape, Shape::Text { .. }) && o.points.len() >= 2).map(|o| o.id).collect();
        for id in &ids {
            let Some(object) = self.object_mut(*id) else { continue };
            let line = std::mem::take(&mut object.points);
            let corner = v2(object.bounds.x, object.bounds.y);
            object.refit_text();
            (object.bounds.x, object.bounds.y) = (corner.x, corner.y);
            let mut path = object.clone();
            path.id = self.alloc_id();
            path.name = t("Path").into();
            path.shape = Shape::Path { closed: false };
            path.bounds = Rect::of_points(&line);
            path.points = line;
            (path.fill, path.fill_style, path.filters, path.mask) = (None, None, Vec::new(), None);
            path.stroke = Some(Stroke { color: Color::BLACK, width: 1.0 });
            self.add_object(path);
        }
        self.selection = ids.clone();
        !ids.is_empty()
    }

    /// Fireworks' Reverse Direction: text on a path runs the other way along it.
    pub fn reverse_text_path(&mut self) -> bool {
        let mut any = false;
        self.for_selected(|o| {
            if matches!(o.shape, Shape::Text { .. }) && o.points.len() >= 2 {
                o.points.reverse();
                o.refit_text();
                any = true;
            }
        });
        any
    }

    /// Fireworks' Subselect: the members of each selected group become the
    /// selection, to be worked on one by one. The group is whole again as soon as
    /// none of its members is selected. Returns false if no group was selected.
    pub fn subselect(&mut self) -> bool {
        let groups: Vec<u64> = self.selected().iter().filter(|o| matches!(o.shape, Shape::Group { .. })).map(|o| o.id).collect();
        if groups.is_empty() {
            return false;
        }
        let mut picked: Vec<u64> = self.selection.iter().copied().filter(|id| !groups.contains(id)).collect();
        for id in groups {
            let Some(group) = self.object(id).cloned() else { continue };
            // The members come out where they show, with numbers of their own: copies of a group share theirs.
            let mut members = group.instance_parts();
            for member in &mut members {
                member.id = self.alloc_id();
            }
            let ids: Vec<u64> = members.iter().map(|member| member.id).collect();
            // What the members were given on the way out is not given again when they go back in.
            let shell = Object { shape: Shape::Group { objects: Arc::new(Vec::new()) }, opacity: 1.0, placement: None, ..group };
            let page = self.doc.page_mut();
            for layer in &mut page.layers {
                if let Some(at) = layer.objects.iter().position(|o| o.id == id) {
                    layer.objects.splice(at..=at, members);
                    break;
                }
            }
            page.open_groups.push(OpenGroup { shell, members: ids.clone() });
            picked.extend(ids);
        }
        self.selection = picked;
        true
    }

    /// The open groups that have a selected member.
    fn open_groups_selected(&self) -> Vec<usize> {
        let open = &self.doc.page().open_groups;
        (0..open.len()).filter(|index| open[*index].members.iter().any(|id| self.selection.contains(id))).collect()
    }

    pub fn can_superselect(&self) -> bool {
        !self.open_groups_selected().is_empty()
    }

    /// Fireworks' Superselect: the whole group is selected in place of the members picked out of it.
    pub fn superselect(&mut self) -> bool {
        let open = self.open_groups_selected();
        for index in open.iter().rev() {
            self.close_group(*index, true);
        }
        !open.is_empty()
    }

    /// Puts an open group back together from the members that are still on the
    /// page, where the lowest of them lies. With `select` the group takes their place in the selection.
    fn close_group(&mut self, index: usize, select: bool) {
        let page = self.doc.page_mut();
        if index >= page.open_groups.len() {
            return;
        }
        let OpenGroup { mut shell, members } = page.open_groups.remove(index);
        let mut found = Vec::new();
        let mut spot = None;
        for (layer_index, layer) in page.layers.iter_mut().enumerate() {
            let mut at = 0;
            while at < layer.objects.len() {
                if members.contains(&layer.objects[at].id) {
                    spot.get_or_insert((layer_index, at));
                    found.push(layer.objects.remove(at));
                } else {
                    at += 1;
                }
            }
        }
        let was_selected = self.selection.iter().any(|id| members.contains(id));
        self.selection.retain(|id| !members.contains(id));
        // A group whose members were all deleted is gone with them.
        let (Some((layer, at)), Some(bounds)) = (spot, found.iter().map(|o| o.bounds).reduce(Rect::union)) else { return };
        shell.bounds = bounds;
        shell.shape = Shape::Group { objects: Arc::new(found) };
        let id = shell.id;
        let page = self.doc.page_mut();
        let at = at.min(page.layers[layer].objects.len());
        page.layers[layer].objects.insert(at, shell);
        if select && was_selected {
            self.selection.push(id);
        }
    }

    /// Closes the open groups none of whose members is selected any more.
    pub fn sync_open_groups(&mut self) {
        let selected = self.open_groups_selected();
        for index in (0..self.doc.page().open_groups.len()).rev() {
            if !selected.contains(&index) {
                self.close_group(index, false);
            }
        }
    }

    /// Closes every open group, as before saving or leaving the page. Selected members give way to their group.
    pub fn close_groups(&mut self) {
        for index in (0..self.doc.page().open_groups.len()).rev() {
            self.close_group(index, true);
        }
    }

    /// Replaces the selection with `objects`, where the bottom-most selected object was.
    pub fn replace_selected(&mut self, objects: Vec<Object>) {
        let selection = std::mem::take(&mut self.selection);
        let page = self.doc.page_mut();
        let spot = page.layers.iter().enumerate().find_map(|(l, layer)| {
            layer.objects.iter().position(|o| selection.contains(&o.id)).map(|i| (l, i))
        });
        for layer in &mut page.layers {
            layer.objects.retain(|o| !selection.contains(&o.id));
        }
        self.selection = objects.iter().map(|o| o.id).collect();
        let (layer, index) = spot.unwrap_or((page.active_layer, page.layers[page.active_layer].objects.len()));
        page.layers[layer].objects.splice(index..index, objects);
    }

    pub fn apply_style(&mut self, style: &Style) {
        self.for_selected(|o| {
            if o.takes_color() {
                o.fill = style.fill;
                o.fill_style = style.fill_style.clone();
                o.stroke = style.stroke;
                o.opacity = style.opacity;
                o.filters = style.filters.clone();
            }
        });
    }

    /// Saves the first selected vector object's look as a new style.
    pub fn add_style_from_selection(&mut self) -> bool {
        let Some(source) = self.selected().into_iter().find(|o| o.takes_color()).cloned() else { return false };
        let name = tf("Style {}", &[&(self.doc.styles.len() + 1)]);
        self.doc.styles.push(Style { name, fill: source.fill, stroke: source.stroke, opacity: source.opacity, filters: source.filters.clone(), fill_style: source.fill_style.clone() });
        true
    }

    /// Adds a symbol to the library, or finds the one that already has this name.
    pub fn add_library_symbol(&mut self, name: &str, objects: &[Object], kind: SymbolKind) -> u64 {
        if let Some(existing) = self.doc.symbols.iter().find(|s| s.name == name) {
            return existing.id;
        }
        let id = self.alloc_id();
        self.doc.symbols.push(Symbol { id, name: name.to_string(), objects: normalized(objects), kind, looks: Vec::new() });
        id
    }

    fn new_instance(&mut self, symbol: u64, bounds: Rect) -> Object {
        let name = self.doc.symbols.iter().find(|s| s.id == symbol).map_or(t("Symbol").to_string(), |s| s.name.clone());
        let objects = resolve_symbol(&self.doc.symbols, symbol, 0);
        Object {
            id: self.alloc_id(),
            name,
            shape: Shape::Instance { symbol, objects },
            bounds,
            points: Vec::new(),
            handles: Vec::new(),
            placement: None,
            pivot: None,

            behaviors: Vec::new(),
            attached: String::new(),
            blend: crate::blend::BlendMode::Normal,
            mask: None,
            mask_of: None,
            fill_style: None,
            stroke_profile: Vec::new(),
            fill_look: Default::default(),
            link: String::new(),
            filters: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    /// Places a symbol at its natural size, centered on the canvas.
    #[cfg(test)]
    pub fn place_symbol(&mut self, symbol: u64) -> bool {
        self.place_symbol_at(symbol, self.doc.page().rect().center())
    }

    /// The size a copy of the symbol has when it is placed.
    pub fn symbol_size(&self, symbol: u64) -> Option<(f32, f32)> {
        let bounds = self.doc.symbols.iter().find(|s| s.id == symbol)?.objects.iter().map(|o| o.bounds).reduce(Rect::union)?;
        Some((bounds.w, bounds.h))
    }

    /// Places a copy of the symbol with its middle at `center`.
    pub fn place_symbol_at(&mut self, symbol: u64, center: Vec2) -> bool {
        let size = self.doc.symbols.iter().find(|s| s.id == symbol).and_then(|s| s.objects.iter().map(|o| o.bounds).reduce(Rect::union));
        let Some(size) = size else { return false };
        let bounds = Rect { x: (center.x - size.w / 2.0).round(), y: (center.y - size.h / 2.0).round(), w: size.w, h: size.h };
        let instance = self.new_instance(symbol, bounds);
        self.add_object(instance)
    }

    /// Turns the selected objects into a library symbol and one placed copy.
    pub fn make_symbol(&mut self) -> Option<u64> {
        let objects: Vec<Object> = self.selected().into_iter().filter(|o| !o.is_web()).cloned().collect();
        let bounds = objects.iter().map(|o| o.bounds).reduce(Rect::union)?;
        self.selection = objects.iter().map(|o| o.id).collect();
        let id = self.alloc_id();
        let name = tf("Symbol {}", &[&(self.doc.symbols.len() + 1)]);
        self.doc.symbols.push(Symbol { id, name, objects: normalized(&objects), kind: SymbolKind::Graphic, looks: Vec::new() });
        let instance = self.new_instance(id, bounds);
        self.replace_selected(vec![instance]);
        Some(id)
    }

    /// Adds an empty symbol to the library, as New Symbol and New Button do before it is drawn.
    pub fn new_symbol(&mut self, name: &str, kind: SymbolKind) -> u64 {
        let id = self.alloc_id();
        self.doc.symbols.push(Symbol { id, name: name.to_string(), objects: Vec::new(), kind, looks: Vec::new() });
        id
    }

    pub fn editing_symbol(&self) -> Option<&SymbolEdit> {
        self.doc.page().symbol_edit.as_ref()
    }

    /// The symbol being edited, as the library has it.
    pub fn edited_symbol(&self) -> Option<&Symbol> {
        let id = self.editing_symbol()?.symbol;
        self.doc.symbols.iter().find(|s| s.id == id)
    }

    /// Whether Edit Symbol has something to open: one selected copy of a symbol.
    pub fn can_edit_symbol(&self) -> bool {
        self.selected().iter().any(|o| matches!(o.shape, Shape::Instance { .. }))
    }

    /// Opens the selected copy of a symbol for editing where it lies.
    pub fn edit_selected_symbol(&mut self) -> bool {
        let Some(instance) = self.selected().into_iter().find(|o| matches!(o.shape, Shape::Instance { .. })).cloned() else { return false };
        let Shape::Instance { symbol, .. } = instance.shape else { return false };
        self.edit_symbol(symbol, Some(instance), false)
    }

    /// Parts of a symbol as they lie on the page when the symbol, whose Up look fills `natural`, fills `frame`.
    fn laid(&mut self, parts: &[Object], natural: Rect, frame: Rect) -> Vec<Object> {
        let Some(own) = parts.iter().map(|o| o.bounds).reduce(Rect::union) else { return Vec::new() };
        let sx = if natural.w > 0.0 { frame.w / natural.w } else { 1.0 };
        let sy = if natural.h > 0.0 { frame.h / natural.h } else { 1.0 };
        let mut linked = parts.to_vec();
        for part in &mut linked {
            link_object(&self.doc.symbols, part, 1);
        }
        let mut holder = linked[0].clone();
        (holder.placement, holder.opacity, holder.mask) = (None, 1.0, None);
        holder.bounds = Rect { x: frame.x + (own.x - natural.x) * sx, y: frame.y + (own.y - natural.y) * sy, w: own.w * sx, h: own.h * sy };
        holder.shape = Shape::Group { objects: Arc::new(linked) };
        let mut laid = holder.instance_parts();
        for part in &mut laid {
            part.id = self.alloc_id();
        }
        laid
    }

    /// Opens a symbol for editing on the page. With a copy of it, the symbol's objects
    /// take the copy's place; without one they are put in the middle of the page.
    pub fn edit_symbol(&mut self, symbol: u64, instance: Option<Object>, place: bool) -> bool {
        self.leave_symbol();
        self.end_mask_edit();
        self.close_groups();
        let Some(entry) = self.doc.symbols.iter().find(|s| s.id == symbol).cloned() else { return false };
        let middle = self.doc.page().rect().center();
        let natural = entry.objects.iter().map(|o| o.bounds).reduce(Rect::union).unwrap_or_default();
        let frame = match &instance {
            Some(object) => object.placement.map_or(object.bounds, |p| p.frame),
            None => Rect { x: (middle.x - natural.w / 2.0).round(), y: (middle.y - natural.h / 2.0).round(), w: natural.w, h: natural.h },
        };
        let mut looks: [Vec<Object>; 4] = Default::default();
        for (look, parts) in std::iter::once(&entry.objects).chain(entry.looks.iter()).take(4).enumerate() {
            looks[look] = self.laid(parts, natural, frame);
        }
        let page = self.doc.page_mut();
        let mut spot = (page.active_layer, page.layers[page.active_layer].objects.len());
        if let Some(object) = &instance {
            for (index, layer) in page.layers.iter_mut().enumerate() {
                if let Some(at) = layer.objects.iter().position(|o| o.id == object.id) {
                    layer.objects.remove(at);
                    spot = (index, at);
                }
            }
        }
        let outside: Vec<u64> = page.objects().map(|o| o.id).collect();
        let mut locked = Vec::new();
        for object in page.layers.iter_mut().flat_map(|l| l.objects.iter_mut()).filter(|o| !o.locked) {
            object.locked = true;
            locked.push(object.id);
        }
        let shown = std::mem::take(&mut looks[0]);
        page.layers[spot.0].objects.splice(spot.1..spot.1, shown);
        page.symbol_edit = Some(SymbolEdit { symbol, shell: instance, spot, place, outside, locked, frame, look: 0, looks });
        self.selection.clear();
        true
    }

    /// Takes the objects of the symbol being edited off the page, bottom-most first.
    fn take_edit_members(&mut self) -> Vec<Object> {
        let page = self.doc.page_mut();
        let Some(edit) = &page.symbol_edit else { return Vec::new() };
        let outside = edit.outside.clone();
        let mut members = Vec::new();
        for layer in &mut page.layers {
            let mut at = 0;
            while at < layer.objects.len() {
                let object = &layer.objects[at];
                if outside.contains(&object.id) || object.is_web() {
                    at += 1;
                } else {
                    members.push(layer.objects.remove(at));
                }
            }
        }
        self.selection.retain(|id| !members.iter().any(|o| o.id == *id));
        members
    }

    fn put_edit_members(&mut self, members: Vec<Object>) {
        let page = self.doc.page_mut();
        let Some(edit) = &page.symbol_edit else { return };
        let layer = edit.spot.0.min(page.layers.len() - 1);
        let at = edit.spot.1.min(page.layers[layer].objects.len());
        page.layers[layer].objects.splice(at..at, members);
    }

    /// Shows another look of the button being edited.
    pub fn set_symbol_look(&mut self, look: usize) -> bool {
        if look > 3 || self.editing_symbol().is_none_or(|edit| edit.look == look) {
            return false;
        }
        let members = self.take_edit_members();
        let Some(edit) = self.doc.page_mut().symbol_edit.as_mut() else { return false };
        edit.looks[edit.look] = members;
        edit.look = look;
        let shown = std::mem::take(&mut edit.looks[look]);
        self.put_edit_members(shown);
        true
    }

    /// Fireworks' Copy Up Graphic, Copy Over Graphic and Copy Down Graphic: the look
    /// before the one on show is copied into it.
    pub fn copy_previous_look(&mut self) -> bool {
        let Some(edit) = self.editing_symbol().filter(|edit| edit.look > 0) else { return false };
        let mut copies = edit.looks[edit.look - 1].clone();
        if copies.is_empty() {
            return false;
        }
        for copy in &mut copies {
            copy.id = self.alloc_id();
        }
        self.selection = copies.iter().map(|o| o.id).collect();
        self.put_edit_members(copies);
        true
    }

    /// Ends the edit of a symbol: what lies on the page becomes the symbol, and every
    /// copy of it follows. Returns whether a symbol was being edited.
    pub fn leave_symbol(&mut self) -> bool {
        if self.editing_symbol().is_none() {
            return false;
        }
        self.end_mask_edit();
        self.close_groups();
        let members = self.take_edit_members();
        let Some(mut edit) = self.doc.page_mut().symbol_edit.take() else { return false };
        edit.looks[edit.look] = members;
        for object in self.doc.page_mut().layers.iter_mut().flat_map(|l| l.objects.iter_mut()) {
            if edit.locked.contains(&object.id) {
                object.locked = false;
            }
        }
        self.selection.clear();
        let union = |objects: &[Object]| objects.iter().map(|o| o.bounds).reduce(Rect::union);
        let now = union(&edit.looks[0]).or_else(|| union(&edit.looks.concat()));
        let id = edit.symbol;
        let Some(now) = now else {
            // Emptied: the symbol stays in the library with nothing in it, and the copy that was opened is gone.
            if let Some(symbol) = self.doc.symbols.iter_mut().find(|s| s.id == id) {
                (symbol.objects, symbol.looks) = (Vec::new(), Vec::new());
            }
            self.doc.relink_symbols();
            return true;
        };
        let mut looks: Vec<Vec<Object>> = edit
            .looks
            .iter()
            .map(|look| {
                let mut look = look.clone();
                for object in &mut look {
                    object.translate(-now.x, -now.y);
                }
                look
            })
            .collect();
        let up = looks.remove(0);
        while looks.last().is_some_and(Vec::is_empty) {
            looks.pop();
        }
        if let Some(symbol) = self.doc.symbols.iter_mut().find(|s| s.id == id) {
            (symbol.objects, symbol.looks) = (up, looks);
        }
        // The other copies keep their own scale: they grow and move as the symbol did.
        let before = edit.frame;
        if before.w > 0.0 && before.h > 0.0 {
            for object in self.doc.pages.iter_mut().flat_map(Page::all_objects_mut) {
                if !matches!(object.shape, Shape::Instance { symbol, .. } if symbol == id) {
                    continue;
                }
                let frame = object.placement.map_or(object.bounds, |p| p.frame);
                let (kx, ky) = (frame.w / before.w, frame.h / before.h);
                let fitted = Rect { x: frame.x + (now.x - before.x) * kx, y: frame.y + (now.y - before.y) * ky, w: now.w * kx, h: now.h * ky };
                match &mut object.placement {
                    Some(placement) => {
                        placement.frame = fitted;
                        object.bounds = placement.bounds();
                    }
                    None => object.bounds = fitted,
                }
            }
        }
        let copy = match edit.shell {
            Some(mut shell) => {
                match &mut shell.placement {
                    Some(placement) => {
                        placement.frame = now;
                        shell.bounds = placement.bounds();
                    }
                    None => shell.bounds = now,
                }
                Some(shell)
            }
            None if edit.place => Some(self.new_instance(id, now)),
            None => None,
        };
        if let Some(copy) = copy {
            self.selection = vec![copy.id];
            let page = self.doc.page_mut();
            let layer = edit.spot.0.min(page.layers.len() - 1);
            let at = edit.spot.1.min(page.layers[layer].objects.len());
            page.layers[layer].objects.insert(at, copy);
        }
        self.doc.relink_symbols();
        true
    }

    /// Ends a symbol edit as its own undo step, before something that cannot go on inside one.
    pub fn leave_symbol_for_good(&mut self) {
        if self.leave_symbol() {
            self.commit(t("Edit Symbol"));
        }
    }

    /// Whether Tween Instances has what it needs: two or more selected copies of one symbol.
    pub fn can_tween(&self) -> bool {
        let symbols: Vec<u64> = self.selected().iter().filter_map(|o| match o.shape {
            Shape::Instance { symbol, .. } => Some(symbol),
            _ => None,
        }).collect();
        symbols.len() >= 2 && symbols.len() == self.selection.len() && symbols.iter().all(|s| *s == symbols[0])
    }

    /// Fireworks' Tween Instances: between each selected copy of a symbol and the next
    /// one above it, `steps` further copies are made that go from the one to the other in
    /// place, size, turn and opacity. With `distribute`, the whole run is then dealt out
    /// to the states, one copy to each, from the state on show; states are added as needed.
    pub fn tween_instances(&mut self, steps: u32, distribute: bool) -> usize {
        if !self.can_tween() {
            return 0;
        }
        let steps = steps.clamp(1, 500) as usize;
        let ends = self.selected_in_order();
        let pose = |object: &Object| {
            let frame = object.placement.map_or(object.bounds, |p| p.frame);
            (object.bounds.center(), frame.w, frame.h, object.placement.map_or(0.0, |p| p.warp.turn()), object.opacity)
        };
        let mut run: Vec<Object> = Vec::new();
        let mut made = 0;
        for pair in ends.windows(2) {
            let (from, to) = (pose(&pair[0]), pose(&pair[1]));
            // The turn goes the short way round.
            let mut sweep = to.3 - from.3;
            if sweep > std::f32::consts::PI {
                sweep -= std::f32::consts::TAU;
            } else if sweep < -std::f32::consts::PI {
                sweep += std::f32::consts::TAU;
            }
            run.push(pair[0].clone());
            for step in 1..=steps {
                let part = step as f32 / (steps + 1) as f32;
                let mix = |a: f32, b: f32| a + (b - a) * part;
                let center = v2(mix(from.0.x, to.0.x), mix(from.0.y, to.0.y));
                let (w, h) = (mix(from.1, to.1), mix(from.2, to.2));
                let mut copy = pair[0].clone();
                copy.id = self.alloc_id();
                copy.placement = None;
                copy.set_bounds(Rect { x: center.x - w / 2.0, y: center.y - h / 2.0, w, h });
                let angle = from.3 + sweep * part;
                if angle.abs() > 0.0001 {
                    copy.warp(&Warp::rotation(center, angle));
                }
                copy.opacity = mix(from.4, to.4);
                run.push(copy);
                made += 1;
            }
        }
        run.extend(ends.last().cloned());
        let ids: Vec<u64> = ends.iter().map(|o| o.id).collect();
        let page = self.doc.page_mut();
        let Some((layer, at)) = page.layers.iter().enumerate().find_map(|(l, layer)| layer.objects.iter().position(|o| ids.contains(&o.id)).map(|i| (l, i))) else { return 0 };
        for each in &mut page.layers {
            each.objects.retain(|o| !ids.contains(&o.id));
        }
        if !distribute {
            self.selection = run.iter().map(|o| o.id).collect();
            let at = at.min(page.layers[layer].objects.len());
            page.layers[layer].objects.splice(at..at, run);
            return made;
        }
        let first = page.active_state;
        while page.states.len() < first + run.len() {
            let mut state = State::new(tf("State {}", &[&(page.states.len() + 1)]));
            state.objects = vec![Vec::new(); page.layers.len()];
            page.states.push(state);
        }
        self.selection = run.first().map(|o| o.id).into_iter().collect();
        for (index, object) in run.into_iter().enumerate() {
            if let Some(objects) = self.doc.page_mut().state_layer_mut(first + index, layer) {
                objects.push(object);
            }
        }
        made
    }

    /// Shows another frame of the scene. Like switching state or page, this is not a step to undo.
    pub fn goto_frame(&mut self, frame: usize) {
        if self.doc.page().timeline.as_ref().is_some_and(|timeline| timeline.current != frame) {
            self.leave_symbol_for_good();
            self.end_mask_edit();
            self.close_groups();
            self.navigate(|doc| doc.page_mut().goto_frame(frame));
            self.selection.clear();
        }
    }

    /// Changes the timeline at the active layer and the frame on show. Returns what the change returned.
    pub fn on_timeline<T>(&mut self, change: impl FnOnce(&mut Timeline, usize, usize) -> T) -> Option<T> {
        self.leave_symbol();
        self.end_mask_edit();
        self.close_groups();
        let page = self.doc.page_mut();
        let (layer, frame) = (page.active_layer, page.timeline.as_ref()?.current);
        let done = page.edit_timeline(|timeline| change(timeline, layer, frame));
        self.prune_selection();
        done
    }

    /// The animation a selected object belongs to.
    pub fn selected_animation(&self) -> Option<usize> {
        self.doc.page().animations.iter().position(|a| a.objects.iter().any(|id| self.selection.contains(id)))
    }

    /// Fireworks' Animate Selection: the selection becomes an animation symbol with a
    /// copy in each of a run of states, moved, resized, turned and faded step by step.
    /// States are added if the page has too few.
    pub fn animate_selection(&mut self, settings: AnimationSettings) -> bool {
        if self.selected_animation().is_some() {
            return self.set_animation(settings);
        }
        self.leave_symbol();
        self.end_mask_edit();
        self.close_groups();
        let one_instance = match self.selected().as_slice() {
            [object] => match object.shape {
                Shape::Instance { symbol, .. } => Some(symbol),
                _ => None,
            },
            _ => None,
        };
        let Some(symbol) = one_instance.or_else(|| self.make_symbol()) else { return false };
        if let Some(entry) = self.doc.symbols.iter_mut().find(|s| s.id == symbol) {
            entry.kind = SymbolKind::Animation;
        }
        let Some(start) = self.selected().first().map(|o| (*o).clone()) else { return false };
        let page = self.doc.page_mut();
        let Some(layer) = page.layers.iter().position(|l| l.objects.iter().any(|o| o.id == start.id)) else { return false };
        page.animations.push(Animation { symbol, settings, first_state: page.active_state, layer, objects: vec![start.id] });
        self.lay_animation(self.doc.page().animations.len() - 1, start);
        true
    }

    /// Changes the settings of the animation the selection belongs to.
    pub fn set_animation(&mut self, settings: AnimationSettings) -> bool {
        let Some(index) = self.selected_animation() else { return false };
        let Some(start) = self.animation_start(index) else { return false };
        self.doc.page_mut().animations[index].settings = settings;
        self.lay_animation(index, start);
        true
    }

    /// The copy an animation starts from, as it would be with nothing done to it.
    fn animation_start(&self, index: usize) -> Option<Object> {
        let page = self.doc.page();
        let animation = page.animations.get(index)?;
        let first = *animation.objects.first()?;
        let mut start = page.objects().chain(page.states.iter().flat_map(|s| s.objects.iter().flatten())).find(|o| o.id == first)?.clone();
        start.opacity = 1.0;
        Some(start)
    }

    /// Puts the copies of an animation into its states afresh, from the copy it starts with.
    fn lay_animation(&mut self, index: usize, start: Object) {
        let animation = self.doc.page().animations[index].clone();
        let settings = animation.settings;
        let count = settings.states.clamp(1, 250) as usize;
        let page = self.doc.page_mut();
        page.retain_all(|o| !animation.objects.contains(&o.id));
        while page.states.len() < animation.first_state + count {
            let mut state = State::new(tf("State {}", &[&(page.states.len() + 1)]));
            state.objects = vec![Vec::new(); page.layers.len()];
            page.states.push(state);
        }
        let mut ids = Vec::new();
        for step in 0..count {
            let mut copy = settings.pose(&start, step);
            copy.id = if step == 0 { start.id } else { self.alloc_id() };
            ids.push(copy.id);
            if let Some(objects) = self.doc.page_mut().state_layer_mut(animation.first_state + step, animation.layer) {
                objects.push(copy);
            }
        }
        self.doc.page_mut().animations[index].objects = ids.clone();
        self.selection = ids.into_iter().filter(|id| self.doc.page().objects().any(|o| o.id == *id)).collect();
    }

    /// Moves the copies of an animation in every state. With `others`, only the copies
    /// that are not selected: the selected one has been moved by hand already.
    pub fn shift_animation(&mut self, index: usize, dx: f32, dy: f32, others: bool) {
        let Some(animation) = self.doc.page().animations.get(index) else { return };
        let ids: Vec<u64> = animation.objects.iter().copied().filter(|id| !others || !self.selection.contains(id)).collect();
        for object in self.doc.page_mut().all_objects_mut().filter(|o| ids.contains(&o.id)) {
            object.translate(dx, dy);
        }
    }

    /// Fireworks' Remove Animation: the copy in the first state stays, as an ordinary copy of the symbol.
    pub fn remove_animation(&mut self) -> bool {
        let Some(index) = self.selected_animation() else { return false };
        let animation = self.doc.page_mut().animations.remove(index);
        let rest: Vec<u64> = animation.objects.iter().skip(1).copied().collect();
        self.doc.page_mut().retain_all(|o| !rest.contains(&o.id));
        if let Some(entry) = self.doc.symbols.iter_mut().find(|s| s.id == animation.symbol) {
            entry.kind = SymbolKind::Graphic;
        }
        self.selection.retain(|id| !rest.contains(id));
        true
    }

    /// The middles of an animation's copies, state by state: the path Fireworks draws for it.
    pub fn animation_path(&self, index: usize) -> Vec<Vec2> {
        let Some(animation) = self.doc.page().animations.get(index) else { return Vec::new() };
        let Some(start) = self.animation_start(index) else { return Vec::new() };
        (0..animation.settings.states.max(1) as usize).map(|step| animation.settings.center(start.bounds.center(), step)).collect()
    }

    /// Replaces selected symbol copies with their own objects. Returns how many were broken.
    pub fn break_apart(&mut self) -> usize {
        let selected: Vec<Object> = self.selected().into_iter().cloned().collect();
        let count = selected.iter().filter(|o| o.is_instance()).count();
        if count == 0 {
            return 0;
        }
        let mut replacement = Vec::new();
        for object in selected {
            if object.is_instance() {
                for mut part in object.instance_parts() {
                    part.id = self.alloc_id();
                    replacement.push(part);
                }
            } else {
                replacement.push(object);
            }
        }
        self.replace_selected(replacement);
        count
    }

    /// Makes a symbol's contents match the selection and updates every placed copy.
    pub fn redefine_symbol(&mut self, id: u64) -> bool {
        let objects: Vec<Object> = self.selected().into_iter().filter(|o| !o.is_web()).cloned().collect();
        let uses_self = objects.iter().any(|o| object_uses(&self.doc.symbols, o, id, 0));
        if objects.is_empty() || uses_self || !self.doc.symbols.iter().any(|s| s.id == id) {
            return false;
        }
        let parts = normalized(&objects);
        if let Some(symbol) = self.doc.symbols.iter_mut().find(|s| s.id == id) {
            symbol.objects = parts;
        }
        self.doc.relink_symbols();
        true
    }

    /// Combines the selected objects into one group, in their stacking order.
    /// The selected objects, bottom-most first.
    fn selected_in_order(&self) -> Vec<Object> {
        self.doc.page().objects().filter(|o| self.is_selected(o.id)).cloned().collect()
    }

    /// One object standing for `objects`: the object itself if there is only one, a group otherwise.
    fn one_object(&mut self, mut objects: Vec<Object>) -> Option<Object> {
        match objects.len() {
            0 => None,
            1 => objects.pop(),
            _ => self.group_of(objects),
        }
    }

    /// Fireworks' Group as Mask: the top-most selected object masks the others. A vector
    /// on top masks by how light it is drawn, as a bitmap does.
    pub fn group_as_mask(&mut self) -> bool {
        let mut objects = self.selected_in_order();
        if objects.len() < 2 {
            return false;
        }
        let Some(mask) = objects.pop() else { return false };
        let Some(mut masked) = self.one_object(objects) else { return false };
        masked.mask = Some(Arc::new(ObjectMask::new(mask, true)));
        self.replace_selected(vec![masked]);
        true
    }

    /// Fireworks' Paste as Mask: what is on the clipboard masks each selected object,
    /// a vector by its outline and a bitmap by how light it is.
    ///
    /// With `as_bitmap` the clipboard is turned into one bitmap first. With `add`, an
    /// object that already has a mask keeps it, and the new one is joined to it.
    pub fn paste_as_mask(&mut self, as_bitmap: bool, add: bool) -> bool {
        let clipboard = self.clipboard.clone();
        if self.selection.is_empty() {
            return false;
        }
        let Some(mut mask) = self.one_object(clipboard) else { return false };
        if as_bitmap && !mask.is_image() {
            let Some((bitmap, bounds)) = crate::export::render_object(&mask) else { return false };
            mask = Object::new_bitmap(self.alloc_id(), t("Mask"), bitmap, bounds.x, bounds.y);
        }
        let ids = self.selection.clone();
        for id in ids {
            let existing = self.object(id).and_then(|o| o.mask.clone()).filter(|_| add);
            let new_mask = match existing {
                Some(existing) => {
                    let Some(joined) = self.group_of(vec![existing.object.clone(), mask.clone()]) else { continue };
                    ObjectMask { object: joined, ..(*existing).clone() }
                }
                None => ObjectMask::new(mask.clone(), false),
            };
            if let Some(object) = self.object_mut(id) {
                object.mask = Some(Arc::new(new_mask));
            }
        }
        true
    }

    /// How many objects on the clipboard are vectors, which Paste as Mask asks about.
    pub fn clipboard_vectors(&self) -> usize {
        self.clipboard.iter().filter(|o| !o.is_image()).count()
    }

    /// Links the masks of the selected objects to them, or lets them go.
    pub fn toggle_mask_links(&mut self) {
        self.for_selected(|o| {
            if let Some(mask) = &mut o.mask {
                let mask = Arc::make_mut(mask);
                mask.linked = !mask.linked;
            }
        });
    }

    /// Fireworks' Paste Inside: what is on the clipboard is pasted and shows only inside
    /// the selected object, which stays visible around it.
    pub fn paste_inside(&mut self) -> bool {
        let mut pasted = self.clipboard.clone();
        let Some(frame) = self.selected_in_order().pop() else { return false };
        for object in &mut pasted {
            object.id = self.alloc_id();
        }
        let Some(mut inside) = self.one_object(pasted) else { return false };
        inside.mask = Some(Arc::new(ObjectMask { object: frame.clone(), kind: MaskKind::PathOutline, show_fill_stroke: true, enabled: true, linked: true }));
        self.selection = vec![frame.id];
        self.replace_selected(vec![inside]);
        true
    }

    /// Gives each selected object a bitmap mask that shows all of it or none. With
    /// `pixels`, the selected pixels of a bitmap are what shows, or what is hidden.
    pub fn add_bitmap_mask(&mut self, reveal: bool, pixels: Option<&crate::bitmap::Mask>) -> bool {
        let ids = self.selection.clone();
        let mut done = false;
        for id in ids {
            let Some(bounds) = self.object(id).map(|o| o.bounds) else { continue };
            let (width, height) = pixels.map_or(((bounds.w.ceil() as u32).max(1), (bounds.h.ceil() as u32).max(1)), |mask| mask.size());
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for y in 0..height {
                for x in 0..width {
                    let shown = pixels.map_or(reveal, |mask| mask.is_set(x, y) == reveal);
                    let level = if shown { 255 } else { 0 };
                    rgba.extend_from_slice(&[level, level, level, 255]);
                }
            }
            let Some(bitmap) = Bitmap::from_rgba(width, height, rgba) else { continue };
            let mut mask = Object::new_bitmap(self.alloc_id(), t("Mask"), bitmap, bounds.x, bounds.y);
            mask.set_bounds(bounds);
            if let Some(object) = self.object_mut(id) {
                object.mask = Some(Arc::new(ObjectMask::new(mask, false)));
                done = true;
            }
        }
        done
    }

    /// Starts editing the mask of the selected object: a stand-in for the mask is put
    /// above it and selected, for the tools to work on. Returns false without a mask.
    pub fn edit_mask(&mut self) -> bool {
        self.end_mask_edit();
        let Some(owner) = self.selected().into_iter().find(|o| o.mask.is_some()).cloned() else { return false };
        let Some(mask) = &owner.mask else { return false };
        let mut stand_in = mask.object.clone();
        stand_in.id = self.alloc_id();
        stand_in.mask_of = Some(owner.id);
        (stand_in.visible, stand_in.locked) = (true, false);
        let id = stand_in.id;
        for layer in &mut self.doc.page_mut().layers {
            if let Some(at) = layer.objects.iter().position(|o| o.id == owner.id) {
                layer.objects.insert(at + 1, stand_in);
                self.selection = vec![id];
                return true;
            }
        }
        false
    }

    /// Whether a mask is being edited.
    pub fn editing_mask(&self) -> bool {
        self.doc.page().objects().any(|o| o.mask_of.is_some())
    }

    /// Copies every mask stand-in back into the mask it stands for, so the artwork
    /// follows the edit. Stand-ins that are no longer selected are done with and removed.
    pub fn sync_mask_edit(&mut self) {
        if !self.editing_mask() {
            return;
        }
        let selection = self.selection.clone();
        let stand_ins: Vec<Object> = self.doc.page().objects().filter(|o| o.mask_of.is_some()).cloned().collect();
        for stand_in in &stand_ins {
            let Some(owner) = stand_in.mask_of.and_then(|id| self.object_mut(id)) else { continue };
            let Some(mask) = &mut owner.mask else { continue };
            let mut edited = stand_in.clone();
            (edited.id, edited.mask_of) = (mask.object.id, None);
            if mask.object != edited {
                Arc::make_mut(mask).object = edited;
            }
        }
        let finished: Vec<u64> = stand_ins.iter().filter(|o| !selection.contains(&o.id)).map(|o| o.id).collect();
        if !finished.is_empty() {
            for layer in &mut self.doc.page_mut().layers {
                layer.objects.retain(|o| !finished.contains(&o.id));
            }
        }
    }

    /// Finishes any mask edit, keeping what was done.
    pub fn end_mask_edit(&mut self) {
        if self.editing_mask() {
            let stand_ins: Vec<u64> = self.doc.page().objects().filter(|o| o.mask_of.is_some()).map(|o| o.id).collect();
            self.selection.retain(|id| !stand_ins.contains(id));
            self.sync_mask_edit();
        }
    }

    /// Switches the masks of the selected objects off or back on.
    pub fn toggle_masks(&mut self) {
        self.for_selected(|o| {
            if let Some(mask) = &mut o.mask {
                let mask = Arc::make_mut(mask);
                mask.enabled = !mask.enabled;
            }
        });
    }

    /// Removes the masks of the selected objects. With `apply`, each object first
    /// becomes the bitmap of what its mask let show.
    pub fn delete_masks(&mut self, apply: bool) {
        self.for_selected(|o| {
            if o.mask.is_none() {
                return;
            }
            if apply {
                if let Some((bitmap, bounds)) = crate::export::render_object(o) {
                    o.shape = Shape::Image { bitmap: Arc::new(bitmap) };
                    o.bounds = bounds;
                    (o.fill, o.stroke, o.placement, o.opacity) = (None, None, None, 1.0);
                    o.points.clear();
                    o.handles.clear();
                }
            }
            o.mask = None;
        });
    }

    pub fn group_selected(&mut self) -> bool {
        let objects = self.selected_in_order();
        let Some(group) = self.group_of(objects).filter(|group| matches!(&group.shape, Shape::Group { objects } if objects.len() >= 2)) else {
            return false;
        };
        self.replace_selected(vec![group]);
        true
    }

    /// A group holding `objects`.
    fn group_of(&mut self, objects: Vec<Object>) -> Option<Object> {
        let bounds = objects.iter().map(|o| o.bounds).reduce(Rect::union)?;
        Some(Object {
            id: self.alloc_id(),
            name: tc("object", "Group").into(),
            shape: Shape::Group { objects: Arc::new(objects) },
            bounds,
            points: Vec::new(),
            handles: Vec::new(),
            placement: None,
            pivot: None,

            behaviors: Vec::new(),
            attached: String::new(),
            blend: crate::blend::BlendMode::Normal,
            mask: None,
            mask_of: None,
            fill_style: None,
            stroke_profile: Vec::new(),
            fill_look: Default::default(),
            link: String::new(),
            filters: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        })
    }

    /// Turns the selection a quarter turn around its center. Returns how many objects could not turn.
    pub fn rotate_selected(&mut self, clockwise: bool) -> usize {
        let Some(bounds) = self.selection_bounds() else { return 0 };
        let center = bounds.center();
        let mut skipped = 0;
        self.for_selected(|o| {
            if !o.rotate90(clockwise, center) {
                skipped += 1;
            }
        });
        skipped
    }

    /// Changes the canvas size, keeping the artwork centered.
    pub fn resize_canvas(&mut self, width: u32, height: u32) {
        let page = self.doc.page_mut();
        let dx = ((width as f32 - page.width as f32) / 2.0).round();
        let dy = ((height as f32 - page.height as f32) / 2.0).round();
        page.width = width;
        page.height = height;
        page.all_objects_mut().for_each(|o| o.translate(dx, dy));
    }

    /// Scales the canvas and all of its artwork to a new size.
    pub fn resize_image(&mut self, width: u32, height: u32) {
        let page = self.doc.page_mut();
        let sx = width as f32 / page.width.max(1) as f32;
        let sy = height as f32 / page.height.max(1) as f32;
        page.width = width;
        page.height = height;
        for object in page.all_objects_mut() {
            let b = object.bounds;
            object.set_bounds(Rect { x: b.x * sx, y: b.y * sy, w: b.w * sx, h: b.h * sy });
            if let Shape::Text { size, .. } = &mut object.shape {
                *size *= sy;
            }
            if let Some(stroke) = &mut object.stroke {
                stroke.width *= (sx + sy) / 2.0;
            }
        }
    }

    /// Fits the canvas exactly around all artwork. Returns false on an empty page.
    pub fn fit_canvas(&mut self) -> bool {
        let page = self.doc.page_mut();
        let Some(area) = page.all_objects_mut().filter(|o| !o.is_web()).map(|o| o.bounds).reduce(Rect::union) else {
            return false;
        };
        let (dx, dy) = (-area.x.floor(), -area.y.floor());
        page.width = (area.right() + dx).ceil().max(1.0) as u32;
        page.height = (area.bottom() + dy).ceil().max(1.0) as u32;
        page.all_objects_mut().for_each(|o| o.translate(dx, dy));
        true
    }

    /// Removes a symbol and every copy of it, including copies inside other symbols.
    /// The symbol whose movie clip is being edited.
    pub fn editing_clip(&self) -> Option<u64> {
        self.doc.page().clip_of
    }

    /// Opens the timeline of a movie clip: the clip gets a page of its own, where every tool
    /// and the whole timeline work on it as they do on a scene. The first time, the page
    /// starts with what the symbol holds, in the middle.
    pub fn open_clip(&mut self, symbol: u64) -> bool {
        self.leave_symbol();
        self.end_mask_edit();
        self.close_groups();
        self.leave_clip();
        let Some(entry) = self.doc.symbols.iter().find(|s| s.id == symbol).cloned() else { return false };
        let back = self.doc.active_page;
        let index = match self.doc.clip_page(symbol) {
            Some(index) => index,
            None => {
                let scene = self.doc.page();
                let (fps, middle) = (scene.timeline.as_ref().map_or(24.0, |timeline| timeline.fps), scene.rect().center());
                let mut page = Page::new(entry.name.clone(), scene.width, scene.height);
                page.canvas = scene.canvas;
                let natural = entry.objects.iter().map(|o| o.bounds).reduce(Rect::union).unwrap_or_default();
                let origin = v2((middle.x - natural.w / 2.0).round(), (middle.y - natural.h / 2.0).round());
                let mut parts = entry.objects.clone();
                for part in &mut parts {
                    link_object(&self.doc.symbols, part, 1);
                    part.translate(origin.x - natural.x, origin.y - natural.y);
                }
                page.layers[0].objects = parts;
                (page.clip_of, page.clip_origin) = (Some(symbol), origin);
                page.start_timeline(fps);
                self.doc.pages.push(page);
                self.doc.pages.len() - 1
            }
        };
        self.doc.clip_return = Some(back);
        self.doc.active_page = index;
        self.selection.clear();
        true
    }

    /// Writes the first frame of the clip being edited into its symbol, so that every copy on the
    /// stage shows it, and keeps the copies the size and place they were. The clip's page stays open.
    pub fn sync_clip(&mut self) {
        let Some(symbol) = self.editing_clip() else { return };
        let page = self.doc.page_mut();
        page.store_frame();
        let first: Vec<Object> = page.at_frame(0).played().objects().filter(|o| o.visible && !o.is_web()).cloned().collect();
        let before_origin = page.clip_origin;
        let now = first.iter().map(|o| o.bounds).reduce(Rect::union);
        let Some(entry) = self.doc.symbols.iter_mut().find(|s| s.id == symbol) else { return };
        let before = entry.objects.iter().map(|o| o.bounds).reduce(Rect::union).unwrap_or_default();
        entry.objects = normalized(&first);
        let Some(now) = now else {
            self.doc.relink_symbols();
            return;
        };
        self.doc.page_mut().clip_origin = v2(now.x, now.y);
        // The copies keep their own scale: they grow and move as the clip's first frame did.
        if before.w > 0.0 && before.h > 0.0 {
            for object in self.doc.pages.iter_mut().flat_map(Page::all_objects_mut) {
                if !matches!(object.shape, Shape::Instance { symbol: of, .. } if of == symbol) {
                    continue;
                }
                let frame = object.placement.map_or(object.bounds, |p| p.frame);
                let (kx, ky) = (frame.w / before.w, frame.h / before.h);
                let fitted = Rect { x: frame.x + (now.x - before_origin.x) * kx, y: frame.y + (now.y - before_origin.y) * ky, w: now.w * kx, h: now.h * ky };
                match &mut object.placement {
                    Some(placement) => {
                        placement.frame = fitted;
                        object.bounds = placement.bounds();
                    }
                    None => object.bounds = fitted,
                }
            }
        }
        self.doc.relink_symbols();
    }

    /// Ends the edit of a movie clip and goes back to the scene it was opened from. Returns whether one was being edited.
    pub fn leave_clip(&mut self) -> bool {
        if self.editing_clip().is_none() {
            return false;
        }
        self.leave_symbol();
        self.end_mask_edit();
        self.close_groups();
        self.sync_clip();
        let scenes = self.doc.scenes();
        let back = self.doc.clip_return.take().filter(|index| scenes.contains(index)).or(scenes.first().copied()).unwrap_or(0);
        self.doc.active_page = back;
        self.selection.clear();
        true
    }

    /// Brings what another document shows into this one: the objects of its page on show go
    /// onto the active layer, selected, and the symbols they use into the library, movie
    /// clips with their timelines. Everything gets numbers of this document's own.
    /// Returns how many objects came in.
    pub fn import_document(&mut self, other: &Document) -> usize {
        self.leave_symbol();
        self.end_mask_edit();
        self.close_groups();
        let mut symbols: HashMap<u64, u64> = HashMap::new();
        for symbol in &other.symbols {
            symbols.insert(symbol.id, self.alloc_id());
        }
        // An object with a number of this document's, its copies of symbols pointed at the symbols brought in.
        fn renumbered(object: &Object, symbols: &HashMap<u64, u64>, next: &mut u64) -> Object {
            let mut object = object.clone();
            object.id = *next;
            *next += 1;
            match &mut object.shape {
                Shape::Instance { symbol, .. } => *symbol = symbols.get(symbol).copied().unwrap_or(*symbol),
                Shape::Group { objects } => *objects = Arc::new(objects.iter().map(|part| renumbered(part, symbols, next)).collect()),
                _ => {}
            }
            object
        }
        let mut next = self.doc.next_id;
        for symbol in &other.symbols {
            let mut copy = symbol.clone();
            copy.id = symbols[&symbol.id];
            copy.objects = symbol.objects.iter().map(|o| renumbered(o, &symbols, &mut next)).collect();
            copy.looks = symbol.looks.iter().map(|look| look.iter().map(|o| renumbered(o, &symbols, &mut next)).collect()).collect();
            self.doc.symbols.push(copy);
        }
        // The pages that hold the timelines of movie clips come along with their clips.
        for page in other.pages.iter().filter(|page| page.clip_of.is_some()) {
            let mut copy = page.clone();
            copy.clip_of = page.clip_of.and_then(|id| symbols.get(&id).copied());
            for layer in &mut copy.layers {
                layer.objects = layer.objects.iter().map(|o| renumbered(o, &symbols, &mut next)).collect();
            }
            if let Some(timeline) = &mut copy.timeline {
                for span in timeline.layers.iter_mut().flatten() {
                    span.objects = span.objects.iter().map(|o| renumbered(o, &symbols, &mut next)).collect();
                }
            }
            for slot in copy.states.iter_mut().flat_map(|s| s.objects.iter_mut()) {
                *slot = slot.iter().map(|o| renumbered(o, &symbols, &mut next)).collect();
            }
            self.doc.pages.push(copy);
        }
        let shown: Vec<Object> = other.page().layers.iter().filter(|l| l.visible).flat_map(|l| l.objects.iter()).filter(|o| o.visible && !o.is_web()).map(|o| renumbered(o, &symbols, &mut next)).collect();
        self.doc.next_id = next;
        self.doc.relink_symbols();
        let ids: Vec<u64> = shown.iter().map(|o| o.id).collect();
        let mut added = 0;
        for object in shown {
            if self.add_object(object) {
                added += 1;
            }
        }
        self.doc.relink_symbols();
        self.selection = ids;
        added
    }

    /// The document as the other program takes it. From the animation program the scenes go as plain
    /// pages showing the frame they are on; movie clips keep their pages, which PisiWorks carries along untouched.
    pub fn for_other_program(&self, plain_pages: bool) -> Document {
        let mut doc = self.doc.clone();
        doc.clip_return = None;
        for page in &mut doc.pages {
            page.store_frame();
            page.symbol_edit = None;
            if plain_pages && page.clip_of.is_none() {
                page.timeline = None;
            }
        }
        if doc.page().clip_of.is_some() {
            doc.active_page = doc.scenes().first().copied().unwrap_or(0);
        }
        doc
    }

    /// Adds a sound to the document. Returns its number.
    pub fn add_sound(&mut self, name: &str, mime: &str, bytes: &[u8]) -> u64 {
        use base64::Engine;
        let id = self.alloc_id();
        self.doc.sounds.push(Sound { id, name: name.to_string(), mime: mime.to_string(), data: Arc::new(base64::engine::general_purpose::STANDARD.encode(bytes)) });
        id
    }

    pub fn delete_symbol(&mut self, id: u64) {
        // A movie clip's own page goes with it.
        if let Some(index) = self.doc.clip_page(id) {
            if self.doc.active_page == index {
                self.leave_clip();
            }
            self.doc.pages.remove(index);
            self.doc.clip_return = None;
            if self.doc.active_page > index {
                self.doc.active_page -= 1;
            }
        }
        let is_copy = |o: &Object| matches!(o.shape, Shape::Instance { symbol, .. } if symbol == id);
        self.doc.symbols.retain(|s| s.id != id);
        for symbol in &mut self.doc.symbols {
            symbol.objects.retain(|o| !is_copy(o));
            symbol.objects.iter_mut().for_each(|o| strip_symbol(o, id));
        }
        for page in &mut self.doc.pages {
            page.retain_all(|o| !is_copy(o));
            page.all_objects_mut().for_each(|o| strip_symbol(o, id));
        }
        self.doc.relink_symbols();
        self.prune_selection();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_rect_editor() -> Editor {
        let mut e = Editor::new(Document::new(200, 200));
        e.doc.page_mut().layers[0].objects.push(rect(1, 10.0, 10.0, 20.0, 20.0));
        e.doc.page_mut().layers[0].objects.push(rect(2, 60.0, 40.0, 20.0, 10.0));
        e.doc.next_id = 3;
        e.selection = vec![1, 2];
        e
    }

    #[test]
    fn group_replaces_members_and_ungroup_restores_positions() {
        let mut e = two_rect_editor();
        assert!(e.group_selected());
        assert_eq!(e.doc.page().objects().count(), 1);
        let group = e.selected()[0].clone();
        assert_eq!(group.bounds, Rect { x: 10.0, y: 10.0, w: 70.0, h: 40.0 });
        assert!(e.doc.page().objects().next().unwrap().is_instance());
        e.for_selected(|o| o.translate(5.0, 0.0));
        e.break_apart();
        let xs: Vec<f32> = e.doc.page().objects().map(|o| o.bounds.x).collect();
        assert_eq!(xs, vec![15.0, 65.0]);
    }

    #[test]
    fn group_needs_two_objects() {
        let mut e = two_rect_editor();
        e.selection = vec![1];
        assert!(!e.group_selected());
    }

    #[test]
    fn grouped_symbol_copies_survive_relinking() {
        let mut e = two_rect_editor();
        e.selection = vec![1];
        e.make_symbol().unwrap();
        e.selection = e.doc.page().objects().map(|o| o.id).collect();
        assert!(e.group_selected());
        e.doc.relink_symbols();
        assert_eq!(e.doc.page().objects().next().unwrap().leaves().len(), 2);
    }

    #[test]
    fn rotation_swaps_size_and_turns_around_the_selection() {
        let mut e = two_rect_editor();
        e.selection = vec![2];
        assert_eq!(e.rotate_selected(true), 0);
        let b = e.selected()[0].bounds;
        assert_eq!((b.w, b.h), (10.0, 20.0));
        assert_eq!(b.center(), v2(70.0, 45.0));
    }

    #[test]
    fn four_quarter_turns_restore_an_object() {
        let mut e = two_rect_editor();
        let before: Vec<Rect> = e.doc.page().objects().map(|o| o.bounds).collect();
        for _ in 0..4 {
            e.rotate_selected(false);
        }
        let after: Vec<Rect> = e.doc.page().objects().map(|o| o.bounds).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn text_stays_text_when_it_turns() {
        let mut text = rect(1, 0.0, 0.0, 40.0, 10.0);
        text.shape = Shape::Text { content: "a".into(), size: 12.0, style: TextStyle::default() };
        assert!(text.rotate90(true, v2(0.0, 0.0)));
        assert!(matches!(&text.shape, Shape::Text { content, .. } if content == "a"));
        // A quarter turn clockwise around the origin stands the box on its end, left of the origin.
        assert_eq!(text.bounds, Rect { x: -10.0, y: 0.0, w: 10.0, h: 40.0 });
        assert_eq!(text.placement.unwrap().frame, Rect { x: 0.0, y: 0.0, w: 40.0, h: 10.0 });
    }

    #[test]
    fn canvas_size_keeps_artwork_centered() {
        let mut e = two_rect_editor();
        e.resize_canvas(300, 100);
        assert_eq!((e.doc.page().width, e.doc.page().height), (300, 100));
        let first = e.doc.page().objects().next().unwrap().bounds;
        assert_eq!((first.x, first.y), (60.0, -40.0));
    }

    #[test]
    fn image_size_scales_artwork() {
        let mut e = two_rect_editor();
        e.resize_image(400, 100);
        let first = e.doc.page().objects().next().unwrap().bounds;
        assert_eq!(first, Rect { x: 20.0, y: 5.0, w: 40.0, h: 10.0 });
    }

    /// An editor with a 100 by 80 page and three boxes on it: red, green over it, and blue apart.
    fn three_boxes() -> Editor {
        let mut e = Editor::new(Document::new(100, 80));
        for (x, color) in [(10.0, Color::rgb(255, 0, 0)), (20.0, Color::rgb(0, 255, 0)), (60.0, Color::rgb(0, 0, 255))] {
            let id = e.alloc_id();
            let mut object = rect(id, x, 10.0, 20.0, 20.0);
            object.fill = Some(color);
            e.add_object(object);
        }
        e
    }

    fn ids(e: &Editor) -> Vec<u64> {
        e.doc.page().objects().map(|o| o.id).collect()
    }

    #[test]
    fn guides_are_found_and_snapped_to() {
        let guides = Guides { horizontal: vec![50.0, 120.0], vertical: vec![30.0] };
        assert!(!guides.is_empty() && Guides::default().is_empty());
        // The nearest guide within reach is the one under the mouse.
        assert_eq!(guides.near(v2(200.0, 52.0), 3.0), Some((true, 0)));
        assert_eq!(guides.near(v2(31.0, 80.0), 3.0), Some((false, 0)));
        assert_eq!(guides.near(v2(31.0, 52.0), 3.0), Some((false, 0)), "the vertical one is a pixel nearer");
        assert_eq!(guides.near(v2(200.0, 80.0), 3.0), None);
        // A box moves the shortest way that puts an edge or its middle on a target.
        let (xs, ys) = snap_places(Rect { x: 10.0, y: 46.0, w: 30.0, h: 10.0 });
        assert_eq!((xs, ys), ([10.0, 25.0, 40.0], [46.0, 51.0, 56.0]));
        assert_eq!(snap_offset(&ys, &guides.horizontal, 5.0), Some(-1.0), "its middle is nearest");
        assert_eq!(snap_offset(&xs, &guides.vertical, 5.0), Some(5.0));
        assert_eq!(snap_offset(&xs, &guides.vertical, 4.0), None);
        assert_eq!(snap_offset(&xs, &[], 5.0), None);
        // Guides are kept in the file, and a page without any is written as before.
        let mut page = Page::new("p", 10, 10);
        assert!(!serde_json::to_string(&page).unwrap().contains("guides"));
        page.guides = guides.clone();
        let back: Page = serde_json::from_str(&serde_json::to_string(&page).unwrap()).unwrap();
        assert_eq!(back.guides, guides);
    }

    #[test]
    fn smart_guides_line_up_with_the_other_objects_and_the_page() {
        let e = three_boxes();
        let all = ids(&e);
        // Moving the blue box, the red and green ones and the page give the places to line up with.
        let (across, down) = e.doc.page().smart_targets(&[all[2]]);
        for x in [10.0, 20.0, 30.0, 40.0, 0.0, 50.0, 100.0] {
            assert!(across.contains(&x), "{x}");
        }
        assert!(!across.contains(&60.0) && !across.contains(&80.0), "its own edges are not");
        assert!(down.contains(&10.0) && down.contains(&20.0) && down.contains(&30.0) && down.contains(&40.0) && down.contains(&80.0));
        // A hidden object is nothing to line up with.
        let mut hidden = three_boxes();
        hidden.selection = vec![all[0]];
        hidden.for_selected(|o| o.visible = false);
        assert!(!hidden.doc.page().smart_targets(&[all[2]]).0.contains(&10.0));
    }

    #[test]
    fn subselect_opens_a_group_and_it_closes_when_the_selection_leaves() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.selection = vec![all[0], all[1]];
        assert!(e.group_selected());
        let group = e.selection[0];
        e.for_selected(|o| o.name = "Pair".into());
        assert!(!e.can_superselect());
        // Subselect puts the two members on the page and selects them.
        assert!(e.subselect());
        let members = e.selection.clone();
        assert_eq!((members.len(), ids(&e).len(), e.doc.page().open_groups.len()), (2, 3, 1));
        assert!(e.object(group).is_none() && e.can_superselect());
        assert_eq!(e.object(members[0]).unwrap().bounds, Rect { x: 10.0, y: 10.0, w: 20.0, h: 20.0 });
        // One member alone is recolored and moved; the group stays open while it is selected.
        e.selection = vec![members[1]];
        e.for_selected(|o| {
            o.fill = Some(Color::rgb(9, 9, 9));
            o.translate(0.0, 30.0);
        });
        e.sync_open_groups();
        assert_eq!(e.doc.page().open_groups.len(), 1);
        // Selecting something else closes it: the group is back under its own name and number, with the change in it.
        e.selection = vec![all[2]];
        e.sync_open_groups();
        assert!(e.doc.page().open_groups.is_empty());
        let closed = e.object(group).expect("the group is back").clone();
        assert_eq!((closed.name.as_str(), ids(&e)), ("Pair", vec![group, all[2]]));
        assert_eq!(closed.bounds, Rect { x: 10.0, y: 10.0, w: 30.0, h: 50.0 });
        let parts = closed.instance_parts();
        assert_eq!((parts[1].fill, parts[1].bounds.y), (Some(Color::rgb(9, 9, 9)), 40.0));
        // Superselect goes from a member back to the whole group at once.
        e.selection = vec![group];
        assert!(e.subselect());
        e.selection.truncate(1);
        assert!(e.superselect());
        assert_eq!(e.selection, vec![group]);
        assert!(e.doc.page().open_groups.is_empty() && !e.superselect());
        // A member deleted while the group is open is gone from it; with all deleted, so is the group.
        assert!(e.subselect());
        e.selection.truncate(1);
        e.delete_selected();
        e.sync_open_groups();
        assert_eq!(e.object(group).unwrap().instance_parts().len(), 1);
        e.selection = vec![group];
        assert!(e.subselect());
        e.delete_selected();
        e.sync_open_groups();
        assert_eq!(ids(&e), vec![all[2]]);
        // Nothing but a group can be subselected.
        e.selection = vec![all[2]];
        assert!(!e.subselect());
    }

    #[test]
    fn an_open_group_is_closed_before_the_page_is_left_and_keeps_its_look() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.selection = vec![all[0], all[1]];
        assert!(e.group_selected());
        let group = e.selection[0];
        // A half see-through group, twice as wide: its members come out as they show.
        e.for_selected(|o| {
            o.opacity = 0.5;
            o.set_bounds(Rect { x: 10.0, y: 10.0, w: 60.0, h: 20.0 });
        });
        assert!(e.subselect());
        let member = e.object(e.selection[0]).unwrap().clone();
        assert_eq!((member.opacity, member.bounds), (0.5, Rect { x: 10.0, y: 10.0, w: 40.0, h: 20.0 }));
        // Closed again, nothing is applied twice.
        e.close_groups();
        assert_eq!(e.selection, vec![group]);
        let back = e.object(group).unwrap();
        assert_eq!((back.opacity, back.bounds, back.instance_parts()[0].opacity), (1.0, Rect { x: 10.0, y: 10.0, w: 60.0, h: 20.0 }, 0.5));
        // Going to another state closes it too, and it is not written to a file while open.
        assert!(e.subselect());
        e.doc.page_mut().add_state();
        e.doc.page_mut().goto_state(0);
        e.set_active_state(1);
        e.set_active_state(0);
        assert!(e.doc.page().open_groups.is_empty() && e.object(group).is_some());
    }

    #[test]
    fn nine_slice_scaling_stretches_only_the_middle() {
        let from = Rect { x: 0.0, y: 0.0, w: 90.0, h: 30.0 };
        let to = Rect { x: 0.0, y: 0.0, w: 180.0, h: 30.0 };
        let thirds = [1.0 / 3.0, 2.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0];
        // The left third stays where it is, the right third keeps its distance from the right edge.
        assert_eq!(nine_slice_point(v2(20.0, 10.0), from, to, thirds), v2(20.0, 10.0));
        assert_eq!(nine_slice_point(v2(80.0, 10.0), from, to, thirds), v2(170.0, 10.0));
        // The middle third, 30 wide, becomes 120 wide.
        assert_eq!(nine_slice_point(v2(45.0, 15.0), from, to, thirds), v2(90.0, 15.0));
        // Made smaller than its two ends together, it scales as a whole.
        let tiny = Rect { x: 0.0, y: 0.0, w: 30.0, h: 30.0 };
        assert_eq!(nine_slice_point(v2(45.0, 15.0), from, tiny, thirds).x, 15.0);
        // A rounded rectangle keeps the size of its corners when made wider.
        let mut pill = rect(1, 0.0, 0.0, 90.0, 30.0);
        pill.shape = Shape::Rect { radius: 10.0 };
        pill.nine_slice(from, to, thirds);
        assert_eq!(pill.bounds, to);
        let outline = pill.outline();
        assert!(outline.iter().any(|p| (p.x - 10.0).abs() < 0.6 && p.y.abs() < 0.01), "the top side starts 10 in, as before");
        assert!(outline.iter().any(|p| (p.x - 170.0).abs() < 0.6 && p.y.abs() < 0.01), "and ends 10 from the new right edge");
        // The way back takes each point to where it came from.
        for x in [5.0, 29.0, 31.0, 90.0, 150.0, 175.0] {
            let there = v2(x, 12.0);
            let back = nine_slice_source(there, from, to, thirds);
            let again = nine_slice_point(back, from, to, thirds);
            assert!((again.x - x).abs() < 0.01 && (again.y - 12.0).abs() < 0.01, "{x}: {back:?} {again:?}");
        }
        // A bitmap keeps its ends pixel for pixel: red, 30 of blue, then green, made 180 wide.
        let stripes: Vec<u8> = (0..90 * 30).flat_map(|i| match i % 90 { 0..=29 => [255, 0, 0, 255], 30..=59 => [0, 0, 255, 255], _ => [0, 255, 0, 255] }).collect();
        let mut image = Object::new_bitmap(2, "b", Bitmap::from_rgba(90, 30, stripes).unwrap(), 0.0, 0.0);
        image.nine_slice(from, to, thirds);
        let wide = image.bitmap().unwrap();
        assert_eq!((image.bounds, wide.width()), (to, 180));
        assert_eq!((wide.pixel(29, 5), wide.pixel(30, 5), wide.pixel(149, 5), wide.pixel(150, 5)), (Some([255, 0, 0, 255]), Some([0, 0, 255, 255]), Some([0, 0, 255, 255]), Some([0, 255, 0, 255])));
        // In a group each member is scaled in the group's box: the one at the right end moves, and keeps its size.
        let mut e = Editor::new(Document::new(200, 100));
        for (id, x) in [(10, 0.0), (11, 70.0)] {
            e.add_object(rect(id, x, 0.0, 20.0, 30.0));
        }
        e.selection = vec![10, 11];
        assert!(e.group_selected());
        let mut group = e.selected()[0].clone();
        group.nine_slice(from, to, thirds);
        let parts = group.instance_parts();
        assert_eq!((group.bounds, parts[0].bounds, parts[1].bounds), (to, Rect { x: 0.0, y: 0.0, w: 20.0, h: 30.0 }, Rect { x: 160.0, y: 0.0, w: 20.0, h: 30.0 }));
        // Inside a polygon, and outside it.
        let triangle = [v2(0.0, 0.0), v2(10.0, 0.0), v2(0.0, 10.0)];
        assert!(point_in_polygon(v2(2.0, 2.0), &triangle) && !point_in_polygon(v2(8.0, 8.0), &triangle));
    }

    #[test]
    fn redraw_path_replaces_the_part_under_the_stroke() {
        let line: Vec<Vec2> = (0..=10).map(|i| v2(i as f32 * 10.0, 0.0)).collect();
        // From the third point to the eighth, by way of a bump.
        let bump = [v2(21.0, 1.0), v2(50.0, 30.0), v2(79.0, -1.0)];
        let redrawn = redraw_path(&line, false, &bump, 5.0).unwrap();
        assert_eq!(redrawn, [v2(0.0, 0.0), v2(10.0, 0.0), bump[0], bump[1], bump[2], v2(90.0, 0.0), v2(100.0, 0.0)]);
        // Drawn the other way it gives the same path.
        let back: Vec<Vec2> = bump.iter().rev().copied().collect();
        assert_eq!(redraw_path(&line, false, &back, 5.0).unwrap(), redrawn);
        // A stroke that starts off the path changes nothing.
        assert!(redraw_path(&line, false, &[v2(20.0, 40.0), v2(60.0, 40.0)], 5.0).is_none());
        // One that leaves an open path redraws it to the nearer end.
        let tail = redraw_path(&line, false, &[v2(80.0, 0.0), v2(90.0, 20.0), v2(120.0, 40.0)], 5.0).unwrap();
        assert_eq!((tail.len(), tail[7], tail[10]), (11, v2(70.0, 0.0), v2(120.0, 40.0)));
        // On a closed square the shorter way round is replaced.
        let square = [v2(0.0, 0.0), v2(10.0, 0.0), v2(20.0, 0.0), v2(20.0, 20.0), v2(0.0, 20.0)];
        let dented = redraw_path(&square, true, &[v2(0.0, 0.0), v2(10.0, 8.0), v2(20.0, 0.0)], 2.0).unwrap();
        assert!(dented.contains(&v2(10.0, 8.0)) && !dented.contains(&v2(10.0, 0.0)) && dented.contains(&v2(20.0, 20.0)));
        assert!(redraw_path(&square, true, &[v2(0.0, 0.0), v2(50.0, 50.0)], 2.0).is_none());
    }

    #[test]
    fn web_areas_can_be_round_or_many_sided() {
        let corners = [v2(10.0, 10.0), v2(50.0, 10.0), v2(30.0, 40.0)];
        let mut triangle = Object::new_web_polygon(1, WebKind::Slice, &corners);
        assert_eq!(triangle.bounds, Rect { x: 10.0, y: 10.0, w: 40.0, h: 30.0 });
        assert_eq!(triangle.web_outline().unwrap(), corners);
        // Moved and resized, its corners go with its box.
        triangle.set_bounds(Rect { x: 0.0, y: 0.0, w: 80.0, h: 60.0 });
        assert_eq!(triangle.web_outline().unwrap(), [v2(0.0, 0.0), v2(80.0, 0.0), v2(40.0, 60.0)]);
        let round = Object::new_web_circle(2, Rect { x: 0.0, y: 0.0, w: 20.0, h: 20.0 });
        let outline = round.web_outline().unwrap();
        assert!(outline.iter().all(|p| ((p.x - 10.0).hypot(p.y - 10.0) - 10.0).abs() < 0.01));
        assert!(Object::new_web(3, WebKind::Hotspot, Rect::default()).web_outline().is_none());
        // The form is kept in the file, and a rectangular area is written as before.
        let back: Object = serde_json::from_str(&serde_json::to_string(&triangle).unwrap()).unwrap();
        assert_eq!(back.shape, triangle.shape);
        assert!(!serde_json::to_string(&Object::new_web(3, WebKind::Hotspot, Rect::default())).unwrap().contains("form"));
    }

    #[test]
    fn text_is_attached_to_a_path_and_detached_again() {
        let mut e = three_boxes();
        let all = ids(&e);
        let text_id = e.alloc_id();
        let mut text = rect(text_id, 5.0, 5.0, 10.0, 10.0);
        text.shape = Shape::Text { content: "HH".into(), size: 12.0, style: TextStyle::default() };
        text.refit_text();
        e.add_object(text);
        // Without a path in the selection there is nothing to attach to.
        assert!(!e.attach_text_to_path());
        e.selection = vec![text_id, all[2]];
        assert!(e.attach_text_to_path());
        // The blue box is gone into the text, which now carries its outline, closed back on its start.
        assert_eq!(ids(&e), vec![all[0], all[1], text_id]);
        let attached = e.object(text_id).unwrap().clone();
        // It starts at the lower left corner, the leftmost, and goes up and over the top.
        assert_eq!((attached.points.len(), attached.points[0], attached.points[1], attached.points[4]), (5, v2(60.0, 30.0), v2(60.0, 10.0), v2(60.0, 30.0)));
        assert!(e.has_text_on_path());
        // It moves with the text, and reversing it turns the order round.
        e.for_selected(|o| o.translate(10.0, 0.0));
        assert_eq!(e.object(text_id).unwrap().points[0], v2(70.0, 30.0));
        assert!(e.reverse_text_path());
        assert_eq!(e.object(text_id).unwrap().points[1], v2(90.0, 30.0));
        // Detached, the path is a path again and the text has none.
        assert!(e.detach_text_from_path());
        assert!(!e.has_text_on_path() && e.object(text_id).unwrap().points.is_empty());
        let path = e.doc.page().objects().last().unwrap();
        assert!(matches!(path.shape, Shape::Path { closed: false }) && path.points.len() == 5 && path.stroke.is_some());
        assert!(!e.detach_text_from_path() && !e.reverse_text_path());
    }

    #[test]
    fn trim_canvas_cuts_back_to_the_artwork_and_never_grows() {
        let mut e = three_boxes();
        assert!(e.trim_canvas());
        assert_eq!((e.doc.page().width, e.doc.page().height), (70, 20));
        assert_eq!(e.doc.page().objects().next().unwrap().bounds, Rect { x: 0.0, y: 0.0, w: 20.0, h: 20.0 });
        assert!(!e.trim_canvas(), "there is nothing left to trim");
        // Artwork hanging over the edge stays as it is: the canvas is not made bigger for it.
        let over = e.doc.page().objects().next().unwrap().id;
        e.selection = vec![over];
        e.for_selected(|o| o.translate(-5.0, 0.0));
        assert!(!e.trim_canvas());
        assert_eq!(e.doc.page().width, 70);
        assert!(!Editor::new(Document::new(10, 10)).trim_canvas());
    }

    #[test]
    fn flatten_selection_makes_one_bitmap_where_the_topmost_was() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.selection = vec![all[0], all[1]];
        assert!(e.flatten_selection());
        let page = e.doc.page();
        let objects: Vec<&Object> = page.objects().collect();
        // The red and green boxes are one bitmap now, under the blue one as before.
        assert_eq!((objects.len(), objects[1].id), (2, all[2]));
        let flat = objects[0];
        assert!(flat.is_image() && e.selection == vec![flat.id]);
        assert_eq!(flat.bounds, Rect { x: 10.0, y: 10.0, w: 30.0, h: 20.0 });
        let bitmap = flat.bitmap().unwrap();
        // Red where only red was, green where green lay over it.
        assert_eq!((bitmap.pixel(2, 5), bitmap.pixel(15, 5)), (Some([255, 0, 0, 255]), Some([0, 255, 0, 255])));
        // With nothing selected there is nothing to flatten.
        e.selection.clear();
        assert!(!e.flatten_selection());
    }

    #[test]
    fn merge_down_needs_a_bitmap_right_under_the_selection() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.selection = vec![all[1]];
        assert!(!e.can_merge_down() && !e.merge_down(), "what lies under it is not a bitmap");
        // Once the red box is a bitmap, the green one merges into it.
        e.selection = vec![all[0]];
        assert!(e.flatten_selection());
        e.selection = vec![all[1]];
        assert!(e.can_merge_down() && e.merge_down());
        let merged: Vec<&Object> = e.doc.page().objects().collect();
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].bitmap().unwrap().pixel(15, 5), Some([0, 255, 0, 255]));
        // The lowest object has nothing under it.
        e.selection = vec![merged[0].id];
        assert!(!e.can_merge_down());
    }

    #[test]
    fn flatten_layers_leaves_one_layer_with_one_bitmap_and_the_slices() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.add_layer();
        let hidden = e.alloc_id();
        e.add_object(rect(hidden, 0.0, 50.0, 10.0, 10.0));
        e.doc.page_mut().layers[1].visible = false;
        e.doc.page_mut().active_layer = 0;
        e.selection = vec![all[2]];
        assert!(e.insert_web(WebKind::Slice, true));
        e.doc.page_mut().add_state();
        e.doc.page_mut().goto_state(0);
        assert!(e.flatten_layers());
        let page = e.doc.page();
        assert_eq!((page.layers.len(), page.layers[0].objects.len()), (1, 2));
        // The hidden layer went with what was on it; the bitmap spans the boxes that showed.
        assert!(page.layers[0].objects[0].is_image() && page.layers[0].objects[1].is_web());
        assert_eq!(page.layers[0].objects[0].bounds, Rect { x: 10.0, y: 10.0, w: 70.0, h: 20.0 });
        assert!(page.states.iter().all(|state| state.objects.len() <= 1), "the other states have one layer too");
        assert!(!Editor::new(Document::new(10, 10)).flatten_layers());
        // A second state with two boxes of its own is flattened to one bitmap as well.
        let mut e = three_boxes();
        e.doc.page_mut().duplicate_state();
        e.doc.page_mut().goto_state(0);
        e.add_layer();
        assert!(e.flatten_layers());
        let other = e.doc.page().at_state(1);
        assert_eq!(other.objects().count(), 1);
        assert!(other.objects().next().unwrap().is_image());
        assert_eq!(other.objects().next().unwrap().bounds, Rect { x: 10.0, y: 10.0, w: 70.0, h: 20.0 });
    }

    #[test]
    fn hotspots_and_slices_are_inserted_over_the_selection() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.selection = vec![all[0], all[2]];
        // One for all of them spans them all.
        assert!(e.insert_web(WebKind::Hotspot, true));
        let one = e.selected();
        assert_eq!((one.len(), one[0].bounds, one[0].is_web()), (1, Rect { x: 10.0, y: 10.0, w: 70.0, h: 20.0 }, true));
        // One each sits on each.
        e.selection = vec![all[0], all[2]];
        assert!(e.insert_web(WebKind::Slice, false));
        let each: Vec<Rect> = e.selected().iter().map(|o| o.bounds).collect();
        assert_eq!(each, [Rect { x: 10.0, y: 10.0, w: 20.0, h: 20.0 }, Rect { x: 60.0, y: 10.0, w: 20.0, h: 20.0 }]);
        // A slice does not get a slice of its own.
        assert!(!e.insert_web(WebKind::Slice, false));
        // An empty bitmap is as big as the canvas and clear.
        assert!(e.insert_empty_bitmap());
        let bitmap = e.selected()[0];
        assert_eq!((bitmap.bounds, bitmap.bitmap().unwrap().pixel(50, 40)), (Rect { x: 0.0, y: 0.0, w: 100.0, h: 80.0 }, Some([0, 0, 0, 0])));
    }

    #[test]
    fn paste_attributes_gives_the_look_of_what_was_copied() {
        let mut e = three_boxes();
        let all = ids(&e);
        e.selection = vec![all[0]];
        e.for_selected(|o| {
            o.stroke = Some(Stroke { color: Color::BLACK, width: 3.0 });
            o.opacity = 0.5;
            o.filters = vec![crate::effects::LiveFilter::new(crate::effects::FilterKind::Blur)];
        });
        e.copy();
        e.selection = vec![all[2]];
        assert!(e.paste_attributes());
        let (from, to) = (e.object(all[0]).unwrap().clone(), e.object(all[2]).unwrap().clone());
        assert_eq!((to.fill, to.stroke, to.opacity, to.filters.len()), (from.fill, from.stroke, 0.5, 1));
        assert_eq!(to.bounds, Rect { x: 60.0, y: 10.0, w: 20.0, h: 20.0 }, "it stays where and what it was");
        // From text to text the size and the style of the letters go along; the words stay.
        let bold = TextStyle { bold: true, align: TextAlign::Center, ..TextStyle::default() };
        e.object_mut(all[0]).unwrap().shape = Shape::Text { content: "a".into(), size: 30.0, style: bold.clone() };
        e.object_mut(all[2]).unwrap().shape = Shape::Text { content: "bc".into(), size: 12.0, style: TextStyle::default() };
        e.selection = vec![all[0]];
        e.copy();
        e.selection = vec![all[2]];
        assert!(e.paste_attributes());
        assert_eq!(e.object(all[2]).unwrap().shape, Shape::Text { content: "bc".into(), size: 30.0, style: bold });
        // Nothing copied, or nothing selected, and nothing happens.
        e.selection.clear();
        assert!(!e.paste_attributes());
        assert!(!three_boxes().paste_attributes());
    }

    #[test]
    fn fit_canvas_wraps_artwork() {
        let mut e = two_rect_editor();
        assert!(e.fit_canvas());
        assert_eq!((e.doc.page().width, e.doc.page().height), (70, 40));
        assert_eq!(e.doc.page().objects().next().unwrap().bounds.x, 0.0);
        let mut empty = Editor::new(Document::new(10, 10));
        assert!(!empty.fit_canvas());
    }

    #[test]
    fn opened_image_sits_on_a_background_layer() {
        let doc = Document::from_bitmap("a", Bitmap::from_rgba(1, 1, vec![0, 0, 0, 255]).unwrap());
        let names: Vec<&str> = doc.page().layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, vec!["Background", "Layer 1"]);
        assert_eq!(doc.page().active_layer, 0);
    }

    fn rect(id: u64, x: f32, y: f32, w: f32, h: f32) -> Object {
        Object {
            id,
            name: format!("r{id}"),
            shape: Shape::Rect { radius: 0.0 },
            bounds: Rect { x, y, w, h },
            points: vec![],
            handles: Vec::new(),
            placement: None,
            pivot: None,

            behaviors: Vec::new(),
            attached: String::new(),
            blend: crate::blend::BlendMode::Normal,
            mask: None,
            mask_of: None,
            fill_style: None,
            stroke_profile: Vec::new(),
            fill_look: Default::default(),
            link: String::new(),
            filters: Vec::new(),
            fill: Some(Color::BLACK),
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    fn editor() -> Editor {
        let mut e = Editor::new(Document::new(500, 400));
        for (x, id) in [(0.0, 100), (50.0, 101), (200.0, 102)] {
            e.add_object(rect(id, x, 10.0, 40.0, 40.0));
        }
        e.selection.clear();
        e.commit("seed");
        e
    }

    #[test]
    fn a_symbol_is_edited_where_it_lies_and_its_copies_follow() {
        let mut e = editor();
        e.doc.next_id = 1000;
        e.selection = vec![100];
        let symbol = e.make_symbol().unwrap();
        let first = e.selection[0];
        // A second copy, twice the size.
        assert!(e.place_symbol_at(symbol, v2(300.0, 300.0)));
        let second = e.selection[0];
        e.object_mut(second).unwrap().bounds = Rect { x: 260.0, y: 260.0, w: 80.0, h: 80.0 };
        e.selection = vec![first];
        assert!(e.can_edit_symbol() && e.edit_selected_symbol());
        assert_eq!(e.edited_symbol().unwrap().id, symbol);
        // The copy is gone, its one object lies loose where it was, and the rest of the page is locked.
        assert!(e.object(first).is_none() && e.object(101).unwrap().locked);
        let member = e.doc.page().objects().find(|o| !o.locked).unwrap().clone();
        assert_eq!(member.bounds, Rect { x: 0.0, y: 10.0, w: 40.0, h: 40.0 });
        // Something drawn during the edit joins the symbol: it grows 20 to the right.
        e.add_object(rect(0, 40.0, 10.0, 20.0, 40.0));
        assert!(e.leave_symbol() && e.editing_symbol().is_none());
        assert_eq!(e.doc.symbols[0].objects.len(), 2);
        assert_eq!(e.object(first).unwrap().bounds, Rect { x: 0.0, y: 10.0, w: 60.0, h: 40.0 });
        assert_eq!(e.object(second).unwrap().bounds, Rect { x: 260.0, y: 260.0, w: 120.0, h: 80.0 });
        assert!(!e.object(101).unwrap().locked && e.selection == vec![first]);
        assert_eq!(e.object(first).unwrap().instance_parts().len(), 2);
        assert!(!e.leave_symbol());
    }

    #[test]
    fn a_new_button_gets_its_looks_and_its_first_copy() {
        let mut e = editor();
        let symbol = e.new_symbol("Button", SymbolKind::Button);
        assert!(e.edit_symbol(symbol, None, true));
        e.add_object(rect(0, 100.0, 100.0, 80.0, 30.0));
        // Over starts empty, and takes the Up look when asked.
        assert!(e.set_symbol_look(1) && !e.set_symbol_look(1));
        assert_eq!(e.doc.page().objects().filter(|o| !o.locked).count(), 0);
        assert!(e.copy_previous_look());
        let over = e.selection[0];
        e.object_mut(over).unwrap().translate(5.0, 0.0);
        assert!(e.set_symbol_look(0));
        assert_eq!(e.doc.page().objects().filter(|o| !o.locked).count(), 1);
        assert!(e.leave_symbol());
        let entry = &e.doc.symbols[0];
        assert_eq!((entry.objects.len(), entry.looks.len()), (1, 1));
        assert_eq!(entry.objects[0].bounds, Rect { x: 0.0, y: 0.0, w: 80.0, h: 30.0 });
        assert_eq!(entry.looks[0][0].bounds.x, 5.0, "a look keeps its place against the Up look");
        // The new symbol's first copy is on the page, selected.
        let copy = e.selected()[0].clone();
        assert!(matches!(copy.shape, Shape::Instance { symbol: s, .. } if s == symbol));
        assert_eq!(copy.bounds, Rect { x: 100.0, y: 100.0, w: 80.0, h: 30.0 });
        // Opening it again brings the looks back where they were.
        assert!(e.edit_selected_symbol() && e.set_symbol_look(1));
        assert_eq!(e.doc.page().objects().find(|o| !o.locked).unwrap().bounds.x, 105.0);
        // An emptied symbol leaves no copy behind.
        e.set_symbol_look(0);
        let members: Vec<u64> = e.doc.page().objects().filter(|o| !o.locked).map(|o| o.id).collect();
        e.selection = members;
        e.delete_selected();
        e.set_symbol_look(1);
        e.selection = e.doc.page().objects().filter(|o| !o.locked).map(|o| o.id).collect();
        e.delete_selected();
        assert!(e.leave_symbol());
        assert!(e.doc.symbols[0].objects.is_empty() && e.doc.page().objects().count() == 3);
    }

    #[test]
    fn an_open_group_is_still_drawn_with_its_own_filters() {
        let mut e = editor();
        e.selection = vec![100, 101];
        assert!(e.group_selected());
        let group = e.selection[0];
        e.object_mut(group).unwrap().filters = vec![crate::effects::LiveFilter::new(crate::effects::FilterKind::Blur)];
        assert!(e.subselect());
        // Open, the members lie loose and none of them carries the filter; the page still asks to be drawn with it.
        assert!(e.doc.page().objects().all(|o| o.filters.is_empty()));
        assert!(e.doc.page().open_groups_need_drawing() && e.needs_compositing());
        let drawn = e.doc.page().with_groups_closed();
        assert_eq!(drawn.objects().count(), 2);
        let whole = drawn.objects().find(|o| matches!(o.shape, Shape::Group { .. })).unwrap();
        assert_eq!((whole.filters.len(), whole.instance_parts().len()), (1, 2));
        // The page itself is left as it was.
        assert_eq!((e.doc.page().objects().count(), e.doc.page().open_groups.len()), (3, 1));
    }

    #[test]
    fn a_timeline_keeps_keyframes_and_tweens_between_them() {
        let mut e = editor();
        e.doc.page_mut().start_timeline(12.0);
        let frames = |e: &Editor| e.doc.page().timeline.as_ref().unwrap().frame_count();
        let xs = |e: &Editor| e.doc.page().objects().map(|o| o.bounds.x).collect::<Vec<_>>();
        assert_eq!((frames(&e), xs(&e)), (1, vec![0.0, 50.0, 200.0]));
        // A keyframe on frame 10 starts with what frame 0 shows; the layer now lasts 11 frames.
        e.goto_frame(10);
        assert!(e.doc.page().objects().count() == 0, "past the end of the layer nothing shows");
        assert_eq!(e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false)), Some(true));
        assert_eq!((frames(&e), xs(&e)), (11, vec![0.0, 50.0, 200.0]));
        // Moved there, the first box is 100 further right on frame 10 and still at 0 on frames 0 to 9.
        e.selection = vec![100];
        e.for_selected(|o| {
            o.translate(100.0, 0.0);
            o.opacity = 0.0;
        });
        e.commit("move");
        e.goto_frame(5);
        assert_eq!(xs(&e)[0], 0.0);
        // With a motion tween on the first keyframe, frame 5 is halfway.
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween::default())));
        e.goto_frame(5);
        let halfway = e.object(100).unwrap().clone();
        assert_eq!((halfway.bounds.x, halfway.opacity), (50.0, 0.5));
        assert_eq!(e.object(101).unwrap().bounds.x, 50.0, "what does not change between the keyframes stays put");
        // Easing out is ahead of an even pace at the middle; easing in is behind it.
        assert!(Tween { ease: 100.0, ..Tween::default() }.eased(0.5) > 0.7 && Tween { ease: -100.0, ..Tween::default() }.eased(0.5) < 0.3);
        // Changing an object inside the tween makes a keyframe of that frame, and the tween goes through it.
        e.selection = vec![100];
        e.for_selected(|o| o.translate(0.0, 40.0));
        e.goto_frame(0);
        let timeline = e.doc.page().timeline.as_ref().unwrap();
        assert!(timeline.is_keyframe(0, 5) && timeline.layers[0].len() == 3);
        assert_eq!(timeline.layers[0].iter().map(|span| (span.start, span.length, span.tween.is_some())).collect::<Vec<_>>(), [(0, 5, true), (5, 5, true), (10, 1, false)]);
        assert_eq!(e.doc.page().at_frame(5).objects().next().unwrap().bounds.y, 50.0);
        assert_eq!(e.doc.page().at_frame(7).objects().next().unwrap().bounds.x, 70.0);
        // Undo takes the page and the timeline back together.
        e.commit("keyframe by moving");
        // Clearing the keyframe gives its frames back to the one before; removing frames shortens the layer.
        e.goto_frame(5);
        assert_eq!(e.on_timeline(|timeline, layer, frame| timeline.clear_keyframe(layer, frame)), Some(true));
        assert_eq!(e.doc.page().timeline.as_ref().unwrap().layers[0].len(), 2);
        e.on_timeline(|timeline, layer, frame| timeline.remove_frame(layer, frame));
        e.on_timeline(|timeline, layer, frame| timeline.insert_frames(layer, frame, 3));
        assert_eq!(frames(&e), 13);
        assert_eq!(e.on_timeline(|timeline, layer, frame| timeline.clear_keyframe(layer, frame)), Some(false), "frame 5 is no keyframe now");
        // A new layer lasts as long as the scene and is empty; deleting it takes its frames along.
        e.add_layer();
        let timeline = e.doc.page().timeline.as_ref().unwrap();
        assert_eq!((timeline.layers.len(), timeline.layers[1][0].length), (2, 13));
        assert!(e.delete_layer());
        assert_eq!(e.doc.page().timeline.as_ref().unwrap().layers.len(), 1);
        // The whole of it survives a file.
        e.goto_frame(0);
        let again = document_from_json(&document_to_json(&e.doc).unwrap()).unwrap();
        assert_eq!(again.pages[0].timeline, e.doc.pages[0].timeline);
        assert_eq!(again.pages[0].at_frame(12).objects().next().unwrap().bounds.x, 100.0);
    }

    #[test]
    fn a_shape_tween_changes_one_shape_into_another() {
        let mut e = Editor::new(Document::new(300, 200));
        let mut square = rect(1, 0.0, 0.0, 40.0, 40.0);
        square.fill = Some(Color::rgb(255, 0, 0));
        e.add_object(square);
        e.doc.page_mut().start_timeline(12.0);
        // On frame 10 a blank keyframe with a wider, blue box further right: another object altogether.
        e.goto_frame(10);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, true));
        let mut wide = rect(2, 200.0, 100.0, 80.0, 40.0);
        wide.fill = Some(Color::rgb(0, 0, 255));
        e.add_object(wide);
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween { shape: true, ..Tween::default() })));
        let halfway = e.doc.page().at_frame(5);
        let shape = halfway.objects().next().unwrap();
        // Halfway it is a path halfway between the two boxes in place, size and color.
        assert!(matches!(shape.shape, Shape::Path { closed: true }));
        assert_eq!(shape.bounds, Rect { x: 100.0, y: 50.0, w: 60.0, h: 40.0 });
        assert_eq!(shape.fill, Some(Color::rgb(128, 0, 128)));
        assert_eq!(shape.points.len(), 64);
        // The keyframes themselves stay what they were.
        assert!(matches!(e.doc.page().at_frame(0).objects().next().unwrap().shape, Shape::Rect { .. }));
        assert_eq!(e.doc.page().at_frame(10).objects().next().unwrap().id, 2);
        // A shape with no partner in the next keyframe fades away.
        e.goto_frame(0);
        e.add_object(rect(3, 250.0, 150.0, 10.0, 10.0));
        let faded = e.doc.page().at_frame(5);
        assert_eq!((faded.objects().count(), faded.objects().nth(1).unwrap().opacity), (2, 0.5));
        // Evenly spaced points go all the way round a closed outline, and from end to end of an open one.
        let round = resampled(&[v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)], true, 8);
        assert_eq!((round[0], round[2], round[7]), (v2(0.0, 0.0), v2(10.0, 0.0), v2(0.0, 5.0)));
        let line = resampled(&[v2(0.0, 0.0), v2(10.0, 0.0)], false, 3);
        assert_eq!(line, [v2(0.0, 0.0), v2(5.0, 0.0), v2(10.0, 0.0)]);
    }

    #[test]
    fn guide_and_mask_layers_do_their_work() {
        let mut e = Editor::new(Document::new(300, 200));
        e.add_object(rect(1, 0.0, 90.0, 20.0, 20.0));
        e.doc.page_mut().start_timeline(12.0);
        e.goto_frame(10);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.selection = vec![1];
        e.for_selected(|o| o.translate(200.0, 0.0));
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween::default())));
        // Straight across without a guide: halfway the box is at the same height.
        assert_eq!(e.doc.page().at_frame(5).objects().next().unwrap().bounds.center(), v2(110.0, 100.0));
        // A guide layer above, with a path that goes up to the top in the middle and down again.
        e.add_layer();
        let mut path = rect(2, 10.0, 20.0, 200.0, 80.0);
        (path.shape, path.points) = (Shape::Path { closed: false }, vec![v2(10.0, 100.0), v2(110.0, 20.0), v2(210.0, 100.0)]);
        e.add_object(path);
        e.doc.page_mut().edit_timeline(|timeline| {
            timeline.insert_frames(1, 0, 10);
            timeline.set_kind(1, LayerKind::Guide);
        });
        let middle = e.doc.page().at_frame(5);
        let center = middle.layers[0].objects[0].bounds.center();
        assert!((center.x - 110.0).abs() < 0.5 && (center.y - 20.0).abs() < 0.5, "halfway the box is at the top of the path, not {center:?}");
        // Played, the guide's path is not drawn.
        assert_eq!(middle.played().objects().count(), 1);
        assert!(middle.played().timeline.is_none());
        // As a mask, the upper layer lets the lower one show only where it has something.
        e.doc.page_mut().edit_timeline(|timeline| timeline.set_kind(1, LayerKind::Mask));
        e.goto_frame(0);
        e.doc.page_mut().layers[1].objects = vec![rect(3, 0.0, 90.0, 10.0, 20.0)];
        e.object_mut(1).unwrap().fill = Some(Color::rgb(255, 0, 0));
        e.object_mut(3).unwrap().fill = Some(Color::BLACK);
        let played = e.doc.page().played();
        assert_eq!(played.objects().count(), 1);
        assert!(played.objects().next().unwrap().mask.is_some());
        let drawn = crate::export::render_page(&played).unwrap();
        // The box reaches from 0 to 20; the mask covers 0 to 10. Inside the mask the red shows, outside it the white canvas.
        assert_eq!(drawn.pixel(5, 100).unwrap().green(), 0);
        assert_eq!(drawn.pixel(15, 100).unwrap().green(), 255);
        // A second layer tied to the mask is masked as well; let go, it shows whole.
        e.doc.page_mut().active_layer = 0;
        e.add_layer();
        e.goto_frame(0);
        let mut second = rect(4, 0.0, 150.0, 20.0, 20.0);
        second.fill = Some(Color::rgb(0, 0, 255));
        e.doc.page_mut().layers[1].objects = vec![second];
        let tied = |e: &mut Editor, layer: usize, on: bool| e.doc.page_mut().edit_timeline(|timeline| timeline.set_bound(layer, on)).unwrap();
        assert!(tied(&mut e, 1, true));
        let timeline = e.doc.page().timeline.clone().unwrap();
        assert_eq!((timeline.master(0), timeline.master(1)), (Some((2, LayerKind::Mask)), Some((2, LayerKind::Mask))));
        assert_eq!(e.doc.page().played().objects().filter(|o| o.mask.is_some()).count(), 2);
        // While editing the mask shows only once it and what it masks are locked, as in Flash.
        assert!(!e.doc.page().shows_locked_masks() && !e.needs_compositing());
        for layer in &mut e.doc.page_mut().layers {
            layer.locked = true;
        }
        assert!(e.doc.page().shows_locked_masks() && e.needs_compositing());
        assert_eq!(e.doc.page().with_locked_masks().objects().filter(|o| o.mask.is_some()).count(), 2);
        // Letting the upper tied layer go lets the one tied through it go too.
        assert!(tied(&mut e, 1, false));
        let timeline = e.doc.page().timeline.clone().unwrap();
        assert_eq!((timeline.master(0), timeline.master(1)), (None, None));
        // A layer with nothing to be tied to cannot be tied.
        assert!(!tied(&mut e, 0, true));
        e.doc.page_mut().edit_timeline(|timeline| timeline.set_bound(1, true));
        e.doc.page_mut().edit_timeline(|timeline| timeline.set_bound(0, true));
        // Made an ordinary layer again, a mask lets go of all it had.
        e.doc.page_mut().edit_timeline(|timeline| timeline.set_kind(2, LayerKind::Normal));
        assert_eq!(e.doc.page().timeline.as_ref().unwrap().bound.iter().filter(|b| **b).count(), 0);
        e.doc.page_mut().edit_timeline(|timeline| timeline.set_kind(2, LayerKind::Mask));
        e.doc.page_mut().edit_timeline(|timeline| timeline.set_kind(1, LayerKind::Normal));
        // New layers and deleted ones keep each layer's kind with it.
        e.doc.page_mut().active_layer = 0;
        e.add_layer();
        let timeline = e.doc.page().timeline.as_ref().unwrap();
        assert_eq!((timeline.kind(0), timeline.kind(1), timeline.kind(2), timeline.kind(3)), (LayerKind::Normal, LayerKind::Normal, LayerKind::Normal, LayerKind::Mask));
        assert!(timeline.is_bound(2) && !timeline.is_bound(1), "the layer under the mask is still the tied one");
    }

    #[test]
    fn another_document_is_brought_in_with_its_symbols() {
        // The other document: a box and a copy of a symbol, on a page with a timeline.
        let mut other = editor();
        other.doc.next_id = 500;
        other.selection = vec![100];
        let symbol = other.make_symbol().unwrap();
        other.doc.page_mut().start_timeline(12.0);
        // This document already has objects numbered 100 to 102 and a symbol of its own.
        let mut e = editor();
        e.doc.next_id = 500;
        e.selection = vec![102];
        let own = e.make_symbol().unwrap();
        assert_eq!(e.import_document(&other.doc), 3);
        assert_eq!(e.doc.page().objects().count(), 6);
        assert_eq!(e.selection.len(), 3);
        // Nothing that came in shares a number with what was there, and the copy points at the symbol that came with it.
        let mut ids: Vec<u64> = e.doc.page().objects().map(|o| o.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 6);
        assert_eq!(e.doc.symbols.len(), 2);
        let brought = e.doc.symbols[1].id;
        assert!(brought != own && brought != symbol);
        let copy = e.selected().into_iter().find(|o| matches!(o.shape, Shape::Instance { .. })).unwrap().clone();
        assert!(matches!(copy.shape, Shape::Instance { symbol: of, .. } if of == brought));
        assert_eq!(copy.instance_parts().len(), 1);
        // Handed to PisiWorks, an animation's scenes are plain pages showing the frame they were on.
        other.goto_frame(3);
        other.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, true));
        let plain = other.for_other_program(true);
        assert!(plain.pages[0].timeline.is_none() && plain.pages[0].objects().count() == 0);
        assert!(other.for_other_program(false).pages[0].timeline.is_some());
    }

    #[test]
    fn actions_are_kept_as_text_and_read_back() {
        for action in [Action::Stop, Action::Play, Action::GotoPlay(12), Action::GotoStop(3), Action::Scene(2), Action::Url("https://pardus.org.tr".into())] {
            assert_eq!(Action::parse(&action.text()), Some(action.clone()));
            assert_eq!(Action::chosen(Action::choice(&Some(action.clone())), &Some(action.clone())), Some(action));
        }
        assert_eq!(Action::parse(""), None);
        assert_eq!(Action::parse("action:goto-play:0"), None, "frames are counted from 1");
        assert_eq!(Action::parse("action:nonsense"), None);
        // Choosing another action that takes a number keeps the number.
        assert_eq!(Action::chosen(4, &Some(Action::GotoPlay(7))), Some(Action::GotoStop(7)));
        assert_eq!(Action::chosen(0, &Some(Action::Stop)), None);
    }

    #[test]
    fn a_movie_clip_has_a_timeline_of_its_own() {
        let mut e = editor();
        e.doc.page_mut().start_timeline(12.0);
        e.selection = vec![100];
        let symbol = e.make_symbol().unwrap();
        let copy = e.selection[0];
        e.doc.symbols[0].kind = SymbolKind::Animation;
        assert_eq!(e.doc.scenes(), [0]);
        // Opened, the clip has a page of its own with what the symbol holds in the middle, and a timeline.
        assert!(e.open_clip(symbol));
        assert_eq!((e.doc.active_page, e.doc.pages.len(), e.editing_clip()), (1, 2, Some(symbol)));
        assert_eq!(e.doc.scenes(), [0], "the clip's page is no scene");
        let page = e.doc.page();
        assert!(page.timeline.is_some());
        assert_eq!(page.objects().next().unwrap().bounds, Rect { x: 230.0, y: 180.0, w: 40.0, h: 40.0 });
        // A second keyframe further right; in the first frame the box is made wider.
        e.goto_frame(5);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        let part = e.doc.page().objects().next().unwrap().id;
        e.selection = vec![part];
        e.for_selected(|o| o.translate(100.0, 0.0));
        e.goto_frame(0);
        e.selection = vec![part];
        e.for_selected(|o| o.set_bounds(Rect { x: 230.0, y: 180.0, w: 80.0, h: 40.0 }));
        // Leaving goes back to the scene; the copy shows the first frame and is as much wider.
        assert!(e.leave_clip() && !e.leave_clip());
        assert_eq!(e.doc.active_page, 0);
        assert_eq!(e.object(copy).unwrap().bounds, Rect { x: 0.0, y: 10.0, w: 80.0, h: 40.0 });
        assert_eq!(e.doc.symbols[0].objects[0].bounds, Rect { x: 0.0, y: 0.0, w: 80.0, h: 40.0 });
        // Opened again, it is the same page, with its frames.
        assert!(e.open_clip(symbol));
        assert_eq!((e.doc.pages.len(), e.doc.page().timeline.as_ref().unwrap().frame_count()), (2, 6));
        e.leave_clip();
        // A new scene has a timeline and is named as one; the clip's page still is not a scene.
        e.add_page();
        assert_eq!(e.doc.scenes(), [0, 2]);
        assert!(e.doc.page().timeline.is_some() && e.doc.page().name == "Scene 2");
        // The whole survives a file, and deleting the symbol takes the clip's page along.
        let again = document_from_json(&document_to_json(&e.doc).unwrap()).unwrap();
        assert_eq!((again.pages.len(), again.clip_page(symbol)), (3, Some(1)));
        e.delete_symbol(symbol);
        assert_eq!((e.doc.pages.len(), e.doc.scenes(), e.doc.active_page), (2, vec![0, 1], 1));
        // A sound is kept as it was imported.
        let sound = e.add_sound("zil.wav", "audio/wav", b"RIFF");
        assert_eq!((e.doc.sounds[0].id, e.doc.sounds[0].data.as_str()), (sound, "UklGRg=="));
    }

    #[test]
    fn orienting_to_a_path_and_shape_hints() {
        // A box travels down a path that turns from going right to going down.
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 0.0, 0.0, 20.0, 10.0));
        e.doc.page_mut().start_timeline(12.0);
        e.goto_frame(10);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.selection = vec![1];
        e.for_selected(|o| o.translate(190.0, 190.0));
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween { orient: true, ..Tween::default() })));
        e.add_layer();
        let mut path = rect(2, 10.0, 5.0, 190.0, 190.0);
        (path.shape, path.points) = (Shape::Path { closed: false }, vec![v2(10.0, 5.0), v2(200.0, 5.0), v2(200.0, 195.0)]);
        e.add_object(path);
        e.doc.page_mut().edit_timeline(|timeline| {
            timeline.insert_frames(1, 0, 10);
            timeline.set_kind(1, LayerKind::Guide);
        });
        // On the level stretch it lies as it was drawn; on the stretch going down it has turned a quarter turn.
        let early = e.doc.page().at_frame(2);
        assert!(early.layers[0].objects[0].placement.is_none_or(|p| p.warp.turn().abs() < 0.001));
        let late = e.doc.page().at_frame(8);
        let turned = &late.layers[0].objects[0];
        assert!((turned.placement.unwrap().warp.turn() - std::f32::consts::FRAC_PI_2).abs() < 0.001);
        assert!((turned.bounds.center().x - 200.0).abs() < 0.5);
        // Shape hints: a square becomes the same square. Left alone, no point moves.
        let square = rect(5, 0.0, 0.0, 40.0, 40.0);
        let still = morphed(&square, &square, 0.5, &[]);
        assert!(still.points.iter().zip(resampled(&square.outline(), true, 64)).all(|(p, q)| (p.x - q.x).abs() < 0.01 && (p.y - q.y).abs() < 0.01));
        // A hint that sends the top left corner to the bottom right one: the outline starts from that corner,
        // which is halfway to its place halfway through, and the shape is no longer the square standing still.
        let hinted = morphed(&square, &square, 0.5, &[(v2(0.0, 0.0), v2(40.0, 40.0))]);
        assert_eq!(hinted.points[0], v2(20.0, 20.0));
        assert!(hinted.points != still.points);
        // A second hint settles which way round: top right goes to bottom left, so the square turns half a turn
        // and every point passes through the middle.
        let turned = morphed(&square, &square, 0.5, &[(v2(0.0, 0.0), v2(40.0, 40.0)), (v2(40.0, 0.0), v2(0.0, 40.0))]);
        assert!(turned.points.iter().all(|p| (p.x - 20.0).abs() < 0.01 && (p.y - 20.0).abs() < 0.01), "{:?}", &turned.points[..3]);
    }

    #[test]
    fn a_tween_turns_an_object_around_its_transformation_point() {
        // An arm hanging from a shoulder at 100, 100, swung up to the right on the next keyframe.
        let arm = |to: Vec2, pivot: bool| {
            let mut object = rect(1, 100.0, 100.0, 0.0, 40.0);
            (object.shape, object.points) = (Shape::Line, vec![v2(100.0, 100.0), to]);
            object.bounds = Rect::of_points(&object.points);
            if pivot {
                object.set_pivot_point(Some(v2(100.0, 100.0)));
            }
            object
        };
        let (down, across) = (arm(v2(100.0, 140.0), true), arm(v2(140.0, 100.0), true));
        assert_eq!(down.pivot_point().map(|p| (p.x.round(), p.y.round())), Some((100.0, 100.0)));
        let half = between(&down, &across, 0.5, 0);
        let (shoulder, hand) = (half.points[0], half.points[1]);
        assert!((shoulder.x - 100.0).abs() < 0.01 && (shoulder.y - 100.0).abs() < 0.01, "the shoulder stays where it is: {shoulder:?}");
        assert!((hand.x - 128.28).abs() < 0.1 && (hand.y - 128.28).abs() < 0.1, "the hand is halfway round: {hand:?}");
        // Without a point of its own it turns around its middle, as before, and the shoulder wanders.
        let loose = between(&arm(v2(100.0, 140.0), false), &arm(v2(140.0, 100.0), false), 0.5, 0).points[0];
        assert!((loose.x - 100.0).abs() > 3.0);
        // The point follows the object: moved and turned, it is still on the same end.
        let mut moved = down.clone();
        moved.warp(&Warp::rotation(v2(0.0, 0.0), 1.0).then(&Warp::translation(30.0, -20.0)));
        let (point, end) = (moved.pivot_point().unwrap(), moved.points[0]);
        assert!((point.x - end.x).abs() < 0.01 && (point.y - end.y).abs() < 0.01);
        // A symbol's copy, a rectangle or a text keeps it through a turn too, by the transform it records.
        let mut plate = rect(2, 0.0, 0.0, 40.0, 10.0);
        plate.set_pivot_point(Some(v2(0.0, 5.0)));
        let mut turned = plate.clone();
        turned.warp(&Warp::rotation(v2(0.0, 5.0), std::f32::consts::FRAC_PI_2));
        let half = between(&plate, &turned, 0.5, 0).pivot_point().unwrap();
        assert!(half.x.abs() < 0.01 && (half.y - 5.0).abs() < 0.01, "{half:?}");
    }

    #[test]
    fn a_gear_drawn_as_an_auto_shape_can_be_made_to_turn() {
        let gear = |id: u64| {
            let mut object = rect(id, 100.0, 100.0, 100.0, 100.0);
            object.shape = Shape::Auto { kind: crate::autoshape::AutoKind::Gear, a: 0.7, b: 0.0, count: 12 };
            object
        };
        let far = |a: &[Vec2], b: &[Vec2]| a.len() != b.len() || a.iter().zip(b).any(|(p, q)| (p.x - q.x).abs() + (p.y - q.y).abs() > 1.0);
        // The way a user tries first: the same gear on two keyframes, and one whole turn asked of the tween.
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(gear(1));
        e.doc.page_mut().start_timeline(24.0);
        e.goto_frame(12);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween { turns: 1, ..Tween::default() })));
        let start = e.doc.page().at_frame(0).objects().next().unwrap().outline();
        let middle = e.doc.page().at_frame(1).objects().next().unwrap().outline();
        assert!(far(&start, &middle), "one whole turn asked of the tween turns the gear");
        // The other way: the gear turned by hand on the second keyframe, by a tenth of a turn.
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(gear(1));
        e.doc.page_mut().start_timeline(24.0);
        e.goto_frame(10);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.selection = vec![1];
        e.for_selected(|o| {
            o.warp(&Warp::rotation(v2(150.0, 150.0), 0.6));
        });
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween::default())));
        let end = e.doc.page().at_frame(10).objects().next().unwrap().outline();
        let half = e.doc.page().at_frame(5).objects().next().unwrap().outline();
        let expected: Vec<Vec2> = { let mut o = gear(1); o.warp(&Warp::rotation(v2(150.0, 150.0), 0.3)); o.outline() };
        assert!(far(&start, &end) && !far(&expected, &half), "turned by hand, halfway is half as far");
        // A third way: Rotate 90° from the Modify menu on the second keyframe.
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(gear(1));
        e.doc.page_mut().start_timeline(24.0);
        e.goto_frame(10);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.selection = vec![1];
        e.rotate_selected(true);

        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween::default())));
        let half = e.doc.page().at_frame(5).objects().next().unwrap().outline();
        let expected: Vec<Vec2> = { let mut o = gear(1); o.warp(&Warp::rotation(v2(150.0, 150.0), std::f32::consts::FRAC_PI_4)); o.outline() };
        // The command redraws the gear as a path and keeps no record of the turn; the tween reads it off the outline.
        assert!(far(&start, &half) && !far(&expected, &half), "halfway the gear is an eighth of a turn round");
        // A box that is not square, turned the same way, is found from whichever corner its outline now starts at.
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 100.0, 120.0, 100.0, 60.0));
        e.doc.page_mut().start_timeline(24.0);
        e.goto_frame(10);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.selection = vec![1];
        e.rotate_selected(true);
        assert_eq!(e.selected()[0].bounds, Rect { x: 120.0, y: 100.0, w: 60.0, h: 100.0 });
        e.goto_frame(0);
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween::default())));
        let half = e.doc.page().at_frame(5).objects().next().unwrap().clone();
        // Halfway it stands on a corner: as wide as it is tall, and wider than either side was.
        assert!((half.bounds.w - half.bounds.h).abs() < 0.5 && half.bounds.w > 110.0, "{:?}", half.bounds);
        // A plain move is still a plain move: the box stays a box.
        assert_eq!(turned_between(&[v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 5.0)], &[v2(3.0, 4.0), v2(13.0, 4.0), v2(13.0, 9.0)]).map(|found| found.0.abs() < 0.001), Some(true));
        // And a stretch is no turn at all.
        assert!(turned_between(&[v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 5.0), v2(0.0, 5.0)], &[v2(0.0, 0.0), v2(30.0, 0.0), v2(30.0, 5.0), v2(0.0, 5.0)]).is_none());
    }

    #[test]
    fn a_tween_turns_and_resizes_on_its_way() {
        let mut e = editor();
        e.doc.page_mut().start_timeline(24.0);
        e.goto_frame(4);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
        e.selection = vec![100];
        e.for_selected(|o| o.set_bounds(Rect { x: 0.0, y: 10.0, w: 80.0, h: 80.0 }));
        e.goto_frame(0);
        // One whole turn on the way, though the two keyframes stand the same way up.
        e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(Tween { turns: 1, ..Tween::default() })));
        let quarter = e.doc.page().at_frame(1);
        let object = quarter.objects().next().unwrap();
        assert!((object.placement.unwrap().warp.turn() - std::f32::consts::FRAC_PI_2).abs() < 0.001);
        assert_eq!(object.placement.unwrap().frame.w, 50.0);
        // The keyframes themselves are untouched by the tween.
        assert_eq!(e.doc.page().at_frame(4).objects().next().unwrap().bounds.w, 80.0);
        assert!(e.doc.page().at_frame(0).objects().next().unwrap().placement.is_none());
    }

    #[test]
    fn tween_instances_fills_in_between_two_copies_of_a_symbol() {
        let mut e = editor();
        e.selection = vec![100];
        let symbol = e.make_symbol().unwrap();
        let first = e.selection[0];
        assert!(!e.can_tween());
        assert!(e.place_symbol_at(symbol, v2(220.0, 130.0)));
        let second = e.selection[0];
        e.object_mut(second).unwrap().set_bounds(Rect { x: 180.0, y: 90.0, w: 80.0, h: 80.0 });
        e.object_mut(second).unwrap().opacity = 0.0;
        e.selection = vec![first, second];
        assert!(e.can_tween());
        // Three copies in between: the middle one is halfway in place, size and opacity.
        let mut on_page = e.clone_for_test();
        assert_eq!(on_page.tween_instances(3, false), 3);
        let run: Vec<Object> = on_page.selected_in_order();
        assert_eq!(run.len(), 5);
        assert_eq!((run[0].id, run[4].id), (first, second));
        assert_eq!(run[2].bounds, Rect { x: 90.0, y: 50.0, w: 60.0, h: 60.0 });
        assert!((run[2].opacity - 0.5).abs() < 0.001 && (run[1].opacity - 0.75).abs() < 0.001);
        assert_eq!(on_page.doc.page().states.len(), 1);
        // Distributed, each of the five has a state to itself.
        assert_eq!(e.tween_instances(3, true), 3);
        let page = e.doc.page();
        assert_eq!(page.states.len(), 5);
        let copies = |state: usize| page.at_state(state).objects().filter(|o| matches!(o.shape, Shape::Instance { .. })).count();
        assert!((0..5).all(|state| copies(state) == 1));
        assert_eq!(page.at_state(4).objects().find(|o| o.id == second).unwrap().bounds.w, 80.0);
        assert_eq!(e.selection, vec![first]);
        // A turned copy is reached by turning step by step.
        let mut turned = editor();
        turned.selection = vec![100];
        let symbol = turned.make_symbol().unwrap();
        let a = turned.selection[0];
        turned.place_symbol_at(symbol, v2(300.0, 30.0));
        let b = turned.selection[0];
        let center = turned.object(b).unwrap().bounds.center();
        turned.object_mut(b).unwrap().warp(&Warp::rotation(center, std::f32::consts::FRAC_PI_2));
        turned.selection = vec![a, b];
        assert_eq!(turned.tween_instances(1, false), 1);
        let middle = &turned.selected_in_order()[1];
        assert!((middle.placement.unwrap().warp.turn() - std::f32::consts::FRAC_PI_4).abs() < 0.001);
    }

    #[test]
    fn auto_vector_mask_and_twist_and_fade() {
        let mut e = editor();
        e.selection = vec![100];
        assert!(e.auto_vector_mask(0) && !e.auto_vector_mask(99));
        let mask = e.object(100).unwrap().mask.clone().unwrap();
        assert_eq!((mask.kind, mask.object.bounds), (MaskKind::Grayscale, Rect { x: 0.0, y: 10.0, w: 40.0, h: 40.0 }));
        let style = mask.object.fill_style.clone().unwrap();
        assert_eq!((style.kind, style.start, style.end), (FillKind::Linear, v2(0.5, 0.0), v2(0.5, 1.0)));
        assert_eq!((style.stops[0].1, style.stops[1].1), (Color::WHITE, Color::BLACK));
        // Three copies of the second box, each 10 further right, half the size of the one before, fading to nothing.
        e.selection = vec![101];
        let twist = TwistSettings { steps: 3, spacing: 10.0, direction: 0.0, rotation: 0.0, scale: 50.0, fade_to: 0.0 };
        assert_eq!(e.twist_and_fade(twist), 3);
        let objects: Vec<&Object> = e.doc.page().objects().collect();
        assert_eq!(objects.len(), 6);
        // They lie under the original, the furthest lowest.
        assert_eq!(objects[4].id, 101);
        let (furthest, nearest) = (objects[1], objects[3]);
        assert_eq!(nearest.bounds, Rect { x: 70.0, y: 20.0, w: 20.0, h: 20.0 });
        assert_eq!((furthest.bounds.center(), furthest.bounds.w, furthest.opacity), (v2(100.0, 30.0), 5.0, 0.0));
        assert!((nearest.opacity - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn add_shadow_lays_a_dark_copy_under_the_object() {
        let mut e = editor();
        e.selection = vec![100];
        assert_eq!(e.add_shadows(), 1);
        let objects: Vec<&Object> = e.doc.page().objects().collect();
        assert_eq!(objects.len(), 4);
        // The shadow lies under the object it belongs to, half as tall, with the same foot, leaning to the right.
        let (shadow, original) = (objects[0], objects[1]);
        assert_eq!(original.id, 100);
        assert_eq!((shadow.fill, shadow.opacity), (Some(Color::BLACK), 0.4));
        assert!((shadow.bounds.bottom() - original.bounds.bottom()).abs() < 0.01 && (shadow.bounds.h - 20.0).abs() < 0.01);
        assert!(shadow.bounds.right() > original.bounds.right() + 5.0 && (shadow.bounds.x - original.bounds.x).abs() < 0.01);
        assert_eq!(e.selection, vec![100]);
    }

    #[test]
    fn the_ready_document_commands_work_on_layers_and_states() {
        let mut e = editor();
        e.selection = vec![100, 102];
        assert_eq!(e.distribute_to_layers(), 2);
        let page = e.doc.page();
        assert_eq!(page.layers.len(), 3);
        assert_eq!(page.layers[0].objects.iter().map(|o| o.id).collect::<Vec<_>>(), [101]);
        assert_eq!((page.layers[1].objects[0].id, page.layers[2].objects[0].id), (100, 102));
        assert_eq!(page.layers[2].name, "r102");
        assert!(e.set_other_layers(true) && !e.set_other_layers(true));
        assert_eq!(e.doc.page().layers.iter().map(|l| l.visible).collect::<Vec<_>>(), [false, false, true]);
        assert!(e.set_other_layers(false));
        assert_eq!(e.doc.page().layers.iter().map(|l| l.locked).collect::<Vec<_>>(), [true, true, false]);
        // Three states: the first has the three objects, the others are empty. Turned round, the last has them.
        e.doc.page_mut().add_state();
        e.doc.page_mut().add_state();
        e.set_active_state(0);
        assert_eq!(e.doc.page().objects().count(), 3);
        assert!(e.reverse_states(0, 2) && !e.reverse_states(1, 1));
        let page = e.doc.page();
        assert_eq!(page.active_state, 2);
        assert_eq!(page.objects().count(), 3);
        assert_eq!(page.at_state(0).objects().count(), 0);
        assert_eq!(page.states.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["State 3", "State 2", "State 1"]);
    }

    #[test]
    fn animate_selection_lays_a_copy_in_each_state() {
        let mut e = editor();
        e.selection = vec![100];
        let settings = AnimationSettings { states: 5, distance: 80.0, direction: 0.0, scale: 200.0, opacity_from: 100.0, opacity_to: 0.0, ..Default::default() };
        assert!(e.animate_selection(settings));
        let page = e.doc.page();
        assert_eq!((page.states.len(), page.animations.len()), (5, 1));
        assert_eq!(e.doc.symbols[0].kind, SymbolKind::Animation);
        assert_eq!(e.animation_path(0), [20.0, 40.0, 60.0, 80.0, 100.0].map(|x| v2(x, 30.0)));
        let last = page.at_state(4);
        let copy = last.objects().find(|o| o.id == page.animations[0].objects[4]).unwrap();
        assert_eq!(copy.bounds, Rect { x: 60.0, y: -10.0, w: 80.0, h: 80.0 });
        assert_eq!(copy.opacity, 0.0);
        // The first state keeps the copy that was there, at full strength, and it is the one selected.
        assert_eq!(e.selected()[0].bounds, Rect { x: 0.0, y: 10.0, w: 40.0, h: 40.0 });
        assert_eq!(e.selected()[0].opacity, 1.0);
        // Up is up: 90 degrees takes the object toward the top of the page.
        assert!(e.animate_selection(AnimationSettings { states: 3, distance: 10.0, direction: 90.0, ..Default::default() }));
        assert_eq!(e.animation_path(0)[2], v2(20.0, 20.0));
        let page = e.doc.page();
        assert_eq!(page.animations[0].objects.len(), 3);
        // Fewer states: the copies beyond them are gone, the states stay.
        assert_eq!(page.states.len(), 5);
        assert_eq!(page.at_state(4).objects().count(), 0);
        assert_eq!(page.at_state(2).objects().count(), 1);
        // Moved as a whole, every copy goes along.
        e.shift_animation(0, 5.0, 7.0, false);
        assert_eq!(e.animation_path(0)[0], v2(25.0, 37.0));
        assert_eq!(e.doc.page().at_state(2).objects().next().unwrap().bounds.center(), v2(25.0, 27.0));
        // With one copy moved by hand, only the others follow.
        e.shift_animation(0, 1.0, 0.0, true);
        assert_eq!(e.selected()[0].bounds.center(), v2(25.0, 37.0));
        assert_eq!(e.doc.page().at_state(2).objects().next().unwrap().bounds.center(), v2(26.0, 27.0));
        assert!(e.remove_animation() && !e.remove_animation());
        let page = e.doc.page();
        assert!(page.animations.is_empty() && page.at_state(1).objects().count() == 0 && page.objects().count() == 3);
    }

    fn order(e: &Editor) -> Vec<u64> {
        e.doc.page().layers[0].objects.iter().map(|o| o.id).collect()
    }

    fn bitmap(w: u32, h: u32) -> Bitmap {
        Bitmap::from_rgba(w, h, [255u8, 255, 255, 255].repeat((w * h) as usize)).unwrap()
    }

    #[test]
    fn pixel_at_accounts_for_scaled_bounds() {
        let mut o = Object::new_bitmap(1, "b", bitmap(100, 50), 10.0, 10.0);
        o.set_bounds(Rect { x: 10.0, y: 10.0, w: 200.0, h: 100.0 });
        assert_eq!(o.pixel_at(v2(10.0, 10.0)), Some((0, 0)));
        assert_eq!(o.pixel_at(v2(110.0, 60.0)), Some((50, 25)));
        assert_eq!(o.pixel_at(v2(210.0, 110.0)), Some((99, 49)));
        assert_eq!(o.pixel_at(v2(0.0, 0.0)), None);
    }

    #[test]
    fn stroke_maps_through_object_position_and_scale() {
        let mut o = Object::new_bitmap(1, "b", bitmap(40, 40), 100.0, 100.0);
        o.set_bounds(Rect { x: 100.0, y: 100.0, w: 80.0, h: 80.0 });
        o.stroke_bitmap(&[v2(120.0, 140.0), v2(160.0, 140.0)], 8.0, PixelStroke::Paint(Color::BLACK), None);
        let m = o.bitmap().unwrap();
        assert_eq!(m.pixel(20, 20), Some([0, 0, 0, 255]));
        assert_eq!(m.pixel(20, 5), Some([255, 255, 255, 255]));
    }

    #[test]
    fn bitmap_edits_are_copy_on_write_and_undoable() {
        let mut e = Editor::new(Document::from_bitmap("photo", bitmap(20, 20)));
        let original = e.object(1).unwrap().bitmap().unwrap().clone();
        e.object_mut(1).unwrap().stroke_bitmap(&[v2(5.0, 5.0)], 4.0, PixelStroke::Paint(Color::BLACK), None);
        e.commit("Brush");
        assert_ne!(e.object(1).unwrap().bitmap().unwrap(), &original);
        e.undo();
        assert_eq!(e.object(1).unwrap().bitmap().unwrap(), &original);
        e.redo();
        assert_ne!(e.object(1).unwrap().bitmap().unwrap(), &original);
    }

    #[test]
    fn document_from_bitmap_matches_image_and_saves() {
        let doc = Document::from_bitmap("photo", bitmap(30, 20));
        assert_eq!((doc.page().width, doc.page().height), (30, 20));
        assert_eq!(doc.page().canvas, None);
        assert_eq!(doc.next_id, 2);
        let json = document_to_json(&doc).unwrap();
        assert_eq!(document_from_json(&json).unwrap(), doc);
    }

    #[test]
    fn flip_also_flips_pixels() {
        let mut pixels = bitmap(2, 1);
        pixels.paint_stroke(&[v2(0.5, 0.5)], 1.0, Color::BLACK);
        let left = pixels.pixel(0, 0);
        let mut o = Object::new_bitmap(1, "b", pixels, 0.0, 0.0);
        o.flip(true);
        assert_eq!(o.bitmap().unwrap().pixel(1, 0), left);
    }

    #[test]
    fn filter_only_touches_selected_bitmaps() {
        let mut e = Editor::new(Document::from_bitmap("a", bitmap(2, 2)));
        e.apply_filter(Filter::Invert, None);
        assert_eq!(e.object(1).unwrap().bitmap().unwrap().pixel(0, 0), Some([255, 255, 255, 255]));
        e.selection = vec![1];
        assert!(e.has_selected_bitmap());
        e.apply_filter(Filter::Invert, None);
        assert_eq!(e.object(1).unwrap().bitmap().unwrap().pixel(0, 0), Some([0, 0, 0, 255]));
    }

    #[test]
    fn filter_respects_pixel_selection() {
        let mut e = Editor::new(Document::from_bitmap("a", bitmap(4, 1)));
        e.selection = vec![1];
        let mask = Mask::rect(4, 1, v2(0.0, 0.0), v2(2.0, 1.0));
        e.apply_filter(Filter::Invert, Some((1, &mask)));
        let pixels = e.object(1).unwrap().bitmap().unwrap();
        assert_eq!(pixels.pixel(0, 0), Some([0, 0, 0, 255]));
        assert_eq!(pixels.pixel(3, 0), Some([255, 255, 255, 255]));
    }

    #[test]
    fn copied_pixels_paste_where_they_came_from_at_object_scale() {
        let mut o = Object::new_bitmap(50, "a", bitmap(10, 10), 100.0, 50.0);
        o.set_bounds(Rect { x: 100.0, y: 50.0, w: 20.0, h: 20.0 });
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(o);
        e.copy_pixels(50, &Mask::rect(10, 10, v2(2.0, 3.0), v2(6.0, 8.0)));
        e.paste(0.0);
        let piece = e.selected()[0];
        assert_eq!(piece.bounds, Rect { x: 104.0, y: 56.0, w: 8.0, h: 10.0 });
        assert_eq!(piece.bitmap().unwrap().width(), 4);
    }

    #[test]
    fn shape_converts_to_closed_path() {
        let mut o = rect(1, 0.0, 0.0, 10.0, 10.0);
        assert!(o.to_path());
        assert_eq!(o.shape, Shape::Path { closed: true });
        assert_eq!(o.points.len(), 4);
        assert!(!Object::new_bitmap(2, "b", bitmap(1, 1), 0.0, 0.0).to_path());
    }

    #[test]
    fn subdivide_limits_segment_length() {
        let open = subdivide(&[v2(0.0, 0.0), v2(10.0, 0.0)], false, 2.5);
        assert_eq!(open.len(), 5);
        assert_eq!(*open.last().unwrap(), v2(10.0, 0.0));
        let closed = subdivide(&[v2(0.0, 0.0), v2(4.0, 0.0), v2(4.0, 4.0), v2(0.0, 4.0)], true, 2.0);
        assert_eq!(closed.len(), 8);
    }

    #[test]
    fn pull_fades_with_distance() {
        let base = [v2(0.0, 0.0), v2(5.0, 0.0), v2(20.0, 0.0)];
        let pulled = pull_points(&base, v2(0.0, 0.0), v2(0.0, 10.0), 10.0);
        assert_eq!(pulled[0], v2(0.0, 10.0));
        assert!(pulled[1].y > 0.0 && pulled[1].y < 10.0);
        assert_eq!(pulled[2], base[2]);
    }

    #[test]
    fn knife_splits_open_path_in_two() {
        let pieces = split_path(&[v2(0.0, 0.0), v2(10.0, 0.0)], false, v2(5.0, -5.0), v2(5.0, 5.0)).unwrap();
        assert_eq!(pieces, vec![vec![v2(0.0, 0.0), v2(5.0, 0.0)], vec![v2(5.0, 0.0), v2(10.0, 0.0)]]);
        assert!(split_path(&[v2(0.0, 0.0), v2(10.0, 0.0)], false, v2(50.0, -5.0), v2(50.0, 5.0)).is_none());
    }

    #[test]
    fn knife_through_closed_square_keeps_all_vertices() {
        let square = [v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)];
        let pieces = split_path(&square, true, v2(5.0, -5.0), v2(5.0, 15.0)).unwrap();
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0], vec![v2(5.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(5.0, 10.0)]);
        assert_eq!(pieces[1], vec![v2(5.0, 10.0), v2(0.0, 10.0), v2(0.0, 0.0), v2(5.0, 0.0)]);
    }

    #[test]
    fn single_cut_opens_a_closed_path_at_that_point() {
        let square = [v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)];
        let pieces = split_path(&square, true, v2(5.0, -5.0), v2(5.0, 5.0)).unwrap();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].len(), 6);
        assert_eq!(pieces[0][0], *pieces[0].last().unwrap());
    }

    #[test]
    fn editor_knife_replaces_only_selected_objects() {
        let mut e = editor();
        e.add_object(rect(200, 0.0, 10.0, 40.0, 40.0));
        e.selection = vec![200];
        assert_eq!(e.knife(v2(20.0, 0.0), v2(20.0, 60.0)), 1);
        assert!(e.object(200).is_none());
        assert!(e.object(101).is_some());
        assert_eq!(e.selection.len(), 2);
        assert_eq!(e.object(e.selection[0]).unwrap().shape, Shape::Path { closed: false });
        e.selection.clear();
        assert_eq!(e.knife(v2(-50.0, -50.0), v2(-40.0, -40.0)), 0);
    }

    #[test]
    fn web_areas_are_transparent_to_art_tools_and_can_be_hidden() {
        let mut e = editor();
        let area = Rect { x: 0.0, y: 0.0, w: 300.0, h: 300.0 };
        e.add_object(Object::new_web(500, WebKind::Hotspot, area));
        let page = e.doc.page();
        assert_eq!(page.hit_test(v2(10.0, 20.0), 1.0), Some(500));
        assert_eq!(page.art_at(v2(10.0, 20.0), 1.0), Some(100));
        assert!(e.web_visible());
        e.set_web_visible(false);
        assert!(e.selection.is_empty());
        assert!(!e.web_visible());
        assert_eq!(e.doc.page().hit_test(v2(10.0, 20.0), 1.0), Some(100));
    }

    #[test]
    fn nearest_path_prefers_selection_and_respects_reach() {
        let mut e = editor();
        e.add_object(rect(200, 0.0, 0.0, 40.0, 40.0));
        e.add_object(rect(201, 2.0, 0.0, 40.0, 40.0));
        let page = e.doc.page();
        assert_eq!(page.nearest_path(v2(2.0, 20.0), 5.0, &[]), Some(201));
        assert_eq!(page.nearest_path(v2(2.0, 20.0), 5.0, &[200]), Some(200));
        assert_eq!(page.nearest_path(v2(1000.0, 1000.0), 5.0, &[]), None);
    }

    #[test]
    fn bitmap_at_prefers_selection_then_top() {
        let mut e = Editor::new(Document::from_bitmap("a", bitmap(20, 20)));
        let top = Object::new_bitmap(2, "b", bitmap(20, 20), 0.0, 0.0);
        e.add_object(top);
        let page = e.doc.page();
        assert_eq!(page.bitmap_at(v2(5.0, 5.0), &[]), Some(2));
        assert_eq!(page.bitmap_at(v2(5.0, 5.0), &[1]), Some(1));
        assert_eq!(page.bitmap_at(v2(50.0, 5.0), &[1]), None);
    }

    #[test]
    fn rect_intersection() {
        let a = Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 };
        assert_eq!(a.intersect(Rect { x: 5.0, y: -5.0, w: 10.0, h: 10.0 }), Some(Rect { x: 5.0, y: 0.0, w: 5.0, h: 5.0 }));
        assert_eq!(a.intersect(Rect { x: 20.0, y: 0.0, w: 5.0, h: 5.0 }), None);
    }

    #[test]
    fn crop_resizes_canvas_and_shifts_objects() {
        let mut e = editor();
        assert!(e.crop_canvas(Rect { x: 50.0, y: 5.0, w: 100.0, h: 60.0 }));
        assert_eq!((e.doc.page().width, e.doc.page().height), (100, 60));
        assert_eq!(e.object(101).unwrap().bounds.x, 0.0);
        assert_eq!(e.object(101).unwrap().bounds.y, 5.0);
        assert!(!e.crop_canvas(Rect { x: 0.0, y: 0.0, w: 0.2, h: 10.0 }));
    }

    #[test]
    fn hex_round_trip_and_rejects_bad_input() {
        assert_eq!(Color::parse_hex("#00a8ff"), Some(Color::rgb(0, 0xa8, 0xff)));
        assert_eq!(Color::parse_hex("f00"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(Color::rgb(1, 2, 255).hex(), "#0102FF");
        assert_eq!(Color::parse_hex("#12"), None);
        assert_eq!(Color::parse_hex("zzzzzz"), None);
        assert_eq!(Color::parse_hex("ééé"), None);
    }

    #[test]
    fn undo_redo_restores_exact_states() {
        let mut e = editor();
        let before = e.doc.clone();
        e.selection = vec![100];
        e.for_selected(|o| o.translate(5.0, 5.0));
        e.commit("Move");
        let after = e.doc.clone();
        e.undo();
        assert_eq!(e.doc, before);
        e.redo();
        assert_eq!(e.doc, after);
    }

    #[test]
    fn commit_after_undo_drops_redo_branch() {
        let mut e = editor();
        e.selection = vec![100];
        e.delete_selected();
        e.commit("Delete");
        e.undo();
        assert!(e.can_redo());
        e.selection = vec![101];
        e.delete_selected();
        e.commit("Delete other");
        assert!(!e.can_redo());
        assert_eq!(order(&e), vec![100, 102]);
    }

    #[test]
    fn commit_without_change_adds_no_history() {
        let mut e = editor();
        let n = e.history.len();
        e.commit("Nothing");
        assert_eq!(e.history.len(), n);
    }

    #[test]
    fn rollback_discards_uncommitted_drag() {
        let mut e = editor();
        let before = e.doc.clone();
        e.selection = vec![100];
        e.for_selected(|o| o.translate(30.0, 0.0));
        e.rollback();
        assert_eq!(e.doc, before);
        assert_eq!(e.selection, vec![100]);
    }

    #[test]
    fn undo_prunes_selection_of_removed_object() {
        let mut e = editor();
        let id = e.alloc_id();
        e.add_object(rect(id, 0.0, 0.0, 5.0, 5.0));
        e.commit("Add");
        assert_eq!(e.selection, vec![id]);
        e.undo();
        assert!(e.selection.is_empty());
    }

    #[test]
    fn hit_test_prefers_top_object_and_skips_hidden_and_locked() {
        let mut e = editor();
        e.add_object(rect(200, 0.0, 10.0, 40.0, 40.0));
        assert_eq!(e.doc.page().hit_test(v2(10.0, 20.0), 1.0), Some(200));
        e.object_mut(200).unwrap().visible = false;
        assert_eq!(e.doc.page().hit_test(v2(10.0, 20.0), 1.0), Some(100));
        e.object_mut(100).unwrap().locked = true;
        assert_eq!(e.doc.page().hit_test(v2(10.0, 20.0), 1.0), None);
    }

    #[test]
    fn unfilled_shape_is_hit_on_stroke_only() {
        let mut o = rect(1, 0.0, 0.0, 100.0, 100.0);
        o.fill = None;
        o.stroke = Some(Stroke { color: Color::BLACK, width: 2.0 });
        assert!(!o.hit(v2(50.0, 50.0), 1.0));
        assert!(o.hit(v2(0.5, 50.0), 1.0));
    }

    #[test]
    fn set_bounds_scales_path_vertices() {
        let mut o = rect(1, 10.0, 10.0, 10.0, 10.0);
        o.shape = Shape::Path { closed: false };
        o.points = vec![v2(10.0, 10.0), v2(20.0, 20.0)];
        o.set_bounds(Rect { x: 0.0, y: 0.0, w: 20.0, h: 40.0 });
        assert_eq!(o.points, vec![v2(0.0, 0.0), v2(20.0, 40.0)]);
    }

    #[test]
    fn flip_mirrors_vertices_inside_bounds() {
        let mut o = rect(1, 10.0, 0.0, 10.0, 10.0);
        o.shape = Shape::Line;
        o.points = vec![v2(10.0, 0.0), v2(20.0, 10.0)];
        o.flip(true);
        assert_eq!(o.points, vec![v2(20.0, 0.0), v2(10.0, 10.0)]);
    }

    #[test]
    fn arrange_moves_selection_through_stack() {
        let mut e = editor();
        e.selection = vec![100];
        e.arrange(Arrange::Forward);
        assert_eq!(order(&e), vec![101, 100, 102]);
        e.arrange(Arrange::Front);
        assert_eq!(order(&e), vec![101, 102, 100]);
        e.arrange(Arrange::Backward);
        assert_eq!(order(&e), vec![101, 100, 102]);
        e.arrange(Arrange::Back);
        assert_eq!(order(&e), vec![100, 101, 102]);
    }

    #[test]
    fn align_uses_selection_bounds_then_canvas_for_single() {
        let mut e = editor();
        e.selection = vec![100, 102];
        e.align(Align::Right);
        assert_eq!(e.object(100).unwrap().bounds.right(), 240.0);
        e.selection = vec![101];
        e.align(Align::CenterX);
        assert_eq!(e.object(101).unwrap().bounds.center().x, 250.0);
    }

    #[test]
    fn duplicate_gets_new_ids_and_selects_copies() {
        let mut e = editor();
        e.selection = vec![100];
        e.duplicate(10.0);
        assert_eq!(e.selection.len(), 1);
        assert_ne!(e.selection[0], 100);
        assert_eq!(e.selected()[0].bounds.x, 10.0);
    }

    #[test]
    fn last_layer_and_page_cannot_be_deleted() {
        let mut e = editor();
        assert!(!e.delete_layer());
        assert!(!e.delete_page());
        e.add_layer();
        e.add_page();
        assert_eq!(e.doc.pages.len(), 2);
        assert!(e.delete_page());
        assert!(e.delete_layer());
    }

    #[test]
    fn add_object_avoids_locked_layer_and_fails_if_all_locked() {
        let mut e = editor();
        e.add_layer();
        e.doc.page_mut().layers[1].locked = true;
        assert!(e.add_object(rect(300, 0.0, 0.0, 1.0, 1.0)));
        assert!(e.doc.page().layers[0].objects.iter().any(|o| o.id == 300));
        e.doc.page_mut().layers[0].locked = true;
        assert!(!e.add_object(rect(301, 0.0, 0.0, 1.0, 1.0)));
    }

    #[test]
    fn file_round_trip_and_validation() {
        let e = editor();
        let json = document_to_json(&e.doc).unwrap();
        assert_eq!(document_from_json(&json).unwrap(), e.doc);
        assert!(document_from_json("{}").is_err());
        let wrong = json.replace("\"emberframe\"", "\"other\"");
        assert!(document_from_json(&wrong).is_err());
        let newer = json.replace("\"version\": 1", "\"version\": 99");
        assert!(document_from_json(&newer).is_err());
    }

    #[test]
    fn dirty_tracks_saved_state_through_undo() {
        let mut e = editor();
        e.mark_saved();
        assert!(!e.is_dirty());
        e.selection = vec![100];
        e.delete_selected();
        e.commit("Delete");
        assert!(e.is_dirty());
        e.undo();
        assert!(!e.is_dirty());
    }

    fn state_ids(e: &Editor) -> Vec<u64> {
        e.doc.page().objects().map(|o| o.id).collect()
    }

    #[test]
    fn states_keep_their_own_objects() {
        let mut e = Editor::new(Document::new(100, 100));
        e.add_object(rect(1, 0.0, 0.0, 5.0, 5.0));
        e.commit("Add");
        e.doc.page_mut().add_state();
        e.commit("New State");
        assert!(state_ids(&e).is_empty());
        e.add_object(rect(2, 0.0, 0.0, 5.0, 5.0));
        e.commit("Add");
        e.set_active_state(0);
        assert_eq!(state_ids(&e), vec![1]);
        e.set_active_state(1);
        assert_eq!(state_ids(&e), vec![2]);
    }

    #[test]
    fn duplicate_state_copies_and_delete_keeps_the_last() {
        let mut page = Page::new("p", 10, 10);
        page.layers[0].objects.push(rect(1, 0.0, 0.0, 1.0, 1.0));
        page.duplicate_state();
        assert_eq!(page.active_state, 1);
        assert_eq!(page.objects().count(), 1);
        page.layers[0].objects.clear();
        assert!(page.delete_state());
        assert_eq!(page.active_state, 0);
        assert_eq!(page.objects().count(), 1);
        assert!(!page.delete_state());
    }

    #[test]
    fn shared_layers_appear_in_every_state() {
        let mut page = Page::new("p", 10, 10);
        page.layers[0].objects.push(rect(1, 0.0, 0.0, 1.0, 1.0));
        page.add_state();
        assert_eq!(page.objects().count(), 0);
        page.goto_state(0);
        page.set_layer_shared(0, true);
        page.goto_state(1);
        assert_eq!(page.objects().count(), 1);
        page.set_layer_shared(0, false);
        page.goto_state(0);
        assert_eq!(page.objects().count(), 1);
    }

    #[test]
    fn layers_added_and_removed_stay_lined_up_across_states() {
        let mut e = Editor::new(Document::new(10, 10));
        e.doc.page_mut().add_state();
        e.add_layer();
        e.doc.page_mut().goto_state(0);
        assert_eq!(e.doc.page().states[1].objects.len(), 2);
        assert!(e.delete_layer());
        assert_eq!(e.doc.page().states[1].objects.len(), 1);
    }

    #[test]
    fn switching_state_or_page_is_not_an_edit() {
        let mut e = Editor::new(Document::new(10, 10));
        e.doc.page_mut().add_state();
        e.add_page();
        e.commit("Setup");
        e.mark_saved();
        e.set_active_page(0);
        e.set_active_state(0);
        assert!(!e.is_dirty());
        assert_eq!(e.doc, e.history[e.history_pos].1);
        e.rollback();
        assert_eq!(e.doc.active_page, 0);
    }

    fn symbol_editor() -> (Editor, u64) {
        let mut e = Editor::new(Document::new(200, 200));
        e.doc.next_id = 50;
        e.add_object(rect(1, 10.0, 10.0, 20.0, 20.0));
        e.add_object(rect(2, 30.0, 10.0, 20.0, 40.0));
        e.select_all();
        let id = e.make_symbol().unwrap();
        (e, id)
    }

    #[test]
    fn symbol_replaces_selection_and_keeps_position() {
        let (e, _) = symbol_editor();
        let objects: Vec<_> = e.doc.page().objects().collect();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].bounds, Rect { x: 10.0, y: 10.0, w: 40.0, h: 40.0 });
        let parts = objects[0].instance_parts();
        assert_eq!(parts[1].bounds, Rect { x: 30.0, y: 10.0, w: 20.0, h: 40.0 });
    }

    #[test]
    fn scaled_instance_scales_its_parts() {
        let (mut e, _) = symbol_editor();
        e.for_selected(|o| o.set_bounds(Rect { x: 0.0, y: 0.0, w: 80.0, h: 80.0 }));
        let object = e.doc.page().objects().next().unwrap();
        assert_eq!(object.instance_parts()[1].bounds, Rect { x: 40.0, y: 0.0, w: 40.0, h: 80.0 });
    }

    #[test]
    fn break_apart_restores_editable_objects() {
        let (mut e, _) = symbol_editor();
        assert_eq!(e.break_apart(), 1);
        assert_eq!(e.doc.page().objects().count(), 2);
        assert_eq!(e.selection.len(), 2);
    }

    #[test]
    fn redefining_a_symbol_updates_every_copy() {
        let (mut e, id) = symbol_editor();
        assert!(e.place_symbol(id));
        e.add_object(rect(3, 0.0, 0.0, 7.0, 7.0));
        assert!(e.redefine_symbol(id));
        for object in e.doc.page().objects().filter(|o| o.is_instance()) {
            assert_eq!(object.instance_parts().len(), 1);
        }
    }

    #[test]
    fn a_symbol_cannot_contain_itself() {
        let (mut e, id) = symbol_editor();
        e.select_all();
        assert!(!e.redefine_symbol(id));
    }

    #[test]
    fn deleting_a_symbol_removes_its_copies_in_every_state() {
        let (mut e, id) = symbol_editor();
        e.doc.page_mut().duplicate_state();
        e.delete_symbol(id);
        assert_eq!(e.doc.page().objects().count(), 0);
        e.doc.page_mut().goto_state(0);
        assert_eq!(e.doc.page().objects().count(), 0);
    }

    #[test]
    fn symbols_states_and_styles_survive_saving() {
        let (mut e, _) = symbol_editor();
        e.doc.page_mut().duplicate_state();
        e.doc.urls.push("https://example.com".into());
        let loaded = document_from_json(&document_to_json(&e.doc).unwrap()).unwrap();
        assert_eq!(loaded, e.doc);
    }

    #[test]
    fn older_files_get_default_panel_data() {
        let mut value: serde_json::Value = serde_json::from_str(&document_to_json(&Document::new(10, 10)).unwrap()).unwrap();
        let document = value["document"].as_object_mut().unwrap();
        document.remove("styles");
        document.remove("symbols");
        document.remove("urls");
        let page = document["pages"][0].as_object_mut().unwrap();
        page.remove("states");
        page.remove("active_state");
        page.remove("export");
        let loaded = document_from_json(&value.to_string()).unwrap();
        assert_eq!(loaded.pages[0].states.len(), 1);
        assert!(!loaded.styles.is_empty());
    }

    #[test]
    fn bridge_and_contours_round_trip() {
        let outer = vec![v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)];
        let hole = vec![v2(2.0, 2.0), v2(2.0, 8.0), v2(8.0, 8.0), v2(8.0, 2.0)];
        let joined = bridge(outer, vec![hole]);
        assert!(!point_in_polygon(v2(5.0, 5.0), &joined));
        assert!(point_in_polygon(v2(1.0, 5.0), &joined));
        let rings = contours(&joined);
        assert_eq!(rings.len(), 2);
        assert_eq!(rings.iter().map(Vec::len).sum::<usize>(), 8);
    }

    #[test]
    fn plain_outlines_are_one_ring() {
        let square = vec![v2(0.0, 0.0), v2(1.0, 0.0), v2(1.0, 1.0), v2(0.0, 1.0)];
        assert_eq!(contours(&square), vec![square]);
    }

    fn close(a: Vec2, b: Vec2) -> bool {
        (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01
    }

    #[test]
    fn rotation_turns_clockwise_on_screen() {
        let quarter = Warp::rotation(v2(0.0, 0.0), std::f32::consts::FRAC_PI_2);
        // Right becomes down, because y grows downward.
        assert!(close(quarter.apply(v2(1.0, 0.0)), v2(0.0, 1.0)));
        let about = Warp::rotation(v2(10.0, 10.0), std::f32::consts::PI);
        assert!(close(about.apply(v2(10.0, 10.0)), v2(10.0, 10.0)));
        assert!(close(about.apply(v2(12.0, 13.0)), v2(8.0, 7.0)));
    }

    #[test]
    fn shear_keeps_one_edge_still() {
        // The line y = 50 stays, and the line y = 0 slides 20 to the right.
        let slide = Warp::shear_x(50.0, 20.0 / -50.0);
        assert!(close(slide.apply(v2(7.0, 50.0)), v2(7.0, 50.0)));
        assert!(close(slide.apply(v2(7.0, 0.0)), v2(27.0, 0.0)));
        let lift = Warp::shear_y(0.0, 10.0 / 100.0);
        assert!(close(lift.apply(v2(0.0, 5.0)), v2(0.0, 5.0)));
        assert!(close(lift.apply(v2(100.0, 5.0)), v2(100.0, 15.0)));
    }

    #[test]
    fn quad_sends_each_corner_to_its_target_and_can_be_undone() {
        let from = Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0 };
        let to = [v2(0.0, 0.0), v2(120.0, 10.0), v2(100.0, 90.0), v2(15.0, 60.0)];
        let warp = Warp::quad(from, to).unwrap();
        let corners = [v2(10.0, 20.0), v2(110.0, 20.0), v2(110.0, 70.0), v2(10.0, 70.0)];
        for (corner, target) in corners.iter().zip(to) {
            assert!(close(warp.apply(*corner), target), "{corner:?}");
        }
        let back = warp.inverse().unwrap();
        let inside = v2(42.0, 33.0);
        assert!(close(back.apply(warp.apply(inside)), inside));
        // The same rectangle maps onto itself without moving anything.
        let same = Warp::quad(from, corners).unwrap();
        assert!(close(same.apply(inside), inside));
    }

    #[test]
    fn quad_refuses_shapes_that_fold_over() {
        let from = Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 };
        // The top-left corner dragged past the opposite corner.
        assert!(Warp::quad(from, [v2(20.0, 20.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)]).is_none());
        assert!(Warp::quad(Rect { w: 0.0, ..from }, [v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0), v2(0.0, 10.0)]).is_none());
    }

    #[test]
    fn a_turned_rectangle_is_still_a_rectangle() {
        let mut o = rect(1, 0.0, 0.0, 100.0, 50.0);
        o.shape = Shape::Rect { radius: 8.0 };
        assert!(o.warp(&Warp::rotation(o.bounds.center(), std::f32::consts::FRAC_PI_2)));
        assert_eq!(o.shape, Shape::Rect { radius: 8.0 }, "the roundness can still be edited");
        let b = o.bounds;
        assert!((b.x - 25.0).abs() < 0.01 && (b.y + 25.0).abs() < 0.01 && (b.w - 50.0).abs() < 0.01 && (b.h - 100.0).abs() < 0.01, "{b:?}");
        // Changing the roundness reshapes the turned rectangle.
        let before = o.outline();
        o.shape = Shape::Rect { radius: 0.0 };
        o.refit_placed();
        assert_eq!(o.outline().len(), 4);
        assert_ne!(o.outline(), before);
        // Standing it up again gives back the rectangle it was.
        assert!(o.remove_transform());
        assert_eq!(o.bounds, Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0 });
    }

    #[test]
    fn a_turned_circle_stays_round_and_becomes_four_curve_points_as_a_path() {
        let mut o = rect(1, 0.0, 0.0, 100.0, 100.0);
        o.shape = Shape::Ellipse;
        assert!(o.warp(&Warp::rotation(v2(50.0, 50.0), 0.5)));
        assert_eq!(o.shape, Shape::Ellipse);
        let round = |o: &Object, slack: f32| {
            for p in o.outline() {
                let radius = ((p.x - 50.0).powi(2) + (p.y - 50.0).powi(2)).sqrt();
                assert!((radius - 50.0).abs() < slack, "radius was {radius}");
            }
        };
        round(&o, 0.01);
        assert!((o.bounds.w - 100.0).abs() < 0.1 && (o.bounds.h - 100.0).abs() < 0.1);
        // As a path it is four curve points, not a polygon, and keeps its turn.
        assert!(o.to_path());
        assert_eq!(o.shape, Shape::Path { closed: true });
        assert_eq!(o.points.len(), 4);
        assert!(o.is_curved() && o.placement.is_none());
        round(&o, 0.05);
        let first = o.points[0];
        assert!((first.x - (50.0 + 50.0 * 0.5_f32.cos())).abs() < 0.01 && (first.y - (50.0 + 50.0 * 0.5_f32.sin())).abs() < 0.01, "{first:?}");
    }

    #[test]
    fn rounded_corners_become_curves_when_a_rectangle_becomes_a_path() {
        let mut o = rect(1, 0.0, 0.0, 100.0, 60.0);
        o.shape = Shape::Rect { radius: 20.0 };
        let before = o.outline();
        assert!(o.to_path());
        assert_eq!(o.points.len(), 8);
        let b = o.bounds;
        assert!(b.x.abs() < 0.01 && b.y.abs() < 0.01 && (b.w - 100.0).abs() < 0.01 && (b.h - 60.0).abs() < 0.01);
        // The curves follow the arcs they replace: no point of the old outline is far from the new one.
        let after = o.outline();
        for p in before {
            let nearest = after.iter().map(|q| ((p.x - q.x).powi(2) + (p.y - q.y).powi(2)).sqrt()).fold(f32::MAX, f32::min);
            assert!(nearest < 1.6, "{p:?} is {nearest} away");
        }
    }

    #[test]
    fn flipping_mirrors_text_symbols_and_groups() {
        // Text keeps its box and its words, and is drawn mirrored through its placement.
        let mut text = label("Pardus");
        let bounds = text.bounds;
        text.flip(true);
        assert!(same_rect(text.bounds, bounds));
        assert!(matches!(&text.shape, Shape::Text { content, .. } if content == "Pardus"));
        let placement = text.placement.expect("a mirrored text carries a placement");
        let left_edge = placement.warp.apply(v2(bounds.x, bounds.y));
        assert!((left_edge.x - bounds.right()).abs() < 0.01, "the start of the text is now on the right");
        // Flipping twice is no flip at all.
        text.flip(true);
        let back = text.placement.unwrap().warp.apply(v2(bounds.x, bounds.y));
        assert!((back.x - bounds.x).abs() < 0.01 && (back.y - bounds.y).abs() < 0.01);

        // In a group the parts change sides.
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 0.0, 0.0, 20.0, 20.0));
        e.add_object(rect(2, 80.0, 40.0, 10.0, 10.0));
        e.selection = vec![1, 2];
        assert!(e.group_selected());
        let mut group = e.selected()[0].clone();
        let whole = group.bounds;
        group.flip(true);
        assert!(same_rect(group.bounds, whole));
        let parts = group.instance_parts();
        assert!(same_rect(parts[0].bounds, Rect { x: 70.0, y: 0.0, w: 20.0, h: 20.0 }));
        assert!(same_rect(parts[1].bounds, Rect { x: 0.0, y: 40.0, w: 10.0, h: 10.0 }));
        group.flip(false);
        let parts = group.instance_parts();
        assert!(same_rect(parts[0].bounds, Rect { x: 70.0, y: 30.0, w: 20.0, h: 20.0 }));
    }

    #[test]
    fn skewing_a_curve_moves_its_handles_with_it() {
        let mut o = arch();
        // Slide the top right by 45 while y = 0 stays where it is.
        assert!(o.warp(&Warp::shear_x(0.0, -1.0)));
        assert_eq!(o.points, vec![v2(0.0, 0.0), v2(100.0, 0.0)]);
        assert!(close(o.handles[0].after, v2(60.0, -60.0)));
        let top = o.outline().into_iter().min_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
        assert!((top.y + 45.0).abs() < 0.5 && (top.x - 95.0).abs() < 2.0, "{top:?}");
    }

    #[test]
    fn a_group_turns_as_one() {
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 0.0, 0.0, 20.0, 20.0));
        e.add_object(rect(2, 80.0, 0.0, 20.0, 20.0));
        e.selection = vec![1, 2];
        assert!(e.group_selected());
        let mut group = e.selected()[0].clone();
        let bounds = group.bounds;
        assert!(group.warp(&Warp::rotation(bounds.center(), std::f32::consts::FRAC_PI_2)));
        let b = group.bounds;
        assert!((b.w - 20.0).abs() < 0.01 && (b.h - 100.0).abs() < 0.01, "{b:?}");
        assert_eq!(group.instance_parts().len(), 2);
    }

    #[test]
    fn web_areas_are_left_alone() {
        let mut web = Object::new_web(1, WebKind::Slice, Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 });
        let before = web.clone();
        assert!(!web.warpable());
        assert!(!web.warp(&Warp::rotation(v2(0.0, 0.0), 1.0)));
        assert_eq!(web, before);
    }

    #[test]
    fn a_bitmap_is_redrawn_when_it_turns() {
        // Red on the left, blue on the right.
        let pixels = Bitmap::from_rgba(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]).unwrap();
        let mut o = Object::new_bitmap(1, "b", pixels, 0.0, 0.0);
        assert!(o.warp(&Warp::rotation(o.bounds.center(), std::f32::consts::PI)));
        let Shape::Image { bitmap } = &o.shape else { panic!("still a bitmap") };
        assert_eq!((bitmap.width(), bitmap.height()), (2, 1));
        assert_eq!(bitmap.pixel(0, 0), Some([0, 0, 255, 255]));
        assert_eq!(bitmap.pixel(1, 0), Some([255, 0, 0, 255]));

        // Turned an eighth of a turn, a square needs a larger box with empty corners.
        let square = Bitmap::from_rgba(40, 40, [10, 200, 30, 255].repeat(1600)).unwrap();
        let mut o = Object::new_bitmap(2, "s", square, 0.0, 0.0);
        assert!(o.warp(&Warp::rotation(v2(20.0, 20.0), std::f32::consts::FRAC_PI_4)));
        let Shape::Image { bitmap } = &o.shape else { panic!("still a bitmap") };
        assert_eq!((bitmap.width(), bitmap.height()), (58, 58));
        assert_eq!(o.bounds, Rect { x: -9.0, y: -9.0, w: 58.0, h: 58.0 });
        assert_eq!(bitmap.pixel(0, 0), Some([0, 0, 0, 0]));
        assert_eq!(bitmap.pixel(29, 29), Some([10, 200, 30, 255]));
    }

    #[test]
    fn text_becomes_paths_with_the_holes_of_its_letters() {
        let mut o = rect(1, 10.0, 10.0, 60.0, 120.0);
        o.shape = Shape::Text { content: "o".into(), size: 100.0, style: TextStyle::default() };
        o.fill = Some(Color::BLACK);
        // Without an installed font there is nothing to outline.
        if !o.text_to_paths() {
            return;
        }
        let parts = o.instance_parts();
        assert_eq!(parts.len(), 1);
        assert_eq!(contours(&parts[0].outline()).len(), 2, "an outer ring and the hole");
        let center = parts[0].bounds.center();
        assert!(!parts[0].hit(center, 0.5), "the middle of the letter is empty");
    }

    fn label(content: &str) -> Object {
        let mut o = rect(1, 10.0, 20.0, 80.0, 30.0);
        o.shape = Shape::Text { content: content.into(), size: 24.0, style: TextStyle::default() };
        o.fill = Some(Color::BLACK);
        o
    }

    fn same_rect(a: Rect, b: Rect) -> bool {
        (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01 && (a.w - b.w).abs() < 0.01 && (a.h - b.h).abs() < 0.01
    }

    #[test]
    fn a_placement_follows_moving_and_scaling() {
        let mut o = label("Pardus");
        assert!(o.warp(&Warp::rotation(o.bounds.center(), 0.6)));
        let turned = o.bounds;
        assert!(turned.h > 30.0, "a turned label is taller than it was");

        o.translate(5.0, -3.0);
        assert!(same_rect(o.bounds, Rect { x: turned.x + 5.0, y: turned.y - 3.0, ..turned }));
        assert!(same_rect(o.placement.unwrap().bounds(), o.bounds), "the placement moved with the box");

        let doubled = Rect { w: o.bounds.w * 2.0, h: o.bounds.h * 2.0, ..o.bounds };
        o.set_bounds(doubled);
        assert!(same_rect(o.placement.unwrap().bounds(), doubled), "and grew with it");
        // The text itself was never touched.
        assert!(matches!(&o.shape, Shape::Text { content, size, .. } if content == "Pardus" && *size == 24.0));
        assert_eq!(o.placement.unwrap().frame, Rect { x: 10.0, y: 20.0, w: 80.0, h: 30.0 });
    }

    #[test]
    fn removing_a_transform_stands_the_object_up_where_it_was() {
        let mut o = label("Pardus");
        let center = o.bounds.center();
        assert!(!o.remove_transform(), "nothing to remove yet");
        o.warp(&Warp::rotation(center, 1.0));
        assert!(o.remove_transform());
        assert!(o.placement.is_none());
        assert!(same_rect(o.bounds, Rect { x: 10.0, y: 20.0, w: 80.0, h: 30.0 }));
    }

    #[test]
    fn editing_turned_text_resizes_its_frame_not_its_box() {
        let mut o = label("Pardus");
        o.refit_text();
        let upright = o.bounds;
        o.warp(&Warp::quarter_turn(upright.center(), true));
        if let Shape::Text { content, .. } = &mut o.shape {
            content.push_str(" 25");
        }
        o.refit_text();
        let frame = o.placement.unwrap().frame;
        // Without an installed font text is measured by a rule of thumb, which still grows with its length.
        assert!(frame.w > upright.w, "the upright frame got wider");
        assert!((o.bounds.h - frame.w).abs() < 0.01, "so the turned box got taller");
        assert!((o.bounds.w - frame.h).abs() < 0.01);
    }

    #[test]
    fn turned_text_is_drawn_and_converted_where_it_shows() {
        let mut o = label("o");
        o.refit_text();
        assert!(o.placed_text_paths().is_none(), "upright text is drawn as text");
        o.warp(&Warp::rotation(o.bounds.center(), 0.7));
        // Without an installed font there are no outlines to draw.
        let Some(paths) = o.placed_text_paths() else { return };
        assert!(matches!(paths.shape, Shape::Group { .. }) && paths.placement.is_none());
        let (drawn, shown) = (paths.bounds, o.bounds);
        assert!(drawn.x >= shown.x - 0.5 && drawn.right() <= shown.right() + 0.5, "{drawn:?} is not inside {shown:?}");
        assert!(drawn.y >= shown.y - 0.5 && drawn.bottom() <= shown.bottom() + 0.5, "{drawn:?} is not inside {shown:?}");
        // Converting to paths keeps the letters where they were drawn.
        let mut converted = o.clone();
        assert!(converted.text_to_paths());
        assert!(same_rect(converted.bounds, drawn));
    }

    #[test]
    fn group_as_mask_puts_the_top_object_over_the_rest() {
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 0.0, 0.0, 100.0, 100.0));
        e.add_object(rect(2, 20.0, 20.0, 40.0, 40.0));
        e.selection = vec![1];
        assert!(!e.group_as_mask(), "one object has nothing to be masked by");
        e.selection = vec![1, 2];
        assert!(e.group_as_mask());
        let masked = e.selected()[0].clone();
        assert_eq!(e.doc.page().objects().count(), 1);
        assert_eq!(masked.id, 1);
        let mask = masked.mask.expect("the box below is masked");
        assert_eq!((mask.object.id, mask.kind), (2, MaskKind::Grayscale));
    }

    #[test]
    fn pasting_as_a_mask_and_pasting_inside() {
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 0.0, 0.0, 100.0, 100.0));
        e.add_object(rect(2, 20.0, 20.0, 40.0, 40.0));
        e.selection = vec![2];
        e.copy();
        // The copied box, as a mask on the other one, works by its outline.
        e.selection = vec![1];
        assert_eq!(e.clipboard_vectors(), 1);
        assert!(e.paste_as_mask(false, false));
        let mask = e.selected()[0].mask.clone().unwrap();
        assert_eq!((mask.kind, mask.show_fill_stroke), (MaskKind::PathOutline, false));
        // Added to the mask that is there, the two are one mask; replaced, only the new one is left.
        assert!(e.paste_as_mask(false, true));
        let joined = e.selected()[0].mask.clone().unwrap();
        assert!(matches!(&joined.object.shape, Shape::Group { objects } if objects.len() == 2));
        assert_eq!(joined.kind, MaskKind::PathOutline);
        // As a bitmap, the copied box is a picture of itself.
        assert!(e.paste_as_mask(true, false));
        let bitmap = e.selected()[0].mask.clone().unwrap();
        assert!(bitmap.object.is_image() && bitmap.kind == MaskKind::Grayscale);
        assert!(e.paste_as_mask(false, false));
        // An unlinked mask stays where it is when its object moves.
        e.toggle_mask_links();
        e.for_selected(|o| o.translate(5.0, 0.0));
        assert_eq!(e.selected()[0].mask.as_ref().unwrap().object.bounds.x, 20.0);
        e.toggle_mask_links();
        e.for_selected(|o| o.translate(-5.0, 0.0));
        assert_eq!(e.selected()[0].mask.as_ref().unwrap().object.bounds.x, 15.0);
        e.for_selected(|o| {
            if let Some(mask) = &mut o.mask {
                Arc::make_mut(mask).object.translate(5.0, 0.0);
            }
        });
        // Pasted inside box 1, a copy of box 2 takes its place, masked by box 1 which stays in view.
        assert!(e.paste_inside());
        let inside = e.selected()[0].clone();
        assert_eq!(inside.bounds, Rect { x: 20.0, y: 20.0, w: 40.0, h: 40.0 });
        let frame = inside.mask.unwrap();
        assert_eq!((frame.object.id, frame.show_fill_stroke), (1, true));
        assert_eq!(e.doc.page().objects().count(), 2);
    }

    #[test]
    fn bitmap_masks_reveal_hide_and_can_be_applied() {
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 10.0, 10.0, 20.0, 20.0));
        e.selection = vec![1];
        assert!(e.add_bitmap_mask(false, None));
        let mask = e.selected()[0].mask.clone().unwrap();
        assert_eq!(mask.object.bounds, Rect { x: 10.0, y: 10.0, w: 20.0, h: 20.0 });
        assert_eq!(mask.object.bitmap().unwrap().pixel(5, 5), Some([0, 0, 0, 255]), "Hide All is a black mask");
        e.toggle_masks();
        assert!(!e.selected()[0].mask.as_ref().unwrap().enabled);
        e.toggle_masks();
        // Thrown away, the box is itself again.
        e.delete_masks(false);
        assert!(e.selected()[0].mask.is_none() && matches!(e.selected()[0].shape, Shape::Rect { .. }));
        // Applied, a hidden box is a bitmap with nothing in it.
        e.add_bitmap_mask(false, None);
        e.delete_masks(true);
        let applied = e.selected()[0].clone();
        assert!(applied.mask.is_none());
        assert_eq!(applied.bitmap().unwrap().pixel(10, 10).map(|p| p[3]), Some(0));
        // Revealing only some pixels of a bitmap makes a mask that is white there and black elsewhere.
        e.selection = vec![applied.id];
        let pixels = crate::bitmap::Mask::from_fn(applied.bitmap().unwrap().width(), applied.bitmap().unwrap().height(), |x, _| x < 3);
        assert!(e.add_bitmap_mask(true, Some(&pixels)));
        let mask = e.selected()[0].mask.clone().unwrap();
        assert_eq!(mask.object.bitmap().unwrap().pixel(1, 1), Some([255, 255, 255, 255]));
        assert_eq!(mask.object.bitmap().unwrap().pixel(8, 1), Some([0, 0, 0, 255]));
    }

    #[test]
    fn editing_a_mask_works_on_a_stand_in_that_is_copied_back() {
        let mut e = Editor::new(Document::new(300, 300));
        // Ids the editor hands out itself, so the stand-in's cannot clash with them.
        let (base, top) = (e.alloc_id(), e.alloc_id());
        e.add_object(rect(base, 0.0, 0.0, 100.0, 100.0));
        e.add_object(rect(top, 20.0, 20.0, 40.0, 40.0));
        e.selection = vec![base, top];
        e.group_as_mask();
        assert!(e.edit_mask());
        assert!(e.editing_mask());
        let stand_in = e.selected()[0].clone();
        assert_eq!(stand_in.mask_of, Some(base));
        assert_eq!(e.doc.page().objects().count(), 2);
        // Moving the stand-in moves the mask, once the two are brought in line.
        e.for_selected(|o| o.translate(30.0, 0.0));
        e.sync_mask_edit();
        let owner = e.object(base).unwrap();
        assert_eq!(owner.mask.as_ref().unwrap().object.bounds.x, 50.0);
        assert_eq!(owner.mask.as_ref().unwrap().object.id, top, "the mask keeps its own identity");
        assert!(e.editing_mask(), "still selected, so still being edited");
        // Selecting something else ends the edit and takes the stand-in away.
        e.selection = vec![base];
        e.sync_mask_edit();
        assert!(!e.editing_mask());
        assert_eq!(e.doc.page().objects().count(), 1);
        assert_eq!(e.object(base).unwrap().mask.as_ref().unwrap().object.bounds.x, 50.0);
        // A stand-in never reaches a file.
        e.edit_mask();
        e.end_mask_edit();
        assert!(!document_to_json(&e.doc).unwrap().contains("mask_of"));
        assert_eq!(e.doc.page().objects().count(), 1);
    }

    #[test]
    fn a_symbol_turns_and_stays_a_symbol() {
        let mut e = Editor::new(Document::new(300, 300));
        e.add_object(rect(1, 0.0, 0.0, 40.0, 20.0));
        e.selection = vec![1];
        let symbol = e.make_symbol().unwrap();
        let mut copy = e.selected()[0].clone();
        assert!(matches!(copy.shape, Shape::Instance { .. }));
        let center = copy.bounds.center();
        assert!(copy.warp(&Warp::quarter_turn(center, true)));
        assert!(matches!(copy.shape, Shape::Instance { symbol: placed, .. } if placed == symbol), "still linked to the library");
        assert!(same_rect(copy.bounds, Rect { x: 10.0, y: -10.0, w: 20.0, h: 40.0 }));
        // What is drawn, and what Break Apart would leave behind, is the turned artwork.
        let parts = copy.instance_parts();
        assert_eq!(parts.len(), 1);
        assert!(same_rect(parts[0].bounds, copy.bounds));
    }

    #[test]
    fn placements_survive_saving() {
        let mut e = Editor::new(Document::new(100, 100));
        let mut o = label("Pardus");
        o.warp(&Warp::rotation(v2(50.0, 50.0), 0.3));
        e.add_object(o);
        let json = document_to_json(&e.doc).unwrap();
        assert!(json.contains("placement"));
        assert_eq!(document_from_json(&json).unwrap(), e.doc);
        // Upright objects are written as they always were.
        let mut e = Editor::new(Document::new(100, 100));
        e.add_object(label("Pardus"));
        assert!(!document_to_json(&e.doc).unwrap().contains("placement"));
    }

    /// An open path from (0,0) to (100,0) that bulges upward: both handles point to y = -60.
    fn arch() -> Object {
        let mut o = rect(1, 0.0, 0.0, 100.0, 0.0);
        o.shape = Shape::Path { closed: false };
        o.fill = None;
        o.points = vec![v2(0.0, 0.0), v2(100.0, 0.0)];
        o.handles = vec![
            Handles { before: v2(0.0, 0.0), after: v2(0.0, -60.0) },
            Handles { before: v2(0.0, -60.0), after: v2(0.0, 0.0) },
        ];
        o.refit_to_points();
        o
    }

    #[test]
    fn corners_flatten_to_the_vertices_themselves() {
        let points = vec![v2(0.0, 0.0), v2(10.0, 0.0), v2(10.0, 10.0)];
        assert_eq!(flatten_curve(&points, &[], true), points);
        assert_eq!(flatten_curve(&points, &[Handles::default(); 3], false), points);
    }

    #[test]
    fn a_curve_keeps_its_ends_and_bulges_toward_its_handles() {
        let o = arch();
        let outline = o.outline();
        assert_eq!(outline.first(), Some(&v2(0.0, 0.0)));
        assert_eq!(outline.last(), Some(&v2(100.0, 0.0)));
        assert!(outline.len() > 8);
        // The top of this curve is three quarters of the way to the handles.
        let top = outline.iter().map(|p| p.y).fold(0.0, f32::min);
        assert!((top + 45.0).abs() < 0.5, "top was {top}");
        // The bounds follow the curve, not just its two vertices.
        assert!((o.bounds.y + 45.0).abs() < 0.5 && (o.bounds.h - 45.0).abs() < 0.5);
    }

    #[test]
    fn a_curve_is_hit_along_the_curve_not_along_the_chord() {
        let o = arch();
        assert!(o.hit(v2(50.0, -45.0), 1.5));
        assert!(!o.hit(v2(50.0, 0.0), 1.5));
    }

    #[test]
    fn only_the_curved_side_of_a_vertex_bends() {
        // A triangle whose first side alone is curved.
        let points = vec![v2(0.0, 0.0), v2(100.0, 0.0), v2(50.0, 80.0)];
        let handles = vec![Handles { before: v2(0.0, 0.0), after: v2(30.0, -30.0) }, Handles::default(), Handles::default()];
        let outline = flatten_curve(&points, &handles, true);
        let first_side = outline.iter().position(|p| *p == points[1]).unwrap();
        assert!(first_side > 2, "the curved side has points between its vertices");
        assert_eq!(&outline[first_side..], &points[1..], "the straight sides have none");
    }

    #[test]
    fn handles_follow_scaling_flipping_and_turning() {
        let mut o = arch();
        let b = o.bounds;
        o.set_bounds(Rect { x: b.x, y: b.y * 2.0, w: b.w * 2.0, h: b.h * 2.0 });
        assert_eq!(o.handles[0].after, v2(0.0, -120.0));

        let mut o = arch();
        o.flip(false);
        assert_eq!(o.handles[0].after, v2(0.0, 60.0));
        let lowest = o.outline().iter().map(|p| p.y).fold(f32::MIN, f32::max);
        assert!(lowest > o.points[0].y, "the arch now hangs below its ends");

        let mut o = arch();
        assert!(o.rotate90(true, v2(0.0, 0.0)));
        // Up becomes right when turning clockwise.
        assert_eq!(o.handles[0].after, v2(60.0, 0.0));
        let moved = arch();
        let mut translated = moved.clone();
        translated.translate(7.0, 9.0);
        assert_eq!(translated.handles, moved.handles);
    }

    #[test]
    fn a_vertex_added_to_a_curve_leaves_its_shape_alone() {
        let mut o = arch();
        let before = o.outline();
        let (segment, along) = o.nearest_segment(v2(50.0, -45.0), 2.0).expect("the top of the arch is on the path");
        assert_eq!(segment, 0);
        assert!((along - 0.5).abs() < 0.05, "along was {along}");
        assert!(o.nearest_segment(v2(50.0, 0.0), 2.0).is_none(), "the chord is not on the path");
        let index = o.insert_point(segment, 0.5);
        assert_eq!((index, o.points.len(), o.handles.len()), (1, 3, 3));
        assert!(close(o.points[1], v2(50.0, -45.0)));
        // The new vertex is smooth, and the curve still passes where it did.
        let h = o.handles[1];
        assert!(close(v2(-h.before.x, -h.before.y), h.after) && h.after.x > 0.0);
        let after = o.outline();
        for p in before {
            let nearest = after.iter().map(|q| ((p.x - q.x).powi(2) + (p.y - q.y).powi(2)).sqrt()).fold(f32::MAX, f32::min);
            assert!(nearest < 1.6, "{p:?} is {nearest} away");
        }
    }

    #[test]
    fn a_vertex_added_to_a_straight_side_is_a_corner() {
        let mut o = rect(1, 0.0, 0.0, 100.0, 50.0);
        assert!(o.to_path());
        // A long side is found even between the points it is sampled at.
        let (segment, along) = o.nearest_segment(v2(37.0, 1.0), 2.0).unwrap();
        assert_eq!(segment, 0);
        let index = o.insert_point(segment, along);
        assert_eq!(o.points.len(), 5);
        assert!((o.points[index].x - 37.0).abs() < 1.0 && o.points[index].y == 0.0, "{:?}", o.points[index]);
        assert!(o.handles.is_empty(), "still all corners");
        // The closing side of a closed path takes one too, at the end of the list.
        let (segment, along) = o.nearest_segment(v2(0.0, 25.0), 2.0).unwrap();
        assert_eq!(segment, 4);
        assert_eq!(o.insert_point(segment, along), 5);
    }

    #[test]
    fn removing_vertices_stops_at_a_line() {
        let mut o = arch();
        o.insert_point(0, 0.5);
        assert!(o.remove_point(1));
        assert_eq!((o.points.len(), o.handles.len()), (2, 2));
        assert!(!o.remove_point(0), "two vertices are the least a path can have");
        assert!(!o.remove_point(9));
        // Taking the only curved vertex away leaves plain corners.
        let mut o = rect(1, 0.0, 0.0, 10.0, 10.0);
        o.to_path();
        o.set_handles(1, Handles::smooth(v2(3.0, 0.0)));
        assert!(o.remove_point(1));
        assert!(o.handles.is_empty());
        assert_eq!(o.points.len(), 3);
    }

    #[test]
    fn handles_are_dropped_once_every_vertex_is_a_corner() {
        let mut o = arch();
        o.set_handles(0, Handles::default());
        assert!(o.is_curved());
        o.set_handles(1, Handles::default());
        assert!(o.handles.is_empty());
        o.set_handles(1, Handles::smooth(v2(10.0, 0.0)));
        assert_eq!(o.handles.len(), 2);
        assert_eq!(o.handles_at(1).before, v2(-10.0, 0.0));
        assert_eq!(o.handles_at(5), Handles::default());
    }

    #[test]
    fn flattening_keeps_the_shape_and_drops_the_handles() {
        let mut o = arch();
        let outline = o.outline();
        o.flatten();
        assert!(o.handles.is_empty());
        assert_eq!(o.points, outline);
        assert_eq!(o.outline(), outline);
    }

    #[test]
    fn curves_survive_saving_and_old_files_still_open() {
        let mut e = Editor::new(Document::new(100, 100));
        e.add_object(arch());
        let json = document_to_json(&e.doc).unwrap();
        assert_eq!(document_from_json(&json).unwrap(), e.doc);

        // A straight path writes no "handles" at all, as before curves existed.
        let mut straight = arch();
        straight.handles.clear();
        let mut e = Editor::new(Document::new(100, 100));
        e.add_object(straight);
        let json = document_to_json(&e.doc).unwrap();
        assert!(!json.contains("handles"));
        assert_eq!(document_from_json(&json).unwrap(), e.doc);
    }
}
