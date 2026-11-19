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

//! Blend modes: how an object's colors mix with what is under it.
//!
//! The list and its order follow Fireworks. A mode takes the color already on the
//! page (the base) and the object's color (the top) and gives the color shown where
//! both are opaque. [`composite`] then weighs that by how transparent each one is.

use serde::{Deserialize, Serialize};
use tiny_skia::{Pixmap, PremultipliedColorU8};

use crate::i18n::t;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    Dissolve,
    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    Overlay,
    SoftLight,
    HardLight,
    VividLight,
    LinearLight,
    PinLight,
    HardMix,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Average,
    Negation,
    InverseColorBurn,
    SoftBurn,
    InverseColorDodge,
    SoftDodge,
    FuzzyLight,
    Reflect,
    Glow,
    Freeze,
    Heat,
    Additive,
    Subtractive,
    Interpolation,
    Stamp,
    Xor,
    Red,
    Green,
    Blue,
    Invert,
    Tint,
    Erase,
}

use BlendMode::*;

impl BlendMode {
    pub const ALL: [BlendMode; 45] = [
        Normal, Dissolve, Darken, Multiply, ColorBurn, LinearBurn, Lighten, Screen, ColorDodge, LinearDodge, Overlay, SoftLight, HardLight,
        VividLight, LinearLight, PinLight, HardMix, Difference, Exclusion, Hue, Saturation, Color, Luminosity, Average, Negation,
        InverseColorBurn, SoftBurn, InverseColorDodge, SoftDodge, FuzzyLight, Reflect, Glow, Freeze, Heat, Additive, Subtractive,
        Interpolation, Stamp, Xor, Red, Green, Blue, Invert, Tint, Erase,
    ];

    pub fn is_normal(&self) -> bool {
        *self == Normal
    }

    pub fn label(self) -> &'static str {
        match self {
            Normal => t("Normal"),
            Dissolve => t("Dissolve"),
            Darken => t("Darken"),
            Multiply => t("Multiply"),
            ColorBurn => t("Color Burn"),
            LinearBurn => t("Linear Burn"),
            Lighten => t("Lighten"),
            Screen => t("Screen"),
            ColorDodge => t("Color Dodge"),
            LinearDodge => t("Linear Dodge"),
            Overlay => t("Overlay"),
            SoftLight => t("Soft Light"),
            HardLight => t("Hard Light"),
            VividLight => t("Vivid Light"),
            LinearLight => t("Linear Light"),
            PinLight => t("Pin Light"),
            HardMix => t("Hard Mix"),
            Difference => t("Difference"),
            Exclusion => t("Exclusion"),
            Hue => t("Hue"),
            Saturation => t("Saturation"),
            Color => t("Color"),
            Luminosity => t("Luminosity"),
            Average => t("Average"),
            Negation => t("Negation"),
            InverseColorBurn => t("Inverse Color Burn"),
            SoftBurn => t("Soft Burn"),
            InverseColorDodge => t("Inverse Color Dodge"),
            SoftDodge => t("Soft Dodge"),
            FuzzyLight => t("Fuzzy Light"),
            Reflect => t("Reflect"),
            Glow => t("Glow"),
            Freeze => t("Freeze"),
            Heat => t("Heat"),
            Additive => t("Additive"),
            Subtractive => t("Subtractive"),
            Interpolation => t("Interpolation"),
            Stamp => t("Stamp"),
            Xor => t("XOR"),
            Red => t("Red"),
            Green => t("Green"),
            Blue => t("Blue"),
            Invert => t("Invert"),
            Tint => t("Tint"),
            Erase => t("Erase"),
        }
    }

    /// The color shown where the base and the top are both opaque. Channels run from 0 to 1.
    pub fn mix(self, base: [f32; 3], top: [f32; 3]) -> [f32; 3] {
        let each = |f: fn(f32, f32) -> f32| [f(base[0], top[0]), f(base[1], top[1]), f(base[2], top[2])];
        match self {
            Normal | Dissolve | Erase => top,
            Darken => each(f32::min),
            Multiply => each(|b, s| b * s),
            ColorBurn => each(color_burn),
            LinearBurn | Subtractive => each(|b, s| (b + s - 1.0).max(0.0)),
            Lighten => each(f32::max),
            Screen => each(|b, s| b + s - b * s),
            ColorDodge => each(color_dodge),
            LinearDodge | Additive => each(|b, s| (b + s).min(1.0)),
            Overlay => each(|b, s| hard_light(s, b)),
            SoftLight => each(soft_light),
            HardLight => each(hard_light),
            VividLight => each(|b, s| if s <= 0.5 { color_burn(b, 2.0 * s) } else { color_dodge(b, 2.0 * s - 1.0) }),
            LinearLight => each(|b, s| (b + 2.0 * s - 1.0).clamp(0.0, 1.0)),
            PinLight => each(|b, s| if s <= 0.5 { b.min(2.0 * s) } else { b.max(2.0 * s - 1.0) }),
            HardMix => each(|b, s| if b + s >= 1.0 { 1.0 } else { 0.0 }),
            Difference => each(|b, s| (b - s).abs()),
            Exclusion => each(|b, s| b + s - 2.0 * b * s),
            Hue => set_luminosity(set_saturation(top, saturation(base)), luminosity(base)),
            Saturation => set_luminosity(set_saturation(base, saturation(top)), luminosity(base)),
            Color => set_luminosity(top, luminosity(base)),
            Luminosity => set_luminosity(base, luminosity(top)),
            Average => each(|b, s| (b + s) / 2.0),
            Negation => each(|b, s| 1.0 - (1.0 - b - s).abs()),
            InverseColorBurn => each(|b, s| color_burn(s, b)),
            SoftBurn => each(soft_burn),
            InverseColorDodge => each(|b, s| color_dodge(s, b)),
            SoftDodge => each(|b, s| soft_burn(s, b)),
            // The gentler soft light that squares the base instead of taking its root.
            FuzzyLight => each(|b, s| ((1.0 - 2.0 * s) * b * b + 2.0 * b * s).clamp(0.0, 1.0)),
            Reflect => each(reflect),
            Glow => each(|b, s| reflect(s, b)),
            Freeze => each(freeze),
            Heat => each(|b, s| freeze(s, b)),
            Interpolation => each(|b, s| 0.5 - 0.25 * (std::f32::consts::PI * b).cos() - 0.25 * (std::f32::consts::PI * s).cos()),
            Stamp => each(|b, s| (b + 2.0 * s - 1.0).clamp(0.0, 1.0)),
            Xor => each(|b, s| ((b * 255.0).round() as u8 ^ (s * 255.0).round() as u8) as f32 / 255.0),
            Red => [top[0], base[1], base[2]],
            Green => [base[0], top[1], base[2]],
            Blue => [base[0], base[1], top[2]],
            // These two only look at what is underneath: the object is a stencil.
            Invert => [1.0 - base[0], 1.0 - base[1], 1.0 - base[2]],
            Tint => [(base[0] + 0.5) / 2.0, (base[1] + 0.5) / 2.0, (base[2] + 0.5) / 2.0],
        }
    }
}

