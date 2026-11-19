#![cfg_attr(not(test), windows_subsystem = "windows")]
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


use pisiworks::{i18n, icons, settings, theme, workspace};

use gpui::{
    point, prelude::*, px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};

use workspace::Workspace;

/// Text of the panic GPUI raises when it finds no graphics driver it can draw with.
const NO_GRAPHICS_MARKERS: [&str; 2] = ["Unable to init GPU context", "NoSupportedDeviceFound"];

/// Replaces the bare panic of a start without a Vulkan driver with words the user can act on.
fn explain_missing_graphics() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let text = info.to_string();
        if !NO_GRAPHICS_MARKERS.iter().any(|marker| text.contains(marker)) {
            return default(info);
        }
        let message = i18n::t(
            "PisiWorks needs a Vulkan graphics driver, and none was found. Install the mesa-vulkan-drivers package, then start PisiWorks again.",
        );
        eprintln!("{}: {message}", theme::APP_NAME);
        // No window of our own can open without the driver, so the desktop's tools show the message.
        let dialogs: [(&str, &[&str]); 3] =
            [("zenity", &["--error", "--no-wrap", "--title", theme::APP_NAME, "--text"]), ("kdialog", &["--title", theme::APP_NAME, "--error"]), ("xmessage", &["-center"])];
        for (program, arguments) in dialogs {
            if std::process::Command::new(program).args(arguments).arg(message).status().is_ok() {
                break;
            }
        }
        std::process::exit(1);
    }));
}

fn main() {
    explain_missing_graphics();
    // The user's choice wins. Without one, follow the desktop.
    i18n::set_language(settings::Settings::load().language().unwrap_or_else(i18n::system_language));
    Application::new().with_assets(icons::Assets).run(|cx: &mut App| {
        workspace::app_quit_on_close(cx);
        let bounds = Bounds::centered(None, size(px(1360.), px(860.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Maximized(bounds)),
            // The menu bar doubles as the title bar, so the system one is hidden.
            titlebar: Some(TitlebarOptions {
                title: Some(theme::APP_NAME.into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(0.), px(0.))),
            }),
            app_id: Some(theme::APP_NAME.to_string()),
            ..Default::default()
        };
        cx.open_window(options, |window, cx| {
            let view = cx.new(Workspace::new);
            // A file named on the command line is opened, which is how the desktop starts
            // the application when a document is double-clicked.
            // It waits for the first frame, because fitting the page needs the size of the canvas.
            if let Some(path) = std::env::args_os().nth(1).map(std::path::PathBuf::from) {
                let path = std::fs::canonicalize(&path).unwrap_or(path);
                let opener = view.downgrade();
                window.on_next_frame(move |_, cx| {
                    opener
                        .update(cx, |workspace, cx| {
                            workspace.open_file(path);
                            cx.notify();
                        })
                        .ok();
                });
            }
            window.focus(&view.read(cx).focus);
            let handle = view.downgrade();
            window.on_window_should_close(cx, move |window, cx| {
                handle.update(cx, |this, cx| this.request_close(window, cx)).unwrap_or(true)
            });
            view
        })
        .expect("failed to open the main window");
        cx.activate(true);
    });
}
