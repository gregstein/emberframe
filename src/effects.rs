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

//! Live filters: effects kept on an object and applied again every time it is drawn,
//! so the object underneath stays editable.
//!
//! A filter works on the object drawn alone on a transparent layer. [`apply`] takes
//! that layer as straight RGBA pixels and returns it changed. The list of filters,
//! their order and their settings follow the Filters menu of Fireworks.

use serde::{Deserialize, Serialize};

use crate::blend::BlendMode;
use crate::i18n::t;
use crate::model::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterKind {
    AutoLevels,
    BrightnessContrast,
    ColorFill,
    Curves,
    HueSaturation,
    Invert,
    Levels,
    PhotoshopEffects,
    InnerBevel,
    InsetEmboss,
    OuterBevel,
    RaisedEmboss,
    Blur,
    BlurMore,
    GaussianBlur,
    MotionBlur,
    RadialBlur,
    ZoomBlur,
    AddNoise,
    ConvertToAlpha,
    FindEdges,
    DropShadow,
    Glow,
    InnerGlow,
    InnerShadow,
    SolidShadow,
    Sharpen,
    SharpenMore,
    UnsharpMask,
}

use FilterKind::*;

/// The groups of the Filters menu, in its order.
pub const MENU: [(&str, &[FilterKind]); 8] = [
    ("Adjust Color", &[AutoLevels, BrightnessContrast, ColorFill, Curves, HueSaturation, Invert, Levels]),
    ("Bevel and Emboss", &[InnerBevel, InsetEmboss, OuterBevel, RaisedEmboss]),
    ("Blur", &[Blur, BlurMore, GaussianBlur, MotionBlur, RadialBlur, ZoomBlur]),
    ("Noise", &[AddNoise]),
    ("Other", &[ConvertToAlpha, FindEdges]),
    ("Shadow and Glow", &[DropShadow, Glow, InnerGlow, InnerShadow, SolidShadow]),
    ("Sharpen", &[Sharpen, SharpenMore, UnsharpMask]),
    ("Photoshop Live Effects", &[PhotoshopEffects]),
];

/// One setting of a filter: its name, its range and what it starts at.
#[derive(Clone, Copy, Debug)]
pub struct Setting {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub start: f32,
    /// A setting that is only on or off, shown as a box to tick.
    pub toggle: bool,
}

const fn number(name: &'static str, min: f32, max: f32, start: f32) -> Setting {
    Setting { name, min, max, start, toggle: false }
}

const fn toggle(name: &'static str, start: bool) -> Setting {
    Setting { name, min: 0.0, max: 1.0, start: if start { 1.0 } else { 0.0 }, toggle: true }
}

/// The channels Levels and Curves can work on, by the number stored for "Channel".
pub const CHANNELS: [&str; 4] = ["RGB", "Red", "Green", "Blue"];

/// Where the five Levels numbers of a channel start among the settings: all channels
/// together first, as before channels could be told apart, then red, green and blue.
pub fn levels_start(channel: usize) -> usize {
    if channel == 0 { 0 } else { 6 + (channel.min(3) - 1) * 5 }
}

/// The edge shapes of a bevel, by the number stored for "Edge shape".
pub const BEVEL_SHAPES: [&str; 7] = ["Flat", "Smooth", "Sloped", "Frame 1", "Frame 2", "Ring", "Ruffle"];

impl FilterKind {
    pub fn label(self) -> &'static str {
        match self {
            AutoLevels => t("Auto Levels"),
            BrightnessContrast => t("Brightness/Contrast"),
            ColorFill => t("Color Fill"),
            Curves => t("Curves"),
            PhotoshopEffects => t("Photoshop Live Effects"),
            HueSaturation => t("Hue/Saturation"),
            Invert => t("Invert"),
            Levels => t("Levels"),
            InnerBevel => t("Inner Bevel"),
            InsetEmboss => t("Inset Emboss"),
            OuterBevel => t("Outer Bevel"),
            RaisedEmboss => t("Raised Emboss"),
            Blur => t("Blur"),
            BlurMore => t("Blur More"),
            GaussianBlur => t("Gaussian Blur"),
            MotionBlur => t("Motion Blur"),
            RadialBlur => t("Radial Blur"),
            ZoomBlur => t("Zoom Blur"),
            AddNoise => t("Add Noise"),
            ConvertToAlpha => t("Convert to Alpha"),
            FindEdges => t("Find Edges"),
            DropShadow => t("Drop Shadow"),
            Glow => t("Glow"),
            InnerGlow => t("Inner Glow"),
            InnerShadow => t("Inner Shadow"),
            SolidShadow => t("Solid Shadow"),
            Sharpen => t("Sharpen"),
            SharpenMore => t("Sharpen More"),
            UnsharpMask => t("Unsharp Mask"),
        }
    }

    /// The numbers the filter is set with, in the order they are stored.
    pub fn settings(self) -> &'static [Setting] {
        const BEVEL: [Setting; 5] = [
            number("Edge shape", 0.0, 6.0, 1.0),
            number("Width", 0.0, 35.0, 10.0),
            number("Contrast", 0.0, 100.0, 75.0),
            number("Softness", 0.0, 10.0, 3.0),
            number("Angle", 0.0, 360.0, 135.0),
        ];
        const EMBOSS: [Setting; 5] = [
            number("Width", 0.0, 35.0, 2.0),
            number("Contrast", 0.0, 100.0, 75.0),
            number("Softness", 0.0, 10.0, 2.0),
            number("Angle", 0.0, 360.0, 135.0),
            toggle("Show object", true),
        ];
        const SHADOW: [Setting; 5] = [
            number("Distance", 0.0, 100.0, 7.0),
            number("Opacity", 0.0, 100.0, 65.0),
            number("Softness", 0.0, 30.0, 4.0),
            number("Angle", 0.0, 360.0, 315.0),
            toggle("Knock out", false),
        ];
        const GLOW: [Setting; 4] =
            [number("Width", 0.0, 35.0, 6.0), number("Opacity", 0.0, 100.0, 65.0), number("Softness", 0.0, 30.0, 12.0), number("Offset", 0.0, 30.0, 0.0)];
        const BRIGHTNESS: [Setting; 2] = [number("Brightness", -100.0, 100.0, 0.0), number("Contrast", -100.0, 100.0, 0.0)];
        const FILL: [Setting; 2] = [number("Opacity", 0.0, 100.0, 100.0), number("Blend Mode", 0.0, 44.0, 0.0)];
        const HUE: [Setting; 4] =
            [number("Hue", -180.0, 180.0, 0.0), number("Saturation", -100.0, 100.0, 0.0), number("Lightness", -100.0, 100.0, 0.0), toggle("Colorize", false)];
        const LEVEL: [Setting; 5] = [
            number("Input black", 0.0, 255.0, 0.0),
            number("Gamma", 0.1, 10.0, 1.0),
            number("Input white", 0.0, 255.0, 255.0),
            number("Output black", 0.0, 255.0, 0.0),
            number("Output white", 0.0, 255.0, 255.0),
        ];
        // The five numbers for all channels together, the channel on show, then five each for red, green and blue.
        const LEVELS: [Setting; 21] = {
            let mut all = [number("Channel", 0.0, 3.0, 0.0); 21];
            let mut i = 0;
            while i < 21 {
                if i != 5 {
                    all[i] = LEVEL[if i < 5 { i } else { (i - 6) % 5 }];
                }
                i += 1;
            }
            all
        };
        const CURVES: [Setting; 1] = [number("Channel", 0.0, 3.0, 0.0)];
        const GAUSSIAN: [Setting; 1] = [number("Radius", 0.1, 250.0, 4.0)];
        const MOTION: [Setting; 2] = [number("Angle", 0.0, 360.0, 0.0), number("Distance", 1.0, 100.0, 10.0)];
        const SPIN: [Setting; 2] = [number("Amount", 1.0, 100.0, 10.0), number("Quality", 1.0, 100.0, 20.0)];
        const NOISE: [Setting; 2] = [number("Amount", 0.0, 400.0, 20.0), toggle("Color", false)];
        const SOLID: [Setting; 3] = [number("Angle", 0.0, 360.0, 315.0), number("Distance", 0.0, 100.0, 10.0), toggle("Solid color", true)];
        const UNSHARP: [Setting; 3] =
            [number("Sharpen amount", 1.0, 500.0, 50.0), number("Pixel radius", 0.1, 250.0, 1.0), number("Threshold", 0.0, 255.0, 0.0)];
        match self {
            BrightnessContrast => &BRIGHTNESS,
            ColorFill => &FILL,
            HueSaturation => &HUE,
            Levels => &LEVELS,
            Curves => &CURVES,
            PhotoshopEffects => crate::pseffects::settings(),
            InnerBevel | OuterBevel => &BEVEL,
            InsetEmboss | RaisedEmboss => &EMBOSS,
            GaussianBlur => &GAUSSIAN,
            MotionBlur => &MOTION,
            RadialBlur | ZoomBlur => &SPIN,
            AddNoise => &NOISE,
            DropShadow | InnerShadow => &SHADOW,
            Glow | InnerGlow => &GLOW,
            SolidShadow => &SOLID,
            UnsharpMask => &UNSHARP,
            AutoLevels | Invert | Blur | BlurMore | ConvertToAlpha | FindEdges | Sharpen | SharpenMore => &[],
        }
    }

    /// The color the filter starts with, for those that have one.
    pub fn start_color(self) -> Option<Color> {
        match self {
            ColorFill | Glow | InnerGlow => Some(Color::rgb(255, 0, 0)),
            PhotoshopEffects => Some(Color::BLACK),
            OuterBevel => Some(Color::rgb(223, 223, 223)),
            DropShadow | InnerShadow | SolidShadow => Some(Color::BLACK),
            _ => None,
        }
    }
}

