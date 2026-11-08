//! Renders the docked workspace.

use std::sync::Arc;

use gpui::{
    anchored, canvas, deferred, div, prelude::*, px, rgb, svg, AnyElement, AnyView, App, Context, CursorStyle, Div,
    ElementInputHandler, FontWeight, MouseButton, MouseDownEvent, ParentElement, Render, SharedString, Stateful, Svg,
    Window, WindowControlArea,
};

use crate::canvas_view::{paint_thumbnail_with, Scene, Textures};
use crate::icons::{colored, layer_path};
use crate::menus::{Cmd, Entry, Panel, SlotAction, ToolSlot, COLOR_TOOLS, DOCK_GROUPS, MENUS, SCREEN_MODES, TOOL_SECTIONS, VIEW_TOOLS};
use crate::model::{Align, Arrange, Color, Object, Rect, Shape};
use crate::optimize::settings_label;
use crate::paths::PathOp;
use crate::theme::*;
use crate::workspace::{kind_label, ColorTarget, Combo, FieldId, PropertiesTab, ViewMode, Workspace};

/// Height of each dock group when expanded. `None` takes the remaining space.
const GROUP_HEIGHTS: [Option<f32>; DOCK_GROUPS.len()] =
    [Some(262.), None, Some(170.), Some(175.), Some(180.), Some(130.), Some(150.)];
/// How far an open group may shrink when the dock runs out of room.
const MIN_GROUP_HEIGHT: f32 = 70.;
const MIN_FLEX_GROUP_HEIGHT: f32 = 110.;
/// Marks the Web Layer in the list of collapsed layers.
const WEB_LAYER: usize = usize::MAX;
const TAB_LINE: u32 = 0x737373;

struct Tip(SharedString);

impl Render for Tip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_1p5()
            .py_0p5()
            .bg(rgb(0xf2f2d8))
            .border_1()
            .border_color(rgb(BORDER))
            .text_color(rgb(0x202020))
            .text_size(px(FONT_SIZE))
            .font_family(FONT)
            .child(self.0.clone())
    }
}

pub(crate) fn tip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |_, cx| cx.new(|_| Tip(text.clone())).into()
}

pub(crate) fn icon(name: &str, size: f32, color: u32) -> Svg {
    svg().path(SharedString::from(format!("icons/{name}.svg"))).size(px(size)).text_color(rgb(color))
}

/// How a colored icon looks when its command is unavailable.
fn washed_out(color: u32) -> u32 {
    let channel = |shift: u32| ((color >> shift) & 0xff) as f32;
    let luminance = 0.3 * channel(16) + 0.59 * channel(8) + 0.11 * channel(0);
    let gray = (luminance * 0.35 + 150.0).min(235.0) as u32;
    gray << 16 | gray << 8 | gray
}

/// An icon in its full colors, built from one tinted shape per color.
pub(crate) fn color_icon(name: &str, size: f32, enabled: bool) -> AnyElement {
    let Some(layers) = colored(name) else {
        return icon(name, size, if enabled { TEXT } else { TEXT_DISABLED }).into_any_element();
    };
    let shapes: Vec<_> = layers
        .iter()
        .enumerate()
        .map(|(index, (color, _))| {
            svg()
                .path(layer_path(name, index))
                .absolute()
                .top_0()
                .left_0()
                .size(px(size))
                .text_color(rgb(if enabled { *color } else { washed_out(*color) }))
        })
        .collect();
    div().relative().size(px(size)).flex_none().children(shapes).into_any_element()
}

