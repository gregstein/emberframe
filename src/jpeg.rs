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

//! A baseline JPEG encoder that can give part of the image a different quality.
//!
//! A JPEG file carries one set of quantization tables for the whole image, so a
//! region cannot simply be written with tables of its own. Selective quality is
//! done the usual way instead: the file gets the finer of the two tables, and the
//! blocks of the coarser region are rounded to what the coarser table would have
//! kept before they are written. Those blocks then hold mostly zeros and compress
//! as if the coarser table had been used, while every decoder reads the file as
//! an ordinary JPEG.
//!
//! The encoder is the plain sequential one of the standard: YCbCr without
//! subsampling, an 8x8 DCT and the Huffman tables the standard suggests.

/// Quantization steps for brightness at quality 50, in reading order.
const LUMA_STEPS: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56, 14, 17, 22, 29, 51, 87, 80, 62, 18, 22,
    37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

/// Quantization steps for the two color channels at quality 50.
const CHROMA_STEPS: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99, 47, 66, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

/// The order coefficients are written in, from the lowest frequency to the highest.
const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56,
    57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

/// A Huffman table as the file stores it: how many codes there are of each length, then the values in code order.
struct HuffmanSpec {
    counts: [u8; 16],
    values: &'static [u8],
}

const DC_LUMA: HuffmanSpec = HuffmanSpec { counts: [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0], values: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] };
const DC_CHROMA: HuffmanSpec = HuffmanSpec { counts: [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0], values: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] };
const AC_LUMA: HuffmanSpec = HuffmanSpec {
    counts: [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d],
    values: &[
        0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91,
        0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a,
        0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53,
        0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
        0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5,
        0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9,
        0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2,
        0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
    ],
};
const AC_CHROMA: HuffmanSpec = HuffmanSpec {
    counts: [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77],
    values: &[
        0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14,
        0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0, 0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17,
        0x18, 0x19, 0x1a, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a,
        0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78,
        0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3,
        0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
        0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf2,
        0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
    ],
};

/// The code and its length for every value of a Huffman table.
struct HuffmanCodes {
    code: [u16; 256],
    length: [u8; 256],
}

impl HuffmanCodes {
    fn new(spec: &HuffmanSpec) -> Self {
        let (mut code, mut length) = ([0u16; 256], [0u8; 256]);
        let (mut next, mut index) = (0u16, 0usize);
        for (bits, count) in spec.counts.iter().enumerate() {
            for _ in 0..*count {
                let value = spec.values[index] as usize;
                code[value] = next;
                length[value] = bits as u8 + 1;
                next += 1;
                index += 1;
            }
            next <<= 1;
        }
        Self { code, length }
    }
}

/// The quantization table for a quality from 1 to 100, by the scaling every common encoder uses.
fn steps_for(base: &[u16; 64], quality: u8) -> [u16; 64] {
    let quality = quality.clamp(1, 100) as u32;
    let scale = if quality < 50 { 5000 / quality } else { 200 - quality * 2 };
    base.map(|step| ((step as u32 * scale + 50) / 100).clamp(1, 255) as u16)
}

/// Writes bits into bytes, adding the zero byte the format wants after every 0xFF.
struct BitWriter {
    out: Vec<u8>,
    held: u32,
    count: u8,
}

impl BitWriter {
    fn put(&mut self, code: u16, length: u8) {
        if length == 0 {
            return;
        }
        self.held = (self.held << length) | (code as u32 & ((1u32 << length) - 1));
        self.count += length;
        while self.count >= 8 {
            let byte = (self.held >> (self.count - 8)) as u8;
            self.out.push(byte);
            if byte == 0xff {
                self.out.push(0);
            }
            self.count -= 8;
        }
    }

    fn finish(&mut self) {
        if self.count > 0 {
            // The last byte is filled with ones.
            let pad = 8 - self.count;
            self.put((1u16 << pad) - 1, pad);
        }
    }
}