/// A filter on an object, with its settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveFilter {
    pub kind: FilterKind,
    pub values: Vec<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default = "on")]
    pub enabled: bool,
    /// The curves of a Curves filter: one for all channels together, then red, green and
    /// blue. Each point is an input level and the output it gives, from 0 to 255.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curves: Vec<Vec<(f32, f32)>>,
    /// The colors of the effects in Photoshop Live Effects, one for each.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<Color>,
    /// The gradients of Photoshop Live Effects, by their numbers there. One that is empty has not been made yet.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gradients: Vec<Vec<crate::pseffects::Stop>>,
    /// The opacity stops of those gradients: only the alpha of each stop counts. None means opaque throughout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gradient_opacity: Vec<Vec<crate::pseffects::Stop>>,
}

/// The curve that changes nothing, which every channel starts with.
pub const STRAIGHT: [(f32, f32); 2] = [(0.0, 0.0), (255.0, 255.0)];

/// Where the middle slider of Levels stands between the black and the white one,
/// from 0 to 1, for a gamma: the level that comes out as mid gray.
pub fn gamma_place(gamma: f32) -> f32 {
    0.5f32.powf(gamma.clamp(0.1, 10.0))
}

/// The five numbers of a Levels channel after one of its sliders is dragged to
/// `level`: 0 to 2 are black, gamma and white on the way in, 3 and 4 black and
/// white on the way out. Black and white on the way in cannot meet or cross.
pub fn levels_dragged(mut values: [f32; 5], slider: usize, level: f32) -> [f32; 5] {
    let exact = level.clamp(0.0, 255.0);
    let level = exact.round();
    match slider {
        0 => values[0] = level.min(values[2] - 2.0).max(0.0),
        1 => {
            let place = ((exact - values[0]) / (values[2] - values[0]).max(1.0)).clamp(0.001, 0.999);
            values[1] = ((place.ln() / 0.5f32.ln()).clamp(0.1, 10.0) * 100.0).round() / 100.0;
        }
        2 => values[2] = level.max(values[0] + 2.0).min(255.0),
        3 | 4 => values[slider] = level,
        _ => {}
    }
    values
}

/// The filters as they will be with the edited one put at its place among those
/// from before the dialog opened: in place of the one there, or at the end if it is new.
pub fn placed(before: &[LiveFilter], index: usize, edited: LiveFilter) -> Vec<LiveFilter> {
    let mut filters = before.to_vec();
    match filters.get_mut(index) {
        Some(slot) => *slot = edited,
        None => filters.push(edited),
    }
    filters
}

/// A press on a curve at `level`: the point within `reach` levels of it is taken
/// hold of, or a new one is put there. Gives the points and the one held.
pub fn curve_pressed(mut points: Vec<(f32, f32)>, level: (f32, f32), reach: f32) -> (Vec<(f32, f32)>, usize) {
    let level = (level.0.clamp(0.0, 255.0).round(), level.1.clamp(0.0, 255.0).round());
    if let Some(index) = points.iter().position(|p| (p.0 - level.0).abs() <= reach && (p.1 - level.1).abs() <= reach) {
        return (points, index);
    }
    // Two points cannot share an input level: the new one goes beside the old.
    let taken = |x: f32, points: &[(f32, f32)]| points.iter().any(|p| (p.0 - x).abs() < 0.5);
    let mut x = level.0;
    while taken(x, &points) {
        x += if level.0 < 128.0 { 1.0 } else { -1.0 };
    }
    points.push((x, level.1));
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    let index = points.iter().position(|p| p.0 == x).unwrap_or(0);
    (points, index)
}

/// The held point of a curve dragged to `level`. It stays between its neighbors;
/// one taken more than `out` levels outside the box leaves the curve, unless it is
/// an end. Gives the points and whether the point is still there.
pub fn curve_dragged(mut points: Vec<(f32, f32)>, index: usize, level: (f32, f32), out: f32) -> (Vec<(f32, f32)>, bool) {
    if index >= points.len() {
        return (points, false);
    }
    let is_end = index == 0 || index + 1 == points.len();
    let outside = |v: f32| v < -out || v > 255.0 + out;
    if !is_end && (outside(level.0) || outside(level.1)) {
        points.remove(index);
        return (points, false);
    }
    let low = if index == 0 { 0.0 } else { points[index - 1].0 + 1.0 };
    let high = if index + 1 == points.len() { 255.0 } else { points[index + 1].0 - 1.0 };
    points[index] = (level.0.round().clamp(low, high.max(low)), level.1.round().clamp(0.0, 255.0));
    (points, true)
}

fn on() -> bool {
    true
}

impl LiveFilter {
    pub fn new(kind: FilterKind) -> Self {
        Self {
            kind,
            values: kind.settings().iter().map(|s| s.start).collect(),
            color: kind.start_color(),
            enabled: true,
            curves: if kind == Curves { vec![STRAIGHT.to_vec(); 4] } else { Vec::new() },
            colors: if kind == PhotoshopEffects { crate::pseffects::start_colors() } else { Vec::new() },
            gradients: Vec::new(),
            gradient_opacity: Vec::new(),
        }
    }

    /// A setting by its place, kept inside its range. Files with fewer values than
    /// the filter has today read the missing ones at their starting value.
    pub fn value(&self, index: usize) -> f32 {
        let Some(setting) = self.kind.settings().get(index) else { return 0.0 };
        self.values.get(index).copied().filter(|v| v.is_finite()).unwrap_or(setting.start).clamp(setting.min, setting.max)
    }

    pub fn set_value(&mut self, index: usize, value: f32) {
        let Some(setting) = self.kind.settings().get(index) else { return };
        self.values.resize(self.kind.settings().len(), 0.0);
        self.values[index] = if value.is_finite() { value.clamp(setting.min, setting.max) } else { setting.start };
    }

    /// The color stops of one of the gradients of Photoshop Live Effects, or with `opacity` its opacity stops, in order.
    pub fn gradient_stops(&self, gradient: usize, opacity: bool) -> Vec<crate::pseffects::Stop> {
        let mut stops = if opacity {
            self.gradient_opacity.get(gradient).cloned().unwrap_or_default()
        } else {
            self.gradients.get(gradient).filter(|stops| stops.len() >= 2).cloned().unwrap_or_else(|| crate::pseffects::start_gradient(self, gradient))
        };
        stops.sort_by(|a, b| a.0.total_cmp(&b.0));
        stops
    }

    /// One of the gradients of Photoshop Live Effects as it is drawn: its colors with its opacity stops worked in.
    pub fn gradient(&self, gradient: usize) -> Vec<crate::pseffects::Stop> {
        crate::pseffects::baked(&self.gradient_stops(gradient, false), &self.gradient_stops(gradient, true))
    }

    #[cfg(test)]
    pub fn set_gradient(&mut self, gradient: usize, stops: Vec<crate::pseffects::Stop>) {
        self.set_gradient_stops(gradient, false, stops);
    }

    pub fn set_gradient_stops(&mut self, gradient: usize, opacity: bool, stops: Vec<crate::pseffects::Stop>) {
        let list = if opacity { &mut self.gradient_opacity } else { &mut self.gradients };
        if list.len() <= gradient {
            list.resize(gradient + 1, Vec::new());
        }
        list[gradient] = stops;
    }

    /// One of the colors of Photoshop Live Effects, by its number there.
    pub fn ps_color(&self, color: usize) -> Color {
        self.colors.get(color).copied().or_else(|| crate::pseffects::start_colors().get(color).copied()).unwrap_or(Color::BLACK)
    }

    pub fn set_ps_color(&mut self, color: usize, to: Color) {
        let count = crate::pseffects::start_colors().len();
        if self.colors.len() < count {
            self.colors = (0..count).map(|i| self.ps_color(i)).collect();
        }
        if let Some(slot) = self.colors.get_mut(color) {
            *slot = to;
        }
    }

    /// The points of a channel's curve, from left to right.
    pub fn curve(&self, channel: usize) -> Vec<(f32, f32)> {
        let mut points: Vec<(f32, f32)> = self.curves.get(channel).filter(|c| !c.is_empty()).map_or(STRAIGHT.to_vec(), |c| c.clone());
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        points
    }