pub(crate) fn color_u32(c: Color) -> u32 {
    (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32
}

pub(crate) fn label(text: impl Into<SharedString>) -> Div {
    div().text_color(rgb(TEXT_DIM)).child(text.into())
}

fn web_safe_colors() -> impl Iterator<Item = Color> {
    (0..6u8).flat_map(|r| (0..6u8).flat_map(move |g| (0..6u8).map(move |b| Color::rgb(r * 51, g * 51, b * 51))))
}

impl Workspace {
    fn app_bar(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let titles: Vec<_> = MENUS
            .iter()
            .enumerate()
            .map(|(i, (title, entries))| {
                let open = self.open_menu == Some(i);
                div()
                    .relative()
                    .h_full()
                    .px_2()
                    .flex()
                    .items_center()
                    .when(open, |d| d.bg(rgb(ACCENT)))
                    .when(!open, |d| d.hover(|s| s.bg(rgb(HOVER))))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.finish_pending();
                            this.keep_overlays = true;
                            this.open_menu = if this.open_menu == Some(i) { None } else { Some(i) };
                            this.open_sub = None;
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, _, _, cx| {
                        if this.open_menu.is_some() && this.open_menu != Some(i) {
                            this.open_menu = Some(i);
                            this.open_sub = None;
                            cx.notify();
                        }
                    }))
                    .child(*title)
                    .when(open, |d| {
                        d.child(deferred(
                            self.menu_panel(entries, true, cx).absolute().top(px(MENU_BAR_HEIGHT)).left_0(),
                        ))
                    })
            })
            .collect();
        let window_button = |id: &'static str, name: &'static str, area: WindowControlArea, close: bool| {
            div()
                .id(id)
                .w(px(34.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(PANEL_RAISED))
                .border_1()
                .border_color(rgb(INPUT_BORDER))
                .hover(move |s| s.bg(rgb(if close { WINDOW_CLOSE_HOVER } else { HOVER })))
                .occlude()
                .window_control_area(area)
                .child(icon(name, 10., TEXT))
        };
        let zoom = (self.zoom * 100.0).round() as u32;
        let mode = if self.compact_workspace { "Collapsed Mode" } else { "Expanded Mode" };
        div()
            .h(px(MENU_BAR_HEIGHT))
            .flex()
            .flex_none()
            .items_center()
            .bg(rgb(PANEL))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .w(px(40.))
                    .h_full()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .text_size(px(19.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(APP_MARK_COLOR))
                    .occlude()
                    .window_control_area(WindowControlArea::Drag)
                    .child(APP_MARK),
            )
            .child(div().h_full().flex().flex_none().children(titles))
            .child(div().w(px(1.)).h(px(20.)).mx_2().flex_none().bg(rgb(SEPARATOR)))
            .child(div().id("app-bar-tools").flex().flex_none().children(VIEW_TOOLS.iter().map(|s| self.tool_slot(s, cx)).collect::<Vec<_>>()))
            .child(div().ml_1().flex_none().child(self.combo(Combo::Zoom, format!("{zoom}%"), 64., cx).bg(rgb(PANEL_RAISED))))
            .child(div().flex_1().h_full().occlude().window_control_area(WindowControlArea::Drag))
            .child(div().mr_2().flex_none().child(self.combo(Combo::WorkspaceMode, mode, 112., cx).bg(rgb(PANEL_RAISED))))
            .child(
                div()
                    .mr_1()
                    .flex()
                    .flex_none()
                    .gap(px(2.))
                    .child(window_button("window-min", "win-min", WindowControlArea::Min, false))
                    .child(window_button(
                        "window-max",
                        if window.is_maximized() { "win-restore" } else { "win-max" },
                        WindowControlArea::Max,
                        false,
                    ))
                    .child(window_button("window-close", "win-close", WindowControlArea::Close, true)),
            )
    }

    fn menu_panel(&self, entries: &'static [Entry], top_level: bool, cx: &mut Context<Self>) -> Div {
        let rows: Vec<AnyElement> =
            entries.iter().enumerate().map(|(i, entry)| self.menu_row(entry, i, top_level, cx)).collect();
        div()
            .occlude()
            .w(px(240.))
            .py_1()
            .flex()
            .flex_col()
            .bg(rgb(PANEL_RAISED))
            .border_1()
            .border_color(rgb(BORDER))
            .shadow_md()
            .children(rows)
    }

    fn menu_row(&self, entry: &'static Entry, index: usize, top_level: bool, cx: &mut Context<Self>) -> AnyElement {
        let row = || div().relative().h(px(20.)).px_3().flex().items_center().justify_between();
        match entry {
            Entry::Sep => div().my_1().h(px(1.)).bg(rgb(BORDER)).into_any_element(),
            Entry::Item(name, keys, cmd) => {
                let enabled = cmd.is_some_and(|c| self.is_enabled(c));
                let checked = cmd.is_some_and(|c| self.is_checked(c));
                let color = if enabled { TEXT } else { TEXT_DISABLED };
                let command = *cmd;
                row()
                    .text_color(rgb(color))
                    .when(enabled, |d| d.hover(|s| s.bg(rgb(ACCENT))))
                    .when_some(command.filter(|_| enabled), |d, c| {
                        d.on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| this.run(c, window, cx)))
                    })
                    .when(top_level, |d| {
                        d.on_mouse_move(cx.listener(|this, _, _, cx| {
                            if this.open_sub.take().is_some() {
                                cx.notify();
                            }
                        }))
                    })
                    .when(checked, |d| d.child(div().absolute().left(px(1.)).child(icon("check", 10., TEXT))))
                    .child(*name)
                    .child(div().text_color(rgb(TEXT_DIM)).child(*keys))
                    .into_any_element()
            }
            Entry::Sub(name, children) => {
                let open = self.open_sub == Some(index);
                row()
                    .text_color(rgb(TEXT))
                    .when(open, |d| d.bg(rgb(ACCENT)))
                    .on_mouse_move(cx.listener(move |this, _, _, cx| {
                        if this.open_sub != Some(index) {
                            this.open_sub = Some(index);
                            cx.notify();
                        }
                    }))
                    .child(*name)
                    .child(icon("chevron", 10., TEXT_DIM))
                    .when(open, |d| {
                        d.child(self.menu_panel(children, false, cx).absolute().left(px(238.)).top(px(-5.)))
                    })
                    .into_any_element()
            }
        }
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let sep = || div().w(px(1.)).h(px(18.)).mx_1().flex_none().bg(rgb(SEPARATOR));
        div()
            .id("toolbar")
            .h(px(TOOLBAR_HEIGHT))
            .flex()
            .flex_none()
            .items_center()
            .px_1()
            .bg(rgb(PANEL))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(self.tool_button("new", "New", Cmd::New, cx))
            .child(self.tool_button("save", "Save", Cmd::Save, cx))
            .child(self.tool_button("open", "Open", Cmd::Open, cx))
            .child(self.tool_button("import", "Import", Cmd::Import, cx))
            .child(self.tool_button("export", "Export", Cmd::Export, cx))
            .child(div().id("print").size(px(24.)).flex().items_center().justify_center().tooltip(tip("Print (not available yet)")).child(color_icon("print", 16., false)))
            .child(sep())
            .child(self.tool_button("undo", "Undo", Cmd::Undo, cx))
            .child(self.tool_button("redo", "Redo", Cmd::Redo, cx))
            .child(sep())
            .child(self.tool_button("cut", "Cut", Cmd::Cut, cx))
            .child(self.tool_button("copy", "Copy", Cmd::Copy, cx))
            .child(self.tool_button("paste", "Paste", Cmd::Paste, cx))
            .child(sep())
            .child(self.tool_button("group", "Group", Cmd::Group, cx))
            .child(self.tool_button("ungroup", "Ungroup", Cmd::Ungroup, cx))
            .child(self.tool_button("join", "Join", Cmd::PathOp(PathOp::Join), cx))
            .child(self.tool_button("split", "Split", Cmd::PathOp(PathOp::Split), cx))
            .child(sep())
            .child(self.tool_button("arrange-front", "Bring to Front", Cmd::Arrange(Arrange::Front), cx))
            .child(self.tool_button("arrange-forward", "Bring Forward", Cmd::Arrange(Arrange::Forward), cx))
            .child(self.tool_button("arrange-backward", "Send Backward", Cmd::Arrange(Arrange::Backward), cx))
            .child(self.tool_button("arrange-back", "Send to Back", Cmd::Arrange(Arrange::Back), cx))
            .child(sep())
            .child(self.combo(Combo::Align, "Align", 62., cx).bg(rgb(PANEL_RAISED)))
            .child(sep())
            .child(self.tool_button("rotate-ccw", "Rotate 90 CCW", Cmd::Rotate { clockwise: false }, cx))
            .child(self.tool_button("rotate-cw", "Rotate 90 CW", Cmd::Rotate { clockwise: true }, cx))
            .child(self.tool_button("flip-h", "Flip Horizontal", Cmd::FlipH, cx))
            .child(self.tool_button("flip-v", "Flip Vertical", Cmd::FlipV, cx))
    }

    fn tool_button(&self, name: &'static str, hint: &'static str, cmd: Cmd, cx: &mut Context<Self>) -> Stateful<Div> {
        self.command_button(name, hint, cmd, 24., 16., cx)
    }

    fn command_button(&self, name: &'static str, hint: &'static str, cmd: Cmd, box_size: f32, icon_size: f32, cx: &mut Context<Self>) -> Stateful<Div> {
        let enabled = self.is_enabled(cmd);
        div()
            .id(name)
            .size(px(box_size))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .tooltip(tip(hint))
            .when(enabled, |d| {
                d.hover(|s| s.bg(rgb(HOVER))).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| this.run(cmd, window, cx)),
                )
            })
            .child(color_icon(name, icon_size, enabled))
    }

    pub(crate) fn tool_slot(&self, slot: &'static ToolSlot, cx: &mut Context<Self>) -> Stateful<Div> {
        self.sized_tool_slot(slot, 30., cx)
    }

    fn sized_tool_slot(&self, slot: &'static ToolSlot, width: f32, cx: &mut Context<Self>) -> Stateful<Div> {
        let action = slot.action;
        let active = matches!(action, SlotAction::Select(tool) if tool == self.tool);
        div()
            .id(slot.label)
            .w(px(width))
            .h(px(26.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .tooltip(tip(slot.label))
            .when(active, |d| d.bg(rgb(INPUT)).border_1().border_color(rgb(INPUT_BORDER)))
            .when(!active, |d| {
                d.hover(|s| s.bg(rgb(HOVER))).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        match action {
                            SlotAction::Select(tool) => this.set_tool(tool),
                            SlotAction::Run(cmd) => this.run(cmd, window, cx),
                        }
                        cx.notify();
                    }),
                )
            })
            .child(color_icon(slot.icon, 17., true))
    }

    fn tools_panel(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let heading = |text: &'static str| {
            div()
                .h(px(16.))
                .mx_1()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .border_b_1()
                .border_color(rgb(SEPARATOR))
                .text_color(rgb(TEXT_DIM))
                .text_size(px(10.))
                .child(text)
        };
        let group = |this: &Self, slots: &'static [ToolSlot], cx: &mut Context<Self>| {
            div().flex().flex_wrap().justify_center().py(px(2.)).children(slots.iter().map(|s| this.tool_slot(s, cx)).collect::<Vec<_>>())
        };
        let mut panel = div()
            .id("tools")
            .w(px(TOOLS_WIDTH))
            .flex_none()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .bg(rgb(PANEL))
            .border_r_1()
            .border_color(rgb(DIVIDER));
        for (name, slots) in TOOL_SECTIONS {
            panel = panel.child(heading(name)).child(group(self, slots, cx));
        }
        let color_row = |this: &Self, name: &'static str, target, cx: &mut Context<Self>| {
            div().h(px(23.)).flex().flex_none().items_center().justify_center().gap_1().child(color_icon(name, 15., true)).child(this.swatch(target, 26., cx))
        };
        let mini = |name: &'static str, hint: &'static str, action: fn(&mut Workspace), cx: &mut Context<Self>| {
            div()
                .id(name)
                .size(px(19.))
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .tooltip(tip(hint))
                .hover(|s| s.bg(rgb(HOVER)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        action(this);
                        cx.notify();
                    }),
                )
                .child(color_icon(name, 13., true))
        };
        panel
            .child(heading("Colors"))
            .child(group(self, COLOR_TOOLS, cx))
            .child(color_row(self, "pencil", ColorTarget::Stroke, cx))
            .child(color_row(self, "bucket", ColorTarget::Fill, cx))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .justify_center()
                    .child(mini("defaults", "Set Default Stroke/Fill Colors", Workspace::default_colors, cx))
                    .child(mini("none", "No Fill", |w| w.apply_color(ColorTarget::Fill, None), cx))
                    .child(mini("swap", "Swap Stroke/Fill Colors", Workspace::swap_colors, cx)),
            )
            .child(heading("View"))
            .child(div().flex().flex_none().justify_center().py(px(2.)).children(SCREEN_MODES.iter().map(|s| self.sized_tool_slot(s, 20., cx)).collect::<Vec<_>>()))
            .child(group(self, VIEW_TOOLS, cx))
    }

    pub(crate) fn swatch(&self, target: ColorTarget, size: f32, cx: &mut Context<Self>) -> Div {
        let color = self.current_color(target);
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .border_color(rgb(BORDER))
            .bg(rgb(color.map_or(0xffffff, color_u32)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    this.finish_pending();
                    this.keep_overlays = true;
                    this.color_popup = Some((target, ev.position));
                    cx.notify();
                }),
            )
            .when(color.is_none(), |d| d.child(icon("none", size - 2., 0xd02020)))
    }

    pub(crate) fn field(&self, id: FieldId, width: f32, cx: &mut Context<Self>) -> Div {
        let editing = self.field.as_ref().filter(|f| f.id == id);
        let text = match editing {
            Some(f) => format!("{}|", f.buffer),
            None => self.field_value(id),
        };
        div()
            .w(px(width))
            .h(px(18.))
            .px_1()
            .flex()
            .items_center()
            .overflow_hidden()
            .bg(rgb(INPUT))
            .border_1()
            .border_color(rgb(if editing.is_some() { SELECTION } else { INPUT_BORDER }))
            .text_color(rgb(TEXT))
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.begin_field(id);
                    cx.notify();
                }),
            )
            .child(text)
    }

    fn labelled(&self, name: &'static str, id: FieldId, width: f32, cx: &mut Context<Self>) -> Div {
        div().flex().items_center().gap_1().child(div().w(px(14.)).child(label(name))).child(self.field(id, width, cx))
    }

    fn view_button(&self, mode: ViewMode, name: &'static str, text: &'static str, cx: &mut Context<Self>) -> Div {
        let active = self.view_mode == mode;
        div()
            .h(px(20.))
            .px_1p5()
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .when(active, |d| d.bg(rgb(INPUT)).border_1().border_color(rgb(INPUT_BORDER)))
            .when(!active, |d| d.hover(|s| s.bg(rgb(HOVER))))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.set_view_mode(mode);
                    cx.notify();
                }),
            )
            .child(color_icon(name, 14., true))
            .child(text)
    }

    fn document(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let page = self.ed.doc.page();
        let zoom = (self.zoom * 100.0).round() as u32;
        let dirty = if self.ed.is_dirty() { "*" } else { "" };
        let tab = format!("{dirty}{}", self.document_name());
        let bounds_cell = self.canvas_bounds.clone();
        let scene = Scene::capture(self);
        let focus = self.focus.clone();
        let entity = cx.entity();
        let cursor = if self.view_mode != ViewMode::Original {
            CursorStyle::OpenHand
        } else {
            match self.active_tool() {
                crate::menus::Tool::Pointer | crate::menus::Tool::Subselect | crate::menus::Tool::Scale => CursorStyle::Arrow,
                crate::menus::Tool::Hand => CursorStyle::OpenHand,
                crate::menus::Tool::Text => CursorStyle::IBeam,
                _ => CursorStyle::Crosshair,
            }
        };
        let tab_strip = div()
            .h(px(DOCUMENT_TAB_HEIGHT))
            .flex()
            .flex_none()
            .items_end()
            .px_1()
            .bg(rgb(PANEL_DARK))
            .border_b_1()
            .border_color(rgb(TAB_LINE))
            .child(
                div()
                    .h(px(20.))
                    .pl_2()
                    .pr_1()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(rgb(PANEL))
                    .border_t_1()
                    .border_l_1()
                    .border_r_1()
                    .border_color(rgb(TAB_LINE))
                    .text_color(rgb(TEXT))
                    .child(tab)
                    .child(
                        div()
                            .size(px(13.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(rgb(HOVER)))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| this.run(Cmd::Close, window, cx)))
                            .child(icon("close", 8., TEXT_DIM)),
                    ),
            );
        let view_strip = div()
            .id("view-strip")
            .h(px(VIEW_STRIP_HEIGHT))
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .px_1()
            .bg(rgb(PANEL))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(self.view_button(ViewMode::Original, "view-original", "Original", cx))
                    .child(self.view_button(ViewMode::Preview, "view-preview", "Preview", cx))
                    .child(self.view_button(ViewMode::TwoUp, "view-2up", "2-Up", cx))
                    .child(self.view_button(ViewMode::FourUp, "view-4up", "4-Up", cx)),
            )
            .child(self.combo(Combo::Page, page.name.clone(), 110., cx));
        let status_bar = div()
            .id("status-bar")
            .h(px(24.))
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .px_2()
            .gap_3()
            .bg(rgb(PANEL))
            .border_t_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .overflow_hidden()
                    .child(div().flex_none().child(label(settings_label(&page.export))))
                    .child(div().min_w_0().truncate().child(label(self.status.clone()))),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .child(self.command_button("first", "First State", Cmd::FirstState, 18., 11., cx))
                            .child(self.command_button(if self.playing { "stop" } else { "play" }, "Play or stop the animation", Cmd::PlayStates, 18., 11., cx))
                            .child(self.command_button("last", "Last State", Cmd::LastState, 18., 11., cx))
                            .child(div().w(px(24.)).flex().justify_center().child(label(format!("{}", page.active_state + 1))))
                            .child(self.command_button("prev", "Previous State", Cmd::PreviousState, 18., 11., cx))
                            .child(self.command_button("next", "Next State", Cmd::NextState, 18., 11., cx)),
                    )
                    .child(label(format!("{} x {}", page.width, page.height)))
                    .child(self.combo(Combo::Zoom, format!("{zoom}%"), 64., cx)),
            );
        div()
            .id("document")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .child(tab_strip)
            .child(view_strip)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .overflow_hidden()
                    .cursor(cursor)
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::canvas_down))
                    .on_scroll_wheel(cx.listener(Self::canvas_scroll))
                    .child(
                        canvas(move |bounds, _, _| bounds_cell.set(bounds), move |bounds, _, window, cx| {
                            window.handle_input(&focus, ElementInputHandler::new(bounds, entity), cx);
                            scene.paint(bounds, window, cx)
                        })
                        .size_full(),
                    ),
            )
            .child(status_bar)
    }

    fn properties(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let tab = |text: &'static str, which: PropertiesTab, cx: &mut Context<Self>| {
            let front = self.properties_tab == which;
            div()
                .h_full()
                .px_3()
                .flex()
                .items_center()
                .when(front, |d| d.bg(rgb(PANEL)).font_weight(FontWeight::SEMIBOLD).border_t_1().border_color(rgb(PANEL_RAISED)))
                .when(!front, |d| d.text_color(rgb(TEXT_DIM)).hover(|s| s.bg(rgb(HOVER))))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.finish_pending();
                        this.properties_tab = which;
                        this.properties_collapsed = false;
                        cx.notify();
                    }),
                )
                .child(text)
        };
        let collapsed = self.properties_collapsed;
        let strip = div()
            .h(px(TAB_HEIGHT))
            .flex()
            .flex_none()
            .items_end()
            .justify_between()
            .bg(rgb(PANEL_DARK))
            .border_b_1()
            .border_color(rgb(TAB_LINE))
            .child(div().h_full().flex().child(tab("Properties", PropertiesTab::Properties, cx)).child(tab("Symbol Properties", PropertiesTab::Symbol, cx)))
            .child(
                div()
                    .id("properties-collapse")
                    .h_full()
                    .w(px(22.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .tooltip(tip(if collapsed { "Expand" } else { "Collapse" }))
                    .hover(|s| s.bg(rgb(HOVER)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.properties_collapsed = !this.properties_collapsed;
                            cx.notify();
                        }),
                    )
                    .child(icon(if collapsed { "triangle-closed" } else { "triangle-open" }, 9., TEXT_DIM)),
            );
        let shell = div().id("properties").flex().flex_none().flex_col().border_t_1().border_color(rgb(DIVIDER)).child(strip);
        if collapsed {
            return shell;
        }
        let body = match self.properties_tab {
            PropertiesTab::Properties => self.properties_body(cx),
            PropertiesTab::Symbol => self.symbol_properties_body(cx).into_any_element(),
        };
        shell.h(px(PROPERTIES_HEIGHT)).child(body)
    }

    /// The small preview of the page shown when nothing is selected.
    fn document_thumbnail(&self, size: f32) -> Div {
        let page = self.ed.doc.page();
        let background = Object {
            id: 0,
            name: String::new(),
            shape: Shape::Rect { radius: 0.0 },
            bounds: Rect { x: 0.0, y: 0.0, w: page.width as f32, h: page.height as f32 },
            points: Vec::new(),
            fill: page.canvas.or(Some(Color::rgb(0xff, 0xff, 0xff))),
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
        };
        let mut objects = vec![background];
        objects.extend(page.layers.iter().filter(|l| l.visible).flat_map(|l| &l.objects).filter(|o| o.visible).cloned());
        div().size(px(size)).flex_none().bg(rgb(0xf4f4f4)).border_1().border_color(rgb(INPUT_BORDER)).child(self.object_preview(objects, Arc::new(self.textures.clone())))
    }

    fn object_preview(&self, objects: Vec<Object>, textures: Arc<Textures>) -> impl IntoElement {
        canvas(|_, _, _| (), move |bounds, _, window, cx| paint_thumbnail_with(&objects, &textures, bounds, window, cx)).size_full()
    }

    fn properties_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let shell = div().flex_1().min_h_0().flex().gap_4().p_2().bg(rgb(PANEL));
        let selected = self.ed.selected();
        use crate::menus::Tool;
        let tool_size = match self.tool {
            Tool::Brush | Tool::Eraser | Tool::Blur | Tool::Stamp => Some(("Brush size:", FieldId::BrushSize)),
            Tool::Freeform => Some(("Size:", FieldId::FreeformSize)),
            _ => None,
        };
        let brush_size = tool_size.map(|(name, id)| {
            div().flex().items_center().gap_1().h(px(20.)).child(label(name)).child(self.field(id, 44., cx))
        });
        let Some(first) = selected.first().copied() else {
            let page = self.ed.doc.page();
            let button = |this: &Self, text: &'static str, cmd: Cmd, cx: &mut Context<Self>| {
                this.text_button(text, this.is_enabled(cmd), cx, move |this, window, cx| this.run(cmd, window, cx))
            };
            return shell
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .gap_2()
                        .child(self.document_thumbnail(46.))
                        .child(
                            div()
                                .w(px(110.))
                                .overflow_hidden()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().font_weight(FontWeight::BOLD).text_color(rgb(TEXT)).child("Document"))
                                .child(label(self.document_name())),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_none()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .h(px(20.))
                                .child(label("Canvas:"))
                                .child(self.swatch(ColorTarget::Canvas, 18., cx))
                                .child(self.field(FieldId::CanvasHex, 66., cx)),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_1()
                                .child(button(self, "Canvas Size...", Cmd::CanvasSize, cx))
                                .child(button(self, "Image Size...", Cmd::ImageSize, cx))
                                .child(button(self, "Fit Canvas", Cmd::FitCanvas, cx)),
                        ),
                )
                .child(div().w(px(1.)).h_full().flex_none().bg(rgb(SEPARATOR)))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .h(px(20.))
                        .child(label("State:"))
                        .child(self.combo(Combo::State, page.states[page.active_state].name.clone(), 120., cx)),
                )
                .children(brush_size)
                .into_any_element();
        };
        let no_colors = selected.iter().all(|o| !o.takes_color());
        let is_web = first.is_web();
        let single = selected.len() == 1;
        let title: SharedString =
            if single { kind_label(&first.shape).into() } else { format!("{} objects", selected.len()).into() };
        let mut extra = div().flex().flex_col().gap_1();
        match first.shape {
            crate::model::Shape::Rect { .. } => {
                extra = extra.child(div().flex().items_center().gap_1().child(label("Roundness:")).child(self.field(FieldId::Radius, 44., cx)));
            }
            crate::model::Shape::Text { .. } => {
                extra = extra.child(div().flex().items_center().gap_1().child(label("Size:")).child(self.field(FieldId::TextSize, 44., cx)));
            }
            crate::model::Shape::Web { kind: crate::model::WebKind::Hotspot, .. } => {
                extra = extra.child(div().flex().items_center().gap_1().child(label("Link:")).child(self.field(FieldId::Url, 220., cx)));
            }
            _ => {}
        }
        let color_line = |this: &Self, name: &'static str, target, hex, cx: &mut Context<Self>| {
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(div().w(px(34.)).child(label(name)))
                .child(this.swatch(target, 18., cx))
                .child(this.field(hex, 66., cx))
        };
        shell
            .child(div().w(px(110.)).overflow_hidden().flex().flex_col().gap_1().child(div().font_weight(FontWeight::BOLD).text_color(rgb(TEXT)).child(title)).when(single, |d| d.child(label(first.name.clone()))))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().flex().gap_2().child(self.labelled("W:", FieldId::W, 52., cx)).child(self.labelled("X:", FieldId::X, 52., cx)))
                    .child(div().flex().gap_2().child(self.labelled("H:", FieldId::H, 52., cx)).child(self.labelled("Y:", FieldId::Y, 52., cx))),
            )
            .when(!no_colors, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(color_line(self, "Fill:", ColorTarget::Fill, FieldId::FillHex, cx))
                        .child(color_line(self, "Stroke:", ColorTarget::Stroke, FieldId::StrokeHex, cx))
                        .child(div().flex().items_center().gap_1().child(div().w(px(34.)).child(label("Width:"))).child(self.field(FieldId::StrokeWidth, 44., cx))),
                )
            })
            .when(!is_web, |d| {
                d.child(div().flex().items_center().gap_1().h(px(20.)).child(label("Opacity:")).child(self.field(FieldId::Opacity, 44., cx)).child(label("%")))
            })
            .child(extra)
            .children(brush_size)
            .into_any_element()
    }

    /// One tab group of the dock, or nothing if all of its panels are closed.
    fn panel_group(&self, group: usize, cx: &mut Context<Self>) -> Option<Div> {
        let active = self.group_active(group)?;
        let collapsed = self.groups[group].collapsed;
        let tabs: Vec<_> = DOCK_GROUPS[group]
            .iter()
            .copied()
            .filter(|p| self.panel_open(*p))
            .map(|panel| {
                let front = panel == active && !collapsed;
                div()
                    .h_full()
                    .px(px(6.))
                    .flex()
                    .flex_none()
                    .items_center()
                    .when(front, |d| d.bg(rgb(PANEL)).text_color(rgb(TEXT)).font_weight(FontWeight::SEMIBOLD).border_t_1().border_color(rgb(PANEL_RAISED)))
                    .when(!front, |d| d.text_color(rgb(TEXT_DIM)).hover(|s| s.bg(rgb(HOVER))))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                            this.finish_pending();
                            let state = &mut this.groups[group];
                            if state.collapsed {
                                state.collapsed = false;
                            } else if ev.click_count >= 2 && state.active == panel {
                                state.collapsed = true;
                            }
                            state.active = panel;
                            cx.notify();
                        }),
                    )
                    .child(panel.title())
            })
            .collect();
        let strip = div()
            .h(px(TAB_HEIGHT))
            .flex()
            .flex_none()
            .justify_between()
            .overflow_hidden()
            .bg(rgb(PANEL_DARK))
            .border_b_1()
            .border_color(rgb(TAB_LINE))
            .child(div().h_full().flex_1().min_w_0().overflow_hidden().flex().children(tabs))
            .child(
                div()
                    .h_full()
                    .w(px(20.))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.bg(rgb(HOVER)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                            this.finish_pending();
                            this.keep_overlays = true;
                            this.open_combo = Some((Combo::PanelMenu(group), ev.position));
                            cx.notify();
                        }),
                    )
                    .child(icon("panel-menu", 10., TEXT_DIM)),
            );
        let shell = div().flex().flex_col().min_h_0().border_b_1().border_color(rgb(DIVIDER));
        if collapsed {
            return Some(shell.flex_none().child(strip));
        }
        let shell = match GROUP_HEIGHTS[group] {
            Some(height) => shell.h(px(height)).min_h(px(MIN_GROUP_HEIGHT)),
            None => shell.flex_1().min_h(px(MIN_FLEX_GROUP_HEIGHT)),
        };
        Some(shell.child(strip).child(div().flex_1().min_h_0().bg(rgb(PANEL)).overflow_hidden().child(self.panel_body(active, cx))))
    }

    pub(crate) fn list_row() -> Div {
        div().h(px(20.)).px_2().flex().flex_none().items_center().gap_1()
    }

    pub(crate) fn history_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let rows: Vec<_> = self
            .ed
            .history
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let current = i == self.ed.history_pos;
                let future = i > self.ed.history_pos;
                Self::list_row()
                    .when(current, |d| d.bg(rgb(ACCENT)))
                    .when(!current, |d| d.hover(|s| s.bg(rgb(HOVER))))
                    .text_color(rgb(if future { TEXT_DISABLED } else { TEXT }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.finish_pending();
                            this.ed.goto_history(i);
                            cx.notify();
                        }),
                    )
                    .child(name.clone())
            })
            .collect();
        div().id("history").size_full().flex().flex_col().overflow_y_scroll().bg(rgb(LIST)).children(rows).into_any_element()
    }

    pub(crate) fn align_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let button = |this: &Self, name: &'static str, hint: &'static str, op: Align, cx: &mut Context<Self>| {
            this.tool_button(name, hint, Cmd::Align(op), cx)
        };
        div()
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(button(self, "align-left", "Align Left", Align::Left, cx))
                    .child(button(self, "align-center-x", "Center", Align::CenterX, cx))
                    .child(button(self, "align-right", "Align Right", Align::Right, cx))
                    .child(button(self, "align-top", "Align Top", Align::Top, cx))
                    .child(button(self, "align-center-y", "Center Vertically", Align::CenterY, cx))
                    .child(button(self, "align-bottom", "Align Bottom", Align::Bottom, cx)),
            )
            .child(label("A single object aligns to the canvas."))
            .into_any_element()
    }

    fn panel_footer(&self, text: Option<String>, add: fn(&mut Workspace), remove: fn(&mut Workspace), cx: &mut Context<Self>) -> Div {
        let button = |name: &'static str, action: fn(&mut Workspace), cx: &mut Context<Self>| {
            div()
                .size(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .hover(|s| s.bg(rgb(HOVER)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.finish_pending();
                        action(this);
                        cx.notify();
                    }),
                )
                .child(color_icon(name, 14., true))
        };
        div()
            .h(px(24.))
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .px_1()
            .bg(rgb(PANEL))
            .border_t_1()
            .border_color(rgb(SEPARATOR))
            .child(label(text.unwrap_or_default()))
            .child(div().flex().child(button("new-layer", add, cx)).child(button("trash", remove, cx)))
    }

    pub(crate) fn pages_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.ed.doc.active_page;
        let rows: Vec<_> = self
            .ed
            .doc
            .pages
            .iter()
            .enumerate()
            .map(|(i, page)| {
                Self::list_row()
                    .when(i == active, |d| d.bg(rgb(ACCENT)))
                    .when(i != active, |d| d.hover(|s| s.bg(rgb(HOVER))))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                            this.finish_pending();
                            this.ed.set_active_page(i);
                            if ev.click_count >= 2 {
                                this.begin_field(FieldId::RenamePage(i));
                            }
                            cx.notify();
                        }),
                    )
                    .child(self.name_or_field(FieldId::RenamePage(i), page.name.clone(), 200., cx))
            })
            .collect();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().id("pages").flex_1().min_h_0().flex().flex_col().overflow_y_scroll().bg(rgb(LIST)).children(rows))
            .child(self.panel_footer(
                None,
                |w| {
                    w.ed.add_page();
                    w.ed.commit("New Page");
                },
                |w| {
                    if w.ed.delete_page() {
                        w.ed.commit("Delete Page");
                    } else {
                        w.status = "A document needs at least one page".into();
                    }
                },
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn toggle(name: &'static str, on: bool) -> Div {
        div().size(px(16.)).flex().flex_none().items_center().justify_center().when(on, |d| d.child(icon(name, 12., TEXT)))
    }

    /// A bordered column cell of the Layers list, such as the eye or the lock.
    fn layer_cell() -> Div {
        div().w(px(18.)).h_full().flex().flex_none().items_center().justify_center().border_r_1().border_color(rgb(LIST_BORDER))
    }

    fn layer_row() -> Div {
        div().h(px(20.)).flex().flex_none().items_center().gap_1().border_b_1().border_color(rgb(LIST_BORDER))
    }

    fn layer_object_row(&self, li: usize, object: &Object, textures: &Arc<Textures>, cx: &mut Context<Self>) -> AnyElement {
        let id = object.id;
        let selected = self.ed.is_selected(id);
        Self::layer_row()
            .when(selected, |d| d.bg(rgb(ACCENT)))
            .when(!selected, |d| d.hover(|s| s.bg(rgb(HOVER))))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    this.finish_pending();
                    this.ed.selection = vec![id];
                    this.ed.set_active_layer(li);
                    if ev.click_count >= 2 {
                        this.begin_field(FieldId::RenameObject(id));
                    }
                    cx.notify();
                }),
            )
            .child(Self::layer_cell().when(object.visible, |d| d.child(icon("eye", 12., TEXT))).on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if let Some(object) = this.ed.object_mut(id) {
                        object.visible = !object.visible;
                    }
                    this.ed.commit("Object Visibility");
                    cx.notify();
                }),
            ))
            .child(Self::layer_cell().when(object.locked, |d| d.child(icon("lock", 12., TEXT))).on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if let Some(object) = this.ed.object_mut(id) {
                        object.locked = !object.locked;
                    }
                    this.ed.commit("Object Lock");
                    cx.notify();
                }),
            ))
            .child(
                div()
                    .ml_3()
                    .size(px(17.))
                    .flex_none()
                    .bg(rgb(INPUT))
                    .border_1()
                    .border_color(rgb(LIST_BORDER))
                    .child(self.object_preview(vec![object.clone()], textures.clone())),
            )
            .child(self.name_or_field(FieldId::RenameObject(id), object.name.clone(), 130., cx))
            .into_any_element()
    }

    fn layer_triangle(&self, key: usize, collapsed: bool, cx: &mut Context<Self>) -> Div {
        div()
            .size(px(12.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if !this.collapsed_layers.contains(&key) {
                        this.collapsed_layers.push(key);
                    } else {
                        this.collapsed_layers.retain(|k| *k != key);
                    }
                    cx.notify();
                }),
            )
            .child(icon(if collapsed { "triangle-closed" } else { "triangle-open" }, 8., TEXT_DIM))
    }

    pub(crate) fn layers_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let page = self.ed.doc.page();
        let textures = Arc::new(self.textures.clone());
        let mut rows: Vec<AnyElement> = Vec::new();
        let web_objects: Vec<(usize, &Object)> =
            page.layers.iter().enumerate().flat_map(|(li, layer)| layer.objects.iter().map(move |o| (li, o))).filter(|(_, o)| o.is_web()).collect();
        let web_collapsed = self.collapsed_layers.contains(&WEB_LAYER);
        let web_visible = web_objects.is_empty() || self.ed.web_visible();
        rows.push(
            Self::layer_row()
                .bg(rgb(LIST_SPECIAL))
                .font_weight(FontWeight::SEMIBOLD)
                .child(Self::layer_cell().when(web_visible, |d| d.child(icon("eye", 12., TEXT))).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.ed.set_web_visible(!web_visible);
                        this.ed.commit("Web Layer Visibility");
                        cx.notify();
                    }),
                ))
                .child(Self::layer_cell())
                .child(self.layer_triangle(WEB_LAYER, web_collapsed, cx))
                .child(color_icon("folder", 14., true))
                .child(div().flex_1().min_w_0().overflow_hidden().child("Web Layer"))
                .into_any_element(),
        );
        if !web_collapsed {
            for (li, object) in web_objects.iter().rev() {
                rows.push(self.layer_object_row(*li, object, &textures, cx));
            }
        }
        for (li, layer) in page.layers.iter().enumerate().rev() {
            let holds_selection = layer.objects.iter().any(|o| self.ed.is_selected(o.id));
            let active = li == page.active_layer && !holds_selection;
            let collapsed = self.collapsed_layers.contains(&li);
            rows.push(
                Self::layer_row()
                    .when(active, |d| d.bg(rgb(ACCENT)))
                    .font_weight(FontWeight::SEMIBOLD)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                            this.finish_pending();
                            this.ed.set_active_layer(li);
                            if ev.click_count >= 2 {
                                this.begin_field(FieldId::RenameLayer(li));
                            }
                            cx.notify();
                        }),
                    )
                    .child(Self::layer_cell().when(layer.visible, |d| d.child(icon("eye", 12., TEXT))).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let layer = &mut this.ed.doc.page_mut().layers[li];
                            layer.visible = !layer.visible;
                            this.ed.commit("Layer Visibility");
                            cx.notify();
                        }),
                    ))
                    .child(Self::layer_cell().when(layer.locked, |d| d.child(icon("lock", 12., TEXT))).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let layer = &mut this.ed.doc.page_mut().layers[li];
                            layer.locked = !layer.locked;
                            this.ed.commit("Layer Lock");
                            cx.notify();
                        }),
                    ))
                    .child(self.layer_triangle(li, collapsed, cx))
                    .child(color_icon("folder", 14., true))
                    .child(self.name_or_field(FieldId::RenameLayer(li), layer.name.clone(), 120., cx))
                    .child(Self::toggle("share", layer.shared).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let page = this.ed.doc.page_mut();
                            let shared = !page.layers[li].shared;
                            page.set_layer_shared(li, shared);
                            this.ed.commit("Share Layer");
                            cx.notify();
                        }),
                    ))
                    .child(div().mr_1().size(px(8.)).flex_none().rounded_full().border_1().border_color(rgb(BORDER)).when(holds_selection, |d| d.bg(rgb(SELECTION))))
                    .into_any_element(),
            );
            if !collapsed {
                for object in layer.objects.iter().rev().filter(|o| !o.is_web()) {
                    rows.push(self.layer_object_row(li, object, &textures, cx));
                }
            }
        }
        let state = page.states.get(page.active_state).map(|s| s.name.clone());
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(26.))
                    .px_1()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_between()
                    .child(self.disabled_combo("Normal", 110.))
                    .child(div().flex().items_center().gap_1().child(label("Opacity")).child(self.field(FieldId::Opacity, 40., cx)).child(label("%"))),
            )
            .child(div().id("layers").flex_1().min_h_0().flex().flex_col().overflow_y_scroll().bg(rgb(LIST)).border_t_1().border_color(rgb(LIST_BORDER)).children(rows))
            .child(self.panel_footer(
                state,
                |w| {
                    w.ed.add_layer();
                    w.ed.commit("New Layer");
                },
                |w| {
                    if w.ed.delete_layer() {
                        w.ed.commit("Delete Layer");
                    } else {
                        w.status = "A page needs at least one layer".into();
                    }
                },
                cx,
            ))
            .into_any_element()
    }

    fn swatch_cell(color: Color, size: f32) -> Div {
        div().size(px(size)).flex_none().bg(rgb(color_u32(color)))
    }

    pub(crate) fn swatches_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let cells: Vec<_> = web_safe_colors()
            .map(|color| {
                Self::swatch_cell(color, 12.).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                        this.finish_pending();
                        let target = if ev.modifiers.shift { ColorTarget::Stroke } else { ColorTarget::Fill };
                        this.apply_color(target, Some(color));
                        cx.notify();
                    }),
                )
            })
            .collect();
        div().id("swatches").size_full().p_1().overflow_y_scroll().child(div().w(px(252.)).flex().flex_wrap().children(cells)).into_any_element()
    }

    fn dock(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let groups: Vec<_> = (0..DOCK_GROUPS.len()).filter_map(|group| self.panel_group(group, cx)).collect();
        if groups.is_empty() {
            return None;
        }
        Some(
            div()
                .id("dock")
                .w(px(DOCK_WIDTH))
                .flex_none()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .bg(rgb(PANEL))
                .border_l_1()
                .border_color(rgb(DIVIDER))
                .children(groups),
        )
    }

    fn color_popup_view(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (target, position) = self.color_popup?;
        let cells: Vec<_> = web_safe_colors()
            .map(|color| {
                Self::swatch_cell(color, 13.).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.apply_color(target, Some(color));
                        this.color_popup = None;
                        cx.notify();
                    }),
                )
            })
            .collect();
        let hex = self.current_color(target).map_or("None".to_string(), Color::hex);
        Some(
            deferred(
                anchored().position(position).snap_to_window().child(
                    div()
                        .occlude()
                        .w(px(242.))
                        .p_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .bg(rgb(PANEL_RAISED))
                        .border_1()
                        .border_color(rgb(BORDER))
                        .shadow_md()
                        .child(
                            div().flex().items_center().justify_between().child(hex).child(
                                div()
                                    .px_2()
                                    .bg(rgb(INPUT))
                                    .hover(|s| s.bg(rgb(HOVER)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.apply_color(target, None);
                                            this.color_popup = None;
                                            cx.notify();
                                        }),
                                    )
                                    .child("None"),
                            ),
                        )
                        .child(div().flex().flex_wrap().children(cells)),
                ),
            )
            .with_priority(2)
            .into_any_element(),
        )
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_textures(window, cx);
        self.refresh_estimate(cx);
        self.refresh_previews(cx);
        let title = self.title();
        if title != self.last_title {
            window.set_window_title(&title);
            self.last_title = title;
        }
        div()
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(PANEL))
            .text_color(rgb(TEXT))
            .text_size(px(FONT_SIZE))
            .font_family(FONT)
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::root_mouse_down))
            .on_mouse_move(cx.listener(Self::mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .child(self.app_bar(window, cx))
            .when(self.panel_open(Panel::Toolbar), |d| d.child(self.toolbar(cx)))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .when(self.panel_open(Panel::Tools), |d| d.child(self.tools_panel(cx)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(self.document(cx))
                            .when(self.panel_open(Panel::Properties), |d| d.child(self.properties(cx))),
                    )
                    .children(self.dock(cx)),
            )
            .children(self.size_dialog_view(cx))
            .children(self.color_popup_view(cx))
            .children(self.combo_popup(cx))
    }
}
