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

//! Preferences kept between runs, in the user's configuration folder.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
use crate::theme::APP_NAME;

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    /// A language code such as `tr`. Empty until the user picks one, so the
    /// interface follows the desktop's language.
    #[serde(default)]
    language: Option<String>,
    /// What File > Page Setup was last set to.
    #[serde(default)]
    pub print: crate::print::PrintSetup,
    /// The words Check Spelling was told are right, though no dictionary has them.
    #[serde(default)]
    pub personal_words: Vec<String>,
    /// What Edit > Preferences sets.
    #[serde(default)]
    pub preferences: Preferences,
    /// Keyboard shortcuts the user has changed: the command, by its English menu name, and the keys it has now. No keys means it has none.
    #[serde(default)]
    pub shortcuts: std::collections::HashMap<String, String>,
}

/// What Edit > Preferences sets.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    /// How many steps Undo can go back.
    #[serde(default = "default_undo_steps")]
    pub undo_steps: u32,
    /// How near the pointer has to come to an object to pick it, in pixels on screen.
    #[serde(default = "default_pick_distance")]
    pub pick_distance: f32,
    /// How near something has to come to a guide, the grid or another object to snap to it, in pixels on screen.
    #[serde(default = "default_snap_distance")]
    pub snap_distance: f32,
    /// How far apart the lines of the grid are, across and down, in pixels of the page.
    #[serde(default = "default_grid")]
    pub grid: (f32, f32),
    #[serde(default = "default_grid_color")]
    pub grid_color: crate::model::Color,
    #[serde(default)]
    pub snap_to_grid: bool,
}

fn default_undo_steps() -> u32 {
    200
}

fn default_pick_distance() -> f32 {
    3.0
}

fn default_snap_distance() -> f32 {
    5.0
}

fn default_grid() -> (f32, f32) {
    (36.0, 36.0)
}

fn default_grid_color() -> crate::model::Color {
    crate::model::Color::rgb(0, 255, 255)
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            undo_steps: default_undo_steps(),
            pick_distance: default_pick_distance(),
            snap_distance: default_snap_distance(),
            grid: default_grid(),
            grid_color: default_grid_color(),
            snap_to_grid: false,
        }
    }
}

impl Settings {
    pub fn language(&self) -> Option<Language> {
        self.language.as_deref().and_then(Language::from_code)
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = Some(language.code().to_string());
    }

    /// Reads the settings. A missing or damaged file gives the defaults.
    pub fn load() -> Self {
        Self::path().map(|path| Self::load_from(&path)).unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        match Self::path() {
            Some(path) => self.save_to(&path),
            None => anyhow::bail!("no configuration folder"),
        }
    }

    fn path() -> Option<PathBuf> {
        Some(dirs::config_dir()?.join(APP_NAME.to_lowercase()).join("settings.json"))
    }

    fn load_from(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
    }

    fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_survives_a_save_and_load() {
        let path = std::env::temp_dir().join("pisiworks_settings_test").join("settings.json");
        let mut settings = Settings::default();
        assert_eq!(settings.language(), None);
        settings.set_language(Language::Turkish);
        settings.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path).language(), Some(Language::Turkish));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn damaged_file_gives_the_defaults() {
        let path = std::env::temp_dir().join("pisiworks_settings_damaged.json");
        std::fs::write(&path, "not json").unwrap();
        assert_eq!(Settings::load_from(&path).language(), None);
        std::fs::remove_file(path).unwrap();
    }
}