    pub fn set_curve(&mut self, channel: usize, points: Vec<(f32, f32)>) {
        self.curves.resize(4, STRAIGHT.to_vec());
        if let Some(curve) = self.curves.get_mut(channel) {
            *curve = points;
        }
    }

    /// What the eyedroppers of Levels and Curves do with the color picked: 0 makes it
    /// black, 2 makes it white, and 1 makes it a gray, taking any tint out of it.
    /// `sample` is the red, green and blue picked, from 0 to 255.
    pub fn pick_point(&mut self, dropper: usize, sample: [f32; 3]) {
        let mean = (sample[0] + sample[1] + sample[2]) / 3.0;
        for (channel, level) in sample.into_iter().enumerate() {
            let level = level.clamp(0.0, 255.0).round();
            match self.kind {
                Levels => {
                    let at = levels_start(channel + 1);
                    let (black, white) = (self.value(at), self.value(at + 2));
                    match dropper {
                        0 => self.set_value(at, level.min(white - 2.0).max(0.0)),
                        2 => self.set_value(at + 2, level.max(black + 2.0)),
                        _ => {
                            // Each color's gamma is set so that all three come out at the level they share on average.
                            let place = |v: f32| ((v - black) / (white - black).max(1.0)).clamp(0.001, 0.999);
                            let shared = ((mean - black) / (white - black).max(1.0)).clamp(0.001, 0.999);
                            self.set_value(at + 1, place(level).ln() / shared.ln());
                        }
                    }
                }
                Curves => {
                    let mut points = self.curve(channel + 1);
                    match dropper {
                        0 => {
                            points.retain(|p| p.0 > level);
                            points.insert(0, (level.min(253.0), 0.0));
                        }
                        2 => {
                            points.retain(|p| p.0 < level);
                            points.push((level.max(2.0), 255.0));
                        }
                        _ => {
                            points.retain(|p| (p.0 - level).abs() >= 1.0 || p.0 <= 0.0 || p.0 >= 255.0);
                            if level > 0.0 && level < 255.0 {
                                points.push((level, mean.round()));
                            }
                        }
                    }
                    points.sort_by(|a, b| a.0.total_cmp(&b.0));
                    if points.len() < 2 {
                        points = STRAIGHT.to_vec();
                    }
                    self.set_curve(channel + 1, points);
                }
                _ => {}
            }
        }
    }

    /// What the Auto button of Levels and Curves does: each color is stretched from
    /// its darkest level in the object to its lightest, as Auto Levels does. `counts`
    /// is how many pixels have each level of red, green and blue.
    pub fn stretch(&mut self, counts: &[[u32; 256]; 3]) {
        for (channel, counts) in counts.iter().enumerate() {
            let (Some(low), Some(high)) = (counts.iter().position(|n| *n > 0), counts.iter().rposition(|n| *n > 0)) else { continue };
            if high <= low {
                continue;
            }
            match self.kind {
                Levels => {
                    for (offset, value) in [low as f32, 1.0, high as f32, 0.0, 255.0].into_iter().enumerate() {
                        self.set_value(levels_start(channel + 1) + offset, value);
                    }
                }
                Curves => self.set_curve(channel + 1, vec![(low as f32, 0.0), (high as f32, 255.0)]),
                _ => {}
            }
        }
    }

    /// The output level for every input level of a channel's curve: a smooth curve
    /// through its points that never turns back on itself.
    pub fn curve_table(&self, channel: usize) -> [f32; 256] {
        let mut points: Vec<(f32, f32)> = self.curve(channel).iter().map(|(x, y)| (x.clamp(0.0, 255.0), y.clamp(0.0, 255.0))).collect();
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        points.dedup_by(|a, b| (a.0 - b.0).abs() < 0.5);
        let mut table = [0.0; 256];
        if points.len() < 2 {
            for (level, slot) in table.iter_mut().enumerate() {
                *slot = points.first().map_or(level as f32, |p| p.1);
            }
            return table;
        }
        // Monotone cubic interpolation (Fritsch and Carlson): the slope at each point
        // is cut back where a plain spline would overshoot.
        let count = points.len();
        let slopes: Vec<f32> = (0..count - 1).map(|i| (points[i + 1].1 - points[i].1) / (points[i + 1].0 - points[i].0)).collect();
        let mut tangents: Vec<f32> = (0..count)
            .map(|i| match i {
                0 => slopes[0],
                i if i == count - 1 => slopes[count - 2],
                i if slopes[i - 1] * slopes[i] <= 0.0 => 0.0,
                i => (slopes[i - 1] + slopes[i]) / 2.0,
            })
            .collect();
        for i in 0..count - 1 {
            if slopes[i] == 0.0 {
                (tangents[i], tangents[i + 1]) = (0.0, 0.0);
                continue;
            }
            let (a, b) = (tangents[i] / slopes[i], tangents[i + 1] / slopes[i]);
            let length = (a * a + b * b).sqrt();
            if length > 3.0 {
                (tangents[i], tangents[i + 1]) = (3.0 * a / length * slopes[i], 3.0 * b / length * slopes[i]);
            }
        }
        for (level, slot) in table.iter_mut().enumerate() {
            let x = level as f32;
            *slot = if x <= points[0].0 {
                points[0].1
            } else if x >= points[count - 1].0 {
                points[count - 1].1
            } else {
                let i = (0..count - 1).find(|i| x <= points[i + 1].0).unwrap_or(count - 2);
                let width = points[i + 1].0 - points[i].0;
                let t = (x - points[i].0) / width;
                let (t2, t3) = (t * t, t * t * t);
                (2.0 * t3 - 3.0 * t2 + 1.0) * points[i].1
                    + (t3 - 2.0 * t2 + t) * width * tangents[i]
                    + (-2.0 * t3 + 3.0 * t2) * points[i + 1].1
                    + (t3 - t2) * width * tangents[i + 1]
            }
            .clamp(0.0, 255.0);
        }
        table
    }

    fn rgb(&self) -> [f32; 3] {
        let c = self.color.or(self.kind.start_color()).unwrap_or(Color::BLACK);
        [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0]
    }
}

/// An object drawn alone, as straight RGBA with every channel from 0 to 1.
#[derive(Clone)]
pub struct Layer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[f32; 4]>,
}

impl Layer {
    fn at(&self, x: i64, y: i64) -> [f32; 4] {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return [0.0; 4];
        }
        self.pixels[y as usize * self.width + x as usize]
    }

    /// The pixel at a point between pixels, mixed from the four around it.
    fn sample(&self, x: f32, y: f32) -> [f32; 4] {
        let (left, top) = (x.floor(), y.floor());
        let (fx, fy) = (x - left, y - top);
        let (left, top) = (left as i64, top as i64);
        let mut out = [0.0; 4];
        for (dx, dy, weight) in [(0, 0, (1.0 - fx) * (1.0 - fy)), (1, 0, fx * (1.0 - fy)), (0, 1, (1.0 - fx) * fy), (1, 1, fx * fy)] {
            let px = self.at(left + dx, top + dy);
            // Weighed by alpha, so clear pixels do not darken what is mixed with them.
            for c in 0..3 {
                out[c] += px[c] * px[3] * weight;
            }
            out[3] += px[3] * weight;
        }
        if out[3] > 0.0 {
            for c in 0..3 {
                out[c] /= out[3];
            }
        }
        out
    }

    pub(crate) fn alpha(&self) -> Vec<f32> {
        self.pixels.iter().map(|p| p[3]).collect()
    }
}

/// Blurs one channel with three passes of a box, which is close to a Gaussian of that radius.
pub(crate) fn blur_channel(values: &[f32], width: usize, height: usize, radius: f32) -> Vec<f32> {
    if radius <= 0.05 || width == 0 || height == 0 {
        return values.to_vec();
    }
    // Three boxes of this half-width have the spread of a Gaussian with sigma = radius / 2.
    let sigma = radius / 2.0;
    let half = ((sigma * sigma * 4.0 + 1.0).sqrt() - 1.0) / 2.0;
    let (whole, part) = (half.floor() as i64, half.fract());
    let pass = |source: &[f32], length: usize, stride: usize, lines: usize, line_stride: usize| {
        let mut out = vec![0.0; source.len()];
        let weight = 1.0 / (2.0 * half + 1.0);
        for line in 0..lines {
            let base = line * line_stride;
            let get = |i: i64| if i < 0 || i >= length as i64 { 0.0 } else { source[base + i as usize * stride] };
            for i in 0..length as i64 {
                let mut sum = 0.0;
                for k in -whole..=whole {
                    sum += get(i + k);
                }
                sum += part * (get(i - whole - 1) + get(i + whole + 1));
                out[base + i as usize * stride] = sum * weight;
            }
        }
        out
    };
    let mut current = values.to_vec();
    for _ in 0..3 {
        current = pass(&current, width, 1, height, width);
        current = pass(&current, height, width, width, 1);
    }
    current
}

