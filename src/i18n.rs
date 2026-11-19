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

//! Interface language.
//!
//! English text is the key. Each other language is a table from that key to its
//! translation, and text without an entry is shown in English.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

pub(crate) mod tr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Turkish,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::Turkish];

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Turkish => "tr",
        }
    }

    /// The language's own name, as the Language menu lists it.
    #[cfg(test)]
    pub fn name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Turkish => "Türkçe",
        }
    }

    /// Reads a language code or a locale such as `tr_TR.UTF-8` or `tr-TR`.
    pub fn from_code(code: &str) -> Option<Self> {
        let prefix: String = code.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
        Self::ALL.into_iter().find(|language| prefix.eq_ignore_ascii_case(language.code()))
    }

    fn entries(self) -> Option<&'static HashMap<&'static str, &'static str>> {
        static TURKISH: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
        match self {
            Language::English => None,
            Language::Turkish => Some(TURKISH.get_or_init(|| tr::ENTRIES.iter().copied().collect())),
        }
    }

    /// Returns `text` in this language, or `text` itself when there is no translation.
    pub fn translate<'a>(self, text: &'a str) -> &'a str {
        self.entries().and_then(|entries| entries.get(text).copied()).unwrap_or(text)
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn language() -> Language {
    Language::ALL[CURRENT.load(Ordering::Relaxed) as usize]
}

pub fn set_language(language: Language) {
    CURRENT.store(language as u8, Ordering::Relaxed);
}

/// The language the desktop is set to, or English when it is one we do not have.
pub fn system_language() -> Language {
    // LANGUAGE is a colon-separated list. The first entry we know wins.
    ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .flat_map(|value| value.split(':').filter_map(Language::from_code).collect::<Vec<_>>())
        .next()
        .or_else(|| sys_locale::get_locale().as_deref().and_then(Language::from_code))
        .unwrap_or(Language::English)
}

/// Translates interface text into the current language.
pub fn t(text: &str) -> &str {
    language().translate(text)
}

/// Translates text whose English wording means something else elsewhere in the interface.
/// The table lists it as `text|context`.
pub fn tc<'a>(context: &str, text: &'a str) -> &'a str {
    match language().entries().and_then(|entries| entries.get(format!("{text}|{context}").as_str())) {
        Some(translated) => translated,
        None => text,
    }
}

