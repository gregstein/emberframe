//! Straight-alpha RGBA images and the pixel operations applied to them.

use std::fmt;
use std::io::Cursor;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::{ImageFormat, RgbaImage};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::model::{v2, Color, Vec2};

pub const MAX_DIMENSION: u32 = 16000;
pub const FILL_TOLERANCE: u8 = 32;
const BRIGHTNESS_STEP: i32 = 25;
const BLUR_SIGMA: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Invert,
    Grayscale,
    Brighten,
    Darken,
    Blur,
    Sharpen,
}

impl Filter {
    pub fn label(self) -> &'static str {
        match self {
            Filter::Invert => "Invert",
            Filter::Grayscale => "Grayscale",
            Filter::Brighten => "Brighten",
            Filter::Darken => "Darken",
            Filter::Blur => "Blur",
            Filter::Sharpen => "Sharpen",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskMode {
    Replace,
    Add,
    Subtract,
}

/// A pixel selection, one cell per bitmap pixel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mask {
    width: u32,
    height: u32,
    cells: Vec<bool>,
}

impl Mask {
    pub fn empty(width: u32, height: u32) -> Self {
        Self { width, height, cells: vec![false; width as usize * height as usize] }
    }

    /// Selects the pixels between two corners, clamped to the image.
    pub fn rect(width: u32, height: u32, a: Vec2, b: Vec2) -> Self {
        let mut mask = Self::empty(width, height);
        let clamp = |v: f32, max: u32| v.round().clamp(0.0, max as f32) as u32;
        let (x0, x1) = (clamp(a.x.min(b.x), width), clamp(a.x.max(b.x), width));
        let (y0, y1) = (clamp(a.y.min(b.y), height), clamp(a.y.max(b.y), height));
        for y in y0..y1 {
            let start = y as usize * width as usize;
            mask.cells[start + x0 as usize..start + x1 as usize].fill(true);
        }
        mask
    }

    /// Selects the inside of a polygon given in pixel coordinates.
    pub fn polygon(width: u32, height: u32, points: &[Vec2]) -> Self {
        let mut mask = Self::empty(width, height);
        let (Some(first), true) = (points.first(), points.len() >= 3) else { return mask };
        let mut builder = PathBuilder::new();
        builder.move_to(first.x, first.y);
        for p in &points[1..] {
            builder.line_to(p.x, p.y);
        }
        builder.close();
        let (Some(path), Some(mut layer)) = (builder.finish(), Pixmap::new(width, height)) else { return mask };
        let mut paint = Paint::default();
        paint.anti_alias = false;
        paint.set_color_rgba8(255, 255, 255, 255);
        layer.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
        for (cell, px) in mask.cells.iter_mut().zip(layer.pixels()) {
            *cell = px.alpha() > 0;
        }
        mask
    }

    pub fn fits(&self, bitmap: &Bitmap) -> bool {
        self.width == bitmap.width && self.height == bitmap.height
    }

    pub fn is_empty(&self) -> bool {
        !self.cells.contains(&true)
    }

    fn get(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && x < self.width as i64
            && y < self.height as i64
            && self.cells[y as usize * self.width as usize + x as usize]
    }

    pub fn is_set(&self, x: u32, y: u32) -> bool {
        self.get(x as i64, y as i64)
    }

    /// Merges `other` in. Masks of different sizes are ignored.
    pub fn combine(&mut self, other: &Mask, mode: MaskMode) {
        if (self.width, self.height) != (other.width, other.height) {
            return;
        }
        for (cell, new) in self.cells.iter_mut().zip(&other.cells) {
            *cell = match mode {
                MaskMode::Replace => *new,
                MaskMode::Add => *cell || *new,
                MaskMode::Subtract => *cell && !*new,
            };
        }
    }

    /// Returns x, y, width and height of the smallest box around the selection.
    fn bounds(&self) -> Option<(u32, u32, u32, u32)> {
        let w = self.width as usize;
        let first = self.cells.iter().position(|c| *c)?;
        let last = self.cells.iter().rposition(|c| *c)?;
        let (mut left, mut right) = (u32::MAX, 0);
        for row in self.cells.chunks_exact(w) {
            if let (Some(a), Some(b)) = (row.iter().position(|c| *c), row.iter().rposition(|c| *c)) {
                left = left.min(a as u32);
                right = right.max(b as u32);
            }
        }
        let (top, bottom) = ((first / w) as u32, (last / w) as u32);
        Some((left, top, right - left + 1, bottom - top + 1))
    }