/// Softens a channel the way Fireworks softens a shadow. A file written by Fireworks,
/// with a drop shadow of softness 1, shows each pixel spread over its neighbours as
/// 1, 2, 1 across and down: a box of softness + 1 pixels passed over twice, once
/// leaning each way. Softness above 1 follows the same rule but was not in the file.
pub(crate) fn soften(values: &[f32], width: usize, height: usize, softness: f32) -> Vec<f32> {
    let reach = softness.round().max(0.0) as i64;
    if reach == 0 || width == 0 || height == 0 {
        return values.to_vec();
    }
    let weight = 1.0 / (reach + 1) as f32;
    let pass = |source: &[f32], length: usize, stride: usize, lines: usize, line_stride: usize, forward: bool| {
        let mut out = vec![0.0; source.len()];
        for line in 0..lines {
            let base = line * line_stride;
            let get = |i: i64| if i < 0 || i >= length as i64 { 0.0 } else { source[base + i as usize * stride] };
            for i in 0..length as i64 {
                let window = if forward { i..=i + reach } else { i - reach..=i };
                out[base + i as usize * stride] = window.map(get).sum::<f32>() * weight;
            }
        }
        out
    };
    let mut current = values.to_vec();
    for forward in [true, false] {
        current = pass(&current, width, 1, height, width, forward);
        current = pass(&current, height, width, width, 1, forward);
    }
    current
}

/// Blurs a layer, colors and transparency together.
fn blur_layer(layer: &Layer, radius: f32) -> Layer {
    let (w, h) = (layer.width, layer.height);
    let alpha = blur_channel(&layer.alpha(), w, h, radius);
    let mut out = layer.clone();
    for c in 0..3 {
        let premultiplied: Vec<f32> = layer.pixels.iter().map(|p| p[c] * p[3]).collect();
        let blurred = blur_channel(&premultiplied, w, h, radius);
        for (px, (value, a)) in out.pixels.iter_mut().zip(blurred.iter().zip(&alpha)) {
            px[c] = if *a > 1e-4 { (value / a).clamp(0.0, 1.0) } else { 0.0 };
        }
    }
    for (px, a) in out.pixels.iter_mut().zip(&alpha) {
        px[3] = a.clamp(0.0, 1.0);
    }
    out
}

/// Puts `top` over `bottom`.
pub(crate) fn over(top: [f32; 4], bottom: [f32; 4]) -> [f32; 4] {
    let alpha = top[3] + bottom[3] * (1.0 - top[3]);
    if alpha <= 0.0 {
        return [0.0; 4];
    }
    let mut out = [0.0, 0.0, 0.0, alpha];
    for c in 0..3 {
        out[c] = (top[c] * top[3] + bottom[c] * bottom[3] * (1.0 - top[3])) / alpha;
    }
    out
}

fn luminance(p: [f32; 4]) -> f32 {
    0.3 * p[0] + 0.59 * p[1] + 0.11 * p[2]
}

/// A number from -1 to 1 that is always the same for the same pixel and channel.
pub(crate) fn noise(x: usize, y: usize, channel: usize) -> f32 {
    let mut n = (x as u32).wrapping_mul(374_761_393).wrapping_add((y as u32).wrapping_mul(668_265_263)).wrapping_add((channel as u32).wrapping_mul(2_246_822_519));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n ^ (n >> 16)) & 0xffff) as f32 / 32767.5 - 1.0
}

/// The unit vector a light or a shadow goes along, for an angle in degrees, counterclockwise from the right.
pub(crate) fn direction(degrees: f32) -> (f32, f32) {
    let radians = degrees.to_radians();
    (radians.cos(), -radians.sin())
}

/// Sharpens by adding back the difference from a blurred copy.
fn unsharp(layer: &Layer, amount: f32, radius: f32, threshold: f32) -> Layer {
    let blurred = blur_layer(layer, radius);
    let mut out = layer.clone();
    for (px, soft) in out.pixels.iter_mut().zip(&blurred.pixels) {
        for c in 0..3 {
            let difference = px[c] - soft[c];
            if difference.abs() >= threshold {
                px[c] = (px[c] + difference * amount).clamp(0.0, 1.0);
            }
        }
    }
    out
}

/// How high the surface stands at each pixel of a bevel: 0 at the edge of the object, 1 on the flat top.
fn bevel_height(alpha: &[f32], width: usize, height: usize, bevel: f32, softness: f32, shape: usize, inside: bool) -> Vec<f32> {
    // A blurred edge rises across about the width of the blur; outside the object it is the mirror image.
    let ramp = blur_channel(alpha, width, height, bevel.max(0.5));
    let profile = |t: f32| -> f32 {
        let t = t.clamp(0.0, 1.0);
        match shape {
            0 => t,
            2 => t * t,
            3 => (t * 2.0).min(1.0) * 0.5 + if t > 0.5 { 0.5 } else { 0.0 },
            4 => ((t * 3.0).floor() / 2.0).min(1.0),
            5 => (t * std::f32::consts::PI).sin(),
            6 => 0.5 - 0.5 * (t * std::f32::consts::PI * 3.0).cos(),
            _ => t * t * (3.0 - 2.0 * t),
        }
    };
    let raw: Vec<f32> = ramp
        .iter()
        .map(|r| if inside { profile((r - 0.5) * 2.0) } else { profile(r * 2.0) })
        .collect();
    blur_channel(&raw, width, height, softness)
}

/// Lights a surface from an angle: more than 0 where it faces the light, less where it faces away.
fn lighting(heights: &[f32], width: usize, height: usize, angle: f32) -> Vec<f32> {
    let (lx, ly) = direction(angle);
    let at = |x: i64, y: i64| heights[y.clamp(0, height as i64 - 1) as usize * width + x.clamp(0, width as i64 - 1) as usize];
    let mut out = vec![0.0; heights.len()];
    for y in 0..height as i64 {
        for x in 0..width as i64 {
            let (dx, dy) = (at(x + 1, y) - at(x - 1, y), at(x, y + 1) - at(x, y - 1));
            // The slope rises away from the edge, so the side whose slope points at the light is lit.
            out[y as usize * width + x as usize] = -(dx * lx + dy * ly);
        }
    }
    out
}

