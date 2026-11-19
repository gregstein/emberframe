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

//! The examples that come with the program: documents to open, look into and change.
//!
//! Each is built here, from the same commands a user has, when it is asked for.
//! That keeps them in step with the program, lets their texts follow the language
//! of the interface, and means there are no files to install or to lose.

use std::sync::Arc;

use crate::autoshape::AutoKind;
use crate::effects::{FilterKind, LiveFilter};
use crate::i18n::t;
use crate::model::{
    AnimationSettings, Color, Document, Editor, FillKind, FillStyle, Object, Rect, Shape, Stroke, SymbolKind, TextAlign, TextStyle, Vec2, WebKind,
};
use crate::optimize::ExportFormat;

/// An example: its name, what it shows, and how it is made.
pub struct Example {
    pub name: &'static str,
    pub about: &'static str,
    pub build: fn() -> Document,
}

pub const EXAMPLES: &[Example] = &[
    Example { name: "Web banner", about: "Gradient fill, text, a live filter and an Auto Shape", build: banner },
    Example { name: "Navigation buttons", about: "Button symbols with Up, Over and Down states, and links", build: buttons },
    Example { name: "Flying star", about: "An animation symbol across eight states, ready to export as an animated GIF", build: flying_star },
    Example { name: "School poster", about: "Text on a path, a mask and several live filters", build: poster },
    Example { name: "Fills and gradients", about: "The twelve kinds of gradient fill side by side", build: fills },
    Example { name: "Live filters", about: "One shape under eight live filters", build: filters },
    Example { name: "Web page mockup", about: "A grid of guides, layers, slices and hotspots", build: web_page },
];

pub(crate) const WHITE: Color = Color::WHITE;
pub(crate) const INK: Color = Color::rgb(0x1c, 0x24, 0x38);
pub(crate) const BLUE: Color = Color::rgb(0x1e, 0x78, 0xd2);
pub(crate) const TEAL: Color = Color::rgb(0x14, 0xa0, 0x9a);
pub(crate) const ORANGE: Color = Color::rgb(0xf2, 0x8c, 0x1e);
pub(crate) const YELLOW: Color = Color::rgb(0xff, 0xd2, 0x3c);
pub(crate) const PALE: Color = Color::rgb(0xee, 0xf2, 0xf7);

pub(crate) fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

/// Adds a shape to the active layer and selects it. Returns its number.
pub(crate) fn add(e: &mut Editor, name: &str, shape: Shape, bounds: Rect, fill: Option<Color>, stroke: Option<Stroke>) -> u64 {
    let mut object = Object::new_web(e.alloc_id(), WebKind::Slice, bounds);
    (object.name, object.shape, object.fill, object.stroke) = (name.to_string(), shape, fill, stroke);
    let id = object.id;
    e.add_object(object);
    e.selection = vec![id];
    id
}

pub(crate) fn boxed(e: &mut Editor, name: &str, bounds: Rect, radius: f32, fill: Color) -> u64 {
    add(e, name, Shape::Rect { radius }, bounds, Some(fill), None)
}

/// Adds a text whose top left corner is at `x`, `y`.
pub(crate) fn text(e: &mut Editor, content: &str, x: f32, y: f32, size: f32, color: Color, style: TextStyle) -> u64 {
    let id = add(e, content, Shape::Text { content: content.to_string(), size, style }, rect(x, y, 10.0, size), Some(color), None);
    if let Some(object) = e.object_mut(id) {
        object.refit_text();
    }
    id
}

/// A text with its middle at `x`.
pub(crate) fn centered(e: &mut Editor, content: &str, x: f32, y: f32, size: f32, color: Color, style: TextStyle) -> u64 {
    let id = text(e, content, x, y, size, color, TextStyle { align: TextAlign::Center, ..style });
    if let Some(object) = e.object_mut(id) {
        let width = object.bounds.w;
        object.translate(-width / 2.0, 0.0);
    }
    id
}

pub(crate) fn gradient(e: &mut Editor, id: u64, kind: FillKind, from: Color, to: Color, start: (f32, f32), end: (f32, f32)) {
    if let Some(object) = e.object_mut(id) {
        let mut style = FillStyle::new(kind, from, to);
        (style.start, style.end) = (Vec2 { x: start.0, y: start.1 }, Vec2 { x: end.0, y: end.1 });
        object.fill_style = Some(Arc::new(style));
    }
}

