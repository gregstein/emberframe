// This file is part of PisiWorks.
// Copyright (C) 2026 Murat SAPMAZ
// Portions copyright (c) 2026 Med Amine Belbachir, from Emberframe, under the MIT license:
// see LICENSES/MIT.txt and NOTICE.
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

//! Workspace colors and metrics.
//!
//! The grays and list colors were sampled from a screenshot of the reference workspace.

pub const APP_NAME: &str = "PisiWorks";
pub const APP_MARK: &str = "Pw";
pub const FILE_EXTENSION: &str = "pisi";
/// The extension of the animation program's documents. Either program opens the other's.
pub const SAHNE_EXTENSION: &str = "sahne";

/// Whether this is PisiAnimasyon, the animation program, and not PisiWorks. Set once, as the program starts.
static SAHNE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Makes the program PisiAnimasyon: the same workspace with a timeline under the stage.
pub fn set_sahne() {
    SAHNE.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn is_sahne() -> bool {
    SAHNE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The name of the program that is running.
pub fn app_name() -> &'static str {
    if is_sahne() { "PisiAnimasyon" } else { APP_NAME }
}

/// The two letters at the start of the menu bar.
pub fn app_mark() -> &'static str {
    // Not "Ps": those two letters are the mark of another company's program.
    if is_sahne() { "Pa" } else { APP_MARK }
}

/// The extension new documents are saved with.
pub fn document_extension() -> &'static str {
    if is_sahne() { SAHNE_EXTENSION } else { FILE_EXTENSION }
}
/// The extension documents had before, and Emberframe documents still have. They open and save as before.
pub const LEGACY_EXTENSION: &str = "ember";

pub const PANEL: u32 = 0xd6d6d6;
/// The strip behind panel and document tabs.
pub const PANEL_DARK: u32 = 0xa3a3a3;
/// Buttons, menus and combo boxes.
pub const PANEL_RAISED: u32 = 0xe8e8e8;
pub const BORDER: u32 = 0x7f7f7f;
/// Dividers between the main areas of the window.
pub const DIVIDER: u32 = 0x646464;
pub const SEPARATOR: u32 = 0xbcbcbc;
pub const INPUT: u32 = 0xffffff;
pub const INPUT_BORDER: u32 = 0x8d8d8d;
pub const PASTEBOARD: u32 = 0xc8c8c8;
pub const TEXT: u32 = 0x141414;
pub const TEXT_DIM: u32 = 0x2b2b2b;
pub const TEXT_DISABLED: u32 = 0x8d8d8d;
pub const APP_MARK_COLOR: u32 = 0xa0a0a0;
/// Selected rows and open menus.
pub const ACCENT: u32 = 0xa8c6ed;
pub const HOVER: u32 = 0xf0f0f0;
pub const WINDOW_CLOSE_HOVER: u32 = 0xe81123;
/// Background of lists such as Layers.
pub const LIST: u32 = 0xededed;
pub const LIST_BORDER: u32 = 0xc4c4c4;
/// The Web Layer row, set apart from ordinary layers.
pub const LIST_SPECIAL: u32 = 0xd4d0c8;

pub const SELECTION: u32 = 0x00a8ff;
pub const RULER: u32 = 0xdfdfdf;

pub const FONT_SIZE: f32 = 11.0;

pub const MENU_BAR_HEIGHT: f32 = 32.0;
pub const TOOLBAR_HEIGHT: f32 = 28.0;
pub const TOOLS_WIDTH: f32 = 66.0;
pub const PROPERTIES_HEIGHT: f32 = 116.0;
pub const DOCK_WIDTH: f32 = 308.0;
pub const TAB_HEIGHT: f32 = 20.0;
pub const DOCUMENT_TAB_HEIGHT: f32 = 22.0;
pub const VIEW_STRIP_HEIGHT: f32 = 26.0;
pub const RULER_SIZE: f32 = 16.0;
