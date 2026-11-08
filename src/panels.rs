//! Bodies of the dockable panels added on top of the core workspace.

use gpui::{
    anchored, canvas, deferred, div, fill, point, prelude::*, px, rgb, size, AnyElement, Bounds, Context, Div, FontWeight,
    MouseButton, MouseDownEvent, SharedString, Stateful, Window,
};

use crate::autoshape::AutoKind;
use crate::canvas_view::paint_thumbnail;
use crate::library::common_items;
use crate::menus::{Cmd, Panel, COLOR_TOOLS, DOCK_GROUPS, TOOL_SECTIONS, ZOOM_LEVELS};
use crate::model::{Align, Color, Object, Rect, Shape, Stroke};
use crate::bitmap::Filter;
use crate::optimize::{format_size, ExportFormat, PRESETS};
use crate::paths::PathOp;
use crate::theme::*;
use crate::ui::{color_icon, color_u32, icon, label, tip};
use crate::workspace::{ColorTarget, Combo, FieldId, FindMode, SizeDialogKind, Workspace};

const TILE: f32 = 44.0;
/// Width of a tile with its name underneath.
const CAPTIONED_TILE: f32 = 60.0;
const SPECIAL_CHARACTERS: &str = "©®™€£¥¢§¶•…–—«»‹›°±×÷µ¼½¾¡¿ÀÉÑÜßæøπΩ→←↑↓✓★♥";
/// Color steps drawn in a Color Mixer ramp.
const RAMP_STEPS: usize = 32;
const PALETTE_HUES: usize = 24;
const PALETTE_ROWS: usize = 7;
const PALETTE_CELL: f32 = 10.5;
const ALIGN_ITEMS: [(&str, Align); 6] = [
    ("Left", Align::Left),
    ("Center Horizontally", Align::CenterX),
    ("Right", Align::Right),
    ("Top", Align::Top),
    ("Center Vertically", Align::CenterY),
    ("Bottom", Align::Bottom),
];

type Action = dyn Fn(&mut Workspace, &MouseDownEvent, &mut Window, &mut Context<Workspace>);

fn style_preview(fill: Option<Color>, stroke: Option<Stroke>, opacity: f32) -> Object {
    Object {
        id: 0,
        name: String::new(),
        shape: Shape::Rect { radius: 5.0 },
        bounds: Rect { x: 0.0, y: 0.0, w: 32.0, h: 32.0 },
        points: Vec::new(),
        fill,
        stroke,
        opacity,
        visible: true,
        locked: false,
    }
}

fn auto_preview(kind: AutoKind) -> Object {
    let d = kind.defaults();
    Object {
        shape: Shape::Auto { kind, a: d.a, b: d.b, count: d.count },
        ..style_preview(Some(Color::rgb(0x4f, 0x8f, 0xd8)), Some(Stroke { color: Color::rgb(0x24, 0x52, 0x8c), width: 1.0 }), 1.0)
    }
}

fn heading(text: impl Into<SharedString>) -> Div {
    div().h(px(16.)).px_1().flex().flex_none().items_center().text_color(rgb(TEXT_DIM)).border_b_1().border_color(rgb(SEPARATOR)).child(text.into())
}

/// Converts hue (0..360), saturation and value (0..1) to a color.
fn from_hsv(hue: f32, saturation: f32, value: f32) -> Color {
    let sector = (hue.rem_euclid(360.0)) / 60.0;
    let chroma = value * saturation;
    let x = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let channel = |c: f32| (((c + value - chroma) * 255.0).round()) as u8;
    Color::rgb(channel(r), channel(g), channel(b))
}

/// Returns the hue, saturation and value of a color.
fn to_hsv(color: Color) -> (f32, f32, f32) {
    let (r, g, b) = (color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0);
    let max = r.max(g).max(b);
    let delta = max - r.min(g).min(b);
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (hue, if max == 0.0 { 0.0 } else { delta / max }, max)
}

fn labelled_row(name: impl Into<SharedString>, width: f32, control: impl IntoElement) -> Div {
    div().min_h(px(22.)).flex().flex_none().items_center().gap_1().child(div().w(px(width)).flex_none().child(label(name))).child(control)
}

