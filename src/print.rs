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

//! Printing: the page setup, the page laid out on paper as a PDF, and handing that to the system's printers.
//!
//! The page is printed at 72 pixels to the inch, as Fireworks does for a document at
//! its default resolution, so a pixel is a point on paper.

use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::i18n::{t, tf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSize {
    A3,
    #[default]
    A4,
    A5,
    Letter,
    Legal,
}

impl PaperSize {
    pub const ALL: [PaperSize; 5] = [PaperSize::A3, PaperSize::A4, PaperSize::A5, PaperSize::Letter, PaperSize::Legal];

    /// The name printers know the paper by, which is also what the dialog shows.
    pub fn name(self) -> &'static str {
        match self {
            PaperSize::A3 => "A3",
            PaperSize::A4 => "A4",
            PaperSize::A5 => "A5",
            PaperSize::Letter => "Letter",
            PaperSize::Legal => "Legal",
        }
    }

    /// Width and height in points, held upright.
    pub fn points(self) -> (f32, f32) {
        match self {
            PaperSize::A3 => (841.89, 1190.55),
            PaperSize::A4 => (595.28, 841.89),
            PaperSize::A5 => (419.53, 595.28),
            PaperSize::Letter => (612.0, 792.0),
            PaperSize::Legal => (612.0, 1008.0),
        }
    }
}

/// What File > Page Setup sets, kept between runs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrintSetup {
    #[serde(default)]
    pub paper: PaperSize,
    #[serde(default)]
    pub landscape: bool,
    /// Left, right, top and bottom margins in millimeters.
    #[serde(default = "default_margins")]
    pub margins: [f32; 4],
    /// Scales the page to fill the paper inside the margins, instead of printing it at its own size.
    #[serde(default)]
    pub fit: bool,
}

fn default_margins() -> [f32; 4] {
    [10.0; 4]
}

impl Default for PrintSetup {
    fn default() -> Self {
        Self { paper: PaperSize::default(), landscape: false, margins: default_margins(), fit: false }
    }
}

const POINTS_PER_MM: f32 = 72.0 / 25.4;

/// Where a page lands on the paper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// The paper, in points, as it is turned.
    pub paper: (f32, f32),
    /// The printed page: left, bottom, width and height in points, measured from the bottom left of the paper.
    pub image: (f32, f32, f32, f32),
    /// The part of the paper inside the margins that the page is cut to, the same way.
    pub clip: (f32, f32, f32, f32),
}

impl PrintSetup {
    /// Lays a page of `width` by `height` pixels out on the paper: at its own size
    /// from the top left of the margins, or scaled to fit inside them and centered.
    pub fn layout(&self, width: u32, height: u32) -> Layout {
        let (short, long) = self.paper.points();
        let paper = if self.landscape { (long, short) } else { (short, long) };
        let [left, right, top, bottom] = self.margins.map(|mm| mm.max(0.0) * POINTS_PER_MM);
        let (room_w, room_h) = ((paper.0 - left - right).max(1.0), (paper.1 - top - bottom).max(1.0));
        let clip = (left, bottom, room_w, room_h);
        let (w, h) = (width.max(1) as f32, height.max(1) as f32);
        let image = if self.fit {
            let scale = (room_w / w).min(room_h / h);
            let (w, h) = (w * scale, h * scale);
            (left + (room_w - w) / 2.0, bottom + (room_h - h) / 2.0, w, h)
        } else {
            (left, paper.1 - top - h, w, h)
        };
        Layout { paper, image, clip }
    }

    /// Whether a page printed at its own size is too big for the paper inside the margins.
    pub fn overflows(&self, width: u32, height: u32) -> bool {
        let layout = self.layout(width, height);
        !self.fit && (layout.image.2 > layout.clip.2 + 0.5 || layout.image.3 > layout.clip.3 + 0.5)
    }
}

