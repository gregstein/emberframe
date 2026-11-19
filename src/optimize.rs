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

//! Export encoders driven by the Optimize panel.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, ensure, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use serde::{Deserialize, Serialize};
use tiny_skia::Pixmap;

use crate::i18n::{t, tf};
use crate::export::{export_image, render_page};
use crate::model::{Color, Page};

const ALPHA_CUTOFF: u8 = 128;
/// Bits kept per channel when matching pixels to a reduced palette.
const MATCH_BITS: u32 = 5;
const MAX_GIF_SIZE: u32 = u16::MAX as u32;
pub const MIN_COLORS: u16 = 2;
pub const MAX_COLORS: u16 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Gif,
    AnimatedGif,
    Jpeg,
    Png8,
    Png24,
    Png32,
}

impl ExportFormat {
    pub const ALL: [ExportFormat; 6] = [
        ExportFormat::Gif,
        ExportFormat::AnimatedGif,
        ExportFormat::Jpeg,
        ExportFormat::Png8,
        ExportFormat::Png24,
        ExportFormat::Png32,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ExportFormat::Gif => "GIF",
            ExportFormat::AnimatedGif => t("Animated GIF"),
            ExportFormat::Jpeg => "JPEG",
            ExportFormat::Png8 => "PNG 8",
            ExportFormat::Png24 => "PNG 24",
            ExportFormat::Png32 => "PNG 32",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Gif | ExportFormat::AnimatedGif => "gif",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::Png8 | ExportFormat::Png24 | ExportFormat::Png32 => "png",
        }
    }

    /// Whether the format stores a limited palette.
    pub fn is_indexed(self) -> bool {
        matches!(self, ExportFormat::Gif | ExportFormat::AnimatedGif | ExportFormat::Png8)
    }

    /// Whether Index Transparency can be switched on.
    pub fn has_index_transparency(self) -> bool {
        self.is_indexed()
    }
}

