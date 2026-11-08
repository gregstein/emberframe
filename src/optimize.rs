//! Export encoders driven by the Optimize panel.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, ensure, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use serde::{Deserialize, Serialize};
use tiny_skia::Pixmap;

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
            ExportFormat::AnimatedGif => "Animated GIF",
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
    ("GIF Web 216", ExportSettings { format: ExportFormat::Gif, quality: 80, colors: 216, transparent: false, matte: None }),
    ("GIF Web 256", ExportSettings { format: ExportFormat::Gif, quality: 80, colors: 256, transparent: false, matte: None }),
    ("Animated GIF", ExportSettings { format: ExportFormat::AnimatedGif, quality: 80, colors: 256, transparent: false, matte: None }),
    ("JPEG - Better Quality", ExportSettings { format: ExportFormat::Jpeg, quality: 80, colors: 256, transparent: false, matte: None }),
    ("JPEG - Smaller File", ExportSettings { format: ExportFormat::Jpeg, quality: 60, colors: 256, transparent: false, matte: None }),
    ("PNG 8", ExportSettings { format: ExportFormat::Png8, quality: 80, colors: 256, transparent: false, matte: None }),
    ("PNG 32", ExportSettings { format: ExportFormat::Png32, quality: 80, colors: 256, transparent: false, matte: None }),
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
        format if format.is_indexed() => format!("{} {} colors", format.label(), settings.colors),
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
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self { format: ExportFormat::Png32, quality: 80, colors: 256, transparent: false, matte: None }
    }
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
    ensure!(page.width <= MAX_GIF_SIZE && page.height <= MAX_GIF_SIZE, "GIF images cannot be larger than {MAX_GIF_SIZE} pixels");
    let animated = settings.format == ExportFormat::AnimatedGif;
    let states: Vec<usize> = if animated {
        (0..page.states.len()).filter(|i| page.states[*i].exported).collect()
    } else {
        vec![page.active_state]
    };
    ensure!(!states.is_empty(), "Every state is switched off in the States panel");
    let mut out = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut out, page.width as u16, page.height as u16, &[])?;
        if animated {
            encoder.set_repeat(gif::Repeat::Infinite)?;
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
                JpegEncoder::new_with_quality(&mut out, settings.quality.clamp(1, 100)).write_image(&rgb, w, h, ExtendedColorType::Rgb8)?;
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
            std::fs::write(path, bytes).with_context(|| format!("could not write {}", path.display()))
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