    /// The selection outline as axis-aligned segments in pixel coordinates.
    pub fn edges(&self) -> Vec<(Vec2, Vec2)> {
        let (w, h) = (self.width as i64, self.height as i64);
        let mut edges = Vec::new();
        for y in 0..=h {
            let mut run = None;
            for x in 0..=w {
                let boundary = x < w && self.get(x, y - 1) != self.get(x, y);
                match (boundary, run) {
                    (true, None) => run = Some(x),
                    (false, Some(start)) => {
                        edges.push((v2(start as f32, y as f32), v2(x as f32, y as f32)));
                        run = None;
                    }
                    _ => {}
                }
            }
        }
        for x in 0..=w {
            let mut run = None;
            for y in 0..=h {
                let boundary = y < h && self.get(x - 1, y) != self.get(x, y);
                match (boundary, run) {
                    (true, None) => run = Some(y),
                    (false, Some(start)) => {
                        edges.push((v2(x as f32, start as f32), v2(x as f32, y as f32)));
                        run = None;
                    }
                    _ => {}
                }
            }
        }
        edges
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Bitmap {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bitmap({}x{})", self.width, self.height)
    }
}

impl Bitmap {
    /// Returns `None` when `rgba` does not hold exactly `width * height` pixels.
    pub fn from_rgba(width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        let expected = width as usize * height as usize * 4;
        (width > 0 && height > 0 && rgba.len() == expected).then_some(Self { width, height, rgba })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let decoded = image::load_from_memory(bytes).context("not a supported image")?.to_rgba8();
        let (width, height) = decoded.dimensions();
        anyhow::ensure!(
            width <= MAX_DIMENSION && height <= MAX_DIMENSION,
            "image is larger than {MAX_DIMENSION} pixels on a side"
        );
        Self::from_rgba(width, height, decoded.into_raw()).ok_or_else(|| anyhow!("image has no pixels"))
    }

    pub fn encode_png(&self) -> Result<Vec<u8>> {
        let mut out = Cursor::new(Vec::new());
        self.to_image().write_to(&mut out, ImageFormat::Png)?;
        Ok(out.into_inner())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn to_image(&self) -> RgbaImage {
        RgbaImage::from_raw(self.width, self.height, self.rgba.clone()).expect("buffer size is an invariant")
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width as usize + x as usize) * 4
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = self.index(x, y);
        Some([self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]])
    }