/// Translates `template`, then puts each of `args` in place of the next `{}`.
pub fn tf(template: &str, args: &[&dyn Display]) -> String {
    let mut parts = t(template).split("{}");
    let mut out = parts.next().unwrap_or_default().to_string();
    for (i, part) in parts.enumerate() {
        if let Some(arg) = args.get(i) {
            out.push_str(&arg.to_string());
        }
        out.push_str(part);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_names_map_to_languages() {
        assert_eq!(Language::from_code("tr_TR.UTF-8"), Some(Language::Turkish));
        assert_eq!(Language::from_code("tr-TR"), Some(Language::Turkish));
        assert_eq!(Language::from_code("TR"), Some(Language::Turkish));
        assert_eq!(Language::from_code("en_US.UTF-8"), Some(Language::English));
        assert_eq!(Language::from_code("C"), None);
        // "tru" is not Turkish.
        assert_eq!(Language::from_code("tru"), None);
    }

    #[test]
    fn missing_entries_fall_back_to_english() {
        assert_eq!(Language::Turkish.translate("Layers"), "Katmanlar");
        assert_eq!(Language::Turkish.translate("my own layer name"), "my own layer name");
        assert_eq!(Language::English.translate("Layers"), "Layers");
    }

    #[test]
    fn tf_replaces_placeholders_in_order() {
        assert_eq!(tf("Exported {} and {} web file(s) beside it", &[&"a.png", &3]), "Exported a.png and 3 web file(s) beside it");
        assert_eq!(tf("{} objects", &[&2]), "2 objects");
        assert_eq!(tf("no placeholders", &[&1]), "no placeholders");
    }

    #[test]
    fn turkish_entries_are_unique_and_keep_their_placeholders() {
        let mut seen = std::collections::HashSet::new();
        for (english, turkish) in tr::ENTRIES {
            assert!(seen.insert(english), "duplicate entry: {english}");
            assert!(!turkish.is_empty(), "empty translation: {english}");
            assert_eq!(
                english.matches("{}").count(),
                turkish.matches("{}").count(),
                "placeholders differ: {english}"
            );
            assert_eq!(english.ends_with("..."), turkish.ends_with("..."), "ellipsis differs: {english}");
            assert_eq!(english.ends_with(':'), turkish.ends_with(':'), "colon differs: {english}");
        }
    }

    /// The string literals passed to `t(` and `tf(` in `source`.
    fn translated_literals(source: &str) -> Vec<String> {
        let mut found = Vec::new();
        for call in ["t(\"", "tf(\""] {
            let mut rest = source;
            while let Some(at) = rest.find(call) {
                let boundary = rest[..at].chars().next_back().is_none_or(|c| !c.is_alphanumeric() && c != '_');
                rest = &rest[at + call.len()..];
                if !boundary {
                    continue;
                }
                let mut literal = String::new();
                let mut chars = rest.chars();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => literal.push(chars.next().unwrap_or('\\')),
                        _ => literal.push(c),
                    }
                }
                found.push(literal);
            }
        }
        found
    }

    #[test]
    fn every_translated_literal_has_a_turkish_entry() {
        let sources = [
            include_str!("autoshape.rs"),
            include_str!("bitmap.rs"),
            include_str!("canvas_view.rs"),
            include_str!("export.rs"),
            include_str!("library.rs"),
            include_str!("main.rs"),
            include_str!("model.rs"),
            include_str!("optimize.rs"),
            include_str!("panels.rs"),
            include_str!("paths.rs"),
            include_str!("ui.rs"),
            include_str!("workspace.rs"),
        ];
        let entries = Language::Turkish.entries().unwrap();
        let mut missing: Vec<String> = sources
            .iter()
            .flat_map(|source| translated_literals(source))
            .filter(|literal| !entries.contains_key(literal.as_str()))
            .collect();
        missing.sort();
        missing.dedup();
        assert!(missing.is_empty(), "no Turkish entry for: {missing:#?}");
    }

    #[test]
    fn every_menu_and_tool_has_a_turkish_entry() {
        use crate::menus::{Entry, Panel, COLOR_TOOLS, MENUS, SCREEN_MODES, TOOL_GROUPS, TOOL_SECTIONS, VIEW_TOOLS};

        fn collect(entries: &'static [Entry], out: &mut Vec<&'static str>) {
            for entry in entries {
                match entry {
                    Entry::Item(name, _, _) => out.push(name),
                    Entry::Sub(name, children) => {
                        out.push(name);
                        collect(children, out);
                    }
                    Entry::Sep => {}
                }
            }
        }

        let mut labels = Vec::new();
        for (title, entries) in MENUS {
            labels.push(*title);
            collect(entries, &mut labels);
        }
        for (title, slots) in TOOL_SECTIONS {
            labels.push(*title);
            labels.extend(slots.iter().map(|slot| slot.label));
        }
        for slots in [COLOR_TOOLS, SCREEN_MODES, VIEW_TOOLS].into_iter().chain(TOOL_GROUPS.iter().copied()) {
            labels.extend(slots.iter().map(|slot| slot.label));
        }
        labels.extend(Panel::ALL.iter().map(|panel| panel.title()));

        let entries = Language::Turkish.entries().unwrap();
        // Zoom levels, text sizes, the language names and the URL panel read the same in Turkish.
        let same = |label: &str| label.ends_with('%') || label == "URL" || label.chars().all(|c| c.is_ascii_digit()) || Language::ALL.iter().any(|l| l.name() == label);
        let missing: Vec<_> = labels.into_iter().filter(|label| !same(label) && !entries.contains_key(label)).collect();
        assert!(missing.is_empty(), "no Turkish entry for: {missing:#?}");
    }
}