impl Workspace {
    pub(crate) fn text_button(
        &self,
        text: impl Into<SharedString>,
        enabled: bool,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Div {
        div()
            .h(px(20.))
            .px_2()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .bg(rgb(PANEL_RAISED))
            .border_1()
            .border_color(rgb(BORDER))
            .text_color(rgb(if enabled { TEXT } else { TEXT_DISABLED }))
            .when(enabled, |d| {
                d.hover(|s| s.bg(rgb(HOVER))).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        this.finish_pending();
                        action(this, window, cx);
                        cx.notify();
                    }),
                )
            })
            .child(text.into())
    }

    fn icon_button(
        &self,
        name: &'static str,
        hint: &'static str,
        active: bool,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        div()
            .id(name)
            .size(px(20.))
            .flex()
            .items_center()
            .justify_center()
            .tooltip(tip(hint))
            .when(active, |d| d.bg(rgb(PANEL_RAISED)))
            .hover(|s| s.bg(rgb(HOVER)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.finish_pending();
                    action(this, cx);
                    cx.notify();
                }),
            )
            .child(color_icon(name, 13., true))
    }

    fn footer() -> Div {
        div().h(px(24.)).flex().flex_none().items_center().justify_end().px_1().gap_1().bg(rgb(PANEL)).border_t_1().border_color(rgb(SEPARATOR))
    }

    /// A combo box for a setting that cannot be changed yet.
    pub(crate) fn disabled_combo(&self, value: impl Into<SharedString>, width: f32) -> Div {
        div()
            .w(px(width))
            .h(px(18.))
            .px_1()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .bg(rgb(PANEL_RAISED))
            .border_1()
            .border_color(rgb(SEPARATOR))
            .text_color(rgb(TEXT_DISABLED))
            .child(value.into())
            .child(icon("chevron-down", 8., TEXT_DISABLED))
    }

    /// A tile with its name below it.
    fn captioned_tile(
        &self,
        id: impl Into<SharedString>,
        caption: SharedString,
        objects: Vec<Object>,
        cx: &mut Context<Self>,
        action: Box<Action>,
    ) -> Div {
        div()
            .w(px(CAPTIONED_TILE + 3.))
            .flex()
            .flex_none()
            .flex_col()
            .items_center()
            .child(self.tile(id, caption.clone(), objects, false, cx, action).size(px(CAPTIONED_TILE)))
            .child(div().w_full().h(px(14.)).px(px(2.)).text_center().truncate().text_size(px(9.)).text_color(rgb(TEXT_DIM)).child(caption))
    }

    /// A list row's name, or its rename field while that is being edited.
    pub(crate) fn name_or_field(&self, id: FieldId, name: String, width: f32, cx: &mut Context<Self>) -> AnyElement {
        if self.field.as_ref().is_some_and(|f| f.id == id) {
            self.field(id, width, cx).into_any_element()
        } else {
            div().flex_1().min_w_0().overflow_hidden().child(name).into_any_element()
        }
    }

    fn tile(
        &self,
        id: impl Into<SharedString>,
        hint: impl Into<SharedString>,
        objects: Vec<Object>,
        selected: bool,
        cx: &mut Context<Self>,
        action: Box<Action>,
    ) -> Stateful<Div> {
        let id: SharedString = id.into();
        div()
            .id(id)
            .size(px(TILE))
            .flex_none()
            .m(px(1.))
            .bg(rgb(0xf4f4f4))
            .border_1()
            .border_color(rgb(if selected { SELECTION } else { BORDER }))
            .tooltip(tip(hint.into()))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, ev, window, cx| {
                this.finish_pending();
                action(this, ev, window, cx);
                cx.notify();
            }))
            .child(canvas(|_, _, _| (), move |bounds, _, window, cx| paint_thumbnail(&objects, bounds, window, cx)).size_full())
    }

    // ---- Optimize -------------------------------------------------------------------

    pub(crate) fn combo(&self, combo: Combo, value: impl Into<SharedString>, width: f32, cx: &mut Context<Self>) -> Div {
        let value: SharedString = value.into();
        div()
            .w(px(width))
            .h(px(18.))
            .px_1()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .overflow_hidden()
            .bg(rgb(INPUT))
            .border_1()
            .border_color(rgb(INPUT_BORDER))
            .hover(|s| s.bg(rgb(HOVER)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    this.finish_pending();
                    this.keep_overlays = true;
                    this.open_combo = if this.open_combo.is_some_and(|(open, _)| open == combo) { None } else { Some((combo, ev.position)) };
                    cx.notify();
                }),
            )
            .child(div().min_w_0().overflow_hidden().child(value))
            .child(icon("chevron-down", 8., TEXT_DIM))
    }

    pub(crate) fn combo_popup(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (combo, position) = self.open_combo?;
        type ItemAction = Box<dyn Fn(&mut Workspace, &mut Window, &mut Context<Workspace>)>;
        let item = |text: SharedString, chosen: bool, cx: &mut Context<Self>, action: ItemAction| {
            div()
                .h(px(20.))
                .px_3()
                .flex()
                .flex_none()
                .items_center()
                .when(chosen, |d| d.bg(rgb(ACCENT)))
                .hover(|s| s.bg(rgb(ACCENT)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        this.open_combo = None;
                        action(this, window, cx);
                        cx.notify();
                    }),
                )
                .child(text)
                .into_any_element()
        };
        let page = self.ed.doc.page();
        let settings = page.export;
        let items: Vec<AnyElement> = match combo {
            Combo::Format => ExportFormat::ALL
                .iter()
                .map(|format| {
                    let format = *format;
                    item(format.label().into(), settings.format == format, cx, Box::new(move |this, _, _| this.set_export(|s| s.format = format)))
                })
                .collect(),
            Combo::Transparency => [("No Transparency", false), ("Index Transparency", true)]
                .into_iter()
                .map(|(text, value)| item(text.into(), settings.transparent == value, cx, Box::new(move |this, _, _| this.set_export(|s| s.transparent = value))))
                .collect(),
            Combo::Preset => PRESETS
                .iter()
                .map(|(name, preset)| {
                    let preset = *preset;
                    item((*name).into(), settings == preset, cx, Box::new(move |this, _, _| this.set_export(|s| *s = preset)))
                })
                .collect(),
            Combo::Zoom => ZOOM_LEVELS
                .iter()
                .map(|level| {
                    let level = *level;
                    let current = (self.zoom * 100.0).round() as u32 == level;
                    item(format!("{level}%").into(), current, cx, Box::new(move |this, window, cx| this.run(Cmd::ZoomTo(level), window, cx)))
                })
                .collect(),
            Combo::WorkspaceMode => [("Expanded Mode", false), ("Collapsed Mode", true)]
                .into_iter()
                .map(|(text, compact)| item(text.into(), self.compact_workspace == compact, cx, Box::new(move |this, _, _| this.set_compact_workspace(compact))))
                .collect(),
            Combo::Page => self
                .ed
                .doc
                .pages
                .iter()
                .enumerate()
                .map(|(index, page)| item(page.name.clone().into(), index == self.ed.doc.active_page, cx, Box::new(move |this, _, _| this.ed.set_active_page(index))))
                .collect(),
            Combo::State => page
                .states
                .iter()
                .enumerate()
                .map(|(index, state)| item(state.name.clone().into(), index == page.active_state, cx, Box::new(move |this, _, _| this.go_to_state(index))))
                .collect(),
            Combo::Align => ALIGN_ITEMS
                .into_iter()
                .map(|(text, op)| {
                    item(
                        text.into(),
                        false,
                        cx,
                        Box::new(move |this, window, cx| {
                            if this.is_enabled(Cmd::Align(op)) {
                                this.run(Cmd::Align(op), window, cx);
                            }
                        }),
                    )
                })
                .collect(),
            Combo::PanelMenu(group) => {
                let collapse = if self.groups[group].collapsed { "Expand Panel Group" } else { "Collapse Panel Group" };
                vec![
                    item(collapse.into(), false, cx, Box::new(move |this, _, _| this.groups[group].collapsed = !this.groups[group].collapsed)),
                    item(
                        "Close Panel Group".into(),
                        false,
                        cx,
                        Box::new(move |this, _, _| {
                            for panel in DOCK_GROUPS[group] {
                                if this.panel_open(*panel) {
                                    this.hidden_panels.push(*panel);
                                }
                            }
                        }),
                    ),
                ]
            }
        };
        Some(
            deferred(
                anchored().position(position).snap_to_window().child(
                    div().occlude().w(px(170.)).py_1().flex().flex_col().bg(rgb(PANEL_RAISED)).border_1().border_color(rgb(BORDER)).shadow_md().children(items),
                ),
            )
            .with_priority(2)
            .into_any_element(),
        )
    }

    pub(crate) fn optimize_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let settings = self.ed.doc.page().export;
        let format = settings.format;
        let size_text = match (&self.estimate, &self.estimate_error) {
            (_, Some(error)) => error.clone(),
            (Some(bytes), _) => format!("Estimated size: {}", format_size(*bytes)),
            _ => "Estimating size...".to_string(),
        };
        let animated_note = (format == ExportFormat::AnimatedGif).then(|| {
            let frames = self.ed.doc.page().states.iter().filter(|s| s.exported).count();
            label(format!("{frames} frame(s) from the States panel"))
        });
        let preset = PRESETS.iter().find(|(_, p)| *p == settings).map_or("Custom", |(name, _)| *name);
        let greyed = |text: &'static str| div().text_color(rgb(TEXT_DISABLED)).child(text);
        let palette: Vec<_> = self.estimate_palette.iter().map(|color| Self::palette_chip(*color)).collect();
        let row = || div().min_h(px(23.)).flex().flex_none().items_center().gap_1();
        div()
            .size_full()
            .p_1()
            .flex()
            .flex_col()
            .child(row().child(self.combo(Combo::Preset, preset, 256., cx)))
            .child(
                row()
                    .justify_between()
                    .child(self.combo(Combo::Format, format.label(), 140., cx))
                    .when(format != ExportFormat::Png32, |d| {
                        d.child(div().flex().items_center().gap_1().child(label("Matte:")).child(self.swatch(ColorTarget::Matte, 18., cx)))
                    }),
            )
            .when(format.is_indexed(), |d| {
                d.child(
                    row()
                        .justify_between()
                        .child(self.combo(Combo::Transparency, if settings.transparent { "Index Transparency" } else { "No Transparency" }, 140., cx))
                        .child(div().flex().items_center().gap_1().child(label("Colors:")).child(self.field(FieldId::OptimizeColors, 44., cx))),
                )
            })
            .when(format == ExportFormat::Jpeg, |d| {
                d.child(row().child(label("Quality:")).child(self.field(FieldId::OptimizeQuality, 44., cx)).child(label("%")))
                    .child(row().justify_between().child(greyed("Selective quality: 90")).child(greyed("Smoothing: 0")))
            })
            .children(animated_note)
            .child(
                div()
                    .id("color-table")
                    .flex_1()
                    .min_h_0()
                    .p(px(2.))
                    .overflow_y_scroll()
                    .bg(rgb(LIST))
                    .border_1()
                    .border_color(rgb(LIST_BORDER))
                    .child(div().flex().flex_wrap().children(palette)),
            )
            .child(
                row()
                    .justify_between()
                    .child(greyed("Sort by: None"))
                    .child(label(if format.is_indexed() { format!("{} colors", self.estimate_palette.len()) } else { String::new() })),
            )
            .child(
                row()
                    .justify_between()
                    .child(label(size_text))
                    .child(self.text_button("Export...", true, cx, |this, window, cx| this.run(Cmd::Export, window, cx))),
            )
            .into_any_element()
    }

    fn palette_chip(color: Color) -> Div {
        div().size(px(12.)).m(px(1.)).flex_none().bg(rgb(color_u32(color)))
    }

    // ---- Color Palette ----------------------------------------------------------------

    fn palette_cell(color: Color, size: f32, cx: &mut Context<Self>) -> Div {
        div().size(px(size)).flex_none().bg(rgb(color_u32(color))).on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                this.finish_pending();
                let target = if ev.modifiers.shift { ColorTarget::Stroke } else { ColorTarget::Fill };
                this.apply_color(target, Some(color));
                cx.notify();
            }),
        )
    }

    pub(crate) fn color_palette_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut grid = Vec::with_capacity(PALETTE_HUES * PALETTE_ROWS);
        for row in 0..PALETTE_ROWS {
            for hue in 0..PALETTE_HUES {
                let hue = hue as f32 * 360.0 / PALETTE_HUES as f32;
                let t = row as f32 / (PALETTE_ROWS - 1) as f32;
                // Tints at the top, full color in the middle, shades at the bottom.
                let (saturation, value) = if t < 0.5 { (0.2 + 1.6 * t, 1.0) } else { (1.0, 1.0 - 1.4 * (t - 0.5)) };
                grid.push(Self::palette_cell(from_hsv(hue, saturation, value), PALETTE_CELL, cx));
            }
        }
        let base = self.current_color(ColorTarget::Fill).unwrap_or(Color::rgb(0xff, 0x66, 0x00));
        let (hue, saturation, value) = to_hsv(base);
        let harmony = |name: &'static str, offsets: &[f32], cx: &mut Context<Self>| {
            div()
                .h(px(20.))
                .flex()
                .flex_none()
                .items_center()
                .gap_1()
                .child(div().w(px(84.)).child(label(name)))
                .children(offsets.iter().map(|offset| Self::palette_cell(from_hsv(hue + offset, saturation, value), 16., cx)).collect::<Vec<_>>())
        };
        div()
            .id("color-palette")
            .size_full()
            .p_1()
            .flex()
            .flex_col()
            .gap_1()
            .overflow_y_scroll()
            .child(div().w(px(PALETTE_CELL * PALETTE_HUES as f32 + 1.)).flex().flex_none().flex_wrap().children(grid))
            .child(harmony("Complementary", &[0., 180.], cx))
            .child(harmony("Analogous", &[-30., 0., 30.], cx))
            .child(harmony("Triad", &[0., 120., 240.], cx))
            .into_any_element()
    }

    // ---- Symbol Properties --------------------------------------------------------------

    pub(crate) fn symbol_properties_body(&self, cx: &mut Context<Self>) -> Div {
        let shell = div().flex_1().min_h_0().p_2().flex().items_start().gap_4().bg(rgb(PANEL));
        let selected = self.ed.selected();
        let symbol = selected.iter().find_map(|o| match o.shape {
            Shape::Instance { symbol, .. } => Some(symbol),
            _ => None,
        });
        let Some(symbol) = symbol.filter(|id| self.ed.doc.symbols.iter().any(|s| s.id == *id)) else {
            return shell.child(label("Select a symbol copy to see its properties"));
        };
        let copies = self
            .ed
            .doc
            .pages
            .iter()
            .flat_map(|page| page.objects())
            .filter(|o| matches!(o.shape, Shape::Instance { symbol: placed, .. } if placed == symbol))
            .count();
        shell
            .child(labelled_row("Symbol:", 46., self.field(FieldId::RenameSymbol(symbol), 150., cx)))
            .child(div().h(px(22.)).flex().items_center().child(label(format!("{copies} cop{} in this document", if copies == 1 { "y" } else { "ies" }))))
            .child(self.text_button("Break Apart", true, cx, |this, window, cx| this.run(Cmd::BreakApart, window, cx)))
    }

    // ---- Size dialog ----------------------------------------------------------------------

    pub(crate) fn size_dialog_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let dialog = self.size_dialog?;
        let is_image = dialog.kind == SizeDialogKind::Image;
        let title = if is_image { "Image Size" } else { "Canvas Size" };
        Some(
            deferred(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .occlude()
                    .child(
                        div()
                            .w(px(240.))
                            .p_3()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .bg(rgb(PANEL))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .shadow_lg()
                            .child(div().font_weight(FontWeight::BOLD).child(title))
                            .child(labelled_row("Width:", 50., div().flex().items_center().gap_1().child(self.field(FieldId::DialogW, 70., cx)).child(label("pixels"))))
                            .child(labelled_row("Height:", 50., div().flex().items_center().gap_1().child(self.field(FieldId::DialogH, 70., cx)).child(label("pixels"))))
                            .when(is_image, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                this.commit_field();
                                                if let Some(dialog) = this.size_dialog.as_mut() {
                                                    dialog.constrain = !dialog.constrain;
                                                }
                                                cx.notify();
                                            }),
                                        )
                                        .child(
                                            div()
                                                .size(px(13.))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .bg(rgb(INPUT))
                                                .border_1()
                                                .border_color(rgb(INPUT_BORDER))
                                                .when(dialog.constrain, |d| d.child(icon("check", 9., TEXT))),
                                        )
                                        .child("Constrain Proportions"),
                                )
                            })
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap_1()
                                    .child(self.text_button("OK", true, cx, |this, _, _| this.apply_size_dialog()))
                                    .child(self.text_button("Cancel", true, cx, |this, _, _| {
                                        this.field = None;
                                        this.size_dialog = None;
                                    })),
                            ),
                    ),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }

    // ---- States ---------------------------------------------------------------------

    pub(crate) fn states_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let page = self.ed.doc.page();
        let active = page.active_state;
        let rows: Vec<_> = page
            .states
            .iter()
            .enumerate()
            .map(|(i, state)| {
                Self::list_row()
                    .when(i == active, |d| d.bg(rgb(ACCENT)))
                    .when(i != active, |d| d.hover(|s| s.bg(rgb(HOVER))))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                            // A click on the delay field has already started editing it.
                            if this.keep_overlays {
                                return;
                            }
                            this.finish_pending();
                            this.stop_playing();
                            this.ed.set_active_state(i);
                            if ev.click_count >= 2 {
                                this.begin_field(FieldId::RenameState(i));
                            }
                            cx.notify();
                        }),
                    )
                    .child(Self::toggle("eye", state.exported).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let state = &mut this.ed.doc.page_mut().states[i];
                            state.exported = !state.exported;
                            this.ed.commit("State Export");
                            cx.notify();
                        }),
                    ))
                    .child(div().w(px(14.)).flex_none().text_color(rgb(TEXT_DIM)).child(format!("{}", i + 1)))
                    .child(self.name_or_field(FieldId::RenameState(i), state.name.clone(), 120., cx))
                    .child(self.field(FieldId::StateDelay(i), 40., cx))
                    .child(label("ms"))
            })
            .collect();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().id("states").flex_1().min_h_0().flex().flex_col().overflow_y_scroll().bg(rgb(LIST)).children(rows))
            .child(
                Self::footer()
                    .child(self.icon_button(if self.playing { "stop" } else { "play" }, "Play or stop the animation", self.playing, cx, |this, cx| this.toggle_play(cx)))
                    .child(self.icon_button("onion", "Onion skinning: show neighbouring states", self.onion, cx, |this, _| this.onion = !this.onion))
                    .child(self.icon_button("duplicate", "Duplicate State", false, cx, |this, _| {
                        this.stop_playing();
                        this.ed.selection.clear();
                        this.ed.doc.page_mut().duplicate_state();
                        this.ed.commit("Duplicate State");
                    }))
                    .child(self.icon_button("plus", "New State", false, cx, |this, _| {
                        this.stop_playing();
                        this.ed.selection.clear();
                        this.ed.doc.page_mut().add_state();
                        this.ed.commit("New State");
                    }))
                    .child(self.icon_button("minus", "Delete State", false, cx, |this, _| {
                        this.stop_playing();
                        this.ed.selection.clear();
                        if this.ed.doc.page_mut().delete_state() {
                            this.ed.commit("Delete State");
                        } else {
                            this.status = "A page needs at least one state".into();
                        }
                    })),
            )
            .into_any_element()
    }

    // ---- Styles ---------------------------------------------------------------------

    pub(crate) fn styles_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let tiles: Vec<_> = self
            .ed
            .doc
            .styles
            .iter()
            .enumerate()
            .map(|(i, style)| {
                let selected = self.style_selection == Some(i);
                self.tile(
                    format!("style-{i}"),
                    style.name.clone(),
                    vec![style_preview(style.fill, style.stroke, style.opacity)],
                    selected,
                    cx,
                    Box::new(move |this, _, _, _| this.use_style(i)),
                )
            })
            .collect();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().h(px(26.)).px_1().flex().flex_none().items_center().child(self.disabled_combo("Current Document", 256.)))
            .child(div().id("styles").flex_1().min_h_0().p_1().overflow_y_scroll().bg(rgb(INPUT)).border_t_1().border_color(rgb(LIST_BORDER)).child(div().w(px(252.)).flex().flex_wrap().children(tiles)))
            .child(
                Self::footer()
                    .child(self.icon_button("plus", "New Style from Selection", false, cx, |this, _| {
                        if this.ed.add_style_from_selection() {
                            this.ed.commit("New Style");
                        } else {
                            this.status = "Select a vector object to make a style from it".into();
                        }
                    }))
                    .child(self.icon_button("minus", "Delete Style", false, cx, |this, _| {
                        match this.style_selection.take().filter(|i| *i < this.ed.doc.styles.len()) {
                            Some(index) => {
                                this.ed.doc.styles.remove(index);
                                this.ed.commit("Delete Style");
                            }
                            None => this.status = "Click a style first".into(),
                        }
                    })),
            )
            .into_any_element()
    }

    // ---- Path -----------------------------------------------------------------------

    pub(crate) fn path_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let button = |this: &Self, op: PathOp, cx: &mut Context<Self>| {
            let enabled = this.is_enabled(Cmd::PathOp(op));
            this.text_button(op.label(), enabled, cx, move |this, window, cx| this.run(Cmd::PathOp(op), window, cx))
        };
        let group = |this: &Self, ops: &[PathOp], cx: &mut Context<Self>| {
            div().flex().flex_wrap().gap_1().children(ops.iter().map(|op| button(this, *op, cx)).collect::<Vec<_>>())
        };
        div()
            .id("path-panel")
            .size_full()
            .p_1()
            .flex()
            .flex_col()
            .gap_1()
            .overflow_y_scroll()
            .child(heading("Combine Paths"))
            .child(group(self, &[PathOp::Union, PathOp::Intersect, PathOp::Punch, PathOp::Crop, PathOp::Join, PathOp::Split], cx))
            .child(heading("Alter Paths"))
            .child(group(self, &[PathOp::Simplify, PathOp::ExpandStroke, PathOp::Inset, PathOp::Outset], cx))
            .child(labelled_row("Amount", 48., div().flex().items_center().gap_1().child(self.field(FieldId::PathAmount, 44., cx)).child(label("px"))))
            .into_any_element()
    }

    // ---- Auto Shapes ----------------------------------------------------------------

    pub(crate) fn auto_shapes_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let tiles: Vec<_> = AutoKind::ALL
            .iter()
            .map(|kind| {
                let kind = *kind;
                self.captioned_tile(format!("auto-{}", kind.label()), kind.label().into(), vec![auto_preview(kind)], cx, Box::new(move |this, _, _, _| this.insert_auto_shape(kind)))
            })
            .collect();
        div().id("auto-shapes").size_full().p_1().overflow_y_scroll().bg(rgb(INPUT)).child(div().w(px(252.)).flex().flex_wrap().children(tiles)).into_any_element()
    }

    pub(crate) fn auto_properties_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.ed.selected();
        let Some(kind) = selected.iter().find_map(|o| match o.shape {
            Shape::Auto { kind, .. } => Some(kind),
            _ => None,
        }) else {
            return div().size_full().flex().items_center().justify_center().child(label("Select an auto shape")).into_any_element();
        };
        let rows: Vec<_> = kind
            .params()
            .iter()
            .enumerate()
            .filter_map(|(slot, info)| info.map(|info| (slot, info)))
            .map(|(slot, info)| {
                let suffix = if info.max <= 1.0 { "%" } else { "" };
                labelled_row(info.label, 80., div().flex().items_center().gap_1().child(self.field(FieldId::AutoParam(slot), 52., cx)).child(label(suffix)))
            })
            .collect();
        div()
            .p_2()
            .flex()
            .flex_col()
            .child(div().pb_1().font_weight(FontWeight::BOLD).child(kind.label()))
            .children(rows)
            .child(div().pt_1().child(self.text_button("Convert to Path", true, cx, |this, _, _| {
                this.ed.for_selected(|o| {
                    if matches!(o.shape, Shape::Auto { .. }) {
                        o.to_path();
                    }
                });
                this.ed.commit("Convert to Path");
            })))
            .into_any_element()
    }

    // ---- Libraries ------------------------------------------------------------------

    pub(crate) fn document_library_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let picked = self.symbol_selection.filter(|id| self.ed.doc.symbols.iter().any(|s| s.id == *id));
        let tiles: Vec<_> = self
            .ed
            .doc
            .symbols
            .iter()
            .map(|symbol| {
                let id = symbol.id;
                self.tile(
                    format!("symbol-{id}"),
                    symbol.name.clone(),
                    symbol.objects.iter().flat_map(Object::leaves).collect(),
                    picked == Some(id),
                    cx,
                    Box::new(move |this, ev, _, _| {
                        this.symbol_selection = Some(id);
                        if ev.click_count >= 2 {
                            this.place_symbol(id);
                        }
                    }),
                )
            })
            .collect();
        let has_selection = !self.ed.selection.is_empty();
        let grid = if tiles.is_empty() {
            div().size_full().flex().items_center().justify_center().child(label("Select objects, then press New")).into_any_element()
        } else {
            div().w(px(252.)).flex().flex_wrap().children(tiles).into_any_element()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().id("doc-library").flex_1().min_h_0().p_1().overflow_y_scroll().child(grid))
            .when_some(picked, |d, id| d.child(div().px_1().child(labelled_row("Name", 34., self.field(FieldId::RenameSymbol(id), 200., cx)))))
            .child(
                Self::footer()
                    .child(self.text_button("New", has_selection, cx, |this, window, cx| this.run(Cmd::ConvertToSymbol, window, cx)))
                    .child(self.text_button("Insert", picked.is_some(), cx, move |this, _, _| {
                        if let Some(id) = picked {
                            this.place_symbol(id);
                        }
                    }))
                    .child(self.text_button("Update", picked.is_some() && has_selection, cx, move |this, _, _| {
                        let Some(id) = picked else { return };
                        if this.ed.redefine_symbol(id) {
                            this.ed.commit("Update Symbol");
                        } else {
                            this.status = "A symbol cannot contain a copy of itself".into();
                        }
                    }))
                    .child(self.text_button("Delete", picked.is_some(), cx, move |this, _, _| {
                        if let Some(id) = picked {
                            this.ed.delete_symbol(id);
                            this.symbol_selection = None;
                            this.ed.commit("Delete Symbol");
                        }
                    })),
            )
            .into_any_element()
    }

    pub(crate) fn common_library_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut children: Vec<AnyElement> = Vec::new();
        let mut category = "";
        let mut tiles: Vec<AnyElement> = Vec::new();
        let flush = |children: &mut Vec<AnyElement>, tiles: &mut Vec<AnyElement>| {
            if !tiles.is_empty() {
                children.push(div().w(px(252.)).flex().flex_wrap().children(std::mem::take(tiles)).into_any_element());
            }
        };
        for (index, item) in common_items().iter().enumerate() {
            if item.category != category {
                flush(&mut children, &mut tiles);
                category = item.category;
                children.push(heading(category).into_any_element());
            }
            let objects = item.objects().iter().flat_map(Object::leaves).collect();
            tiles.push(
                self.captioned_tile(format!("common-{}", item.name), item.name.into(), objects, cx, Box::new(move |this, _, _, _| this.insert_common(index)))
                    .into_any_element(),
            );
        }
        flush(&mut children, &mut tiles);
        div().id("common-library").size_full().p_1().flex().flex_col().overflow_y_scroll().bg(rgb(INPUT)).children(children).into_any_element()
    }

    // ---- Color Mixer ----------------------------------------------------------------

    pub(crate) fn mixer_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.current_color(self.mixer_target);
        let chip = |this: &Self, text: &'static str, target: ColorTarget, cx: &mut Context<Self>| {
            let on = this.mixer_target == target;
            div()
                .h(px(20.))
                .px_2()
                .flex()
                .items_center()
                .when(on, |d| d.bg(rgb(ACCENT)))
                .when(!on, |d| d.bg(rgb(PANEL_RAISED)).hover(|s| s.bg(rgb(HOVER))))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.finish_pending();
                        this.mixer_target = target;
                        cx.notify();
                    }),
                )
                .child(text)
        };
        let base = current.unwrap_or(Color::BLACK);
        let channels: Vec<_> = (0..3usize)
            .map(|channel| {
                let cell = self.mixer_bounds[channel].clone();
                let ramp = canvas(move |bounds, _, _| cell.set(bounds), move |bounds: Bounds<gpui::Pixels>, _, window: &mut Window, _| {
                    let step = f32::from(bounds.size.width) / RAMP_STEPS as f32;
                    for i in 0..RAMP_STEPS {
                        let value = (i * 255 / (RAMP_STEPS - 1)) as u8;
                        let color = match channel {
                            0 => Color { r: value, ..base },
                            1 => Color { g: value, ..base },
                            _ => Color { b: value, ..base },
                        };
                        let origin = point(bounds.origin.x + px(i as f32 * step), bounds.origin.y);
                        window.paint_quad(fill(Bounds { origin, size: size(px(step + 0.5), bounds.size.height) }, rgb(color_u32(color))));
                    }
                })
                .size_full();
                div()
                    .h(px(22.))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_1()
                    .child(div().w(px(12.)).child(label(["R", "G", "B"][channel])))
                    .child(
                        div()
                            .flex_1()
                            .h(px(14.))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                                    this.mixer_pick(channel, ev.position);
                                    cx.notify();
                                }),
                            )
                            .child(ramp),
                    )
                    .child(self.field(FieldId::Mixer(channel), 36., cx))
            })
            .collect();
        div()
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(chip(self, "Fill", ColorTarget::Fill, cx))
                    .child(chip(self, "Stroke", ColorTarget::Stroke, cx))
                    .child(div().ml_auto().child(self.swatch(self.mixer_target, 20., cx)))
                    .child(label(current.map_or("None".to_string(), Color::hex))),
            )
            .children(channels)
            .into_any_element()
    }

    // ---- Info -----------------------------------------------------------------------

    pub(crate) fn info_body(&self) -> AnyElement {
        let p = self.cursor_doc;
        let bounds = self.ed.selection_bounds();
        let color = self.color_under(p);
        let row = |name: &'static str, value: String| div().h(px(18.)).flex().justify_between().child(label(name)).child(value);
        div()
            .p_2()
            .flex()
            .flex_col()
            .child(row("X", format!("{:.0}", p.x)))
            .child(row("Y", format!("{:.0}", p.y)))
            .child(row("Color", color.map_or("None".to_string(), |c| format!("{}  (R{} G{} B{})", c.hex(), c.r, c.g, c.b))))
            .child(row("Selection", bounds.map_or("None".to_string(), |b| format!("{:.0} x {:.0} at {:.0}, {:.0}", b.w, b.h, b.x, b.y))))
            .into_any_element()
    }

    // ---- URL ------------------------------------------------------------------------

    pub(crate) fn url_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let rows: Vec<_> = self
            .ed
            .doc
            .urls
            .iter()
            .enumerate()
            .map(|(i, url)| {
                let chosen = self.url_selection == Some(i);
                Self::list_row()
                    .when(chosen, |d| d.bg(rgb(ACCENT)))
                    .when(!chosen, |d| d.hover(|s| s.bg(rgb(HOVER))))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.finish_pending();
                            this.use_url(i);
                            cx.notify();
                        }),
                    )
                    .child(div().overflow_hidden().child(url.clone()))
            })
            .collect();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().p_1().flex_none().child(labelled_row("Add", 26., self.field(FieldId::NewUrl, 220., cx))))
            .child(div().id("urls").flex_1().min_h_0().flex().flex_col().overflow_y_scroll().children(rows))
            .child(Self::footer().child(self.icon_button("minus", "Delete URL", false, cx, |this, _| {
                match this.url_selection.take().filter(|i| *i < this.ed.doc.urls.len()) {
                    Some(index) => {
                        this.ed.doc.urls.remove(index);
                        this.ed.commit("Delete URL");
                    }
                    None => this.status = "Click a URL first".into(),
                }
            })))
            .into_any_element()
    }

    // ---- Find -----------------------------------------------------------------------

    pub(crate) fn find_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let chip = |this: &Self, text: &'static str, on: bool, cx: &mut Context<Self>, action: fn(&mut Workspace)| {
            this.text_button(text, true, cx, move |this, _, _| action(this)).when(on, |d| d.bg(rgb(ACCENT)))
        };
        let text_mode = self.find_mode == FindMode::Text;
        div()
            .p_2()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(chip(self, "Text", text_mode, cx, |w| w.find_mode = FindMode::Text))
                    .child(chip(self, "Color", !text_mode, cx, |w| w.find_mode = FindMode::Color))
                    .child(div().ml_auto().child(chip(self, "All Pages", self.find_all_pages, cx, |w| w.find_all_pages = !w.find_all_pages))),
            )
            .child(labelled_row("Find", 44., self.field(FieldId::FindText, 190., cx)))
            .child(labelled_row("Change", 44., self.field(FieldId::ReplaceText, 190., cx)))
            .child(
                div()
                    .pt_1()
                    .flex()
                    .gap_1()
                    .child(self.text_button("Find", true, cx, |this, _, _| this.run_find(false)))
                    .child(self.text_button("Replace All", true, cx, |this, _, _| this.run_find(true))),
            )
            .into_any_element()
    }

    // ---- Special Characters -----------------------------------------------------------

    pub(crate) fn special_characters_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let cells: Vec<_> = SPECIAL_CHARACTERS
            .chars()
            .map(|ch| {
                div()
                    .size(px(24.))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .text_size(px(14.))
                    .hover(|s| s.bg(rgb(HOVER)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            // Keeps an open field or text edit active so the character lands in it.
                            this.keep_overlays = true;
                            this.insert_special(ch);
                            cx.notify();
                        }),
                    )
                    .child(ch.to_string())
            })
            .collect();
        div().id("characters").size_full().p_1().overflow_y_scroll().child(div().w(px(252.)).flex().flex_wrap().children(cells)).into_any_element()
    }

    // ---- Image Editing ----------------------------------------------------------------

    pub(crate) fn image_editing_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let bitmap_tools = TOOL_SECTIONS.iter().find(|(name, _)| *name == "Bitmap").map_or(&[][..], |(_, slots)| *slots);
        let slots: Vec<_> = bitmap_tools.iter().chain(COLOR_TOOLS).map(|s| self.tool_slot(s, cx)).collect();
        let filters = [Filter::Brighten, Filter::Darken, Filter::Grayscale, Filter::Invert, Filter::Blur, Filter::Sharpen];
        let buttons: Vec<_> = filters
            .into_iter()
            .map(|filter| {
                let enabled = self.is_enabled(Cmd::Filter(filter));
                self.text_button(filter.label(), enabled, cx, move |this, window, cx| this.run(Cmd::Filter(filter), window, cx))
            })
            .collect();
        div()
            .id("image-editing")
            .size_full()
            .p_1()
            .flex()
            .flex_col()
            .gap_1()
            .overflow_y_scroll()
            .child(div().flex().flex_wrap().children(slots))
            .child(heading("Filters"))
            .child(div().flex().flex_wrap().gap_1().children(buttons))
            .into_any_element()
    }

    /// Renders one dock panel.
    pub(crate) fn panel_body(&self, panel: Panel, cx: &mut Context<Self>) -> AnyElement {
        match panel {
            Panel::Optimize => self.optimize_body(cx),
            Panel::History => self.history_body(cx),
            Panel::Align => self.align_body(cx),
            Panel::Pages => self.pages_body(cx),
            Panel::States => self.states_body(cx),
            Panel::Layers => self.layers_body(cx),
            Panel::Styles => self.styles_body(cx),
            Panel::Path => self.path_body(cx),
            Panel::ColorPalette => self.color_palette_body(cx),
            Panel::Swatches => self.swatches_body(cx),
            Panel::AutoShapeProperties => self.auto_properties_body(cx),
            Panel::AutoShapes => self.auto_shapes_body(cx),
            Panel::DocumentLibrary => self.document_library_body(cx),
            Panel::CommonLibrary => self.common_library_body(cx),
            Panel::Mixer => self.mixer_body(cx),
            Panel::Info => self.info_body(),
            Panel::Url => self.url_body(cx),
            Panel::Find => self.find_body(cx),
            Panel::SpecialCharacters => self.special_characters_body(cx),
            Panel::ImageEditing => self.image_editing_body(cx),
            Panel::Toolbar | Panel::Tools | Panel::Properties => div().into_any_element(),
        }
    }
}
