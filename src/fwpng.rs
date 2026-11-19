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

//! Reading the layers of a Fireworks PNG.
//!
//! Fireworks keeps its editable document inside the PNG, in chunks of its own beside
//! the flat picture every program reads. The format is not published; what is here
//! was worked out from files written by Fireworks 8:
//!
//! - `mkTS` holds the document, zlib-compressed, as a tree of tagged values. A tag is
//!   three letters and a fourth for the kind of value, then the value in braces:
//!   `v{...}` a list of further tags, `i{..}` a whole number in hexadecimal, `f{..}` a
//!   number, `b{..}` yes or no, `s{..}` a text as a count and that many UTF-16 units.
//! - `mkBT` holds one square tile of a bitmap: a number that names it, then, after a
//!   header of 76 bytes in all, its pixels zlib-compressed, four bytes each in the
//!   order alpha, red, green, blue.
//!
//! Layers, their names and visibility, the states, and bitmap objects with a drop
//! shadow are read. Paths, text, groups and other effects were not in any file there
//! was to learn from; a document that has them is reported as not fully readable, so
//! that the caller can fall back on the flat picture.

use std::collections::HashMap;
use std::io::Read;

use anyhow::{bail, ensure, Context, Result};

use crate::bitmap::Bitmap;
use crate::effects::{FilterKind, LiveFilter};
use crate::i18n::{t, tf};
use crate::model::{Color, Document, Layer, Object, State};

/// What came out of a Fireworks PNG.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub layers: usize,
    pub bitmaps: usize,
    /// Objects of kinds that are not read: paths, text, groups and the like.
    pub unread_objects: usize,
    /// Effects other than a drop shadow.
    pub unread_effects: usize,
}

impl Report {
    /// Whether the document is all there, so that it can stand for the file.
    pub fn complete(&self) -> bool {
        self.unread_objects == 0
    }
}

#[derive(Debug)]
enum Value {
    List(Vec<(String, Value)>),
    Text(String),
}

impl Value {
    fn list(&self) -> &[(String, Value)] {
        match self {
            Value::List(items) => items,
            Value::Text(_) => &[],
        }
    }

    fn get(&self, tag: &str) -> Option<&Value> {
        self.list().iter().find(|(name, _)| name == tag).map(|(_, value)| value)
    }

    fn all<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a Value> {
        self.list().iter().filter(move |(name, _)| name == tag).map(|(_, value)| value)
    }

    fn text(&self, tag: &str) -> Option<&str> {
        match self.get(tag)? {
            Value::Text(text) => Some(text),
            Value::List(_) => None,
        }
    }

    fn number(&self, tag: &str) -> Option<f32> {
        self.text(tag)?.parse().ok().filter(|v: &f32| v.is_finite())
    }

    fn hex(&self, tag: &str) -> Option<u32> {
        u32::from_str_radix(self.text(tag)?, 16).ok()
    }

    fn flag(&self, tag: &str) -> bool {
        self.text(tag) == Some("1")
    }
}

/// Reads the tags of a list up to the brace that closes it. Returns them and where that brace is.
fn parse(data: &[u8], mut at: usize, depth: usize) -> Result<(Vec<(String, Value)>, usize)> {
    ensure!(depth < 64, "nested too deep");
    let mut items = Vec::new();
    while at < data.len() && data[at] != b'}' {
        ensure!(at + 5 <= data.len() && data[at + 4] == b'{', "not a tag");
        let name = String::from_utf8_lossy(&data[at..at + 3]).into_owned();
        let kind = data[at + 3];
        at += 5;
        let value = match kind {
            b'v' => {
                let (inner, end) = parse(data, at, depth + 1)?;
                at = end;
                Value::List(inner)
            }
            b's' => {
                ensure!(at + 2 <= data.len(), "cut short");
                let units = u16::from_be_bytes([data[at], data[at + 1]]) as usize;
                let end = at + 2 + units * 2;
                ensure!(end <= data.len(), "cut short");
                let text: Vec<u16> = data[at + 2..end].chunks_exact(2).map(|pair| u16::from_be_bytes([pair[0], pair[1]])).collect();
                at = end;
                Value::Text(String::from_utf16_lossy(&text))
            }
            _ => {
                let end = data[at..].iter().position(|b| *b == b'}').context("cut short")? + at;
                let text = String::from_utf8_lossy(&data[at..end]).into_owned();
                at = end;
                Value::Text(text)
            }
        };
        ensure!(at < data.len() && data[at] == b'}', "tag not closed");
        at += 1;
        items.push((name, value));
    }
    Ok((items, at))
}