/// A picture to print: three bytes for each pixel, row after row from the top, and its size in pixels.
pub struct Picture {
    pub rgb: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl PrintSetup {
    /// The sheets of paper a picture is printed on: for each, where the picture
    /// lies on it. One sheet when it fits or is scaled to fit; otherwise as many
    /// as it takes, each showing the next part of it through the margins, from
    /// the top left across and then down.
    pub fn sheets(&self, width: u32, height: u32) -> Vec<Layout> {
        let whole = self.layout(width, height);
        if !self.overflows(width, height) {
            return vec![whole];
        }
        let (across, down) = ((whole.image.2 / whole.clip.2).ceil().max(1.0) as u32, (whole.image.3 / whole.clip.3).ceil().max(1.0) as u32);
        let mut sheets = Vec::new();
        for row in 0..down {
            for column in 0..across {
                let mut sheet = whole;
                // The picture is slid left and up by whole printable areas; what is on this sheet shows through the margins.
                sheet.image.0 = whole.image.0 - column as f32 * whole.clip.2;
                sheet.image.1 = whole.image.1 + row as f32 * whole.clip.3;
                sheets.push(sheet);
            }
        }
        sheets
    }
}

/// A PDF of several pictures, each on as many sheets as it needs.
pub fn pdf_of(pictures: &[Picture], setup: &PrintSetup) -> Result<Vec<u8>> {
    if pictures.is_empty() || pictures.iter().any(|p| p.rgb.is_empty() || p.rgb.len() != p.width as usize * p.height as usize * 3) {
        bail!(t("rendered image has an invalid size"));
    }
    let sheets: Vec<(usize, Layout)> = pictures.iter().enumerate().flat_map(|(index, p)| setup.sheets(p.width, p.height).into_iter().map(move |sheet| (index, sheet))).collect();
    // The objects of the file, by number: the catalog, the list of pages, each picture, then a page and its drawing for each sheet.
    let picture_object = |index: usize| 3 + index;
    let page_object = |sheet: usize| 3 + pictures.len() + sheet * 2;
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    let mut object = |out: &mut Vec<u8>, head: String, stream: Option<&[u8]>| {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{head}\n", offsets.len()).as_bytes());
        if let Some(stream) = stream {
            out.extend_from_slice(b"stream\n");
            out.extend_from_slice(stream);
            out.extend_from_slice(b"\nendstream\n");
        }
        out.extend_from_slice(b"endobj\n");
    };
    object(&mut out, "<< /Type /Catalog /Pages 2 0 R >>".into(), None);
    let kids: Vec<String> = (0..sheets.len()).map(|sheet| format!("{} 0 R", page_object(sheet))).collect();
    object(&mut out, format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), sheets.len()), None);
    for picture in pictures {
        object(
            &mut out,
            format!("<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>", picture.width, picture.height, picture.rgb.len()),
            Some(&picture.rgb),
        );
    }
    for (sheet, (index, Layout { paper, image, clip })) in sheets.iter().enumerate() {
        object(
            &mut out,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /XObject << /Im0 {} 0 R >> >> /Contents {} 0 R >>",
                paper.0,
                paper.1,
                picture_object(*index),
                page_object(sheet) + 1
            ),
            None,
        );
        // Cut to the margins, then the picture stretched over its place on the paper.
        let content = format!("q {:.2} {:.2} {:.2} {:.2} re W n {:.2} 0 0 {:.2} {:.2} {:.2} cm /Im0 Do Q\n", clip.0, clip.1, clip.2, clip.3, image.2, image.3, image.0, image.1);
        object(&mut out, format!("<< /Length {} >>", content.len()), Some(content.as_bytes()));
    }
    let table = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
    for offset in &offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{table}\n%%EOF\n", offsets.len() + 1).as_bytes());
    Ok(out)
}

/// A PDF with one picture on it.
#[cfg(test)]
pub fn pdf(rgb: &[u8], width: u32, height: u32, setup: &PrintSetup) -> Result<Vec<u8>> {
    pdf_of(&[Picture { rgb: rgb.to_vec(), width, height }], setup)
}

/// Which pages of the document are printed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageRange {
    #[default]
    Current,
    All,
    /// From one page to another, counted from 1, both included.
    Pages,
}

