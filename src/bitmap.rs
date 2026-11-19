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

//! Straight-alpha RGBA images and the pixel operations applied to them.

use std::fmt;
use std::io::Cursor;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::{ImageFormat, RgbaImage};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::i18n::{t, tf};
use crate::model::{v2, Color, Rect, Vec2, Warp};

pub const MAX_DIMENSION: u32 = 16000;
/// The longest side, in pixels, a free transform will give a bitmap.
const WARP_MAX_SIDE: f32 = 8000.0;
/// Rounding error allowed when fitting a transformed bitmap to whole pixels.
const WARP_SNAP: f32 = 1e-3;
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
            Filter::Invert => t("Invert"),
            Filter::Grayscale => t("Grayscale"),
            Filter::Brighten => t("Brighten"),
            Filter::Darken => t("Darken"),
            Filter::Blur => t("Blur"),
            Filter::Sharpen => t("Sharpen"),
        }
    }
}

/// The tones Dodge and Burn work on most.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToneRange {
    Shadows,
    #[default]
    Midtones,
    Highlights,
}

impl ToneRange {
    pub const ALL: [ToneRange; 3] = [ToneRange::Shadows, ToneRange::Midtones, ToneRange::Highlights];

    pub fn label(self) -> &'static str {
        match self {
            ToneRange::Shadows => crate::i18n::t("Shadows"),
            ToneRange::Midtones => crate::i18n::t("Midtones"),
            ToneRange::Highlights => crate::i18n::t("Highlights"),
        }
    }

    /// How much a pixel of this lightness, from 0 to 1, is changed.
    fn weight(self, light: f32) -> f32 {
        match self {
            ToneRange::Shadows => (1.0 - light).powi(2),
            ToneRange::Midtones => (4.0 * light * (1.0 - light)).clamp(0.0, 1.0).max(0.15),
            ToneRange::Highlights => light.powi(2),
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
    /// For a feathered selection, how much of each pixel is selected, from 0 to
    /// 255. `cells` then marks those at least half selected, which is where the
    /// marquee is drawn.
    soft: Option<Vec<u8>>,
}

/// How a mask is written to a file: its size, then how many cells in a row are
/// unset, set, unset and so on, row after row.
#[derive(Serialize, Deserialize)]
struct MaskRuns {
    width: u32,
    height: u32,
    runs: Vec<u32>,
}

impl Serialize for Mask {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut runs = Vec::new();
        let (mut current, mut length) = (false, 0u32);
        for cell in &self.cells {
            if *cell != current {
                runs.push(length);
                (current, length) = (*cell, 0);
            }
            length += 1;
        }
        runs.push(length);
        MaskRuns { width: self.width, height: self.height, runs }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Mask {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let file = MaskRuns::deserialize(deserializer)?;
        let total = file.width as u64 * file.height as u64;
        if file.width > MAX_DIMENSION || file.height > MAX_DIMENSION || file.runs.iter().map(|r| *r as u64).sum::<u64>() != total {
            return Err(serde::de::Error::custom("mask does not fill its size"));
        }
        let mut cells = Vec::with_capacity(total as usize);
        for (index, run) in file.runs.iter().enumerate() {
            cells.extend(std::iter::repeat_n(index % 2 == 1, *run as usize));
        }
        Ok(Mask { width: file.width, height: file.height, cells, soft: None })
    }
}

impl Mask {
    /// A mask of the given size with the cells set for which `set` says so.
    pub fn from_fn(width: u32, height: u32, set: impl Fn(u32, u32) -> bool) -> Self {
        let mut mask = Self::empty(width, height);
        for y in 0..height {
            for x in 0..width {
                mask.cells[y as usize * width as usize + x as usize] = set(x, y);
            }
        }
        mask
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The set cells as rectangles (left, top, width, height), with equal runs on
    /// neighbouring rows joined, so a simple mask is a handful of them.
    pub fn rects(&self) -> Vec<(u32, u32, u32, u32)> {
        let mut done: Vec<(u32, u32, u32, u32)> = Vec::new();
        let mut open: Vec<(u32, u32, u32, u32)> = Vec::new();
        for y in 0..=self.height {
            let mut runs = Vec::new();
            let mut x = 0;
            while y < self.height && x < self.width {
                if self.is_set(x, y) {
                    let start = x;
                    while x < self.width && self.is_set(x, y) {
                        x += 1;
                    }
                    runs.push((start, x - start));
                } else {
                    x += 1;
                }
            }
            let mut still_open = Vec::new();
            for rect in open.drain(..) {
                match runs.iter().position(|run| *run == (rect.0, rect.2)) {
                    Some(at) => {
                        runs.remove(at);
                        still_open.push((rect.0, rect.1, rect.2, rect.3 + 1));
                    }
                    None => done.push(rect),
                }
            }
            still_open.extend(runs.into_iter().map(|(left, width)| (left, y, width, 1)));
            open = still_open;
        }
        done
    }

    pub fn empty(width: u32, height: u32) -> Self {
        Self { width, height, cells: vec![false; width as usize * height as usize], soft: None }
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

    /// The mask moved by whole pixels. What is pushed past an edge is lost.
    pub fn shifted(&self, dx: i64, dy: i64) -> Mask {
        let mut moved = Self::empty(self.width, self.height);
        for y in 0..self.height as i64 {
            for x in 0..self.width as i64 {
                if self.get(x - dx, y - dy) {
                    moved.cells[y as usize * self.width as usize + x as usize] = true;
                }
            }
        }
        if let Some(soft) = &self.soft {
            let (w, h) = (self.width as i64, self.height as i64);
            let at = |x: i64, y: i64| if x < 0 || y < 0 || x >= w || y >= h { 0 } else { soft[(y * w + x) as usize] };
            moved.soft = Some((0..h).flat_map(|y| (0..w).map(move |x| (x, y))).map(|(x, y)| at(x - dx, y - dy)).collect());
        }
        moved
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
        // With a feathered selection on either side it is the parts of each pixel that are added or taken away.
        if (self.soft.is_some() || other.soft.is_some()) && mode != MaskMode::Replace {
            let weights: Vec<u8> = (0..self.cells.len())
                .map(|cell| {
                    let (mine, theirs) = (self.weight(cell), other.weight(cell));
                    let kept = if mode == MaskMode::Add { mine.max(theirs) } else { mine.min(1.0 - theirs) };
                    (kept * 255.0).round() as u8
                })
                .collect();
            self.cells = weights.iter().map(|w| *w >= 128).collect();
            self.soft = Some(weights);
            return;
        }
        self.soft = if mode == MaskMode::Replace { other.soft.clone() } else { None };
        for (cell, new) in self.cells.iter_mut().zip(&other.cells) {
            *cell = match mode {
                MaskMode::Replace => *new,
                MaskMode::Add => *cell || *new,
                MaskMode::Subtract => *cell && !*new,
            };
        }
    }

    /// How far each cell is from the nearest cell that is set, or that is not.
    fn distance_to(&self, set: bool) -> Vec<f32> {
        let weights: Vec<f32> = self.cells.iter().map(|cell| if *cell == set { 1.0 } else { 0.0 }).collect();
        crate::pseffects::distances(&weights, self.width as usize, self.height as usize)
    }

    fn with_cells(&self, cells: Vec<bool>) -> Mask {
        Mask { width: self.width, height: self.height, cells, soft: None }
    }

    /// The selection with everything else selected instead.
    pub fn inverted(&self) -> Mask {
        let mut inverse = self.with_cells(self.cells.iter().map(|cell| !cell).collect());
        inverse.soft = self.soft.as_ref().map(|soft| soft.iter().map(|w| 255 - w).collect());
        inverse
    }

    /// Fireworks' Feather: the edge of the selection fades out over about `radius`
    /// pixels to each side of where it was.
    pub fn feathered(&self, radius: f32) -> Mask {
        let hard: Vec<f32> = (0..self.cells.len()).map(|cell| self.weight(cell)).collect();
        let blurred = crate::effects::blur_channel(&hard, self.width as usize, self.height as usize, radius.max(0.1));
        let mut soft = self.with_cells(blurred.iter().map(|w| *w >= 0.5).collect());
        soft.soft = Some(blurred.iter().map(|w| (w.clamp(0.0, 1.0) * 255.0).round() as u8).collect());
        soft
    }

    /// How much of the pixel in a cell is selected, from 0 to 1.
    pub fn weight(&self, cell: usize) -> f32 {
        match &self.soft {
            Some(soft) => soft.get(cell).map_or(0.0, |w| *w as f32 / 255.0),
            None => self.cells.get(cell).map_or(0.0, |set| if *set { 1.0 } else { 0.0 }),
        }
    }

    #[cfg(test)]
    pub fn is_feathered(&self) -> bool {
        self.soft.is_some()
    }

    /// Whether any part of the pixel in a cell is selected.
    fn touches(&self, cell: usize) -> bool {
        match &self.soft {
            Some(soft) => soft[cell] > 0,
            None => self.cells[cell],
        }
    }

    /// How wide the soft edge of a feathered selection is, as the radius it was feathered by; none for a plain one.
    fn softness(&self) -> Option<f32> {
        let soft = self.soft.as_ref()?;
        // Partly selected pixels lie in a band about three radii wide along the edge.
        let partly = soft.iter().filter(|w| **w > 12 && **w < 243).count() as f32;
        let edge = self.edges().len().max(1) as f32;
        Some((partly / edge / 2.0).max(0.5))
    }

    /// A plain result made soft again, if the selection it was made from was.
    fn softened_like(self, from: &Mask) -> Mask {
        match from.softness() {
            Some(radius) => self.feathered(radius),
            None => self,
        }
    }

    /// The selection grown by `by` pixels all around. A feathered one stays feathered.
    pub fn expanded(&self, by: u32) -> Mask {
        self.with_cells(self.distance_to(true).iter().map(|d| *d <= by as f32).collect()).softened_like(self)
    }

    /// The selection shrunk by `by` pixels all around. The edge of the bitmap is not an edge of the selection.
    pub fn contracted(&self, by: u32) -> Mask {
        self.with_cells(self.distance_to(false).iter().map(|d| *d > by as f32).collect()).softened_like(self)
    }

    /// A band `width` pixels wide along the edge of the selection, half of it to each side.
    pub fn border(&self, width: u32) -> Mask {
        let grown = self.with_cells(self.distance_to(true).iter().map(|d| *d <= width.div_ceil(2) as f32).collect());
        let shrunk = self.with_cells(self.distance_to(false).iter().map(|d| *d > (width / 2) as f32).collect());
        self.with_cells(grown.cells.iter().zip(&shrunk.cells).map(|(o, i)| *o && !*i).collect()).softened_like(self)
    }

    /// The selection with its edge evened out: a pixel is selected if most of the
    /// pixels within `radius` of it, in a square, are.
    pub fn smoothed(&self, radius: u32) -> Mask {
        let (w, h, r) = (self.width as i64, self.height as i64, radius as i64);
        // How many cells are set above and to the left of each corner.
        let mut sums = vec![0u32; ((w + 1) * (h + 1)) as usize];
        for y in 0..h {
            for x in 0..w {
                let set = self.cells[(y * w + x) as usize] as u32;
                sums[((y + 1) * (w + 1) + x + 1) as usize] = set + sums[(y * (w + 1) + x + 1) as usize] + sums[((y + 1) * (w + 1) + x) as usize] - sums[(y * (w + 1) + x) as usize];
            }
        }
        let cells = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| {
                let (left, top, right, bottom) = ((x - r).max(0), (y - r).max(0), (x + r + 1).min(w), (y + r + 1).min(h));
                let at = |x: i64, y: i64| sums[(y * (w + 1) + x) as usize];
                let set = at(right, bottom) + at(left, top) - at(right, top) - at(left, bottom);
                set * 2 > ((right - left) * (bottom - top)) as u32
            })
            .collect();
        self.with_cells(cells)
    }

    /// Returns x, y, width and height of the smallest box around the selection.
    fn bounds(&self) -> Option<(u32, u32, u32, u32)> {
        let w = self.width as usize;
        // A feathered selection reaches as far as its faintest pixel.
        if self.soft.is_some() {
            let touched: Vec<usize> = (0..self.cells.len()).filter(|cell| self.touches(*cell)).collect();
            let (first, last) = (*touched.first()?, *touched.last()?);
            let (left, right) = touched.iter().fold((usize::MAX, 0), |(l, r), cell| (l.min(cell % w), r.max(cell % w)));
            return Some((left as u32, (first / w) as u32, (right - left + 1) as u32, (last / w - first / w + 1) as u32));
        }
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
        let decoded = image::load_from_memory(bytes).context(t("not a supported image"))?.to_rgba8();
        let (width, height) = decoded.dimensions();
        anyhow::ensure!(
            width <= MAX_DIMENSION && height <= MAX_DIMENSION,
            tf("image is larger than {} pixels on a side", &[&MAX_DIMENSION])
        );
        Self::from_rgba(width, height, decoded.into_raw()).ok_or_else(|| anyhow!(t("image has no pixels")))
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

    /// Sharpens under a round stroke, by pushing each pixel away from a blurred copy of itself.
    pub fn sharpen_stroke(&mut self, points: &[Vec2], width: f32) {
        let Some((left, top, layer)) = self.coverage(points, width) else { return };
        let pad = (BLUR_SIGMA * 3.0).ceil() as u32;
        let (x0, y0) = (left.saturating_sub(pad), top.saturating_sub(pad));
        let x1 = (left + layer.width() + pad).min(self.width);
        let y1 = (top + layer.height() + pad).min(self.height);
        let source = self.to_image();
        let region = image::imageops::crop_imm(&source, x0, y0, x1 - x0, y1 - y0).to_image();
        let blurred = image::imageops::blur(&region, BLUR_SIGMA);
        self.apply_layer(left, top, &layer, |x, y, dst, coverage| {
            let soft = blurred.get_pixel(x - x0, y - y0).0;
            for channel in 0..3 {
                let sharp = dst[channel] as f32 + (dst[channel] as f32 - soft[channel] as f32) * coverage;
                dst[channel] = sharp.round().clamp(0.0, 255.0) as u8;
            }
        });
    }

    /// Lightens (Dodge) or darkens (Burn) under a round stroke. `exposure`, from 0 to
    /// 1, is how far a pixel goes toward white or black, and `range` says which
    /// tones are worked on most.
    pub fn tone_stroke(&mut self, points: &[Vec2], width: f32, lighten: bool, exposure: f32, range: ToneRange) {
        let Some((left, top, layer)) = self.coverage(points, width) else { return };
        self.apply_layer(left, top, &layer, |_, _, dst, coverage| {
            let light = (0.3 * dst[0] as f32 + 0.59 * dst[1] as f32 + 0.11 * dst[2] as f32) / 255.0;
            let amount = exposure.clamp(0.0, 1.0) * coverage * range.weight(light);
            for value in dst.iter_mut().take(3) {
                let to = if lighten { 255.0 } else { 0.0 };
                *value = (*value as f32 + (to - *value as f32) * amount).round() as u8;
            }
        });
    }

    /// Smudges along a stroke: the paint under the brush where it starts is carried
    /// along and rubbed into what the brush passes over, picking that up in turn.
    ///
    /// `pressure`, from 0 to 1, is how strongly it rubs in. With `start` the brush
    /// begins loaded with that color, as with Fireworks' Smudge color.
    pub fn smudge_stroke(&mut self, points: &[Vec2], width: f32, pressure: f32, start: Option<Color>) {
        let radius = (width / 2.0).max(1.0);
        let reach = radius.ceil() as i64;
        // The stroke in steps of a quarter of the brush, so the smear has no gaps.
        let mut steps: Vec<Vec2> = Vec::new();
        for pair in points.windows(2) {
            let (dx, dy) = (pair[1].x - pair[0].x, pair[1].y - pair[0].y);
            let count = ((dx.hypot(dy) / (radius / 2.0).max(0.5)).ceil() as usize).max(1);
            steps.extend((0..count).map(|i| Vec2 { x: pair[0].x + dx * i as f32 / count as f32, y: pair[0].y + dy * i as f32 / count as f32 }));
        }
        steps.extend(points.last().copied());
        let Some(first) = steps.first().copied() else { return };
        let side = (reach * 2 + 1) as usize;
        let cell = |ox: i64, oy: i64| ((oy + reach) as usize) * side + (ox + reach) as usize;
        let inside = |x: i64, y: i64, w: u32, h: u32| x >= 0 && y >= 0 && x < w as i64 && y < h as i64;
        // What the brush carries: the pixels under it, by their place in the brush.
        let mut carried: Vec<Option<[u8; 4]>> = vec![None; side * side];
        let (w, h) = (self.width, self.height);
        for oy in -reach..=reach {
            for ox in -reach..=reach {
                let (x, y) = (first.x.floor() as i64 + ox, first.y.floor() as i64 + oy);
                if inside(x, y, w, h) {
                    let i = self.index(x as u32, y as u32);
                    carried[cell(ox, oy)] = Some(match start {
                        Some(color) => [color.r, color.g, color.b, 255],
                        None => [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]],
                    });
                }
            }
        }
        // Loaded with a color, the brush rubs it in where it starts as well.
        for step in steps.iter().skip(if start.is_some() { 0 } else { 1 }) {
            for oy in -reach..=reach {
                for ox in -reach..=reach {
                    let far = ((ox * ox + oy * oy) as f32).sqrt();
                    let (x, y) = (step.x.floor() as i64 + ox, step.y.floor() as i64 + oy);
                    if far > radius || !inside(x, y, w, h) {
                        continue;
                    }
                    let i = self.index(x as u32, y as u32);
                    let Some(paint) = carried[cell(ox, oy)] else {
                        carried[cell(ox, oy)] = Some([self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]]);
                        continue;
                    };
                    // Strongest in the middle of the brush, fading to nothing at its rim.
                    let strength = pressure.clamp(0.0, 1.0) * (1.0 - far / radius).clamp(0.0, 1.0).sqrt();
                    mix(&mut self.rgba[i..i + 4], &paint, strength);
                    carried[cell(ox, oy)] = Some([self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]]);
                }
            }
        }
    }

    /// Replaces one color with another under a round stroke: pixels within
    /// `tolerance` of `from` take the color `to`, and keep how opaque they were.
    pub fn replace_color_stroke(&mut self, points: &[Vec2], width: f32, from: [u8; 4], to: Color, tolerance: u8) {
        let Some((left, top, layer)) = self.coverage(points, width) else { return };
        self.apply_layer(left, top, &layer, |_, _, dst, coverage| {
            if dst[3] > 0 && (0..3).all(|c| dst[c].abs_diff(from[c]) <= tolerance) {
                let alpha = dst[3];
                mix(dst, &[to.r, to.g, to.b, alpha], coverage);
                dst[3] = alpha;
            }
        });
    }

    /// Takes the red out of eyes between two corners. `tolerance`, from 0 to 1, is
    /// how readily a pixel counts as red: at 0 only the purest red, at 1 anything
    /// redder than it is green or blue. `strength`, from 0 to 1, is how much of
    /// the red is taken away. Returns how many pixels changed.
    pub fn remove_red_eye(&mut self, a: Vec2, b: Vec2, tolerance: f32, strength: f32) -> usize {
        let (x0, x1) = (a.x.min(b.x).floor().max(0.0) as u32, (a.x.max(b.x).ceil().max(0.0) as u32).min(self.width));
        let (y0, y1) = (a.y.min(b.y).floor().max(0.0) as u32, (a.y.max(b.y).ceil().max(0.0) as u32).min(self.height));
        // How many times redder than green and blue a pixel must be: 2 at no tolerance, 1 at full.
        let ratio = 2.0 - tolerance.clamp(0.0, 1.0);
        let mut changed = 0;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = self.index(x, y);
                let (r, g, bl) = (self.rgba[i] as f32, self.rgba[i + 1] as f32, self.rgba[i + 2] as f32);
                if self.rgba[i + 3] > 0 && r > 80.0 && r > g * ratio && r > bl * ratio {
                    self.rgba[i] = (r + ((g + bl) / 2.0 - r) * strength.clamp(0.0, 1.0)).round() as u8;
                    changed += 1;
                }
            }
        }
        changed
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
    /// Every pixel whose color is like those selected: each channel within
    /// `tolerance` of the range the selected pixels span.
    pub fn similar(&self, mask: &Mask, tolerance: u8) -> Option<Mask> {
        if !mask.fits(self) || mask.is_empty() {
            return None;
        }
        // Every color in the selection counts by itself: a pixel is alike if it is within
        // the tolerance of one of them, not merely somewhere between the darkest and the lightest.
        // The colors are kept in boxes as wide as the tolerance, so a pixel is held against few of them.
        let step = tolerance as u16 + 1;
        let cell = |px: &[u8]| [px[0] as u16 / step, px[1] as u16 / step, px[2] as u16 / step, px[3] as u16 / step];
        let mut boxes: std::collections::HashMap<[u16; 4], Vec<[u8; 4]>> = std::collections::HashMap::new();
        for (px, _) in self.rgba.chunks_exact(4).zip(&mask.cells).filter(|(_, set)| **set) {
            let colors = boxes.entry(cell(px)).or_default();
            let color = [px[0], px[1], px[2], px[3]];
            if !colors.contains(&color) {
                colors.push(color);
            }
        }
        let near = |px: &[u8], color: &[u8; 4]| (0..4).all(|c| px[c].abs_diff(color[c]) <= tolerance);
        let alike = |px: &[u8]| {
            let at = cell(px);
            // A color within the tolerance lies in the pixel's own box or in one beside it.
            (0..81u16).any(|n| {
                let beside = [at[0] + n % 3, at[1] + n / 3 % 3, at[2] + n / 9 % 3, at[3] + n / 27 % 3];
                if beside.contains(&0) {
                    return false;
                }
                boxes.get(&beside.map(|v| v - 1)).is_some_and(|colors| colors.iter().any(|color| near(px, color)))
            })
        };
        Some(Mask { width: self.width, height: self.height, cells: self.rgba.chunks_exact(4).map(alike).collect(), soft: None })
    }

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
        for (cell, (px, old)) in self.rgba.chunks_exact_mut(4).zip(before.rgba.chunks_exact(4)).enumerate() {
            let weight = mask.weight(cell);
            if weight <= 0.0 {
                px.copy_from_slice(old);
            } else if weight < 1.0 {
                // A partly selected pixel takes that part of the change.
                for (new, old) in px.iter_mut().zip(old) {
                    *new = (*old as f32 + (*new as f32 - *old as f32) * weight).round() as u8;
                }
            }
        }
    }

    /// Makes the selected pixels fully transparent.
    pub fn clear(&mut self, mask: &Mask) {
        if mask.fits(self) {
            for (cell, px) in self.rgba.chunks_exact_mut(4).enumerate() {
                let weight = mask.weight(cell);
                if weight >= 1.0 {
                    px.fill(0);
                } else if weight > 0.0 {
                    px[3] = (px[3] as f32 * (1.0 - weight)).round() as u8;
                }
            }
        }
    }

    /// Lays `piece` over this bitmap with its top-left corner at (`x`, `y`). The
    /// part of it that falls outside is dropped, and its transparent pixels show
    /// what is underneath.
    pub fn overlay(&mut self, piece: &Bitmap, x: i64, y: i64) {
        for py in 0..piece.height as i64 {
            for px in 0..piece.width as i64 {
                let (tx, ty) = (x + px, y + py);
                if tx < 0 || ty < 0 || tx >= self.width as i64 || ty >= self.height as i64 {
                    continue;
                }
                let from = (py as usize * piece.width as usize + px as usize) * 4;
                let alpha = piece.rgba[from + 3] as f32 / 255.0;
                if alpha <= 0.0 {
                    continue;
                }
                let to = self.index(tx as u32, ty as u32);
                let under = self.rgba[to + 3] as f32 / 255.0;
                let out = alpha + under * (1.0 - alpha);
                for channel in 0..3 {
                    let mixed = (piece.rgba[from + channel] as f32 * alpha + self.rgba[to + channel] as f32 * under * (1.0 - alpha)) / out;
                    self.rgba[to + channel] = mixed.round() as u8;
                }
                self.rgba[to + 3] = (out * 255.0).round() as u8;
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
            let weight = mask.weight(y as usize * self.width as usize + x as usize);
            if weight > 0.0 {
                let i = self.index(x, y);
                px.copy_from_slice(&self.rgba[i..i + 4]);
                px[3] = (px[3] as f32 * weight).round() as u8;
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

    /// Redraws the bitmap as a free transform moves it. `bounds` is where the bitmap
    /// sits in the document. Returns the new pixels and where they sit, or `None`
    /// when the result would be empty or too large.
    pub fn warped(&self, bounds: Rect, warp: &Warp) -> Option<(Bitmap, Rect)> {
        if bounds.w <= 0.0 || bounds.h <= 0.0 {
            return None;
        }
        let inverse = warp.inverse()?;
        let corners = [v2(bounds.x, bounds.y), v2(bounds.right(), bounds.y), v2(bounds.right(), bounds.bottom()), v2(bounds.x, bounds.bottom())]
            .map(|corner| warp.apply(corner));
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for corner in corners {
            (x0, y0, x1, y1) = (x0.min(corner.x), y0.min(corner.y), x1.max(corner.x), y1.max(corner.y));
        }
        // A half turn lands a hair off whole pixels, which must not add an empty row.
        let (left, top) = ((x0 + WARP_SNAP).floor(), (y0 + WARP_SNAP).floor());
        let target = Rect { x: left, y: top, w: (x1 - WARP_SNAP).ceil() - left, h: (y1 - WARP_SNAP).ceil() - top };
        // Pixels per document unit of the source, kept so a bitmap shown small stays sharp.
        let (sx, sy) = (self.width as f32 / bounds.w, self.height as f32 / bounds.h);
        let mut density = ((sx + sy) / 2.0).clamp(0.25, 4.0);
        let longest = target.w.max(target.h);
        if longest * density > WARP_MAX_SIDE {
            density = WARP_MAX_SIDE / longest;
        }
        let (width, height) = ((target.w * density).round() as u32, (target.h * density).round() as u32);
        if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
            return None;
        }
        let (w, h) = (self.width as i64, self.height as i64);
        // Premultiplied, so transparent pixels do not tint the edges they blend with.
        let sample = |x: i64, y: i64| -> [f32; 4] {
            if x < 0 || y < 0 || x >= w || y >= h {
                return [0.0; 4];
            }
            let at = ((y * w + x) * 4) as usize;
            let alpha = self.rgba[at + 3] as f32 / 255.0;
            [self.rgba[at] as f32 * alpha, self.rgba[at + 1] as f32 * alpha, self.rgba[at + 2] as f32 * alpha, alpha * 255.0]
        };
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        for row in 0..height {
            for column in 0..width {
                let doc = v2(target.x + (column as f32 + 0.5) / density, target.y + (row as f32 + 0.5) / density);
                let from = inverse.apply(doc);
                let (u, v) = ((from.x - bounds.x) * sx - 0.5, (from.y - bounds.y) * sy - 0.5);
                if u < -1.0 || v < -1.0 || u > w as f32 || v > h as f32 {
                    continue;
                }
                let (left, top) = (u.floor(), v.floor());
                let (fx, fy) = (u - left, v - top);
                let (left, top) = (left as i64, top as i64);
                let mut mixed = [0.0f32; 4];
                for (dx, dy, weight) in [(0, 0, (1.0 - fx) * (1.0 - fy)), (1, 0, fx * (1.0 - fy)), (0, 1, (1.0 - fx) * fy), (1, 1, fx * fy)] {
                    let px = sample(left + dx, top + dy);
                    for channel in 0..4 {
                        mixed[channel] += px[channel] * weight;
                    }
                }
                let alpha = mixed[3];
                if alpha <= 0.0 {
                    continue;
                }
                let at = (row as usize * width as usize + column as usize) * 4;
                for channel in 0..3 {
                    rgba[at + channel] = (mixed[channel] * 255.0 / alpha).round().clamp(0.0, 255.0) as u8;
                }
                rgba[at + 3] = alpha.round().clamp(0.0, 255.0) as u8;
            }
        }
        Some((Bitmap { width, height, rgba }, target))
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
    fn a_marquee_can_be_grown_shrunk_bordered_inverted_and_smoothed() {
        // A square of 10 in the middle of a bitmap of 30.
        let square = Mask::rect(30, 30, Vec2 { x: 10.0, y: 10.0 }, Vec2 { x: 20.0, y: 20.0 });
        let count = |mask: &Mask| mask.cells.iter().filter(|c| **c).count();
        assert_eq!(count(&square), 100);
        // Grown by 3 it reaches 3 further each way, with round corners.
        let bigger = square.expanded(3);
        assert!(bigger.is_set(7, 15) && !bigger.is_set(6, 15) && bigger.is_set(22, 15) && !bigger.is_set(23, 15));
        assert!(bigger.is_set(15, 7) && !bigger.is_set(7, 7));
        // Shrunk by 3 it is a square of 4.
        let smaller = square.contracted(3);
        assert_eq!(count(&smaller), 16);
        assert!(smaller.is_set(13, 13) && !smaller.is_set(12, 13));
        assert!(square.contracted(5).is_empty());
        // A selection of the whole bitmap has no edge to shrink from.
        assert_eq!(count(&Mask::from_fn(8, 8, |_, _| true).contracted(2)), 64);
        // A border of 4 is 2 to each side of the edge.
        let band = square.border(4);
        assert!(band.is_set(8, 15) && band.is_set(11, 15) && !band.is_set(12, 15) && !band.is_set(7, 15));
        // Inverted, everything else is selected; twice, it is back.
        assert_eq!(count(&square.inverted()), 800);
        assert_eq!(square.inverted().inverted(), square);
        // Smoothing drops a stray pixel and fills a pinhole.
        let mut rough = square.clone();
        rough.cells[2 * 30 + 2] = true;
        rough.cells[15 * 30 + 15] = false;
        let even = rough.smoothed(2);
        assert!(!even.is_set(2, 2) && even.is_set(15, 15) && even.is_set(12, 12));
    }

    #[test]
    fn a_feathered_selection_fades_out_and_what_is_done_in_it_fades_with_it() {
        let square = Mask::rect(40, 40, Vec2 { x: 10.0, y: 10.0 }, Vec2 { x: 30.0, y: 30.0 });
        assert!(!square.is_feathered() && square.weight(20 * 40 + 20) == 1.0 && square.weight(0) == 0.0);
        let soft = square.feathered(4.0);
        assert!(soft.is_feathered());
        let weight = |x: usize, y: usize| soft.weight(y * 40 + x);
        // Whole in the middle, half at the old edge, a little beyond it, nothing far away.
        assert!(weight(20, 20) > 0.98 && (weight(10, 20) - 0.5).abs() < 0.15, "{}", weight(10, 20));
        assert!(weight(7, 20) > 0.02 && weight(7, 20) < 0.4 && weight(13, 20) > 0.6 && weight(0, 20) == 0.0);
        // The marquee is drawn where half is selected.
        assert!(soft.is_set(20, 20) && !soft.is_set(5, 20));
        // Deleting through it leaves a soft hole.
        let mut bitmap = Bitmap::from_rgba(40, 40, [200, 0, 0, 255].repeat(1600)).unwrap();
        bitmap.clear(&soft);
        let alpha = |b: &Bitmap, x: u32, y: u32| b.pixel(x, y).unwrap()[3];
        assert!(alpha(&bitmap, 20, 20) < 5 && alpha(&bitmap, 0, 20) == 255 && (60..200).contains(&alpha(&bitmap, 10, 20)), "{}", alpha(&bitmap, 10, 20));
        // Copying through it gives a piece with soft edges, as big as the selection reaches.
        let whole = Bitmap::from_rgba(40, 40, [200, 0, 0, 255].repeat(1600)).unwrap();
        let (piece, left, top) = whole.extract(&soft).unwrap();
        assert!(left < 10 && top < 10 && piece.width() > 20);
        assert!(alpha(&piece, piece.width() / 2, piece.height() / 2) > 250 && alpha(&piece, 0, piece.height() / 2) < 40);
        // A change made to the whole bitmap is kept in proportion.
        let mut changed = Bitmap::from_rgba(40, 40, [0, 0, 0, 255].repeat(1600)).unwrap();
        changed.restore_outside(&whole, &soft);
        assert_eq!(changed.pixel(0, 20).unwrap()[0], 200);
        assert!(changed.pixel(20, 20).unwrap()[0] < 5 && (60..160).contains(&changed.pixel(10, 20).unwrap()[0]));
        // Inverted it is the other part of every pixel; moved, it keeps its softness; added to, it is plain again.
        assert!((soft.inverted().weight(20 * 40 + 10) + weight(10, 20) - 1.0).abs() < 0.01);
        assert!((soft.shifted(5, 0).weight(20 * 40 + 15) - weight(10, 20)).abs() < 0.01);
        // Added to, it takes the larger part of each pixel and stays soft; subtracted from, what is taken leaves a soft hole.
        let other = Mask::rect(40, 40, Vec2 { x: 20.0, y: 10.0 }, Vec2 { x: 38.0, y: 30.0 });
        let mut added = soft.clone();
        added.combine(&other, MaskMode::Add);
        assert!(added.is_feathered() && added.weight(20 * 40 + 35) == 1.0 && (added.weight(20 * 40 + 10) - weight(10, 20)).abs() < 0.01);
        let mut taken = soft.clone();
        taken.combine(&soft.shifted(8, 0), MaskMode::Subtract);
        assert!(taken.is_feathered() && taken.weight(20 * 40 + 25) < 0.05 && taken.weight(20 * 40 + 11) > 0.5);
        // Grown or shrunk it is still feathered, about as softly as before.
        let grown = soft.expanded(4);
        assert!(grown.is_feathered() && grown.weight(20 * 40 + 6) > weight(6, 20) + 0.2);
        let edge = grown.weight(20 * 40 + 6);
        assert!(edge > 0.2 && edge < 0.8, "{edge}");
        assert!(soft.contracted(3).is_feathered() && !square.expanded(4).is_feathered());
    }

    #[test]
    fn the_retouching_brushes_change_only_what_is_under_them() {
        // Black on the left half, white on the right.
        let halves = || Bitmap::from_rgba(40, 20, (0..800).flat_map(|i| if i % 40 < 20 { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect()).unwrap();
        let stroke = [Vec2 { x: 5.0, y: 10.0 }, Vec2 { x: 35.0, y: 10.0 }];
        let red = |b: &Bitmap, x: u32, y: u32| b.pixel(x, y).unwrap()[0];
        // Dodge lightens the black, Burn darkens the white, and neither reaches the top row.
        let mut dodged = halves();
        dodged.tone_stroke(&stroke, 6.0, true, 0.25, ToneRange::Shadows);
        assert!(red(&dodged, 8, 10) > 40 && red(&dodged, 8, 1) == 0 && red(&dodged, 30, 10) == 255);
        let mut burned = halves();
        burned.tone_stroke(&stroke, 6.0, false, 0.25, ToneRange::Highlights);
        assert!(red(&burned, 30, 10) < 215 && red(&burned, 30, 1) == 255 && red(&burned, 8, 10) == 0);
        // A higher exposure goes further, and the range decides which tones move: highlights leave black alone.
        let mut stronger = halves();
        stronger.tone_stroke(&stroke, 6.0, true, 0.75, ToneRange::Shadows);
        assert!(red(&stronger, 8, 10) > red(&dodged, 8, 10) + 60);
        let mut highlights = halves();
        highlights.tone_stroke(&stroke, 6.0, true, 0.75, ToneRange::Highlights);
        assert_eq!(red(&highlights, 8, 10), 0);
        // Smudge drags black across the border into the white, and leaves the far end alone.
        let mut smudged = halves();
        smudged.smudge_stroke(&[Vec2 { x: 14.0, y: 10.0 }, Vec2 { x: 26.0, y: 10.0 }], 8.0, 0.5, None);
        assert!(red(&smudged, 21, 10) < 200 && red(&smudged, 38, 10) == 255 && red(&smudged, 21, 1) == 255, "{}", red(&smudged, 21, 10));
        assert_eq!(red(&smudged, 5, 10), 0);
        // More pressure drags more of it over, and a smudge color is rubbed in from the start.
        let mut harder = halves();
        harder.smudge_stroke(&[Vec2 { x: 14.0, y: 10.0 }, Vec2 { x: 26.0, y: 10.0 }], 8.0, 1.0, None);
        assert!(red(&harder, 23, 10) < red(&smudged, 23, 10));
        let mut tinted = halves();
        tinted.smudge_stroke(&[Vec2 { x: 26.0, y: 10.0 }, Vec2 { x: 34.0, y: 10.0 }], 8.0, 1.0, Some(Color::rgb(0, 255, 0)));
        let px = tinted.pixel(27, 10).unwrap();
        assert!(px[0] < 120 && px[1] == 255, "{px:?}");
        // Sharpen makes the two sides of a soft edge differ more.
        let soft: Vec<u8> = (0..800).flat_map(|i| { let v = ((i % 40) * 255 / 39) as u8; [v, v, v, 255] }).collect();
        let mut sharpened = Bitmap::from_rgba(40, 20, soft.clone()).unwrap();
        let before = Bitmap::from_rgba(40, 20, soft).unwrap();
        sharpened.sharpen_stroke(&stroke, 6.0);
        assert_eq!(red(&sharpened, 20, 1), red(&before, 20, 1));
        assert!(sharpened != before);
        // Replace Color changes only the color asked for.
        let mut replaced = halves();
        replaced.replace_color_stroke(&stroke, 6.0, [255, 255, 255, 255], Color::rgb(0, 200, 0), 10);
        assert_eq!((replaced.pixel(30, 10).unwrap(), replaced.pixel(8, 10).unwrap(), replaced.pixel(30, 1).unwrap()), ([0, 200, 0, 255], [0, 0, 0, 255], [255, 255, 255, 255]));
        // Red eye: strong red inside the box loses its red, other colors and the outside stay.
        let mut eyes = Bitmap::from_rgba(4, 1, vec![220, 30, 40, 255, 200, 180, 170, 255, 220, 30, 40, 255, 0, 0, 0, 0]).unwrap();
        assert_eq!(eyes.remove_red_eye(Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 2.0, y: 1.0 }, 0.5, 1.0), 1);
        assert_eq!((eyes.pixel(0, 0).unwrap(), eyes.pixel(1, 0).unwrap(), eyes.pixel(2, 0).unwrap()), ([35, 30, 40, 255], [200, 180, 170, 255], [220, 30, 40, 255]));
        // At half strength half of the red goes; with no tolerance a pinkish red is not taken for an eye.
        let mut half = Bitmap::from_rgba(2, 1, vec![220, 30, 40, 255, 200, 120, 120, 255]).unwrap();
        assert_eq!(half.remove_red_eye(Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 2.0, y: 1.0 }, 0.0, 0.5), 1);
        assert_eq!((half.pixel(0, 0).unwrap()[0], half.pixel(1, 0).unwrap()[0]), (128, 200));
        assert_eq!(half.remove_red_eye(Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 2.0, y: 1.0 }, 1.0, 1.0), 2);
    }

    #[test]
    fn similar_pixels_are_found_all_over_the_bitmap() {
        // Red on the left and the right, blue between them.
        let rgba: Vec<u8> = (0..9).flat_map(|i| if (3..6).contains(&(i % 9)) { [0, 0, 255, 255] } else { [250 - i as u8, 0, 0, 255] }).collect();
        let bitmap = Bitmap::from_rgba(9, 1, rgba).unwrap();
        let picked = Mask::from_fn(9, 1, |x, _| x == 0);
        let alike = bitmap.similar(&picked, 32).unwrap();
        assert_eq!((0..9).map(|x| alike.is_set(x, 0)).collect::<Vec<_>>(), [true, true, true, false, false, false, true, true, true]);
        // With no room for difference only the same color is taken.
        assert_eq!(bitmap.similar(&picked, 0).unwrap().cells.iter().filter(|c| **c).count(), 1);
        assert!(bitmap.similar(&Mask::empty(9, 1), 32).is_none());
        // Black and white picked together take black and white, and not the grey between them.
        let grey = Bitmap::from_rgba(4, 1, vec![0, 0, 0, 255, 128, 128, 128, 255, 255, 255, 255, 255, 250, 250, 250, 255]).unwrap();
        let both = Mask::from_fn(4, 1, |x, _| x == 0 || x == 2);
        let alike = grey.similar(&both, 10).unwrap();
        assert_eq!((0..4).map(|x| alike.is_set(x, 0)).collect::<Vec<_>>(), [true, false, true, true]);
    }

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
    fn a_mask_is_saved_as_runs_and_read_back_the_same() {
        let mask = Mask::from_fn(7, 5, |x, y| (x + y) % 3 == 0 || y == 4);
        let text = serde_json::to_string(&mask).unwrap();
        assert_eq!(serde_json::from_str::<Mask>(&text).unwrap(), mask);
        for mask in [Mask::empty(3, 3), Mask::from_fn(3, 3, |_, _| true)] {
            assert_eq!(serde_json::from_str::<Mask>(&serde_json::to_string(&mask).unwrap()).unwrap(), mask);
        }
        // Runs that do not add up to the size are refused.
        assert!(serde_json::from_str::<Mask>(r#"{"width":2,"height":2,"runs":[1,1]}"#).is_err());
    }

    #[test]
    fn a_mask_breaks_down_into_few_rectangles() {
        let block = Mask::rect(10, 10, v2(2.0, 3.0), v2(6.0, 8.0));
        assert_eq!(block.rects(), vec![(2, 3, 4, 5)]);
        assert!(Mask::empty(4, 4).rects().is_empty());
        // An L shape is two rectangles, and together they cover exactly the set cells.
        let ell = Mask::from_fn(6, 6, |x, y| x < 2 || y >= 4);
        let rects = ell.rects();
        assert_eq!(rects.len(), 2);
        let covered = Mask::from_fn(6, 6, |x, y| rects.iter().any(|(l, t, w, h)| x >= *l && x < l + w && y >= *t && y < t + h));
        assert_eq!(covered, ell);
    }

    #[test]
    fn a_mask_moves_by_whole_pixels_and_loses_what_leaves_the_image() {
        let mask = Mask::rect(4, 4, v2(0.0, 0.0), v2(2.0, 2.0));
        let moved = mask.shifted(1, 2);
        assert!(moved.is_set(1, 2) && moved.is_set(2, 3) && !moved.is_set(0, 0) && !moved.is_set(3, 3));
        assert!(mask.shifted(3, 0).is_set(3, 1) && !mask.shifted(3, 0).is_set(2, 1));
        assert!(mask.shifted(-5, 0).is_empty());
    }

    #[test]
    fn overlay_blends_over_what_is_there_and_ignores_what_falls_outside() {
        let mut base = Bitmap::from_rgba(2, 1, vec![0, 0, 255, 255, 0, 0, 0, 0]).unwrap();
        // An opaque red pixel and a transparent one, placed one pixel to the right.
        let piece = Bitmap::from_rgba(2, 1, vec![255, 0, 0, 255, 9, 9, 9, 0]).unwrap();
        base.overlay(&piece, 1, 0);
        assert_eq!(base.pixel(0, 0), Some([0, 0, 255, 255]));
        assert_eq!(base.pixel(1, 0), Some([255, 0, 0, 255]));
        // Half-transparent white over opaque blue is a lighter blue, still opaque.
        let mut base = Bitmap::from_rgba(1, 1, vec![0, 0, 255, 255]).unwrap();
        base.overlay(&Bitmap::from_rgba(1, 1, vec![255, 255, 255, 128]).unwrap(), 0, 0);
        let [r, g, b, a] = base.pixel(0, 0).unwrap();
        assert!((r as i32 - 128).abs() <= 1 && r == g && b == 255 && a == 255, "{r} {g} {b} {a}");
        base.overlay(&piece, -5, 3);
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