/// The chunks of a PNG, each with its name.
fn chunks(bytes: &[u8]) -> Vec<(&[u8], &[u8])> {
    let mut found = Vec::new();
    let mut at = 8;
    while at + 8 <= bytes.len() {
        let length = u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as usize;
        let Some(data) = bytes.get(at + 8..at + 8 + length) else { break };
        found.push((&bytes[at + 4..at + 8], data));
        at += 12 + length;
    }
    found
}

/// Whether a file is a PNG that carries a Fireworks document.
pub fn is_fireworks(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG\r\n\x1a\n") && chunks(bytes).iter().any(|(name, _)| *name == b"mkTS")
}

fn inflate(data: &[u8], limit: u64) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(data).take(limit).read_to_end(&mut out).context("not zlib data")?;
    Ok(out)
}

const TILE_HEADER: usize = 76;
/// No document is let grow past this many pixels in one bitmap.
const MAX_PIXELS: u64 = 16384 * 16384;

/// A bitmap object put together from its tiles, which run across and then down.
fn bitmap_of(element: &Value, tiles: &HashMap<u32, &[u8]>) -> Result<Bitmap> {
    let (width, height, tile) = (element.hex("WPX").context("no width")?, element.hex("HPX").context("no height")?, element.hex("TSZ").unwrap_or(128));
    ensure!(width > 0 && height > 0 && (1..=4096).contains(&tile) && width as u64 * height as u64 <= MAX_PIXELS, "bitmap of no size");
    let across = width.div_ceil(tile);
    let ids: Vec<u32> = element.get("TIL").map_or(&[][..], Value::list).iter().filter_map(|(name, value)| match value {
        Value::Text(text) if name == "TID" => u32::from_str_radix(text, 16).ok(),
        _ => None,
    }).collect();
    ensure!(ids.len() as u32 >= across * height.div_ceil(tile), "tiles missing");
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    for (index, id) in ids.iter().enumerate() {
        let packed = tiles.get(id).with_context(|| format!("tile {id:x} missing"))?;
        let pixels = inflate(packed, tile as u64 * tile as u64 * 4)?;
        ensure!(pixels.len() as u64 == tile as u64 * tile as u64 * 4, "tile cut short");
        let (left, top) = ((index as u32 % across) * tile, (index as u32 / across) * tile);
        for y in 0..tile.min(height.saturating_sub(top)) {
            for x in 0..tile.min(width.saturating_sub(left)) {
                let from = ((y * tile + x) * 4) as usize;
                let to = (((top + y) * width + left + x) * 4) as usize;
                // Stored as alpha, red, green, blue.
                rgba[to..to + 4].copy_from_slice(&[pixels[from + 1], pixels[from + 2], pixels[from + 3], pixels[from]]);
            }
        }
    }
    Bitmap::from_rgba(width, height, rgba).context("bitmap of no size")
}

/// The live filter an effect of the file stands for, if it is one that is read.
fn filter_of(effect: &Value) -> Option<LiveFilter> {
    let settings: HashMap<&str, &str> = effect.get("EPS")?.all("DCE").filter_map(|entry| Some((entry.text("DCK")?, entry.text("DCV")?))).collect();
    if *settings.get("mkbFile_WriteOnly_TemporaryEffectUiName")? != "Drop Shadow" {
        return None;
    }
    let number = |key: &str| settings.get(key).and_then(|v| v.parse::<f32>().ok());
    let mut filter = LiveFilter::new(FilterKind::DropShadow);
    // The color is written as #rrggbbaa, and its alpha is the strength of the shadow.
    let color = settings.get("ShadowColor").map(|c| c.trim_start_matches('#')).filter(|c| c.len() == 8 && c.is_ascii());
    if let Some(color) = color {
        filter.color = Color::parse_hex(&color[..6]);
        if let Ok(alpha) = u8::from_str_radix(&color[6..], 16) {
            filter.values[1] = (alpha as f32 / 255.0 * 100.0).round();
        }
    }
    filter.values[0] = number("ShadowDistance")?;
    filter.values[2] = number("ShadowBlur")?;
    filter.values[3] = number("ShadowAngle")?;
    filter.enabled = settings.get("EffectIsVisible") != Some(&"false");
    Some(filter)
}