fn color_dodge(b: f32, s: f32) -> f32 {
    if b <= 0.0 {
        0.0
    } else if s >= 1.0 {
        1.0
    } else {
        (b / (1.0 - s)).min(1.0)
    }
}

fn color_burn(b: f32, s: f32) -> f32 {
    if b >= 1.0 {
        1.0
    } else if s <= 0.0 {
        0.0
    } else {
        1.0 - ((1.0 - b) / s).min(1.0)
    }
}

fn hard_light(b: f32, s: f32) -> f32 {
    if s <= 0.5 { 2.0 * b * s } else { 1.0 - 2.0 * (1.0 - b) * (1.0 - s) }
}

fn soft_light(b: f32, s: f32) -> f32 {
    if s <= 0.5 {
        b - (1.0 - 2.0 * s) * b * (1.0 - b)
    } else {
        let d = if b <= 0.25 { ((16.0 * b - 12.0) * b + 4.0) * b } else { b.sqrt() };
        b + (2.0 * s - 1.0) * (d - b)
    }
}

fn soft_burn(b: f32, s: f32) -> f32 {
    if b + s < 1.0 {
        if b >= 1.0 { 1.0 } else { (s / (2.0 * (1.0 - b))).min(1.0) }
    } else if s <= 0.0 {
        0.0
    } else {
        (1.0 - (1.0 - b) / (2.0 * s)).max(0.0)
    }
}

fn reflect(b: f32, s: f32) -> f32 {
    if s >= 1.0 { 1.0 } else { (b * b / (1.0 - s)).min(1.0) }
}

fn freeze(b: f32, s: f32) -> f32 {
    if s <= 0.0 { 0.0 } else { (1.0 - (1.0 - b) * (1.0 - b) / s).max(0.0) }
}

