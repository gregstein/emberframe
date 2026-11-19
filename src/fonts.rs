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

//! Picks the interface font from the fonts installed on this system.
//!
//! The same face is used on screen and for export, so text measures the same in both.

use std::sync::OnceLock;

use ab_glyph::{Font, FontVec};
use fontdb::{Database, Family, Query};

/// Tried in order. Segoe UI keeps the look on Windows, the rest ship with Linux desktops.
const PREFERRED: &[&str] =
    &["Segoe UI", "Noto Sans", "Liberation Sans", "DejaVu Sans", "Cantarell", "Ubuntu", "Arial"];

/// A font must draw these to be usable for a Turkish interface.
const REQUIRED_GLYPHS: &str = "ğĞşŞıİçÇöÖüÜ";

struct Picked {
    family: String,
    outlines: Option<FontVec>,
}

fn load(db: &Database, family: Family) -> Option<Picked> {
    let id = db.query(&Query { families: &[family], ..Query::default() })?;
    let name = db.face(id)?.families.first()?.0.clone();
    let font = db.with_face_data(id, |data, index| FontVec::try_from_vec_and_index(data.to_vec(), index).ok())??;
    let complete = REQUIRED_GLYPHS.chars().all(|c| font.glyph_id(c).0 != 0);
    complete.then_some(Picked { family: name, outlines: Some(font) })
}

fn picked() -> &'static Picked {
    static PICKED: OnceLock<Picked> = OnceLock::new();
    PICKED.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        PREFERRED
            .iter()
            .find_map(|name| load(&db, Family::Name(name)))
            .or_else(|| load(&db, Family::SansSerif))
            // GPUI falls back to a system font of its own when the family is missing.
            .unwrap_or_else(|| Picked { family: PREFERRED[0].to_string(), outlines: None })
    })
}

/// The family name to hand to GPUI.
pub fn family() -> &'static str {
    &picked().family
}

/// A font whose letters are all of one width, for text written in lines: the first of these that is installed.
pub fn mono() -> &'static str {
    static MONO: OnceLock<String> = OnceLock::new();
    MONO.get_or_init(|| {
        let db = database();
        let has = |family: Family| db.query(&Query { families: &[family], ..Query::default() });
        ["DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono", "Ubuntu Mono", "Courier New"]
            .iter()
            .find(|name| has(Family::Name(name)).is_some())
            .map(|name| name.to_string())
            .or_else(|| has(Family::Monospace).and_then(|id| db.face(id)).and_then(|face| face.families.first().map(|family| family.0.clone())))
            .unwrap_or_else(|| "monospace".to_string())
    })
}

fn database() -> &'static Database {
    static DATABASE: OnceLock<Database> = OnceLock::new();
    DATABASE.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        db
    })
}

/// The font families installed on this system, in alphabetical order.
pub fn families() -> &'static [String] {
    static FAMILIES: OnceLock<Vec<String>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        let mut names: Vec<String> = database().faces().filter_map(|face| face.families.first().map(|(name, _)| name.clone())).collect();
        names.sort_by_key(|name| name.to_lowercase());
        names.dedup();
        names
    })
}

/// A face that was asked for, and what it lacks: whether it had to stand in for a bold or an italic face the family does not have.
type Found = Option<(&'static FontVec, bool, bool)>;

fn found(family: &str, bold: bool, italic: bool) -> Found {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static FACES: OnceLock<Mutex<HashMap<(String, bool, bool), Found>>> = OnceLock::new();
    let mut faces = FACES.get_or_init(Default::default).lock().ok()?;
    *faces.entry((family.to_string(), bold, italic)).or_insert_with(|| {
        let name = if family.is_empty() { self::family() } else { family };
        let query = Query {
            families: &[Family::Name(name)],
            weight: if bold { fontdb::Weight::BOLD } else { fontdb::Weight::NORMAL },
            style: if italic { fontdb::Style::Italic } else { fontdb::Style::Normal },
            ..Query::default()
        };
        let db = database();
        let id = db.query(&query)?;
        let (heavy, slanted) = db.face(id).map_or((false, false), |face| (face.weight.0 >= 600, face.style != fontdb::Style::Normal));
        let font = db.with_face_data(id, |data, index| FontVec::try_from_vec_and_index(data.to_vec(), index).ok())??;
        // Kept for as long as the program runs: there are only so many fonts.
        Some((&*Box::leak(Box::new(font)), bold && !heavy, italic && !slanted))
    })
}

/// A face of a family as outlines: bold, italic, both or neither, or the nearest
/// the family has. An empty family is the program's own font.
pub fn face(family: &str, bold: bool, italic: bool) -> Option<&'static FontVec> {
    if family.is_empty() && !bold && !italic {
        return outlines();
    }
    found(family, bold, italic).map(|(font, _, _)| font)
}

/// Whether bold and italic have to be made up for a family, because it has no such face of its own.
pub fn made_up(family: &str, bold: bool, italic: bool) -> (bool, bool) {
    if !bold && !italic {
        return (false, false);
    }
    found(family, bold, italic).map_or((false, false), |(_, heavy, slanted)| (heavy, slanted))
}

/// The same face as outlines, for measuring and exporting text.
pub fn outlines() -> Option<&'static FontVec> {
    picked().outlines.as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picked_font_draws_turkish_letters() {
        let Some(font) = outlines() else { return };
        assert!(REQUIRED_GLYPHS.chars().all(|c| font.glyph_id(c).0 != 0), "{} lacks Turkish letters", family());
    }
}
