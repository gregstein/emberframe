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

//! Rasterizes a page to PNG, and measures text with the same font used for export.

use std::path::Path;

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use anyhow::{anyhow, Context, Result};
use tiny_skia::{
    BlendMode, ColorU8, FillRule, FilterQuality, IntRect, LineCap, LineJoin, Paint, PathBuilder, Pixmap, PixmapPaint,
    PremultipliedColorU8, Stroke as SkStroke, Transform,
};

use crate::i18n::{t, tf};
use crate::bitmap::Bitmap;
use crate::model::{Color, MaskKind, Object, ObjectMask, Page, Rect, Shape, TextAlign, TextOrientation, TextStyle, Vec2, WebKind};

pub const LINE_HEIGHT: f32 = 1.2;

fn font() -> Option<&'static FontVec> {
    crate::fonts::outlines()
}

/// The face a text is set in.
fn styled_font(style: &TextStyle) -> Option<&'static FontVec> {
    crate::fonts::face(&style.font, style.bold, style.italic).or_else(font)
}

/// How wide one line of text is.
pub fn line_width(line: &str, size: f32, style: &TextStyle) -> f32 {
    match styled_font(style) {
        Some(f) => {
            let scaled = f.as_scaled(em_scale(f, size));
            line.chars().map(|c| scaled.h_advance(scaled.glyph_id(c))).sum::<f32>()
        }
        None => line.chars().count() as f32 * size * 0.55,
    }
}

/// How far from the left of its box a line of text starts, for an alignment.
pub fn line_start(box_width: f32, line_width: f32, align: TextAlign) -> f32 {
    match align {
        // A spread or stretched line starts at the left and reaches the right by other means.
        TextAlign::Left | TextAlign::Justify | TextAlign::Stretch => 0.0,
        TextAlign::Center => ((box_width - line_width) / 2.0).max(0.0),
        TextAlign::Right => (box_width - line_width).max(0.0),
    }
}