/// How many bits a coefficient needs, and the bits themselves, as the format encodes magnitudes.
fn magnitude(value: i32) -> (u8, u16) {
    let size = (32 - value.unsigned_abs().leading_zeros()) as u8;
    let bits = if value < 0 { (value - 1) as u16 & ((1u16 << size) - 1) } else { value as u16 };
    (size, bits)
}

/// The two-dimensional DCT of an 8x8 block of samples centered on zero.
fn forward_dct(block: &[f32; 64]) -> [f32; 64] {
    let cosine = |position: usize, frequency: usize| ((2 * position + 1) as f32 * frequency as f32 * std::f32::consts::PI / 16.0).cos();
    let scale = |frequency: usize| if frequency == 0 { std::f32::consts::FRAC_1_SQRT_2 } else { 1.0 };
    // Rows first, then columns.
    let mut rows = [0.0f32; 64];
    for y in 0..8 {
        for u in 0..8 {
            rows[y * 8 + u] = (0..8).map(|x| block[y * 8 + x] * cosine(x, u)).sum::<f32>() * scale(u) / 2.0;
        }
    }
    let mut out = [0.0f32; 64];
    for u in 0..8 {
        for v in 0..8 {
            out[v * 8 + u] = (0..8).map(|y| rows[y * 8 + u] * cosine(y, v)).sum::<f32>() * scale(v) / 2.0;
        }
    }
    out
}

/// A second quality for part of the image.
pub struct Selective<'a> {
    /// Whether the pixel at (x, y) belongs to the part that gets `quality`.
    pub covers: &'a dyn Fn(u32, u32) -> bool,
    pub quality: u8,
}