/// The pages a range stands for, counted from 0, in a document of `count` pages whose page on show is `current`.
pub fn pages_in(range: PageRange, current: usize, count: usize, from: u32, to: u32) -> Vec<usize> {
    match range {
        PageRange::Current => vec![current.min(count.saturating_sub(1))],
        PageRange::All => (0..count).collect(),
        PageRange::Pages => {
            let (low, high) = (from.min(to).max(1) as usize, (from.max(to) as usize).min(count));
            (low..=high).map(|page| page - 1).collect()
        }
    }
}

/// The names in the output of `lpstat -e`, one printer to a line.
fn parse_printers(listing: &str) -> Vec<String> {
    listing.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

/// The printer named in the output of `lpstat -d`, which ends with its name after a colon when there is one.
fn parse_default(answer: &str) -> Option<String> {
    let (_, name) = answer.lines().next()?.rsplit_once(": ")?;
    (!name.trim().is_empty()).then(|| name.trim().to_string())
}

/// The printers set up on this system, and which of them is the default.
pub fn printers() -> (Vec<String>, Option<String>) {
    let run = |flag: &str| Command::new("lpstat").arg(flag).output().ok().filter(|out| out.status.success()).map(|out| String::from_utf8_lossy(&out.stdout).into_owned());
    let names = run("-e").map(|listing| parse_printers(&listing)).unwrap_or_default();
    let default = run("-d").and_then(|answer| parse_default(&answer)).filter(|name| names.contains(name));
    (names, default)
}

/// What `lp` is run with to print a PDF that is already laid out for its paper.
/// What is asked of the printer itself, in the words every CUPS printer understands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// On both sides of the paper, turned along its long edge (its short edge for a landscape page).
    pub two_sided: bool,
    pub grayscale: bool,
}

fn lp_arguments(file: &Path, printer: &str, copies: u32, setup: &PrintSetup, title: &str, options: Options) -> Vec<String> {
    let mut asked = Vec::new();
    if options.two_sided {
        asked.extend(["-o".to_string(), format!("sides={}", if setup.landscape { "two-sided-short-edge" } else { "two-sided-long-edge" })]);
    }
    if options.grayscale {
        asked.extend(["-o".to_string(), "print-color-mode=monochrome".to_string()]);
    }
    let mut arguments = vec![
        "-d".into(),
        printer.into(),
        "-n".into(),
        copies.clamp(1, 999).to_string(),
        "-t".into(),
        title.into(),
        "-o".into(),
        format!("media={}", setup.paper.name()),
        // The PDF is turned already; the printer must not turn or scale it again.
        "-o".into(),
        "print-scaling=none".into(),
    ];
    arguments.extend(asked);
    arguments.extend(["--".to_string(), file.display().to_string()]);
    arguments
}