/// The outlines of text set along a path. The lines of the text follow one another,
/// and each letter is placed by its middle and stood on the path as the style says.
pub fn glyph_outlines_on_path(content: &str, size: f32, style: &TextStyle, path: &[Vec2]) -> Vec<Vec<Vec<Vec2>>> {
    use crate::model::PathOrientation;
    let line = content.replace('\n', " ");
    // Laid out straight first, from the left, with its baseline through 0.
    let straight = TextStyle { align: TextAlign::Left, ..style.clone() };
    let Some(font) = styled_font(style) else { return Vec::new() };
    let scaled = font.as_scaled(em_scale(font, size));
    let baseline = (size * LINE_HEIGHT - (scaled.ascent() - scaled.descent())) / 2.0 + scaled.ascent();
    let glyphs = glyph_outlines(&line, size, Vec2 { x: 0.0, y: -baseline }, &straight);
    // How long the path is up to each of its points.
    let mut lengths = vec![0.0f32];
    for pair in path.windows(2) {
        lengths.push(lengths[lengths.len() - 1] + (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y));
    }
    let total = *lengths.last().unwrap_or(&0.0);
    if path.len() < 2 || total <= 0.0 {
        return glyphs;
    }
    // Where a distance along the path falls, and which way the path runs there. Past its ends it goes on straight.
    let along = |distance: f32| -> (Vec2, f32, f32) {
        let piece = lengths.windows(2).position(|pair| distance <= pair[1]).unwrap_or(path.len() - 2);
        let piece = if distance < 0.0 { 0 } else { piece };
        let (a, b) = (path[piece], path[piece + 1]);
        let length = (lengths[piece + 1] - lengths[piece]).max(0.001);
        let (dx, dy) = ((b.x - a.x) / length, (b.y - a.y) / length);
        let part = distance - lengths[piece];
        (Vec2 { x: a.x + dx * part, y: a.y + dy * part }, dx, dy)
    };
    let width = line_width(&line, size, style);
    let start = style.path_offset
        + match style.align {
            TextAlign::Center => (total - width) / 2.0,
            TextAlign::Right => total - width,
            _ => 0.0,
        };
    // Justified text is spread from one end of the path to the other; stretched text has its letters widened as well.
    let spread = if matches!(style.align, TextAlign::Justify | TextAlign::Stretch) && width > 0.0 { total / width } else { 1.0 };
    let widen = if style.align == TextAlign::Stretch { spread } else { 1.0 };
    glyphs
        .into_iter()
        .map(|rings| {
            let (low, high) = rings.iter().flatten().fold((f32::MAX, f32::MIN), |(low, high), p| (low.min(p.x), high.max(p.x)));
            let middle = (low + high) / 2.0;
            let (at, dx, dy) = along(start + middle * spread);
            rings
                .into_iter()
                .map(|ring| {
                    ring.into_iter()
                        .map(|p| {
                            // The letter's own place: across from its middle, and up from the baseline (down is positive).
                            let (u, v) = ((p.x - middle) * widen, p.y);
                            match style.on_path {
                                PathOrientation::Rotate => Vec2 { x: at.x + u * dx - v * dy, y: at.y + u * dy + v * dx },
                                PathOrientation::Vertical => Vec2 { x: at.x + u, y: at.y + v },
                                PathOrientation::SkewVertical => Vec2 { x: at.x + u * dx, y: at.y + u * dy + v },
                                PathOrientation::SkewHorizontal => Vec2 { x: at.x + u - v * dy, y: at.y + v * dx },
                            }
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// Whether text in this style has to be drawn from the outlines of its letters:
/// spread or stretched lines, and bold or italic made up for a font that has none.
pub fn needs_outlines(style: &TextStyle) -> bool {
    let (heavy, slanted) = crate::fonts::made_up(&style.font, style.bold, style.italic);
    matches!(style.align, TextAlign::Justify | TextAlign::Stretch) || heavy || slanted || style.orientation != TextOrientation::Horizontal
}

/// How far down a letter of vertical text is from the one above it: the height of the font.
fn vertical_step(size: f32, style: &TextStyle) -> f32 {
    match styled_font(style) {
        Some(f) => {
            let scaled = f.as_scaled(em_scale(f, size));
            scaled.ascent() - scaled.descent()
        }
        None => size * LINE_HEIGHT,
    }
}

/// Where the caret stands after the last letter of a text laid out in `frame`.
pub fn caret_place(content: &str, size: f32, style: &TextStyle, frame: Rect) -> Vec2 {
    let last = content.rsplit('\n').next().unwrap_or("");
    let rows = content.split('\n').count();
    let line_height = size * LINE_HEIGHT;
    if style.orientation.is_vertical() {
        let step = vertical_step(size, style);
        let own = last.chars().count() as f32 * step;
        let top = match style.align {
            TextAlign::Center => ((frame.h - own) / 2.0).max(0.0),
            TextAlign::Right => (frame.h - own).max(0.0),
            _ => 0.0,
        };
        // The last column is the left-most when the columns run to the left.
        let left = if style.orientation == TextOrientation::VerticalRtl { frame.x } else { frame.x + (rows - 1) as f32 * line_height };
        return Vec2 { x: left + line_height / 2.0, y: frame.y + top + own };
    }
    let width = if last.is_empty() { 0.0 } else { line_width(last, size, style) };
    let start = line_start(frame.w, width, style.align);
    // Text that runs from the right grows at its left end.
    let across = if style.orientation == TextOrientation::HorizontalRtl { start } else { start + width };
    Vec2 { x: frame.x + across, y: frame.y + (rows as f32 - 1.0) * line_height }
}

/// Whether bold has to be made up for this style, by thickening the letters.
pub fn made_up_bold(style: &TextStyle) -> bool {
    crate::fonts::made_up(&style.font, style.bold, style.italic).0
}

/// How a line is fitted to its box: how much wider each space is made, and how much every letter is stretched across.
fn line_fit(line: &str, width: f32, box_width: f32, last: bool, align: TextAlign) -> (f32, f32) {
    let spaces = line.chars().filter(|c| *c == ' ').count();
    match align {
        TextAlign::Justify if !last && spaces > 0 && width < box_width => ((box_width - width) / spaces as f32, 1.0),
        TextAlign::Stretch if width > 0.0 => (0.0, box_width / width),
        _ => (0.0, 1.0),
    }
}

/// Returns the width and height of multi-line text at `size` pixels, in a style.
pub fn measure_styled(content: &str, size: f32, style: &TextStyle) -> (f32, f32) {
    let line_count = content.split('\n').count() as f32;
    if style.orientation.is_vertical() {
        // Each line is a column as wide as a line is high; the box is as tall as the longest column.
        let longest = content.split('\n').map(|line| line.chars().count()).max().unwrap_or(0) as f32;
        return (line_count * size * LINE_HEIGHT, (longest * vertical_step(size, style)).max(size * 0.5));
    }
    let width = content.split('\n').map(|line| line_width(line, size, style)).fold(0.0, f32::max);
    (width.max(size * 0.5), line_count * size * LINE_HEIGHT)
}

/// The box of the line under underlined text: left, top, width and height, for a line of text starting at `left` on a baseline.
fn underline_box(left: f32, baseline: f32, width: f32, size: f32) -> (f32, f32, f32, f32) {
    let thick = (size / 14.0).max(1.0);
    (left, baseline + thick, width, thick)
}

/// Converts an em size, as used by the canvas, to the font's pixel-height scale.
fn em_scale(font: &FontVec, size: f32) -> PxScale {
    match font.units_per_em() {
        Some(units) => PxScale::from(size * font.height_unscaled() / units),
        None => PxScale::from(size),
    }
}

/// Returns the width and height of multi-line text at `size` pixels.
pub fn measure_text(content: &str, size: f32) -> (f32, f32) {
    let line_count = content.split('\n').count() as f32;
    let width = content
        .split('\n')
        .map(|line| match font() {
            Some(f) => {
                let scaled = f.as_scaled(em_scale(f, size));
                line.chars().map(|c| scaled.h_advance(scaled.glyph_id(c))).sum::<f32>()
            }
            None => line.chars().count() as f32 * size * 0.55,
        })
        .fold(0.0, f32::max);
    (width.max(size * 0.5), line_count * size * LINE_HEIGHT)
}

fn sk_paint(color: Color, opacity: f32) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(color.r, color.g, color.b, (color.a as f32 * opacity) as u8);
    paint
}

/// The outline of every character that draws something, as closed rings in document
/// space. Text is laid out exactly as [`draw_text`] paints it.
pub fn glyph_outlines(content: &str, size: f32, origin: Vec2, style: &TextStyle) -> Vec<Vec<Vec<Vec2>>> {
    let Some(font) = styled_font(style) else { return Vec::new() };
    let box_width = measure_styled(content, size, style).0;
    let scale = em_scale(font, size);
    let scaled = font.as_scaled(scale);
    let line_height = size * LINE_HEIGHT;
    let lead = (line_height - (scaled.ascent() - scaled.descent())) / 2.0;
    let mut glyphs = Vec::new();
    // An italic the font does not have is made by leaning the letters over.
    let lean = if crate::fonts::made_up(&style.font, style.bold, style.italic).1 { 0.2 } else { 0.0 };
    let rows = content.split('\n').count();
    if style.orientation.is_vertical() {
        // Letters stand upright one under another; a line of the text is a column, and
        // the alignments work down the column as they work along a line.
        let (_, box_height) = measure_styled(content, size, style);
        let step = vertical_step(size, style);
        for (column, line) in content.split('\n').enumerate() {
            let count = line.chars().count();
            let own = count as f32 * step;
            let last = column + 1 == rows;
            let (gap, stretch) = match style.align {
                TextAlign::Justify if !last && count > 1 && own < box_height => ((box_height - own) / (count - 1) as f32, 1.0),
                TextAlign::Stretch if own > 0.0 => (0.0, box_height / own),
                _ => (0.0, 1.0),
            };
            let top = origin.y
                + match style.align {
                    TextAlign::Center => ((box_height - own) / 2.0).max(0.0),
                    TextAlign::Right => (box_height - own).max(0.0),
                    _ => 0.0,
                };
            let place_of = if style.orientation == TextOrientation::VerticalRtl { rows - 1 - column } else { column };
            let left = origin.x + place_of as f32 * line_height;
            for (index, ch) in line.chars().enumerate() {
                let id = scaled.glyph_id(ch);
                let across = left + (line_height - scaled.h_advance(id)) / 2.0;
                let baseline = top + index as f32 * (step * stretch + gap) + scaled.ascent() * stretch;
                let place = |p: &ab_glyph::Point| {
                    let up = p.y * scaled.v_scale_factor();
                    Vec2 { x: across + p.x * scaled.h_scale_factor() + up * lean, y: baseline - up * stretch }
                };
                let rings = glyph_rings(font, id, place);
                if !rings.is_empty() {
                    glyphs.push(rings);
                }
            }
        }
        return glyphs;
    }
    for (row, line) in content.split('\n').enumerate() {
        // Text that runs from the right is set with its letters in the opposite order.
        let turned: String;
        let line = if style.orientation == TextOrientation::HorizontalRtl {
            turned = line.chars().rev().collect();
            turned.as_str()
        } else {
            line
        };
        let width = line_width(line, size, style);
        let (wider, stretch) = line_fit(line, width, box_width, row + 1 == rows, style.align);
        let start = origin.x + line_start(box_width, width, style.align);
        // The caret counts in the letters' own widths; a stretched line is widened when its letters are placed.
        let mut caret = 0.0;
        let baseline = origin.y + row as f32 * line_height + lead + scaled.ascent();
        if style.underline && !line.is_empty() {
            let reach = if stretch != 1.0 || wider > 0.0 { box_width } else { width };
            let (x, y, w, h) = underline_box(start, baseline, reach, size);
            glyphs.push(vec![vec![Vec2 { x, y }, Vec2 { x: x + w, y }, Vec2 { x: x + w, y: y + h }, Vec2 { x, y: y + h }]]);
        }
        for ch in line.chars() {
            let id = scaled.glyph_id(ch);
            // Font outlines have y pointing up.
            let place = |p: &ab_glyph::Point| {
                let up = p.y * scaled.v_scale_factor();
                Vec2 { x: start + (caret + p.x * scaled.h_scale_factor() + up * lean) * stretch, y: baseline - up }
            };
            let rings = glyph_rings(font, id, place);
            if !rings.is_empty() {
                glyphs.push(rings);
            }
            caret += scaled.h_advance(id) + if ch == ' ' { wider } else { 0.0 };
        }
    }
    glyphs
}

/// The outline of one letter as closed rings, each point put where `place` says.
fn glyph_rings(font: &FontVec, id: ab_glyph::GlyphId, place: impl Fn(&ab_glyph::Point) -> Vec2) -> Vec<Vec<Vec2>> {
    use ab_glyph::OutlineCurve;
    let far = |a: Vec2, b: Vec2| (a.x - b.x).abs() + (a.y - b.y).abs() > 0.01;
    let Some(outline) = font.outline(id) else { return Vec::new() };
    let mut rings: Vec<Vec<Vec2>> = Vec::new();
    for curve in &outline.curves {
        let controls: Vec<Vec2> = match curve {
            OutlineCurve::Line(a, b) => vec![place(a), place(b)],
            OutlineCurve::Quad(a, b, c) => vec![place(a), place(b), place(c)],
            OutlineCurve::Cubic(a, b, c, d) => vec![place(a), place(b), place(c), place(d)],
        };
        let (start, end) = (controls[0], controls[controls.len() - 1]);
        // A curve that does not start where the last one ended opens a new ring.
        if rings.last().and_then(|ring| ring.last()).is_none_or(|last| far(*last, start)) {
            rings.push(vec![start]);
        }
        let Some(ring) = rings.last_mut() else { continue };
        let length: f32 = controls.windows(2).map(|w| ((w[0].x - w[1].x).powi(2) + (w[0].y - w[1].y).powi(2)).sqrt()).sum();
        let steps = if controls.len() == 2 { 1 } else { (length.ceil() as usize).clamp(2, 16) };
        for step in 1..steps {
            let t = step as f32 / steps as f32;
            // De Casteljau: works for a quadratic and a cubic alike.
            let mut level = controls.clone();
            while level.len() > 1 {
                level = level.windows(2).map(|w| Vec2 { x: w[0].x + (w[1].x - w[0].x) * t, y: w[0].y + (w[1].y - w[0].y) * t }).collect();
            }
            ring.push(level[0]);
        }
        ring.push(end);
    }
    for ring in &mut rings {
        if ring.len() > 1 && !far(ring[0], ring[ring.len() - 1]) {
            ring.pop();
        }
    }
    rings.retain(|ring| ring.len() >= 3);
    rings
}

fn sk_path(points: &[Vec2], closed: bool) -> Option<tiny_skia::Path> {
    let (first, rest) = points.split_first()?;
    let mut builder = PathBuilder::new();
    builder.move_to(first.x, first.y);
    for p in rest {
        builder.line_to(p.x, p.y);
    }
    if closed {
        builder.close();
    }
    builder.finish()
}

fn blend_pixel(pixmap: &mut Pixmap, x: i32, y: i32, color: Color, coverage: f32) {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    if x < 0 || y < 0 || x >= w || y >= h {
        return;
    }
    let alpha = (color.a as f32 / 255.0 * coverage).clamp(0.0, 1.0);
    let index = (y * w + x) as usize;
    let dst = pixmap.pixels()[index];
    let over = |src: u8, dst: u8| (src as f32 * alpha + dst as f32 * (1.0 - alpha)).round() as u8;
    let a = (255.0 * alpha + dst.alpha() as f32 * (1.0 - alpha)).round() as u8;
    let min = |c: u8| c.min(a);
    if let Some(px) = PremultipliedColorU8::from_rgba(
        min(over(color.r, dst.red())),
        min(over(color.g, dst.green())),
        min(over(color.b, dst.blue())),
        a,
    ) {
        pixmap.pixels_mut()[index] = px;
    }
}

fn draw_text(pixmap: &mut Pixmap, object: &Object, content: &str, size: f32, style: &TextStyle) {
    let Some(font) = styled_font(style) else { return };
    let Some(fill) = object.fill else { return };
    let color = Color { a: (fill.a as f32 * object.opacity) as u8, ..fill };
    let scale = em_scale(font, size);
    let scaled = font.as_scaled(scale);
    let line_height = size * LINE_HEIGHT;
    let lead = (line_height - (scaled.ascent() - scaled.descent())) / 2.0;
    for (row, line) in content.split('\n').enumerate() {
        let width = line_width(line, size, style);
        let mut caret = object.bounds.x + line_start(object.bounds.w, width, style.align);
        let baseline = object.bounds.y + row as f32 * line_height + lead + scaled.ascent();
        if style.underline && !line.is_empty() {
            let (x, y, w, h) = underline_box(caret, baseline, width, size);
            for py in y.round() as i32..(y + h).round().max(y.round() + 1.0) as i32 {
                for px in x.round() as i32..(x + w).round() as i32 {
                    blend_pixel(pixmap, px, py, color, 1.0);
                }
            }
        }
        for ch in line.chars() {
            let id = scaled.glyph_id(ch);
            let glyph = id.with_scale_and_position(scale, ab_glyph::point(caret, baseline));
            if let Some(outlined) = font.outline_glyph(glyph) {
                let b = outlined.px_bounds();
                outlined.draw(|gx, gy, coverage| {
                    blend_pixel(pixmap, b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32, color, coverage);
                });
            }
            caret += scaled.h_advance(id);
        }
    }
}

fn draw_bitmap(target: &mut Pixmap, object: &Object, bitmap: &Bitmap) {
    let Some(mut source) = Pixmap::new(bitmap.width(), bitmap.height()) else { return };
    for (dst, px) in source.pixels_mut().iter_mut().zip(bitmap.rgba().chunks_exact(4)) {
        *dst = ColorU8::from_rgba(px[0], px[1], px[2], px[3]).premultiply();
    }
    let b = object.bounds;
    let (sx, sy) = (b.w / bitmap.width() as f32, b.h / bitmap.height() as f32);
    let quality = if sx == 1.0 && sy == 1.0 && b.x.fract() == 0.0 && b.y.fract() == 0.0 {
        FilterQuality::Nearest
    } else {
        FilterQuality::Bilinear
    };
    let paint = PixmapPaint { opacity: object.opacity, blend_mode: BlendMode::SourceOver, quality };
    target.draw_pixmap(0, 0, source.as_ref(), &paint, Transform::from_row(sx, 0.0, 0.0, sy, b.x, b.y), None);
}

/// How much of each pixel a mask lets through, from 0 to 1, for a page of the given size.
fn mask_coverage(mask: &ObjectMask, width: u32, height: u32) -> Option<Vec<f32>> {
    let mut drawn = Pixmap::new(width, height)?;
    let mut shape = mask.object.clone();
    shape.visible = true;
    shape.mask = None;
    shape.blend = crate::blend::BlendMode::Normal;
    if mask.kind == MaskKind::PathOutline {
        // Only the outline counts: whatever the mask is filled or stroked with, its inside shows.
        // A mask made of several objects shows what is inside any of them.
        for part in shape.leaves() {
            if let Some(path) = sk_path(&part.outline(), true) {
                drawn.fill_path(&path, &sk_paint(Color::WHITE, 1.0), FillRule::EvenOdd, Transform::identity(), None);
            }
        }
        return Some(drawn.pixels().iter().map(|px| px.alpha() as f32 / 255.0).collect());
    }
    shape.opacity = 1.0;
    draw_object(&mut drawn, &shape);
    Some(
        drawn
            .pixels()
            .iter()
            .map(|px| match mask.kind {
                MaskKind::Alpha => px.alpha() as f32 / 255.0,
                // Premultiplied channels already carry the alpha, so a clear pixel is black.
                _ => (0.3 * px.red() as f32 + 0.59 * px.green() as f32 + 0.11 * px.blue() as f32) / 255.0,
            })
            .collect(),
    )
}

/// One object drawn on nothing, as a bitmap, with where it sits in the document.
/// Draws several objects together, the first lowest, onto nothing: the image, and where on the page it lies.
pub fn render_objects(objects: &[Object]) -> Option<(Bitmap, Rect)> {
    let area = objects
        .iter()
        .map(|object| {
            let reach = (object.stroke.map_or(0.0, |s| s.width / 2.0) + object.filters.iter().filter(|f| f.enabled).map(crate::effects::reach).sum::<f32>()).ceil();
            let b = object.bounds;
            Rect::from_corners(Vec2 { x: (b.x - reach).floor(), y: (b.y - reach).floor() }, Vec2 { x: (b.right() + reach).ceil(), y: (b.bottom() + reach).ceil() })
        })
        .reduce(Rect::union)?;
    let (width, height) = (area.w.max(1.0) as u32, area.h.max(1.0) as u32);
    let mut pixmap = Pixmap::new(width, height)?;
    for object in objects {
        let mut moved = object.clone();
        moved.translate(-area.x, -area.y);
        draw_object(&mut pixmap, &moved);
    }
    let rgba: Vec<u8> = pixmap.pixels().iter().flat_map(|px| {
        let c = px.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect();
    Some((Bitmap::from_rgba(width, height, rgba)?, Rect { x: area.x, y: area.y, w: width as f32, h: height as f32 }))
}

pub fn render_object(object: &Object) -> Option<(Bitmap, Rect)> {
    let b = object.bounds;
    let reach = (object.stroke.map_or(0.0, |s| s.width / 2.0) + object.filters.iter().filter(|f| f.enabled).map(crate::effects::reach).sum::<f32>()).ceil();
    let (left, top) = ((b.x - reach).floor(), (b.y - reach).floor());
    let (width, height) = (((b.right() + reach).ceil() - left).max(1.0) as u32, ((b.bottom() + reach).ceil() - top).max(1.0) as u32);
    let mut pixmap = Pixmap::new(width, height)?;
    let mut moved = object.clone();
    moved.translate(-left, -top);
    moved.visible = true;
    draw_object(&mut pixmap, &moved);
    let rgba: Vec<u8> = pixmap.pixels().iter().flat_map(|px| {
        let c = px.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect();
    Some((Bitmap::from_rgba(width, height, rgba)?, Rect { x: left, y: top, w: width as f32, h: height as f32 }))
}

fn draw_object(pixmap: &mut Pixmap, object: &Object) {
    // The stand-in for a mask being edited is not artwork.
    if !object.visible || object.is_web() || object.mask_of.is_some() {
        return;
    }
    // An object with a blend mode is drawn on a layer of its own, which is then mixed into the page.
    if !object.blend.is_normal() {
        let Some(mut layer) = Pixmap::new(pixmap.width(), pixmap.height()) else { return };
        let mut plain = object.clone();
        plain.blend = crate::blend::BlendMode::Normal;
        draw_object(&mut layer, &plain);
        crate::blend::composite(pixmap, &layer, object.blend);
        return;
    }
    // Live filters work on the object drawn alone, in the part of the page it can reach.
    if object.filters.iter().any(|filter| filter.enabled) {
        let reach: f32 = object.filters.iter().filter(|f| f.enabled).map(crate::effects::reach).sum::<f32>() + object.stroke.map_or(0.0, |s| s.width) + 2.0;
        let (page_w, page_h) = (pixmap.width() as f32, pixmap.height() as f32);
        let b = object.bounds;
        let (x0, y0) = ((b.x - reach).floor().clamp(0.0, page_w), (b.y - reach).floor().clamp(0.0, page_h));
        let (x1, y1) = ((b.right() + reach).ceil().clamp(0.0, page_w), (b.bottom() + reach).ceil().clamp(0.0, page_h));
        let (width, height) = ((x1 - x0) as u32, (y1 - y0) as u32);
        let Some(mut drawn) = Pixmap::new(width.max(1), height.max(1)).filter(|_| width > 0 && height > 0) else { return };
        let mut bare = object.clone();
        bare.filters.clear();
        bare.translate(-x0, -y0);
        draw_object(&mut drawn, &bare);
        let mut layer = crate::effects::Layer {
            width: width as usize,
            height: height as usize,
            pixels: drawn.pixels().iter().map(|px| {
                let c = px.demultiply();
                [c.red() as f32 / 255.0, c.green() as f32 / 255.0, c.blue() as f32 / 255.0, c.alpha() as f32 / 255.0]
            }).collect(),
        };
        // The shadow and the glow of Photoshop Live Effects are mixed into the page by their own
        // blend modes, under the object, so they are kept on layers of their own. A filter that
        // comes after works on the object and on each of those layers alike.
        use crate::effects::FilterKind;
        let mut under: Vec<(crate::effects::Layer, crate::blend::BlendMode)> = Vec::new();
        for filter in object.filters.iter().filter(|f| f.enabled) {
            if filter.kind == FilterKind::PhotoshopEffects && under.is_empty() {
                (layer, under) = crate::pseffects::apply_apart(&layer, filter);
                continue;
            }
            layer = crate::effects::apply(&layer, filter);
            // A filter that draws around the object draws around the object only, not around its shadow as well.
            let surrounds = matches!(
                filter.kind,
                FilterKind::DropShadow | FilterKind::SolidShadow | FilterKind::Glow | FilterKind::OuterBevel | FilterKind::RaisedEmboss | FilterKind::InsetEmboss | FilterKind::PhotoshopEffects
            );
            if !surrounds {
                for (part, _) in &mut under {
                    *part = crate::effects::apply(part, filter);
                }
            }
        }
        let fill = |target: &mut Pixmap, layer: &crate::effects::Layer| {
            for (px, value) in target.pixels_mut().iter_mut().zip(&layer.pixels) {
                let alpha = value[3].clamp(0.0, 1.0);
                let a = (alpha * 255.0).round() as u8;
                let channel = |v: f32| ((v.clamp(0.0, 1.0) * alpha * 255.0).round() as u8).min(a);
                *px = PremultipliedColorU8::from_rgba(channel(value[0]), channel(value[1]), channel(value[2]), a).unwrap_or(PremultipliedColorU8::TRANSPARENT);
            }
        };
        for (part, mode) in &under {
            fill(&mut drawn, part);
            if mode.is_normal() {
                pixmap.draw_pixmap(x0 as i32, y0 as i32, drawn.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            } else if let Some(mut whole) = Pixmap::new(pixmap.width(), pixmap.height()) {
                whole.draw_pixmap(x0 as i32, y0 as i32, drawn.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                crate::blend::composite(pixmap, &whole, *mode);
            }
        }
        fill(&mut drawn, &layer);
        pixmap.draw_pixmap(x0 as i32, y0 as i32, drawn.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        return;
    }
    // A masked object is drawn on a layer of its own, and the mask takes away what it hides.
    if let Some(mask) = object.mask.as_ref().filter(|mask| mask.enabled) {
        let (Some(mut layer), Some(coverage)) = (Pixmap::new(pixmap.width(), pixmap.height()), mask_coverage(mask, pixmap.width(), pixmap.height())) else {
            return;
        };
        let mut bare = object.clone();
        bare.mask = None;
        draw_object(&mut layer, &bare);
        for (px, shown) in layer.pixels_mut().iter_mut().zip(&coverage) {
            let keep = |c: u8| (c as f32 * shown).round() as u8;
            *px = PremultipliedColorU8::from_rgba(keep(px.red()), keep(px.green()), keep(px.blue()), keep(px.alpha())).unwrap_or(PremultipliedColorU8::TRANSPARENT);
        }
        // A path mask can show itself as well: its fill behind what it masks, its stroke in front.
        let shows_itself = mask.show_fill_stroke && mask.kind == MaskKind::PathOutline;
        let part_of_mask = |fill: bool| {
            let mut part = mask.object.clone();
            if fill { part.stroke = None } else { part.fill = None }
            part.mask = None;
            part
        };
        if shows_itself {
            draw_object(pixmap, &part_of_mask(true));
        }
        pixmap.draw_pixmap(0, 0, layer.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        if shows_itself {
            draw_object(pixmap, &part_of_mask(false));
        }
        return;
    }
    if let Some(paths) = object.placed_text_paths() {
        return draw_object(pixmap, &paths);
    }
    if object.is_instance() {
        for part in object.instance_parts() {
            draw_object(pixmap, &part);
        }
        return;
    }
    if let Shape::Text { content, size, style } = &object.shape {
        // Text filled with a gradient, a pattern or a texture is filled through the outlines of its letters.
        if object.fill_style.is_some() || !object.fill_look.is_plain() {
            if let Some(path) = text_path(object, content, *size, style) {
                draw_styled_fill(pixmap, object, &path);
            }
        } else {
            draw_text(pixmap, object, content, *size, style);
        }
        return;
    }
    if let Shape::Image { bitmap } = &object.shape {
        draw_bitmap(pixmap, object, bitmap);
        return;
    }
    if object.fill.is_some() && (object.fill_style.is_some() || !object.fill_look.is_plain()) {
        if let Some(path) = sk_path(&object.outline(), object.is_closed()) {
            draw_styled_fill(pixmap, object, &path);
        }
    } else if let Some(fill) = object.fill {
        if let Some(path) = sk_path(&object.outline(), object.is_closed()) {
            pixmap.fill_path(&path, &sk_paint(fill, object.opacity), FillRule::EvenOdd, Transform::identity(), None);
        }
    }
    if let Some(stroke) = object.stroke {
        let style = SkStroke {
            width: stroke.width.max(0.1),
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..SkStroke::default()
        };
        // A stroke of uneven weight is drawn as a ribbon: a piece between each two points, and a round joint at each.
        if let Some(weights) = object.stroke_weights() {
            let count = object.points.len();
            let pieces = if object.is_closed() { count } else { count.saturating_sub(1) };
            let half = |index: usize| (stroke.width * weights[index] / 2.0).max(0.0);
            // Drawn on its own first, so where pieces overlap a see-through stroke is not painted twice.
            let Some(mut layer) = Pixmap::new(pixmap.width(), pixmap.height()) else { return };
            let ink = sk_paint(stroke.color, 1.0);
            for (index, point) in object.points.iter().enumerate() {
                if let Some(joint) = PathBuilder::from_circle(point.x, point.y, half(index)).filter(|_| half(index) > 0.05) {
                    layer.fill_path(&joint, &ink, FillRule::Winding, Transform::identity(), None);
                }
            }
            for index in 0..pieces {
                let (a, b) = (object.points[index], object.points[(index + 1) % count]);
                let (dx, dy) = (b.x - a.x, b.y - a.y);
                let length = dx.hypot(dy);
                if length < 0.001 {
                    continue;
                }
                // Across the piece, as far as half the stroke at each end.
                let (nx, ny, wa, wb) = (-dy / length, dx / length, half(index), half((index + 1) % count));
                let mut piece = PathBuilder::new();
                piece.move_to(a.x + nx * wa, a.y + ny * wa);
                piece.line_to(b.x + nx * wb, b.y + ny * wb);
                piece.line_to(b.x - nx * wb, b.y - ny * wb);
                piece.line_to(a.x - nx * wa, a.y - ny * wa);
                piece.close();
                if let Some(piece) = piece.finish() {
                    layer.fill_path(&piece, &ink, FillRule::Winding, Transform::identity(), None);
                }
            }
            let paint = PixmapPaint { opacity: object.opacity, ..PixmapPaint::default() };
            pixmap.draw_pixmap(0, 0, layer.as_ref(), &paint, Transform::identity(), None);
            return;
        }
        for (run, closed) in object.stroke_runs() {
            if let Some(path) = sk_path(&run, closed) {
                pixmap.stroke_path(&path, &sk_paint(stroke.color, object.opacity), &style, Transform::identity(), None);
            }
        }
    }
}

/// Fills an object with its gradient or pattern: its shape is drawn on its own to
/// find how much of each pixel it covers, and each covered pixel gets the color the fill has there.
fn draw_styled_fill(pixmap: &mut Pixmap, object: &Object, path: &tiny_skia::Path) {
    use crate::model::{FillEdge, FillKind};
    let (style, look) = (object.fill_style.as_deref(), object.fill_look);
    let Some(solid) = object.fill else { return };
    let b = object.bounds;
    // The part of the page the shape can touch; a feathered edge reaches further out.
    let room = if look.edge == FillEdge::Feather { (look.feather * 3.0).ceil() as i32 + 1 } else { 1 };
    let (left, top) = ((b.x.floor() as i32 - room).max(0), (b.y.floor() as i32 - room).max(0));
    let (right, bottom) = ((b.right().ceil() as i32 + room).min(pixmap.width() as i32), (b.bottom().ceil() as i32 + room).min(pixmap.height() as i32));
    if right <= left || bottom <= top {
        return;
    }
    let (w, h) = ((right - left) as u32, (bottom - top) as u32);
    let Some(mut shape) = Pixmap::new(w, h) else { return };
    shape.fill_path(path, &sk_paint(Color::BLACK, 1.0), FillRule::EvenOdd, Transform::from_translate(-left as f32, -top as f32), None);
    let drawn: Vec<f32> = shape.pixels().iter().map(|px| px.alpha() as f32 / 255.0).collect();
    let covered: Vec<f32> = match look.edge {
        FillEdge::AntiAlias => drawn.clone(),
        FillEdge::Hard => drawn.iter().map(|c| if *c >= 0.5 { 1.0 } else { 0.0 }).collect(),
        FillEdge::Feather => crate::effects::blur_channel(&drawn, w as usize, h as usize, look.feather.max(0.1)),
    };
    // A contour follows the outline, so it needs to know how deep inside each pixel lies.
    let depth: Vec<f32> = if style.is_some_and(|style| style.kind == FillKind::Contour) {
        let outside: Vec<f32> = drawn.iter().map(|c| if *c < 0.5 { 1.0 } else { 0.0 }).collect();
        let far = crate::pseffects::distances(&outside, w as usize, h as usize);
        // The frame of the box counts as outside too, for a shape that fills it.
        let edge = |at: usize| ((at % w as usize).min(w as usize - 1 - at % w as usize).min(at / w as usize).min(h as usize - 1 - at / w as usize)) as f32 + 1.0;
        let far: Vec<f32> = far.iter().enumerate().map(|(at, d)| d.min(edge(at))).collect();
        let deepest = far.iter().zip(&drawn).filter(|(_, c)| **c >= 0.5).map(|(d, _)| *d).fold(1.0, f32::max);
        far.iter().map(|d| (d / deepest).clamp(0.0, 1.0)).collect()
    } else {
        Vec::new()
    };
    let plain = [solid.r as f32 / 255.0, solid.g as f32 / 255.0, solid.b as f32 / 255.0];
    // The colors of a gradient with its opacity stops worked in.
    let stops = style.map(|style| crate::pseffects::baked(&style.stops, &style.opacity_stops)).unwrap_or_default();
    let texture = (look.texture > 0).then(|| (look.texture as usize - 1, (look.texture_amount / 100.0).clamp(0.0, 1.0)));
    for (at, coverage) in covered.iter().enumerate().filter(|(_, c)| **c > 0.0) {
        let (x, y) = (left + (at as u32 % w) as i32, top + (at as u32 / w) as i32);
        let p = Vec2 { x: x as f32 + 0.5, y: y as f32 + 0.5 };
        let (mut color, opacity) = match style {
            None => (plain, solid.a as f32 / 255.0),
            // A pattern is laid from the corner of the object, so it moves with it.
            Some(style) if style.kind == FillKind::Pattern => {
                let scale = (style.scale / 100.0).max(0.01);
                (crate::pseffects::pattern(style.pattern as usize, (p.x - b.x) / scale, (p.y - b.y) / scale), 1.0)
            }
            // A web dither is its first and its last color, pixel by pixel in turn.
            Some(style) if style.kind == FillKind::WebDither => {
                let odd = (x + y).rem_euclid(2) == 1;
                crate::pseffects::gradient_color(&stops, if odd { 1.0 } else { 0.0 })
            }
            Some(style) => crate::pseffects::gradient_color(&stops, style.place(p, b, depth.get(at).copied().unwrap_or(0.0))),
        };
        // A texture darkens the fill where the texture is dark, as much as its amount says.
        if let Some((which, amount)) = texture {
            let dark = (0.92 - crate::pseffects::pattern(which, p.x - b.x, p.y - b.y)[0]) / 0.57;
            color = color.map(|c| c * (1.0 - dark * amount));
        }
        let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let tone = Color { r: channel(color[0]), g: channel(color[1]), b: channel(color[2]), a: channel(opacity * object.opacity) };
        blend_pixel(pixmap, x, y, tone, coverage.clamp(0.0, 1.0));
    }
}

/// The letters of a text as one path, for filling them with something other than one color.
fn text_path(object: &Object, content: &str, size: f32, style: &TextStyle) -> Option<tiny_skia::Path> {
    let frame = object.placement.map_or(object.bounds, |p| p.frame);
    let mut builder = PathBuilder::new();
    let letters = match object.points.len() {
        0 | 1 => glyph_outlines(content, size, Vec2 { x: frame.x, y: frame.y }, style),
        _ => glyph_outlines_on_path(content, size, style, &object.points),
    };
    for ring in letters.into_iter().flatten() {
        let Some((first, rest)) = ring.split_first() else { continue };
        builder.move_to(first.x, first.y);
        for p in rest {
            builder.line_to(p.x, p.y);
        }
        builder.close();
    }
    builder.finish()
}

pub fn render_page(page: &Page) -> Result<Pixmap> {
    let mut pixmap = Pixmap::new(page.width.max(1), page.height.max(1))
        .ok_or_else(|| anyhow!(tf("canvas size {}x{} is too large", &[&page.width, &page.height])))?;
    if let Some(c) = page.canvas {
        pixmap.fill(tiny_skia::Color::from_rgba8(c.r, c.g, c.b, c.a));
    }
    for layer in page.layers.iter().filter(|l| l.visible) {
        for object in &layer.objects {
            draw_object(&mut pixmap, object);
        }
    }
    Ok(pixmap)
}

/// Writes the page to `path` in the format named by its extension.
pub fn export_image(page: &Page, path: &Path) -> Result<()> {
    save_pixmap(&render_page(page)?, path)
}

/// Writes slice images and a hotspot image map beside an exported image.
/// Returns how many extra files were written.
pub fn export_web(page: &Page, path: &Path) -> Result<usize> {
    let shapes: Vec<(&Object, WebKind, &str)> = page
        .layers
        .iter()
        .filter(|l| l.visible)
        .flat_map(|l| l.objects.iter())
        .filter(|o| o.visible)
        .filter_map(|o| match &o.shape {
            Shape::Web { kind, url, .. } => Some((o, *kind, url.as_str())),
            _ => None,
        })
        .collect();
    let buttons = page.buttons();
    if shapes.is_empty() && buttons.is_empty() {
        return Ok(0);
    }
    let stem = path.file_stem().map_or_else(|| "export".into(), |s| s.to_string_lossy().into_owned());
    let extension = path.extension().map_or_else(|| "png".into(), |e| e.to_string_lossy().into_owned());
    let pixmap = render_page(page)?;
    let mut written = 0;
    for (object, _, _) in shapes.iter().filter(|(_, kind, _)| *kind == WebKind::Slice) {
        let Some(area) = page.rect().intersect(object.bounds) else { continue };
        let crop = IntRect::from_xywh(area.x.round() as i32, area.y.round() as i32, area.w.round().max(1.0) as u32, area.h.round().max(1.0) as u32);
        let Some(mut piece) = crop.and_then(|c| pixmap.clone_rect(c)) else { continue };
        // A many-sided slice keeps only what lies inside its outline; the rest of its box is left clear.
        if let Some(corners) = object.web_outline() {
            let (left, top, columns) = (area.x.round(), area.y.round(), piece.width() as usize);
            for (at, px) in piece.pixels_mut().iter_mut().enumerate() {
                let p = Vec2 { x: left + (at % columns) as f32 + 0.5, y: top + (at / columns) as f32 + 0.5 };
                if !crate::model::point_in_polygon(p, &corners) {
                    *px = PremultipliedColorU8::TRANSPARENT;
                }
            }
        }
        written += 1;
        save_pixmap(&piece, &path.with_file_name(format!("{stem}_slice{written}.{extension}")))?;
    }
    let hotspots: Vec<_> = shapes.iter().filter(|(_, kind, _)| *kind == WebKind::Hotspot).collect();
    let image = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    if !buttons.is_empty() {
        // Each button is cut out of the page once for every look it has, and a web page
        // lays the cut-outs over the image and swaps them as the mouse comes, presses and goes.
        let mut links = String::new();
        for (number, button) in buttons.iter().enumerate() {
            let Some(area) = page.button_area(button.id) else { continue };
            let Some(crop) = IntRect::from_xywh(area.x as i32, area.y as i32, area.w as u32, area.h as u32) else { continue };
            let mut files: [Option<String>; 4] = Default::default();
            for (look, suffix) in ["up", "over", "down", "overdown"].into_iter().enumerate() {
                let has = look == 0 || page.button_looks.iter().any(|b| matches!(button.shape, Shape::Instance { symbol, .. } if symbol == b.symbol) && b.looks.get(look - 1).is_some_and(|l| !l.is_empty()));
                if !has {
                    continue;
                }
                let shown = if look == 0 { pixmap.clone() } else { render_page(&page.with_button_look(button.id, look))? };
                let Some(piece) = shown.clone_rect(crop) else { continue };
                let name = format!("{stem}_button{}_{suffix}.{extension}", number + 1);
                save_pixmap(&piece, &path.with_file_name(&name))?;
                written += 1;
                files[look] = Some(name);
            }
            let Some(up) = files[0].clone() else { continue };
            let over = files[1].clone().unwrap_or_else(|| up.clone());
            let down = files[2].clone().unwrap_or_else(|| over.clone());
            let href = if button.link.is_empty() { "#" } else { button.link.as_str() };
            links.push_str(&format!(
                "  <a href=\"{}\" style=\"position:absolute;left:{}px;top:{}px\"><img src=\"{up}\" width=\"{}\" height=\"{}\" alt=\"{}\" style=\"display:block;border:0\" onmouseover=\"this.src='{over}'\" onmouseout=\"this.src='{up}'\" onmousedown=\"this.src='{down}'\" onmouseup=\"this.src='{over}'\"></a>\n",
                escape_html(href),
                area.x as i32,
                area.y as i32,
                area.w as u32,
                area.h as u32,
                escape_html(&button.name),
            ));
        }
        let map = if hotspots.is_empty() { String::new() } else { map_markup(hotspots.iter().map(|(object, _, url)| (*object, object.name.as_str(), *url))) };
        let html = format!(
            "<!doctype html>\n<meta charset=\"utf-8\">\n<div style=\"position:relative;width:{}px;height:{}px\">\n  <img src=\"{}\" width=\"{}\" height=\"{}\"{} alt=\"\" style=\"display:block\">\n{links}</div>\n{map}",
            page.width,
            page.height,
            escape_html(&image),
            page.width,
            page.height,
            if hotspots.is_empty() { "" } else { " usemap=\"#map\"" },
        );
        let target = path.with_file_name(format!("{stem}.html"));
        std::fs::write(&target, html).with_context(|| tf("could not write {}", &[&target.display()]))?;
        written += 1;
    } else if !hotspots.is_empty() {
        let html = image_map(&image, page, hotspots.iter().map(|(object, _, url)| (*object, object.name.as_str(), *url)));
        let target = path.with_file_name(format!("{stem}_map.html"));
        std::fs::write(&target, html).with_context(|| tf("could not write {}", &[&target.display()]))?;
        written += 1;
    }
    Ok(written)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}

fn image_map<'a>(image: &str, page: &Page, areas: impl Iterator<Item = (&'a Object, &'a str, &'a str)>) -> String {
    format!("<!doctype html>\n<img src=\"{}\" width=\"{}\" height=\"{}\" usemap=\"#map\" alt=\"\">\n{}", escape_html(image), page.width, page.height, map_markup(areas))
}

/// The map that tells a browser where the hotspots of an image are and where each leads.
fn map_markup<'a>(areas: impl Iterator<Item = (&'a Object, &'a str, &'a str)>) -> String {
    let mut html = String::from("<map name=\"map\">\n");
    for (object, name, url) in areas {
        let href = if url.is_empty() { "#" } else { url };
        let b = object.bounds;
        // A hotspot is a rectangle, a circle or a polygon, each with the coordinates HTML asks for.
        let (shape, coords) = match &object.shape {
            Shape::Web { form: crate::model::WebForm::Circle, .. } => {
                ("circle", format!("{},{},{}", b.center().x.round() as i32, b.center().y.round() as i32, (b.w.min(b.h) / 2.0).round() as i32))
            }
            Shape::Web { form: crate::model::WebForm::Polygon(_), .. } => {
                let corners = object.web_outline().unwrap_or_default();
                ("poly", corners.iter().map(|p| format!("{},{}", p.x.round() as i32, p.y.round() as i32)).collect::<Vec<_>>().join(","))
            }
            _ => ("rect", format!("{},{},{},{}", b.x.round() as i32, b.y.round() as i32, b.right().round() as i32, b.bottom().round() as i32)),
        };
        html.push_str(&format!("  <area shape=\"{shape}\" coords=\"{coords}\" href=\"{}\" alt=\"{}\">\n", escape_html(href), escape_html(name)));
    }
    html.push_str("</map>\n");
    html
}

fn save_pixmap(pixmap: &Pixmap, path: &Path) -> Result<()> {
    let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    if extension == "png" {
        return pixmap.save_png(path).with_context(|| tf("could not write {}", &[&path.display()]));
    }
    let mut rgba = Vec::with_capacity(pixmap.data().len());
    for px in pixmap.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    let image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), rgba)
        .ok_or_else(|| anyhow!(t("rendered image has an invalid size")))?;
    let result = if matches!(extension.as_str(), "jpg" | "jpeg") {
        flatten_on_white(&image).save(path)
    } else {
        image.save(path)
    };
    result.with_context(|| tf("could not write {}", &[&path.display()]))
}

/// JPEG has no alpha channel, so transparency is composited over white.
fn flatten_on_white(image: &image::RgbaImage) -> image::RgbImage {
    let (w, h) = image.dimensions();
    image::RgbImage::from_fn(w, h, |x, y| {
        let [r, g, b, a] = image.get_pixel(x, y).0;
        let alpha = a as f32 / 255.0;
        let over = |c: u8| (c as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8;
        image::Rgb([over(r), over(g), over(b)])
    })
}

#[cfg(test)]
mod button_tests {
    use super::*;
    use crate::model::{Document, Editor, SymbolKind};

    #[test]
    fn a_button_is_exported_with_a_picture_for_each_look_and_a_page_that_swaps_them() {
        let mut e = Editor::new(Document::new(100, 60));
        let symbol = e.new_symbol("Go", SymbolKind::Button);
        assert!(e.edit_symbol(symbol, None, true));
        let red = crate::model::Color { r: 255, g: 0, b: 0, a: 255 };
        let green = crate::model::Color { r: 0, g: 255, b: 0, a: 255 };
        let mut up = Object::new_web(1, crate::model::WebKind::Slice, Rect { x: 10.0, y: 10.0, w: 40.0, h: 20.0 });
        (up.shape, up.fill, up.stroke) = (Shape::Rect { radius: 0.0 }, Some(red), None);
        e.add_object(up);
        assert!(e.set_symbol_look(1) && e.copy_previous_look());
        e.for_selected(|o| o.fill = Some(green));
        assert!(e.leave_symbol());
        e.for_selected(|o| o.link = "a.html?x=1&y=2".into());
        let page = e.export_page();
        assert_eq!(page.buttons().len(), 1);
        let id = page.buttons()[0].id;
        assert_eq!(page.button_area(id), Some(Rect { x: 10.0, y: 10.0, w: 40.0, h: 20.0 }));
        // The Over look is drawn where the button is.
        let over = render_page(&page.with_button_look(id, 1)).unwrap();
        let at = over.pixel(30, 20).unwrap();
        assert_eq!((at.red(), at.green()), (0, 255));
        // A look the button does not have leaves it showing the Up look.
        let down = render_page(&page.with_button_look(id, 2)).unwrap();
        assert_eq!(down.pixel(30, 20).unwrap().red(), 255);
        let dir = std::env::temp_dir().join(format!("pisiworks-buttons-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("menu.png");
        assert_eq!(export_web(&page, &path).unwrap(), 3);
        let html = std::fs::read_to_string(dir.join("menu.html")).unwrap();
        assert!(html.contains("href=\"a.html?x=1&amp;y=2\""), "{html}");
        assert!(html.contains("left:10px;top:10px") && html.contains("onmouseover=\"this.src='menu_button1_over.png'\""), "{html}");
        // With no Down look, a press shows Over.
        assert!(html.contains("onmousedown=\"this.src='menu_button1_over.png'\""));
        let piece = image::open(dir.join("menu_button1_over.png")).unwrap().to_rgba8();
        assert_eq!((piece.width(), piece.height(), piece.get_pixel(5, 5).0), (40, 20, [0, 255, 0, 255]));
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Rect, Stroke};

    fn page() -> Page {
        Page::new("t", 40, 40)
    }

    fn rgba(p: &Pixmap, x: u32, y: u32) -> (u8, u8, u8, u8) {
        let c = p.pixel(x, y).unwrap().demultiply();
        (c.red(), c.green(), c.blue(), c.alpha())
    }

    fn red_box(x: f32, w: f32) -> Object {
        Object {
            id: 1,
            name: "box".into(),
            shape: Shape::Rect { radius: 0.0 },
            bounds: Rect { x, y: 0.0, w, h: 40.0 },
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
            fill: Some(Color::rgb(255, 0, 0)),
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    #[test]
    fn fills_canvas_and_object() {
        let mut p = page();
        p.layers[0].objects.push(red_box(0.0, 20.0));
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 5, 20), (255, 0, 0, 255));
        assert_eq!(rgba(&img, 30, 20), (255, 255, 255, 255));
    }

    #[test]
    fn transparent_canvas_stays_transparent() {
        let mut p = page();
        p.canvas = None;
        assert_eq!(rgba(&render_page(&p).unwrap(), 10, 10).3, 0);
    }

    #[test]
    fn hidden_layers_and_objects_are_not_exported() {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.visible = false;
        p.layers[0].objects.push(o);
        assert_eq!(rgba(&render_page(&p).unwrap(), 5, 5), (255, 255, 255, 255));
        p.layers[0].objects[0].visible = true;
        p.layers[0].visible = false;
        assert_eq!(rgba(&render_page(&p).unwrap(), 5, 5), (255, 255, 255, 255));
    }

    #[test]
    fn opacity_blends_with_canvas() {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.opacity = 0.5;
        p.layers[0].objects.push(o);
        let (r, g, _, a) = rgba(&render_page(&p).unwrap(), 20, 20);
        assert_eq!((r, a), (255, 255));
        assert!((120..=135).contains(&g), "g was {g}");
    }

    #[test]
    fn stroke_is_painted_on_open_path() {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.shape = Shape::Line;
        o.fill = None;
        o.points = vec![crate::model::v2(0.0, 20.0), crate::model::v2(40.0, 20.0)];
        o.stroke = Some(Stroke { color: Color::BLACK, width: 4.0 });
        p.layers[0].objects.push(o);
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 20, 20), (0, 0, 0, 255));
        assert_eq!(rgba(&img, 20, 5), (255, 255, 255, 255));
    }

    #[test]
    fn a_scrubbed_stroke_is_heavier_where_it_was_scrubbed() {
        let mut p = page();
        p.canvas = None;
        let mut line = red_box(0.0, 40.0);
        line.shape = Shape::Line;
        line.fill = None;
        line.points = vec![Vec2 { x: 2.0, y: 20.0 }, Vec2 { x: 38.0, y: 20.0 }];
        line.stroke = Some(Stroke { color: Color::rgb(0, 0, 0), width: 4.0 });
        assert!(line.stroke_weights().is_none());
        assert!(line.prepare_scrub());
        assert_eq!(line.stroke_weights().map(|w| w.len()), Some(line.points.len()));
        // Thicker on the left, thinner on the right, untouched in the middle.
        line.scrub(Vec2 { x: 8.0, y: 20.0 }, 6.0, 1.0);
        line.scrub(Vec2 { x: 32.0, y: 20.0 }, 6.0, -0.9);
        p.layers[0].objects.push(line.clone());
        let image = render_page(&p).unwrap();
        let tall = |x: u32| (0..40).filter(|y| rgba(&image, x, *y).3 > 128).count();
        assert!(tall(8) >= 7 && tall(20) == 4 && tall(32) <= 1, "{} {} {}", tall(8), tall(20), tall(32));
        // The weights are kept in the file, and are let go once the path no longer has a number for each point.
        let back: Object = serde_json::from_str(&serde_json::to_string(&line).unwrap()).unwrap();
        assert_eq!(back.stroke_profile, line.stroke_profile);
        line.points.pop();
        assert!(line.stroke_weights().is_none());
        assert!(!serde_json::to_string(&red_box(0.0, 4.0)).unwrap().contains("stroke_profile"));
    }

    #[test]
    fn a_gradient_or_pattern_fills_an_object_in_place_of_its_color() {
        use crate::model::{FillKind, FillStyle};
        use std::sync::Arc;
        let (black, white) = (Color::rgb(0, 0, 0), Color::rgb(255, 255, 255));
        let filled = |style: FillStyle| {
            let mut p = page();
            p.canvas = None;
            let mut object = red_box(4.0, 32.0);
            object.bounds = Rect { x: 4.0, y: 4.0, w: 32.0, h: 32.0 };
            object.fill_style = Some(Arc::new(style));
            p.layers[0].objects.push(object);
            render_page(&p).unwrap()
        };
        // Linear: black at the left of the object, white at the right, gray in between, nothing outside it.
        let linear = filled(FillStyle::new(FillKind::Linear, black, white));
        let (left, middle, right) = (rgba(&linear, 5, 20), rgba(&linear, 20, 20), rgba(&linear, 35, 20));
        assert!(left.0 < 20 && right.0 > 235 && (110..150).contains(&middle.0), "{left:?} {middle:?} {right:?}");
        assert_eq!((left.3, rgba(&linear, 1, 20).3, rgba(&linear, 20, 38).3), (255, 0, 0));
        assert_eq!(rgba(&linear, 20, 6), rgba(&linear, 20, 34), "the same all the way down");
        // Radial: black in the middle, lighter toward every side.
        let radial = filled(FillStyle::new(FillKind::Radial, black, white));
        assert!(rgba(&radial, 20, 20).0 < 20 && rgba(&radial, 34, 20).0 > 200 && rgba(&radial, 20, 6).0 > 200 && rgba(&radial, 6, 20).0 > 200);
        // Contour: like radial it starts deep inside, but it follows the outline: white along every edge.
        let contour = filled(FillStyle::new(FillKind::Contour, black, white));
        assert!(rgba(&contour, 20, 20).0 < 60 && rgba(&contour, 5, 20).0 > 200 && rgba(&contour, 20, 34).0 > 200 && rgba(&contour, 5, 5).0 > 200, "{:?}", rgba(&contour, 20, 20));
        // A handle moved to the middle makes the gradient half as long.
        let mut short = FillStyle::new(FillKind::Linear, black, white);
        short.set_handle(true, Vec2 { x: 20.0, y: 20.0 }, Rect { x: 4.0, y: 4.0, w: 32.0, h: 32.0 });
        assert_eq!(short.handles(Rect { x: 4.0, y: 4.0, w: 32.0, h: 32.0 }).1, Vec2 { x: 20.0, y: 20.0 });
        assert!(rgba(&filled(short), 21, 20).0 > 235);
        // A stop that is see-through lets what is under the object show.
        let mut fading = FillStyle::new(FillKind::Linear, black, Color { r: 0, g: 0, b: 0, a: 0 });
        fading.stops[0].1 = Color::rgb(255, 0, 0);
        let faded = filled(fading);
        assert!(rgba(&faded, 5, 20).3 > 235 && rgba(&faded, 35, 20).3 < 20);
        // A pattern is two grays, and twice as large at a scale of 200.
        let mut pattern = FillStyle::new(FillKind::Pattern, black, white);
        let small = filled(pattern.clone());
        assert_ne!(rgba(&small, 6, 6), rgba(&small, 14, 6));
        pattern.scale = 200.0;
        let large = filled(pattern);
        assert_eq!(rgba(&large, 6, 6), rgba(&large, 14, 6));
        assert_ne!(rgba(&large, 6, 6), rgba(&large, 22, 6));
        // An Ellipse gradient is half as wide as it is long, until its third handle says otherwise.
        let mut ellipse = FillStyle::new(FillKind::Ellipse, black, white);
        let narrow = filled(ellipse.clone());
        assert!(rgba(&narrow, 20, 12).0 > 235 && rgba(&narrow, 28, 20).0 < 150);
        let frame = Rect { x: 4.0, y: 4.0, w: 32.0, h: 32.0 };
        assert_eq!(ellipse.width_handle(frame), Some(Vec2 { x: 20.0, y: 28.0 }));
        ellipse.set_width_handle(Vec2 { x: 20.0, y: 36.0 }, frame);
        assert!(rgba(&filled(ellipse), 20, 12).0 < 150, "as wide as long, it is round");
        assert!(FillStyle::new(FillKind::Linear, black, white).width_handle(frame).is_none());
        // A web dither is its two colors, pixel by pixel in turn.
        let dither = filled(FillStyle::new(FillKind::WebDither, black, white));
        assert_eq!((rgba(&dither, 10, 10).0, rgba(&dither, 11, 10).0, rgba(&dither, 11, 11).0), (0, 255, 0));
        // The edge: hard has no half-covered pixels, feathered fades out past the shape, and a texture darkens in places.
        use crate::model::{FillEdge, FillLook};
        let looked = |look: FillLook| {
            let mut p = page();
            p.canvas = None;
            let mut object = red_box(4.0, 32.0);
            object.shape = Shape::Ellipse;
            object.bounds = Rect { x: 4.0, y: 4.0, w: 32.0, h: 32.0 };
            object.fill_look = look;
            p.layers[0].objects.push(object);
            render_page(&p).unwrap()
        };
        let hard = looked(FillLook { edge: FillEdge::Hard, ..FillLook::default() });
        assert!(hard.pixels().iter().all(|px| px.alpha() == 0 || px.alpha() == 255));
        assert!(looked(FillLook::default()).pixels().iter().any(|px| px.alpha() > 0 && px.alpha() < 255));
        let soft = looked(FillLook { edge: FillEdge::Feather, feather: 3.0, ..FillLook::default() });
        assert!(rgba(&soft, 20, 20).3 > 250 && (40..215).contains(&rgba(&soft, 4, 20).3) && rgba(&soft, 1, 20).3 > 0, "{:?}", rgba(&soft, 4, 20));
        let rough = looked(FillLook { texture: 1, texture_amount: 100.0, ..FillLook::default() });
        let reds: std::collections::HashSet<u8> = (12..28).map(|x| rgba(&rough, x, 20).0).collect();
        assert!(reds.len() >= 2 && reds.contains(&255), "{reds:?}");
        assert!(FillLook::default().is_plain() && !FillLook { texture: 1, ..FillLook::default() }.is_plain());
        // Text is filled through its letters: red at the left of the word, blue at the right.
        if font().is_some() {
            let mut p = page();
            p.canvas = None;
            let mut text = red_box(2.0, 36.0);
            text.shape = Shape::Text { content: "HHHH".into(), size: 12.0, style: Default::default() };
            text.refit_text();
            text.fill_style = Some(Arc::new(FillStyle::new(FillKind::Linear, Color::rgb(255, 0, 0), Color::rgb(0, 0, 255))));
            let (left, right) = (text.bounds.x as u32 + 1, text.bounds.right() as u32 - 2);
            p.layers[0].objects.push(text);
            let image = render_page(&p).unwrap();
            let inked = |x: u32| (0..40).map(|y| rgba(&image, x, y)).find(|px| px.3 > 200);
            let (first, last) = ((left..left + 6).find_map(inked).unwrap(), (right - 6..right).rev().find_map(inked).unwrap());
            assert!(first.0 > 180 && last.2 > 180, "{first:?} {last:?}");
        }
        // Every kind draws something and stays inside the object.
        for kind in FillKind::ALL {
            let drawn = filled(FillStyle::new(kind, black, white));
            assert!(rgba(&drawn, 20, 20).3 == 255 && rgba(&drawn, 1, 1).3 == 0, "{kind:?}");
        }
        // The fill is kept in the file, and an object without one is written as before.
        let mut object = red_box(0.0, 10.0);
        assert!(!serde_json::to_string(&object).unwrap().contains("fill_style"));
        object.fill_style = Some(Arc::new(FillStyle::new(FillKind::Cone, black, white)));
        let back: Object = serde_json::from_str(&serde_json::to_string(&object).unwrap()).unwrap();
        assert_eq!(back.fill_style, object.fill_style);
    }

    #[test]
    fn text_is_aligned_underlined_and_set_in_other_faces() {
        use crate::model::{TextAlign, TextStyle};
        let plain = TextStyle::default();
        assert!(plain.is_plain());
        // The lines of a text stand in a box as wide as its longest line.
        let (width, height) = measure_styled("HHHH\nH", 20.0, &plain);
        let short = line_width("H", 20.0, &plain);
        assert!(width > short * 3.5 && height == 2.0 * 20.0 * LINE_HEIGHT);
        assert_eq!(measure_styled("HHHH\nH", 20.0, &plain), measure_text("HHHH\nH", 20.0));
        assert_eq!(line_start(width, short, TextAlign::Left), 0.0);
        assert_eq!(line_start(width, short, TextAlign::Center), (width - short) / 2.0);
        assert_eq!(line_start(width, short, TextAlign::Right), width - short);
        assert_eq!(line_start(10.0, 30.0, TextAlign::Right), 0.0, "a line never starts left of its box");
        if font().is_none() {
            return;
        }
        // Right-aligned, the short second line is drawn at the right of the box.
        let left_most = |style: &TextStyle| {
            let glyphs = glyph_outlines("HHHH\nH", 20.0, Vec2 { x: 0.0, y: 0.0 }, style);
            glyphs.last().unwrap().iter().flatten().map(|p| p.x).fold(f32::MAX, f32::min)
        };
        let right = TextStyle { align: TextAlign::Right, ..TextStyle::default() };
        assert!(left_most(&plain) < short && left_most(&right) > width - short - 1.0);
        // Underlined text has a bar under each line, as wide as the line.
        let underlined = TextStyle { underline: true, ..TextStyle::default() };
        let (bare, lined) = (glyph_outlines("HH", 20.0, Vec2 { x: 0.0, y: 0.0 }, &plain), glyph_outlines("HH", 20.0, Vec2 { x: 0.0, y: 0.0 }, &underlined));
        assert_eq!(lined.len(), bare.len() + 1);
        let bar = &lined[0][0];
        assert!((bar[1].x - bar[0].x - line_width("HH", 20.0, &plain)).abs() < 0.01);
        // And it is drawn: more of the page is inked than without it.
        let inked = |style: &TextStyle| {
            let mut p = page();
            let mut text = red_box(2.0, 30.0);
            text.shape = Shape::Text { content: "HH".into(), size: 14.0, style: style.clone() };
            p.layers[0].objects.push(text);
            render_page(&p).unwrap().pixels().iter().filter(|px| px.red() > 200 && px.green() < 60).count()
        };
        assert!(inked(&underlined) > inked(&plain));
        // Justified: the first of two lines is spread to the width of the box by its spaces; the last line is left as it is.
        let widest = |text: &str, style: &TextStyle, row_top: f32| {
            let glyphs = glyph_outlines(text, 20.0, Vec2 { x: 0.0, y: 0.0 }, style);
            glyphs.iter().flatten().flatten().filter(|p| p.y >= row_top && p.y < row_top + 20.0 * LINE_HEIGHT).map(|p| p.x).fold(0.0, f32::max)
        };
        let two_lines = "H H\nHHHHHH\nH H";
        // Vertical text: a line is a column, and the letters stand one under another.
        let down = TextStyle { orientation: TextOrientation::VerticalLtr, ..TextStyle::default() };
        assert!(needs_outlines(&down));
        let (w, h) = measure_styled("HHH\nH", 20.0, &down);
        let (across, _) = measure_styled("HHH\nH", 20.0, &TextStyle::default());
        assert_eq!(w, 2.0 * 20.0 * LINE_HEIGHT);
        assert!(h > across, "three letters down are taller than three across are wide");
        let letters = glyph_outlines("HHH\nH", 20.0, Vec2 { x: 0.0, y: 0.0 }, &down);
        let middle = |glyph: &Vec<Vec<Vec2>>| Rect::of_points(&glyph.concat()).center();
        let places: Vec<Vec2> = letters.iter().map(middle).collect();
        assert_eq!(places.len(), 4);
        assert!((places[0].x - places[1].x).abs() < 0.01 && places[1].y > places[0].y + 15.0 && places[2].y > places[1].y + 15.0);
        assert!(places[3].x > places[0].x + 20.0 && (places[3].y - places[0].y).abs() < 0.01, "the second line is a column to the right");
        // Running to the left, the first column is the right-most.
        let leftward = glyph_outlines("HHH\nH", 20.0, Vec2 { x: 0.0, y: 0.0 }, &TextStyle { orientation: TextOrientation::VerticalRtl, ..TextStyle::default() });
        assert!(middle(&leftward[3]).x + 20.0 < middle(&leftward[0]).x);
        // Bottom alignment puts the short column at the foot of the box.
        let foot = glyph_outlines("HHH\nH", 20.0, Vec2 { x: 0.0, y: 0.0 }, &TextStyle { align: TextAlign::Right, ..down.clone() });
        assert!((middle(&foot[3]).y - middle(&foot[2]).y).abs() < 0.01);
        // The caret stands under the last letter of the last column.
        let caret = caret_place("HHH\nH", 20.0, &down, Rect { x: 0.0, y: 0.0, w, h });
        assert!(caret.x > 20.0 * LINE_HEIGHT && caret.y > 15.0 && caret.y < h / 2.0);
        // From the right, the letters of a line come in the opposite order.
        let rtl = TextStyle { orientation: TextOrientation::HorizontalRtl, ..TextStyle::default() };
        let forward = glyph_outlines("Hi", 20.0, Vec2 { x: 0.0, y: 0.0 }, &TextStyle::default());
        let turned = glyph_outlines("Hi", 20.0, Vec2 { x: 0.0, y: 0.0 }, &rtl);
        let wide = |glyph: &Vec<Vec<Vec2>>| Rect::of_points(&glyph.concat()).w;
        assert!(wide(&forward[0]) > wide(&forward[1]) && wide(&turned[0]) < wide(&turned[1]));
        let justified = TextStyle { align: TextAlign::Justify, ..TextStyle::default() };
        let full = measure_styled(two_lines, 20.0, &plain).0;
        assert!(widest(two_lines, &plain, 0.0) < full * 0.6);
        assert!(widest(two_lines, &justified, 0.0) > full * 0.9, "the first line reaches the right edge");
        assert!(widest(two_lines, &justified, 2.0 * 20.0 * LINE_HEIGHT) < full * 0.6, "the last line does not");
        // Stretched: every line reaches it, by widening the letters themselves.
        let stretched = TextStyle { align: TextAlign::Stretch, ..TextStyle::default() };
        assert!(widest(two_lines, &stretched, 2.0 * 20.0 * LINE_HEIGHT) > full * 0.9);
        assert!(needs_outlines(&justified) && needs_outlines(&stretched) && !needs_outlines(&plain));
        // Along a path that runs straight down, the letters follow one another downward, each turned to stand on it.
        let down = [Vec2 { x: 50.0, y: 0.0 }, Vec2 { x: 50.0, y: 200.0 }];
        let on_path = glyph_outlines_on_path("HHH", 20.0, &plain, &down);
        let middle = |rings: &Vec<Vec<Vec2>>| {
            let all: Vec<&Vec2> = rings.iter().flatten().collect();
            (all.iter().map(|p| p.x).sum::<f32>() / all.len() as f32, all.iter().map(|p| p.y).sum::<f32>() / all.len() as f32)
        };
        let (first, last) = (middle(&on_path[0]), middle(&on_path[2]));
        assert!(last.1 > first.1 + 20.0 && (first.0 - last.0).abs() < 1.0, "{first:?} {last:?}");
        // The letters stand to the right of a path going down: their tops point that way.
        assert!(first.0 > 50.0);
        // Upright instead, each letter keeps its own shape and only its place follows the path.
        let upright = glyph_outlines_on_path("HHH", 20.0, &TextStyle { on_path: crate::model::PathOrientation::Vertical, ..TextStyle::default() }, &down);
        let tall = |rings: &Vec<Vec<Vec2>>| rings.iter().flatten().map(|p| p.y).fold(f32::MIN, f32::max) - rings.iter().flatten().map(|p| p.y).fold(f32::MAX, f32::min);
        let flat_h = tall(&glyph_outlines("H", 20.0, Vec2 { x: 0.0, y: 0.0 }, &plain)[0]);
        assert!((tall(&upright[0]) - flat_h).abs() < 0.5 && (tall(&on_path[0]) - flat_h).abs() > 0.5);
        // An offset starts the text further along, and on a path of one point the text is left straight.
        let later = glyph_outlines_on_path("HHH", 20.0, &TextStyle { path_offset: 40.0, ..TextStyle::default() }, &down);
        // Justified, the letters reach from one end of the path to the other; stretched, they are widened too.
        let long = [Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 400.0, y: 0.0 }];
        let reach = |style: &TextStyle| Rect::of_points(&glyph_outlines_on_path("HHH", 20.0, style, &long).concat().concat());
        let first_wide = |style: &TextStyle| Rect::of_points(&glyph_outlines_on_path("HHH", 20.0, style, &long)[0].concat()).w;
        let (plain_reach, spread_reach, wide_reach) = (reach(&TextStyle::default()), reach(&TextStyle { align: TextAlign::Justify, ..TextStyle::default() }), reach(&TextStyle { align: TextAlign::Stretch, ..TextStyle::default() }));
        assert!(plain_reach.w < 80.0 && spread_reach.w > 250.0 && wide_reach.w > 350.0);
        assert!((first_wide(&TextStyle { align: TextAlign::Justify, ..TextStyle::default() }) - first_wide(&TextStyle::default())).abs() < 0.01);
        assert!(first_wide(&TextStyle { align: TextAlign::Stretch, ..TextStyle::default() }) > first_wide(&TextStyle::default()) * 3.0);
        assert!((middle(&later[0]).1 - first.1 - 40.0).abs() < 0.5);
        assert_eq!(glyph_outlines_on_path("H", 20.0, &plain, &down[..1]).len(), 1);
        // A style that is not plain is kept in the file; plain text is written as before.
        let styled = Shape::Text { content: "a".into(), size: 12.0, style: TextStyle { font: "Some Font".into(), bold: true, ..TextStyle::default() } };
        let back: Shape = serde_json::from_str(&serde_json::to_string(&styled).unwrap()).unwrap();
        assert_eq!(back, styled);
        assert!(!serde_json::to_string(&Shape::Text { content: "a".into(), size: 12.0, style: plain }).unwrap().contains("style"));
        let old: Shape = serde_json::from_str(r#"{"Text":{"content":"a","size":12.0}}"#).unwrap();
        assert!(matches!(old, Shape::Text { style, .. } if style.is_plain()));
        // An unknown family falls back to a font that is there, so text never vanishes.
        assert!(line_width("H", 20.0, &TextStyle { font: "No Such Font 12345".into(), ..TextStyle::default() }) > 0.0);
    }

    #[test]
    fn a_photoshop_shadow_is_mixed_into_the_page_by_its_blend_mode() {
        use crate::blend::BlendMode;
        use crate::effects::{FilterKind, LiveFilter};
        use crate::pseffects::{index, DROP_SHADOW};
        let mode = |mode: BlendMode| BlendMode::ALL.iter().position(|m| *m == mode).unwrap() as f32;
        let mut p = page();
        // A blue box on red, with a hard white shadow thrown down and to the right.
        p.layers[0].objects.push(red_box(0.0, 40.0));
        let mut boxed = red_box(10.0, 12.0);
        boxed.fill = Some(Color::rgb(0, 0, 255));
        let mut effects = LiveFilter::new(FilterKind::PhotoshopEffects);
        effects.set_ps_color(DROP_SHADOW, Color::rgb(255, 255, 255));
        for (slot, value) in [(1, 100.0), (3, 8.0), (4, 0.0)] {
            effects.set_value(index(DROP_SHADOW, slot), value);
        }
        boxed.filters = vec![effects];
        p.layers[0].objects.push(boxed);
        let shadow_at = |p: &Page| rgba(&render_page(p).unwrap(), 24, 27);
        // Multiplied, as a shadow starts out, white leaves the red as it is.
        assert_eq!(shadow_at(&p), (255, 0, 0, 255));
        // Laid on normally it covers the red; screened it lightens it to white as well.
        p.layers[0].objects[1].filters[0].set_value(index(DROP_SHADOW, 5), mode(BlendMode::Normal));
        assert_eq!(shadow_at(&p), (255, 255, 255, 255));
        // A dark green shadow screened onto red gives a dull yellow; multiplied, black.
        p.layers[0].objects[1].filters[0].set_ps_color(DROP_SHADOW, Color::rgb(0, 128, 0));
        p.layers[0].objects[1].filters[0].set_value(index(DROP_SHADOW, 5), mode(BlendMode::Screen));
        assert_eq!(shadow_at(&p), (255, 128, 0, 255));
        p.layers[0].objects[1].filters[0].set_value(index(DROP_SHADOW, 5), mode(BlendMode::Multiply));
        assert_eq!(shadow_at(&p), (0, 0, 0, 255));
        // The box itself is drawn over its shadow, untouched.
        assert_eq!(rgba(&render_page(&p).unwrap(), 15, 15), (0, 0, 255, 255));
        // A filter after it works on the shadow as on the object, and the shadow is still mixed by its mode:
        // inverted, the dark green shadow is a light magenta, which multiplied into red leaves red.
        p.layers[0].objects[1].filters.push(LiveFilter::new(FilterKind::Invert));
        p.layers[0].objects[1].filters[1].enabled = false;
        assert_eq!(shadow_at(&p), (0, 0, 0, 255), "a filter that is off does nothing");
        p.layers[0].objects[1].filters[1].enabled = true;
        assert_eq!(shadow_at(&p), (255, 0, 0, 255));
        assert_eq!(rgba(&render_page(&p).unwrap(), 15, 15), (255, 255, 0, 255), "and the box is inverted too");
        // A Fireworks shadow added after it falls from the object, not from the Photoshop shadow as well.
        p.layers[0].objects[1].filters[1] = LiveFilter::new(FilterKind::DropShadow);
        assert_eq!(rgba(&render_page(&p).unwrap(), 15, 15), (0, 0, 255, 255));
    }

    #[test]
    fn a_blend_mode_mixes_an_object_with_what_is_under_it() {
        use crate::blend::BlendMode;
        let mut p = page();
        // A red box, and over its right half a box of pure green.
        p.layers[0].objects.push(red_box(0.0, 40.0));
        let mut over = red_box(20.0, 20.0);
        over.fill = Some(Color::rgb(0, 255, 0));
        over.blend = BlendMode::Multiply;
        p.layers[0].objects.push(over.clone());
        let img = render_page(&p).unwrap();
        // Red times green is black; outside the green box the red is untouched.
        assert_eq!(rgba(&img, 30, 10), (0, 0, 0, 255));
        assert_eq!(rgba(&img, 10, 10), (255, 0, 0, 255));
        // Screen adds them to yellow, and Normal just covers.
        p.layers[0].objects[1].blend = BlendMode::Screen;
        assert_eq!(rgba(&render_page(&p).unwrap(), 30, 10), (255, 255, 0, 255));
        p.layers[0].objects[1].blend = BlendMode::Normal;
        assert_eq!(rgba(&render_page(&p).unwrap(), 30, 10), (0, 255, 0, 255));
        // Erase cuts through to the canvas.
        p.layers[0].objects[1].blend = BlendMode::Erase;
        p.canvas = None;
        assert_eq!(rgba(&render_page(&p).unwrap(), 30, 10).3, 0);
        // The mode is saved with the object, and a normal object is written as before.
        let text = serde_json::to_string(&over).unwrap();
        assert!(text.contains("Multiply"));
        assert_eq!(serde_json::from_str::<Object>(&text).unwrap().blend, BlendMode::Multiply);
        assert!(!serde_json::to_string(&red_box(0.0, 1.0)).unwrap().contains("blend"));
    }

    /// A red box 40 wide, masked by `mask`.
    fn masked(mask: Object, kind: MaskKind) -> Page {
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.mask = Some(std::sync::Arc::new(ObjectMask { object: mask, kind, show_fill_stroke: false, enabled: true, linked: true }));
        p.layers[0].objects.push(o);
        p
    }

    #[test]
    fn a_path_mask_shows_only_what_is_inside_its_outline() {
        // The mask covers the left half. What it is filled with does not matter.
        let mut left = red_box(0.0, 20.0);
        left.fill = Some(Color::BLACK);
        let mut p = masked(left, MaskKind::PathOutline);
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 10, 10), (255, 0, 0, 255));
        assert_eq!(rgba(&img, 30, 10), (255, 255, 255, 255), "the canvas shows where the mask hides the box");
        // Switched off, the mask hides nothing.
        let object = &mut p.layers[0].objects[0];
        std::sync::Arc::make_mut(object.mask.as_mut().unwrap()).enabled = false;
        assert_eq!(rgba(&render_page(&p).unwrap(), 30, 10), (255, 0, 0, 255));
    }

    #[test]
    fn a_grayscale_mask_shows_more_where_it_is_lighter() {
        let shade = |level: u8| {
            let mut mask = red_box(0.0, 40.0);
            mask.fill = Some(Color::rgb(level, level, level));
            masked(mask, MaskKind::Grayscale)
        };
        assert_eq!(rgba(&render_page(&shade(255)).unwrap(), 10, 10), (255, 0, 0, 255));
        assert_eq!(rgba(&render_page(&shade(0)).unwrap(), 10, 10), (255, 255, 255, 255));
        // Mid gray lets half of the red through, over the white canvas.
        let (r, g, _, _) = rgba(&render_page(&shade(128)).unwrap(), 10, 10);
        assert!(r == 255 && (120..=135).contains(&g), "g was {g}");
        // By its alpha channel, the same black mask shows everything.
        let mut black = red_box(0.0, 40.0);
        black.fill = Some(Color::BLACK);
        assert_eq!(rgba(&render_page(&masked(black, MaskKind::Alpha)).unwrap(), 10, 10), (255, 0, 0, 255));
    }

    #[test]
    fn a_path_mask_can_show_its_own_fill_and_stroke() {
        let mut frame = red_box(0.0, 20.0);
        frame.fill = Some(Color::rgb(0, 0, 255));
        // The masked box is narrower than the mask, so the mask's fill shows beside it.
        let mut p = page();
        let mut inside = red_box(0.0, 10.0);
        inside.mask = Some(std::sync::Arc::new(ObjectMask { object: frame, kind: MaskKind::PathOutline, show_fill_stroke: true, enabled: true, linked: true }));
        p.layers[0].objects.push(inside);
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 5, 10), (255, 0, 0, 255));
        assert_eq!(rgba(&img, 15, 10), (0, 0, 255, 255));
        assert_eq!(rgba(&img, 30, 10), (255, 255, 255, 255));
    }

    #[test]
    fn a_mask_moves_and_stretches_with_its_object() {
        let mut p = masked(red_box(0.0, 20.0), MaskKind::PathOutline);
        let object = &mut p.layers[0].objects[0];
        object.translate(10.0, 0.0);
        assert_eq!(object.mask.as_ref().unwrap().object.bounds.x, 10.0);
        object.set_bounds(Rect { x: 0.0, y: 0.0, w: 80.0, h: 40.0 });
        let mask = object.mask.as_ref().unwrap().object.bounds;
        assert_eq!((mask.x, mask.w), (0.0, 40.0), "twice as wide, and still over the left half of the box");
        // Saved and read back, the mask is still there; an object without one is written as before.
        let text = serde_json::to_string(&*object).unwrap();
        assert_eq!(serde_json::from_str::<Object>(&text).unwrap().mask, object.mask);
        assert!(!serde_json::to_string(&red_box(0.0, 1.0)).unwrap().contains("mask"));
    }

    #[test]
    fn curved_path_is_exported_along_its_curve() {
        use crate::model::{v2, Handles};
        let mut p = page();
        let mut o = red_box(0.0, 40.0);
        o.shape = Shape::Path { closed: false };
        o.fill = None;
        // From (0,30) to (40,30), pulled up so the middle of the curve passes y = 7.5.
        o.points = vec![v2(0.0, 30.0), v2(40.0, 30.0)];
        o.handles = vec![
            Handles { before: v2(0.0, 0.0), after: v2(0.0, -30.0) },
            Handles { before: v2(0.0, -30.0), after: v2(0.0, 0.0) },
        ];
        o.stroke = Some(Stroke { color: Color::BLACK, width: 4.0 });
        p.layers[0].objects.push(o);
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 20, 7), (0, 0, 0, 255));
        // Nothing is drawn on the straight line between the two ends.
        assert_eq!(rgba(&img, 20, 30), (255, 255, 255, 255));
    }

    #[test]
    fn png_file_is_written_with_page_size() {
        let path = std::env::temp_dir().join("emberframe_export_test.png");
        export_image(&page(), &path).unwrap();
        let img = Pixmap::load_png(&path).unwrap();
        assert_eq!((img.width(), img.height()), (40, 40));
        std::fs::remove_file(path).ok();
    }

    fn green_bitmap() -> Object {
        let pixels = [0u8, 255, 0, 255].repeat(20 * 20);
        Object::new_bitmap(9, "g", Bitmap::from_rgba(20, 20, pixels).unwrap(), 10.0, 10.0)
    }

    #[test]
    fn bitmap_is_drawn_at_its_bounds() {
        let mut p = page();
        p.layers[0].objects.push(green_bitmap());
        let img = render_page(&p).unwrap();
        assert_eq!(rgba(&img, 20, 20), (0, 255, 0, 255));
        assert_eq!(rgba(&img, 5, 5), (255, 255, 255, 255));
    }

    #[test]
    fn scaled_bitmap_covers_scaled_bounds() {
        let mut p = page();
        let mut o = green_bitmap();
        o.set_bounds(Rect { x: 0.0, y: 0.0, w: 40.0, h: 40.0 });
        p.layers[0].objects.push(o);
        assert_eq!(rgba(&render_page(&p).unwrap(), 36, 36), (0, 255, 0, 255));
    }

    #[test]
    fn jpeg_export_flattens_transparency_on_white() {
        let path = std::env::temp_dir().join("emberframe_export_test.jpg");
        let mut p = page();
        p.canvas = None;
        export_image(&p, &path).unwrap();
        let decoded = image::open(&path).unwrap().to_rgb8();
        assert!(decoded.get_pixel(20, 20).0.iter().all(|c| *c > 250));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn web_areas_stay_out_of_the_image_but_produce_files() {
        let dir = std::env::temp_dir().join("emberframe_web_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("site.png");
        let mut p = page();
        let before = rgba(&render_page(&p).unwrap(), 5, 5);
        p.layers[0].objects.push(Object::new_web(7, WebKind::Slice, Rect { x: 0.0, y: 0.0, w: 10.0, h: 20.0 }));
        let mut hotspot = Object::new_web(8, WebKind::Hotspot, Rect { x: 5.0, y: 6.0, w: 10.0, h: 10.0 });
        hotspot.shape = Shape::Web { kind: WebKind::Hotspot, url: "https://example.com/?a=1&b=2".into(), form: Default::default() };
        p.layers[0].objects.push(hotspot);
        assert_eq!(rgba(&render_page(&p).unwrap(), 5, 5), before);
        assert_eq!(export_web(&p, &path).unwrap(), 2);
        let slice = Pixmap::load_png(dir.join("site_slice1.png")).unwrap();
        assert_eq!((slice.width(), slice.height()), (10, 20));
        let html = std::fs::read_to_string(dir.join("site_map.html")).unwrap();
        assert!(html.contains("coords=\"5,6,15,16\""));
        assert!(html.contains("a=1&amp;b=2"));
        // A round hotspot and a many-sided one are written as HTML's circle and poly.
        p.layers[0].objects.push(Object::new_web_circle(50, Rect { x: 10.0, y: 10.0, w: 20.0, h: 20.0 }));
        p.layers[0].objects.push(Object::new_web_polygon(51, WebKind::Hotspot, &[Vec2 { x: 1.0, y: 2.0 }, Vec2 { x: 30.0, y: 2.0 }, Vec2 { x: 15.0, y: 25.0 }]));
        export_web(&p, &path).unwrap();
        let html = std::fs::read_to_string(path.with_file_name(format!("{}_map.html", path.file_stem().unwrap().to_string_lossy()))).unwrap();
        assert!(html.contains("shape=\"circle\" coords=\"20,20,10\""), "{html}");
        assert!(html.contains("shape=\"poly\" coords=\"1,2,30,2,15,25\""), "{html}");
        assert!(html.contains("shape=\"rect\" coords=\"5,6,15,16\""));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn filled_open_path_keeps_its_fill() {
        let mut p = page();
        let mut piece = Object::new_web(9, WebKind::Slice, Rect::default());
        piece.shape = Shape::Path { closed: false };
        piece.points = vec![
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 39.0, y: 0.0 },
            Vec2 { x: 39.0, y: 39.0 },
            Vec2 { x: 0.0, y: 39.0 },
        ];
        piece.fill = Some(Color::rgb(0, 255, 0));
        p.layers[0].objects.push(piece);
        assert_eq!(rgba(&render_page(&p).unwrap(), 20, 20), (0, 255, 0, 255));
    }

    #[test]
    fn no_web_areas_means_no_extra_files() {
        assert_eq!(export_web(&page(), Path::new("unused.png")).unwrap(), 0);
    }

    #[test]
    fn unknown_extension_reports_an_error() {
        let path = std::env::temp_dir().join("emberframe_export_test.nope");
        assert!(export_image(&page(), &path).is_err());
    }

    #[test]
    fn text_is_measured_at_em_size() {
        let Some(f) = font() else { return };
        let id = f.glyph_id('H');
        let upem = f.units_per_em().unwrap();
        let expected = 4.0 * f.h_advance_unscaled(id) / upem * 100.0;
        let (width, _) = measure_text("HHHH", 100.0);
        assert!((width - expected).abs() < 0.5, "width {width}, expected {expected}");
    }

    #[test]
    fn measure_grows_with_lines() {
        let (_, one) = measure_text("a", 10.0);
        let (_, two) = measure_text("a\nb", 10.0);
        assert!(two > one);
    }
}
