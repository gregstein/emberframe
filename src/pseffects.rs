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

//! Photoshop Live Effects: Photoshop's layer styles, kept together in one live filter.
//!
//! Each effect has a number, which is where its settings and its color are stored.
//! The dialog lists the effects in [`ORDER`], and [`rows`] says what each one is set with.

use std::sync::LazyLock;

use crate::blend::BlendMode;
use crate::effects::{blur_channel, direction, noise, over, Layer, LiveFilter, Setting};
use crate::i18n::{t, tc};
use crate::model::Color;

pub const DROP_SHADOW: usize = 0;
pub const INNER_SHADOW: usize = 1;
pub const OUTER_GLOW: usize = 2;
pub const INNER_GLOW: usize = 3;
pub const BEVEL: usize = 4;
pub const SATIN: usize = 5;
pub const COLOR_OVERLAY: usize = 6;
pub const STROKE: usize = 7;
pub const GRADIENT_OVERLAY: usize = 8;
pub const PATTERN_OVERLAY: usize = 9;

/// The names of the effects, by their numbers.
pub const NAMES: [&str; 10] =
    ["Drop Shadow", "Inner Shadow", "Outer Glow", "Inner Glow", "Bevel and Emboss", "Satin", "Color Overlay", "Stroke", "Gradient Overlay", "Pattern Overlay"];

/// The effects in the order the dialog lists them, which is Photoshop's.
pub const ORDER: [usize; 10] =
    [DROP_SHADOW, INNER_SHADOW, OUTER_GLOW, INNER_GLOW, BEVEL, SATIN, COLOR_OVERLAY, GRADIENT_OVERLAY, PATTERN_OVERLAY, STROKE];

/// The colors beyond one for each effect: the shade of a bevel, and the far ends of the two gradients.
pub const BEVEL_SHADE: usize = 10;
pub const GRADIENT_END: usize = 11;
pub const STROKE_END: usize = 12;

/// The colors the effects start with, by their numbers.
pub fn start_colors() -> Vec<Color> {
    let (black, white, glow, red) = (Color::BLACK, Color::rgb(255, 255, 255), Color::rgb(255, 255, 190), Color::rgb(255, 0, 0));
    vec![black, black, glow, glow, white, black, red, red, black, black, black, white, white]
}

// The places an effect keeps its settings in. The first five are the same for every effect.
const ON: usize = 0;
const OPACITY: usize = 1;
const ANGLE: usize = 2;
const DISTANCE: usize = 3;
const SIZE: usize = 4;
const BLEND: usize = 5;
/// Spread, choke, a bevel's altitude, or the pattern of a stroke.
const SPREAD: usize = 6;
/// Noise, or the blend mode of a bevel's shade.
const NOISE: usize = 7;
/// Technique of a glow, style of a bevel or a gradient, fill type of a stroke, or which pattern.
const STYLE: usize = 8;
/// Source of an inner glow, technique of a bevel, or position of a stroke.
const SOURCE: usize = 9;
/// A box to tick: knock out, invert or reverse. For a bevel, the opacity of its shade.
const FLAG: usize = 10;
/// Align with layer, or the direction of a bevel.
const ALIGN: usize = 11;
/// Range of a glow, soften of a bevel, or scale of a gradient or pattern.
const RANGE: usize = 12;
const SLOTS: usize = 13;

/// How many numbers Photoshop Live Effects stores.
pub const COUNT: usize = 130;

/// Where an effect's setting is among the numbers of the filter. The first eight
/// effects keep their first five settings where they were before the others came.
pub fn index(effect: usize, slot: usize) -> usize {
    match (effect, slot) {
        (0..=7, 0..=4) => effect * 5 + slot,
        (_, 0..=4) => 120 + (effect - 8) * 5 + slot,
        _ => 40 + effect * 8 + (slot - 5),
    }
}