/// Encodes 8-bit RGB pixels as a JPEG file. With `selective`, the 8x8 blocks that touch
/// its region are kept at its quality and the rest at `quality`.
pub fn encode(rgb: &[u8], width: u32, height: u32, quality: u8, selective: Option<Selective>) -> Vec<u8> {
    let other = selective.as_ref().map_or(quality, |s| s.quality);
    // The file is written with the finer tables, and coarser blocks are rounded down to them.
    let finest = quality.max(other);
    let file_steps = [steps_for(&LUMA_STEPS, finest), steps_for(&CHROMA_STEPS, finest)];
    let steps_at = |q: u8| [steps_for(&LUMA_STEPS, q), steps_for(&CHROMA_STEPS, q)];
    let (plain_steps, region_steps) = (steps_at(quality), steps_at(other));

    let mut out = vec![0xff, 0xd8];
    // JFIF header: version 1.1, no density, no thumbnail.
    out.extend_from_slice(&[0xff, 0xe0, 0, 16, b'J', b'F', b'I', b'F', 0, 1, 1, 0, 0, 1, 0, 1, 0, 0]);
    for (id, steps) in file_steps.iter().enumerate() {
        out.extend_from_slice(&[0xff, 0xdb, 0, 67, id as u8]);
        out.extend(ZIGZAG.iter().map(|at| steps[*at] as u8));
    }
    // Frame: 8 bits, three channels, none of them subsampled.
    out.extend_from_slice(&[0xff, 0xc0, 0, 17, 8]);
    out.extend_from_slice(&(height as u16).to_be_bytes());
    out.extend_from_slice(&(width as u16).to_be_bytes());
    out.extend_from_slice(&[3, 1, 0x11, 0, 2, 0x11, 1, 3, 0x11, 1]);
    for (class_and_id, spec) in [(0x00u8, &DC_LUMA), (0x10, &AC_LUMA), (0x01, &DC_CHROMA), (0x11, &AC_CHROMA)] {
        out.extend_from_slice(&[0xff, 0xc4]);
        out.extend_from_slice(&(19 + spec.values.len() as u16).to_be_bytes());
        out.push(class_and_id);
        out.extend_from_slice(&spec.counts);
        out.extend_from_slice(spec.values);
    }
    out.extend_from_slice(&[0xff, 0xda, 0, 12, 3, 1, 0x00, 2, 0x11, 3, 0x11, 0, 63, 0]);

    let dc_codes = [HuffmanCodes::new(&DC_LUMA), HuffmanCodes::new(&DC_CHROMA)];
    let ac_codes = [HuffmanCodes::new(&AC_LUMA), HuffmanCodes::new(&AC_CHROMA)];
    let mut bits = BitWriter { out, held: 0, count: 0 };
    let mut previous_dc = [0i32; 3];
    let pixel = |x: u32, y: u32| {
        // Blocks that hang over the edge repeat the last row and column.
        let at = (y.min(height - 1) as usize * width as usize + x.min(width - 1) as usize) * 3;
        (rgb[at] as f32, rgb[at + 1] as f32, rgb[at + 2] as f32)
    };
    for block_y in (0..height).step_by(8) {
        for block_x in (0..width).step_by(8) {
            let in_region = selective.as_ref().is_some_and(|s| {
                (0..8).any(|dy| (0..8).any(|dx| block_x + dx < width && block_y + dy < height && (s.covers)(block_x + dx, block_y + dy)))
            });
            let block_steps = if in_region { &region_steps } else { &plain_steps };
            let mut channels = [[0.0f32; 64]; 3];
            for dy in 0..8 {
                for dx in 0..8 {
                    let (r, g, b) = pixel(block_x + dx, block_y + dy);
                    let at = (dy * 8 + dx) as usize;
                    channels[0][at] = 0.299 * r + 0.587 * g + 0.114 * b - 128.0;
                    channels[1][at] = -0.168_736 * r - 0.331_264 * g + 0.5 * b;
                    channels[2][at] = 0.5 * r - 0.418_688 * g - 0.081_312 * b;
                }
            }
            for (channel, samples) in channels.iter().enumerate() {
                let table = usize::from(channel > 0);
                let transformed = forward_dct(samples);
                let mut levels = [0i32; 64];
                for (slot, at) in ZIGZAG.iter().enumerate() {
                    let (own, file) = (block_steps[table][*at] as f32, file_steps[table][*at] as f32);
                    // What this block's own quality would keep, expressed in the file's steps.
                    levels[slot] = ((transformed[*at] / own).round() * own / file).round() as i32;
                }
                let (size, extra) = magnitude(levels[0] - previous_dc[channel]);
                previous_dc[channel] = levels[0];
                bits.put(dc_codes[table].code[size as usize], dc_codes[table].length[size as usize]);
                bits.put(extra, size);
                let last = (1..64).rev().find(|slot| levels[*slot] != 0).unwrap_or(0);
                let mut zeros = 0u8;
                for level in &levels[1..=last] {
                    if *level == 0 {
                        zeros += 1;
                        continue;
                    }
                    while zeros > 15 {
                        bits.put(ac_codes[table].code[0xf0], ac_codes[table].length[0xf0]);
                        zeros -= 16;
                    }
                    let (size, extra) = magnitude(*level);
                    let symbol = ((zeros << 4) | size) as usize;
                    bits.put(ac_codes[table].code[symbol], ac_codes[table].length[symbol]);
                    bits.put(extra, size);
                    zeros = 0;
                }
                if last < 63 {
                    bits.put(ac_codes[table].code[0], ac_codes[table].length[0]);
                }
            }
        }
    }
    bits.finish();
    let mut out = bits.out;
    out.extend_from_slice(&[0xff, 0xd9]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture with smooth color and a busy stripe pattern, so both kinds of content are tried.
    fn picture(width: u32, height: u32) -> Vec<u8> {
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        for y in 0..height {
            for x in 0..width {
                let stripes = if (x / 3 + y / 5) % 2 == 0 { 40 } else { 0 };
                rgb.extend_from_slice(&[(x * 255 / width) as u8, (y * 255 / height) as u8, (100 + stripes) as u8]);
            }
        }
        rgb
    }

    fn decode(file: &[u8]) -> (u32, u32, Vec<u8>) {
        let image = image::load_from_memory_with_format(file, image::ImageFormat::Jpeg).expect("a decoder reads the file").to_rgb8();
        (image.width(), image.height(), image.into_raw())
    }

    /// The average difference per channel between two pictures, within a rectangle.
    fn difference(a: &[u8], b: &[u8], width: u32, area: (u32, u32, u32, u32)) -> f32 {
        let (x0, y0, x1, y1) = area;
        let mut total = 0u64;
        for y in y0..y1 {
            for x in x0..x1 {
                let at = ((y * width + x) * 3) as usize;
                total += (0..3).map(|c| (a[at + c] as i64 - b[at + c] as i64).unsigned_abs()).sum::<u64>();
            }
        }
        total as f32 / ((x1 - x0) * (y1 - y0) * 3) as f32
    }

    #[test]
    fn a_standard_decoder_reads_the_file_back() {
        // Sizes that are not multiples of the block size are the ones that go wrong.
        for (width, height) in [(64, 48), (37, 21), (8, 8), (1, 1), (100, 3)] {
            let rgb = picture(width, height);
            let file = encode(&rgb, width, height, 92, None);
            assert_eq!(&file[..2], &[0xff, 0xd8]);
            assert_eq!(&file[file.len() - 2..], &[0xff, 0xd9]);
            let (w, h, decoded) = decode(&file);
            assert_eq!((w, h), (width, height));
            let off = difference(&rgb, &decoded, width, (0, 0, width, height));
            assert!(off < 6.0, "{width}x{height} is off by {off}");
        }
    }

    #[test]
    fn lower_quality_gives_a_smaller_and_rougher_file() {
        let rgb = picture(96, 96);
        let (fine, rough) = (encode(&rgb, 96, 96, 95, None), encode(&rgb, 96, 96, 20, None));
        assert!(rough.len() * 2 < fine.len(), "{} against {}", rough.len(), fine.len());
        let whole = (0, 0, 96, 96);
        assert!(difference(&rgb, &decode(&rough).2, 96, whole) > difference(&rgb, &decode(&fine).2, 96, whole));
    }

    #[test]
    fn the_selected_region_keeps_its_quality_and_the_rest_pays_for_the_size() {
        let rgb = picture(128, 64);
        // The left half is kept at 95, the right half written at 20.
        let left = |x: u32, _: u32| x < 64;
        let mixed = encode(&rgb, 128, 64, 20, Some(Selective { covers: &left, quality: 95 }));
        let (fine, rough) = (encode(&rgb, 128, 64, 95, None), encode(&rgb, 128, 64, 20, None));
        assert!(rough.len() < mixed.len() && mixed.len() < fine.len(), "{} {} {}", rough.len(), mixed.len(), fine.len());

        let decoded = decode(&mixed).2;
        let (left_off, right_off) = (difference(&rgb, &decoded, 128, (0, 0, 64, 64)), difference(&rgb, &decoded, 128, (64, 0, 128, 64)));
        assert!(left_off * 2.0 < right_off, "left {left_off}, right {right_off}");
        // Each half looks like the whole picture written at that half's quality.
        let fine_left = difference(&rgb, &decode(&fine).2, 128, (0, 0, 64, 64));
        let rough_right = difference(&rgb, &decode(&rough).2, 128, (64, 0, 128, 64));
        assert!((left_off - fine_left).abs() < 0.6, "{left_off} against {fine_left}");
        assert!((right_off - rough_right).abs() < 1.5, "{right_off} against {rough_right}");
    }

    #[test]
    fn a_region_of_lower_quality_works_the_other_way_round() {
        let rgb = picture(64, 64);
        let top = |_: u32, y: u32| y < 32;
        let mixed = encode(&rgb, 64, 64, 90, Some(Selective { covers: &top, quality: 15 }));
        let decoded = decode(&mixed).2;
        assert!(difference(&rgb, &decoded, 64, (0, 0, 64, 32)) > difference(&rgb, &decoded, 64, (0, 32, 64, 64)) * 2.0);
    }
}
