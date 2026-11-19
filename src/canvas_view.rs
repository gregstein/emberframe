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

//! Paints the document, overlays and rulers.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    fill, font, outline, point, px, rgb, size, App, BorderStyle, Bounds, ContentMask, Corners, Hsla, PathBuilder,
    Pixels, Point, RenderImage, Rgba, SharedString, TextRun, Window,
};
use image::imageops::{self, FilterType};
use image::Frame;

use crate::i18n::{t};
use crate::bitmap::Bitmap;
use crate::export::LINE_HEIGHT;
use crate::menus::Tool;
use crate::model::{flatten_curve, v2, Color, Handles, Object, Page, Rect, Shape, Vec2, WebKind};
use crate::theme;
use crate::workspace::{quad_handles, Drag, PreviewPane, ViewMap, ViewMode, Workspace};

const HANDLE_SIZE: f32 = 7.0;
/// How strongly the JPEG mask is tinted.
const JPEG_MASK_OPACITY: f32 = 0.3;
const CHECKER: f32 = 8.0;
const PANE_GAP: f32 = 2.0;
const PANE_CAPTION_HEIGHT: f32 = 18.0;
const ERASER_PREVIEW: Color = Color::rgb(0xcc, 0xcc, 0xcc);
const LASSO_PREVIEW: Color = Color::rgb(0x00, 0xa8, 0xff);
const DASH: f32 = 4.0;
/// Keeps very fragmented selections from slowing every frame.
const MAX_ANT_DASHES: usize = 40_000;
/// Larger bitmaps are previewed downscaled; edits and export still use every pixel.
const MAX_PREVIEW_SIZE: u32 = 4096;

/// Pixel identity plus baked opacity. Each entry keeps its pixels alive, so a key is never reused.
pub type TextureKey = (usize, u32);
pub type Textures = HashMap<TextureKey, (Arc<Bitmap>, Arc<RenderImage>)>;

pub fn texture_key(object: &Object) -> Option<(TextureKey, &Arc<Bitmap>)> {
    match &object.shape {
        Shape::Image { bitmap } => Some(((Arc::as_ptr(bitmap) as usize, object.opacity.to_bits()), bitmap)),
        _ => None,
    }
}

/// Builds the GPU image: BGRA with straight alpha, with object opacity baked in.
pub fn make_texture(bitmap: &Bitmap, opacity: f32) -> Arc<RenderImage> {
    let longest = bitmap.width().max(bitmap.height());
    let mut buffer = bitmap.to_image();
    if longest > MAX_PREVIEW_SIZE {
        let scale = MAX_PREVIEW_SIZE as f32 / longest as f32;
        let width = ((bitmap.width() as f32 * scale) as u32).max(1);
        let height = ((bitmap.height() as f32 * scale) as u32).max(1);
        buffer = imageops::resize(&buffer, width, height, FilterType::Triangle);
    }
    for px in buffer.chunks_exact_mut(4) {
        px.swap(0, 2);
        px[3] = (px[3] as f32 * opacity).round() as u8;
    }
    Arc::new(RenderImage::new([Frame::new(buffer)]))
}

/// Decodes an encoded image into a GPU image.
/// The GPU image of a rendered page.
pub fn pixmap_texture(pixmap: &tiny_skia::Pixmap) -> Option<Arc<RenderImage>> {
    let mut bgra = Vec::with_capacity(pixmap.pixels().len() * 4);
    for px in pixmap.pixels() {
        let c = px.demultiply();
        bgra.extend_from_slice(&[c.blue(), c.green(), c.red(), c.alpha()]);
    }
    let buffer = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), bgra)?;
    Some(Arc::new(RenderImage::new([Frame::new(buffer)])))
}

pub fn encoded_texture(bytes: &[u8]) -> Option<Arc<RenderImage>> {
    let mut buffer = image::load_from_memory(bytes).ok()?.to_rgba8();
    for px in buffer.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    Some(Arc::new(RenderImage::new([Frame::new(buffer)])))
}

/// Splits the document area into equal panes for the comparison views.
pub fn pane_rects(bounds: Bounds<Pixels>, mode: ViewMode) -> Vec<Bounds<Pixels>> {
    let (columns, rows) = match mode {
        ViewMode::Original | ViewMode::Preview => (1, 1),
        ViewMode::TwoUp => (2, 1),
        ViewMode::FourUp => (2, 2),
    };
    let width = (f32::from(bounds.size.width) - PANE_GAP * (columns - 1) as f32) / columns as f32;
    let height = (f32::from(bounds.size.height) - PANE_GAP * (rows - 1) as f32) / rows as f32;
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .map(|(row, column)| Bounds {
            origin: point(
                bounds.origin.x + px((width + PANE_GAP) * column as f32),
                bounds.origin.y + px((height + PANE_GAP) * row as f32),
            ),
            size: size(px(width), px(height)),
        })
        .collect()
}

fn to_hsla(c: Color, opacity: f32) -> Hsla {
    Rgba {
        r: c.r as f32 / 255.0,
        g: c.g as f32 / 255.0,
        b: c.b as f32 / 255.0,
        a: c.a as f32 / 255.0 * opacity,
    }
    .into()
}

fn screen_rect(map: ViewMap, r: Rect) -> Bounds<Pixels> {
    Bounds { origin: map.to_screen(Vec2 { x: r.x, y: r.y }), size: size(px(r.w * map.zoom), px(r.h * map.zoom)) }
}

/// Screen rectangles of the 8 resize handles, clockwise from top-left.
pub fn handle_rects(bounds: Rect, map: ViewMap) -> [Bounds<Pixels>; 8] {
    let (x0, y0, x1, y1) = (bounds.x, bounds.y, bounds.right(), bounds.bottom());
    let c = bounds.center();
    [(x0, y0), (c.x, y0), (x1, y0), (x1, c.y), (x1, y1), (c.x, y1), (x0, y1), (x0, c.y)].map(|(x, y)| {
        let center = map.to_screen(Vec2 { x, y });
        Bounds {
            origin: point(center.x - px(HANDLE_SIZE / 2.0), center.y - px(HANDLE_SIZE / 2.0)),
            size: size(px(HANDLE_SIZE), px(HANDLE_SIZE)),
        }
    })
}