/// Applies one filter to a layer.
pub fn apply(layer: &Layer, filter: &LiveFilter) -> Layer {
    let (w, h) = (layer.width, layer.height);
    let v = |index: usize| filter.value(index);
    let each = |change: &dyn Fn([f32; 4], usize, usize) -> [f32; 4]| {
        let mut out = layer.clone();
        for (index, px) in out.pixels.iter_mut().enumerate() {
            if px[3] > 0.0 {
                let changed = change(*px, index % w, index / w);
                *px = [changed[0].clamp(0.0, 1.0), changed[1].clamp(0.0, 1.0), changed[2].clamp(0.0, 1.0), changed[3].clamp(0.0, 1.0)];
            }
        }
        out
    };
    match filter.kind {
        Invert => each(&|p, _, _| [1.0 - p[0], 1.0 - p[1], 1.0 - p[2], p[3]]),
        BrightnessContrast => {
            let (brightness, contrast) = (v(0) / 100.0, v(1) / 100.0);
            // Contrast turns around mid gray: 100 is a step, -100 is flat gray.
            let slope = if contrast >= 0.0 { 1.0 / (1.0 - contrast).max(0.01) } else { 1.0 + contrast };
            each(&|p, _, _| {
                let f = |c: f32| (c - 0.5) * slope + 0.5 + brightness;
                [f(p[0]), f(p[1]), f(p[2]), p[3]]
            })
        }
        ColorFill => {
            let (opacity, color) = (v(0) / 100.0, filter.rgb());
            let mode = BlendMode::ALL[(v(1).round() as usize).min(BlendMode::ALL.len() - 1)];
            each(&|p, _, _| {
                let mixed = mode.mix([p[0], p[1], p[2]], color);
                [p[0] + (mixed[0] - p[0]) * opacity, p[1] + (mixed[1] - p[1]) * opacity, p[2] + (mixed[2] - p[2]) * opacity, p[3]]
            })
        }
        HueSaturation => {
            let (turn, saturation, lightness, colorize) = (v(0), v(1) / 100.0, v(2) / 100.0, v(3) >= 0.5);
            each(&|p, _, _| {
                let (mut hue, mut sat, mut light) = to_hsl([p[0], p[1], p[2]]);
                if colorize {
                    // One hue for everything, as strong as the saturation says.
                    hue = turn.rem_euclid(360.0);
                    sat = ((saturation + 1.0) / 2.0).clamp(0.0, 1.0);
                } else {
                    hue = (hue + turn).rem_euclid(360.0);
                    sat = if saturation >= 0.0 { sat + (1.0 - sat) * saturation } else { sat * (1.0 + saturation) };
                }
                light = if lightness >= 0.0 { light + (1.0 - light) * lightness } else { light * (1.0 + lightness) };
                let rgb = from_hsl(hue, sat, light);
                [rgb[0], rgb[1], rgb[2], p[3]]
            })
        }
        Levels => {
            let set = |channel: usize| {
                let at = levels_start(channel);
                [v(at) / 255.0, v(at + 1), v(at + 2) / 255.0, v(at + 3) / 255.0, v(at + 4) / 255.0]
            };
            let sets = [set(0), set(1), set(2), set(3)];
            let level = |c: f32, [in_black, gamma, in_white, out_black, out_white]: [f32; 5]| {
                out_black + ((c - in_black) / (in_white - in_black).max(1.0 / 255.0)).clamp(0.0, 1.0).powf(1.0 / gamma) * (out_white - out_black)
            };
            // Each color by its own numbers first, then all three by the ones they share.
            each(&|p, _, _| {
                let one = |index: usize| level(level(p[index], sets[index + 1]), sets[0]);
                [one(0), one(1), one(2), p[3]]
            })
        }
        Curves => {
            let tables = [filter.curve_table(0), filter.curve_table(1), filter.curve_table(2), filter.curve_table(3)];
            let read = |c: f32, table: &[f32; 256]| {
                // Between two whole levels the table is read in proportion.
                let level = (c * 255.0).clamp(0.0, 255.0);
                let (low, part) = (level.floor() as usize, level.fract());
                (table[low] + (table[(low + 1).min(255)] - table[low]) * part) / 255.0
            };
            each(&|p, _, _| {
                let one = |index: usize| read(read(p[index], &tables[index + 1]), &tables[0]);
                [one(0), one(1), one(2), p[3]]
            })
        }
        PhotoshopEffects => crate::pseffects::apply(layer, filter),
        AutoLevels => {
            // Each channel is stretched so its darkest value is black and its lightest white.
            let mut range = [(1.0f32, 0.0f32); 3];
            for p in layer.pixels.iter().filter(|p| p[3] > 0.0) {
                for c in 0..3 {
                    range[c] = (range[c].0.min(p[c]), range[c].1.max(p[c]));
                }
            }
            each(&|p, _, _| {
                let f = |c: usize| if range[c].1 > range[c].0 { (p[c] - range[c].0) / (range[c].1 - range[c].0) } else { p[c] };
                [f(0), f(1), f(2), p[3]]
            })
        }
        ConvertToAlpha => each(&|p, _, _| [p[0], p[1], p[2], p[3] * (1.0 - luminance(p))]),
        AddNoise => {
            let (amount, colored) = (v(0) / 255.0, v(1) >= 0.5);
            each(&|p, x, y| {
                let gray = noise(x, y, 0);
                let n = |c: usize| if colored { noise(x, y, c + 1) } else { gray } * amount;
                [p[0] + n(0), p[1] + n(1), p[2] + n(2), p[3]]
            })
        }
        Blur => blur_layer(layer, 1.0),
        BlurMore => blur_layer(layer, 2.0),
        GaussianBlur => blur_layer(layer, v(0)),
        MotionBlur => {
            let ((dx, dy), distance) = (direction(v(0)), v(1));
            let steps = (distance.ceil() as usize).max(1);
            streak(layer, steps, &|x, y, k| (x + dx * (k - 0.5) * distance, y + dy * (k - 0.5) * distance))
        }
        RadialBlur | ZoomBlur => {
            let (amount, steps) = (v(0) / 100.0, (v(1).round() as usize).clamp(2, 100));
            let (cx, cy) = (bounds_center(layer).0, bounds_center(layer).1);
            if filter.kind == ZoomBlur {
                // Each pixel is smeared along the line to the middle of the object.
                streak(layer, steps, &|x, y, k| (cx + (x - cx) * (1.0 - amount * 0.5 * k), cy + (y - cy) * (1.0 - amount * 0.5 * k)))
            } else {
                // Each pixel is smeared along its circle around the middle.
                streak(layer, steps, &|x, y, k| {
                    let (sin, cos) = ((k - 0.5) * amount * 0.5).sin_cos();
                    (cx + (x - cx) * cos - (y - cy) * sin, cy + (x - cx) * sin + (y - cy) * cos)
                })
            }
        }
        Sharpen => unsharp(layer, 0.6, 1.0, 0.0),
        SharpenMore => unsharp(layer, 1.5, 1.0, 0.0),
        UnsharpMask => unsharp(layer, v(0) / 100.0, v(1), v(2) / 255.0),
        FindEdges => {
            let mut out = layer.clone();
            for y in 0..h as i64 {
                for x in 0..w as i64 {
                    let here = layer.at(x, y);
                    if here[3] <= 0.0 {
                        continue;
                    }
                    let mut px = [0.0, 0.0, 0.0, here[3]];
                    for c in 0..3 {
                        let g = |dx: i64, dy: i64| {
                            let p = layer.at(x + dx, y + dy);
                            if p[3] > 0.0 { p[c] } else { here[c] }
                        };
                        let gx = g(1, -1) + 2.0 * g(1, 0) + g(1, 1) - g(-1, -1) - 2.0 * g(-1, 0) - g(-1, 1);
                        let gy = g(-1, 1) + 2.0 * g(0, 1) + g(1, 1) - g(-1, -1) - 2.0 * g(0, -1) - g(1, -1);
                        px[c] = (gx * gx + gy * gy).sqrt().min(1.0);
                    }
                    out.pixels[y as usize * w + x as usize] = px;
                }
            }
            out
        }
        DropShadow | InnerShadow | Glow | InnerGlow | SolidShadow => shadow_or_glow(layer, filter),
        InnerBevel | OuterBevel | InsetEmboss | RaisedEmboss => bevel_or_emboss(layer, filter),
    }
}

/// Averages each pixel with others found by `along`, which is given a number from 0 to 1 for each step.
fn streak(layer: &Layer, steps: usize, along: &dyn Fn(f32, f32, f32) -> (f32, f32)) -> Layer {
    let mut out = layer.clone();
    for y in 0..layer.height {
        for x in 0..layer.width {
            let mut sum = [0.0f32; 4];
            for step in 0..=steps {
                let (sx, sy) = along(x as f32, y as f32, step as f32 / steps as f32);
                let p = layer.sample(sx, sy);
                for c in 0..3 {
                    sum[c] += p[c] * p[3];
                }
                sum[3] += p[3];
            }
            let alpha = sum[3] / (steps + 1) as f32;
            out.pixels[y * layer.width + x] =
                if sum[3] > 1e-4 { [sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3], alpha.clamp(0.0, 1.0)] } else { [0.0; 4] };
        }
    }
    out
}