fn luminosity(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn saturation(c: [f32; 3]) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

/// Moves a color to the given brightness, pulling it back into range without changing its hue.
fn set_luminosity(c: [f32; 3], target: f32) -> [f32; 3] {
    let shift = target - luminosity(c);
    let mut out = [c[0] + shift, c[1] + shift, c[2] + shift];
    let light = luminosity(out);
    let (low, high) = (out[0].min(out[1]).min(out[2]), out[0].max(out[1]).max(out[2]));
    if low < 0.0 {
        out = out.map(|v| light + (v - light) * light / (light - low));
    }
    if high > 1.0 {
        out = out.map(|v| light + (v - light) * (1.0 - light) / (high - light));
    }
    // The scaling above lands a hair outside the range now and then.
    out.map(|v| v.clamp(0.0, 1.0))
}

/// Stretches or squeezes a color's channels so they span `target`, keeping their order.
fn set_saturation(c: [f32; 3], target: f32) -> [f32; 3] {
    let (low, high) = (c[0].min(c[1]).min(c[2]), c[0].max(c[1]).max(c[2]));
    if high <= low {
        return [0.0; 3];
    }
    c.map(|v| (v - low) * target / (high - low))
}

/// A number from 0 to 1 that is always the same for the same pixel, for Dissolve.
fn speckle(x: u32, y: u32) -> f32 {
    let mut n = x.wrapping_mul(374_761_393).wrapping_add(y.wrapping_mul(668_265_263));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n ^ (n >> 16)) & 0xffff) as f32 / 65535.0
}

/// Lays `layer`, which holds one object drawn on nothing, onto `target` with a blend mode.
pub fn composite(target: &mut Pixmap, layer: &Pixmap, mode: BlendMode) {
    let width = target.width();
    for (index, (under, over)) in target.pixels_mut().iter_mut().zip(layer.pixels()).enumerate() {
        let mut top_alpha = over.alpha() as f32 / 255.0;
        if top_alpha <= 0.0 {
            continue;
        }
        let base_alpha = under.alpha() as f32 / 255.0;
        let straight = |c: u8, alpha: f32| if alpha > 0.0 { (c as f32 / 255.0 / alpha).min(1.0) } else { 0.0 };
        let base = [straight(under.red(), base_alpha), straight(under.green(), base_alpha), straight(under.blue(), base_alpha)];
        let top = [straight(over.red(), top_alpha), straight(over.green(), top_alpha), straight(over.blue(), top_alpha)];
        if mode == Erase {
            // The object rubs out what is under it and adds nothing of its own.
            let kept = base_alpha * (1.0 - top_alpha);
            *under = premultiplied(base, kept);
            continue;
        }
        if mode == Dissolve {
            // Each pixel is either wholly the object or wholly what was there.
            if speckle(index as u32 % width, index as u32 / width) >= top_alpha {
                continue;
            }
            top_alpha = 1.0;
        }
        let mixed = mode.mix(base, top);
        // Where there is nothing underneath, a stencil mode has nothing to change and shows nothing.
        let alone = if matches!(mode, Invert | Tint) { 0.0 } else { 1.0 };
        let alpha = top_alpha * (alone * (1.0 - base_alpha) + base_alpha) + base_alpha * (1.0 - top_alpha);
        if alpha <= 0.0 {
            continue;
        }
        let mut color = [0.0; 3];
        for channel in 0..3 {
            let over_base = top_alpha * base_alpha * mixed[channel];
            let over_nothing = alone * top_alpha * (1.0 - base_alpha) * top[channel];
            let base_only = base_alpha * (1.0 - top_alpha) * base[channel];
            color[channel] = (over_base + over_nothing + base_only) / alpha;
        }
        *under = premultiplied(color, alpha);
    }
}