fn build_path(points: &[Point<Pixels>], closed: bool, stroke_width: Option<f32>) -> Option<gpui::Path<Pixels>> {
    let (first, rest) = points.split_first()?;
    let mut builder = match stroke_width {
        Some(width) => PathBuilder::stroke(px(width)),
        None => PathBuilder::fill(),
    };
    builder.move_to(*first);
    for p in rest {
        builder.line_to(*p);
    }
    if closed {
        builder.close();
    }
    builder.build().ok()
}

/// A frame's snapshot of everything the canvas draws.
pub struct Scene {
    page: Page,
    zoom: f32,
    pan: Vec2,
    selected: Vec<Object>,
    selection_bounds: Option<Rect>,
    marquee: Option<Rect>,
    draft: Vec<Vec2>,
    /// Curve handles to show, each as its vertex and the handle's own position.
    handle_guides: Vec<(Vec2, Vec2)>,
    /// With Subselection or the Pen, every vertex of a selected path is shown, and these are the picked ones.
    picked_points: Option<Vec<(u64, usize)>>,
    draft_closed: bool,
    /// Stroke width of the draft in document units; `None` draws a hairline.
    draft_width: Option<f32>,
    draft_color: Option<Color>,
    crop: Option<Rect>,
    /// Pixel-selection outline, with the object that maps it to the document.
    ants: Option<(Arc<Vec<(Vec2, Vec2)>>, Object)>,
    stamp_mark: Option<Vec2>,
    all_handles: bool,
    /// The page's JPEG mask, tinted so the area kept at the selective quality can be seen.
    jpeg_mask: Option<(Arc<Vec<Rect>>, Color)>,
    /// The box of the Scale, Skew and Distort tools, with the point rotation turns around.
    transform_box: Option<([Vec2; 4], Vec2)>,
    caret: Option<(Vec2, f32)>,
    show_rulers: bool,
    show_grid: bool,
    /// How far apart the grid lines are, and their color.
    grid: ((f32, f32), crate::model::Color),
    /// The lines drawn across the whole view: whether each runs across, where it is on the page, and its color.
    guide_lines: Vec<(bool, f32, u32)>,
    /// The handles of the selected object's gradient: where it starts and where it ends.
    fill_handles: Option<(Vec2, Vec2, Option<Vec2>)>,
    /// The path of the selected animation: where its symbol is in each state.
    motion: Vec<Vec2>,
    /// Shape hints: where each is, its number, and whether it is the end of its pair.
    hints: Vec<(Vec2, usize, bool)>,
    /// The cuts of 9-slice scaling: the box of the selection, and where the two upright and two level cuts are.
    nine_cuts: Option<(Rect, [f32; 4])>,
    stroke: Option<Color>,
    fill: Option<Color>,
    textures: Textures,
    /// Faded neighbouring states, drawn under the page when onion skinning.
    ghosts: Vec<Object>,
    view_mode: ViewMode,
    previews: Vec<PreviewPane>,
    /// The page as the exporter renders it, shown in place of the objects when the
    /// page has blend modes the screen cannot draw one object at a time.
    raster: Option<Arc<RenderImage>>,
}