/// One line of an effect's settings in the dialog.
#[derive(Clone, Copy, Debug)]
pub enum Row {
    Heading(&'static str),
    Number { name: &'static str, slot: usize, min: f32, max: f32, start: f32 },
    Toggle { name: &'static str, slot: usize, start: bool },
    List { name: &'static str, slot: usize, items: &'static [&'static str], start: usize },
    Blend { name: &'static str, slot: usize, start: BlendMode },
    Color { name: &'static str, color: usize },
    /// One of the filter's gradients, edited in place.
    Gradient { name: &'static str, gradient: usize },
}

const fn number(name: &'static str, slot: usize, min: f32, max: f32, start: f32) -> Row {
    Row::Number { name, slot, min, max, start }
}

const fn opacity(start: f32) -> Row {
    number("Opacity:", OPACITY, 0.0, 100.0, start)
}

const fn angle(start: f32) -> Row {
    number("Angle:", ANGLE, 0.0, 360.0, start)
}

const fn blend(start: BlendMode) -> Row {
    Row::Blend { name: "Blend Mode:", slot: BLEND, start }
}

const fn color(effect: usize) -> Row {
    Row::Color { name: "Color:", color: effect }
}

/// The gradients of the filter, by their numbers: the overlay's, the stroke's, and the two glows'.
pub const OVERLAY_GRADIENT: usize = 0;
pub const STROKE_GRADIENT: usize = 1;
pub const OUTER_GLOW_GRADIENT: usize = 2;
pub const INNER_GLOW_GRADIENT: usize = 3;

/// A stop of a gradient: where it is, from 0 to 1, and its color, whose alpha is how opaque the gradient is there.
pub type Stop = (f32, Color);

/// The gradient a filter has when none was made for it: the two colors older
/// files kept, or for a glow its color fading to nothing.
pub fn start_gradient(filter: &LiveFilter, gradient: usize) -> Vec<Stop> {
    let clear = |color: Color| Color { a: 0, ..color };
    match gradient {
        OVERLAY_GRADIENT => vec![(0.0, filter.ps_color(GRADIENT_OVERLAY)), (1.0, filter.ps_color(GRADIENT_END))],
        STROKE_GRADIENT => vec![(0.0, filter.ps_color(STROKE)), (1.0, filter.ps_color(STROKE_END))],
        OUTER_GLOW_GRADIENT => vec![(0.0, filter.ps_color(OUTER_GLOW)), (1.0, clear(filter.ps_color(OUTER_GLOW)))],
        _ => vec![(0.0, filter.ps_color(INNER_GLOW)), (1.0, clear(filter.ps_color(INNER_GLOW)))],
    }
}

/// Gradients to start from, by name.
pub const PRESETS: [(&str, &[(f32, [u8; 4])]); 10] = [
    ("Black, White", &[(0.0, [0, 0, 0, 255]), (1.0, [255, 255, 255, 255])]),
    ("White, Black", &[(0.0, [255, 255, 255, 255]), (1.0, [0, 0, 0, 255])]),
    ("Black, Transparent", &[(0.0, [0, 0, 0, 255]), (1.0, [0, 0, 0, 0])]),
    ("Blue, Red, Yellow", &[(0.0, [0, 0, 255, 255]), (0.5, [255, 0, 0, 255]), (1.0, [255, 255, 0, 255])]),
    ("Red, Green, Blue", &[(0.0, [255, 0, 0, 255]), (0.5, [0, 255, 0, 255]), (1.0, [0, 0, 255, 255])]),
    ("Violet, Orange", &[(0.0, [41, 10, 89, 255]), (1.0, [255, 124, 0, 255])]),
    ("Copper", &[(0.0, [151, 70, 26, 255]), (0.3, [251, 216, 197, 255]), (0.83, [108, 46, 22, 255]), (1.0, [239, 219, 205, 255])]),
    ("Silver", &[(0.0, [110, 110, 110, 255]), (0.3, [245, 245, 245, 255]), (0.6, [140, 140, 140, 255]), (1.0, [235, 235, 235, 255])]),
    ("Pastels", &[(0.0, [255, 204, 204, 255]), (0.33, [255, 255, 204, 255]), (0.66, [204, 255, 204, 255]), (1.0, [204, 204, 255, 255])]),
    (
        "Spectrum",
        &[(0.0, [255, 0, 0, 255]), (0.17, [255, 255, 0, 255]), (0.33, [0, 255, 0, 255]), (0.5, [0, 255, 255, 255]), (0.67, [0, 0, 255, 255]), (0.83, [255, 0, 255, 255]), (1.0, [255, 0, 0, 255])],
    ),
];

pub fn preset(which: usize) -> Vec<Stop> {
    PRESETS[which.min(PRESETS.len() - 1)].1.iter().map(|(place, [r, g, b, a])| (*place, Color { r: *r, g: *g, b: *b, a: *a })).collect()
}

/// The color of a gradient at a place from 0 to 1, and how opaque it is there.
pub fn gradient_color(stops: &[Stop], place: f32) -> ([f32; 3], f32) {
    let parts = |color: Color| (rgb(color), color.a as f32 / 255.0);
    let Some(first) = stops.first() else { return ([0.0; 3], 1.0) };
    if place <= first.0 {
        return parts(first.1);
    }
    for pair in stops.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        if place <= to.0 {
            let part = if to.0 > from.0 { (place - from.0) / (to.0 - from.0) } else { 1.0 };
            let ((a, a_opacity), (b, b_opacity)) = (parts(from.1), parts(to.1));
            return ([0, 1, 2].map(|c| a[c] + (b[c] - a[c]) * part), a_opacity + (b_opacity - a_opacity) * part);
        }
    }
    parts(stops[stops.len() - 1].1)
}

/// A gradient with its opacity stops worked into its color stops: at every place
/// either kind has a stop, the color found there with the opacity found there.
/// With no opacity stops the colors are as they are.
pub fn baked(colors: &[Stop], opacity: &[Stop]) -> Vec<Stop> {
    if opacity.is_empty() {
        return colors.to_vec();
    }
    let mut places: Vec<f32> = colors.iter().chain(opacity).map(|stop| stop.0).collect();
    places.sort_by(f32::total_cmp);
    places.dedup_by(|a, b| (*a - *b).abs() < 0.0005);
    let channel = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    places
        .into_iter()
        .map(|place| {
            let (color, own) = gradient_color(colors, place);
            let (_, track) = gradient_color(opacity, place);
            (place, Color { r: channel(color[0]), g: channel(color[1]), b: channel(color[2]), a: channel(own * track) })
        })
        .collect()
}

/// A press under a gradient at `place`: the stop within `reach` of it is taken
/// hold of, or a new one is put there in the color the gradient has at that place.
pub fn stop_pressed(mut stops: Vec<Stop>, place: f32, reach: f32) -> (Vec<Stop>, usize) {
    let place = place.clamp(0.0, 1.0);
    let nearest = stops.iter().enumerate().min_by(|a, b| (a.1.0 - place).abs().total_cmp(&(b.1.0 - place).abs()));
    if let Some((index, _)) = nearest.filter(|(_, stop)| (stop.0 - place).abs() <= reach) {
        return (stops, index);
    }
    let (color, opacity) = gradient_color(&stops, place);
    let channel = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    let index = stops.iter().position(|stop| stop.0 > place).unwrap_or(stops.len());
    stops.insert(index, (place, Color { r: channel(color[0]), g: channel(color[1]), b: channel(color[2]), a: channel(opacity) }));
    (stops, index)
}

/// The held stop dragged to `place`. Stops can pass one another. Pulled `away`
/// from the gradient it is taken off, unless only two are left. Gives the stops
/// and where the held one is now, if it is still there.
pub fn stop_dragged(mut stops: Vec<Stop>, index: usize, place: f32, away: bool) -> (Vec<Stop>, Option<usize>) {
    if index >= stops.len() {
        return (stops, None);
    }
    if away && stops.len() > 2 {
        stops.remove(index);
        return (stops, None);
    }
    let place = (place.clamp(0.0, 1.0) * 100.0).round() / 100.0;
    stops[index].0 = place;
    // Sorting keeps the order of stops at the same place, so the held one is found again by counting.
    let before = stops[..index].iter().filter(|stop| stop.0 <= place).count() + stops[index + 1..].iter().filter(|stop| stop.0 < place).count();
    let held = stops.remove(index);
    stops.insert(before, held);
    (stops, Some(before))
}

/// Which gradient an effect has, if its settings now call for one.
pub fn gradient_of(filter: &LiveFilter, effect: usize) -> Option<usize> {
    rows(effect).iter().filter(|row| row_shows(filter, effect, row)).find_map(|row| match *row {
        Row::Gradient { gradient, .. } => Some(gradient),
        _ => None,
    })
}

pub const TECHNIQUES: [&str; 2] = ["Softer", "Precise"];
pub const SOURCES: [&str; 2] = ["Center", "Edge"];
pub const BEVEL_STYLES: [&str; 5] = ["Outer Bevel", "Inner Bevel", "Emboss", "Pillow Emboss", "Stroke Emboss"];
pub const BEVEL_TECHNIQUES: [&str; 3] = ["Smooth", "Chisel Hard", "Chisel Soft"];
pub const DIRECTIONS: [&str; 2] = ["Up", "Down"];
pub const POSITIONS: [&str; 3] = ["Outside", "Inside", "Center"];
pub const FILL_TYPES: [&str; 3] = ["Color", "Gradient", "Pattern"];
pub const GLOW_FILLS: [&str; 2] = ["Color", "Gradient"];
pub const GRADIENT_STYLES: [&str; 5] = ["Linear", "Radial", "Angle", "Reflected", "Diamond"];
pub const PATTERNS: [&str; 7] = ["Checkerboard", "Horizontal Stripes", "Vertical Stripes", "Diagonal Stripes", "Dots", "Grid", "Bricks"];

const SHADOW_ROWS: [Row; 9] = [
    blend(BlendMode::Multiply),
    color(DROP_SHADOW),
    opacity(75.0),
    angle(120.0),
    number("Distance:", DISTANCE, 0.0, 250.0, 5.0),
    number("Spread:", SPREAD, 0.0, 100.0, 0.0),
    number("Size:", SIZE, 0.0, 250.0, 5.0),
    number("Noise:", NOISE, 0.0, 100.0, 0.0),
    Row::Toggle { name: "Layer Knocks Out Drop Shadow", slot: FLAG, start: true },
];
const INNER_SHADOW_ROWS: [Row; 8] = [
    blend(BlendMode::Multiply),
    color(INNER_SHADOW),
    opacity(75.0),
    angle(120.0),
    number("Distance:", DISTANCE, 0.0, 250.0, 5.0),
    number("Choke:", SPREAD, 0.0, 100.0, 0.0),
    number("Size:", SIZE, 0.0, 250.0, 5.0),
    number("Noise:", NOISE, 0.0, 100.0, 0.0),
];
const OUTER_GLOW_ROWS: [Row; 11] = [
    blend(BlendMode::Screen),
    opacity(75.0),
    number("Noise:", NOISE, 0.0, 100.0, 0.0),
    Row::List { name: "Fill Type:", slot: FLAG, items: &GLOW_FILLS, start: 0 },
    color(OUTER_GLOW),
    Row::Gradient { name: "Gradient:", gradient: OUTER_GLOW_GRADIENT },
    number("Jitter:", ALIGN, 0.0, 100.0, 0.0),
    Row::List { name: "Technique:", slot: STYLE, items: &TECHNIQUES, start: 0 },
    number("Spread:", SPREAD, 0.0, 100.0, 0.0),
    number("Size:", SIZE, 0.0, 250.0, 5.0),
    number("Range:", RANGE, 1.0, 100.0, 50.0),
];
const INNER_GLOW_ROWS: [Row; 12] = [
    blend(BlendMode::Screen),
    opacity(75.0),
    number("Noise:", NOISE, 0.0, 100.0, 0.0),
    Row::List { name: "Fill Type:", slot: FLAG, items: &GLOW_FILLS, start: 0 },
    color(INNER_GLOW),
    Row::Gradient { name: "Gradient:", gradient: INNER_GLOW_GRADIENT },
    number("Jitter:", ALIGN, 0.0, 100.0, 0.0),
    Row::List { name: "Technique:", slot: STYLE, items: &TECHNIQUES, start: 0 },
    Row::List { name: "Source:", slot: SOURCE, items: &SOURCES, start: 1 },
    number("Choke:", SPREAD, 0.0, 100.0, 0.0),
    number("Size:", SIZE, 0.0, 250.0, 5.0),
    number("Range:", RANGE, 1.0, 100.0, 50.0),
];
const BEVEL_ROWS: [Row; 15] = [
    Row::List { name: "Style:", slot: STYLE, items: &BEVEL_STYLES, start: 1 },
    Row::List { name: "Technique:", slot: SOURCE, items: &BEVEL_TECHNIQUES, start: 0 },
    number("Depth:", DISTANCE, 1.0, 1000.0, 100.0),
    Row::List { name: "Direction:", slot: ALIGN, items: &DIRECTIONS, start: 0 },
    number("Size:", SIZE, 0.0, 250.0, 5.0),
    number("Soften:", RANGE, 0.0, 16.0, 0.0),
    Row::Heading("Shading"),
    angle(120.0),
    number("Altitude:", SPREAD, 0.0, 90.0, 30.0),
    Row::Blend { name: "Highlight:", slot: BLEND, start: BlendMode::Screen },
    Row::Color { name: "Color:", color: BEVEL },
    opacity(75.0),
    Row::Blend { name: "Shadow:", slot: NOISE, start: BlendMode::Multiply },
    Row::Color { name: "Color:", color: BEVEL_SHADE },
    number("Opacity:", FLAG, 0.0, 100.0, 75.0),
];
const SATIN_ROWS: [Row; 8] = [
    blend(BlendMode::Multiply),
    color(SATIN),
    opacity(50.0),
    angle(19.0),
    number("Distance:", DISTANCE, 1.0, 250.0, 11.0),
    number("Size:", SIZE, 0.0, 250.0, 14.0),
    Row::Toggle { name: "Invert", slot: FLAG, start: true },
    Row::Toggle { name: "Anti-aliased", slot: ALIGN, start: true },
];
const COLOR_OVERLAY_ROWS: [Row; 3] = [blend(BlendMode::Normal), color(COLOR_OVERLAY), opacity(100.0)];
const STROKE_ROWS: [Row; 8] = [
    number("Size:", SIZE, 1.0, 250.0, 3.0),
    Row::List { name: "Position:", slot: SOURCE, items: &POSITIONS, start: 0 },
    blend(BlendMode::Normal),
    opacity(100.0),
    Row::List { name: "Fill Type:", slot: STYLE, items: &FILL_TYPES, start: 0 },
    color(STROKE),
    Row::Gradient { name: "Gradient:", gradient: STROKE_GRADIENT },
    Row::List { name: "Pattern:", slot: SPREAD, items: &PATTERNS, start: 0 },
];
const GRADIENT_ROWS: [Row; 8] = [
    blend(BlendMode::Normal),
    opacity(100.0),
    Row::Gradient { name: "Gradient:", gradient: OVERLAY_GRADIENT },
    Row::Toggle { name: "Reverse", slot: FLAG, start: false },
    Row::List { name: "Style:", slot: STYLE, items: &GRADIENT_STYLES, start: 0 },
    Row::Toggle { name: "Align with Layer", slot: ALIGN, start: true },
    angle(90.0),
    number("Scale:", RANGE, 10.0, 150.0, 100.0),
];
const PATTERN_ROWS: [Row; 4] = [
    blend(BlendMode::Normal),
    opacity(100.0),
    Row::List { name: "Pattern:", slot: STYLE, items: &PATTERNS, start: 0 },
    number("Scale:", RANGE, 1.0, 1000.0, 100.0),
];

/// What an effect is set with, in the order of its side of the dialog.
pub fn rows(effect: usize) -> &'static [Row] {
    match effect {
        DROP_SHADOW => &SHADOW_ROWS,
        INNER_SHADOW => &INNER_SHADOW_ROWS,
        OUTER_GLOW => &OUTER_GLOW_ROWS,
        INNER_GLOW => &INNER_GLOW_ROWS,
        BEVEL => &BEVEL_ROWS,
        SATIN => &SATIN_ROWS,
        COLOR_OVERLAY => &COLOR_OVERLAY_ROWS,
        STROKE => &STROKE_ROWS,
        GRADIENT_OVERLAY => &GRADIENT_ROWS,
        _ => &PATTERN_ROWS,
    }
}

/// A name of this dialog in the language in use. Some of its words mean something
/// else in the rest of the interface, and those are listed apart.
pub fn label(name: &'static str) -> &'static str {
    let own = tc("photoshop", name);
    if own == name { t(name) } else { own }
}

fn blend_index(mode: BlendMode) -> usize {
    BlendMode::ALL.iter().position(|m| *m == mode).unwrap_or(0)
}

static SETTINGS: LazyLock<Vec<Setting>> = LazyLock::new(|| {
    let mut all = vec![Setting { name: "", min: 0.0, max: 0.0, start: 0.0, toggle: false }; COUNT];
    for effect in 0..NAMES.len() {
        // Only the drop shadow is on to begin with.
        all[index(effect, ON)] = Setting { name: "On", min: 0.0, max: 1.0, start: if effect == DROP_SHADOW { 1.0 } else { 0.0 }, toggle: true };
        for row in rows(effect) {
            let (slot, setting) = match *row {
                Row::Number { name, slot, min, max, start } => (slot, Setting { name, min, max, start, toggle: false }),
                Row::Toggle { name, slot, start } => (slot, Setting { name, min: 0.0, max: 1.0, start: if start { 1.0 } else { 0.0 }, toggle: true }),
                Row::List { name, slot, items, start } => (slot, Setting { name, min: 0.0, max: (items.len() - 1) as f32, start: start as f32, toggle: false }),
                Row::Blend { name, slot, start } => (slot, Setting { name, min: 0.0, max: (BlendMode::ALL.len() - 1) as f32, start: blend_index(start) as f32, toggle: false }),
                Row::Heading(_) | Row::Color { .. } | Row::Gradient { .. } => continue,
            };
            debug_assert!(slot < SLOTS);
            all[index(effect, slot)] = setting;
        }
    }
    all
});

/// The numbers of the filter, as [`crate::effects::FilterKind::settings`] gives them.
pub fn settings() -> &'static [Setting] {
    &SETTINGS
}

/// The names to choose from for the setting stored at `setting`, if it is a list.
pub fn choices(setting: usize) -> Option<Vec<&'static str>> {
    (0..NAMES.len()).flat_map(|effect| rows(effect).iter().map(move |row| (effect, row))).find_map(|(effect, row)| match *row {
        Row::List { slot, items, .. } if index(effect, slot) == setting => Some(items.iter().map(|item| label(item)).collect()),
        Row::Blend { slot, .. } if index(effect, slot) == setting => Some(BlendMode::ALL.iter().map(|mode| mode.label()).collect()),
        _ => None,
    })
}

/// Whether a line of an effect's settings is in use, given the others: a stroke
/// shows only what its fill type needs.
pub fn row_shows(filter: &LiveFilter, effect: usize, row: &Row) -> bool {
    let choice = |slot: usize| filter.value(index(effect, slot)).round() as usize;
    match effect {
        STROKE => match *row {
            Row::Color { .. } => choice(STYLE) == 0,
            Row::Gradient { .. } => choice(STYLE) == 1,
            Row::List { slot: SPREAD, .. } => choice(STYLE) == 2,
            _ => true,
        },
        // A glow is one color or a gradient, and only a gradient can jitter.
        OUTER_GLOW | INNER_GLOW => match *row {
            Row::Color { .. } => choice(FLAG) == 0,
            Row::Gradient { .. } | Row::Number { slot: ALIGN, .. } => choice(FLAG) == 1,
            _ => true,
        },
        _ => true,
    }
}

/// How far the effects that are on reach beyond the object.
pub fn reach(filter: &LiveFilter) -> f32 {
    let v = |effect: usize, slot: usize| filter.value(index(effect, slot));
    let on = |effect: usize| v(effect, ON) >= 0.5;
    let mut reach: f32 = 0.0;
    if on(DROP_SHADOW) {
        reach = reach.max(v(DROP_SHADOW, DISTANCE) + v(DROP_SHADOW, SIZE) * 2.5);
    }
    if on(OUTER_GLOW) {
        reach = reach.max(v(OUTER_GLOW, SIZE) * 3.0);
    }
    if on(STROKE) && v(STROKE, SOURCE).round() as usize != 1 {
        reach = reach.max(v(STROKE, SIZE) * 3.0);
    }
    if on(BEVEL) && v(BEVEL, STYLE).round() as usize != 1 {
        reach = reach.max(v(BEVEL, SIZE) * 3.0 + v(STROKE, SIZE) * 3.0);
    }
    reach + 2.0
}

fn rgb(color: Color) -> [f32; 3] {
    [color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0]
}

/// A channel moved by whole pixels. What comes in from beyond the layer is `beyond`.
fn shifted(source: &[f32], w: usize, h: usize, dx: f32, dy: f32, beyond: f32) -> Vec<f32> {
    let (ix, iy) = (dx.round() as i64, dy.round() as i64);
    (0..w * h)
        .map(|at| {
            let (x, y) = ((at % w) as i64 - ix, (at / w) as i64 - iy);
            if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 { beyond } else { source[y as usize * w + x as usize] }
        })
        .collect()
}

/// How far each pixel is from the shape in a channel: 0 on it, and growing with the distance from its edge.
pub(crate) fn distances(source: &[f32], w: usize, h: usize) -> Vec<f32> {
    const FAR: f32 = 1e12;
    // The squared distance to the nearest pixel of the shape, first along each column, then along each row.
    fn pass(line: &[f32], out: &mut [f32], hull: &mut Vec<usize>, starts: &mut Vec<f32>) {
        let n = line.len();
        hull.clear();
        starts.clear();
        let meet = |a: usize, b: usize| ((line[b] + (b * b) as f32) - (line[a] + (a * a) as f32)) / (2.0 * (b as f32 - a as f32));
        for q in 0..n {
            if line[q] >= FAR {
                continue;
            }
            while let Some(&last) = hull.last() {
                if hull.len() > 1 && meet(last, q) <= starts[hull.len() - 1] {
                    hull.pop();
                    starts.pop();
                } else {
                    starts.push(meet(last, q));
                    break;
                }
            }
            if hull.is_empty() {
                starts.push(f32::NEG_INFINITY);
            }
            hull.push(q);
        }
        let mut k = 0;
        for (q, slot) in out.iter_mut().enumerate().take(n) {
            if hull.is_empty() {
                *slot = FAR;
                continue;
            }
            while k + 1 < hull.len() && starts[k + 1] <= q as f32 {
                k += 1;
            }
            let p = hull[k];
            *slot = line[p] + ((q as f32) - (p as f32)).powi(2);
        }
    }
    let mut squared: Vec<f32> = source.iter().map(|a| if *a >= 0.5 { 0.0 } else { FAR }).collect();
    let (mut hull, mut starts) = (Vec::new(), Vec::new());
    let (mut line, mut out) = (vec![0.0; w.max(h)], vec![0.0; w.max(h)]);
    for x in 0..w {
        for y in 0..h {
            line[y] = squared[y * w + x];
        }
        pass(&line[..h], &mut out[..h], &mut hull, &mut starts);
        for y in 0..h {
            squared[y * w + x] = out[y];
        }
    }
    for y in 0..h {
        line[..w].copy_from_slice(&squared[y * w..(y + 1) * w]);
        pass(&line[..w], &mut out[..w], &mut hull, &mut starts);
        squared[y * w..(y + 1) * w].copy_from_slice(&out[..w]);
    }
    squared.iter().map(|d| d.min(FAR).sqrt()).collect()
}

/// A shape grown outward by `by` pixels all around, keeping a hard edge.
fn grown(source: &[f32], w: usize, h: usize, by: f32) -> Vec<f32> {
    if by < 0.25 {
        return source.to_vec();
    }
    distances(source, w, h).iter().zip(source).map(|(d, a)| (by + 1.0 - d).clamp(0.0, 1.0).max(*a)).collect()
}

/// A shape made `size` bigger: the `spread` part of that with a hard edge, the rest fading out.
fn soft(source: &[f32], w: usize, h: usize, size: f32, spread: f32) -> Vec<f32> {
    let solid = size * spread.clamp(0.0, 1.0);
    let hard = grown(source, w, h, solid);
    if size - solid < 0.25 { hard } else { blur_channel(&hard, w, h, size - solid) }
}

/// Breaks a soft shape up into grains: with all the noise, each pixel is either there or not.
fn speckle(mask: &mut [f32], w: usize, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    for (at, m) in mask.iter_mut().enumerate() {
        let chance = (noise(at % w, at / w, 7) + 1.0) / 2.0;
        *m = *m * (1.0 - amount) + amount * if chance < *m { 1.0 } else { 0.0 };
    }
}

/// Colors the object itself through `mask`, mixing by a blend mode. What is clear stays clear.
fn tint(out: &mut Layer, mask: &dyn Fn(usize) -> f32, color: &dyn Fn(usize) -> [f32; 3], mode: BlendMode) {
    for (at, px) in out.pixels.iter_mut().enumerate().filter(|(_, px)| px[3] > 0.0) {
        let amount = mask(at).clamp(0.0, 1.0);
        if amount > 0.0 {
            let mixed = mode.mix([px[0], px[1], px[2]], color(at));
            for c in 0..3 {
                px[c] += (mixed[c] - px[c]) * amount;
            }
        }
    }
}

/// Lays color over the object and what is around it, through `mask`. On the object it mixes by the blend mode.
fn lay(out: &mut Layer, mask: &dyn Fn(usize) -> f32, color: &dyn Fn(usize) -> [f32; 3], mode: BlendMode) {
    for (at, px) in out.pixels.iter_mut().enumerate() {
        let amount = mask(at).clamp(0.0, 1.0);
        if amount > 0.0 {
            let plain = color(at);
            let mixed = mode.mix([px[0], px[1], px[2]], plain);
            // Where there is nothing under it, there is nothing to mix with.
            let top = [0, 1, 2].map(|c| plain[c] + (mixed[c] - plain[c]) * px[3]);
            *px = over([top[0], top[1], top[2], amount], *px);
        }
    }
}

/// The box around everything in a channel that is not clear: left, top, right, bottom.
fn extent(alpha: &[f32], w: usize, h: usize) -> (f32, f32, f32, f32) {
    let (mut left, mut top, mut right, mut bottom) = (w, h, 0, 0);
    for (at, _) in alpha.iter().enumerate().filter(|(_, a)| **a > 0.0) {
        let (x, y) = (at % w, at / w);
        (left, top, right, bottom) = (left.min(x), top.min(y), right.max(x + 1), bottom.max(y + 1));
    }
    if right <= left || bottom <= top { (0.0, 0.0, w as f32, h as f32) } else { (left as f32, top as f32, right as f32, bottom as f32) }
}

/// How far along a gradient a pixel is, from 0 at its start to 1 at its end.
fn gradient_place(style: usize, x: f32, y: f32, (left, top, right, bottom): (f32, f32, f32, f32), degrees: f32, scale: f32) -> f32 {
    let (cx, cy) = ((left + right) / 2.0, (top + bottom) / 2.0);
    let (dx, dy) = direction(degrees);
    // Along the angle and across it, measured from the middle.
    let (along, across) = ((x + 0.5 - cx) * dx + (y + 0.5 - cy) * dy, -(x + 0.5 - cx) * dy + (y + 0.5 - cy) * dx);
    // Half the length of the box along the angle, which is where a linear gradient ends.
    let half = (((right - left) * dx.abs() + (bottom - top) * dy.abs()) / 2.0).max(0.5) * scale.max(0.01);
    match style {
        1 => along.hypot(across) / half,
        2 => {
            let turn = (-across).atan2(along) / std::f32::consts::TAU;
            if turn < 0.0 { turn + 1.0 } else { turn }
        }
        3 => along.abs() / half,
        4 => (along.abs() + across.abs()) / half,
        _ => 0.5 + along / half / 2.0,
    }
    .clamp(0.0, 1.0)
}

/// The color of a pattern at a pixel. Patterns are two grays, to be mixed in by a blend mode.
pub fn pattern(which: usize, x: f32, y: f32) -> [f32; 3] {
    let (ix, iy) = (x.floor() as i64, y.floor() as i64);
    let dark = match which {
        1 => iy.rem_euclid(8) < 4,
        2 => ix.rem_euclid(8) < 4,
        3 => (ix + iy).rem_euclid(8) < 4,
        4 => {
            let (cx, cy) = (x.rem_euclid(8.0) - 4.0, y.rem_euclid(8.0) - 4.0);
            cx * cx + cy * cy < 5.0
        }
        5 => ix.rem_euclid(8) == 0 || iy.rem_euclid(8) == 0,
        6 => {
            // Every other course of bricks is set half a brick along.
            let course = iy.div_euclid(8);
            iy.rem_euclid(8) == 0 || (ix + if course.rem_euclid(2) == 0 { 0 } else { 8 }).rem_euclid(16) == 0
        }
        _ => (ix.div_euclid(8) + iy.div_euclid(8)).rem_euclid(2) == 0,
    };
    if dark { [0.35; 3] } else { [0.92; 3] }
}

/// Applies the effects that are on to a layer, all in one piece: what goes behind
/// the object is simply put under it.
pub fn apply(layer: &Layer, filter: &LiveFilter) -> Layer {
    let (mut out, under) = apply_apart(layer, filter);
    // The last of them is the nearest to the object.
    for (part, _) in under.iter().rev() {
        for (px, below) in out.pixels.iter_mut().zip(&part.pixels) {
            if below[3] > 0.0 {
                *px = over(*px, *below);
            }
        }
    }
    out
}

/// Applies the effects that are on, keeping apart what goes behind the object: the
/// drop shadow and the outer glow, from the lowest up, each with the blend mode it
/// is to be mixed into the page with. The object and what lies on it come first.
pub fn apply_apart(layer: &Layer, filter: &LiveFilter) -> (Layer, Vec<(Layer, BlendMode)>) {
    let (w, h) = (layer.width, layer.height);
    let v = |effect: usize, slot: usize| filter.value(index(effect, slot));
    let on = |effect: usize| v(effect, ON) >= 0.5;
    let choice = |effect: usize, slot: usize| v(effect, slot).round() as usize;
    let mode = |effect: usize, slot: usize| BlendMode::ALL[choice(effect, slot).min(BlendMode::ALL.len() - 1)];
    let shade = |color: usize| rgb(filter.ps_color(color));
    let alpha = layer.alpha();
    let outside: Vec<f32> = alpha.iter().map(|a| 1.0 - a).collect();
    let bounds = extent(&alpha, w, h);
    let mut out = layer.clone();

    // The stroke's band, which the stroke emboss needs as well.
    let band: Vec<f32> = {
        let size = v(STROKE, SIZE);
        let outer = |by: f32| -> Vec<f32> { grown(&alpha, w, h, by).iter().zip(&alpha).map(|(g, a)| (g - a).max(0.0)).collect() };
        let inner = |by: f32| -> Vec<f32> { grown(&outside, w, h, by).iter().zip(&alpha).map(|(g, a)| g.min(*a)).collect() };
        match choice(STROKE, SOURCE) {
            1 => inner(size),
            2 => outer(size / 2.0).iter().zip(inner(size / 2.0)).map(|(o, i)| (o + i).min(1.0)).collect(),
            _ => outer(size),
        }
    };

    // What lies on the object, from the lowest up.
    if on(PATTERN_OVERLAY) {
        let (which, scale, strength) = (choice(PATTERN_OVERLAY, STYLE), v(PATTERN_OVERLAY, RANGE) / 100.0, v(PATTERN_OVERLAY, OPACITY) / 100.0);
        tint(&mut out, &|_| strength, &|at| pattern(which, (at % w) as f32 / scale, (at / w) as f32 / scale), mode(PATTERN_OVERLAY, BLEND));
    }
    if on(GRADIENT_OVERLAY) {
        let stops = filter.gradient(OVERLAY_GRADIENT);
        let (style, degrees, scale, strength) = (choice(GRADIENT_OVERLAY, STYLE), v(GRADIENT_OVERLAY, ANGLE), v(GRADIENT_OVERLAY, RANGE) / 100.0, v(GRADIENT_OVERLAY, OPACITY) / 100.0);
        let reverse = v(GRADIENT_OVERLAY, FLAG) >= 0.5;
        // Aligned with the layer, the gradient spans the object; otherwise all that is drawn of it.
        let span = if v(GRADIENT_OVERLAY, ALIGN) >= 0.5 { bounds } else { (0.0, 0.0, w as f32, h as f32) };
        let color = |at: usize| {
            let place = gradient_place(style, (at % w) as f32, (at / w) as f32, span, degrees, scale);
            gradient_color(&stops, if reverse { 1.0 - place } else { place })
        };
        // Where the gradient is see-through, the object shows as it was.
        tint(&mut out, &|at| strength * color(at).1, &|at| color(at).0, mode(GRADIENT_OVERLAY, BLEND));
    }
    if on(COLOR_OVERLAY) {
        let (color, strength) = (shade(COLOR_OVERLAY), v(COLOR_OVERLAY, OPACITY) / 100.0);
        tint(&mut out, &|_| strength, &|_| color, mode(COLOR_OVERLAY, BLEND));
    }
    if on(SATIN) {
        let ((dx, dy), distance, strength) = (direction(v(SATIN, ANGLE)), v(SATIN, DISTANCE), v(SATIN, OPACITY) / 100.0);
        let blurred = blur_channel(&alpha, w, h, v(SATIN, SIZE).max(0.5));
        // The outline shifted both ways along the angle: where the two copies differ, the sheen shows.
        let (one, other) = (shifted(&blurred, w, h, dx * distance, dy * distance, 0.0), shifted(&blurred, w, h, -dx * distance, -dy * distance, 0.0));
        let invert = v(SATIN, FLAG) >= 0.5;
        let color = shade(SATIN);
        let smooth = v(SATIN, ALIGN) >= 0.5;
        let sheen = |at: usize| {
            let differ = (one[at] - other[at]).abs();
            // Without anti-aliasing the shading goes in a few flat steps.
            let differ = if smooth { differ } else { (differ * 8.0).round() / 8.0 };
            (if invert { 1.0 - differ } else { differ }) * strength
        };
        tint(&mut out, &sheen, &|_| color, mode(SATIN, BLEND));
    }
    // A glow by one of the two techniques: softer is a blur, precise falls off evenly from the edge.
    let glow = |source: &[f32], effect: usize| -> Vec<f32> {
        let (size, spread, range) = (v(effect, SIZE), v(effect, SPREAD) / 100.0, v(effect, RANGE) / 50.0);
        let mut mask = if choice(effect, STYLE) == 1 {
            // Full for the spread part of the size, then fading evenly to nothing at the whole size.
            let fade = (size * (1.0 - spread)).max(0.5);
            distances(source, w, h).iter().zip(source).map(|(d, a)| (1.0 - (d - size * spread).max(0.0) / fade).clamp(0.0, 1.0).max(*a)).collect()
        } else {
            soft(source, w, h, size, spread)
        };
        // A short range gathers the glow close to the edge.
        for m in &mut mask {
            *m = (*m / range.max(0.02)).min(1.0).powf(if range < 1.0 { 1.0 / range.max(0.02) } else { 1.0 });
        }
        speckle(&mut mask, w, v(effect, NOISE) / 100.0);
        mask
    };
    // The color of a glow where it is `amount` strong, and how opaque: one color, or a
    // gradient that starts at the edge of the object and may be jittered along its length.
    let glow_color = |effect: usize, gradient: usize| {
        let (stops, jitter, plain) = (filter.gradient(gradient), v(effect, ALIGN) / 100.0, shade(effect));
        let graded = choice(effect, FLAG) == 1;
        move |at: usize, amount: f32| -> ([f32; 3], f32) {
            if !graded {
                return (plain, amount);
            }
            let place = (1.0 - amount + noise(at % w, at / w, 11) * jitter / 2.0).clamp(0.0, 1.0);
            let (color, opacity) = gradient_color(&stops, place);
            (color, if amount > 0.0 { opacity } else { 0.0 })
        }
    };
    if on(INNER_GLOW) {
        let (edge, strength, from_center) = (glow(&outside, INNER_GLOW), v(INNER_GLOW, OPACITY) / 100.0, choice(INNER_GLOW, SOURCE) == 0);
        let color = glow_color(INNER_GLOW, INNER_GLOW_GRADIENT);
        let amount = |at: usize| if from_center { 1.0 - edge[at] } else { edge[at] };
        tint(&mut out, &|at| color(at, amount(at)).1 * strength, &|at| color(at, amount(at)).0, mode(INNER_GLOW, BLEND));
    }
    // Photoshop's angles point at the light, and a shadow falls the other way.
    let fall = |effect: usize| {
        let (dx, dy) = direction(v(effect, ANGLE) + 180.0);
        (dx * v(effect, DISTANCE), dy * v(effect, DISTANCE))
    };
    if on(INNER_SHADOW) {
        let (dx, dy) = fall(INNER_SHADOW);
        // The shadow falls on the object from what is around it.
        let mut mask = soft(&shifted(&outside, w, h, dx, dy, 1.0), w, h, v(INNER_SHADOW, SIZE), v(INNER_SHADOW, SPREAD) / 100.0);
        speckle(&mut mask, w, v(INNER_SHADOW, NOISE) / 100.0);
        let (strength, color) = (v(INNER_SHADOW, OPACITY) / 100.0, shade(INNER_SHADOW));
        tint(&mut out, &|at| mask[at] * strength, &|_| color, mode(INNER_SHADOW, BLEND));
    }
    if on(STROKE) {
        let (strength, fill) = (v(STROKE, OPACITY) / 100.0, choice(STROKE, STYLE));
        let (from, stops, which) = (shade(STROKE), filter.gradient(STROKE_GRADIENT), choice(STROKE, SPREAD));
        // A gradient stroke runs from its first color at the bottom to its second at the top, over the stroke's own box.
        let span = extent(&band, w, h);
        let color = |at: usize| match fill {
            1 => {
                gradient_color(&stops, gradient_place(0, (at % w) as f32, (at / w) as f32, span, 90.0, 1.0)).0
            }
            2 => pattern(which, (at % w) as f32, (at / w) as f32),
            _ => from,
        };
        lay(&mut out, &|at| band[at] * strength, &color, mode(STROKE, BLEND));
    }
    if on(BEVEL) {
        bevel(&mut out, &alpha, &band, on(STROKE), filter);
    }

    // What lies behind the object, each on a layer of its own.
    let mut under = Vec::new();
    if on(DROP_SHADOW) {
        let (dx, dy) = fall(DROP_SHADOW);
        let mut mask = soft(&shifted(&alpha, w, h, dx, dy, 0.0), w, h, v(DROP_SHADOW, SIZE), v(DROP_SHADOW, SPREAD) / 100.0);
        speckle(&mut mask, w, v(DROP_SHADOW, NOISE) / 100.0);
        let (strength, knocks_out, color) = (v(DROP_SHADOW, OPACITY) / 100.0, v(DROP_SHADOW, FLAG) >= 0.5, shade(DROP_SHADOW));
        // Knocked out, the shadow is cut away under the object even where the object is see-through.
        let pixels = mask.iter().zip(&alpha).map(|(m, a)| [color[0], color[1], color[2], (m * strength * if knocks_out { 1.0 - a } else { 1.0 }).clamp(0.0, 1.0)]).collect();
        under.push((Layer { width: w, height: h, pixels }, mode(DROP_SHADOW, BLEND)));
    }
    if on(OUTER_GLOW) {
        let (mask, strength) = (glow(&alpha, OUTER_GLOW), v(OUTER_GLOW, OPACITY) / 100.0);
        let color = glow_color(OUTER_GLOW, OUTER_GLOW_GRADIENT);
        let pixels = (0..w * h)
            .map(|at| {
                let (tone, opacity) = color(at, mask[at]);
                // The glow belongs around the object, not under it.
                [tone[0], tone[1], tone[2], (opacity * strength * (1.0 - alpha[at])).clamp(0.0, 1.0)]
            })
            .collect();
        under.push((Layer { width: w, height: h, pixels }, mode(OUTER_GLOW, BLEND)));
    }
    (out, under)
}

/// Bevel and Emboss: the edge of the object as a slope, lit from an angle.
fn bevel(out: &mut Layer, alpha: &[f32], band: &[f32], stroked: bool, filter: &LiveFilter) {
    let (w, h) = (out.width, out.height);
    let v = |slot: usize| filter.value(index(BEVEL, slot));
    let (style, technique, size) = (v(STYLE).round() as usize, v(SOURCE).round() as usize, v(SIZE).max(0.5));
    // A stroke emboss works on the stroke alone, and there is none without a stroke.
    if style == 4 && !stroked {
        return;
    }
    // The shape whose edge is sloped: the object, or the object with its stroke.
    let solid: Vec<f32> = if style == 4 { alpha.iter().zip(band).map(|(a, b)| a.max(*b)).collect() } else { alpha.to_vec() };
    let ramp = blur_channel(&solid, w, h, size);
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let mut heights: Vec<f32> = ramp
        .iter()
        .map(|r| {
            let t = match style {
                0 => r * 2.0,
                1 => (r - 0.5) * 2.0,
                // A pillow emboss sinks the edge itself, rising to both sides of it.
                3 => (r - 0.5).abs() * 2.0,
                _ => *r,
            }
            .clamp(0.0, 1.0);
            // A chisel keeps the slope flat; smooth rounds it off.
            if technique == 0 { smooth(t) } else { t }
        })
        .collect();
    if technique == 2 {
        heights = blur_channel(&heights, w, h, 1.0);
    }
    // The light, tilted up by the altitude: straight overhead, nothing is lit from the side.
    let (lx, ly) = direction(v(ANGLE));
    let side = v(SPREAD).to_radians().cos() / 30f32.to_radians().cos();
    let down = if v(ALIGN).round() as usize == 1 { -1.0 } else { 1.0 };
    let gain = v(DISTANCE) / 100.0 * size * side * down;
    let at = |x: i64, y: i64| heights[y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize];
    let mut light = vec![0.0; w * h];
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let (dx, dy) = (at(x + 1, y) - at(x - 1, y), at(x, y + 1) - at(x, y - 1));
            // The slope rises away from the edge, so the side whose slope points at the light is lit.
            light[y as usize * w + x as usize] = -(dx * lx + dy * ly) * gain;
        }
    }
    if v(RANGE) > 0.0 {
        light = blur_channel(&light, w, h, v(RANGE));
    }
    let mode = |slot: usize| BlendMode::ALL[(v(slot).round() as usize).min(BlendMode::ALL.len() - 1)];
    let (bright, dark) = (rgb(filter.ps_color(BEVEL)), rgb(filter.ps_color(BEVEL_SHADE)));
    let (bright_strength, dark_strength) = (v(OPACITY) / 100.0, v(FLAG) / 100.0);
    // Where the bevel shows: on the object, around it, both, or on the stroke.
    let shows = |at: usize| match style {
        0 => 1.0 - alpha[at],
        1 => alpha[at],
        4 => band[at],
        _ => 1.0,
    };
    for (strength, color, mode, sign) in [(bright_strength, bright, mode(BLEND), 1.0), (dark_strength, dark, mode(NOISE), -1.0)] {
        let mask = |at: usize| (light[at] * sign).clamp(0.0, 1.0) * strength * shows(at);
        if style == 1 {
            tint(out, &mask, &|_| color, mode);
        } else {
            lay(out, &mask, &|_| color, mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::FilterKind;

    const SIDE: usize = 80;

    /// A blue square of 40 pixels in the middle of a clear layer.
    fn square() -> Layer {
        let mut pixels = vec![[0.0; 4]; SIDE * SIDE];
        for y in 20..60 {
            for x in 20..60 {
                pixels[y * SIDE + x] = [0.0, 0.0, 1.0, 1.0];
            }
        }
        Layer { width: SIDE, height: SIDE, pixels }
    }

    /// The filter with only the given effect on.
    fn only(effect: usize) -> LiveFilter {
        let mut filter = LiveFilter::new(FilterKind::PhotoshopEffects);
        filter.set_value(index(DROP_SHADOW, ON), 0.0);
        filter.set_value(index(effect, ON), 1.0);
        filter
    }

    fn at(layer: &Layer, x: usize, y: usize) -> [f32; 4] {
        layer.pixels[y * SIDE + x]
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.03
    }

    #[test]
    fn every_setting_has_its_own_place_and_old_places_are_kept() {
        let mut seen = std::collections::HashSet::new();
        for effect in 0..NAMES.len() {
            for slot in 0..SLOTS {
                assert!(index(effect, slot) < COUNT && seen.insert(index(effect, slot)), "{effect} {slot}");
            }
        }
        assert_eq!(seen.len(), COUNT);
        // The first eight effects keep their first five numbers in fives, as files already have them.
        assert_eq!((index(STROKE, SIZE), index(SATIN, ON), index(DROP_SHADOW, OPACITY)), (39, 25, 1));
        assert_eq!(settings().len(), COUNT);
        assert_eq!(start_colors().len(), STROKE_END + 1);
        // Every effect is in the dialog once.
        let mut listed = ORDER;
        listed.sort_unstable();
        assert_eq!(listed, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn every_name_in_the_dialog_has_a_turkish_entry() {
        let table: std::collections::HashMap<&str, &str> = crate::i18n::tr::ENTRIES.iter().copied().collect();
        let known = |name: &str| table.contains_key(name) || table.contains_key(format!("{name}|photoshop").as_str());
        let presets: Vec<&str> = PRESETS.iter().map(|(name, _)| *name).collect();
        let lists: [&[&str]; 10] = [&presets, &TECHNIQUES, &SOURCES, &BEVEL_STYLES, &BEVEL_TECHNIQUES, &DIRECTIONS, &POSITIONS, &FILL_TYPES, &GRADIENT_STYLES, &PATTERNS];
        for name in NAMES.iter().chain(lists.iter().flat_map(|list| list.iter())) {
            assert!(known(name), "{name}");
        }
        for effect in 0..NAMES.len() {
            for row in rows(effect) {
                let (Row::Heading(name)
                | Row::Number { name, .. }
                | Row::Toggle { name, .. }
                | Row::List { name, .. }
                | Row::Blend { name, .. }
                | Row::Color { name, .. }
                | Row::Gradient { name, .. }) = *row;
                assert!(known(name), "{name}");
            }
        }
    }

    #[test]
    fn lists_and_blend_modes_offer_their_names() {
        assert_eq!(choices(index(STROKE, SOURCE)).map(|names| names.len()), Some(POSITIONS.len()));
        assert_eq!(choices(index(BEVEL, NOISE)).map(|names| names.len()), Some(BlendMode::ALL.len()));
        assert_eq!(choices(index(DROP_SHADOW, SIZE)), None);
        // New shadows multiply and new glows screen, as in Photoshop.
        let filter = LiveFilter::new(FilterKind::PhotoshopEffects);
        assert_eq!(BlendMode::ALL[filter.value(index(DROP_SHADOW, BLEND)) as usize], BlendMode::Multiply);
        assert_eq!(BlendMode::ALL[filter.value(index(OUTER_GLOW, BLEND)) as usize], BlendMode::Screen);
        // A stroke shows only what its fill type needs.
        let shown = |filter: &LiveFilter| rows(STROKE).iter().filter(|row| row_shows(filter, STROKE, row)).count();
        let mut stroke = only(STROKE);
        assert_eq!(shown(&stroke), 6);
        stroke.set_value(index(STROKE, STYLE), 1.0);
        assert_eq!(shown(&stroke), 6);
        stroke.set_value(index(STROKE, STYLE), 2.0);
        assert_eq!(shown(&stroke), 6);
    }

    #[test]
    fn a_drop_shadow_falls_away_from_the_light_and_spreads() {
        let layer = square();
        // Light from 120 degrees, the upper left, throws the shadow down and to the right.
        let shadowed = apply(&layer, &LiveFilter::new(FilterKind::PhotoshopEffects));
        assert!(at(&shadowed, 61, 62)[3] > 0.2 && at(&shadowed, 61, 62)[2] < 0.1, "{:?}", at(&shadowed, 61, 62));
        assert!(at(&shadowed, 4, 4)[3] < 0.01);
        assert_eq!(at(&shadowed, 40, 40), [0.0, 0.0, 1.0, 1.0]);
        // With all of its size as spread, the edge of the shadow is hard: full, then soon nothing.
        let mut hard = LiveFilter::new(FilterKind::PhotoshopEffects);
        hard.set_value(index(DROP_SHADOW, SPREAD), 100.0);
        hard.set_value(index(DROP_SHADOW, OPACITY), 100.0);
        let spread = apply(&layer, &hard);
        assert!(at(&spread, 63, 66)[3] > 0.9, "{:?}", at(&spread, 63, 66));
        assert!(at(&spread, 63, 78)[3] < 0.05);
        // Noise breaks it into grains: next to full pixels there are empty ones.
        hard.set_value(index(DROP_SHADOW, SPREAD), 0.0);
        hard.set_value(index(DROP_SHADOW, SIZE), 10.0);
        hard.set_value(index(DROP_SHADOW, NOISE), 100.0);
        let grainy = apply(&layer, &hard);
        let row: Vec<f32> = (20..60).map(|x| at(&grainy, x, 66)[3]).collect();
        assert!(row.iter().all(|a| *a < 0.01 || *a > 0.99) && row.iter().any(|a| *a > 0.99) && row.iter().any(|a| *a < 0.01), "{row:?}");
    }

    #[test]
    fn inner_effects_stay_on_the_object() {
        let layer = square();
        for effect in [INNER_SHADOW, INNER_GLOW, SATIN, COLOR_OVERLAY, GRADIENT_OVERLAY, PATTERN_OVERLAY] {
            let done = apply(&layer, &only(effect));
            for (after, before) in done.pixels.iter().zip(&layer.pixels) {
                assert_eq!(after[3], before[3], "{}", NAMES[effect]);
            }
            assert_ne!(done.pixels, layer.pixels, "{}", NAMES[effect]);
        }
        // An inner shadow darkens the side toward the light; the far side keeps its color.
        let shadow = apply(&layer, &only(INNER_SHADOW));
        assert!(at(&shadow, 21, 40)[2] < 0.7 && near(at(&shadow, 45, 45)[2], 1.0), "{:?}", at(&shadow, 21, 40));
        // An inner glow from the edge lights the rim; from the center, the middle.
        let mut glow = only(INNER_GLOW);
        glow.set_value(index(INNER_GLOW, BLEND), 0.0);
        let rim = apply(&layer, &glow);
        assert!(at(&rim, 20, 40)[0] > 0.3 && at(&rim, 40, 40)[0] < 0.02);
        glow.set_value(index(INNER_GLOW, SOURCE), 0.0);
        let middle = apply(&layer, &glow);
        assert!(at(&middle, 40, 40)[0] > 0.7 && at(&middle, 20, 40)[0] < at(&middle, 40, 40)[0]);
    }

    #[test]
    fn overlays_mix_by_their_blend_mode() {
        let layer = square();
        // Red laid over blue as it is, then multiplied into it, which leaves black.
        let mut overlay = only(COLOR_OVERLAY);
        assert_eq!(at(&apply(&layer, &overlay), 40, 40), [1.0, 0.0, 0.0, 1.0]);
        overlay.set_value(index(COLOR_OVERLAY, BLEND), blend_index(BlendMode::Multiply) as f32);
        assert_eq!(at(&apply(&layer, &overlay), 40, 40), [0.0, 0.0, 0.0, 1.0]);
        // At half opacity it is half way there.
        overlay.set_value(index(COLOR_OVERLAY, OPACITY), 50.0);
        assert!(near(at(&apply(&layer, &overlay), 40, 40)[2], 0.5));
    }

    #[test]
    fn a_gradient_runs_along_its_angle_over_the_object() {
        let layer = square();
        // At 90 degrees it runs up: black at the bottom of the object, white at the top.
        let mut gradient = only(GRADIENT_OVERLAY);
        let up = apply(&layer, &gradient);
        assert!(at(&up, 40, 59)[0] < 0.05 && at(&up, 40, 20)[0] > 0.95 && near(at(&up, 25, 40)[0], 0.5), "{:?}", at(&up, 40, 59));
        // Reversed, it is the other way up.
        gradient.set_value(index(GRADIENT_OVERLAY, FLAG), 1.0);
        assert!(at(&apply(&layer, &gradient), 40, 59)[0] > 0.95);
        gradient.set_value(index(GRADIENT_OVERLAY, FLAG), 0.0);
        // At 0 degrees it runs to the right.
        gradient.set_value(index(GRADIENT_OVERLAY, ANGLE), 0.0);
        let right = apply(&layer, &gradient);
        assert!(at(&right, 20, 40)[0] < 0.05 && at(&right, 59, 40)[0] > 0.95);
        // Radial starts in the middle; reflected is the same on both sides of it.
        gradient.set_value(index(GRADIENT_OVERLAY, STYLE), 1.0);
        let radial = apply(&layer, &gradient);
        assert!(at(&radial, 40, 40)[0] < 0.1 && at(&radial, 59, 40)[0] > 0.9 && at(&radial, 40, 21)[0] > 0.9);
        gradient.set_value(index(GRADIENT_OVERLAY, STYLE), 3.0);
        let reflected = apply(&layer, &gradient);
        assert!(near(at(&reflected, 25, 30)[0], at(&reflected, 54, 30)[0]) && at(&reflected, 40, 30)[0] < 0.1);
        // At half scale it gets from one end to the other in half the distance.
        gradient.set_value(index(GRADIENT_OVERLAY, STYLE), 0.0);
        gradient.set_value(index(GRADIENT_OVERLAY, RANGE), 50.0);
        let short = apply(&layer, &gradient);
        assert!(at(&short, 28, 40)[0] < 0.05 && at(&short, 51, 40)[0] > 0.95);
        // With its own colors.
        gradient.set_ps_color(GRADIENT_END, Color::rgb(0, 255, 0));
        assert!(at(&apply(&layer, &gradient), 55, 40)[1] > 0.95);
        // With three stops it passes through the middle one, and where a stop is clear the object shows.
        gradient.set_value(index(GRADIENT_OVERLAY, RANGE), 100.0);
        gradient.set_gradient(OVERLAY_GRADIENT, vec![(0.0, Color::rgb(255, 0, 0)), (0.5, Color::rgb(0, 255, 0)), (1.0, Color { r: 0, g: 0, b: 0, a: 0 })]);
        let three = apply(&layer, &gradient);
        assert!(at(&three, 20, 40)[0] > 0.95 && at(&three, 39, 40)[1] > 0.9, "{:?}", at(&three, 39, 40));
        assert!(at(&three, 59, 40)[2] > 0.9 && at(&three, 59, 40)[3] == 1.0, "{:?}", at(&three, 59, 40));
    }

    #[test]
    fn gradient_stops_are_added_moved_and_pulled_away() {
        let two = preset(0);
        assert_eq!(gradient_color(&two, 0.5).0, [0.5, 0.5, 0.5]);
        assert_eq!(gradient_color(&two, -1.0).0, [0.0; 3]);
        assert_eq!(gradient_color(&preset(2), 0.5).1, 0.5);
        // A press between the stops adds one in the color found there; a press on a stop takes hold of it.
        let (three, held) = stop_pressed(two.clone(), 0.5, 0.03);
        assert_eq!((three.len(), held, three[1]), (3, 1, (0.5, Color { r: 128, g: 128, b: 128, a: 255 })));
        assert_eq!(stop_pressed(three.clone(), 0.98, 0.03), (three.clone(), 2));
        // A dragged stop can pass another, and is still the one held.
        let (moved, held) = stop_dragged(three.clone(), 1, 1.4, false);
        assert_eq!((held, moved[1].1.r, moved[1].0), (Some(1), 128, 1.0), "at the same place they keep their order");
        let (moved, held) = stop_dragged(three.clone(), 2, 0.2, false);
        assert_eq!((held, moved[1].1.r, moved[2].0), (Some(1), 255, 0.5));
        let (moved, held) = stop_dragged(three.clone(), 0, 0.704, false);
        assert_eq!((held, moved[1], moved[0].0), (Some(1), (0.7, Color::BLACK), 0.5));
        // Pulled away it is gone, but the last two stay.
        let (fewer, held) = stop_dragged(three, 1, 0.5, true);
        assert_eq!((fewer.clone(), held), (two.clone(), None));
        assert_eq!(stop_dragged(two.clone(), 0, 0.2, true).1, Some(0));
        // Every preset runs from 0 to 1 in order.
        for which in 0..PRESETS.len() {
            let stops = preset(which);
            assert!(stops.len() >= 2 && stops[0].0 == 0.0 && stops[stops.len() - 1].0 == 1.0 && stops.windows(2).all(|pair| pair[0].0 <= pair[1].0));
        }
    }

    #[test]
    fn opacity_stops_are_a_track_of_their_own() {
        let colors = preset(0);
        assert_eq!(baked(&colors, &[]), colors);
        // Opaque at the left, clear at the right, whatever the colors do.
        let clear = |alpha: u8| Color { r: 0, g: 0, b: 0, a: alpha };
        let fading = baked(&colors, &[(0.0, clear(255)), (1.0, clear(0))]);
        assert_eq!((gradient_color(&fading, 0.0).1, gradient_color(&fading, 1.0).1), (1.0, 0.0));
        assert!((gradient_color(&fading, 0.5).1 - 0.5).abs() < 0.01 && (gradient_color(&fading, 0.5).0[0] - 0.5).abs() < 0.01);
        // A stop in the middle of the track is a place in the result, with the color the gradient has there.
        let dip = baked(&colors, &[(0.0, clear(255)), (0.25, clear(0)), (1.0, clear(255))]);
        assert_eq!(dip.len(), 3);
        assert_eq!((dip[1].0, dip[1].1.a, dip[1].1.r), (0.25, 0, 64));
        // One opacity stop alone sets the whole gradient to it.
        assert!((gradient_color(&baked(&colors, &[(0.3, clear(128))]), 0.9).1 - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn a_glow_can_be_a_gradient_and_only_then_shows_jitter() {
        let layer = square();
        let mut glow = only(OUTER_GLOW);
        glow.set_value(index(OUTER_GLOW, SIZE), 12.0);
        glow.set_value(index(OUTER_GLOW, STYLE), 1.0);
        glow.set_value(index(OUTER_GLOW, OPACITY), 100.0);
        assert_eq!(gradient_of(&glow, OUTER_GLOW), None);
        glow.set_value(index(OUTER_GLOW, FLAG), 1.0);
        assert_eq!(gradient_of(&glow, OUTER_GLOW), Some(OUTER_GLOW_GRADIENT));
        // Red at the edge of the object, turning blue further out.
        glow.set_gradient(OUTER_GLOW_GRADIENT, vec![(0.0, Color::rgb(255, 0, 0)), (1.0, Color::rgb(0, 0, 255))]);
        let graded = apply(&layer, &glow);
        assert!(at(&graded, 18, 40)[0] > 0.7 && at(&graded, 10, 40)[2] > 0.7, "{:?} {:?}", at(&graded, 18, 40), at(&graded, 10, 40));
        assert!(at(&graded, 2, 40)[3] == 0.0);
        // Jittered, pixels the same distance out differ.
        glow.set_value(index(OUTER_GLOW, ALIGN), 100.0);
        let jittered = apply(&layer, &glow);
        let column: std::collections::HashSet<u32> = (25..55).map(|y| (at(&jittered, 14, y)[0] * 1000.0) as u32).collect();
        assert!(column.len() > 10);
        // The stroke and the overlay have gradients of their own.
        assert_eq!((gradient_of(&only(GRADIENT_OVERLAY), GRADIENT_OVERLAY), gradient_of(&only(STROKE), STROKE)), (Some(OVERLAY_GRADIENT), None));
    }

    #[test]
    fn a_shape_grows_by_the_same_amount_all_around() {
        let alpha = square().alpha();
        let far = distances(&alpha, SIDE, SIDE);
        assert_eq!((far[40 * SIDE + 40], far[40 * SIDE + 15], far[10 * SIDE + 40]), (0.0, 5.0, 10.0));
        // From a corner it is measured straight across, so a grown square has round corners.
        assert!((far[16 * SIDE + 17] - 5.0).abs() < 0.01);
        let bigger = grown(&alpha, SIDE, SIDE, 4.0);
        let on = |x: usize, y: usize| bigger[y * SIDE + x];
        assert_eq!((on(16, 40), on(15, 40), on(63, 40), on(64, 40), on(40, 16), on(40, 64)), (1.0, 0.0, 1.0, 0.0, 1.0, 0.0));
        assert!(on(17, 17) > 0.5 && on(16, 16) == 0.0);
        // A layer with nothing on it grows into nothing.
        assert!(grown(&vec![0.0; 16], 4, 4, 3.0).iter().all(|a| *a == 0.0));
    }

    #[test]
    fn a_pattern_repeats_and_scales() {
        assert_eq!(pattern(0, 1.0, 1.0), pattern(0, 17.0, 1.0));
        assert_ne!(pattern(0, 1.0, 1.0), pattern(0, 9.0, 1.0));
        for which in 0..PATTERNS.len() {
            let tones: std::collections::HashSet<u8> = (0..32).flat_map(|y| (0..32).map(move |x| (pattern(which, x as f32, y as f32)[0] * 100.0) as u8)).collect();
            assert_eq!(tones.len(), 2, "{}", PATTERNS[which]);
        }
        // Twice the scale, squares twice the size.
        let mut overlay = only(PATTERN_OVERLAY);
        overlay.set_value(index(PATTERN_OVERLAY, RANGE), 200.0);
        let big = apply(&square(), &overlay);
        assert_eq!(at(&big, 33, 33), at(&big, 46, 46));
        assert_ne!(at(&big, 33, 33), at(&big, 33, 49));
    }

    #[test]
    fn a_stroke_sits_outside_inside_or_across_the_edge() {
        let layer = square();
        let mut stroke = only(STROKE);
        stroke.set_value(index(STROKE, SIZE), 4.0);
        let red = |px: [f32; 4]| px[0] > 0.9 && px[2] < 0.1 && px[3] > 0.9;
        let outside = apply(&layer, &stroke);
        assert!(red(at(&outside, 18, 40)) && !red(at(&outside, 22, 40)) && at(&outside, 12, 40)[3] < 0.01, "{:?}", at(&outside, 18, 40));
        stroke.set_value(index(STROKE, SOURCE), 1.0);
        let inside = apply(&layer, &stroke);
        assert!(red(at(&inside, 21, 40)) && at(&inside, 18, 40)[3] < 0.01 && !red(at(&inside, 27, 40)), "{:?}", at(&inside, 21, 40));
        stroke.set_value(index(STROKE, SOURCE), 2.0);
        let across = apply(&layer, &stroke);
        assert!(red(at(&across, 19, 40)) && red(at(&across, 20, 40)) && at(&across, 15, 40)[3] < 0.01 && !red(at(&across, 24, 40)));
        // Filled with a gradient, it goes from its color at the bottom to the other at the top.
        stroke.set_value(index(STROKE, SOURCE), 0.0);
        stroke.set_value(index(STROKE, STYLE), 1.0);
        let graded = apply(&layer, &stroke);
        assert!(at(&graded, 40, 61)[1] < 0.1 && at(&graded, 40, 18)[1] > 0.9, "{:?}", at(&graded, 40, 18));
        // Filled with a pattern, it is gray.
        stroke.set_value(index(STROKE, STYLE), 2.0);
        let patterned = at(&apply(&layer, &stroke), 18, 40);
        assert!(near(patterned[0], patterned[2]) && patterned[3] > 0.9);
    }

    #[test]
    fn a_bevel_is_lit_on_the_side_toward_the_light() {
        let layer = square();
        let lightness = |px: [f32; 4]| px[0] + px[1] + px[2];
        // Inner bevel, light from the upper left: the left edge brightens and the right one darkens, on the object only.
        let inner = apply(&layer, &only(BEVEL));
        assert!(lightness(at(&inner, 21, 40)) > 1.2 && lightness(at(&inner, 58, 40)) < 0.8, "{:?} {:?}", at(&inner, 21, 40), at(&inner, 58, 40));
        assert_eq!(at(&inner, 40, 40), [0.0, 0.0, 1.0, 1.0]);
        assert!(inner.pixels.iter().zip(&layer.pixels).all(|(after, before)| after[3] == before[3]));
        // Pointing down, the sides change places.
        let mut bevel = only(BEVEL);
        bevel.set_value(index(BEVEL, ALIGN), 1.0);
        let down = apply(&layer, &bevel);
        assert!(lightness(at(&down, 21, 40)) < 0.8 && lightness(at(&down, 58, 40)) > 1.2);
        bevel.set_value(index(BEVEL, ALIGN), 0.0);
        // An outer bevel is drawn around the object and leaves the object alone.
        bevel.set_value(index(BEVEL, STYLE), 0.0);
        let outer = apply(&layer, &bevel);
        assert!(at(&outer, 18, 40)[3] > 0.1 && at(&outer, 22, 40) == [0.0, 0.0, 1.0, 1.0], "{:?}", at(&outer, 18, 40));
        // An emboss is on both sides of the edge.
        bevel.set_value(index(BEVEL, STYLE), 2.0);
        let emboss = apply(&layer, &bevel);
        assert!(at(&emboss, 18, 40)[3] > 0.1 && lightness(at(&emboss, 21, 40)) > 1.1);
        // A stroke emboss needs a stroke, and stays on it.
        bevel.set_value(index(BEVEL, STYLE), 4.0);
        assert_eq!(apply(&layer, &bevel).pixels, layer.pixels);
        bevel.set_value(index(STROKE, ON), 1.0);
        bevel.set_value(index(STROKE, SIZE), 6.0);
        let mut plain = bevel.clone();
        plain.set_value(index(BEVEL, ON), 0.0);
        let (embossed, stroked) = (apply(&layer, &bevel), apply(&layer, &plain));
        assert_ne!(at(&embossed, 16, 40), at(&stroked, 16, 40));
        assert_eq!(at(&embossed, 40, 40), at(&stroked, 40, 40));
        // With the light straight overhead nothing is lit from the side.
        let mut flat = only(BEVEL);
        flat.set_value(index(BEVEL, SPREAD), 90.0);
        assert!(near(lightness(at(&apply(&layer, &flat), 21, 40)), 1.0));
        // A full depth shades more than a small one.
        let mut shallow = only(BEVEL);
        shallow.set_value(index(BEVEL, DISTANCE), 20.0);
        assert!(lightness(at(&apply(&layer, &shallow), 21, 40)) < lightness(at(&inner, 21, 40)));
    }

    #[test]
    fn an_outer_glow_surrounds_the_object_and_a_short_range_keeps_it_close() {
        let layer = square();
        let mut glow = only(OUTER_GLOW);
        glow.set_value(index(OUTER_GLOW, SIZE), 10.0);
        let soft = apply(&layer, &glow);
        assert!(at(&soft, 17, 40)[3] > 0.1 && at(&soft, 62, 40)[3] > 0.1 && at(&soft, 40, 17)[3] > 0.1 && at(&soft, 2, 2)[3] < 0.01);
        assert_eq!(at(&soft, 40, 40), [0.0, 0.0, 1.0, 1.0]);
        // Precise is stronger near the edge than softer.
        glow.set_value(index(OUTER_GLOW, STYLE), 1.0);
        let precise = apply(&layer, &glow);
        assert!(at(&precise, 18, 40)[3] > at(&soft, 18, 40)[3]);
        // A longer range thins it out.
        glow.set_value(index(OUTER_GLOW, STYLE), 0.0);
        glow.set_value(index(OUTER_GLOW, RANGE), 100.0);
        assert!(at(&apply(&layer, &glow), 17, 40)[3] < at(&soft, 17, 40)[3]);
    }

    #[test]
    fn what_goes_behind_the_object_is_kept_apart_with_its_blend_mode() {
        let layer = square();
        let mut filter = LiveFilter::new(FilterKind::PhotoshopEffects);
        filter.set_value(index(OUTER_GLOW, ON), 1.0);
        let (object, under) = apply_apart(&layer, &filter);
        // The object comes back alone; the shadow is the lowest, then the glow.
        assert_eq!(object.pixels, layer.pixels);
        assert_eq!(under.iter().map(|(_, mode)| *mode).collect::<Vec<_>>(), [BlendMode::Multiply, BlendMode::Screen]);
        assert!(at(&under[0].0, 61, 62)[3] > 0.2 && at(&under[0].0, 40, 40)[3] == 0.0, "knocked out under the object");
        assert!(at(&under[1].0, 18, 40)[3] > 0.1 && at(&under[1].0, 18, 40)[0] > 0.9);
        // Not knocked out, the shadow goes on under the object.
        filter.set_value(index(DROP_SHADOW, FLAG), 0.0);
        assert!(at(&apply_apart(&layer, &filter).1[0].0, 45, 45)[3] > 0.5);
        // Put together they are what the filter gives in one piece.
        let whole = apply(&layer, &filter);
        assert_eq!(at(&whole, 40, 40), [0.0, 0.0, 1.0, 1.0]);
        assert!(at(&whole, 18, 40)[3] > 0.1 && at(&whole, 61, 62)[3] > 0.2);
        // With neither on there is nothing behind.
        assert!(apply_apart(&layer, &only(COLOR_OVERLAY)).1.is_empty());
    }

    #[test]
    fn satin_shades_the_middle_unless_it_is_not_inverted() {
        let layer = square();
        let mut satin = only(SATIN);
        satin.set_value(index(SATIN, BLEND), 0.0);
        satin.set_value(index(SATIN, OPACITY), 100.0);
        // Inverted, as it starts, the middle is shaded, where both shifted copies cover.
        let inverted = apply(&layer, &satin);
        assert!(at(&inverted, 40, 40)[2] < 0.1, "{:?}", at(&inverted, 40, 40));
        // Not inverted, the middle is left and the shading is where the copies differ.
        satin.set_value(index(SATIN, FLAG), 0.0);
        let plain = apply(&layer, &satin);
        assert!(at(&plain, 40, 40)[2] > 0.9 && plain.pixels != layer.pixels);
    }

    #[test]
    fn the_reach_covers_what_is_drawn_around_the_object() {
        let mut filter = only(COLOR_OVERLAY);
        assert_eq!(reach(&filter), 2.0);
        filter.set_value(index(STROKE, ON), 1.0);
        filter.set_value(index(STROKE, SIZE), 10.0);
        assert_eq!(reach(&filter), 32.0);
        // A stroke inside the object needs no room.
        filter.set_value(index(STROKE, SOURCE), 1.0);
        assert_eq!(reach(&filter), 2.0);
        filter.set_value(index(DROP_SHADOW, ON), 1.0);
        assert!(reach(&filter) > 15.0);
    }
}