/// Ready-made settings offered at the top of the Optimize panel.
pub const PRESETS: [(&str, ExportSettings); 7] = [
    ("GIF Web 216", ExportSettings { format: ExportFormat::Gif, quality: 80, colors: 216, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
    ("GIF Web 256", ExportSettings { format: ExportFormat::Gif, quality: 80, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
    ("Animated GIF", ExportSettings { format: ExportFormat::AnimatedGif, quality: 80, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
    ("JPEG - Better Quality", ExportSettings { format: ExportFormat::Jpeg, quality: 80, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
    ("JPEG - Smaller File", ExportSettings { format: ExportFormat::Jpeg, quality: 60, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
    ("PNG 8", ExportSettings { format: ExportFormat::Png8, quality: 80, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
    ("PNG 32", ExportSettings { format: ExportFormat::Png32, quality: 80, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }),
];

/// The settings compared in 4-Up view: the current ones and two cheaper alternatives.
pub fn comparison_settings(current: ExportSettings) -> [ExportSettings; 3] {
    let with = |change: fn(&mut ExportSettings)| {
        let mut settings = current;
        change(&mut settings);
        settings
    };
    match current.format {
        ExportFormat::Jpeg => [
            current,
            ExportSettings { quality: (current.quality as u32 * 7 / 10).max(1) as u8, ..current },
            ExportSettings { quality: (current.quality as u32 * 4 / 10).max(1) as u8, ..current },
        ],
        format if format.is_indexed() => [
            current,
            ExportSettings { colors: (current.colors / 2).max(MIN_COLORS), ..current },
            ExportSettings { colors: (current.colors / 4).max(MIN_COLORS), ..current },
        ],
        _ => [current, with(|s| s.format = ExportFormat::Png8), with(|s| s.format = ExportFormat::Jpeg)],
    }
}

/// A short description of settings, for captions.
pub fn settings_label(settings: &ExportSettings) -> String {
    match settings.format {
        ExportFormat::Jpeg => format!("JPEG {}%", settings.quality),
        format if format.is_indexed() => tf("{} {} colors", &[&format.label(), &settings.colors]),
        format => format.label().to_string(),
    }
}

/// Unique colors of an encoded image, up to `MAX_COLORS`.
pub fn encoded_palette(bytes: &[u8]) -> Vec<Color> {
    let Ok(image) = image::load_from_memory(bytes) else { return Vec::new() };
    let mut seen = HashSet::new();
    let mut palette = Vec::new();
    for px in image.to_rgba8().pixels().filter(|p| p[3] != 0) {
        if seen.insert([px[0], px[1], px[2]]) {
            palette.push(Color::rgb(px[0], px[1], px[2]));
            if palette.len() == MAX_COLORS as usize {
                break;
            }
        }
    }
    palette
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportSettings {
    pub format: ExportFormat,
    /// JPEG quality, 1 to 100.
    pub quality: u8,
    /// Palette size for indexed formats.
    pub colors: u16,
    /// Keeps fully transparent areas in indexed formats.
    pub transparent: bool,
    /// Backdrop for flattened areas; `None` is white.
    pub matte: Option<Color>,
    /// JPEG smoothing, 0 to 8: how much the picture is blurred before it is compressed,
    /// which hides blocking and makes the file smaller.
    #[serde(default)]
    pub smoothing: u8,
    /// Whether the page's JPEG mask, and text if `preserve_text` is set, is written at `selective_quality`.
    #[serde(default)]
    pub selective: bool,
    #[serde(default = "default_selective_quality")]
    pub selective_quality: u8,
    #[serde(default)]
    pub preserve_text: bool,
    /// Whether copies of button symbols are written at the selective quality too.
    #[serde(default)]
    pub preserve_buttons: bool,
}

fn default_selective_quality() -> u8 {
    90
}

/// The largest JPEG smoothing setting.
pub const MAX_SMOOTHING: u8 = 8;

impl Default for ExportSettings {
    fn default() -> Self {
        Self { format: ExportFormat::Png32, quality: 80, colors: 256, transparent: false, matte: None, smoothing: 0, selective: false, selective_quality: 90, preserve_text: false, preserve_buttons: false }
    }
}

/// The pictures of an animated GIF, each drawn whole as it shows, with the time it
/// stays on show in milliseconds, and how many times the animation plays (0 for ever).
/// `None` for anything that is not a GIF of more than one picture.
pub fn gif_frames(bytes: &[u8]) -> Option<(Vec<(crate::bitmap::Bitmap, u32)>, u16)> {
    use image::AnimationDecoder;
    if !bytes.starts_with(b"GIF8") {
        return None;
    }
    let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(bytes)).ok()?;
    let mut frames = Vec::new();
    for frame in decoder.into_frames().take(2000) {
        let frame = frame.ok()?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay = if denominator == 0 { 100 } else { numerator / denominator };
        let picture = frame.into_buffer();
        let (width, height) = (picture.width(), picture.height());
        // A delay of nothing is shown by browsers as a tenth of a second.
        frames.push((crate::bitmap::Bitmap::from_rgba(width, height, picture.into_raw())?, if delay == 0 { 100 } else { delay }));
    }
    if frames.len() < 2 {
        return None;
    }
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::Indexed);
    let loops = match options.read_info(bytes).map(|reader| reader.repeat()) {
        Ok(gif::Repeat::Finite(more)) => more.saturating_add(1),
        Ok(gif::Repeat::Infinite) => 0,
        Err(_) => 0,
    };
    Some((frames, loops))
}

/// Straight-alpha RGBA pixels of a rendered page.
fn straight_rgba(pixmap: &Pixmap) -> Vec<u8> {
    pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}

/// Blends every pixel onto `matte`, keeping `transparent` areas marked with alpha 0.
fn flatten_on(rgba: &mut [u8], matte: Color, keep_transparent: bool) {
    for px in rgba.chunks_exact_mut(4) {
        let alpha = px[3] as u32;
        if keep_transparent && px[3] < ALPHA_CUTOFF {
            px[3] = 0;
            continue;
        }
        for (channel, back) in px.iter_mut().zip([matte.r, matte.g, matte.b]) {
            *channel = ((*channel as u32 * alpha + back as u32 * (255 - alpha) + 127) / 255) as u8;
        }
        px[3] = 255;
    }
}

struct Indexed {
    /// RGB triples.
    palette: Vec<u8>,
    indices: Vec<u8>,
    transparent: Option<u8>,
}

/// Median cut over the image's colors, weighted by how often each appears.
fn median_cut(mut colors: Vec<([u8; 3], u32)>, limit: usize) -> Vec<[u8; 3]> {
    let mut boxes: Vec<Vec<([u8; 3], u32)>> = vec![std::mem::take(&mut colors)];
    while boxes.len() < limit {
        let range = |b: &[([u8; 3], u32)], channel: usize| {
            let values = b.iter().map(|(c, _)| c[channel]);
            values.clone().max().unwrap_or(0) - values.min().unwrap_or(0)
        };
        let widest = |b: &[([u8; 3], u32)]| (0..3).map(|channel| range(b, channel)).max().unwrap_or(0);
        let Some(index) = (0..boxes.len()).filter(|i| boxes[*i].len() > 1).max_by_key(|i| widest(&boxes[*i])) else {
            break;
        };
        let mut split = boxes.swap_remove(index);
        let channel = (0..3).max_by_key(|channel| range(&split, *channel)).unwrap_or(0);
        split.sort_by_key(|(c, _)| c[channel]);
        let half = split.iter().map(|(_, n)| *n as u64).sum::<u64>() / 2;
        let mut seen = 0;
        let mut cut = split.len() - 1;
        for (i, (_, n)) in split.iter().enumerate() {
            seen += *n as u64;
            if seen >= half {
                cut = i + 1;
                break;
            }
        }
        let upper = split.split_off(cut.clamp(1, split.len() - 1));
        boxes.push(split);
        boxes.push(upper);
    }
    boxes
        .iter()
        .filter(|b| !b.is_empty())
        .map(|b| {
            let total: u64 = b.iter().map(|(_, n)| *n as u64).sum();
            let mean = |channel: usize| (b.iter().map(|(c, n)| c[channel] as u64 * *n as u64).sum::<u64>() / total.max(1)) as u8;
            [mean(0), mean(1), mean(2)]
        })
        .collect()
}

/// Reduces flattened RGBA pixels to `colors` entries.
fn quantize(rgba: &[u8], colors: u16) -> Indexed {
    let has_transparency = rgba.chunks_exact(4).any(|p| p[3] == 0);
    let limit = (colors.clamp(MIN_COLORS, MAX_COLORS) as usize) - has_transparency as usize;
    let limit = limit.max(1);
    let mut histogram: HashMap<[u8; 3], u32> = HashMap::new();
    for px in rgba.chunks_exact(4).filter(|p| p[3] != 0) {
        *histogram.entry([px[0], px[1], px[2]]).or_insert(0) += 1;
    }
    let exact = histogram.len() <= limit;
    let unique: Vec<_> = histogram.into_iter().collect();
    let table = if exact { unique.iter().map(|(c, _)| *c).collect() } else { median_cut(unique, limit) };
    let table = if table.is_empty() { vec![[0, 0, 0]] } else { table };
    let offset = has_transparency as u8;
    let exact_index: HashMap<[u8; 3], u8> = if exact {
        table.iter().enumerate().map(|(i, c)| (*c, i as u8 + offset)).collect()
    } else {
        HashMap::new()
    };
    let mut nearest: HashMap<u32, u8> = HashMap::new();
    let shift = 8 - MATCH_BITS;
    let indices = rgba
        .chunks_exact(4)
        .map(|px| {
            if px[3] == 0 {
                return 0;
            }
            let color = [px[0], px[1], px[2]];
            if exact {
                return exact_index[&color];
            }
            let key = ((px[0] >> shift) as u32) << (2 * MATCH_BITS) | ((px[1] >> shift) as u32) << MATCH_BITS | (px[2] >> shift) as u32;
            *nearest.entry(key).or_insert_with(|| {
                let distance = |c: &[u8; 3]| (0..3).map(|i| (c[i] as i32 - color[i] as i32).pow(2)).sum::<i32>();
                table.iter().enumerate().min_by_key(|(_, c)| distance(c)).map_or(0, |(i, _)| i as u8) + offset
            })
        })
        .collect();
    let mut palette = Vec::with_capacity((table.len() + 1) * 3);
    if has_transparency {
        palette.extend([0, 0, 0]);
    }
    palette.extend(table.iter().flatten());
    Indexed { palette, indices, transparent: has_transparency.then_some(0) }
}

/// Blurs RGB pixels a little, as much as the JPEG smoothing setting asks for.
fn smoothed(rgb: Vec<u8>, width: u32, height: u32, smoothing: u8) -> Vec<u8> {
    if smoothing == 0 {
        return rgb;
    }
    // A blur of three pixels across, mixed with the picture: 8 is the full blur.
    let amount = smoothing.min(MAX_SMOOTHING) as f32 / MAX_SMOOTHING as f32;
    let (w, h) = (width as i64, height as i64);
    let at = |x: i64, y: i64, c: usize| rgb[((y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) * 3) as usize + c] as f32;
    let mut out = vec![0u8; rgb.len()];
    for y in 0..h {
        for x in 0..w {
            for c in 0..3 {
                let around: f32 = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))).map(|(dx, dy)| at(x + dx, y + dy, c)).sum::<f32>() / 9.0;
                let here = at(x, y, c);
                out[((y * w + x) * 3) as usize + c] = (here + (around - here) * amount).round() as u8;
            }
        }
    }
    out
}

fn prepared(pixmap: &Pixmap, settings: &ExportSettings) -> Vec<u8> {
    let mut rgba = straight_rgba(pixmap);
    if settings.format != ExportFormat::Png32 {
        let keep = settings.transparent && settings.format.has_index_transparency();
        flatten_on(&mut rgba, settings.matte.unwrap_or(Color::WHITE), keep);
    }
    rgba
}

fn encode_png8(pixmap: &Pixmap, settings: &ExportSettings) -> Result<Vec<u8>> {
    let image = quantize(&prepared(pixmap, settings), settings.colors);
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, pixmap.width(), pixmap.height());
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_palette(image.palette);
        if image.transparent.is_some() {
            encoder.set_trns(vec![0u8]);
        }
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&image.indices)?;
    }
    Ok(out)
}

fn gif_frame(pixmap: &Pixmap, settings: &ExportSettings, delay_ms: u32) -> gif::Frame<'static> {
    let image = quantize(&prepared(pixmap, settings), settings.colors);
    gif::Frame {
        width: pixmap.width() as u16,
        height: pixmap.height() as u16,
        // GIF delays count hundredths of a second.
        delay: (delay_ms / 10).min(u16::MAX as u32) as u16,
        dispose: if image.transparent.is_some() { gif::DisposalMethod::Background } else { gif::DisposalMethod::Keep },
        transparent: image.transparent,
        palette: Some(image.palette),
        buffer: image.indices.into(),
        ..gif::Frame::default()
    }
}

fn encode_gif(page: &Page, settings: &ExportSettings) -> Result<Vec<u8>> {
    ensure!(page.width <= MAX_GIF_SIZE && page.height <= MAX_GIF_SIZE, tf("GIF images cannot be larger than {} pixels", &[&MAX_GIF_SIZE]));
    let animated = settings.format == ExportFormat::AnimatedGif;
    let states: Vec<usize> = if animated {
        (0..page.states.len()).filter(|i| page.states[*i].exported).collect()
    } else {
        vec![page.active_state]
    };
    ensure!(!states.is_empty(), t("Every state is switched off in the States panel"));
    let mut out = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut out, page.width as u16, page.height as u16, &[])?;
        // Played once through, a GIF says nothing about repeating; otherwise it says how many more times.
        if animated && page.gif_loops != 1 {
            encoder.set_repeat(match page.gif_loops {
                0 => gif::Repeat::Infinite,
                times => gif::Repeat::Finite(times - 1),
            })?;
        }
        for index in states {
            let pixmap = render_page(&page.at_state(index))?;
            encoder.write_frame(&gif_frame(&pixmap, settings, page.states[index].delay_ms))?;
        }
    }
    Ok(out)
}

/// Encodes the page with these settings.
pub fn encode(page: &Page, settings: &ExportSettings) -> Result<Vec<u8>> {
    if matches!(settings.format, ExportFormat::Gif | ExportFormat::AnimatedGif) {
        return encode_gif(page, settings);
    }
    let pixmap = render_page(page)?;
    let (w, h) = (pixmap.width(), pixmap.height());
    let mut out = Vec::new();
    match settings.format {
        ExportFormat::Png8 => return encode_png8(&pixmap, settings),
        ExportFormat::Png32 => {
            PngEncoder::new(&mut out).write_image(&straight_rgba(&pixmap), w, h, ExtendedColorType::Rgba8)?;
        }
        ExportFormat::Png24 | ExportFormat::Jpeg => {
            let rgb: Vec<u8> = prepared(&pixmap, settings).chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
            if settings.format == ExportFormat::Png24 {
                PngEncoder::new(&mut out).write_image(&rgb, w, h, ExtendedColorType::Rgb8)?;
            } else {
                let rgb = smoothed(rgb, w, h, settings.smoothing);
                let quality = settings.quality.clamp(1, 100);
                // The areas kept at the selective quality: the JPEG mask, and text if it is to be preserved.
                let mask = page.jpeg_mask.as_ref().filter(|mask| mask.size() == (w, h));
                let kept = |o: &&crate::model::Object| match &o.shape {
                    crate::model::Shape::Text { .. } => settings.preserve_text,
                    crate::model::Shape::Instance { symbol, .. } => settings.preserve_buttons && page.button_symbols.contains(symbol),
                    _ => false,
                };
                let text: Vec<crate::model::Rect> = page.objects().filter(|o| o.visible).filter(kept).map(|o| o.bounds).collect();
                if settings.selective && (mask.is_some() || !text.is_empty()) {
                    let covers = |x: u32, y: u32| {
                        mask.is_some_and(|mask| mask.is_set(x, y)) || text.iter().any(|b| b.contains(crate::model::v2(x as f32 + 0.5, y as f32 + 0.5)))
                    };
                    let selective = crate::jpeg::Selective { covers: &covers, quality: settings.selective_quality.clamp(1, 100) };
                    out = crate::jpeg::encode(&rgb, w, h, quality, Some(selective));
                } else {
                    JpegEncoder::new_with_quality(&mut out, quality).write_image(&rgb, w, h, ExtendedColorType::Rgb8)?;
                }
            }
        }
        ExportFormat::Gif | ExportFormat::AnimatedGif => return Err(anyhow!("handled above")),
    }
    Ok(out)
}

/// Picks encoder settings for a file name. A name that disagrees with the panel wins,
/// so typing `.jpg` always writes a JPEG; other extensions return `None`.
fn settings_for_path(settings: &ExportSettings, path: &Path) -> Option<ExportSettings> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    let wanted = match extension.as_str() {
        "gif" => ExportFormat::Gif,
        "jpg" | "jpeg" => ExportFormat::Jpeg,
        "png" => ExportFormat::Png32,
        _ => return None,
    };
    let format = if settings.format.extension() == ExportFormat::extension(wanted) { settings.format } else { wanted };
    Some(ExportSettings { format, ..*settings })
}

/// Writes the page using the Optimize settings.
pub fn export_page(page: &Page, path: &Path) -> Result<()> {
    match settings_for_path(&page.export, path) {
        Some(settings) => {
            let bytes = encode(page, &settings)?;
            std::fs::write(path, bytes).with_context(|| tf("could not write {}", &[&path.display()]))
        }
        None => export_image(page, path),
    }
}

/// Short readable file size for the panel.
pub fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f32 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f32 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod animation_tests {
    use super::*;
    use crate::model::{Document, Editor};

    #[test]
    fn an_animated_gif_comes_back_as_states_with_its_times_and_looping() {
        // Three states, each a box further right, played four times.
        let mut e = Editor::new((crate::examples::EXAMPLES[2].build)());
        let page = e.doc.page_mut();
        page.gif_loops = 4;
        page.states[1].delay_ms = 250;
        page.states.truncate(3);
        page.animations.clear();
        let page = e.export_page();
        let bytes = encode(&page, &page.export).unwrap();
        let (frames, loops) = gif_frames(&bytes).unwrap();
        assert_eq!((frames.len(), loops), (3, 4));
        assert_eq!(frames.iter().map(|(_, delay)| *delay).collect::<Vec<_>>(), [90, 250, 90]);
        assert!(frames.iter().all(|(bitmap, _)| (bitmap.width(), bitmap.height()) == (480, 300)));
        let doc = Document::from_frames("star", frames, loops).unwrap();
        let opened = &doc.pages[0];
        assert_eq!((opened.states.len(), opened.gif_loops, opened.width, opened.height), (3, 4, 480, 300));
        assert_eq!(opened.states[1].delay_ms, 250);
        assert!((0..3).all(|state| opened.at_state(state).objects().count() == 1));
        // The states differ: the star has moved.
        let first = crate::export::render_page(&opened.at_state(0)).unwrap();
        let last = crate::export::render_page(&opened.at_state(2)).unwrap();
        assert!(first.data() != last.data());
        // It survives being saved as a document of ours.
        let again = crate::model::document_from_json(&crate::model::document_to_json(&doc).unwrap()).unwrap();
        assert_eq!((again.pages[0].gif_loops, again.pages[0].states.len()), (4, 3));
        // Played once through, the file does not ask to be repeated; a still picture is no animation.
        let mut once = page.clone();
        once.gif_loops = 1;
        assert_eq!(gif_frames(&encode(&once, &once.export).unwrap()).unwrap().1, 1);
        let still = Editor::new(Document::new(8, 8)).export_page();
        let still_gif = encode(&still, &ExportSettings { format: ExportFormat::Gif, ..ExportSettings::default() }).unwrap();
        assert!(gif_frames(&still_gif).is_none() && gif_frames(b"not a gif").is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Object, Rect, Shape};

    fn page(canvas: Option<Color>) -> Page {
        let mut page = Page::new("p", 40, 30);
        page.canvas = canvas;
        page.layers[0].objects.push(Object {
            id: 1,
            name: "r".into(),
            shape: Shape::Rect { radius: 0.0 },
            bounds: Rect { x: 5.0, y: 5.0, w: 10.0, h: 10.0 },
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
            fill: Some(Color::rgb(200, 10, 10)),
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        });
        page
    }

    fn settings(format: ExportFormat) -> ExportSettings {
        ExportSettings { format, ..ExportSettings::default() }
    }

    #[test]
    fn every_format_decodes_to_the_page_size() {
        for format in ExportFormat::ALL {
            let bytes = encode(&page(Some(Color::WHITE)), &settings(format)).unwrap();
            let decoded = image::load_from_memory(&bytes).unwrap_or_else(|e| panic!("{format:?}: {e}"));
            assert_eq!((decoded.width(), decoded.height()), (40, 30), "{format:?}");
        }
    }

    #[test]
    fn png32_keeps_transparency_and_png24_flattens_on_the_matte() {
        let source = page(None);
        let png32 = image::load_from_memory(&encode(&source, &settings(ExportFormat::Png32)).unwrap()).unwrap().to_rgba8();
        assert_eq!(png32.get_pixel(0, 0)[3], 0);
        let matte = ExportSettings { matte: Some(Color::rgb(0, 255, 0)), ..settings(ExportFormat::Png24) };
        let png24 = image::load_from_memory(&encode(&source, &matte).unwrap()).unwrap().to_rgba8();
        assert_eq!(png24.get_pixel(0, 0).0, [0, 255, 0, 255]);
        assert_eq!(png24.get_pixel(7, 7).0, [200, 10, 10, 255]);
    }

    #[test]
    fn index_transparency_survives_in_png8_and_gif() {
        for format in [ExportFormat::Png8, ExportFormat::Gif] {
            let options = ExportSettings { transparent: true, ..settings(format) };
            let decoded = image::load_from_memory(&encode(&page(None), &options).unwrap()).unwrap().to_rgba8();
            assert_eq!(decoded.get_pixel(0, 0)[3], 0, "{format:?}");
            assert_eq!(decoded.get_pixel(7, 7).0, [200, 10, 10, 255], "{format:?}");
        }
    }

    #[test]
    fn palette_never_exceeds_the_limit() {
        let rgba: Vec<u8> = (0..4096u32).flat_map(|i| [(i % 256) as u8, (i / 16) as u8, (i * 7 % 256) as u8, 255]).collect();
        for colors in [2u16, 16, 256] {
            let result = quantize(&rgba, colors);
            assert!(result.palette.len() / 3 <= colors as usize, "{colors}");
            assert!(result.indices.iter().all(|i| (*i as usize) < result.palette.len() / 3));
        }
    }

    /// A page filled with a noisy bitmap, which JPEG finds expensive.
    fn noisy_page() -> Page {
        let mut source = Page::new("p", 64, 64);
        let bitmap = crate::bitmap::Bitmap::from_rgba(64, 64, (0..64 * 64).flat_map(|i: u32| [(i * 37 % 256) as u8, (i * 91 % 256) as u8, (i * 13 % 256) as u8, 255]).collect()).unwrap();
        source.layers[0].objects.push(Object::new_bitmap(1, "b", bitmap, 0.0, 0.0));
        source
    }

    #[test]
    fn smoothing_makes_a_jpeg_smaller() {
        let source = noisy_page();
        let size = |smoothing: u8| encode(&source, &ExportSettings { smoothing, ..settings(ExportFormat::Jpeg) }).unwrap().len();
        assert!(size(8) < size(4) && size(4) < size(0), "{} {} {}", size(8), size(4), size(0));
        // Out-of-range values are treated as the strongest setting.
        assert_eq!(size(200), size(8));
    }

    #[test]
    fn the_jpeg_mask_is_written_at_the_selective_quality() {
        let mut source = noisy_page();
        let plain = ExportSettings { quality: 20, ..settings(ExportFormat::Jpeg) };
        let selective = ExportSettings { selective: true, selective_quality: 95, ..plain };
        let rough = encode(&source, &plain).unwrap().len();
        // Without a mask there is nothing to keep, and the file is the plain one.
        assert_eq!(encode(&source, &selective).unwrap().len(), rough);

        source.jpeg_mask = Some(std::sync::Arc::new(crate::bitmap::Mask::from_fn(64, 64, |x, _| x < 32)));
        let mixed = encode(&source, &selective).unwrap();
        let fine = encode(&source, &ExportSettings { quality: 95, ..plain }).unwrap().len();
        assert!(mixed.len() > rough * 2 && mixed.len() < fine, "{rough} {} {fine}", mixed.len());
        assert!(image::load_from_memory(&mixed).is_ok());
        // Switched off, the mask is ignored.
        assert_eq!(encode(&source, &plain).unwrap().len(), rough);
        // A mask left over from another page size is ignored too.
        source.jpeg_mask = Some(std::sync::Arc::new(crate::bitmap::Mask::from_fn(10, 10, |_, _| true)));
        assert_eq!(encode(&source, &selective).unwrap().len(), rough);
    }

    #[test]
    fn text_can_be_kept_at_the_selective_quality() {
        let mut source = noisy_page();
        let mut text = Object::new_bitmap(2, "t", crate::bitmap::Bitmap::from_rgba(1, 1, vec![0, 0, 0, 0]).unwrap(), 8.0, 8.0);
        text.shape = crate::model::Shape::Text { content: "a".into(), size: 20.0, style: Default::default() };
        text.bounds = crate::model::Rect { x: 8.0, y: 8.0, w: 32.0, h: 24.0 };
        source.layers[0].objects.push(text);
        let plain = ExportSettings { quality: 20, selective: true, selective_quality: 95, ..settings(ExportFormat::Jpeg) };
        let without = encode(&source, &plain).unwrap().len();
        let with = encode(&source, &ExportSettings { preserve_text: true, ..plain }).unwrap().len();
        assert!(with > without, "{with} against {without}");
    }

    #[test]
    fn button_symbols_can_be_kept_at_the_selective_quality() {
        let mut ed = crate::model::Editor::new(crate::model::Document::new(64, 64));
        let noise = noisy_page().layers[0].objects[0].clone();
        ed.add_object(noise);
        // A symbol placed over part of the page, first as a graphic and then as a button.
        let mut part = Object::new_bitmap(5, "s", crate::bitmap::Bitmap::from_rgba(1, 1, vec![0, 0, 0, 0]).unwrap(), 0.0, 0.0);
        part.bounds = crate::model::Rect { x: 0.0, y: 0.0, w: 32.0, h: 32.0 };
        let symbol = ed.add_library_symbol("b", &[part], crate::model::SymbolKind::Graphic);
        assert!(ed.place_symbol(symbol));
        let options = ExportSettings { quality: 20, selective: true, selective_quality: 95, preserve_buttons: true, ..settings(ExportFormat::Jpeg) };
        let as_graphic = encode(&ed.export_page(), &options).unwrap().len();
        ed.doc.symbols[0].kind = crate::model::SymbolKind::Button;
        let as_button = encode(&ed.export_page(), &options).unwrap().len();
        assert!(as_button > as_graphic, "{as_button} against {as_graphic}");
        // The option switched off leaves the button at the plain quality.
        assert_eq!(encode(&ed.export_page(), &ExportSettings { preserve_buttons: false, ..options }).unwrap().len(), as_graphic);
    }

    #[test]
    fn settings_saved_before_these_options_existed_still_load() {
        let old = r#"{"format":"Jpeg","quality":70,"colors":256,"transparent":false,"matte":null}"#;
        let loaded: ExportSettings = serde_json::from_str(old).unwrap();
        assert_eq!((loaded.smoothing, loaded.selective, loaded.selective_quality, loaded.preserve_text), (0, false, 90, false));
    }

    #[test]
    fn jpeg_quality_changes_the_size() {
        let mut source = Page::new("p", 64, 64);
        let bitmap = crate::bitmap::Bitmap::from_rgba(64, 64, (0..64 * 64).flat_map(|i: u32| [(i * 37 % 256) as u8, (i * 91 % 256) as u8, (i * 13 % 256) as u8, 255]).collect()).unwrap();
        source.layers[0].objects.push(Object::new_bitmap(1, "b", bitmap, 0.0, 0.0));
        let low = encode(&source, &ExportSettings { quality: 10, ..settings(ExportFormat::Jpeg) }).unwrap();
        let high = encode(&source, &ExportSettings { quality: 95, ..settings(ExportFormat::Jpeg) }).unwrap();
        assert!(low.len() < high.len());
    }

    #[test]
    fn animated_gif_writes_only_exported_states_with_their_delays() {
        let mut source = page(Some(Color::WHITE));
        source.add_state();
        source.add_state();
        source.states[1].exported = false;
        source.states[2].delay_ms = 250;
        let bytes = encode(&source, &settings(ExportFormat::AnimatedGif)).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options.read_info(std::io::Cursor::new(bytes)).unwrap();
        let mut delays = Vec::new();
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            delays.push(frame.delay);
        }
        assert_eq!(delays, vec![50, 25]);
    }

    #[test]
    fn file_extension_overrides_a_mismatched_format() {
        let panel = settings(ExportFormat::Png8);
        assert_eq!(settings_for_path(&panel, Path::new("a.png")).unwrap().format, ExportFormat::Png8);
        assert_eq!(settings_for_path(&panel, Path::new("a.jpg")).unwrap().format, ExportFormat::Jpeg);
        assert_eq!(settings_for_path(&panel, Path::new("a.gif")).unwrap().format, ExportFormat::Gif);
        assert!(settings_for_path(&panel, Path::new("a.bmp")).is_none());
    }
}