/// Sends a PDF to a printer. Gives back what the print system answered, which names the job.
pub fn send(pdf: &[u8], printer: &str, copies: u32, setup: &PrintSetup, title: &str, options: Options) -> Result<String> {
    let file = std::env::temp_dir().join(format!("pisiworks-print-{}.pdf", std::process::id()));
    std::fs::write(&file, pdf).with_context(|| tf("could not write {}", &[&file.display()]))?;
    let ran = Command::new("lp").args(lp_arguments(&file, printer, copies, setup, title, options)).output();
    // The print system has taken its own copy by the time `lp` returns.
    let _ = std::fs::remove_file(&file);
    let out = ran.map_err(|_| anyhow!(t("The print system (CUPS) is not installed")))?;
    if !out.status.success() {
        bail!(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.05
    }

    #[test]
    fn a_page_is_placed_at_its_own_size_from_the_top_left_margin() {
        let setup = PrintSetup::default();
        let layout = setup.layout(200, 100);
        assert_eq!(layout.paper, (595.28, 841.89));
        let margin = 10.0 * POINTS_PER_MM;
        // A pixel is a point; the top of the picture is a margin below the top of the paper.
        assert!(near(layout.image.0, margin) && near(layout.image.2, 200.0) && near(layout.image.3, 100.0));
        assert!(near(layout.image.1 + layout.image.3, 841.89 - margin));
        assert!(near(layout.clip.2, 595.28 - 2.0 * margin) && near(layout.clip.3, 841.89 - 2.0 * margin));
        assert!(!setup.overflows(200, 100) && setup.overflows(2000, 100));
    }

    #[test]
    fn fitting_scales_the_page_to_the_margins_and_centers_it() {
        let setup = PrintSetup { fit: true, landscape: true, margins: [20.0, 20.0, 10.0, 10.0], ..PrintSetup::default() };
        let layout = setup.layout(400, 100);
        assert_eq!(layout.paper, (841.89, 595.28), "turned on its side");
        let (room_w, room_h) = (841.89 - 40.0 * POINTS_PER_MM, 595.28 - 20.0 * POINTS_PER_MM);
        // A wide page fills the width and keeps its shape, half way up.
        assert!(near(layout.image.2, room_w) && near(layout.image.3, room_w / 4.0));
        assert!(near(layout.image.0, 20.0 * POINTS_PER_MM));
        assert!(near(layout.image.1, 10.0 * POINTS_PER_MM + (room_h - room_w / 4.0) / 2.0));
        assert!(!setup.overflows(40000, 100), "a fitted page never overflows");
        // A small page is enlarged too.
        assert!(setup.layout(10, 10).image.3 > 300.0);
    }

    #[test]
    fn the_pdf_holds_the_picture_and_its_table_points_at_its_objects() {
        let rgb: Vec<u8> = (0..4 * 2 * 3).map(|i| i as u8).collect();
        let bytes = pdf(&rgb, 4, 2, &PrintSetup::default()).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.starts_with("%PDF-1.4") && text.trim_end().ends_with("%%EOF"));
        assert!(text.contains("/MediaBox [0 0 595.28 841.89]") && text.contains("/Width 4 /Height 2") && text.contains("/Length 24"));
        // The picture is in there byte for byte.
        assert!(bytes.windows(rgb.len()).any(|window| window == rgb));
        // The table starts where the trailer says, and each entry is where an object begins.
        let find = |needle: &str| bytes.windows(needle.len()).position(|window| window == needle.as_bytes()).unwrap();
        let table: usize = text[text.rfind("startxref").unwrap() + 10..].lines().next().unwrap().parse().unwrap();
        assert_eq!(table, find("xref\n0 6"));
        let entries: Vec<usize> = String::from_utf8_lossy(&bytes[table..]).lines().skip(3).take(5).map(|line| line[..10].parse().unwrap()).collect();
        for (number, offset) in entries.iter().enumerate() {
            assert_eq!(*offset, find(&format!("{} 0 obj", number + 1)), "object {}", number + 1);
        }
        // Written out on request, so the file can be checked with a PDF reader.
        if let Some(path) = std::env::var_os("PISIWORKS_SAMPLE_PDF") {
            let (w, h) = (300u32, 200u32);
            let sample: Vec<u8> = (0..w * h).flat_map(|i| if (i % w) < w / 2 { [220, 30, 30] } else { [30, 30, 220] }).collect();
            std::fs::write(path, pdf(&sample, w, h, &PrintSetup { fit: true, landscape: true, ..PrintSetup::default() }).unwrap()).unwrap();
        }
        // A picture of the wrong size is refused.
        assert!(pdf(&rgb, 5, 2, &PrintSetup::default()).is_err() && pdf(&[], 0, 0, &PrintSetup::default()).is_err());
    }

    #[test]
    fn a_page_too_large_for_the_paper_is_spread_over_several_sheets() {
        let setup = PrintSetup::default();
        // A4 inside 10 mm margins takes about 538 by 785 pixels: 1000 by 1000 needs two sheets across and two down.
        assert_eq!(setup.sheets(400, 300).len(), 1);
        let sheets = setup.sheets(1000, 1000);
        assert_eq!(sheets.len(), 4);
        let room = sheets[0].clip;
        // The second sheet shows the picture slid left by one printable width; the third, slid up by one printable height.
        assert!(near(sheets[1].image.0, sheets[0].image.0 - room.2) && near(sheets[1].image.1, sheets[0].image.1));
        assert!(near(sheets[2].image.1, sheets[0].image.1 + room.3) && near(sheets[2].image.0, sheets[0].image.0));
        assert!(sheets.iter().all(|sheet| sheet.clip == room && near(sheet.image.2, 1000.0)));
        // Scaled to fit, it is one sheet whatever its size.
        assert_eq!(PrintSetup { fit: true, ..setup }.sheets(5000, 5000).len(), 1);
        // The file has a page for each sheet of each picture, and holds each picture once.
        let small = Picture { rgb: vec![7; 4 * 4 * 3], width: 4, height: 4 };
        let large = Picture { rgb: vec![9; 1000 * 1000 * 3], width: 1000, height: 1000 };
        let bytes = pdf_of(&[small, large], &setup).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/Count 5") && text.matches("/Type /Page ").count() == 5 && text.matches("/Subtype /Image").count() == 2);
        assert!(pdf_of(&[], &setup).is_err());
    }

    #[test]
    fn a_range_of_pages_is_counted_from_one_and_kept_inside_the_document() {
        assert_eq!(pages_in(PageRange::Current, 2, 5, 1, 1), [2]);
        assert_eq!(pages_in(PageRange::All, 2, 3, 1, 1), [0, 1, 2]);
        assert_eq!(pages_in(PageRange::Pages, 0, 5, 2, 4), [1, 2, 3]);
        assert_eq!(pages_in(PageRange::Pages, 0, 5, 4, 2), [1, 2, 3], "the wrong way round is put right");
        assert_eq!(pages_in(PageRange::Pages, 0, 3, 0, 99), [0, 1, 2]);
        assert!(pages_in(PageRange::Pages, 0, 3, 7, 9).is_empty());
    }

    #[test]
    fn printers_are_read_from_the_print_system_and_jobs_are_addressed_to_them() {
        assert_eq!(parse_printers("HP_LaserJet\n  Canon-MF\n\n"), ["HP_LaserJet", "Canon-MF"]);
        assert!(parse_printers("").is_empty());
        assert_eq!(parse_default("system default destination: HP_LaserJet\n"), Some("HP_LaserJet".into()));
        assert_eq!(parse_default("sistem öntanımlı hedefi: Canon-MF"), Some("Canon-MF".into()));
        assert_eq!(parse_default("no system default destination\n"), None);
        let setup = PrintSetup { paper: PaperSize::A3, ..PrintSetup::default() };
        let arguments = lp_arguments(Path::new("/tmp/a b.pdf"), "HP_LaserJet", 0, &setup, "Poster", Options::default());
        assert!(!arguments.iter().any(|a| a.starts_with("sides=") || a.starts_with("print-color-mode=")));
        let both = lp_arguments(Path::new("/tmp/a.pdf"), "HP", 1, &setup, "Poster", Options { two_sided: true, grayscale: true });
        assert!(both.contains(&"print-color-mode=monochrome".to_string()) && both.iter().any(|a| a.starts_with("sides=two-sided-")));
        assert_eq!(both[both.len() - 2], "--", "the file still comes last, after every option");
        assert_eq!(arguments[..6], ["-d", "HP_LaserJet", "-n", "1", "-t", "Poster"]);
        assert!(arguments.contains(&"media=A3".to_string()));
        // The file comes last, after the mark that ends the options, so its name is never read as one.
        assert_eq!(arguments[arguments.len() - 2..], ["--", "/tmp/a b.pdf"]);
    }

    #[test]
    fn the_setup_is_kept_in_the_settings_and_old_settings_get_the_defaults() {
        let setup = PrintSetup { paper: PaperSize::Legal, landscape: true, margins: [1.0, 2.0, 3.0, 4.0], fit: true };
        let back: PrintSetup = serde_json::from_str(&serde_json::to_string(&setup).unwrap()).unwrap();
        assert_eq!(back, setup);
        assert_eq!(serde_json::from_str::<PrintSetup>("{}").unwrap(), PrintSetup::default());
        assert_eq!(PaperSize::ALL.len(), 5);
        for paper in PaperSize::ALL {
            let (w, h) = paper.points();
            assert!(w < h && !paper.name().is_empty());
        }
    }
}