    /// Rasterizes a round stroke and returns its offset and coverage layer.
    fn coverage(&self, points: &[Vec2], width: f32) -> Option<(u32, u32, Pixmap)> {
        let first = *points.first()?;
        let pad = width / 2.0 + 2.0;
        let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
        for p in points {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        let left = (x0 - pad).floor().max(0.0) as u32;
        let top = (y0 - pad).floor().max(0.0) as u32;
        let right = ((x1 + pad).ceil().max(0.0) as u32).min(self.width);
        let bottom = ((y1 + pad).ceil().max(0.0) as u32).min(self.height);
        if left >= right || top >= bottom {
            return None;
        }
        let mut layer = Pixmap::new(right - left, bottom - top)?;
        let mut builder = PathBuilder::new();
        let (ox, oy) = (left as f32, top as f32);
        builder.move_to(first.x - ox, first.y - oy);
        for p in &points[1..] {
            builder.line_to(p.x - ox, p.y - oy);
        }
        if points.len() == 1 {
            builder.line_to(first.x - ox + 0.01, first.y - oy);
        }
        let path = builder.finish()?;
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color_rgba8(255, 255, 255, 255);
        let style =
            Stroke { width: width.max(1.0), line_cap: LineCap::Round, line_join: LineJoin::Round, ..Stroke::default() };
        layer.stroke_path(&path, &paint, &style, Transform::identity(), None);
        Some((left, top, layer))
    }

    fn apply_layer(&mut self, left: u32, top: u32, layer: &Pixmap, mut apply: impl FnMut(u32, u32, &mut [u8], f32)) {
        let columns = layer.width();
        for (n, px) in layer.pixels().iter().enumerate() {
            if px.alpha() == 0 {
                continue;
            }
            let x = left + n as u32 % columns;
            let y = top + n as u32 / columns;
            let i = self.index(x, y);
            apply(x, y, &mut self.rgba[i..i + 4], px.alpha() as f32 / 255.0);
        }
    }

    fn for_coverage(&mut self, points: &[Vec2], width: f32, apply: impl FnMut(u32, u32, &mut [u8], f32)) {
        if let Some((left, top, layer)) = self.coverage(points, width) {
            self.apply_layer(left, top, &layer, apply);
        }
    }

    /// Softens the pixels under a round stroke. Points are in pixels.
    pub fn blur_stroke(&mut self, points: &[Vec2], width: f32) {
        let Some((left, top, layer)) = self.coverage(points, width) else { return };
        let pad = (BLUR_SIGMA * 3.0).ceil() as u32;
        let (x0, y0) = (left.saturating_sub(pad), top.saturating_sub(pad));
        let x1 = (left + layer.width() + pad).min(self.width);
        let y1 = (top + layer.height() + pad).min(self.height);
        let source = self.to_image();
        let region = image::imageops::crop_imm(&source, x0, y0, x1 - x0, y1 - y0).to_image();
        let blurred = image::imageops::blur(&region, BLUR_SIGMA);
        self.apply_layer(left, top, &layer, |x, y, dst, coverage| {
            mix(dst, &blurred.get_pixel(x - x0, y - y0).0, coverage);
        });
    }

    /// Copies pixels found at `offset` from each pixel under a round stroke.
    pub fn clone_stroke(&mut self, points: &[Vec2], width: f32, offset: Vec2) {
        let source = self.clone();
        self.for_coverage(points, width, |x, y, dst, coverage| {
            let sx = x as i64 + offset.x.round() as i64;
            let sy = y as i64 + offset.y.round() as i64;
            if let (Ok(sx), Ok(sy)) = (u32::try_from(sx), u32::try_from(sy)) {
                if let Some(px) = source.pixel(sx, sy) {
                    mix(dst, &px, coverage);
                }
            }
        });
    }

    /// Paints a round stroke with source-over blending. Points are in pixels.
    pub fn paint_stroke(&mut self, points: &[Vec2], width: f32, color: Color) {
        self.for_coverage(points, width, |_, _, dst, coverage| {
            let src_a = coverage * color.a as f32 / 255.0;
            let dst_a = dst[3] as f32 / 255.0;
            let out_a = src_a + dst_a * (1.0 - src_a);
            if out_a <= 0.0 {
                return;
            }
            for (channel, src) in dst.iter_mut().zip([color.r, color.g, color.b]) {
                let blended = (src as f32 * src_a + *channel as f32 * dst_a * (1.0 - src_a)) / out_a;
                *channel = blended.round().clamp(0.0, 255.0) as u8;
            }
            dst[3] = (out_a * 255.0).round() as u8;
        });
    }

    /// Reduces alpha along a round stroke. Points are in pixels.
    pub fn erase_stroke(&mut self, points: &[Vec2], width: f32) {
        self.for_coverage(points, width, |_, _, dst, coverage| {
            dst[3] = (dst[3] as f32 * (1.0 - coverage)).round() as u8;
        });
    }

    /// Finds the 4-connected region of similar pixels around a pixel.
    pub fn region(&self, x: u32, y: u32, tolerance: u8) -> Option<Mask> {
        let start = self.pixel(x, y)?;
        let matches = |px: &[u8]| px.iter().zip(start).all(|(a, b)| a.abs_diff(b) <= tolerance);
        let (w, h) = (self.width as usize, self.height as usize);
        let mut mask = Mask::empty(self.width, self.height);
        let mut stack = vec![(x as usize, y as usize)];
        mask.cells[y as usize * w + x as usize] = true;
        while let Some((cx, cy)) = stack.pop() {
            let mut visit = |nx: usize, ny: usize| {
                let cell = ny * w + nx;
                if !mask.cells[cell] && matches(&self.rgba[cell * 4..cell * 4 + 4]) {
                    mask.cells[cell] = true;
                    stack.push((nx, ny));
                }
            };
            if cx > 0 {
                visit(cx - 1, cy);
            }
            if cx + 1 < w {
                visit(cx + 1, cy);
            }
            if cy > 0 {
                visit(cx, cy - 1);
            }
            if cy + 1 < h {
                visit(cx, cy + 1);
            }
        }
        Some(mask)
    }

    /// Fills the region around a pixel. Returns false if out of range.
    pub fn flood_fill(&mut self, x: u32, y: u32, color: Color, tolerance: u8) -> bool {
        let Some(mask) = self.region(x, y, tolerance) else { return false };
        let replacement = [color.r, color.g, color.b, color.a];
        for (px, _) in self.rgba.chunks_exact_mut(4).zip(&mask.cells).filter(|(_, set)| **set) {
            px.copy_from_slice(&replacement);
        }
        true
    }

    /// Puts back every pixel outside `mask`, confining an edit to the selection.
    pub fn restore_outside(&mut self, before: &Bitmap, mask: &Mask) {
        if !mask.fits(self) || before.rgba.len() != self.rgba.len() {
            return;
        }
        let cells = self.rgba.chunks_exact_mut(4).zip(before.rgba.chunks_exact(4)).zip(&mask.cells);
        for ((px, old), _) in cells.filter(|(_, set)| !**set) {
            px.copy_from_slice(old);
        }
    }

    /// Makes the selected pixels fully transparent.
    pub fn clear(&mut self, mask: &Mask) {
        if mask.fits(self) {
            for (px, _) in self.rgba.chunks_exact_mut(4).zip(&mask.cells).filter(|(_, set)| **set) {
                px.fill(0);
            }
        }
    }

    /// Copies the selected pixels into a new bitmap, returned with its position.
    pub fn extract(&self, mask: &Mask) -> Option<(Bitmap, u32, u32)> {
        if !mask.fits(self) {
            return None;
        }
        let (left, top, w, h) = mask.bounds()?;
        let mut rgba = vec![0; w as usize * h as usize * 4];
        for (n, px) in rgba.chunks_exact_mut(4).enumerate() {
            let (x, y) = (left + n as u32 % w, top + n as u32 / w);
            if mask.is_set(x, y) {
                let i = self.index(x, y);
                px.copy_from_slice(&self.rgba[i..i + 4]);
            }
        }
        Some((Bitmap::from_rgba(w, h, rgba)?, left, top))
    }

    pub fn flip(&mut self, horizontal: bool) {
        let row = self.width as usize * 4;
        if horizontal {
            for line in self.rgba.chunks_exact_mut(row) {
                let (mut a, mut b) = (0, row - 4);
                while a < b {
                    for k in 0..4 {
                        line.swap(a + k, b + k);
                    }
                    a += 4;
                    b -= 4;
                }
            }
        } else {
            let mut flipped = Vec::with_capacity(self.rgba.len());
            for line in self.rgba.chunks_exact(row).rev() {
                flipped.extend_from_slice(line);
            }
            self.rgba = flipped;
        }
    }

    /// Turns the pixels a quarter turn, swapping width and height.
    pub fn rotate90(&mut self, clockwise: bool) {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut turned = vec![0; self.rgba.len()];
        for y in 0..h {
            for x in 0..w {
                let (nx, ny) = if clockwise { (h - 1 - y, x) } else { (y, w - 1 - x) };
                let from = (y * w + x) * 4;
                let to = (ny * h + nx) * 4;
                turned[to..to + 4].copy_from_slice(&self.rgba[from..from + 4]);
            }
        }
        self.rgba = turned;
        std::mem::swap(&mut self.width, &mut self.height);
    }

    pub fn apply_filter(&mut self, filter: Filter) {
        match filter {
            Filter::Invert => self.map_rgb(|c| 255 - c),
            Filter::Brighten => self.map_rgb(|c| (c as i32 + BRIGHTNESS_STEP).min(255) as u8),
            Filter::Darken => self.map_rgb(|c| (c as i32 - BRIGHTNESS_STEP).max(0) as u8),
            Filter::Grayscale => {
                for px in self.rgba.chunks_exact_mut(4) {
                    let luma = (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32).round() as u8;
                    px[..3].fill(luma);
                }
            }
            Filter::Blur => self.rgba = image::imageops::blur(&self.to_image(), BLUR_SIGMA).into_raw(),
            Filter::Sharpen => self.rgba = image::imageops::unsharpen(&self.to_image(), 1.5, 4).into_raw(),
        }
    }

    fn map_rgb(&mut self, f: impl Fn(u8) -> u8) {
        for px in self.rgba.chunks_exact_mut(4) {
            for channel in &mut px[..3] {
                *channel = f(*channel);
            }
        }
    }
}

fn mix(dst: &mut [u8], src: &[u8; 4], amount: f32) {
    for (d, s) in dst.iter_mut().zip(src) {
        *d = (*d as f32 + (*s as f32 - *d as f32) * amount).round() as u8;
    }
}

impl Serialize for Bitmap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let png = self.encode_png().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&STANDARD.encode(png))
    }
}

