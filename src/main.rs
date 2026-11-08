#![cfg_attr(not(test), windows_subsystem = "windows")]

mod autoshape;
mod bitmap;
mod canvas_view;
mod export;
mod icons;
mod library;
mod menus;
mod model;
mod optimize;
mod panels;
mod paths;
mod theme;
mod ui;
mod workspace;

use gpui::{
    point, prelude::*, px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};

use workspace::Workspace;

fn main() {
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