/// The middle of the part of the layer that has anything in it.
fn bounds_center(layer: &Layer) -> (f32, f32) {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for (index, _) in layer.pixels.iter().enumerate().filter(|(_, p)| p[3] > 0.0) {
        let (x, y) = (index % layer.width, index / layer.width);
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    if x0 == usize::MAX {
        return (layer.width as f32 / 2.0, layer.height as f32 / 2.0);
    }
    ((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0)
}

fn shadow_or_glow(layer: &Layer, filter: &LiveFilter) -> Layer {
    let (w, h) = (layer.width, layer.height);
    let v = |index: usize| filter.value(index);
    let color = filter.rgb();
    let alpha = layer.alpha();
    let shifted = |source: &[f32], dx: f32, dy: f32| -> Vec<f32> {
        let (ix, iy) = (dx.round() as i64, dy.round() as i64);
        (0..w * h)
            .map(|index| {
                let (x, y) = ((index % w) as i64 - ix, (index / w) as i64 - iy);
                if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 { 0.0 } else { source[y as usize * w + x as usize] }
            })
            .collect()
    };
    let mut out = layer.clone();
    match filter.kind {
        DropShadow => {
            let ((dx, dy), distance, opacity, knock_out) = (direction(v(3)), v(0), v(1) / 100.0, v(4) >= 0.5);
            let shade = soften(&shifted(&alpha, dx * distance, dy * distance), w, h, v(2));
            for (px, s) in out.pixels.iter_mut().zip(&shade) {
                let shadow = [color[0], color[1], color[2], (s * opacity).clamp(0.0, 1.0)];
                // Knocked out, the object leaves a hole in its own shadow.
                *px = if knock_out { [shadow[0], shadow[1], shadow[2], shadow[3] * (1.0 - px[3])] } else { over(*px, shadow) };
            }
        }
        SolidShadow => {
            let ((dx, dy), distance, solid) = (direction(v(0)), v(1), v(2) >= 0.5);
            // The object is stepped back along the angle, each copy behind the last.
            let steps = distance.ceil() as usize;
            for step in (1..=steps).rev() {
                let far = shifted(&alpha, dx * step as f32, dy * step as f32);
                let fade = if solid { 1.0 } else { 1.0 - step as f32 / (steps + 1) as f32 };
                for (px, a) in out.pixels.iter_mut().zip(&far) {
                    *px = over(*px, [color[0], color[1], color[2], a * fade]);
                }
            }
            // The copies went on top in turn, so the object itself is laid back over them.
            for (px, original) in out.pixels.iter_mut().zip(&layer.pixels) {
                *px = over(*original, *px);
            }
        }
        Glow => {
            let (width, opacity, softness, offset) = (v(0), v(1) / 100.0, v(2), v(3));
            // The outline is widened by the width, then pushed out by the offset and softened.
            let spread: Vec<f32> = blur_channel(&alpha, w, h, (width + offset).max(0.5) * 2.0).iter().map(|a| (a * 4.0).min(1.0)).collect();
            let halo = blur_channel(&spread, w, h, softness);
            let hole: Vec<f32> = if offset > 0.0 {
                blur_channel(&alpha, w, h, offset * 2.0).iter().map(|a| (a * 4.0).min(1.0)).collect()
            } else {
                vec![0.0; alpha.len()]
            };
            for ((px, s), inner) in out.pixels.iter_mut().zip(&halo).zip(&hole) {
                *px = over(*px, [color[0], color[1], color[2], (s * (1.0 - inner) * opacity).clamp(0.0, 1.0)]);
            }
        }
        InnerGlow => {
            let (width, opacity, softness) = (v(0), v(1) / 100.0, v(2));
            // What is not the object, grown inward and softened, shows only on the object.
            let outside: Vec<f32> = alpha.iter().map(|a| 1.0 - a).collect();
            // Half of the blurred outside reaches the edge, and it is gone about a width further in.
            let grown: Vec<f32> = blur_channel(&outside, w, h, width.max(0.5)).iter().map(|a| (a * 2.0).min(1.0)).collect();
            let edge = blur_channel(&grown, w, h, softness);
            for (px, e) in out.pixels.iter_mut().zip(&edge) {
                let alpha = px[3];
                let glowing = over([color[0], color[1], color[2], (e * opacity).clamp(0.0, 1.0)], [px[0], px[1], px[2], 1.0]);
                *px = [glowing[0], glowing[1], glowing[2], alpha];
            }
        }
        _ => {
            let ((dx, dy), distance, opacity, knock_out) = (direction(v(3)), v(0), v(1) / 100.0, v(4) >= 0.5);
            // The shadow falls on the object from outside: its shifted outside, softened.
            let outside: Vec<f32> = alpha.iter().map(|a| 1.0 - a).collect();
            let mut far = shifted(&outside, dx * distance, dy * distance);
            // What was shifted in from beyond the layer is outside too.
            for (index, value) in far.iter_mut().enumerate() {
                let (x, y) = ((index % w) as f32 - dx * distance, (index / w) as f32 - dy * distance);
                if x < 0.0 || y < 0.0 || x >= w as f32 || y >= h as f32 {
                    *value = 1.0;
                }
            }
            let shade = soften(&far, w, h, v(2));
            for (px, s) in out.pixels.iter_mut().zip(&shade) {
                let alpha = px[3];
                let strength = (s * opacity).clamp(0.0, 1.0);
                *px = if knock_out {
                    [color[0], color[1], color[2], alpha * strength]
                } else {
                    let shaded = over([color[0], color[1], color[2], strength], [px[0], px[1], px[2], 1.0]);
                    [shaded[0], shaded[1], shaded[2], alpha]
                };
            }
        }
    }
    out
}

fn bevel_or_emboss(layer: &Layer, filter: &LiveFilter) -> Layer {
    let (w, h) = (layer.width, layer.height);
    let v = |index: usize| filter.value(index);
    let alpha = layer.alpha();
    let mut out = layer.clone();
    // Light where the surface faces the light, dark where it faces away, as strong as the contrast.
    let shade = |px: [f32; 4], light: f32, contrast: f32| -> [f32; 4] {
        let amount = (light * contrast).clamp(-1.0, 1.0);
        let toward = if amount >= 0.0 { 1.0 } else { 0.0 };
        let mix = amount.abs();
        [px[0] + (toward - px[0]) * mix, px[1] + (toward - px[1]) * mix, px[2] + (toward - px[2]) * mix, px[3]]
    };
    match filter.kind {
        InnerBevel => {
            let heights = bevel_height(&alpha, w, h, v(1), v(3), v(0).round() as usize, true);
            let light = lighting(&heights, w, h, v(4));
            let strength = v(2) / 100.0 * v(1).max(1.0) * 1.5;
            for (px, l) in out.pixels.iter_mut().zip(&light) {
                if px[3] > 0.0 {
                    *px = shade(*px, *l, strength);
                }
            }
        }
        OuterBevel => {
            let color = filter.rgb();
            let heights = bevel_height(&alpha, w, h, v(1), v(3), v(0).round() as usize, false);
            let light = lighting(&heights, w, h, v(4));
            let strength = v(2) / 100.0 * v(1).max(1.0) * 1.5;
            // The bevel is a rim of the chosen color around the object, lit like a slope.
            let rim: Vec<f32> = blur_channel(&alpha, w, h, v(1).max(0.5)).iter().map(|a| (a * 4.0).min(1.0)).collect();
            for ((px, l), r) in out.pixels.iter_mut().zip(&light).zip(&rim) {
                let edge = shade([color[0], color[1], color[2], *r], *l, strength);
                *px = over(*px, edge);
            }
        }
        _ => {
            let raised = filter.kind == RaisedEmboss;
            let (width, contrast, softness, angle, show) = (v(0), v(1) / 100.0, v(2), v(3), v(4) >= 0.5);
            // An emboss is the object's edge lit on one side and shaded on the other, on what is around it.
            let soft = blur_channel(&alpha, w, h, softness.max(0.3));
            let (dx, dy) = direction(angle);
            let reach = width.max(0.5);
            let sample = |x: f32, y: f32| -> f32 {
                let (ix, iy) = (x.round() as i64, y.round() as i64);
                if ix < 0 || iy < 0 || ix >= w as i64 || iy >= h as i64 { 0.0 } else { soft[iy as usize * w + ix as usize] }
            };
            for y in 0..h {
                for x in 0..w {
                    let (fx, fy) = (x as f32, y as f32);
                    let relief = sample(fx - dx * reach, fy - dy * reach) - sample(fx + dx * reach, fy + dy * reach);
                    let relief = if raised { relief } else { -relief };
                    let strength = (relief.abs() * contrast).clamp(0.0, 1.0);
                    let edge = if relief >= 0.0 { [1.0, 1.0, 1.0, strength] } else { [0.0, 0.0, 0.0, strength] };
                    let px = &mut out.pixels[y * w + x];
                    *px = if show { over(*px, edge) } else { edge };
                }
            }
        }
    }
    out
}

fn to_hsl(c: [f32; 3]) -> (f32, f32, f32) {
    let (max, min) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    let light = (max + min) / 2.0;
    let delta = max - min;
    if delta < 1e-6 {
        return (0.0, 0.0, light);
    }
    let sat = delta / (1.0 - (2.0 * light - 1.0).abs()).max(1e-6);
    let hue = if max == c[0] {
        60.0 * ((c[1] - c[2]) / delta).rem_euclid(6.0)
    } else if max == c[1] {
        60.0 * ((c[2] - c[0]) / delta + 2.0)
    } else {
        60.0 * ((c[0] - c[1]) / delta + 4.0)
    };
    (hue, sat.clamp(0.0, 1.0), light)
}

fn from_hsl(hue: f32, sat: f32, light: f32) -> [f32; 3] {
    let chroma = (1.0 - (2.0 * light - 1.0).abs()) * sat;
    let sector = hue.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let lift = light - chroma / 2.0;
    [r + lift, g + lift, b + lift]
}

/// How far, in pixels, a filter can paint beyond the object it is on.
pub fn reach(filter: &LiveFilter) -> f32 {
    let v = |index: usize| filter.value(index);
    match filter.kind {
        DropShadow => v(0) + v(2) * 2.0,
        SolidShadow => v(1),
        Glow => (v(0) + v(3)) * 3.0 + v(2) * 2.0,
        OuterBevel => v(1) * 3.0 + v(3) * 2.0,
        PhotoshopEffects => crate::pseffects::reach(filter),
        InsetEmboss | RaisedEmboss => v(0) + v(2) * 2.0,
        GaussianBlur => v(0) * 2.0,
        Blur | BlurMore => 4.0,
        MotionBlur => v(1),
        RadialBlur | ZoomBlur => 8.0,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A layer with a square of one color in its middle.
    fn square(size: usize, margin: usize, color: [f32; 3]) -> Layer {
        let mut pixels = vec![[0.0; 4]; size * size];
        for y in margin..size - margin {
            for x in margin..size - margin {
                pixels[y * size + x] = [color[0], color[1], color[2], 1.0];
            }
        }
        Layer { width: size, height: size, pixels }
    }

    fn with(kind: FilterKind, values: &[f32]) -> LiveFilter {
        let mut filter = LiveFilter::new(kind);
        for (index, value) in values.iter().enumerate() {
            filter.set_value(index, *value);
        }
        filter
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.02
    }

    #[test]
    fn every_filter_keeps_its_pixels_in_range() {
        let layer = square(40, 10, [0.8, 0.4, 0.1]);
        for (_, kinds) in MENU {
            for kind in kinds.iter() {
                let out = apply(&layer, &LiveFilter::new(*kind));
                assert_eq!(out.pixels.len(), layer.pixels.len(), "{kind:?}");
                for p in &out.pixels {
                    assert!(p.iter().all(|c| c.is_finite() && (0.0..=1.0).contains(c)), "{kind:?} gave {p:?}");
                }
                assert!(!kind.label().is_empty());
                // Every setting starts inside its own range.
                for setting in kind.settings() {
                    assert!(setting.min <= setting.start && setting.start <= setting.max, "{kind:?} {}", setting.name);
                }
            }
        }
        assert_eq!(MENU.iter().map(|(_, kinds)| kinds.len()).sum::<usize>(), 29);
    }

    #[test]
    fn the_color_filters_do_what_their_names_say() {
        let layer = square(8, 2, [0.8, 0.4, 0.1]);
        let middle = |l: &Layer| l.pixels[4 * 8 + 4];
        let inverted = middle(&apply(&layer, &LiveFilter::new(Invert)));
        assert!(near(inverted[0], 0.2) && near(inverted[1], 0.6) && near(inverted[2], 0.9) && inverted[3] == 1.0);
        let brighter = middle(&apply(&layer, &with(BrightnessContrast, &[20.0, 0.0])));
        assert!(near(brighter[0], 1.0) && near(brighter[1], 0.6) && near(brighter[2], 0.3));
        // Full contrast pushes each channel to black or white, around mid gray.
        let stark = middle(&apply(&layer, &with(BrightnessContrast, &[0.0, 100.0])));
        assert_eq!((stark[0], stark[1], stark[2]), (1.0, 0.0, 0.0));
        // A color fill at full opacity is that color, and at half is half way.
        assert!(near(middle(&apply(&layer, &LiveFilter::new(ColorFill)))[1], 0.0));
        assert!(near(middle(&apply(&layer, &with(ColorFill, &[50.0])))[1], 0.2));
        // No saturation is gray, and turning the hue half way round turns orange to blue.
        let gray = middle(&apply(&layer, &with(HueSaturation, &[0.0, -100.0])));
        assert!(near(gray[0], gray[1]) && near(gray[1], gray[2]));
        let turned = middle(&apply(&layer, &with(HueSaturation, &[180.0])));
        assert!(turned[2] > turned[0] && turned[2] > turned[1], "{turned:?}");
        // Levels that take mid gray as white clip the red and double the blue.
        let levelled = middle(&apply(&layer, &with(Levels, &[0.0, 1.0, 128.0])));
        assert!(near(levelled[0], 1.0) && near(levelled[2], 0.2), "{levelled:?}");
        // The clear pixels around the square stay clear.
        assert_eq!(apply(&layer, &LiveFilter::new(Invert)).pixels[0], [0.0; 4]);
        // Lighter is clearer when a picture is turned into its own transparency.
        let white = square(8, 2, [1.0, 1.0, 1.0]);
        assert!(middle(&apply(&white, &LiveFilter::new(ConvertToAlpha)))[3] < 0.01);
    }

    #[test]
    fn a_blur_spreads_the_object_without_losing_any_of_it() {
        let layer = square(60, 20, [1.0, 0.0, 0.0]);
        let blurred = apply(&layer, &with(GaussianBlur, &[6.0]));
        let total = |l: &Layer| l.pixels.iter().map(|p| p[3]).sum::<f32>();
        assert!((total(&blurred) - total(&layer)).abs() < total(&layer) * 0.01);
        assert!(blurred.pixels[20 * 60 + 17][3] > 0.0, "it reaches past the old edge");
        assert!(blurred.pixels[30 * 60 + 30][3] > 0.99, "the middle is still solid");
        assert!(near(blurred.pixels[20 * 60 + 17][0], 1.0), "and is still red where it spread");
    }

    #[test]
    fn a_drop_shadow_falls_where_the_angle_points() {
        let layer = square(60, 20, [1.0, 0.0, 0.0]);
        // The default angle, 315 degrees, throws the shadow down and to the right.
        let shadowed = apply(&layer, &with(DropShadow, &[8.0, 100.0, 0.0]));
        let at = |l: &Layer, x: usize, y: usize| l.pixels[y * 60 + x];
        assert!(at(&shadowed, 43, 43)[3] > 0.9 && at(&shadowed, 43, 43)[0] < 0.01, "black shadow below right");
        assert!(at(&shadowed, 16, 16)[3] < 0.01, "nothing above left");
        assert_eq!(at(&shadowed, 30, 30), [1.0, 0.0, 0.0, 1.0], "the object is on top of its shadow");
        // Knocked out, only the shadow is left, with a hole where the object was.
        let hollow = apply(&layer, &with(DropShadow, &[8.0, 100.0, 0.0, 315.0, 1.0]));
        assert!(at(&hollow, 30, 30)[3] < 0.01 && at(&hollow, 43, 43)[3] > 0.9);
        // An inner shadow darkens the object on the side the shadow comes from and nothing outside it.
        let inner = apply(&layer, &with(InnerShadow, &[6.0, 100.0, 0.0]));
        assert!(at(&inner, 22, 22)[0] < 0.1 && at(&inner, 36, 36)[0] > 0.9, "{:?} {:?}", at(&inner, 22, 22), at(&inner, 36, 36));
        assert!(at(&inner, 43, 43)[3] < 0.01);
    }

    #[test]
    fn a_glow_surrounds_the_object_and_an_inner_glow_stays_inside() {
        let layer = square(60, 20, [0.0, 0.0, 1.0]);
        let at = |l: &Layer, x: usize, y: usize| l.pixels[y * 60 + x];
        let glowing = apply(&layer, &with(Glow, &[4.0, 100.0, 2.0]));
        for (x, y) in [(17, 30), (42, 30), (30, 17), (30, 42)] {
            assert!(at(&glowing, x, y)[3] > 0.2 && at(&glowing, x, y)[0] > 0.9, "red glow at {x},{y}: {:?}", at(&glowing, x, y));
        }
        assert_eq!(at(&glowing, 30, 30), [0.0, 0.0, 1.0, 1.0]);
        assert!(at(&glowing, 2, 2)[3] < 0.01);
        let inner = apply(&layer, &with(InnerGlow, &[4.0, 100.0, 2.0]));
        assert!(at(&inner, 21, 30)[0] > 0.3, "red just inside the edge");
        assert!(at(&inner, 30, 30)[0] < 0.05, "none in the middle");
        assert!(at(&inner, 17, 30)[3] < 0.01, "and none outside");
    }

    #[test]
    fn a_bevel_lights_the_side_that_faces_the_light() {
        let layer = square(60, 15, [0.5, 0.5, 0.5]);
        let at = |l: &Layer, x: usize, y: usize| l.pixels[y * 60 + x][0];
        // Light from the upper left, at 135 degrees: the left and top edges face it.
        let bevelled = apply(&layer, &with(InnerBevel, &[1.0, 6.0, 100.0, 1.0, 135.0]));
        assert!(at(&bevelled, 17, 30) > 0.6, "the left edge is lit: {}", at(&bevelled, 17, 30));
        assert!(at(&bevelled, 30, 17) > 0.6, "the top edge is lit: {}", at(&bevelled, 30, 17));
        assert!(at(&bevelled, 42, 30) < 0.4, "the right edge is in shade: {}", at(&bevelled, 42, 30));
        assert!(at(&bevelled, 30, 42) < 0.4, "the bottom edge is in shade: {}", at(&bevelled, 30, 42));
        assert!(near(at(&bevelled, 30, 30), 0.5), "the flat top is untouched");
        assert_eq!(bevelled.pixels[0], [0.0; 4]);
    }

    #[test]
    fn levels_and_curves_have_numbers_for_each_channel() {
        let layer = square(8, 2, [0.5, 0.5, 0.5]);
        let middle = |l: &Layer| l.pixels[4 * 8 + 4];
        // Mid gray taken as white, on the red channel only.
        let mut levels = LiveFilter::new(Levels);
        levels.set_value(levels_start(1) + 2, 128.0);
        let red = middle(&apply(&layer, &levels));
        assert!(near(red[0], 1.0) && near(red[1], 0.5) && near(red[2], 0.5), "{red:?}");
        // The shared numbers still work on all three, after each channel's own.
        levels.set_value(levels_start(0) + 4, 128.0);
        let both = middle(&apply(&layer, &levels));
        assert!(near(both[0], 0.5) && near(both[1], 0.25), "{both:?}");
        assert_eq!((levels_start(0), levels_start(1), levels_start(3) + 5), (0, 6, Levels.settings().len()));
        // A new curve is a straight line and changes nothing.
        let mut curve = LiveFilter::new(Curves);
        assert!(near(middle(&apply(&layer, &curve))[0], 0.5));
        let table = curve.curve_table(0);
        assert!((0..256).all(|level| (table[level] - level as f32).abs() < 0.01));
        // Lifting its middle lightens the mid tones and leaves black and white where they are.
        curve.set_curve(0, vec![(0.0, 0.0), (128.0, 192.0), (255.0, 255.0)]);
        let table = curve.curve_table(0);
        assert!(near(table[128] / 255.0, 0.75) && table[0] < 0.5 && table[255] > 254.5);
        assert!(table.windows(2).all(|pair| pair[1] >= pair[0] - 0.01), "it never turns back");
        assert!(near(middle(&apply(&layer, &curve))[0], 0.75));
        // Blue has a curve of its own, applied before the shared one.
        curve.set_curve(0, STRAIGHT.to_vec());
        curve.set_curve(3, vec![(0.0, 0.0), (128.0, 192.0), (255.0, 255.0)]);
        let blue = middle(&apply(&layer, &curve));
        assert!(near(blue[0], 0.5) && near(blue[2], 0.75), "{blue:?}");
        // Points given out of order, or twice, still make a curve.
        curve.set_curve(0, vec![(255.0, 0.0), (0.0, 255.0), (0.0, 255.0)]);
        assert!(curve.curve_table(0)[0] > 254.0 && curve.curve_table(0)[255] < 1.0);
        // A file written before curves had channels, or with none, reads as straight lines.
        curve.curves.clear();
        assert_eq!(curve.curve(2), STRAIGHT.to_vec());
    }

    #[test]
    fn curve_points_are_added_moved_and_dragged_away() {
        // A press above the middle of the line puts a point there and holds it.
        let (points, held) = curve_pressed(STRAIGHT.to_vec(), (128.2, 191.7), 7.0);
        assert_eq!((points.clone(), held), (vec![(0.0, 0.0), (128.0, 192.0), (255.0, 255.0)], 1));
        // Dragged, it follows the mouse but cannot pass its neighbors.
        let (points, kept) = curve_dragged(points, 1, (100.0, 100.0), 24.0);
        assert_eq!((points[1], kept), ((100.0, 100.0), true));
        let (points, _) = curve_dragged(points, 1, (270.0, 100.0), 24.0);
        assert_eq!(points[1], (254.0, 100.0));
        // A press on a point takes hold of it without adding another.
        let (points, held) = curve_pressed(points, (250.0, 104.0), 7.0);
        assert_eq!((points.len(), held), (3, 1));
        // Dragged well out of the box, it is taken off the curve.
        let (points, kept) = curve_dragged(points, 1, (250.0, -40.0), 24.0);
        assert_eq!((points.clone(), kept), (STRAIGHT.to_vec(), false));
        // An end can be dragged anywhere, and stays on the curve.
        let (points, kept) = curve_dragged(points, 0, (30.0, -500.0), 24.0);
        assert_eq!((points.clone(), kept), (vec![(30.0, 0.0), (255.0, 255.0)], true));
        // A new point never shares an input level with another.
        let (points, held) = curve_pressed(points, (30.0, 90.0), 7.0);
        assert_eq!((points[1], held), ((31.0, 90.0), 1));
        let (points, held) = curve_pressed(points, (255.0, 20.0), 7.0);
        assert_eq!((points[2], held, points.len()), ((254.0, 20.0), 2, 4));
    }

    #[test]
    fn levels_sliders_keep_black_below_white_and_gamma_in_between() {
        let start = [0.0, 1.0, 255.0, 0.0, 255.0];
        assert_eq!(levels_dragged(start, 0, 40.2), [40.0, 1.0, 255.0, 0.0, 255.0]);
        // Black cannot reach white, nor white black.
        assert_eq!(levels_dragged(start, 0, 400.0)[0], 253.0);
        assert_eq!(levels_dragged([100.0, 1.0, 255.0, 0.0, 255.0], 2, 20.0)[2], 102.0);
        // The middle slider half way between them is a gamma of 1; a quarter of the way, 2.
        assert_eq!(levels_dragged(start, 1, 127.5)[1], 1.0);
        assert_eq!(levels_dragged(start, 1, 63.75)[1], 2.0);
        assert_eq!(levels_dragged([100.0, 1.0, 200.0, 0.0, 255.0], 1, 175.0)[1], 0.42);
        assert!((gamma_place(1.0) - 0.5).abs() < 1e-6 && (gamma_place(2.0) - 0.25).abs() < 1e-6);
        // The ones for the way out go anywhere, and may cross.
        assert_eq!(levels_dragged(start, 3, 300.0)[3], 255.0);
        assert_eq!(levels_dragged(start, 4, 12.0)[4], 12.0);
    }

    #[test]
    fn eyedroppers_set_black_gray_and_white_from_a_picked_color() {
        let layer = square(8, 2, [40.0 / 255.0, 80.0 / 255.0, 120.0 / 255.0]);
        let middle = |l: &Layer| l.pixels[4 * 8 + 4];
        for kind in [Levels, Curves] {
            // Picked as black, the color comes out black; as white, white.
            let mut filter = LiveFilter::new(kind);
            filter.pick_point(0, [40.0, 80.0, 120.0]);
            let black = middle(&apply(&layer, &filter));
            assert!(black[0] < 0.01 && black[1] < 0.01 && black[2] < 0.01, "{kind:?} {black:?}");
            let mut filter = LiveFilter::new(kind);
            filter.pick_point(2, [40.0, 80.0, 120.0]);
            let white = middle(&apply(&layer, &filter));
            assert!(white[0] > 0.99 && white[1] > 0.99 && white[2] > 0.99, "{kind:?} {white:?}");
            // Picked as gray, it loses its tint and keeps its lightness.
            let mut filter = LiveFilter::new(kind);
            filter.pick_point(1, [40.0, 80.0, 120.0]);
            let gray = middle(&apply(&layer, &filter));
            assert!(near(gray[0], 80.0 / 255.0) && near(gray[1], 80.0 / 255.0) && near(gray[2], 80.0 / 255.0), "{kind:?} {gray:?}");
        }
        // On a curve, picking twice does not pile points up.
        let mut curves = LiveFilter::new(Curves);
        curves.pick_point(1, [40.0, 80.0, 120.0]);
        curves.pick_point(1, [40.0, 90.0, 120.0]);
        assert_eq!(curves.curve(1).len(), 3);
        assert_eq!(curves.curve(0), STRAIGHT.to_vec());
    }

    #[test]
    fn an_edited_filter_takes_its_place_among_the_others() {
        let before = vec![LiveFilter::new(Blur), LiveFilter::new(Invert)];
        let edited = LiveFilter::new(Levels);
        assert_eq!(placed(&before, 1, edited.clone()).iter().map(|f| f.kind).collect::<Vec<_>>(), [Blur, Levels]);
        assert_eq!(placed(&before, 2, edited).iter().map(|f| f.kind).collect::<Vec<_>>(), [Blur, Invert, Levels]);
    }

    #[test]
    fn auto_stretches_each_color_between_its_darkest_and_lightest() {
        let mut counts = [[0u32; 256]; 3];
        (counts[0][40], counts[0][210], counts[1][7]) = (5, 9, 3);
        // Red has a range and takes its ends; green has one level and blue none, so they stay.
        let mut levels = LiveFilter::new(Levels);
        levels.stretch(&counts);
        let red = levels_start(1);
        assert_eq!((levels.value(red), levels.value(red + 2)), (40.0, 210.0));
        assert_eq!((levels.value(levels_start(2)), levels.value(levels_start(2) + 2)), (0.0, 255.0));
        assert_eq!((levels.value(0), levels.value(2)), (0.0, 255.0), "the shared numbers are left alone");
        let mut curves = LiveFilter::new(Curves);
        curves.stretch(&counts);
        assert_eq!(curves.curve(1), vec![(40.0, 0.0), (210.0, 255.0)]);
        assert_eq!((curves.curve(0), curves.curve(2)), (STRAIGHT.to_vec(), STRAIGHT.to_vec()));
    }

    #[test]
    fn settings_stay_in_range_and_old_files_read() {
        let mut filter = LiveFilter::new(GaussianBlur);
        filter.set_value(0, 9999.0);
        assert_eq!(filter.value(0), 250.0);
        filter.set_value(0, f32::NAN);
        assert_eq!(filter.value(0), 4.0);
        // A filter saved with fewer values than it has now reads the rest at their start.
        let short: LiveFilter = serde_json::from_str(r#"{"kind":"DropShadow","values":[12.0]}"#).unwrap();
        assert_eq!((short.value(0), short.value(1), short.value(3)), (12.0, 65.0, 315.0));
        assert!(short.enabled);
        assert!(reach(&short) > 12.0);
    }
}