/// Reads the document of a Fireworks PNG. `None` if the file is not one.
pub fn read(bytes: &[u8]) -> Result<Option<(Document, Report)>> {
    if !is_fireworks(bytes) {
        return Ok(None);
    }
    let parts = chunks(bytes);
    let packed = parts.iter().find(|(name, _)| *name == b"mkTS").map(|(_, data)| *data).unwrap_or_default();
    let text = inflate(packed, 256 << 20)?;
    let (top, _) = parse(&text, 0, 0)?;
    let root = Value::List(top);
    let root = root.get("MKB").context("no document")?;
    let tiles: HashMap<u32, &[u8]> = parts
        .iter()
        .filter(|(name, data)| *name == b"mkBT" && data.len() > TILE_HEADER)
        .map(|(_, data)| (u32::from_be_bytes([data[4], data[5], data[6], data[7]]), &data[TILE_HEADER..]))
        .collect();
    let (width, height) = (root.number("WID").context("no width")?, root.number("HIT").context("no height")?);
    ensure!((1.0..=30000.0).contains(&width) && (1.0..=30000.0).contains(&height), "canvas of no size");
    let mut doc = Document::new(width.round() as u32, height.round() as u32);
    // The canvas color is alpha, red, green, blue; no alpha is a see-through canvas.
    let canvas = root.hex("BGC").unwrap_or(0xffff_ffff);
    doc.pages[0].canvas = (canvas >> 24 != 0).then(|| Color { r: (canvas >> 16) as u8, g: (canvas >> 8) as u8, b: canvas as u8, a: (canvas >> 24) as u8 });
    let frames = (root.hex("FRC").unwrap_or(1) as usize).clamp(1, 1000);
    let mut report = Report::default();
    let mut next_id = 1;
    let mut layers: Vec<Layer> = Vec::new();
    // For each state after the first, what each layer shows in it.
    let mut later: Vec<Vec<Vec<Object>>> = vec![Vec::new(); frames - 1];
    for entry in root.get("LYL").context("no layers")?.all("LAY") {
        let cells: Vec<&Value> = entry.get("CLL").map(|cells| cells.all("CEL").collect()).unwrap_or_default();
        let web = entry.flag("URL");
        let mut shown: Vec<Vec<Object>> = Vec::new();
        for cell in cells.iter().take(frames) {
            let mut objects = Vec::new();
            for (kind, element) in cell.get("ELM").map_or(&[][..], Value::list) {
                if kind != "IMG" || web {
                    report.unread_objects += 1;
                    continue;
                }
                let bitmap = bitmap_of(element, &tiles)?;
                let (x, y) = (element.number("XLC").unwrap_or(0.0), element.number("YLC").unwrap_or(0.0));
                let mut object = Object::new_bitmap(next_id, t("Bitmap"), bitmap, x, y);
                next_id += 1;
                object.locked = element.flag("LCK");
                for effect in element.all("EFL") {
                    match filter_of(effect) {
                        Some(filter) => object.filters.push(filter),
                        None => report.unread_effects += 1,
                    }
                }
                report.bitmaps += 1;
                // The file lists a layer's objects from the top one down.
                objects.insert(0, object);
            }
            shown.push(objects);
        }
        // The Web Layer of the file is left out: this program keeps its own.
        if web {
            continue;
        }
        shown.resize(frames, Vec::new());
        let mut layer = Layer::new(entry.text("LNM").filter(|name| !name.is_empty()).map_or_else(|| tf("Layer {}", &[&(layers.len() + 1)]), str::to_string));
        layer.visible = entry.get("VIF").is_none_or(|v| v.text("VIS") != Some("0"));
        let mut shown = shown.into_iter();
        layer.objects = shown.next().unwrap_or_default();
        for (state, objects) in later.iter_mut().zip(shown) {
            state.push(objects);
        }
        layers.push(layer);
    }
    if layers.is_empty() {
        bail!("no layers");
    }
    report.layers = layers.len();
    let page = &mut doc.pages[0];
    page.active_layer = layers.len() - 1;
    page.layers = layers;
    for (index, objects) in later.into_iter().enumerate() {
        let mut state = State::new(tf("State {}", &[&(index + 2)]));
        state.objects = objects;
        page.states.push(state);
    }
    doc.next_id = next_id;
    Ok(Some((doc, report)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn deflate(data: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn chunk(name: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(name);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0; 4]);
        out
    }

    fn name(text: &str) -> Vec<u8> {
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut out = (units.len() as u16).to_be_bytes().to_vec();
        out.extend(units.iter().flat_map(|unit| unit.to_be_bytes()));
        out
    }

    /// A tile of one color: 4 by 4 pixels, written alpha first.
    fn tile(id: u32, argb: [u8; 4]) -> Vec<u8> {
        let mut data = vec![0xfa, 0xce, 0xca, 0xfe];
        data.extend_from_slice(&id.to_be_bytes());
        data.resize(TILE_HEADER, 0);
        data.extend(deflate(&argb.repeat(16)));
        chunk(b"mkBT", &data)
    }

    /// A file as Fireworks writes it, with two layers and two states, made here from nothing.
    fn sample(extra_element: &str) -> Vec<u8> {
        let mut tree = b"MKBv{FRCi{2}WIDf{10}HITf{8}BGCi{ffffff}LYLv{".to_vec();
        let image = |x: u32, id: u32, effect: &str| format!("IMGv{{LCKb{{0}}XLCf{{{x}}}YLCf{{1}}WPXi{{6}}HPXi{{3}}TSZi{{4}}TILv{{TIDi{{{id:x}}}TIDi{{{id:x}}}}}{effect}DTAv{{}}}}");
        let shadow = "EFLv{CATs{\0\0}EPSv{DCEv{DCKs{K1}DCVs{V1}}}}";
        let _ = shadow;
        tree.extend(b"LAYv{URLb{0}LNMs{");
        tree.extend(name("Arka Plan"));
        tree.extend(format!("}}CLLv{{CELv{{ELMv{{{}{}{}}}}}CELv{{ELMv{{}}}}}}VIFv{{VISb{{0}}}}}}", image(2, 0x7f01, ""), image(0, 0x7f02, ""), extra_element).as_bytes());
        tree.extend(b"LAYv{URLb{0}LNMs{");
        tree.extend(name("Layer 1"));
        tree.extend(format!("}}CLLv{{CELv{{ELMv{{}}}}CELv{{ELMv{{{}}}}}}}VIFv{{VISb{{1}}}}}}", image(4, 0x7f02, "")).as_bytes());
        tree.extend(b"LAYv{URLb{1}LNMs{");
        tree.extend(name("Web Layer"));
        tree.extend(b"}CLLv{CELv{ELMv{}}}}}}");
        let mut file = b"\x89PNG\r\n\x1a\n".to_vec();
        file.extend(chunk(b"IHDR", &[0; 13]));
        file.extend(chunk(b"mkTS", &deflate(&tree)));
        file.extend(tile(0x7f01, [255, 200, 10, 20]));
        file.extend(tile(0x7f02, [128, 1, 2, 3]));
        file.extend(chunk(b"IEND", &[]));
        file
    }

    #[test]
    fn the_layers_states_and_bitmaps_of_a_fireworks_png_are_read() {
        let (doc, report) = read(&sample("")).unwrap().unwrap();
        assert_eq!(report, Report { layers: 2, bitmaps: 3, unread_objects: 0, unread_effects: 0 });
        assert!(report.complete());
        let page = &doc.pages[0];
        assert_eq!((page.width, page.height, page.canvas), (10, 8, None));
        assert_eq!(page.layers.iter().map(|l| (l.name.as_str(), l.visible, l.objects.len())).collect::<Vec<_>>(), [("Arka Plan", false, 2), ("Layer 1", true, 0)]);
        // The first object of the file is the top one, so it is the last of the layer here.
        let top = &page.layers[0].objects[1];
        assert_eq!((top.bounds.x, top.bounds.y, top.bounds.w, top.bounds.h), (2.0, 1.0, 6.0, 3.0));
        let crate::model::Shape::Image { bitmap } = &top.shape else { panic!("not a bitmap") };
        // Six pixels across come from two tiles of four; the colors are turned from alpha-first.
        assert_eq!(&bitmap.rgba()[..4], &[200, 10, 20, 255]);
        assert_eq!(&bitmap.rgba()[5 * 4..6 * 4], &[200, 10, 20, 255]);
        // The second state shows nothing in the first layer and one bitmap in the second.
        assert_eq!(page.states.len(), 2);
        assert_eq!(page.states[1].objects.iter().map(Vec::len).collect::<Vec<_>>(), [0, 1]);
        assert_eq!(doc.next_id, 4);
        // The document survives being saved and opened as one of ours.
        let again = crate::model::document_from_json(&crate::model::document_to_json(&doc).unwrap()).unwrap();
        assert_eq!(again.pages[0].layers.len(), 2);
    }

    #[test]
    fn what_cannot_be_read_is_counted_and_an_ordinary_png_is_left_alone() {
        let (_, report) = read(&sample("PTHv{XLCf{0}}")).unwrap().unwrap();
        assert_eq!((report.unread_objects, report.bitmaps), (1, 3));
        assert!(!report.complete());
        assert!(read(b"\x89PNG\r\n\x1a\n").unwrap().is_none());
        assert!(read(b"not a png").unwrap().is_none());
        // A file cut short is an error, not a crash.
        let whole = sample("");
        let mut broken = whole.clone();
        let at = broken.windows(4).position(|w| w == b"mkBT").unwrap();
        broken.truncate(at + 40);
        assert!(read(&broken).is_err());
    }

    /// Checks the reader against a real file from Fireworks: `FWPNG_SAMPLE=path cargo test -- --ignored fireworks_sample`.
    /// The bitmaps, laid over one another, must give the flat picture the file carries.
    #[test]
    #[ignore]
    fn fireworks_sample_matches_its_flat_picture() {
        let Ok(path) = std::env::var("FWPNG_SAMPLE") else { return };
        let bytes = std::fs::read(&path).unwrap();
        let (mut doc, report) = read(&bytes).unwrap().unwrap();
        doc.pages[0].layers.iter_mut().for_each(|l| l.visible = true);
        let flat = image::load_from_memory(&bytes).unwrap().to_rgba8();
        let differing = |doc: &Document| {
            let drawn = crate::export::render_page(&doc.pages[0]).unwrap();
            let mut count = 0;
            let mut worst = 0;
            for (ours, theirs) in drawn.pixels().iter().zip(flat.pixels()) {
                let ours = ours.demultiply();
                if ours.alpha() == 0 && theirs[3] == 0 {
                    continue;
                }
                let off = [ours.red().abs_diff(theirs[0]), ours.green().abs_diff(theirs[1]), ours.blue().abs_diff(theirs[2]), ours.alpha().abs_diff(theirs[3])].into_iter().max().unwrap_or(0);
                worst = worst.max(off);
                if off > 1 {
                    count += 1;
                }
            }
            (count, worst)
        };
        if std::env::var("FWPNG_SHOW").is_ok() {
            let drawn = crate::export::render_page(&doc.pages[0]).unwrap();
            for y in 0..flat.height() {
                let ours: String = (0..flat.width()).map(|x| format!("{:02x}", drawn.pixel(x, y).unwrap().alpha())).collect::<Vec<_>>().join(" ");
                let theirs: String = (0..flat.width()).map(|x| format!("{:02x}", flat.get_pixel(x, y)[3])).collect::<Vec<_>>().join(" ");
                println!("{ours}   |   {theirs}");
            }
            for object in doc.pages[0].layers.iter().flat_map(|l| l.objects.iter()) {
                println!("{:?} {:?}", object.bounds, object.filters);
            }
        }
        let with_effects = differing(&doc);
        let had_effects = doc.pages[0].layers.iter().flat_map(|l| l.objects.iter()).any(|o| !o.filters.is_empty());
        for object in doc.pages[0].layers.iter_mut().flat_map(|l| l.objects.iter_mut()) {
            object.filters.clear();
        }
        let bare = differing(&doc);
        println!("{path}: {report:?}; of {} pixels, {} differ with the effects drawn (worst by {}), {} with none", flat.width() * flat.height(), with_effects.0, with_effects.1, bare.0);
        assert!(had_effects || report.unread_effects > 0 || bare.0 == 0);
    }
}