pub(crate) fn filter(e: &mut Editor, id: u64, kind: FilterKind, values: &[(usize, f32)]) {
    if let Some(object) = e.object_mut(id) {
        let mut made = LiveFilter::new(kind);
        for (index, value) in values {
            if let Some(slot) = made.values.get_mut(*index) {
                *slot = *value;
            }
        }
        object.filters.push(made);
    }
}

pub(crate) fn bold() -> TextStyle {
    TextStyle { bold: true, ..TextStyle::default() }
}

/// A new layer above the others, with a name, made the active one.
pub(crate) fn layer(e: &mut Editor, name: &str) {
    let top = e.doc.page().layers.len() - 1;
    e.doc.page_mut().active_layer = top;
    e.add_layer();
    let page = e.doc.page_mut();
    page.layers[page.active_layer].name = name.to_string();
}

fn start(width: u32, height: u32, first_layer: &str) -> Editor {
    let mut e = Editor::new(Document::new(width, height));
    e.doc.page_mut().layers[0].name = first_layer.to_string();
    e
}

pub(crate) fn finish(mut e: Editor) -> Document {
    e.leave_symbol();
    e.close_groups();
    e.selection.clear();
    e.doc
}

fn banner() -> Document {
    let mut e = start(728, 90, t("Background"));
    let back = boxed(&mut e, t("Background"), rect(0.0, 0.0, 728.0, 90.0), 0.0, BLUE);
    gradient(&mut e, back, FillKind::Linear, INK, TEAL, (0.0, 0.5), (1.0, 0.5));
    let star = add(&mut e, t("Star"), Shape::Auto { kind: AutoKind::Star, a: 0.45, b: 0.0, count: 5 }, rect(22.0, 13.0, 64.0, 64.0), Some(YELLOW), None);
    filter(&mut e, star, FilterKind::Glow, &[]);
    layer(&mut e, t("Text"));
    let title = text(&mut e, t("Spring Festival"), 108.0, 12.0, 34.0, WHITE, bold());
    filter(&mut e, title, FilterKind::DropShadow, &[(0, 2.0), (1, 60.0), (2, 2.0)]);
    text(&mut e, t("Music, games and exhibitions in the school garden"), 110.0, 56.0, 15.0, PALE, TextStyle::default());
    layer(&mut e, t("Button"));
    let button = boxed(&mut e, t("Button"), rect(590.0, 25.0, 116.0, 40.0), 8.0, ORANGE);
    filter(&mut e, button, FilterKind::DropShadow, &[(0, 2.0), (1, 45.0), (2, 2.0)]);
    centered(&mut e, t("Join in"), 648.0, 33.0, 18.0, WHITE, bold());
    finish(e)
}

fn buttons() -> Document {
    let mut e = start(520, 120, t("Background"));
    boxed(&mut e, t("Background"), rect(0.0, 0.0, 520.0, 120.0), 0.0, PALE);
    layer(&mut e, t("Buttons"));
    let names = [(t("Home"), "index.html"), (t("Lessons"), "dersler.html"), (t("Contact"), "iletisim.html")];
    for (index, (name, link)) in names.into_iter().enumerate() {
        let x = 30.0 + index as f32 * 160.0;
        let symbol = e.new_symbol(name, SymbolKind::Button);
        e.edit_symbol(symbol, None, true);
        // Up: blue. Over: lighter. Down: darker, with the label a step lower.
        boxed(&mut e, t("Button"), rect(x, 40.0, 140.0, 40.0), 8.0, BLUE);
        centered(&mut e, name, x + 70.0, 49.0, 17.0, WHITE, bold());
        for (look, color, nudge) in [(1, Color::rgb(0x4a, 0x9d, 0xee), 0.0), (2, Color::rgb(0x12, 0x55, 0x9c), 1.0)] {
            e.set_symbol_look(look);
            e.copy_previous_look();
            let ids = e.selection.clone();
            for id in ids {
                if let Some(object) = e.object_mut(id) {
                    match object.shape {
                        Shape::Rect { .. } => object.fill = Some(color),
                        _ => object.translate(0.0, nudge),
                    }
                }
            }
        }
        e.set_symbol_look(0);
        e.leave_symbol();
        e.for_selected(|o| o.link = link.to_string());
    }
    finish(e)
}

