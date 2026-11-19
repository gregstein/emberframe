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

//! Saved commands: runs of menu commands kept under a name, to be played again.
//!
//! A command is a plain text file with one step on each line, each written as the
//! menu it is in and its name there: `Modify > Flip Horizontal`. Run Script plays
//! any such file; the saved ones live in a folder beside the settings and show at
//! the foot of the Commands menu.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};

use crate::i18n::tf;
use crate::menus::{menu_commands, Cmd, Entry};
use crate::theme::APP_NAME;

pub const EXTENSION: &str = "txt";

/// How a menu command is written in a command file.
pub fn step(menu: &str, name: &str) -> String {
    format!("{menu} > {}", name.trim_end_matches("..."))
}

/// The step a command is saved as, if it is in the menus.
pub fn step_of(cmd: Cmd) -> Option<String> {
    menu_commands().iter().find(|(_, _, _, c)| *c == cmd).map(|(menu, name, _, _)| step(menu, name))
}

/// The command a step of a file stands for.
pub fn command_of(line: &str) -> Option<Cmd> {
    let line = line.trim();
    menu_commands().iter().find(|(menu, name, _, _)| step(menu, name).eq_ignore_ascii_case(line)).map(|(_, _, _, cmd)| *cmd)
}

/// The steps written in a command file: every line that is not empty and not a remark.
pub fn steps(text: &str) -> Vec<String> {
    text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).map(str::to_string).collect()
}

pub fn folder() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(APP_NAME.to_lowercase()).join("commands"))
}

/// The saved commands, by name.
pub fn saved() -> Vec<(String, PathBuf)> {
    let Some(entries) = folder().and_then(|folder| std::fs::read_dir(folder).ok()) else { return Vec::new() };
    let mut found: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION)))
        .filter_map(|path| Some((path.file_stem()?.to_string_lossy().into_owned(), path)))
        .collect();
    found.sort();
    found
}

pub fn save(name: &str, steps: &[String]) -> Result<PathBuf> {
    let folder = folder().context("no settings folder")?;
    std::fs::create_dir_all(&folder).with_context(|| tf("could not write {}", &[&folder.display()]))?;
    let path = folder.join(format!("{}.{EXTENSION}", crate::templates::file_name(name).replace("template", "command")));
    let text = format!("# PisiWorks command\n{}\n", steps.join("\n"));
    std::fs::write(&path, text).with_context(|| tf("could not write {}", &[&path.display()]))?;
    Ok(path)
}

pub fn read(path: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path).with_context(|| tf("could not read {}", &[&path.display()]))?;
    Ok(steps(&text))
}

/// Gives a saved command another name. The new name must be free.
pub fn rename(path: &Path, name: &str) -> Result<PathBuf> {
    let target = path.with_file_name(format!("{}.{EXTENSION}", crate::templates::file_name(name).replace("template", "command")));
    anyhow::ensure!(!target.exists() || target == path, tf("could not write {}", &[&target.display()]));
    std::fs::rename(path, &target).with_context(|| tf("could not write {}", &[&target.display()]))?;
    Ok(target)
}

/// The Commands menu with the saved commands at its foot. Menus are made of
/// lasting text, so the list is made anew, and kept, only when the names change.
pub fn menu(fixed: &'static [Entry], names: &[String]) -> &'static [Entry] {
    static MADE: Mutex<Option<(Vec<String>, &'static [Entry])>> = Mutex::new(None);
    if names.is_empty() {
        return fixed;
    }
    let mut made = MADE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((for_names, entries)) = made.as_ref() {
        if for_names == names {
            return entries;
        }
    }
    let mut entries = fixed.to_vec();
    entries.push(Entry::Sep);
    for (index, name) in names.iter().take(u16::MAX as usize).enumerate() {
        entries.push(Entry::Item(Box::leak(name.clone().into_boxed_str()), "", Some(Cmd::RunSaved(index as u16))));
    }
    let entries: &'static [Entry] = Box::leak(entries.into_boxed_slice());
    *made = Some((names.to_vec(), entries));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_is_written_as_its_menu_and_name_and_read_back() {
        let written = step_of(Cmd::FlipH).unwrap();
        assert_eq!(written, "Modify > Flip Horizontal");
        assert_eq!(command_of("  modify > flip horizontal "), Some(Cmd::FlipH));
        assert_eq!(command_of("Modify > No Such Thing"), None);
        // The dots of a command that opens a dialog are not part of its name.
        assert!(!step_of(Cmd::Preferences).unwrap().ends_with('.'));
        assert_eq!(command_of(&step_of(Cmd::Preferences).unwrap()), Some(Cmd::Preferences));
    }

    #[test]
    fn a_file_gives_its_steps_without_remarks_and_empty_lines() {
        assert_eq!(steps("# PisiWorks command\n\nModify > Flip Horizontal\n  Edit > Duplicate  \n"), ["Modify > Flip Horizontal", "Edit > Duplicate"]);
        assert!(steps("").is_empty());
    }

    #[test]
    fn the_menu_grows_by_the_saved_commands() {
        let fixed: &'static [Entry] = &[Entry::Sep];
        assert_eq!(menu(fixed, &[]).len(), 1);
        let with = menu(fixed, &["Ayna".to_string(), "İkile".to_string()]);
        assert_eq!(with.len(), 4);
        assert!(matches!(with[3], Entry::Item("İkile", "", Some(Cmd::RunSaved(1)))));
        // Asked again with the same names, it is the same list.
        assert!(std::ptr::eq(with, menu(fixed, &["Ayna".to_string(), "İkile".to_string()])));
    }
}
