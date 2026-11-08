//! Document model and editing operations. Free of UI types so it can be unit tested.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Rect { radius: f32 },
    Ellipse,
    Line,
    Path { closed: bool },
    Text { content: String, size: f32 },
    /// Pixels are shared copy-on-write, so undo snapshots stay cheap.
    Image { bitmap: Arc<Bitmap> },
    /// A web area drawn over the artwork and left out of flattened images.
    Web { kind: WebKind, url: String },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebKind {
    Hotspot,
    Slice,
}

/// How a stroke changes the pixels under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelStroke {
    Paint(Color),
    Erase,
    Blur,
    /// Copies pixels from this document-space offset.
    Clone(Vec2),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: u64,
    pub name: String,
    pub shape: Shape,
    pub bounds: Rect,
    /// Absolute vertices; only used by `Line` and `Path`.
    pub points: Vec<Vec2>,
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
                WebKind::Hotspot => "Hotspot".into(),
                WebKind::Slice => "Slice".into(),
            },
            bounds,
            shape: Shape::Web { kind, url: String::new() },
            points: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
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
        let sx = if natural.w > 0.0 { self.bounds.w / natural.w } else { 1.0 };
        let sy = if natural.h > 0.0 { self.bounds.h / natural.h } else { 1.0 };
        objects
            .iter()
            .map(|part| {
                let mut part = part.clone();
                let b = part.bounds;
                part.set_bounds(Rect {
                    x: self.bounds.x + (b.x - natural.x) * sx,
                    y: self.bounds.y + (b.y - natural.y) * sy,
                    w: b.w * sx,
                    h: b.h * sy,
                });
                part.opacity *= self.opacity;
                if let Shape::Text { size, .. } = &mut part.shape {
                    *size *= sy;
                }
                if let Some(stroke) = &mut part.stroke {
                    stroke.width *= (sx + sy) / 2.0;
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
    pub fn to_path(&mut self) -> bool {
        if !self.reshapable() {
            return false;
        }
        let closed = self.is_closed();
        self.points = self.outline();
        self.shape = Shape::Path { closed };
        true
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
    fn pixel_scale(&self) -> Option<(f32, f32)> {
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
        });
    }

    /// Flattened outline used by canvas drawing, PNG export and hit testing.
    pub fn outline(&self) -> Vec<Vec2> {
        let b = self.bounds;
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
            Shape::Line | Shape::Path { .. } => self.points.clone(),
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
        self.bounds.x += dx;
        self.bounds.y += dy;
        for p in &mut self.points {
            p.x += dx;
            p.y += dy;
        }
    }

    /// Moves or scales the object into `target`, carrying its vertices along.
    pub fn set_bounds(&mut self, target: Rect) {
        let old = self.bounds;
        let sx = if old.w > 0.0 { target.w / old.w } else { 1.0 };
        let sy = if old.h > 0.0 { target.h / old.h } else { 1.0 };
        for p in &mut self.points {
            p.x = target.x + (p.x - old.x) * sx;
            p.y = target.y + (p.y - old.y) * sy;
        }
        self.bounds = target;
    }

    pub fn flip(&mut self, horizontal: bool) {
        if matches!(self.shape, Shape::Auto { .. }) {
            self.to_path();
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
    }

    /// Turns the object a quarter turn around `center`. Returns false if it cannot turn.
    pub fn rotate90(&mut self, clockwise: bool, center: Vec2) -> bool {
        let turn = |p: Vec2| {
            let (dx, dy) = (p.x - center.x, p.y - center.y);
            if clockwise { v2(center.x - dy, center.y + dx) } else { v2(center.x + dy, center.y - dx) }
        };
        match self.shape {
            Shape::Text { .. } | Shape::Instance { .. } => return false,
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
        true
    }

    /// Recomputes `bounds` from the vertices after a vertex edit.
    pub fn refit_to_points(&mut self) {
        if !self.points.is_empty() {
            self.bounds = Rect::of_points(&self.points);
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
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), delay_ms: DEFAULT_STATE_DELAY, exported: true, objects: Vec::new() }
    }
}

fn default_states() -> Vec<State> {
    vec![State::new("State 1")]
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
}

impl Page {
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            canvas: Some(Color::WHITE),
            layers: vec![Layer::new("Layer 1")],
            active_layer: 0,
            states: default_states(),
            active_state: 0,
            export: ExportSettings::default(),
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
        let mut state = State::new(format!("State {}", self.states.len() + 1));
        state.objects = vec![Vec::new(); self.layers.len()];
        self.insert_state(state);
    }

    /// Copies the current state after itself and moves to the copy.
    pub fn duplicate_state(&mut self) {
        let mut state = State::new(format!("State {}", self.states.len() + 1));
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
}

pub fn default_styles() -> Vec<Style> {
    let style = |name: &str, fill: Option<Color>, stroke: Option<(Color, f32)>, opacity: f32| Style {
        name: name.into(),
        fill,
        stroke: stroke.map(|(color, width)| Stroke { color, width }),
        opacity,
    };
    let ink = Color::rgb(0x22, 0x22, 0x22);
    let blue = Color::rgb(0x1e, 0x78, 0xd2);
    let red = Color::rgb(0xd2, 0x3c, 0x32);
    let green = Color::rgb(0x2e, 0xa0, 0x5a);
    let amber = Color::rgb(0xf0, 0xa8, 0x20);
    let gray = Color::rgb(0xcc, 0xcc, 0xcc);
    vec![
        style("Solid Ink", Some(ink), None, 1.0),
        style("Solid Blue", Some(blue), None, 1.0),
        style("Solid Red", Some(red), None, 1.0),
        style("Solid Green", Some(green), None, 1.0),
        style("Solid Amber", Some(amber), None, 1.0),
        style("Paper", Some(Color::WHITE), Some((gray, 1.0)), 1.0),
        style("Outline Ink", None, Some((ink, 2.0)), 1.0),
        style("Outline Blue", None, Some((blue, 2.0)), 1.0),
        style("Heavy Outline", None, Some((ink, 6.0)), 1.0),
        style("Tint Blue", Some(blue), None, 0.35),
        style("Tint Red", Some(red), None, 0.35),
        style("Blue Panel", Some(Color::rgb(0xdc, 0xeb, 0xfa)), Some((blue, 1.0)), 1.0),
    ]
}

/// A reusable group of objects, stored relative to the top-left of its content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub id: u64,
    pub name: String,
    pub objects: Vec<Object>,
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
            pages: vec![Page::new("Page 1", width, height)],
            active_page: 0,
            next_id: 1,
            styles: default_styles(),
            symbols: Vec::new(),
            urls: Vec::new(),
        }
    }

    /// A document sized to an image, with the image as its only object.
    pub fn from_bitmap(name: impl Into<String>, bitmap: Bitmap) -> Self {
        let mut doc = Self::new(bitmap.width(), bitmap.height());
        doc.pages[0].canvas = None;
        doc.pages[0].layers[0].name = "Background".into();
        doc.pages[0].layers[0].objects.push(Object::new_bitmap(1, name, bitmap, 0.0, 0.0));
        doc.pages[0].layers.push(Layer::new("Layer 1"));
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
    let file: FileEnvelope = anyhow::Context::context(serde_json::from_str(text), "not an Emberframe document")?;
    anyhow::ensure!(file.format == FILE_FORMAT, "not an Emberframe document");
    anyhow::ensure!(file.version <= FILE_VERSION, "file was saved by a newer version");
    anyhow::ensure!(!file.document.pages.is_empty(), "document has no pages");
    let mut doc = file.document;
    doc.active_page = doc.active_page.min(doc.pages.len() - 1);
    for page in &mut doc.pages {
        anyhow::ensure!(!page.layers.is_empty(), "page has no layers");
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

const HISTORY_LIMIT: usize = 200;

/// A document plus its selection, undo history and clipboard.
pub struct Editor {
    pub doc: Document,
    pub selection: Vec<u64>,
    pub history: Vec<(String, Document)>,
    pub history_pos: usize,
    /// Changes whenever the document's content or the view of it changes.
    pub revision: u64,
    saved_doc: Document,
    clipboard: Vec<Object>,
}

impl Editor {
    pub fn new(doc: Document) -> Self {
        Self {
            history: vec![("New Document".into(), doc.clone())],
            history_pos: 0,
            revision: 0,
            saved_doc: doc.clone(),
            selection: Vec::new(),
            clipboard: Vec::new(),
            doc,
        }
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
        if self.history.len() > HISTORY_LIMIT {
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
                    piece.points = points;
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
        let name = format!("Layer {}", page.layers.len() + 1);
        let at = page.active_layer + 1;
        page.layers.insert(at, Layer::new(name));
        page.active_layer = at;
        for state in page.inactive_states_mut() {
            state.objects.insert(at, Vec::new());
        }
    }

    /// Removes the active layer. The last remaining layer is kept.
    pub fn delete_layer(&mut self) -> bool {
        let page = self.doc.page_mut();
        if page.layers.len() < 2 {
            return false;
        }
        let at = page.active_layer;
        page.layers.remove(at);
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
        let mut page = Page::new(format!("Page {}", self.doc.pages.len() + 1), w, h);
        page.canvas = canvas;
        self.doc.pages.push(page);
        self.doc.active_page = self.doc.pages.len() - 1;
        self.selection.clear();
    }

    pub fn delete_page(&mut self) -> bool {
        if self.doc.pages.len() < 2 {
            return false;
        }
        self.doc.pages.remove(self.doc.active_page);
        self.doc.active_page = self.doc.active_page.min(self.doc.pages.len() - 1);
        self.selection.clear();
        true
    }

    pub fn set_active_page(&mut self, index: usize) {
        if index < self.doc.pages.len() {
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
            self.navigate(|doc| doc.page_mut().goto_state(index));
            self.selection.clear();
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
                o.stroke = style.stroke;
                o.opacity = style.opacity;
            }
        });
    }

    /// Saves the first selected vector object's look as a new style.
    pub fn add_style_from_selection(&mut self) -> bool {
        let Some(source) = self.selected().into_iter().find(|o| o.takes_color()).cloned() else { return false };
        let name = format!("Style {}", self.doc.styles.len() + 1);
        self.doc.styles.push(Style { name, fill: source.fill, stroke: source.stroke, opacity: source.opacity });
        true
    }

    /// Adds a symbol to the library, or finds the one that already has this name.
    pub fn add_library_symbol(&mut self, name: &str, objects: &[Object]) -> u64 {
        if let Some(existing) = self.doc.symbols.iter().find(|s| s.name == name) {
            return existing.id;
        }
        let id = self.alloc_id();
        self.doc.symbols.push(Symbol { id, name: name.to_string(), objects: normalized(objects) });
        id
    }

    fn new_instance(&mut self, symbol: u64, bounds: Rect) -> Object {
        let name = self.doc.symbols.iter().find(|s| s.id == symbol).map_or("Symbol".to_string(), |s| s.name.clone());
        let objects = resolve_symbol(&self.doc.symbols, symbol, 0);
        Object {
            id: self.alloc_id(),
            name,
            shape: Shape::Instance { symbol, objects },
            bounds,
            points: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    /// Places a symbol at its natural size, centered on the canvas.
    pub fn place_symbol(&mut self, symbol: u64) -> bool {
        let center = self.doc.page().rect().center();
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
        let name = format!("Symbol {}", self.doc.symbols.len() + 1);
        self.doc.symbols.push(Symbol { id, name, objects: normalized(&objects) });
        let instance = self.new_instance(id, bounds);
        self.replace_selected(vec![instance]);
        Some(id)
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
    pub fn group_selected(&mut self) -> bool {
        let objects: Vec<Object> = self.doc.page().objects().filter(|o| self.is_selected(o.id)).cloned().collect();
        let Some(bounds) = objects.iter().map(|o| o.bounds).reduce(Rect::union).filter(|_| objects.len() >= 2) else {
            return false;
        };
        let group = Object {
            id: self.alloc_id(),
            name: "Group".into(),
            shape: Shape::Group { objects: Arc::new(objects) },
            bounds,
            points: Vec::new(),
            fill: None,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        };
        self.replace_selected(vec![group]);
        true
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
    pub fn delete_symbol(&mut self, id: u64) {
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
    fn text_does_not_rotate() {
        let mut text = rect(1, 0.0, 0.0, 10.0, 10.0);
        text.shape = Shape::Text { content: "a".into(), size: 12.0 };
        assert!(!text.rotate90(true, v2(0.0, 0.0)));
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
}