fn flying_star() -> Document {
    let mut e = start(480, 300, t("Background"));
    let sky = boxed(&mut e, t("Sky"), rect(0.0, 0.0, 480.0, 300.0), 0.0, INK);
    gradient(&mut e, sky, FillKind::Linear, INK, Color::rgb(0x3a, 0x2a, 0x6a), (0.5, 0.0), (0.5, 1.0));
    // The sky is the same in every state; only the star's layer changes.
    e.doc.page_mut().set_layer_shared(0, true);
    layer(&mut e, t("Star"));
    let star = add(&mut e, t("Star"), Shape::Auto { kind: AutoKind::Star, a: 0.45, b: 0.0, count: 5 }, rect(30.0, 200.0, 70.0, 70.0), Some(YELLOW), None);
    e.selection = vec![star];
    e.animate_selection(AnimationSettings { states: 8, distance: 380.0, direction: 25.0, scale: 45.0, opacity_from: 100.0, opacity_to: 30.0, rotation: 360.0, clockwise: true });
    let page = e.doc.page_mut();
    page.export.format = ExportFormat::AnimatedGif;
    for state in &mut page.states {
        state.delay_ms = 90;
    }
    finish(e)
}

fn poster() -> Document {
    let mut e = start(420, 594, t("Background"));
    let back = boxed(&mut e, t("Background"), rect(0.0, 0.0, 420.0, 594.0), 0.0, PALE);
    gradient(&mut e, back, FillKind::Linear, WHITE, Color::rgb(0xc8, 0xdc, 0xf4), (0.5, 0.0), (0.5, 1.0));
    // A band that fades away to the right, by a mask of its own size.
    boxed(&mut e, t("Band"), rect(0.0, 430.0, 420.0, 70.0), 0.0, TEAL);
    e.auto_vector_mask(2);
    layer(&mut e, t("Picture"));
    let sun = add(&mut e, t("Sun"), Shape::Ellipse, rect(110.0, 120.0, 200.0, 200.0), Some(ORANGE), None);
    gradient(&mut e, sun, FillKind::Radial, YELLOW, ORANGE, (0.5, 0.5), (1.0, 0.5));
    filter(&mut e, sun, FilterKind::Glow, &[]);
    let gear = add(&mut e, t("Gear"), Shape::Auto { kind: AutoKind::Gear, a: 0.7, b: 0.0, count: 12 }, rect(170.0, 180.0, 80.0, 80.0), Some(WHITE), None);
    filter(&mut e, gear, FilterKind::InnerBevel, &[]);
    filter(&mut e, gear, FilterKind::DropShadow, &[(0, 3.0), (2, 3.0)]);
    layer(&mut e, t("Text"));
    // The words run round the sun: a text and a circle, joined by Attach to Path.
    let around = text(&mut e, t("Science and Technology Week"), 0.0, 0.0, 22.0, INK, bold());
    let circle = add(&mut e, t("Path"), Shape::Ellipse, rect(92.0, 102.0, 236.0, 236.0), None, Some(Stroke { color: INK, width: 1.0 }));
    e.selection = vec![around, circle];
    e.attach_text_to_path();
    let title = centered(&mut e, t("Open Day"), 210.0, 360.0, 44.0, BLUE, bold());
    filter(&mut e, title, FilterKind::DropShadow, &[(0, 2.0), (1, 40.0), (2, 2.0)]);
    centered(&mut e, t("Friday, 2 pm, in the conference hall"), 210.0, 452.0, 18.0, WHITE, bold());
    centered(&mut e, t("Everyone is welcome"), 210.0, 530.0, 16.0, INK, TextStyle::default());
    finish(e)
}

fn fills() -> Document {
    let kinds = [
        FillKind::Linear,
        FillKind::Radial,
        FillKind::Ellipse,
        FillKind::Rectangle,
        FillKind::Cone,
        FillKind::Contour,
        FillKind::Satin,
        FillKind::Starburst,
        FillKind::Folds,
        FillKind::Bars,
        FillKind::Ripples,
        FillKind::Waves,
    ];
    let mut e = start(680, 470, t("Fills"));
    for (index, kind) in kinds.into_iter().enumerate() {
        let (x, y) = (30.0 + (index % 4) as f32 * 160.0, 30.0 + (index / 4) as f32 * 145.0);
        let swatch = boxed(&mut e, kind.label(), rect(x, y, 140.0, 100.0), 6.0, BLUE);
        let (from, to) = kind.start_handles();
        gradient(&mut e, swatch, kind, BLUE, YELLOW, (from.x, from.y), (to.x, to.y));
        centered(&mut e, kind.label(), x + 70.0, y + 106.0, 14.0, INK, TextStyle::default());
    }
    finish(e)
}