impl<'de> Deserialize<'de> for Bitmap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        let png = STANDARD.decode(text).map_err(serde::de::Error::custom)?;
        Bitmap::decode(&png).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_mask_clamps_and_outlines_with_four_edges() {
        let mask = Mask::rect(10, 10, v2(-5.0, 2.0), v2(4.0, 6.0));
        assert!(mask.is_set(0, 2) && mask.is_set(3, 5));
        assert!(!mask.is_set(4, 5) && !mask.is_set(0, 6));
        assert_eq!(mask.edges().len(), 4);
        assert_eq!(mask.bounds(), Some((0, 2, 4, 4)));
    }

    #[test]
    fn polygon_mask_selects_inside() {
        let mask = Mask::polygon(20, 20, &[v2(2.0, 2.0), v2(18.0, 2.0), v2(2.0, 18.0)]);
        assert!(mask.is_set(4, 4));
        assert!(!mask.is_set(17, 17));
        assert!(Mask::polygon(20, 20, &[v2(1.0, 1.0), v2(2.0, 2.0)]).is_empty());
    }

    #[test]
    fn masks_add_and_subtract() {
        let mut mask = Mask::rect(10, 1, v2(0.0, 0.0), v2(4.0, 1.0));
        mask.combine(&Mask::rect(10, 1, v2(6.0, 0.0), v2(8.0, 1.0)), MaskMode::Add);
        assert!(mask.is_set(1, 0) && mask.is_set(7, 0));
        mask.combine(&Mask::rect(10, 1, v2(0.0, 0.0), v2(2.0, 1.0)), MaskMode::Subtract);
        assert!(!mask.is_set(1, 0) && mask.is_set(2, 0));
    }

    #[test]
    fn region_selects_only_connected_similar_pixels() {
        let mut b = solid(5, 1, [255, 255, 255, 255]);
        b.rgba[2 * 4..2 * 4 + 3].fill(0);
        let mask = b.region(0, 0, FILL_TOLERANCE).unwrap();
        assert!(mask.is_set(1, 0) && !mask.is_set(2, 0) && !mask.is_set(3, 0));
    }

    #[test]
    fn edit_can_be_confined_to_a_mask() {
        let before = solid(4, 1, [1, 1, 1, 255]);
        let mut b = before.clone();
        b.apply_filter(Filter::Invert);
        b.restore_outside(&before, &Mask::rect(4, 1, v2(0.0, 0.0), v2(2.0, 1.0)));
        assert_eq!(b.pixel(1, 0), Some([254, 254, 254, 255]));
        assert_eq!(b.pixel(2, 0), Some([1, 1, 1, 255]));
    }

    #[test]
    fn extract_crops_and_clears_unselected_corners() {
        let b = solid(10, 10, [9, 9, 9, 255]);
        let mut mask = Mask::rect(10, 10, v2(2.0, 3.0), v2(5.0, 6.0));
        mask.combine(&Mask::rect(10, 10, v2(2.0, 3.0), v2(3.0, 4.0)), MaskMode::Subtract);
        let (piece, x, y) = b.extract(&mask).unwrap();
        assert_eq!((piece.width(), piece.height(), x, y), (3, 3, 2, 3));
        assert_eq!(piece.pixel(0, 0), Some([0, 0, 0, 0]));
        assert_eq!(piece.pixel(1, 1), Some([9, 9, 9, 255]));
        assert!(b.extract(&Mask::empty(10, 10)).is_none());
    }

    #[test]
    fn clear_removes_selected_pixels_only() {
        let mut b = solid(4, 1, [5, 5, 5, 255]);
        b.clear(&Mask::rect(4, 1, v2(0.0, 0.0), v2(1.0, 1.0)));
        assert_eq!(b.pixel(0, 0), Some([0, 0, 0, 0]));
        assert_eq!(b.pixel(1, 0), Some([5, 5, 5, 255]));
    }

    #[test]
    fn blur_stroke_softens_an_edge_under_the_stroke_only() {
        let mut b = solid(40, 40, [255, 255, 255, 255]);
        for y in 0..40 {
            for x in 20..40 {
                let i = b.index(x, y);
                b.rgba[i..i + 3].fill(0);
            }
        }
        b.blur_stroke(&[v2(20.0, 5.0), v2(20.0, 15.0)], 8.0);
        let near = b.pixel(20, 10).unwrap()[0];
        assert!(near > 0 && near < 255, "edge should be softened, got {near}");
        assert_eq!(b.pixel(20, 35).unwrap()[0], 0);
    }

    #[test]
    fn clone_stroke_copies_from_offset_and_ignores_outside() {
        let mut b = solid(30, 30, [255, 255, 255, 255]);
        b.paint_stroke(&[v2(5.0, 5.0)], 6.0, RED);
        b.clone_stroke(&[v2(20.0, 20.0)], 4.0, v2(-15.0, -15.0));
        assert_eq!(b.pixel(20, 20), Some([255, 0, 0, 255]));
        let before = b.pixel(2, 25);
        b.clone_stroke(&[v2(2.0, 25.0)], 2.0, v2(-50.0, 0.0));
        assert_eq!(b.pixel(2, 25), before);
    }

    fn solid(w: u32, h: u32, px: [u8; 4]) -> Bitmap {
        Bitmap::from_rgba(w, h, px.repeat((w * h) as usize)).unwrap()
    }

    const RED: Color = Color::rgb(255, 0, 0);

    #[test]
    fn rejects_mismatched_buffer() {
        assert!(Bitmap::from_rgba(2, 2, vec![0; 15]).is_none());
        assert!(Bitmap::from_rgba(0, 2, vec![]).is_none());
    }

    #[test]
    fn png_round_trip_preserves_pixels() {
        let mut b = solid(3, 2, [10, 20, 30, 200]);
        b.rgba[0] = 99;
        assert_eq!(Bitmap::decode(&b.encode_png().unwrap()).unwrap(), b);
    }

    #[test]
    fn decode_rejects_non_image_data() {
        assert!(Bitmap::decode(b"definitely not an image").is_err());
    }

    #[test]
    fn serde_round_trip_uses_string() {
        let b = solid(2, 2, [1, 2, 3, 4]);
        let json = serde_json::to_string(&b).unwrap();
        assert!(json.starts_with('"'));
        assert_eq!(serde_json::from_str::<Bitmap>(&json).unwrap(), b);
    }

    #[test]
    fn paint_covers_stroke_and_leaves_rest() {
        let mut b = solid(20, 20, [255, 255, 255, 255]);
        b.paint_stroke(&[v2(2.0, 10.0), v2(18.0, 10.0)], 4.0, RED);
        assert_eq!(b.pixel(10, 10), Some([255, 0, 0, 255]));
        assert_eq!(b.pixel(10, 2), Some([255, 255, 255, 255]));
    }

    #[test]
    fn single_point_paints_a_dot() {
        let mut b = solid(20, 20, [255, 255, 255, 255]);
        b.paint_stroke(&[v2(10.0, 10.0)], 6.0, RED);
        assert_eq!(b.pixel(10, 10), Some([255, 0, 0, 255]));
    }

    #[test]
    fn painting_on_transparent_keeps_true_color() {
        let mut b = solid(20, 20, [0, 0, 0, 0]);
        b.paint_stroke(&[v2(2.0, 10.0), v2(18.0, 10.0)], 4.0, RED);
        assert_eq!(b.pixel(10, 10), Some([255, 0, 0, 255]));
    }

    #[test]
    fn stroke_outside_or_partly_outside_is_safe() {
        let mut b = solid(10, 10, [255, 255, 255, 255]);
        let before = b.clone();
        b.paint_stroke(&[v2(-50.0, -50.0), v2(-40.0, -40.0)], 3.0, RED);
        b.paint_stroke(&[v2(100.0, 100.0)], 3.0, RED);
        assert_eq!(b, before);
        b.paint_stroke(&[v2(-5.0, 5.0), v2(5.0, 5.0)], 3.0, RED);
        assert_eq!(b.pixel(2, 5), Some([255, 0, 0, 255]));
    }

    #[test]
    fn erase_clears_alpha_only_on_stroke() {
        let mut b = solid(20, 20, [9, 8, 7, 255]);
        b.erase_stroke(&[v2(2.0, 10.0), v2(18.0, 10.0)], 4.0);
        assert_eq!(b.pixel(10, 10).unwrap()[3], 0);
        assert_eq!(b.pixel(10, 2), Some([9, 8, 7, 255]));
    }

    #[test]
    fn flood_fill_stops_at_boundary() {
        let mut b = solid(5, 1, [255, 255, 255, 255]);
        b.rgba[2 * 4..2 * 4 + 3].fill(0);
        assert!(b.flood_fill(0, 0, RED, FILL_TOLERANCE));
        assert_eq!(b.pixel(1, 0), Some([255, 0, 0, 255]));
        assert_eq!(b.pixel(2, 0), Some([0, 0, 0, 255]));
        assert_eq!(b.pixel(3, 0), Some([255, 255, 255, 255]));
        assert!(!b.flood_fill(9, 9, RED, 0));
    }

    #[test]
    fn flood_fill_with_same_color_terminates() {
        let mut b = solid(50, 50, [255, 0, 0, 255]);
        assert!(b.flood_fill(25, 25, RED, 255));
        assert_eq!(b.pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn rotation_moves_pixels_and_swaps_size() {
        let mut b = Bitmap::from_rgba(2, 1, vec![1, 0, 0, 255, 2, 0, 0, 255]).unwrap();
        b.rotate90(true);
        assert_eq!((b.width(), b.height()), (1, 2));
        assert_eq!(b.pixel(0, 0).unwrap()[0], 1);
        assert_eq!(b.pixel(0, 1).unwrap()[0], 2);
        b.rotate90(false);
        assert_eq!((b.width(), b.height()), (2, 1));
        assert_eq!(b.pixel(1, 0).unwrap()[0], 2);
    }

    #[test]
    fn flip_moves_pixels_both_ways() {
        let source = Bitmap::from_rgba(2, 2, (1..=16).collect()).unwrap();
        let mut h = source.clone();
        h.flip(true);
        assert_eq!(h.pixel(0, 0), source.pixel(1, 0));
        let mut v = source.clone();
        v.flip(false);
        assert_eq!(v.pixel(0, 0), source.pixel(0, 1));
        v.flip(false);
        assert_eq!(v, source);
    }

    #[test]
    fn point_filters_keep_alpha() {
        let mut b = solid(1, 1, [10, 100, 250, 77]);
        b.apply_filter(Filter::Invert);
        assert_eq!(b.pixel(0, 0), Some([245, 155, 5, 77]));
        b.apply_filter(Filter::Grayscale);
        let p = b.pixel(0, 0).unwrap();
        assert!(p[0] == p[1] && p[1] == p[2] && p[3] == 77);
    }

    #[test]
    fn brightness_clamps() {
        let mut b = solid(1, 1, [250, 5, 100, 255]);
        b.apply_filter(Filter::Brighten);
        assert_eq!(b.pixel(0, 0), Some([255, 30, 125, 255]));
        let mut d = solid(1, 1, [250, 5, 100, 255]);
        d.apply_filter(Filter::Darken);
        assert_eq!(d.pixel(0, 0), Some([225, 0, 75, 255]));
    }

    #[test]
    fn blur_and_sharpen_keep_dimensions() {
        let mut b = solid(16, 9, [120, 60, 30, 255]);
        b.apply_filter(Filter::Blur);
        b.apply_filter(Filter::Sharpen);
        assert_eq!((b.width(), b.height(), b.rgba().len()), (16, 9, 16 * 9 * 4));
    }
}