impl Scene {
    pub fn capture(w: &Workspace) -> Self {
        let mut draft = Vec::new();
        let mut draft_closed = false;
        let mut marquee = None;
        let mut draft_width = None;
        let mut draft_color = None;
        let mut crop = w.crop_area;
        match &w.drag {
            // The Polygon Lasso under way: its corners so far, and a line on to the mouse.
            None if w.polygon_lasso.is_some() || w.polygon_web.is_some() => {
                let corners = w.polygon_lasso.as_ref().map(|(_, points, _)| points).or(w.polygon_web.as_ref());
                if let Some(points) = corners {
                    draft = points.iter().copied().chain([w.cursor_doc]).collect();
                    draft_color = Some(LASSO_PREVIEW);
                }
            }
            Some(Drag::Marquee { start, current }) => marquee = Some(Rect::from_corners(*start, *current)),
            Some(Drag::Crop { start, current }) => {
                crop = w.ed.doc.page().rect().intersect(Rect::from_corners(*start, *current));
            }
            Some(Drag::PixelArea { points, .. }) => {
                if matches!(w.tool, Tool::Marquee | Tool::RedEye) {
                    marquee = points.get(1).map(|end| Rect::from_corners(points[0], *end));
                } else if w.tool == Tool::OvalMarquee {
                    // The oval the drag spans, closed back on itself.
                    if let Some(end) = points.get(1) {
                        let (a, b) = (points[0], *end);
                        let (cx, cy, rx, ry) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.x - b.x).abs() / 2.0, (a.y - b.y).abs() / 2.0);
                        draft = (0..=64).map(|step| step as f32 / 64.0 * std::f32::consts::TAU).map(|turn| Vec2 { x: cx + rx * turn.cos(), y: cy + ry * turn.sin() }).collect();
                        draft_color = Some(LASSO_PREVIEW);
                    }
                } else {
                    draft = points.clone();
                    draft_color = Some(LASSO_PREVIEW);
                }
            }
            Some(Drag::Create { start, current }) => {
                let r = Rect::from_corners(*start, *current);
                match w.tool {
                    Tool::Hotspot | Tool::Slice => marquee = Some(r),
                    // Drawn from their middle outward.
                    Tool::CircleHotspot | Tool::Polygon => {
                        let radius = (current.x - start.x).hypot(current.y - start.y);
                        draft_closed = true;
                        draft = Object::outline(&preview(Shape::Ellipse, Rect { x: start.x - radius, y: start.y - radius, w: radius * 2.0, h: radius * 2.0 }));
                    }
                    Tool::Rect => {
                        draft_closed = true;
                        draft = Object::outline(&preview(Shape::Rect { radius: 0.0 }, r));
                    }
                    Tool::Ellipse => {
                        draft_closed = true;
                        draft = Object::outline(&preview(Shape::Ellipse, r));
                    }
                    _ => draft = vec![*start, *current],
                }
            }
            Some(Drag::Freehand { points, .. }) => {
                draft = points.clone();
                draft_width = Some(w.freehand_width());
                draft_color = matches!(w.tool, Tool::Eraser | Tool::Blur | Tool::Stamp | Tool::Sharpen | Tool::Dodge | Tool::Burn | Tool::Smudge | Tool::ReplaceColor).then_some(ERASER_PREVIEW);
            }
            Some(Drag::PenHandle) => draft = flatten_curve(&w.pen, &w.pen_handles, false),
            _ => {
                if !w.pen.is_empty() {
                    // The segment still being placed runs from the last vertex to the pointer.
                    let mut points = w.pen.clone();
                    points.push(w.cursor_doc);
                    draft = flatten_curve(&points, &w.pen_handles, false);
                }
            }
        }
        // A panel item in hand shows the box it will fill where it is dropped.
        if let Some((item, _)) = w.panel_drag {
            if w.canvas_bounds.get().contains(&w.cursor_screen) {
                let (width, height) = w.panel_item_size(item);
                let at = w.cursor_doc;
                marquee = Some(Rect { x: (at.x - width / 2.0).round(), y: (at.y - height / 2.0).round(), w: width, h: height });
            }
        }
        let mut handle_guides = Vec::new();
        let mut guide = |anchor: Vec2, handles: Handles| {
            for offset in [handles.before, handles.after] {
                if offset != Vec2::default() {
                    handle_guides.push((anchor, v2(anchor.x + offset.x, anchor.y + offset.y)));
                }
            }
        };
        match w.tool {
            Tool::Pen => {
                if let (Some(anchor), Some(handles)) = (w.pen.last(), w.pen_handles.last()) {
                    guide(*anchor, *handles);
                }
            }
            _ => {}
        }
        // Subselection shows the handles of the picked vertices only, as the Pen does after adding one.
        if matches!(w.tool, Tool::Subselect | Tool::Pen) {
            handle_guides.extend(w.visible_handles().into_iter().map(|(_, _, _, anchor, handle)| (anchor, handle)));
        }
        let caret = w.text_edit.and_then(|id| w.ed.object(id)).and_then(|o| match &o.shape {
            Shape::Text { content, size, style } => {
                // The caret is found in the upright frame, then follows the text's transform.
                let frame = o.placement.map_or(o.bounds, |p| p.frame);
                let at = crate::export::caret_place(content, *size, style, frame);
                Some((o.placement.map_or(at, |p| p.warp.apply(at)), *size))
            }
            _ => None,
        });
        let editing = w.text_edit;
        let ants = w.active_pixels().and_then(|s| Some((s.edges.clone(), w.ed.object(s.object)?.clone())));
        let stamp_mark = (w.tool == Tool::Stamp).then_some(w.stamp).flatten().and_then(|stamp| match (&w.drag, stamp.offset) {
            (Some(Drag::Freehand { points, .. }), Some(offset)) => {
                points.last().map(|p| Vec2 { x: p.x + offset.x, y: p.y + offset.y })
            }
            (_, None) => Some(stamp.source),
            _ => None,
        });
        Self {
            page: w.ed.doc.page().clone(),
            zoom: w.zoom,
            pan: w.pan,
            selected: w.ed.selected().into_iter().filter(|o| Some(o.id) != editing).cloned().collect(),
            selection_bounds: if editing.is_some() { None } else { w.ed.selection_bounds() },
            marquee,
            draft,
            handle_guides,
            picked_points: matches!(w.tool, Tool::Subselect | Tool::Pen).then(|| w.picked_points()),
            draft_closed,
            draft_width,
            draft_color,
            crop,
            ants,
            stamp_mark,
            all_handles: matches!(w.tool, Tool::Scale | Tool::Skew | Tool::Distort),
            jpeg_mask: w.jpeg_mask_rects().map(|rects| (rects, w.ed.doc.page().jpeg_mask_color.unwrap_or(crate::workspace::JPEG_MASK_COLOR))),
            transform_box: if editing.is_some() { None } else { w.transform_box() },
            caret,
            show_rulers: w.show_rulers,
            show_grid: w.show_grid,
            grid: (w.prefs.grid, w.prefs.grid_color),
            guide_lines: w.guide_lines(),
            fill_handles: w.fill_handles(),
            motion: w.motion_path(),
            hints: w.shape_hints(),
            nine_cuts: w.ed.selection_bounds().zip(w.nine_slice_lines()),
            stroke: w.stroke,
            fill: w.fill,
            textures: w.textures.clone(),
            ghosts: w.onion_ghosts(),
            view_mode: w.view_mode,
            previews: w.previews.clone(),
            raster: w.page_raster.as_ref().map(|(_, image)| image.clone()),
        }
    }

    pub fn paint(self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        if self.view_mode == ViewMode::Original {
            self.paint_artwork(bounds, true, window, cx);
            return;
        }
        window.paint_quad(fill(bounds, rgb(theme::DIVIDER)));
        let has_original = matches!(self.view_mode, ViewMode::TwoUp | ViewMode::FourUp);
        for (index, pane) in pane_rects(bounds, self.view_mode).into_iter().enumerate() {
            let preview = index.checked_sub(has_original as usize).and_then(|i| self.previews.get(i));
            let caption = if has_original && index == 0 {
                self.paint_artwork(pane, false, window, cx);
                t("Original").to_string()
            } else {
                self.paint_preview(pane, preview, window);
                preview.map_or(t("Working...").to_string(), |p| p.caption.clone())
            };
            paint_caption(pane, caption, window, cx);
        }
    }

    fn paint_preview(&self, pane: Bounds<Pixels>, preview: Option<&PreviewPane>, window: &mut Window) {
        let map = ViewMap::new(pane, self.page.width as f32, self.page.height as f32, self.zoom, self.pan);
        window.paint_quad(fill(pane, rgb(theme::PASTEBOARD)));
        window.with_content_mask(Some(ContentMask { bounds: pane }), |window| {
            let page_bounds = screen_rect(map, self.page.rect());
            if let Some(preview) = preview {
                let _ = window.paint_image(page_bounds, Corners::default(), preview.image.clone(), 0, false);
            }
            window.paint_quad(outline(page_bounds, rgb(theme::BORDER), BorderStyle::Solid));
        });
    }

    /// Draws the live page. `overlays` adds selection marks and rulers.
    fn paint_artwork(&self, bounds: Bounds<Pixels>, overlays: bool, window: &mut Window, cx: &mut App) {
        let map = ViewMap::new(bounds, self.page.width as f32, self.page.height as f32, self.zoom, self.pan);
        window.paint_quad(fill(bounds, rgb(theme::PASTEBOARD)));
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            let page_rect = Rect { x: 0.0, y: 0.0, w: self.page.width as f32, h: self.page.height as f32 };
            let page_bounds = screen_rect(map, page_rect);
            self.paint_canvas(page_bounds, bounds, window);
            // In PisiAnimasyon what lies off the stage shows on the pasteboard around it, as in Flash: that is where
            // things wait to come on, and where what copies are made from is kept. PisiWorks, like Fireworks,
            // shows the canvas only.
            if theme::is_sahne() {
                let outside = |o: &&Object| o.bounds.x < 0.0 || o.bounds.y < 0.0 || o.bounds.right() > page_rect.w || o.bounds.bottom() > page_rect.h;
                for layer in self.page.layers.iter().filter(|l| l.visible) {
                    for object in layer.objects.iter().filter(|o| o.visible && !o.is_web()).filter(outside) {
                        paint_object(object, &self.textures, map, window, cx);
                    }
                }
            }
            window.with_content_mask(Some(ContentMask { bounds: page_bounds }), |window| {
                for object in &self.ghosts {
                    paint_object(object, &self.textures, map, window, cx);
                }
                match &self.raster {
                    Some(image) => {
                        let _ = window.paint_image(page_bounds, Corners::default(), image.clone(), 0, false);
                    }
                    None => {
                        for layer in self.page.layers.iter().filter(|l| l.visible) {
                            for object in layer.objects.iter().filter(|o| o.visible && !o.is_web()) {
                                paint_object(object, &self.textures, map, window, cx);
                            }
                        }
                    }
                }
                if self.show_grid && overlays {
                    paint_grid(page_bounds, map, self.grid, window);
                }
                // Web areas sit above all artwork, whatever layer holds them.
                if overlays {
                    for object in self.page.layers.iter().filter(|l| l.visible).flat_map(|l| &l.objects).filter(|o| o.visible) {
                        paint_web(object, map, window);
                    }
                    self.paint_ants(map, page_bounds, window);
                    self.paint_crop(map, window);
                }
            });
            window.paint_quad(outline(page_bounds, rgb(theme::BORDER), BorderStyle::Solid));
            if overlays {
                self.paint_overlays(map, window);
                if let Some((start, end, width)) = self.fill_handles {
                    let (from, to) = (map.to_screen(start), map.to_screen(end));
                    // The handle for the width stands square to the line, on a line of its own.
                    if let Some(across) = width.map(|p| map.to_screen(p)) {
                        if let Some(line) = build_path(&[from, across], false, Some(1.0)) {
                            window.paint_path(line, rgb(0x000000));
                        }
                        let knob = Bounds { origin: point(across.x - px(3.), across.y - px(3.)), size: size(px(6.), px(6.)) };
                        window.paint_quad(fill(knob, rgb(0x000000)).border_widths(1.).border_color(rgb(0xffffff)));
                    }
                    if let Some(line) = build_path(&[from, to], false, Some(1.0)) {
                        window.paint_path(line, rgb(0x000000));
                    }
                    // A round handle where the gradient starts, a square one where it ends.
                    let knob = |at: Point<Pixels>| Bounds { origin: point(at.x - px(4.), at.y - px(4.)), size: size(px(8.), px(8.)) };
                    window.paint_quad(fill(knob(from), rgb(0xffffff)).corner_radii(px(4.)).border_widths(1.).border_color(rgb(0x000000)));
                    window.paint_quad(fill(knob(to), rgb(0xffffff)).border_widths(1.).border_color(rgb(0x000000)));
                }
                // Shape hints: a lettered disc each, yellow where it starts and green where it ends up, as in Flash.
                for (place, number, end) in &self.hints {
                    let at = map.to_screen(*place);
                    let disc = Bounds { origin: point(at.x - px(7.), at.y - px(7.)), size: size(px(14.), px(14.)) };
                    window.paint_quad(fill(disc, rgb(if *end { 0x40c040 } else { 0xf0d020 })).corner_radii(px(7.)).border_widths(1.).border_color(rgb(0x202020)));
                    let letter = SharedString::from(((b'a' + (*number as u8).min(25)) as char).to_string());
                    let run = TextRun { len: letter.len(), font: font(crate::fonts::family()), color: gpui::black(), background_color: None, underline: None, strikethrough: None };
                    let shaped = window.text_system().shape_line(letter, px(10.), &[run], None);
                    let _ = shaped.paint(point(at.x - px(3.), at.y - px(7.)), px(14.), window, cx);
                }
                // An animation's path: a dot for each state, green where it starts and red where it ends.
                if self.motion.len() > 1 {
                    let dots: Vec<Point<Pixels>> = self.motion.iter().map(|p| map.to_screen(*p)).collect();
                    if let Some(line) = build_path(&dots, false, Some(1.0)) {
                        window.paint_path(line, rgb(0x0000ff));
                    }
                    for (index, at) in dots.iter().enumerate() {
                        let color = if index == 0 { 0x00c000 } else if index + 1 == dots.len() { 0xff0000 } else { 0x0000ff };
                        let knob = Bounds { origin: point(at.x - px(3.), at.y - px(3.)), size: size(px(6.), px(6.)) };
                        window.paint_quad(fill(knob, rgb(color)).corner_radii(px(3.)));
                    }
                }
                // The cuts of 9-slice scaling run across the selection only.
                if let Some((bounds, cuts)) = self.nine_cuts {
                    let area = screen_rect(map, bounds);
                    for (index, at) in cuts.iter().enumerate() {
                        let on_screen = map.to_screen(Vec2 { x: *at, y: *at });
                        let line = if index < 2 {
                            Bounds { origin: point(on_screen.x, area.origin.y), size: size(px(1.), area.size.height) }
                        } else {
                            Bounds { origin: point(area.origin.x, on_screen.y), size: size(area.size.width, px(1.)) }
                        };
                        window.paint_quad(fill(line, rgb(0x2060ff)));
                    }
                }
                for (horizontal, at, color) in &self.guide_lines {
                    let on_screen = map.to_screen(Vec2 { x: *at, y: *at });
                    let line = if *horizontal {
                        Bounds { origin: point(bounds.origin.x, on_screen.y), size: size(bounds.size.width, px(1.)) }
                    } else {
                        Bounds { origin: point(on_screen.x, bounds.origin.y), size: size(px(1.), bounds.size.height) }
                    };
                    window.paint_quad(fill(line, rgb(*color)));
                }
                if self.show_rulers {
                    paint_rulers(bounds, map, window, cx);
                }
            }
        });
    }

    fn paint_canvas(&self, page: Bounds<Pixels>, view: Bounds<Pixels>, window: &mut Window) {
        match self.page.canvas {
            Some(color) => window.paint_quad(fill(page, to_hsla(color, 1.0))),
            None => {
                window.paint_quad(fill(page, rgb(0xffffff)));
                window.with_content_mask(Some(ContentMask { bounds: page }), |window| {
                    // Only cells in view are drawn, so the cost does not grow with zoom.
                    let cell = |from: Pixels, to: Pixels| (f32::from(to - from) / CHECKER).floor() as i32;
                    let columns = cell(page.origin.x, view.origin.x).max(0)..=cell(page.origin.x, view.right().min(page.right()));
                    let rows = cell(page.origin.y, view.origin.y).max(0)..=cell(page.origin.y, view.bottom().min(page.bottom()));
                    for row in rows {
                        for column in columns.clone().filter(|c| (c + row) % 2 == 1) {
                            let origin = point(
                                page.origin.x + px(column as f32 * CHECKER),
                                page.origin.y + px(row as f32 * CHECKER),
                            );
                            window.paint_quad(fill(Bounds { origin, size: size(px(CHECKER), px(CHECKER)) }, rgb(0xcccccc)));
                        }
                    }
                });
            }
        }
    }

    /// Draws the pixel selection as a black and white dashed outline.
    fn paint_ants(&self, map: ViewMap, page: Bounds<Pixels>, window: &mut Window) {
        let Some((edges, object)) = &self.ants else { return };
        let (left, right) = (f32::from(page.origin.x), f32::from(page.right()));
        let (top, bottom) = (f32::from(page.origin.y), f32::from(page.bottom()));
        let mut budget = MAX_ANT_DASHES;
        for (a, b) in edges.iter() {
            let (Some(a), Some(b)) = (object.pixel_to_doc(*a), object.pixel_to_doc(*b)) else { continue };
            let (a, b) = (map.to_screen(a), map.to_screen(b));
            let (x0, y0, x1, y1) = (f32::from(a.x), f32::from(a.y), f32::from(b.x), f32::from(b.y));
            let horizontal = y0 == y1;
            let (from, to, across) = if horizontal {
                (x0.max(left), x1.min(right), y0)
            } else {
                (y0.max(top), y1.min(bottom), x0)
            };
            if across < if horizontal { top } else { left } - 1.0 || across > if horizontal { bottom } else { right } {
                continue;
            }
            let mut t = from;
            while t < to && budget > 0 {
                let end = (t + DASH - t.rem_euclid(DASH)).min(to);
                let color = if (t / DASH).floor() as i64 % 2 == 0 { rgb(0x000000) } else { rgb(0xffffff) };
                let (origin, extent) = if horizontal {
                    (point(px(t), px(across)), size(px(end - t), px(1.)))
                } else {
                    (point(px(across), px(t)), size(px(1.), px(end - t)))
                };
                window.paint_quad(fill(Bounds { origin, size: extent }, color));
                t = end;
                budget -= 1;
            }
        }
    }

    /// Dims everything outside the pending crop area.
    fn paint_crop(&self, map: ViewMap, window: &mut Window) {
        let Some(a) = self.crop else { return };
        let page = self.page.rect();
        let shade = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.5 };
        let around = [
            Rect { x: 0.0, y: 0.0, w: page.w, h: a.y },
            Rect { x: 0.0, y: a.bottom(), w: page.w, h: page.h - a.bottom() },
            Rect { x: 0.0, y: a.y, w: a.x, h: a.h },
            Rect { x: a.right(), y: a.y, w: page.w - a.right(), h: a.h },
        ];
        for side in around {
            window.paint_quad(fill(screen_rect(map, side), shade));
        }
        window.paint_quad(outline(screen_rect(map, a), rgb(theme::SELECTION), BorderStyle::Solid));
    }

    fn paint_overlays(&self, map: ViewMap, window: &mut Window) {
        let accent = rgb(theme::SELECTION);
        if let Some((rects, color)) = &self.jpeg_mask {
            for rect in rects.iter() {
                window.paint_quad(fill(screen_rect(map, *rect), to_hsla(*color, JPEG_MASK_OPACITY)));
            }
        }
        for object in &self.selected {
            if matches!(object.shape, Shape::Text { .. }) {
                window.paint_quad(outline(screen_rect(map, object.bounds), accent, BorderStyle::Solid));
            } else {
                for (run, closed) in object.stroke_runs() {
                    let points: Vec<_> = run.iter().map(|p| map.to_screen(*p)).collect();
                    if let Some(path) = build_path(&points, closed, Some(1.0)) {
                        window.paint_path(path, accent);
                    }
                }
            }
            if matches!(object.shape, Shape::Line | Shape::Path { .. }) || self.picked_points.is_some() {
                match &self.picked_points {
                    // Every vertex, hollow until it is picked. Shapes show the ones they would have as paths.
                    Some(picked) => {
                        for (index, p) in object.path_points().iter().enumerate() {
                            let c = map.to_screen(*p);
                            let dot = Bounds { origin: point(c.x - px(2.5), c.y - px(2.5)), size: size(px(5.), px(5.)) };
                            let solid = picked.contains(&(object.id, index));
                            window.paint_quad(fill(dot, if solid { accent } else { rgb(0xffffff) }));
                            window.paint_quad(outline(dot, accent, BorderStyle::Solid));
                        }
                    }
                    None => {
                        let step = (object.points.len() / 40).max(1);
                        for p in object.points.iter().step_by(step) {
                            let c = map.to_screen(*p);
                            let dot = Bounds { origin: point(c.x - px(2.), c.y - px(2.)), size: size(px(4.), px(4.)) };
                            window.paint_quad(fill(dot, accent));
                        }
                    }
                }
            }
        }
        for (anchor, handle) in &self.handle_guides {
            let (from, to) = (map.to_screen(*anchor), map.to_screen(*handle));
            if let Some(path) = build_path(&[from, to], false, Some(1.0)) {
                window.paint_path(path, accent);
            }
            // Hollow, so a handle is never mistaken for a vertex.
            let knob = Bounds { origin: point(to.x - px(3.), to.y - px(3.)), size: size(px(6.), px(6.)) };
            window.paint_quad(fill(knob, rgb(0xffffff)).corner_radii(px(3.)));
            window.paint_quad(outline(knob, accent, BorderStyle::Solid).corner_radii(px(3.)));
        }
        if let Some((quad, pivot)) = self.transform_box {
            // The box follows the transform, so it may be turned or pulled out of square.
            let corners: Vec<_> = quad.iter().map(|p| map.to_screen(*p)).collect();
            if let Some(path) = build_path(&corners, true, Some(1.0)) {
                window.paint_path(path, accent);
            }
            let square = |at: Point<Pixels>, side: f32| Bounds { origin: point(at.x - px(side / 2.0), at.y - px(side / 2.0)), size: size(px(side), px(side)) };
            for handle in quad_handles(quad) {
                let handle = square(map.to_screen(handle), HANDLE_SIZE);
                window.paint_quad(fill(handle, rgb(0xffffff)));
                window.paint_quad(outline(handle, accent, BorderStyle::Solid));
            }
            let center = square(map.to_screen(pivot), HANDLE_SIZE);
            window.paint_quad(fill(center, rgb(0xffffff)).corner_radii(px(HANDLE_SIZE / 2.0)));
            window.paint_quad(outline(center, accent, BorderStyle::Solid).corner_radii(px(HANDLE_SIZE / 2.0)));
        } else if let Some(bounds) = self.selection_bounds {
            window.paint_quad(outline(screen_rect(map, bounds), accent, BorderStyle::Solid));
            if self.selected.len() == 1 || self.all_handles {
                for handle in handle_rects(bounds, map) {
                    window.paint_quad(fill(handle, rgb(0xffffff)));
                    window.paint_quad(outline(handle, accent, BorderStyle::Solid));
                }
            }
        }
        if let Some(area) = self.marquee {
            let rect = screen_rect(map, area);
            window.paint_quad(fill(rect, Rgba { r: 0.0, g: 0.66, b: 1.0, a: 0.15 }));
            window.paint_quad(outline(rect, accent, BorderStyle::Solid));
        }
        if self.draft.len() >= 2 {
            let screen: Vec<_> = self.draft.iter().map(|p| map.to_screen(*p)).collect();
            if self.draft_closed {
                if let Some(path) = self.fill.and_then(|_| build_path(&screen, true, None)) {
                    window.paint_path(path, to_hsla(self.fill.unwrap_or(Color::WHITE), 0.6));
                }
            }
            let width = self.draft_width.map_or(1.0, |w| (w * map.zoom).max(1.0));
            if let Some(path) = build_path(&screen, self.draft_closed, Some(width)) {
                window.paint_path(path, to_hsla(self.draft_color.or(self.stroke).unwrap_or(Color::BLACK), 1.0));
            }
        }
        if let Some(mark) = self.stamp_mark {
            let c = map.to_screen(mark);
            let arm = px(7.);
            let horizontal = Bounds { origin: point(c.x - arm, c.y), size: size(arm * 2., px(1.)) };
            let vertical = Bounds { origin: point(c.x, c.y - arm), size: size(px(1.), arm * 2.) };
            window.paint_quad(fill(horizontal, accent));
            window.paint_quad(fill(vertical, accent));
        }
        if let Some((at, text_size)) = self.caret {
            let top = map.to_screen(at);
            let height = px(text_size * LINE_HEIGHT * map.zoom);
            window.paint_quad(fill(Bounds { origin: top, size: size(px(1.5), height) }, accent));
        }
    }
}