fn filters() -> Document {
    let kinds = [
        FilterKind::DropShadow,
        FilterKind::Glow,
        FilterKind::InnerBevel,
        FilterKind::RaisedEmboss,
        FilterKind::InnerShadow,
        FilterKind::GaussianBlur,
        FilterKind::HueSaturation,
        FilterKind::Invert,
    ];
    let mut e = start(680, 340, t("Filters"));
    for (index, kind) in kinds.into_iter().enumerate() {
        let (x, y) = (40.0 + (index % 4) as f32 * 160.0, 30.0 + (index / 4) as f32 * 155.0);
        let gear = add(&mut e, kind.label(), Shape::Auto { kind: AutoKind::Gear, a: 0.7, b: 0.0, count: 10 }, rect(x + 15.0, y, 100.0, 100.0), Some(ORANGE), None);
        // Hue/Saturation would change nothing as it comes; here it turns the orange toward green.
        let values: &[(usize, f32)] = if kind == FilterKind::HueSaturation { &[(0, 90.0)] } else { &[] };
        filter(&mut e, gear, kind, values);
        centered(&mut e, kind.label(), x + 65.0, y + 112.0, 14.0, INK, TextStyle::default());
    }
    finish(e)
}

fn web_page() -> Document {
    let grid = crate::templates::BUILT_IN.iter().find(|template| template.columns == 12 && template.width == 960);
    let mut e = Editor::new(grid.map_or_else(|| Document::new(960, 800), |template| template.document()));
    e.doc.page_mut().layers[0].name = t("Background").to_string();
    boxed(&mut e, t("Background"), rect(0.0, 0.0, 960.0, 800.0), 0.0, WHITE);
    layer(&mut e, t("Header"));
    let header = boxed(&mut e, t("Header"), rect(0.0, 0.0, 960.0, 90.0), 0.0, INK);
    gradient(&mut e, header, FillKind::Linear, INK, BLUE, (0.0, 0.5), (1.0, 0.5));
    text(&mut e, t("Our School"), 30.0, 26.0, 30.0, WHITE, bold());
    let pages = [(t("Home"), "index.html"), (t("Lessons"), "dersler.html"), (t("Gallery"), "galeri.html"), (t("Contact"), "iletisim.html")];
    let mut links = Vec::new();
    for (index, (name, link)) in pages.into_iter().enumerate() {
        let id = text(&mut e, name, 570.0 + index as f32 * 95.0, 36.0, 16.0, PALE, TextStyle::default());
        links.push((id, link));
    }
    layer(&mut e, t("Content"));
    let hero = boxed(&mut e, t("Picture"), rect(10.0, 110.0, 940.0, 260.0), 0.0, TEAL);
    gradient(&mut e, hero, FillKind::Linear, TEAL, YELLOW, (0.0, 0.0), (1.0, 1.0));
    centered(&mut e, t("Welcome to our school's web page"), 480.0, 215.0, 32.0, WHITE, bold());
    let mut cards = Vec::new();
    for (index, title) in [t("News"), t("Events"), t("Projects")].into_iter().enumerate() {
        let x = 10.0 + index as f32 * 320.0;
        let card = boxed(&mut e, title, rect(x, 400.0, 300.0, 220.0), 6.0, PALE);
        filter(&mut e, card, FilterKind::DropShadow, &[(0, 2.0), (1, 30.0), (2, 3.0)]);
        cards.push(card);
        text(&mut e, title, x + 20.0, 418.0, 22.0, BLUE, bold());
        text(&mut e, t("Write a few lines here."), x + 20.0, 456.0, 15.0, INK, TextStyle::default());
    }
    let footer = boxed(&mut e, t("Footer"), rect(0.0, 740.0, 960.0, 60.0), 0.0, INK);
    centered(&mut e, t("Made with PisiWorks"), 480.0, 760.0, 14.0, PALE, TextStyle::default());
    // The picture and the cards are cut out as slices; the menu entries are hotspots that lead somewhere.
    e.selection = std::iter::once(hero).chain(cards).chain([footer]).collect();
    e.insert_web(WebKind::Slice, false);
    for (id, link) in links {
        e.selection = vec![id];
        e.insert_web(WebKind::Hotspot, false);
        e.for_selected(|o| {
            if let Shape::Web { url, .. } = &mut o.shape {
                *url = link.to_string();
            }
        });
    }
    finish(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_example_builds_draws_and_survives_saving() {
        for example in EXAMPLES {
            let doc = (example.build)();
            let page = &doc.pages[0];
            assert!(page.objects().count() >= 2, "{} is nearly empty", example.name);
            assert!(page.symbol_edit.is_none() && page.open_groups.is_empty(), "{} was left open", example.name);
            // Nothing lies wholly off the page.
            for object in page.objects() {
                assert!(page.rect().intersect(object.bounds).is_some(), "{}: {} is off the page", example.name, object.name);
            }
            crate::export::render_page(page).unwrap();
            let again = crate::model::document_from_json(&crate::model::document_to_json(&doc).unwrap()).unwrap();
            assert_eq!(again.pages[0].objects().count(), page.objects().count(), "{}", example.name);
        }
    }

    #[test]
    fn the_examples_export_as_they_promise() {
        let find = |name: &str| Editor::new((EXAMPLES.iter().find(|e| e.name == name).unwrap().build)());
        // The star goes out as an animated GIF of eight pictures.
        let star = find("Flying star").export_page();
        let bytes = crate::optimize::encode(&star, &star.export).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::Indexed);
        let mut reader = options.read_info(&bytes[..]).unwrap();
        let mut frames = 0;
        while reader.read_next_frame().unwrap().is_some() {
            frames += 1;
        }
        assert_eq!(frames, 8);
        // The buttons go out with a picture for each look and a page that swaps them.
        let dir = std::env::temp_dir().join(format!("pisiworks-examples-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let buttons = find("Navigation buttons").export_page();
        assert_eq!(crate::export::export_web(&buttons, &dir.join("menu.png")).unwrap(), 10);
        let html = std::fs::read_to_string(dir.join("menu.html")).unwrap();
        assert!(html.contains("href=\"dersler.html\"") && html.contains("menu_button3_down.png"), "{html}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_examples_show_what_they_say() {
        let find = |name: &str| (EXAMPLES.iter().find(|e| e.name == name).unwrap().build)();
        // Three buttons, each a symbol of its own with three looks and a link.
        let doc = find("Navigation buttons");
        assert_eq!(doc.symbols.len(), 3);
        assert!(doc.symbols.iter().all(|s| s.kind == SymbolKind::Button && s.looks.len() == 2 && !s.objects.is_empty()));
        let placed: Vec<&Object> = doc.pages[0].objects().filter(|o| matches!(o.shape, Shape::Instance { .. })).collect();
        assert_eq!(placed.len(), 3);
        assert!(placed.iter().all(|o| o.link.ends_with(".html")));
        // The star is in all eight states, and the sky shows in each of them.
        let doc = find("Flying star");
        let page = &doc.pages[0];
        assert_eq!((page.states.len(), page.animations.len(), page.export.format), (8, 1, ExportFormat::AnimatedGif));
        assert!((0..8).all(|state| page.at_state(state).objects().count() == 2));
        // The poster's words follow a path, and its band is masked.
        let doc = find("School poster");
        assert!(doc.pages[0].objects().any(|o| matches!(o.shape, Shape::Text { .. }) && o.points.len() >= 2));
        assert!(doc.pages[0].objects().any(|o| o.mask.is_some()));
        // The mockup has its guides, five slices and four hotspots with addresses.
        let doc = find("Web page mockup");
        let page = &doc.pages[0];
        assert_eq!(page.guides.vertical.len(), 24);
        let web = |kind: WebKind| page.objects().filter(|o| matches!(&o.shape, Shape::Web { kind: k, .. } if *k == kind)).count();
        assert_eq!((web(WebKind::Slice), web(WebKind::Hotspot)), (5, 4));
        assert!(page.objects().all(|o| !matches!(&o.shape, Shape::Web { kind: WebKind::Hotspot, url, .. } if url.is_empty())));
    }
}
