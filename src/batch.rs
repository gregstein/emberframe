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

//! Batch Process: the same changes made to many files in one go.
//!
//! Each file is opened as a document, scaled if asked, and exported under its new name.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::i18n::tf;
use crate::model::{Document, Editor};
use crate::optimize::{encode, ExportFormat, ExportSettings};

/// How the files are resized, as Fireworks' Scale step offers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    None,
    /// To exactly this width and height.
    ToSize(u32, u32),
    /// As large as fits inside this width and height, keeping the proportions.
    FitArea(u32, u32),
    Percent(f32),
}

impl Scale {
    /// The size an image of `width` by `height` comes out at.
    pub fn of(self, width: u32, height: u32) -> (u32, u32) {
        let whole = |v: f32| v.round().clamp(1.0, 30000.0) as u32;
        match self {
            Scale::None => (width, height),
            Scale::ToSize(w, h) => (w.max(1), h.max(1)),
            Scale::FitArea(w, h) => {
                let by = (w as f32 / width.max(1) as f32).min(h as f32 / height.max(1) as f32);
                (whole(width as f32 * by), whole(height as f32 * by))
            }
            Scale::Percent(percent) => (whole(width as f32 * percent / 100.0), whole(height as f32 * percent / 100.0)),
        }
    }
}

/// What to do with the files.
#[derive(Clone, Debug)]
pub struct Batch {
    pub scale: Scale,
    /// The export settings for every file; without them each file keeps its own:
    /// a document its Optimize settings, an image the format it came in.
    pub settings: Option<ExportSettings>,
    pub prefix: String,
    pub suffix: String,
    /// Where the results go; without it, beside each original.
    pub output: Option<PathBuf>,
}

/// The export settings a file gets when it keeps its own.
fn own_settings(path: &Path, document: Option<&Document>) -> ExportSettings {
    if let Some(document) = document {
        return document.page().export;
    }
    let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let format = match extension.as_str() {
        "jpg" | "jpeg" => ExportFormat::Jpeg,
        "gif" => ExportFormat::Gif,
        _ => ExportFormat::Png32,
    };
    ExportSettings { format, ..ExportSettings::default() }
}

/// The name a file is exported under: its own, between the prefix and the suffix, with the extension of the format.
pub fn target_name(path: &Path, batch: &Batch, format: ExportFormat) -> String {
    let stem = path.file_stem().map_or_else(|| "image".into(), |s| s.to_string_lossy().into_owned());
    format!("{}{stem}{}.{}", batch.prefix, batch.suffix, format.extension())
}

/// Puts one file through the batch. Returns where the result was written.
pub fn process(path: &Path, batch: &Batch, open: impl Fn(&[u8], &Path) -> Result<(Document, bool)>) -> Result<PathBuf> {
    let bytes = std::fs::read(path).with_context(|| tf("could not read {}", &[&path.display()]))?;
    let (document, is_document) = open(&bytes, path)?;
    let settings = batch.settings.unwrap_or_else(|| own_settings(path, is_document.then_some(&document)));
    let mut editor = Editor::new(document);
    let page = editor.doc.page();
    let size = batch.scale.of(page.width, page.height);
    if size != (page.width, page.height) {
        editor.resize_image(size.0, size.1);
    }
    let encoded = encode(&editor.export_page(), &settings)?;
    let folder = batch.output.clone().or_else(|| path.parent().map(Path::to_path_buf)).unwrap_or_default();
    std::fs::create_dir_all(&folder).with_context(|| tf("could not write {}", &[&folder.display()]))?;
    let target = folder.join(target_name(path, batch, settings.format));
    // A result never takes the place of a file without a trace: what was there is kept in a folder of originals.
    if target.exists() {
        let originals = folder.join("Original Files");
        std::fs::create_dir_all(&originals).with_context(|| tf("could not write {}", &[&originals.display()]))?;
        let kept = originals.join(target.file_name().unwrap_or_default());
        if !kept.exists() {
            std::fs::copy(&target, &kept).with_context(|| tf("could not write {}", &[&kept.display()]))?;
        }
    }
    std::fs::write(&target, encoded).with_context(|| tf("could not write {}", &[&target.display()]))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmap::Bitmap;

    fn plain() -> Batch {
        Batch { scale: Scale::None, settings: None, prefix: String::new(), suffix: String::new(), output: None }
    }

    #[test]
    fn scaling_gives_the_size_asked_for() {
        assert_eq!(Scale::None.of(400, 300), (400, 300));
        assert_eq!(Scale::ToSize(50, 60).of(400, 300), (50, 60));
        assert_eq!(Scale::FitArea(100, 100).of(400, 300), (100, 75));
        assert_eq!(Scale::FitArea(1000, 150).of(400, 300), (200, 150));
        assert_eq!(Scale::Percent(25.0).of(400, 300), (100, 75));
        assert_eq!(Scale::Percent(0.01).of(4, 3), (1, 1));
    }

    #[test]
    fn a_file_is_scaled_renamed_and_the_one_it_replaces_is_kept() {
        let dir = std::env::temp_dir().join(format!("pisiworks-batch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("photo.png");
        let image = image::RgbaImage::from_pixel(40, 20, image::Rgba([200, 10, 10, 255]));
        image.save(&source).unwrap();
        let open = |bytes: &[u8], path: &Path| -> Result<(Document, bool)> { Ok((Document::from_bitmap(path.file_stem().unwrap().to_string_lossy().into_owned(), Bitmap::decode(bytes)?), false)) };
        // Renamed into another folder, at half the size, as a JPEG.
        let out = dir.join("out");
        let batch = Batch { scale: Scale::Percent(50.0), settings: Some(ExportSettings { format: ExportFormat::Jpeg, ..ExportSettings::default() }), prefix: "k_".into(), suffix: "_web".into(), output: Some(out.clone()) };
        let target = process(&source, &batch, open).unwrap();
        assert_eq!(target, out.join("k_photo_web.jpg"));
        let made = image::open(&target).unwrap();
        assert_eq!((made.width(), made.height()), (20, 10));
        // With nothing changed the result lands on the original, which is kept aside first.
        let before = std::fs::read(&source).unwrap();
        assert_eq!(process(&source, &plain(), open).unwrap(), source);
        assert_eq!(std::fs::read(dir.join("Original Files").join("photo.png")).unwrap(), before);
        assert_eq!(image::open(&source).unwrap().width(), 40);
        // A file that cannot be read is an error, not a crash.
        assert!(process(&dir.join("missing.png"), &plain(), open).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
