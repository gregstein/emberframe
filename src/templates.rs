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

//! Templates: documents to start a new one from.
//!
//! A few come with the program, made on the spot from a size and a grid; the rest
//! are documents the user saved as templates, kept in a folder beside the settings.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::i18n::tf;
use crate::model::Document;
use crate::theme::APP_NAME;

/// A template that comes with the program.
pub struct Template {
    pub group: &'static str,
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    /// The columns of a grid system, marked with guides; none for a plain page.
    pub columns: u32,
    /// The space between two columns of the grid.
    pub gutter: f32,
}

const fn plain(group: &'static str, name: &'static str, width: u32, height: u32) -> Template {
    Template { group, name, width, height, columns: 0, gutter: 0.0 }
}

pub const BUILT_IN: &[Template] = &[
    plain("Document Presets", "Web page, 800 x 600", 800, 600),
    plain("Document Presets", "Web page, 1024 x 768", 1024, 768),
    plain("Document Presets", "Web page, 1280 x 800", 1280, 800),
    plain("Document Presets", "Web page, 1920 x 1080", 1920, 1080),
    plain("Document Presets", "A4 at 72 dpi, 595 x 842", 595, 842),
    plain("Document Presets", "Banner, 728 x 90", 728, 90),
    plain("Document Presets", "Banner, 300 x 250", 300, 250),
    plain("Document Presets", "Icon, 256 x 256", 256, 256),
    Template { group: "Grid Systems", name: "960 pixels, 12 columns", width: 960, height: 800, columns: 12, gutter: 20.0 },
    Template { group: "Grid Systems", name: "960 pixels, 16 columns", width: 960, height: 800, columns: 16, gutter: 20.0 },
    Template { group: "Grid Systems", name: "1200 pixels, 12 columns", width: 1200, height: 900, columns: 12, gutter: 30.0 },
    plain("Mobile", "Phone, 320 x 480", 320, 480),
    plain("Mobile", "Phone, 360 x 640", 360, 640),
    plain("Mobile", "Tablet, 768 x 1024", 768, 1024),
];

impl Template {
    /// A new document of the template's size, with a guide at each side of every column of its grid.
    pub fn document(&self) -> Document {
        let mut doc = Document::new(self.width, self.height);
        if self.columns > 0 {
            let step = self.width as f32 / self.columns as f32;
            let guides = &mut doc.pages[0].guides.vertical;
            for column in 0..self.columns {
                let left = column as f32 * step;
                guides.push((left + self.gutter / 2.0).round());
                guides.push((left + step - self.gutter / 2.0).round());
            }
        }
        doc
    }
}

/// Where the user's own templates are kept.
pub fn folder() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(APP_NAME.to_lowercase()).join("templates"))
}

/// The user's own templates, by name.
pub fn saved() -> Vec<(String, PathBuf)> {
    let Some(entries) = folder().and_then(|folder| std::fs::read_dir(folder).ok()) else { return Vec::new() };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e.eq_ignore_ascii_case(crate::theme::FILE_EXTENSION)))
        .filter_map(|path| Some((path.file_stem()?.to_string_lossy().into_owned(), path)))
        .collect();
    found.sort();
    found
}

/// A name as it can stand in a file name: nothing that would lead out of the folder.
pub fn file_name(name: &str) -> String {
    let cleaned = name_part(name).trim_matches('.').trim().to_string();
    if cleaned.is_empty() { "template".into() } else { cleaned }
}

/// A piece of a file name, such as a prefix: what cannot stand in one is replaced.
pub fn name_part(text: &str) -> String {
    text.trim().chars().map(|c| if c.is_control() || "/\\:*?\"<>|".contains(c) { '_' } else { c }).collect()
}

/// Keeps a document, already written out as text, as a template under a name.
pub fn save(name: &str, document: &str) -> Result<PathBuf> {
    let folder = folder().context("no settings folder")?;
    std::fs::create_dir_all(&folder).with_context(|| tf("could not write {}", &[&folder.display()]))?;
    let path = folder.join(format!("{}.{}", file_name(name), crate::theme::FILE_EXTENSION));
    std::fs::write(&path, document).with_context(|| tf("could not write {}", &[&path.display()]))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_template_marks_its_columns_with_guides() {
        let grid = BUILT_IN.iter().find(|t| t.name == "960 pixels, 12 columns").unwrap();
        let doc = grid.document();
        let page = &doc.pages[0];
        assert_eq!((page.width, page.height), (960, 800));
        assert_eq!(page.guides.vertical.len(), 24);
        assert_eq!(&page.guides.vertical[..4], &[10.0, 70.0, 90.0, 150.0]);
        assert_eq!(page.guides.vertical[23], 950.0);
        assert!(BUILT_IN[0].document().pages[0].guides.vertical.is_empty());
    }

    #[test]
    fn a_template_name_cannot_lead_out_of_the_folder() {
        assert_eq!(file_name("../../etc/passwd"), "_.._etc_passwd");
        assert_eq!(file_name("  Ders 1: Afiş  "), "Ders 1_ Afiş");
        assert_eq!(file_name(".."), "template");
        assert_eq!(file_name(""), "template");
    }
}