fn premultiplied(color: [f32; 3], alpha: f32) -> PremultipliedColorU8 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    let channel = |v: f32| ((v.clamp(0.0, 1.0) * alpha.clamp(0.0, 1.0) * 255.0).round() as u8).min(a);
    PremultipliedColorU8::from_rgba(channel(color[0]), channel(color[1]), channel(color[2]), a).unwrap_or(PremultipliedColorU8::TRANSPARENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(r: u8, g: u8, b: u8, a: u8) -> Pixmap {
        let mut pixmap = Pixmap::new(2, 2).unwrap();
        pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
        pixmap
    }

    fn result(base: (u8, u8, u8, u8), top: (u8, u8, u8, u8), mode: BlendMode) -> (u8, u8, u8, u8) {
        let mut target = solid(base.0, base.1, base.2, base.3);
        composite(&mut target, &solid(top.0, top.1, top.2, top.3), mode);
        let px = target.pixels()[0].demultiply();
        (px.red(), px.green(), px.blue(), px.alpha())
    }

    #[test]
    fn the_everyday_modes_give_their_textbook_results() {
        let (gray, orange) = ((128, 128, 128, 255), (255, 128, 0, 255));
        assert_eq!(result(gray, orange, Normal), orange);
        assert_eq!(result(gray, orange, Multiply), (128, 64, 0, 255));
        assert_eq!(result(gray, orange, Screen), (255, 192, 128, 255));
        assert_eq!(result(gray, orange, Darken), (128, 128, 0, 255));
        assert_eq!(result(gray, orange, Lighten), (255, 128, 128, 255));
        assert_eq!(result(gray, orange, Difference), (127, 0, 128, 255));
        assert_eq!(result(gray, orange, Additive), (255, 255, 128, 255));
        assert_eq!(result(gray, orange, Average), (192, 128, 64, 255));
        assert_eq!(result(gray, orange, Invert), (127, 127, 127, 255));
        assert_eq!(result(gray, orange, Red), (255, 128, 128, 255));
        assert_eq!(result((0, 0, 0, 255), (255, 255, 255, 255), Multiply), (0, 0, 0, 255));
        assert_eq!(result((255, 255, 255, 255), (10, 200, 30, 255), Multiply), (10, 200, 30, 255));
    }

    #[test]
    fn color_modes_take_hue_from_one_side_and_brightness_from_the_other() {
        let (gray, red) = ((128, 128, 128, 255), (255, 0, 0, 255));
        // The gray's brightness with the red's color: a red as bright as the gray.
        let colored = result(gray, red, Color);
        assert!(colored.0 > colored.1 && colored.1 == colored.2, "{colored:?}");
        let brightness = 0.3 * colored.0 as f32 + 0.59 * colored.1 as f32 + 0.11 * colored.2 as f32;
        assert!((brightness - 128.0).abs() < 2.0, "{brightness}");
        // The red's brightness on a gray: a darker gray.
        let lit = result(gray, red, Luminosity);
        assert!(lit.0 == lit.1 && lit.1 == lit.2 && lit.0 < 128, "{lit:?}");
        // A gray has no hue or saturation to give.
        assert_eq!(result(red, gray, Saturation).0, result(red, gray, Saturation).1);
    }

    #[test]
    fn every_mode_stays_in_range_and_leaves_bare_canvas_and_empty_layers_alone() {
        let samples = [0.0, 0.2, 0.5, 0.8, 1.0];
        for mode in BlendMode::ALL {
            for b in samples {
                for s in samples {
                    let mixed = mode.mix([b, s, 1.0 - b], [s, b, 0.3]);
                    assert!(mixed.iter().all(|v| (0.0..=1.0).contains(v) && v.is_finite()), "{mode:?} gave {mixed:?} for {b} and {s}");
                }
            }
            // Nothing in the layer changes nothing.
            assert_eq!(result((40, 90, 200, 255), (0, 0, 0, 0), mode), (40, 90, 200, 255), "{mode:?}");
            assert!(!mode.label().is_empty());
        }
        // An object over empty canvas shows as itself, whatever its mode, unless the mode only changes what is under it.
        assert_eq!(result((0, 0, 0, 0), (10, 200, 30, 255), Multiply), (10, 200, 30, 255));
        assert_eq!(result((0, 0, 0, 0), (10, 200, 30, 255), Invert).3, 0);
        assert_eq!(result((0, 0, 0, 0), (10, 200, 30, 255), Erase).3, 0);
    }

    #[test]
    fn erase_cuts_a_hole_and_half_transparent_objects_blend_half() {
        assert_eq!(result((40, 90, 200, 255), (0, 0, 0, 255), Erase).3, 0);
        // The color underneath is kept, to within the rounding of half-transparent pixels.
        let rubbed = result((40, 90, 200, 255), (0, 0, 0, 128), Erase);
        assert!(rubbed.3 == 127 && (rubbed.0 as i32 - 40).abs() <= 1 && (rubbed.1 as i32 - 90).abs() <= 1 && (rubbed.2 as i32 - 200).abs() <= 1, "{rubbed:?}");
        // Half a black Multiply over white is half way to black.
        let half = result((255, 255, 255, 255), (0, 0, 0, 128), Multiply);
        assert!((half.0 as i32 - 127).abs() <= 1 && half.3 == 255, "{half:?}");
    }

    #[test]
    fn dissolve_shows_whole_pixels_in_proportion_to_opacity() {
        let mut target = Pixmap::new(40, 40).unwrap();
        target.fill(tiny_skia::Color::from_rgba8(255, 255, 255, 255));
        let mut layer = Pixmap::new(40, 40).unwrap();
        layer.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 64));
        composite(&mut target, &layer, Dissolve);
        let black = target.pixels().iter().filter(|p| p.red() == 0).count();
        let gray = target.pixels().iter().filter(|p| p.red() != 0 && p.red() != 255).count();
        assert_eq!(gray, 0, "no pixel is partly covered");
        assert!((300..500).contains(&black), "{black} of 1600 pixels, for a quarter opacity");
    }
}