fn preview(shape: Shape, bounds: Rect) -> Object {
    Object {
        id: 0,
        name: String::new(),
        shape,
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

fn paint_web(object: &Object, map: ViewMap, window: &mut Window) {
    let Shape::Web { kind, .. } = &object.shape else { return };
    let (r, g, b) = match kind {
        WebKind::Hotspot => (0.0, 0.8, 0.9),
        WebKind::Slice => (0.1, 0.9, 0.2),
    };
    // A round or many-sided area is drawn along its own outline.
    if let Some(corners) = object.web_outline() {
        let on_screen: Vec<Point<Pixels>> = corners.iter().map(|p| map.to_screen(*p)).collect();
        if let Some(path) = build_path(&on_screen, true, None) {
            window.paint_path(path, Rgba { r, g, b, a: 0.35 });
        }
        if let Some(path) = build_path(&on_screen, true, Some(1.0)) {
            window.paint_path(path, Rgba { r, g, b, a: 1.0 });
        }
        return;
    }
    let area = screen_rect(map, object.bounds);
    window.paint_quad(fill(area, Rgba { r, g, b, a: 0.35 }));
    window.paint_quad(outline(area, Rgba { r, g, b, a: 1.0 }, BorderStyle::Solid));
}

pub fn paint_object(object: &Object, textures: &Textures, map: ViewMap, window: &mut Window, cx: &mut App) {
    if object.mask_of.is_some() {
        return;
    }
    // Text that has been rotated, skewed or distorted is drawn from the outlines of its letters.
    if let Some(paths) = object.placed_text_paths() {
        return paint_object(&paths, textures, map, window, cx);
    }
    if object.is_instance() {
        for part in object.instance_parts().iter().filter(|p| p.visible && !p.is_web()) {
            paint_object(part, textures, map, window, cx);
        }
        return;
    }
    if let Some((key, _)) = texture_key(object) {
        match textures.get(&key) {
            Some((_, image)) => {
                let _ = window.paint_image(screen_rect(map, object.bounds), Corners::default(), image.clone(), 0, false);
            }
            // Library thumbnails show images that have no texture as a plain block.
            None => window.paint_quad(fill(screen_rect(map, object.bounds), rgb(0xbbbbbb))),
        }
        return;
    }
    if let Shape::Text { content, size: text_size, style } = &object.shape {
        let Some(color) = object.fill else { return };
        paint_text(object, content, *text_size, style, to_hsla(color, object.opacity), map, window, cx);
        return;
    }
    let to_screen = |points: &[Vec2]| points.iter().map(|p| map.to_screen(*p)).collect::<Vec<_>>();
    // A filled open path, such as a Knife piece, fills as if closed but has no closing stroke.
    if let Some(color) = object.fill {
        if let Some(path) = build_path(&to_screen(&object.outline()), true, None) {
            // Where the page is not drawn pixel by pixel, as in thumbnails, a gradient is shown by its
            // first and last colors along its line.
            match object.fill_style.as_deref().filter(|style| style.stops.len() >= 2 && style.kind != crate::model::FillKind::Pattern) {
                Some(style) => {
                    let (start, end) = style.handles(object.bounds);
                    let turn = (end.x - start.x).atan2(-(end.y - start.y)).to_degrees().rem_euclid(360.0);
                    let (from, to) = (style.stops[0].1, style.stops[style.stops.len() - 1].1);
                    let shade = gpui::linear_gradient(
                        turn,
                        gpui::linear_color_stop(to_hsla(from, object.opacity * from.a as f32 / 255.0), 0.0),
                        gpui::linear_color_stop(to_hsla(to, object.opacity * to.a as f32 / 255.0), 1.0),
                    );
                    window.paint_path(path, shade);
                }
                None => window.paint_path(path, to_hsla(color, object.opacity)),
            }
        }
    }
    if let Some(stroke) = object.stroke.filter(|s| s.width > 0.0) {
        for (run, closed) in object.stroke_runs() {
            if let Some(path) = build_path(&to_screen(&run), closed, Some((stroke.width * map.zoom).max(1.0))) {
                window.paint_path(path, to_hsla(stroke.color, object.opacity));
            }
        }
    }
}

/// Draws objects scaled to fit `bounds`, for panel thumbnails.
/// Labels a comparison pane along its bottom edge.
fn paint_caption(pane: Bounds<Pixels>, text: String, window: &mut Window, cx: &mut App) {
    let strip = Bounds {
        origin: point(pane.origin.x, pane.bottom() - px(PANE_CAPTION_HEIGHT)),
        size: size(pane.size.width, px(PANE_CAPTION_HEIGHT)),
    };
    window.paint_quad(fill(strip, rgb(theme::PANEL)));
    let run = TextRun {
        len: text.len(),
        font: font(crate::fonts::family()),
        color: rgb(theme::TEXT).into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let shaped = window.text_system().shape_line(SharedString::from(text), px(theme::FONT_SIZE), &[run], None);
    let _ = shaped.paint(point(strip.origin.x + px(6.), strip.origin.y + px(2.)), px(14.), window, cx);
}

pub fn paint_thumbnail(objects: &[Object], bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    paint_thumbnail_with(objects, &Textures::new(), bounds, window, cx);
}

/// Like `paint_thumbnail`, drawing bitmaps with their real pixels.
pub fn paint_thumbnail_with(objects: &[Object], textures: &Textures, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    const PADDING: f32 = 4.0;
    let Some(extent) = objects.iter().map(|o| o.bounds).reduce(Rect::union) else { return };
    let (w, h) = (f32::from(bounds.size.width) - 2.0 * PADDING, f32::from(bounds.size.height) - 2.0 * PADDING);
    let zoom = (w / extent.w.max(1.0)).min(h / extent.h.max(1.0));
    let origin = point(
        bounds.origin.x + px((f32::from(bounds.size.width) - extent.w * zoom) / 2.0 - extent.x * zoom),
        bounds.origin.y + px((f32::from(bounds.size.height) - extent.h * zoom) / 2.0 - extent.y * zoom),
    );
    let map = ViewMap { origin, zoom };
    for object in objects {
        paint_object(object, textures, map, window, cx);
    }
}

fn paint_text(object: &Object, content: &str, text_size: f32, style: &crate::model::TextStyle, color: Hsla, map: ViewMap, window: &mut Window, cx: &mut App) {
    let line_height = text_size * LINE_HEIGHT;
    let mut face = font(if style.font.is_empty() { crate::fonts::family().to_string() } else { style.font.clone() });
    if style.bold {
        face = face.bold();
    }
    if style.italic {
        face = face.italic();
    }
    for (row, line) in content.split('\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let run = TextRun {
            len: line.len(),
            font: face.clone(),
            color,
            background_color: None,
            underline: style.underline.then(|| gpui::UnderlineStyle { thickness: px((text_size * map.zoom / 14.0).max(1.0)), color: Some(color), wavy: false }),
            strikethrough: None,
        };
        let shaped = window.text_system().shape_line(SharedString::from(line.to_string()), px(text_size * map.zoom), &[run], None);
        let start = crate::export::line_start(object.bounds.w, crate::export::line_width(line, text_size, style), style.align);
        let origin = map.to_screen(Vec2 { x: object.bounds.x + start, y: object.bounds.y + row as f32 * line_height });
        let _ = shaped.paint(origin, px(line_height * map.zoom), window, cx);
    }
}

fn paint_grid(page: Bounds<Pixels>, map: ViewMap, grid: ((f32, f32), crate::model::Color), window: &mut Window) {
    let ((across, down), tint) = grid;
    let (step_x, step_y) = (across.max(1.0) * map.zoom, down.max(1.0) * map.zoom);
    if step_x < 6.0 || step_y < 6.0 {
        return;
    }
    let color = Rgba { r: tint.r as f32 / 255.0, g: tint.g as f32 / 255.0, b: tint.b as f32 / 255.0, a: 0.5 };
    let (x0, y0) = (f32::from(page.origin.x), f32::from(page.origin.y));
    let (w, h) = (f32::from(page.size.width), f32::from(page.size.height));
    let mut x = step_x;
    while x < w {
        window.paint_quad(fill(Bounds { origin: point(px(x0 + x), px(y0)), size: size(px(1.), px(h)) }, color));
        x += step_x;
    }
    let mut y = step_y;
    while y < h {
        window.paint_quad(fill(Bounds { origin: point(px(x0), px(y0 + y)), size: size(px(w), px(1.)) }, color));
        y += step_y;
    }
}

fn paint_rulers(bounds: Bounds<Pixels>, map: ViewMap, window: &mut Window, cx: &mut App) {
    let thick = px(theme::RULER_SIZE);
    let tick_color = rgb(theme::TEXT_DIM);
    window.paint_quad(fill(Bounds { origin: bounds.origin, size: size(bounds.size.width, thick) }, rgb(theme::RULER)));
    window.paint_quad(fill(Bounds { origin: bounds.origin, size: size(thick, bounds.size.height) }, rgb(theme::RULER)));

    let min_spacing = 50.0;
    let mut step = 1.0_f32;
    'search: for magnitude in [1.0_f32, 10.0, 100.0, 1000.0, 10000.0] {
        for factor in [1.0, 2.0, 5.0] {
            step = magnitude * factor;
            if step * map.zoom >= min_spacing {
                break 'search;
            }
        }
    }
    let label = |text: String, origin: Point<Pixels>, window: &mut Window, cx: &mut App| {
        let run = TextRun {
            len: text.len(),
            font: font(crate::fonts::family()),
            color: rgb(theme::TEXT_DIM).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let shaped = window.text_system().shape_line(SharedString::from(text), px(9.), &[run], None);
        let _ = shaped.paint(origin, px(11.), window, cx);
    };

    let left = f32::from(bounds.origin.x) + theme::RULER_SIZE;
    let right = f32::from(bounds.right());
    let top = f32::from(bounds.origin.y) + theme::RULER_SIZE;
    let bottom = f32::from(bounds.bottom());
    let (origin_x, origin_y) = (f32::from(map.origin.x), f32::from(map.origin.y));
    let first_x = ((left - origin_x) / (step * map.zoom)).floor() as i32;
    let last_x = ((right - origin_x) / (step * map.zoom)).ceil() as i32;
    for i in first_x..=last_x {
        let x = origin_x + i as f32 * step * map.zoom;
        if x >= left && x <= right {
            window.paint_quad(fill(
                Bounds { origin: point(px(x), bounds.origin.y), size: size(px(1.), thick) },
                tick_color,
            ));
            label(format!("{}", (i as f32 * step) as i32), point(px(x + 3.), bounds.origin.y), window, cx);
        }
    }
    let first_y = ((top - origin_y) / (step * map.zoom)).floor() as i32;
    let last_y = ((bottom - origin_y) / (step * map.zoom)).ceil() as i32;
    for i in first_y..=last_y {
        let y = origin_y + i as f32 * step * map.zoom;
        if y >= top && y <= bottom {
            window.paint_quad(fill(
                Bounds { origin: point(bounds.origin.x, px(y)), size: size(thick, px(1.)) },
                tick_color,
            ));
            label(format!("{}", (i as f32 * step) as i32), point(bounds.origin.x + px(2.), px(y + 1.)), window, cx);
        }
    }
    window.paint_quad(fill(Bounds { origin: bounds.origin, size: size(thick, thick) }, rgb(theme::PANEL_DARK)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panes_split_the_view_without_overlap() {
        let bounds = Bounds { origin: point(px(10.), px(10.)), size: size(px(202.), px(102.)) };
        let two = pane_rects(bounds, ViewMode::TwoUp);
        assert_eq!(two.len(), 2);
        assert_eq!(two[0].size.width, px(100.));
        assert_eq!(two[1].origin.x, px(112.));
        let four = pane_rects(bounds, ViewMode::FourUp);
        assert_eq!(four.len(), 4);
        assert_eq!(four[3].bottom(), bounds.bottom());
        assert_eq!(pane_rects(bounds, ViewMode::Preview), vec![bounds]);
    }

    #[test]
    fn texture_is_bgra_with_opacity_baked_in() {
        let bitmap = Bitmap::from_rgba(1, 1, vec![10, 20, 30, 200]).unwrap();
        let texture = make_texture(&bitmap, 0.5);
        assert_eq!(texture.as_bytes(0).unwrap(), &[30, 20, 10, 100]);
    }

    #[test]
    fn oversized_bitmap_is_previewed_within_limit() {
        let bitmap = Bitmap::from_rgba(8192, 2, vec![255; 8192 * 2 * 4]).unwrap();
        let bytes = make_texture(&bitmap, 1.0).as_bytes(0).unwrap().len();
        assert_eq!(bytes, 4096 * 4);
    }
}
