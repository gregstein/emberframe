//! Application state, commands and input handling.

use std::cell::Cell;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context as _, Result};

use gpui::{
    point, px, App, Bounds, Context, EntityInputHandler, FocusHandle, KeyDownEvent, KeyUpEvent, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PathPromptOptions, Pixels, Point, PromptLevel, RenderImage, ScrollDelta,
    ScrollWheelEvent, UTF16Selection, Window,
};

use std::sync::Arc;
use std::time::Duration;

use crate::bitmap::{Bitmap, Mask, MaskMode, FILL_TOLERANCE};
use crate::canvas_view::{encoded_texture, handle_rects, make_texture, texture_key, Textures};
use crate::autoshape::AutoKind;
use crate::export::{export_web, measure_text};
use crate::library::common_items;
use crate::menus::{panel_shortcut, Cmd, Panel, ScreenMode, Tool, DOCK_GROUPS, ZOOM_LEVELS};
use crate::optimize::{comparison_settings, encode, encoded_palette, export_page, format_size, settings_label, ExportSettings, MAX_COLORS, MIN_COLORS};
use crate::paths::PathOp;
use crate::model::{
    document_from_json, document_to_json, pull_points, subdivide, v2, Color, Document, Editor, Object, PixelStroke,
    Rect, Shape, Stroke, Vec2, WebKind,
};
use crate::theme::{APP_NAME, FILE_EXTENSION};

const DEFAULT_WIDTH: u32 = 660;
const DEFAULT_HEIGHT: u32 = 500;
const PASTE_OFFSET: f32 = 10.0;
const MIN_ZOOM: f32 = 0.06;
const MAX_ZOOM: f32 = 64.0;
const DEFAULT_BRUSH_SIZE: f32 = 12.0;
const MAX_BRUSH_SIZE: f32 = 1000.0;
const DEFAULT_FREEFORM_SIZE: f32 = 64.0;
/// Vertex spacing for Freeform, as a fraction of its radius.
const FREEFORM_DETAIL: f32 = 0.25;
const DEFAULT_PATH_AMOUNT: f32 = 4.0;
const AUTO_SHAPE_SIZE: f32 = 120.0;
const ONION_OPACITY: f32 = 0.3;
/// Fastest playback, so a very short delay cannot flood the window with frames.
const MIN_FRAME_MS: u32 = 20;
const MIN_STATE_DELAY: u32 = 10;
const MAX_STATE_DELAY: u32 = 655_000;
/// Dock groups that start open; the rest show only their tabs.
const EXPANDED_GROUPS: usize = 5;
const MAX_CANVAS_SIZE: u32 = 16000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorTarget {
    Fill,
    Stroke,
    Canvas,
    Matte,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Combo {
    Format,
    Transparency,
    Preset,
    Zoom,
    WorkspaceMode,
    Page,
    State,
    Align,
    PanelMenu(usize),
}

/// How the document area shows the page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Original,
    Preview,
    TwoUp,
    FourUp,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropertiesTab {
    Properties,
    Symbol,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SizeDialogKind {
    Canvas,
    Image,
}

/// The Canvas Size or Image Size dialog while it is open.
#[derive(Clone, Copy, Debug)]
pub struct SizeDialog {
    pub kind: SizeDialogKind,
    pub width: u32,
    pub height: u32,
    pub constrain: bool,
}

/// One encoded version of the page, shown by the Preview, 2-Up and 4-Up views.
#[derive(Clone)]
pub struct PreviewPane {
    pub image: Arc<RenderImage>,
    pub caption: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindMode {
    Text,
    Color,
}

/// One tab group of the dock.
#[derive(Clone, Copy)]
pub struct DockGroup {
    pub active: Panel,
    pub collapsed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FieldId {
    X,
    Y,
    W,
    H,
    Radius,
    TextSize,
    StrokeWidth,
    BrushSize,
    FreeformSize,
    Url,
    Opacity,
    FillHex,
    StrokeHex,
    CanvasHex,
    RenamePage(usize),
    RenameState(usize),
    RenameLayer(usize),
    RenameObject(u64),
    RenameSymbol(u64),
    StateDelay(usize),
    AutoParam(usize),
    PathAmount,
    OptimizeQuality,
    OptimizeColors,
    Mixer(usize),
    FindText,
    ReplaceText,
    NewUrl,
    DialogW,
    DialogH,
}

pub struct FieldEdit {
    pub id: FieldId,
    pub buffer: String,
    /// The next typed character replaces the whole value.
    pub fresh: bool,
}

pub enum Drag {
    Move { last: Vec2 },
    Resize { handle: usize, origin: Rect },
    Marquee { start: Vec2, current: Vec2 },
    Create { start: Vec2, current: Vec2 },
    /// `target` is the bitmap being painted; `None` draws a vector path.
    Freehand { points: Vec<Vec2>, target: Option<u64> },
    Crop { start: Vec2, current: Vec2 },
    /// A Marquee or Lasso drag. Marquee keeps just the two corners.
    PixelArea { target: u64, points: Vec<Vec2>, mode: MaskMode },
    Scale { handle: usize, origin: Rect, originals: Vec<Object> },
    Freeform { id: u64, start: Vec2, base: Vec<Vec2>, radius: f32 },
    Vertex { id: u64, index: usize },
    Pan { last: Point<Pixels> },
}

/// Selected pixels of one bitmap. Edits to that bitmap stay inside the mask.
pub struct PixelSelection {
    pub object: u64,
    pub mask: Arc<Mask>,
    pub edges: Arc<Vec<(Vec2, Vec2)>>,
}

/// Rubber Stamp state. `offset` is fixed by the first stroke after setting a source.
#[derive(Clone, Copy)]
pub struct Stamp {
    pub object: u64,
    pub source: Vec2,
    pub offset: Option<Vec2>,
}

pub struct Workspace {
    pub ed: Editor,
    pub tool: Tool,
    pub zoom: f32,
    pub pan: Vec2,
    pub stroke: Option<Color>,
    pub fill: Option<Color>,
    pub stroke_width: f32,
    pub brush_size: f32,
    pub freeform_size: f32,
    pub pixel_sel: Option<PixelSelection>,
    pub stamp: Option<Stamp>,
    pub crop_area: Option<Rect>,
    pub textures: Textures,
    /// The image this document was opened from. Never overwritten by Save.
    pub source_image: Option<PathBuf>,
    pub open_menu: Option<usize>,
    pub open_sub: Option<usize>,
    pub color_popup: Option<(ColorTarget, Point<Pixels>)>,
    pub field: Option<FieldEdit>,
    pub text_edit: Option<u64>,
    pub drag: Option<Drag>,
    pub pen: Vec<Vec2>,
    pub cursor_doc: Vec2,
    pub focus: FocusHandle,
    pub canvas_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub path: Option<PathBuf>,
    pub groups: [DockGroup; DOCK_GROUPS.len()],
    pub hidden_panels: Vec<Panel>,
    /// Hide Panels clears the workspace without forgetting which panels were open.
    pub panels_off: bool,
    pub open_combo: Option<(Combo, Point<Pixels>)>,
    pub onion: bool,
    pub playing: bool,
    play_token: u64,
    pub estimate: Option<usize>,
    /// Colors of the exported image, for indexed formats.
    pub estimate_palette: Vec<Color>,
    pub view_mode: ViewMode,
    pub previews: Vec<PreviewPane>,
    previews_key: Option<(u64, ViewMode)>,
    stale_previews: Vec<Arc<RenderImage>>,
    pub size_dialog: Option<SizeDialog>,
    pub collapsed_layers: Vec<usize>,
    pub properties_collapsed: bool,
    pub properties_tab: PropertiesTab,
    pub compact_workspace: bool,
    pub estimate_error: Option<String>,
    estimate_key: Option<u64>,
    pub path_amount: f32,
    pub style_selection: Option<usize>,
    pub symbol_selection: Option<u64>,
    pub url_selection: Option<usize>,
    pub mixer_target: ColorTarget,
    pub mixer_bounds: [Rc<Cell<Bounds<Pixels>>>; 3],
    pub find_mode: FindMode,
    pub find_all_pages: bool,
    pub find_text: String,
    pub replace_text: String,
    pub show_rulers: bool,
    pub show_grid: bool,
    pub space_held: bool,
    pub status: String,
    /// Set by overlays during a click so the root handler keeps them open.
    pub keep_overlays: bool,
    pub last_title: String,
}

/// Maps document coordinates to window coordinates for a given view.
#[derive(Clone, Copy)]
pub struct ViewMap {
    pub origin: Point<Pixels>,
    pub zoom: f32,
}

impl ViewMap {
    pub fn new(bounds: Bounds<Pixels>, page_w: f32, page_h: f32, zoom: f32, pan: Vec2) -> Self {
        let cx = f32::from(bounds.origin.x) + f32::from(bounds.size.width) / 2.0 + pan.x;
        let cy = f32::from(bounds.origin.y) + f32::from(bounds.size.height) / 2.0 + pan.y;
        Self { origin: point(px((cx - page_w * zoom / 2.0).round()), px((cy - page_h * zoom / 2.0).round())), zoom }
    }

    pub fn to_screen(self, p: Vec2) -> Point<Pixels> {
        point(self.origin.x + px(p.x * self.zoom), self.origin.y + px(p.y * self.zoom))
    }

    pub fn to_doc(self, p: Point<Pixels>) -> Vec2 {
        v2(f32::from(p.x - self.origin.x) / self.zoom, f32::from(p.y - self.origin.y) / self.zoom)
    }
}

fn snap(p: Vec2) -> Vec2 {
    v2(p.x.round(), p.y.round())
}

fn constrain_square(start: Vec2, p: Vec2) -> Vec2 {
    let side = (p.x - start.x).abs().max((p.y - start.y).abs());
    v2(start.x + side.copysign(p.x - start.x), start.y + side.copysign(p.y - start.y))
}

fn constrain_angle(start: Vec2, p: Vec2) -> Vec2 {
    let (dx, dy) = (p.x - start.x, p.y - start.y);
    let step = std::f32::consts::FRAC_PI_4;
    let angle = (dy.atan2(dx) / step).round() * step;
    let len = (dx * dx + dy * dy).sqrt();
    v2(start.x + len * angle.cos(), start.y + len * angle.sin())
}

pub fn kind_label(shape: &Shape) -> &'static str {
    match shape {
        Shape::Rect { .. } => "Rectangle",
        Shape::Ellipse => "Ellipse",
        Shape::Line => "Line",
        Shape::Path { .. } => "Path",
        Shape::Text { .. } => "Text",
        Shape::Image { .. } => "Bitmap",
        Shape::Web { kind: WebKind::Hotspot, .. } => "Hotspot",
        Shape::Web { kind: WebKind::Slice, .. } => "Slice",
        Shape::Auto { kind, .. } => kind.label(),
        Shape::Instance { .. } => "Symbol",
        Shape::Group { .. } => "Group",
    }
}

/// Corner handles keep proportions when `keep_aspect` is set; side handles stretch one axis.
fn scaled(origin: Rect, handle: usize, p: Vec2, keep_aspect: bool) -> Rect {
    if !keep_aspect || !matches!(handle, 0 | 2 | 4 | 6) || origin.w <= 0.0 || origin.h <= 0.0 {
        return resized(origin, handle, p);
    }
    let left_handle = matches!(handle, 0 | 6);
    let top_handle = matches!(handle, 0 | 2);
    let anchor_x = if left_handle { origin.right() } else { origin.x };
    let anchor_y = if top_handle { origin.bottom() } else { origin.y };
    let smallest = 1.0 / origin.w.min(origin.h);
    let factor = ((p.x - anchor_x).abs() / origin.w).max((p.y - anchor_y).abs() / origin.h).max(smallest);
    let (w, h) = (origin.w * factor, origin.h * factor);
    Rect { x: if left_handle { anchor_x - w } else { anchor_x }, y: if top_handle { anchor_y - h } else { anchor_y }, w, h }
}

/// Rescales an object that sat inside `origin` so it sits at the same place inside `target`.
fn scale_object(original: &Object, origin: Rect, target: Rect) -> Object {
    let sx = if origin.w > 0.0 { target.w / origin.w } else { 1.0 };
    let sy = if origin.h > 0.0 { target.h / origin.h } else { 1.0 };
    let b = original.bounds;
    let mut object = original.clone();
    object.set_bounds(Rect {
        x: target.x + (b.x - origin.x) * sx,
        y: target.y + (b.y - origin.y) * sy,
        w: b.w * sx,
        h: b.h * sy,
    });
    if let Shape::Text { content, size } = &mut object.shape {
        *size = (*size * sy).max(1.0);
        (object.bounds.w, object.bounds.h) = measure_text(content, *size);
    }
    object
}

fn selection_mode(modifiers: gpui::Modifiers) -> MaskMode {
    if modifiers.alt {
        MaskMode::Subtract
    } else if modifiers.shift {
        MaskMode::Add
    } else {
        MaskMode::Replace
    }
}

enum Opened {
    Document(Document),
    Image(Bitmap),
}

/// Identifies a file by content, so a misnamed file still opens correctly.
fn parse_file(bytes: &[u8]) -> Result<Opened> {
    if bytes.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{') {
        let text = std::str::from_utf8(bytes).context("not an Emberframe document")?;
        return document_from_json(text).map(Opened::Document);
    }
    Bitmap::decode(bytes).map(Opened::Image)
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map_or_else(|| "Untitled".into(), |s| s.to_string_lossy().into_owned())
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            ed: Editor::new(Document::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)),
            tool: Tool::Pointer,
            zoom: 1.0,
            pan: Vec2::default(),
            stroke: Some(Color::BLACK),
            fill: Some(Color::WHITE),
            stroke_width: 1.0,
            brush_size: DEFAULT_BRUSH_SIZE,
            freeform_size: DEFAULT_FREEFORM_SIZE,
            pixel_sel: None,
            stamp: None,
            crop_area: None,
            textures: Textures::new(),
            source_image: None,
            open_menu: None,
            open_sub: None,
            color_popup: None,
            field: None,
            text_edit: None,
            drag: None,
            pen: Vec::new(),
            cursor_doc: Vec2::default(),
            focus: cx.focus_handle(),
            canvas_bounds: Rc::new(Cell::new(Bounds::default())),
            path: None,
            groups: [
                DockGroup { active: Panel::Optimize, collapsed: false },
                DockGroup { active: Panel::Layers, collapsed: false },
                DockGroup { active: Panel::Styles, collapsed: false },
                DockGroup { active: Panel::AutoShapes, collapsed: false },
                DockGroup { active: Panel::CommonLibrary, collapsed: false },
                DockGroup { active: Panel::Mixer, collapsed: true },
                DockGroup { active: Panel::Find, collapsed: true },
            ],
            hidden_panels: Vec::new(),
            panels_off: false,
            open_combo: None,
            onion: false,
            playing: false,
            play_token: 0,
            estimate: None,
            estimate_palette: Vec::new(),
            view_mode: ViewMode::Original,
            previews: Vec::new(),
            previews_key: None,
            stale_previews: Vec::new(),
            size_dialog: None,
            collapsed_layers: Vec::new(),
            properties_collapsed: false,
            properties_tab: PropertiesTab::Properties,
            compact_workspace: false,
            estimate_error: None,
            estimate_key: None,
            path_amount: DEFAULT_PATH_AMOUNT,
            style_selection: None,
            symbol_selection: None,
            url_selection: None,
            mixer_target: ColorTarget::Fill,
            mixer_bounds: Default::default(),
            find_mode: FindMode::Text,
            find_all_pages: false,
            find_text: String::new(),
            replace_text: String::new(),
            show_rulers: true,
            show_grid: false,
            space_held: false,
            status: String::new(),
            keep_overlays: false,
            last_title: String::new(),
        }
    }

    pub fn title(&self) -> String {
        let dirty = if self.ed.is_dirty() { "*" } else { "" };
        format!("{}{dirty} - {APP_NAME}", self.document_name())
    }

    pub fn document_name(&self) -> String {
        self.path
            .as_ref()
            .or(self.source_image.as_ref())
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("Untitled.{FILE_EXTENSION}"))
    }

    /// Name offered by Save As; an opened image becomes a document beside it.
    fn save_name(&self) -> String {
        match (&self.path, &self.source_image) {
            (None, Some(source)) => format!("{}.{FILE_EXTENSION}", file_stem(source)),
            _ => self.document_name(),
        }
    }

    /// Stroke width of the active freehand tool, in document units.
    pub fn freehand_width(&self) -> f32 {
        if self.tool == Tool::Pencil { 1.0 } else { self.brush_size }
    }

    /// The pixel selection, if its bitmap still exists, is selected and has the same size.
    pub fn active_pixels(&self) -> Option<&PixelSelection> {
        let selection = self.pixel_sel.as_ref()?;
        let bitmap = self.ed.object(selection.object)?.bitmap()?;
        (self.ed.is_selected(selection.object) && selection.mask.fits(bitmap)).then_some(selection)
    }

    fn mask_for(&self, id: u64) -> Option<Arc<Mask>> {
        self.active_pixels().filter(|s| s.object == id).map(|s| s.mask.clone())
    }

    /// Keeps a GPU image for each bitmap on the page and frees the rest.
    pub fn sync_textures(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut current = Textures::new();
        let ghosts = self.onion_ghosts();
        let leaves: Vec<Object> = self
            .ed
            .doc
            .page()
            .objects()
            .chain(ghosts.iter())
            .filter(|o| o.is_image() || o.is_instance())
            .flat_map(Object::leaves)
            .collect();
        for object in &leaves {
            let Some((key, bitmap)) = texture_key(object) else { continue };
            if !current.contains_key(&key) {
                let entry = self
                    .textures
                    .remove(&key)
                    .unwrap_or_else(|| (bitmap.clone(), make_texture(bitmap, object.opacity)));
                current.insert(key, entry);
            }
        }
        for (_, (_, image)) in std::mem::replace(&mut self.textures, current).drain() {
            cx.drop_image(image, Some(window));
        }
        for image in self.stale_previews.drain(..) {
            cx.drop_image(image, Some(window));
        }
    }

    pub fn view_map(&self) -> ViewMap {
        let page = self.ed.doc.page();
        ViewMap::new(self.canvas_bounds.get(), page.width as f32, page.height as f32, self.zoom, self.pan)
    }

    pub fn active_tool(&self) -> Tool {
        if self.space_held { Tool::Hand } else { self.tool }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.finish_pending();
        self.crop_area = None;
        self.tool = tool;
    }

    /// Commits any in-progress field, text or pen edit.
    pub fn finish_pending(&mut self) {
        self.commit_field();
        self.finish_text();
        self.finish_pen(false);
    }

    pub fn close_overlays(&mut self) {
        self.open_menu = None;
        self.open_sub = None;
        self.color_popup = None;
        self.open_combo = None;
    }

    // ---- Panels ---------------------------------------------------------------------

    /// Whether the panel is listed in the workspace, even on a background tab.
    pub fn panel_open(&self, panel: Panel) -> bool {
        !self.panels_off && !self.hidden_panels.contains(&panel)
    }

    /// The panel a dock group is showing, skipping panels that were closed.
    pub fn group_active(&self, group: usize) -> Option<Panel> {
        let active = self.groups[group].active;
        if self.panel_open(active) {
            return Some(active);
        }
        DOCK_GROUPS[group].iter().copied().find(|p| self.panel_open(*p))
    }

    /// Whether the panel's contents are on screen.
    pub fn panel_showing(&self, panel: Panel) -> bool {
        DOCK_GROUPS.iter().enumerate().any(|(i, group)| {
            group.contains(&panel) && !self.groups[i].collapsed && self.group_active(i) == Some(panel)
        })
    }

    /// Opens a panel and brings it to the front, or closes it if it is already in front.
    fn toggle_panel(&mut self, panel: Panel) {
        let Some(group) = DOCK_GROUPS.iter().position(|g| g.contains(&panel)) else {
            match self.hidden_panels.iter().position(|p| *p == panel) {
                Some(index) if !self.panels_off => {
                    self.hidden_panels.remove(index);
                }
                None if !self.panels_off => self.hidden_panels.push(panel),
                _ => {
                    self.hidden_panels.retain(|p| *p != panel);
                    self.panels_off = false;
                }
            }
            return;
        };
        if self.panel_showing(panel) {
            self.hidden_panels.push(panel);
        } else {
            self.panels_off = false;
            self.hidden_panels.retain(|p| *p != panel);
            self.groups[group].active = panel;
            self.groups[group].collapsed = false;
        }
    }

    pub fn is_checked(&self, cmd: Cmd) -> bool {
        match cmd {
            Cmd::Panel(panel) => self.panel_open(panel),
            Cmd::HidePanels => self.panels_off,
            Cmd::Rulers => self.show_rulers,
            Cmd::Grid => self.show_grid,
            _ => false,
        }
    }

    pub fn set_export(&mut self, change: impl FnOnce(&mut ExportSettings)) {
        change(&mut self.ed.doc.page_mut().export);
        self.ed.commit("Optimize");
    }

    /// Measures the exported size in the background whenever the page changes.
    pub fn refresh_estimate(&mut self, cx: &mut Context<Self>) {
        let key = self.ed.revision;
        if !self.panel_showing(Panel::Optimize) || self.estimate_key == Some(key) {
            return;
        }
        self.estimate_key = Some(key);
        self.estimate = None;
        self.estimate_error = None;
        let page = self.ed.doc.page().clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let bytes = encode(&page, &page.export)?;
                    let palette = if page.export.format.is_indexed() { encoded_palette(&bytes) } else { Vec::new() };
                    anyhow::Ok((bytes.len(), palette))
                })
                .await;
            this.update(cx, |this, cx| {
                if this.estimate_key == Some(key) {
                    match result {
                        Ok((bytes, palette)) => {
                            this.estimate = Some(bytes);
                            this.estimate_palette = palette;
                        }
                        Err(error) => this.estimate_error = Some(error.to_string()),
                    }
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Encodes the page for the Preview, 2-Up and 4-Up views whenever it changes.
    pub fn refresh_previews(&mut self, cx: &mut Context<Self>) {
        if self.view_mode == ViewMode::Original {
            self.stale_previews.extend(self.previews.drain(..).map(|pane| pane.image));
            self.previews_key = None;
            return;
        }
        let key = (self.ed.revision, self.view_mode);
        if self.previews_key == Some(key) {
            return;
        }
        self.previews_key = Some(key);
        let page = self.ed.doc.page().clone();
        let settings: Vec<ExportSettings> = match self.view_mode {
            ViewMode::FourUp => comparison_settings(page.export).to_vec(),
            _ => vec![page.export],
        };
        cx.spawn(async move |this, cx| {
            let panes = cx
                .background_executor()
                .spawn(async move {
                    settings
                        .iter()
                        .filter_map(|setting| {
                            let bytes = encode(&page, setting).ok()?;
                            let image = encoded_texture(&bytes)?;
                            Some(PreviewPane { image, caption: format!("{}   {}", settings_label(setting), format_size(bytes.len())) })
                        })
                        .collect::<Vec<_>>()
                })
                .await;
            this.update(cx, |this, cx| {
                if this.previews_key == Some(key) {
                    let old = std::mem::replace(&mut this.previews, panes);
                    this.stale_previews.extend(old.into_iter().map(|pane| pane.image));
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.finish_pending();
        self.stop_playing();
        self.view_mode = mode;
    }

    /// Switches between the full dock and one that shows only its tabs.
    pub fn set_compact_workspace(&mut self, compact: bool) {
        self.compact_workspace = compact;
        for (index, group) in self.groups.iter_mut().enumerate() {
            group.collapsed = compact || index >= EXPANDED_GROUPS;
        }
    }

    fn set_screen_mode(&mut self, mode: ScreenMode, window: &mut Window) {
        if window.is_fullscreen() != (mode != ScreenMode::Standard) {
            window.toggle_fullscreen();
        }
        self.panels_off = mode == ScreenMode::Full;
    }

    fn open_size_dialog(&mut self, kind: SizeDialogKind) {
        let page = self.ed.doc.page();
        self.size_dialog = Some(SizeDialog { kind, width: page.width, height: page.height, constrain: kind == SizeDialogKind::Image });
    }

    /// Applies the open size dialog and closes it.
    pub fn apply_size_dialog(&mut self) {
        self.commit_field();
        let Some(dialog) = self.size_dialog.take() else { return };
        match dialog.kind {
            SizeDialogKind::Canvas => {
                self.ed.resize_canvas(dialog.width, dialog.height);
                self.ed.commit("Canvas Size");
            }
            SizeDialogKind::Image => {
                self.ed.resize_image(dialog.width, dialog.height);
                self.ed.commit("Image Size");
            }
        }
    }

    pub fn go_to_state(&mut self, index: usize) {
        self.ed.selection.clear();
        self.ed.set_active_state(index);
    }

    /// Neighbouring states, faded, when onion skinning is on.
    pub fn onion_ghosts(&self) -> Vec<Object> {
        if !self.onion {
            return Vec::new();
        }
        let page = self.ed.doc.page();
        [page.active_state.checked_sub(1), Some(page.active_state + 1)]
            .into_iter()
            .flatten()
            .flat_map(|state| page.state_objects(state))
            .filter(|o| o.visible && !o.is_web())
            .map(|o| {
                let mut ghost = o.clone();
                ghost.opacity *= ONION_OPACITY;
                ghost
            })
            .collect()
    }

    pub fn stop_playing(&mut self) {
        self.playing = false;
    }

    pub fn toggle_play(&mut self, cx: &mut Context<Self>) {
        if self.playing {
            self.playing = false;
            return;
        }
        if self.ed.doc.page().states.iter().filter(|s| s.exported).count() < 2 {
            self.status = "Add a second state to play an animation".into();
            return;
        }
        self.playing = true;
        self.play_token += 1;
        self.schedule_frame(self.play_token, cx);
    }

    fn schedule_frame(&mut self, token: u64, cx: &mut Context<Self>) {
        let page = self.ed.doc.page();
        let delay = page.states[page.active_state].delay_ms.max(MIN_FRAME_MS) as u64;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(delay)).await;
            this.update(cx, |this, cx| {
                if this.playing && this.play_token == token {
                    this.next_frame();
                    this.schedule_frame(token, cx);
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn next_frame(&mut self) {
        let page = self.ed.doc.page();
        let count = page.states.len();
        let next = (1..=count).map(|step| (page.active_state + step) % count).find(|i| page.states[*i].exported);
        if let Some(next) = next {
            self.ed.set_active_state(next);
        }
    }

    pub fn use_style(&mut self, index: usize) {
        self.style_selection = Some(index);
        let Some(style) = self.ed.doc.styles.get(index).cloned() else { return };
        if self.ed.selection.is_empty() {
            self.fill = style.fill;
            self.stroke = style.stroke.map(|s| s.color);
            if let Some(stroke) = style.stroke {
                self.stroke_width = stroke.width;
            }
            return;
        }
        self.ed.apply_style(&style);
        self.ed.commit("Apply Style");
    }

    fn run_path_op(&mut self, op: PathOp) {
        match self.ed.path_op(op, self.path_amount) {
            Ok(_) => self.ed.commit(op.label()),
            Err(message) => self.status = message.to_string(),
        }
    }

    pub fn insert_auto_shape(&mut self, kind: AutoKind) {
        let d = kind.defaults();
        let center = self.ed.doc.page().rect().center();
        let bounds = Rect {
            x: (center.x - AUTO_SHAPE_SIZE / 2.0).round(),
            y: (center.y - AUTO_SHAPE_SIZE / 2.0).round(),
            w: AUTO_SHAPE_SIZE,
            h: AUTO_SHAPE_SIZE,
        };
        let object = self.new_object(kind.label(), Shape::Auto { kind, a: d.a, b: d.b, count: d.count }, bounds, Vec::new());
        self.add_new(object, "Auto Shape");
    }

    pub fn place_symbol(&mut self, id: u64) {
        if self.ed.place_symbol(id) {
            self.ed.commit("Insert Symbol");
        } else {
            self.status = "All layers are locked".into();
        }
    }

    pub fn insert_common(&mut self, index: usize) {
        let Some(item) = common_items().get(index) else { return };
        let id = self.ed.add_library_symbol(item.name, &item.objects());
        self.symbol_selection = Some(id);
        self.place_symbol(id);
    }

    fn make_symbol(&mut self) {
        match self.ed.make_symbol() {
            Some(id) => {
                self.symbol_selection = Some(id);
                self.pixel_sel = None;
                self.ed.commit("Convert to Symbol");
            }
            None => self.status = "Select objects to make a symbol from".into(),
        }
    }

    /// Assigns a stored URL to the selected hotspots.
    pub fn use_url(&mut self, index: usize) {
        self.url_selection = Some(index);
        let Some(link) = self.ed.doc.urls.get(index).cloned() else { return };
        self.ed.for_selected(|o| {
            if let Shape::Web { url, .. } = &mut o.shape {
                url.clone_from(&link);
            }
        });
        self.ed.commit("Hotspot Link");
    }

    fn set_mixer_channel(&mut self, channel: usize, value: u8) {
        let mut color = self.current_color(self.mixer_target).unwrap_or(Color::BLACK);
        match channel {
            0 => color.r = value,
            1 => color.g = value,
            _ => color.b = value,
        }
        self.apply_color(self.mixer_target, Some(color));
    }

    pub fn mixer_pick(&mut self, channel: usize, position: Point<Pixels>) {
        let bounds = self.mixer_bounds[channel].get();
        let width = f32::from(bounds.size.width);
        if width > 0.0 {
            let fraction = (f32::from(position.x - bounds.origin.x) / width).clamp(0.0, 1.0);
            self.set_mixer_channel(channel, (fraction * 255.0).round() as u8);
        }
    }

    /// The color visible at a document point, ignoring web areas.
    pub fn color_under(&self, p: Vec2) -> Option<Color> {
        let page = self.ed.doc.page();
        let background = page.rect().contains(p).then_some(page.canvas).flatten();
        let Some(object) = page.art_at(p, 0.0).and_then(|id| self.ed.object(id)) else {
            return background;
        };
        // A symbol has no color of its own, so read the part under the pointer.
        let parts;
        let object = if object.is_instance() {
            parts = object.leaves();
            match parts.iter().rev().find(|part| part.hit(p, 0.0)) {
                Some(part) => part,
                None => return background,
            }
        } else {
            object
        };
        if object.is_image() {
            let [r, g, b, _] = object.pixel_at(p).and_then(|(x, y)| object.bitmap()?.pixel(x, y))?;
            return Some(Color::rgb(r, g, b));
        }
        object.fill.or(object.stroke.map(|s| s.color))
    }

    /// Selects matches, or replaces them everywhere the search covers.
    pub fn run_find(&mut self, replace: bool) {
        let (find, change) = (self.find_text.clone(), self.replace_text.clone());
        if find.is_empty() {
            self.status = "Type what to find first".into();
            return;
        }
        let colors = match self.find_mode {
            FindMode::Text => None,
            FindMode::Color => match (Color::parse_hex(&find), Color::parse_hex(&change).or(if replace { None } else { Some(Color::BLACK) })) {
                (Some(from), Some(to)) => Some((from, to)),
                _ => {
                    self.status = "Enter colors as hex, such as #FF0000".into();
                    return;
                }
            },
        };
        let matches = |object: &Object| match colors {
            Some((from, _)) => object.fill == Some(from) || object.stroke.is_some_and(|s| s.color == from),
            None => matches!(&object.shape, Shape::Text { content, .. } if content.contains(&find)),
        };
        if !replace {
            self.ed.selection = self.ed.doc.page().objects().filter(|o| matches(o)).map(|o| o.id).collect();
            let count = self.ed.selection.len();
            self.status = format!("Found {count} on this page");
            return;
        }
        let active = self.ed.doc.active_page;
        let mut count = 0;
        for (index, page) in self.ed.doc.pages.iter_mut().enumerate() {
            if !self.find_all_pages && index != active {
                continue;
            }
            for object in page.all_objects_mut() {
                if !matches(object) {
                    continue;
                }
                count += 1;
                if let Some((from, to)) = colors {
                    if object.fill == Some(from) {
                        object.fill = Some(to);
                    }
                    if let Some(stroke) = object.stroke.as_mut().filter(|s| s.color == from) {
                        stroke.color = to;
                    }
                } else if let Shape::Text { content, size } = &mut object.shape {
                    *content = content.replace(&find, &change);
                    (object.bounds.w, object.bounds.h) = measure_text(content, *size);
                }
            }
        }
        self.ed.commit("Replace");
        self.status = format!("Replaced {count}");
    }

    /// Adds a character to the text being edited, or to the selected text object.
    pub fn insert_special(&mut self, ch: char) {
        let text = ch.to_string();
        if self.field.is_some() || self.text_edit.is_some() {
            self.insert_typed(&text);
            return;
        }
        let mut changed = false;
        self.ed.for_selected(|o| {
            if let Shape::Text { content, size } = &mut o.shape {
                content.push(ch);
                (o.bounds.w, o.bounds.h) = measure_text(content, *size);
                changed = true;
            }
        });
        if changed {
            self.ed.commit("Special Character");
        } else {
            self.status = "Select a text object, or start typing with the Text tool".into();
        }
    }

    pub fn root_mouse_down(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus);
        if self.keep_overlays {
            self.keep_overlays = false;
        } else {
            self.close_overlays();
            self.commit_field();
        }
        cx.notify();
    }

    // ---- Commands -------------------------------------------------------------------

    pub fn is_enabled(&self, cmd: Cmd) -> bool {
        let has_selection = !self.ed.selection.is_empty();
        match cmd {
            Cmd::Undo => self.ed.can_undo(),
            Cmd::Redo => self.ed.can_redo(),
            Cmd::Paste => self.ed.has_clipboard(),
            Cmd::Cut | Cmd::Copy | Cmd::Clear | Cmd::Duplicate | Cmd::Deselect | Cmd::Arrange(_) | Cmd::Align(_)
            | Cmd::FlipH | Cmd::FlipV => has_selection,
            Cmd::Filter(_) => self.ed.has_selected_bitmap(),
            Cmd::ConvertToSymbol => has_selection,
            Cmd::BreakApart | Cmd::Ungroup => self.ed.selected().iter().any(|o| o.is_instance()),
            Cmd::Group => self.ed.selection.len() >= 2,
            Cmd::Rotate { .. } => has_selection,
            Cmd::FirstState | Cmd::PreviousState | Cmd::NextState | Cmd::LastState => self.ed.doc.page().states.len() > 1,
            Cmd::PathOp(op) => self.ed.selection.len() >= if op.needs_two() { 2 } else { 1 },
            _ => true,
        }
    }

    pub fn run(&mut self, cmd: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        self.close_overlays();
        self.finish_pending();
        if !self.is_enabled(cmd) {
            return;
        }
        if !matches!(cmd, Cmd::Panel(_) | Cmd::HidePanels) {
            self.stop_playing();
        }
        let pixels = self.active_pixels().map(|s| (s.object, s.mask.clone()));
        match cmd {
            Cmd::New | Cmd::Open | Cmd::Exit => self.confirm_discard(cmd, window, cx),
            Cmd::Save => self.save(false, window, cx),
            Cmd::SaveAs => self.save(true, window, cx),
            Cmd::Import => self.pick_file(window, cx, |this, path| this.import_file(&path)),
            Cmd::Export => self.export(window, cx),
            Cmd::Undo => self.ed.undo(),
            Cmd::Redo => self.ed.redo(),
            Cmd::Cut => {
                match &pixels {
                    Some((id, mask)) => {
                        self.ed.copy_pixels(*id, mask);
                        self.ed.clear_pixels(*id, mask);
                    }
                    None => {
                        self.ed.copy();
                        self.ed.delete_selected();
                    }
                }
                self.ed.commit("Cut");
            }
            Cmd::Copy => match &pixels {
                Some((id, mask)) => self.ed.copy_pixels(*id, mask),
                None => self.ed.copy(),
            },
            Cmd::Paste => {
                self.pixel_sel = None;
                self.ed.paste(PASTE_OFFSET);
                self.ed.commit("Paste");
            }
            Cmd::Clear => {
                match &pixels {
                    Some((id, mask)) => self.ed.clear_pixels(*id, mask),
                    None => self.ed.delete_selected(),
                }
                self.ed.commit("Clear");
            }
            Cmd::Duplicate => {
                self.ed.duplicate(PASTE_OFFSET);
                self.ed.commit("Duplicate");
            }
            Cmd::ZoomIn => self.step_zoom(true, None),
            Cmd::ZoomOut => self.step_zoom(false, None),
            Cmd::ZoomTo(percent) => self.zoom_about(percent as f32 / 100.0, None),
            Cmd::FitAll => self.fit_all(),
            Cmd::Rulers => self.show_rulers = !self.show_rulers,
            Cmd::Grid => self.show_grid = !self.show_grid,
            Cmd::SelectAll => self.ed.select_all(),
            // The first Deselect drops the pixel selection, the next the object.
            Cmd::Deselect => {
                if self.pixel_sel.take().is_none() {
                    self.ed.selection.clear();
                }
            }
            Cmd::Arrange(op) => {
                self.ed.arrange(op);
                self.ed.commit("Arrange");
            }
            Cmd::Align(op) => {
                self.ed.align(op);
                self.ed.commit("Align");
            }
            Cmd::FlipH | Cmd::FlipV => {
                self.ed.for_selected(|o| o.flip(matches!(cmd, Cmd::FlipH)));
                self.ed.commit("Flip");
            }
            Cmd::Filter(filter) => {
                self.ed.apply_filter(filter, pixels.as_ref().map(|(id, mask)| (*id, mask.as_ref())));
                self.ed.commit(filter.label());
            }
            Cmd::ShowWeb(visible) => {
                self.ed.set_web_visible(visible);
                self.ed.commit(if visible { "Show Slices" } else { "Hide Slices" });
            }
            Cmd::AddLayer => {
                self.ed.add_layer();
                self.ed.commit("New Layer");
            }
            Cmd::AddPage => {
                self.ed.add_page();
                self.ed.commit("New Page");
            }
            Cmd::Panel(panel) => self.toggle_panel(panel),
            Cmd::HidePanels => self.panels_off = !self.panels_off,
            Cmd::ConvertToSymbol => self.make_symbol(),
            Cmd::BreakApart => {
                self.ed.break_apart();
                self.ed.commit("Break Apart");
            }
            Cmd::PathOp(op) => self.run_path_op(op),
            Cmd::Group => {
                if self.ed.group_selected() {
                    self.ed.commit("Group");
                }
            }
            Cmd::Ungroup => {
                self.ed.break_apart();
                self.ed.commit("Ungroup");
            }
            Cmd::Rotate { clockwise } => {
                let skipped = self.ed.rotate_selected(clockwise);
                self.ed.commit("Rotate");
                if skipped > 0 {
                    self.status = format!("{skipped} text or symbol object(s) could not be rotated");
                }
            }
            Cmd::CanvasSize => self.open_size_dialog(SizeDialogKind::Canvas),
            Cmd::ImageSize => self.open_size_dialog(SizeDialogKind::Image),
            Cmd::FitCanvas => {
                if self.ed.fit_canvas() {
                    self.ed.commit("Fit Canvas");
                } else {
                    self.status = "There is no artwork to fit the canvas to".into();
                }
            }
            Cmd::ScreenMode(mode) => self.set_screen_mode(mode, window),
            Cmd::FirstState => self.go_to_state(0),
            Cmd::LastState => {
                let last = self.ed.doc.page().states.len() - 1;
                self.go_to_state(last);
            }
            Cmd::PreviousState | Cmd::NextState => {
                let page = self.ed.doc.page();
                let count = page.states.len();
                let step = if matches!(cmd, Cmd::NextState) { 1 } else { count - 1 };
                let target = (page.active_state + step) % count;
                self.go_to_state(target);
            }
            Cmd::PlayStates => self.toggle_play(cx),
            Cmd::Close => self.confirm_discard(Cmd::New, window, cx),
            Cmd::About => {
                let _ = window.prompt(
                    PromptLevel::Info,
                    APP_NAME,
                    Some(concat!("Version ", env!("CARGO_PKG_VERSION"))),
                    &["OK"],
                    cx,
                );
            }
        }
        cx.notify();
    }

    fn confirm_discard(&mut self, cmd: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        if !self.ed.is_dirty() {
            self.perform_destructive(cmd, window, cx);
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            "Discard unsaved changes?",
            Some("Your changes to this document will be lost."),
            &["Discard", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if answer.await == Ok(0) {
                this.update_in(cx, |this, window, cx| this.perform_destructive(cmd, window, cx)).ok();
            }
        })
        .detach();
    }

    fn perform_destructive(&mut self, cmd: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        match cmd {
            Cmd::New => {
                self.load_document(Document::new(DEFAULT_WIDTH, DEFAULT_HEIGHT), None);
                self.status.clear();
                cx.notify();
            }
            Cmd::Open => self.pick_file(window, cx, |this, path| this.open_file(path)),
            Cmd::Exit => cx.quit(),
            _ => {}
        }
    }

    fn load_document(&mut self, doc: Document, path: Option<PathBuf>) {
        self.ed = Editor::new(doc);
        self.path = path;
        self.source_image = None;
        self.crop_area = None;
        self.pixel_sel = None;
        self.stamp = None;
        self.pan = Vec2::default();
        self.zoom = 1.0;
        self.pen.clear();
        self.text_edit = None;
        self.drag = None;
        self.playing = false;
        self.collapsed_layers.clear();
        self.view_mode = ViewMode::Original;
        self.estimate_key = None;
        self.previews_key = None;
        self.style_selection = None;
        self.symbol_selection = None;
        self.url_selection = None;
    }

    fn pick_file(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        handle: impl FnOnce(&mut Self, PathBuf) + 'static,
    ) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else { return };
            let Some(path) = paths.pop() else { return };
            this.update(cx, |this, cx| {
                handle(this, path);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn open_file(&mut self, path: PathBuf) {
        let opened = std::fs::read(&path).map_err(anyhow::Error::from).and_then(|bytes| parse_file(&bytes));
        match opened {
            Ok(Opened::Document(doc)) => {
                self.status = format!("Opened {}", path.file_name().unwrap_or_default().to_string_lossy());
                self.load_document(doc, Some(path));
            }
            Ok(Opened::Image(bitmap)) => {
                self.status = format!("Opened {}", path.file_name().unwrap_or_default().to_string_lossy());
                let doc = Document::from_bitmap(file_stem(&path), bitmap);
                let id = doc.next_id - 1;
                self.load_document(doc, None);
                self.ed.selection = vec![id];
                self.source_image = Some(path);
                self.fit_if_larger_than_view();
            }
            Err(error) => self.status = format!("Could not open file: {error}"),
        }
    }

    /// Adds an image to the current document, centered on the canvas.
    fn import_file(&mut self, path: &Path) {
        let decoded = std::fs::read(path).map_err(anyhow::Error::from).and_then(|bytes| Bitmap::decode(&bytes));
        match decoded {
            Ok(bitmap) => {
                let page = self.ed.doc.page();
                let x = ((page.width as f32 - bitmap.width() as f32) / 2.0).round();
                let y = ((page.height as f32 - bitmap.height() as f32) / 2.0).round();
                self.status = format!("Imported {}", path.display());
                let object = Object::new_bitmap(self.ed.alloc_id(), file_stem(path), bitmap, x, y);
                self.add_new(object, "Import");
            }
            Err(error) => self.status = format!("Could not import file: {error}"),
        }
    }

    fn save(&mut self, choose_path: bool, window: &mut Window, cx: &mut Context<Self>) {
        match (self.path.clone(), choose_path) {
            (Some(path), false) => self.write_document(path),
            _ => {
                let name = self.save_name();
                let picked = cx.prompt_for_new_path(&Self::default_dir(), Some(&name));
                cx.spawn_in(window, async move |this, cx| {
                    if let Ok(Ok(Some(path))) = picked.await {
                        this.update(cx, |this, cx| {
                            this.write_document(path);
                            cx.notify();
                        })
                        .ok();
                    }
                })
                .detach();
            }
        }
    }

    fn default_dir() -> PathBuf {
        std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
    }

    fn write_document(&mut self, mut path: PathBuf) {
        // Saving under an image name writes a flattened copy, and the editable
        // document stays unsaved. JSON must never land in a file named .png.
        if path.extension().is_some_and(|e| !e.eq_ignore_ascii_case(FILE_EXTENSION)) {
            self.export_to(&path);
            return;
        }
        if path.extension().is_none() {
            path.set_extension(FILE_EXTENSION);
        }
        let result = document_to_json(&self.ed.doc)
            .and_then(|json| std::fs::write(&path, json).map_err(anyhow::Error::from));
        match result {
            Ok(()) => {
                self.ed.mark_saved();
                self.status = format!("Saved {}", path.display());
                self.path = Some(path);
            }
            Err(error) => self.status = format!("Could not save: {error}"),
        }
    }

    fn export_to(&mut self, path: &Path) {
        let page = self.ed.doc.page();
        self.status = match export_page(page, path).and_then(|()| export_web(page, path)) {
            Ok(0) => format!("Exported {}", path.display()),
            Ok(extra) => format!("Exported {} and {extra} web file(s) beside it", path.display()),
            Err(error) => format!("Export failed: {error:#}"),
        };
    }

    pub fn export(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let stem = self.path.as_ref().or(self.source_image.as_ref()).map_or_else(|| "Untitled".into(), |p| file_stem(p));
        let extension = self.ed.doc.page().export.format.extension();
        let picked = cx.prompt_for_new_path(&Self::default_dir(), Some(&format!("{stem}.{extension}")));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(mut path))) = picked.await {
                if path.extension().is_none() {
                    path.set_extension(extension);
                }
                this.update(cx, |this, cx| {
                    this.export_to(&path);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// Handles a window close request. Returns true when the window may close.
    pub fn request_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.finish_pending();
        if !self.ed.is_dirty() {
            return true;
        }
        self.confirm_discard(Cmd::Exit, window, cx);
        false
    }

    // ---- View -----------------------------------------------------------------------

    fn zoom_about(&mut self, zoom: f32, anchor: Option<Point<Pixels>>) {
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if let Some(anchor) = anchor {
            let map = self.view_map();
            let doc = map.to_doc(anchor);
            let page = self.ed.doc.page();
            let bounds = self.canvas_bounds.get();
            let (w, h) = (page.width as f32, page.height as f32);
            let new_map = ViewMap::new(bounds, w, h, zoom, self.pan);
            let moved = new_map.to_screen(doc);
            self.pan.x += f32::from(anchor.x - moved.x);
            self.pan.y += f32::from(anchor.y - moved.y);
        }
        self.zoom = zoom;
    }

    fn step_zoom(&mut self, zoom_in: bool, anchor: Option<Point<Pixels>>) {
        let current = self.zoom * 100.0;
        let next = if zoom_in {
            ZOOM_LEVELS.iter().find(|l| **l as f32 > current + 0.01)
        } else {
            ZOOM_LEVELS.iter().rev().find(|l| (**l as f32) < current - 0.01)
        };
        if let Some(level) = next {
            self.zoom_about(*level as f32 / 100.0, anchor);
        }
    }

    fn fit_if_larger_than_view(&mut self) {
        let bounds = self.canvas_bounds.get();
        let page = self.ed.doc.page();
        if page.width as f32 > f32::from(bounds.size.width) || page.height as f32 > f32::from(bounds.size.height) {
            self.fit_all();
        }
    }

    fn fit_all(&mut self) {
        let bounds = self.canvas_bounds.get();
        let page = self.ed.doc.page();
        let margin = 48.0;
        let fit_w = (f32::from(bounds.size.width) - margin) / page.width as f32;
        let fit_h = (f32::from(bounds.size.height) - margin) / page.height as f32;
        self.zoom = fit_w.min(fit_h).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = Vec2::default();
    }

    // ---- Canvas input ---------------------------------------------------------------

    pub fn canvas_down(&mut self, ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_field();
        window.focus(&self.focus);
        self.stop_playing();
        // The comparison views show finished images, so dragging only pans them.
        if self.view_mode != ViewMode::Original {
            self.drag = Some(Drag::Pan { last: ev.position });
            cx.notify();
            return;
        }
        let p = self.view_map().to_doc(ev.position);
        let tool = self.active_tool();
        if self.text_edit.is_some() {
            self.finish_text();
            if tool != Tool::Text {
                cx.notify();
                return;
            }
        }
        match tool {
            Tool::Pointer => self.pointer_down(ev, p),
            Tool::Scale => self.scale_down(ev, p),
            Tool::Subselect => self.subselect_down(p),
            Tool::Marquee | Tool::Lasso => match self.pixel_target(p) {
                Some(target) => {
                    self.drag = Some(Drag::PixelArea { target, points: vec![p], mode: selection_mode(ev.modifiers) });
                }
                None => self.status = "Selections work on bitmaps. Open or import an image first.".into(),
            },
            Tool::Wand => self.wand_click(ev, p),
            Tool::Blur => match self.ed.doc.page().bitmap_at(p, &self.ed.selection) {
                Some(id) => self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) }),
                None => self.status = "The Blur tool works on bitmaps.".into(),
            },
            Tool::Stamp => self.stamp_down(ev, p),
            Tool::Freeform => self.freeform_down(p),
            Tool::Rect | Tool::Ellipse | Tool::Line | Tool::Knife | Tool::Hotspot | Tool::Slice => {
                let start = snap(p);
                self.drag = Some(Drag::Create { start, current: start });
            }
            Tool::Pen => self.pen_click(p, ev.click_count),
            Tool::Pencil | Tool::Brush => {
                let target = self.ed.doc.page().bitmap_at(p, &self.ed.selection);
                self.drag = Some(Drag::Freehand { points: vec![p], target });
            }
            Tool::Eraser => match self.ed.doc.page().bitmap_at(p, &self.ed.selection) {
                Some(id) => self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) }),
                None => self.status = "The Eraser works on bitmaps. Open or import an image first.".into(),
            },
            Tool::Crop => {
                if ev.click_count >= 2 && self.crop_area.is_some() {
                    self.apply_crop();
                } else {
                    let start = snap(p);
                    self.drag = Some(Drag::Crop { start, current: start });
                }
            }
            Tool::Text => self.begin_text(snap(p)),
            Tool::Eyedropper => self.pick_color(p),
            Tool::Bucket => self.bucket(p),
            Tool::Hand => self.drag = Some(Drag::Pan { last: ev.position }),
            Tool::Zoom => self.step_zoom(!ev.modifiers.alt, Some(ev.position)),
        }
        cx.notify();
    }

    fn tolerance(&self) -> f32 {
        3.0 / self.zoom
    }

    fn pointer_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        if let Some(bounds) = self.ed.selection_bounds() {
            if self.ed.selection.len() == 1 {
                let map = self.view_map();
                let handle = handle_rects(bounds, map).iter().position(|r| r.contains(&ev.position));
                if let Some(handle) = handle {
                    self.drag = Some(Drag::Resize { handle, origin: bounds });
                    return;
                }
            }
        }
        match self.ed.doc.page().hit_test(p, self.tolerance()) {
            Some(id) => {
                if ev.modifiers.shift {
                    if self.ed.is_selected(id) {
                        self.ed.selection.retain(|s| *s != id);
                        return;
                    }
                    self.ed.selection.push(id);
                } else if !self.ed.is_selected(id) {
                    self.ed.selection = vec![id];
                }
                self.drag = Some(Drag::Move { last: p });
            }
            None => {
                if !ev.modifiers.shift {
                    self.ed.selection.clear();
                }
                self.drag = Some(Drag::Marquee { start: p, current: p });
            }
        }
    }

    fn scale_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        if let Some(origin) = self.ed.selection_bounds() {
            let map = self.view_map();
            if let Some(handle) = handle_rects(origin, map).iter().position(|r| r.contains(&ev.position)) {
                let originals = self.ed.selected().into_iter().cloned().collect();
                self.drag = Some(Drag::Scale { handle, origin, originals });
                return;
            }
        }
        self.pointer_down(ev, p);
    }

    /// The bitmap a selection drag applies to: the one under the pointer, or the selected one.
    fn pixel_target(&self, p: Vec2) -> Option<u64> {
        self.ed
            .doc
            .page()
            .bitmap_at(p, &self.ed.selection)
            .or_else(|| self.ed.selected().into_iter().find(|o| o.is_image()).map(|o| o.id))
    }

    fn set_pixel_selection(&mut self, target: u64, new: Option<Mask>, mode: MaskMode) {
        let previous = self.mask_for(target);
        let combined = match (mode, previous) {
            (MaskMode::Replace, _) | (MaskMode::Add, None) => new,
            (MaskMode::Subtract, None) => None,
            (_, Some(previous)) => {
                let mut combined = (*previous).clone();
                if let Some(mask) = &new {
                    combined.combine(mask, mode);
                }
                Some(combined)
            }
        };
        self.pixel_sel = combined.filter(|m| !m.is_empty()).map(|mask| PixelSelection {
            object: target,
            edges: Arc::new(mask.edges()),
            mask: Arc::new(mask),
        });
        if self.pixel_sel.is_some() {
            self.ed.selection = vec![target];
        }
    }

    fn finish_pixel_area(&mut self, target: u64, points: Vec<Vec2>, mode: MaskMode) {
        let Some(object) = self.ed.object(target) else { return };
        let Some((w, h)) = object.bitmap().map(|b| (b.width(), b.height())) else { return };
        let pixels: Vec<Vec2> = points.iter().filter_map(|p| object.doc_to_pixel(*p)).collect();
        let mask = match (self.tool, pixels.as_slice()) {
            (Tool::Marquee, [a, b]) => Some(Mask::rect(w, h, *a, *b)),
            (Tool::Lasso, [_, _, _, ..]) => Some(Mask::polygon(w, h, &pixels)),
            _ => None,
        };
        self.set_pixel_selection(target, mask.filter(|m| !m.is_empty()), mode);
    }

    fn wand_click(&mut self, ev: &MouseDownEvent, p: Vec2) {
        let Some(id) = self.ed.doc.page().bitmap_at(p, &self.ed.selection) else {
            self.status = "The Magic Wand works on bitmaps.".into();
            return;
        };
        let Some(object) = self.ed.object(id) else { return };
        let region = object.pixel_at(p).and_then(|(x, y)| object.bitmap()?.region(x, y, FILL_TOLERANCE));
        self.set_pixel_selection(id, region, selection_mode(ev.modifiers));
    }

    fn stamp_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        let Some(id) = self.ed.doc.page().bitmap_at(p, &self.ed.selection) else {
            self.status = "The Rubber Stamp works on bitmaps.".into();
            return;
        };
        if ev.modifiers.alt {
            self.stamp = Some(Stamp { object: id, source: p, offset: None });
            self.status = "Clone source set. Drag to paint.".into();
            return;
        }
        match self.stamp.as_mut().filter(|s| s.object == id) {
            Some(stamp) => {
                let source = stamp.source;
                stamp.offset.get_or_insert(v2(source.x - p.x, source.y - p.y));
                self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) });
            }
            None => self.status = "Alt+click this image to set the clone source first.".into(),
        }
    }

    fn freeform_down(&mut self, p: Vec2) {
        let radius = self.freeform_size / 2.0;
        let Some(id) = self.ed.doc.page().nearest_path(p, radius, &self.ed.selection) else {
            self.status = "Press near a vector path or shape to reshape it.".into();
            return;
        };
        let Some(object) = self.ed.object_mut(id) else { return };
        if !object.to_path() {
            return;
        }
        let base = subdivide(&object.points, object.is_closed(), (radius * FREEFORM_DETAIL).max(1.0));
        object.points = base.clone();
        self.ed.selection = vec![id];
        self.drag = Some(Drag::Freeform { id, start: p, base, radius });
    }

    fn subselect_down(&mut self, p: Vec2) {
        let reach = 5.0 / self.zoom;
        for object in self.ed.selected() {
            if !matches!(object.shape, Shape::Line | Shape::Path { .. }) {
                continue;
            }
            let near = |v: &Vec2| ((v.x - p.x).powi(2) + (v.y - p.y).powi(2)).sqrt() <= reach;
            if let Some(index) = object.points.iter().position(near) {
                self.drag = Some(Drag::Vertex { id: object.id, index });
                return;
            }
        }
        self.ed.selection = self.ed.doc.page().hit_test(p, self.tolerance()).into_iter().collect();
    }

    fn pick_color(&mut self, p: Vec2) {
        let Some(id) = self.ed.doc.page().art_at(p, self.tolerance()) else { return };
        let Some(object) = self.ed.object(id) else { return };
        if object.is_image() {
            // A picked pixel loads both colors so the Brush and Bucket both use it.
            let pixel = object.pixel_at(p).and_then(|(x, y)| object.bitmap()?.pixel(x, y));
            if let Some([r, g, b, _]) = pixel.filter(|px| px[3] > 0) {
                let color = Color::rgb(r, g, b);
                self.fill = Some(color);
                self.stroke = Some(color);
                self.status = format!("Picked {}", color.hex());
            }
            return;
        }
        if let Some(color) = object.fill.or(object.stroke.map(|s| s.color)) {
            self.fill = Some(color);
            self.status = format!("Picked {}", color.hex());
        }
    }

    fn bucket(&mut self, p: Vec2) {
        let fill = self.fill;
        let Some(id) = self.ed.doc.page().art_at(p, self.tolerance()) else { return };
        if self.ed.object(id).is_some_and(Object::is_image) {
            let Some(color) = fill else {
                self.status = "Choose a fill color first".into();
                return;
            };
            let mask = self.mask_for(id);
            let Some(object) = self.ed.object_mut(id) else { return };
            let Some((x, y)) = object.pixel_at(p) else { return };
            object.edit_pixels(mask.as_deref(), |bitmap| {
                bitmap.flood_fill(x, y, color, FILL_TOLERANCE);
            });
            self.ed.commit("Paint Bucket");
            return;
        }
        if let Some(object) = self.ed.object_mut(id) {
            object.fill = fill;
        }
        self.ed.commit("Paint Bucket");
    }

    fn new_object(&mut self, name: &str, shape: Shape, bounds: Rect, points: Vec<Vec2>) -> Object {
        let is_stroke_only = matches!(shape, Shape::Line | Shape::Path { closed: false } | Shape::Auto { kind: AutoKind::Spiral, .. });
        Object {
            id: self.ed.alloc_id(),
            name: name.to_string(),
            shape,
            bounds,
            points,
            fill: if is_stroke_only { None } else { self.fill },
            stroke: self.stroke.map(|color| Stroke { color, width: self.stroke_width }),
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    fn add_new(&mut self, object: Object, label: &str) {
        if self.ed.add_object(object) {
            self.ed.commit(label);
        } else {
            self.status = "All layers are locked".into();
            self.ed.rollback();
        }
    }

    fn pen_click(&mut self, p: Vec2, clicks: usize) {
        let p = snap(p);
        if let Some(first) = self.pen.first().copied() {
            let reach = 5.0 / self.zoom;
            if self.pen.len() > 2 && (first.x - p.x).abs() <= reach && (first.y - p.y).abs() <= reach {
                self.finish_pen(true);
                return;
            }
        }
        if clicks >= 2 {
            self.finish_pen(false);
            return;
        }
        self.pen.push(p);
    }

    pub fn finish_pen(&mut self, closed: bool) {
        let points = std::mem::take(&mut self.pen);
        if points.len() < 2 {
            return;
        }
        let bounds = Rect::of_points(&points);
        let object = self.new_object("Path", Shape::Path { closed }, bounds, points);
        self.add_new(object, "Pen");
    }

    fn begin_text(&mut self, p: Vec2) {
        let size = 24.0;
        let (w, h) = measure_text("", size);
        let bounds = Rect { x: p.x, y: p.y, w, h };
        let mut object = self.new_object("Text", Shape::Text { content: String::new(), size }, bounds, Vec::new());
        object.fill = self.stroke.or(self.fill).or(Some(Color::BLACK));
        object.stroke = None;
        let id = object.id;
        if self.ed.add_object(object) {
            self.text_edit = Some(id);
        } else {
            self.status = "All layers are locked".into();
        }
    }

    fn edit_text(&mut self, edit: impl FnOnce(&mut String)) {
        let Some(id) = self.text_edit else { return };
        let Some(object) = self.ed.object_mut(id) else { return };
        if let Shape::Text { content, size } = &mut object.shape {
            edit(content);
            let (w, h) = measure_text(content, *size);
            object.bounds.w = w;
            object.bounds.h = h;
        }
    }

    pub fn finish_text(&mut self) {
        let Some(id) = self.text_edit.take() else { return };
        let empty = matches!(self.ed.object(id).map(|o| &o.shape), Some(Shape::Text { content, .. }) if content.is_empty());
        if empty {
            self.ed.selection = vec![id];
            self.ed.delete_selected();
            self.ed.rollback();
        } else {
            self.ed.commit("Text");
        }
    }

    pub fn mouse_move(&mut self, ev: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let map = self.view_map();
        let p = map.to_doc(ev.position);
        self.cursor_doc = p;
        let Some(drag) = self.drag.as_mut() else {
            if !self.pen.is_empty() || self.panel_showing(Panel::Info) {
                cx.notify();
            }
            return;
        };
        match drag {
            Drag::Move { last } => {
                let (dx, dy) = ((p.x - last.x).round(), (p.y - last.y).round());
                if dx != 0.0 || dy != 0.0 {
                    last.x += dx;
                    last.y += dy;
                    self.ed.for_selected(|o| o.translate(dx, dy));
                }
            }
            Drag::Resize { handle, origin } => {
                let target = resized(*origin, *handle, snap(p));
                self.ed.for_selected(|o| o.set_bounds(target));
            }
            Drag::Marquee { current, .. } => *current = p,
            Drag::Crop { current, .. } => *current = snap(p),
            Drag::PixelArea { points, .. } => {
                if self.tool == Tool::Marquee {
                    points.truncate(1);
                    points.push(p);
                } else if points.last().is_none_or(|l| (l.x - p.x).abs() + (l.y - p.y).abs() >= 1.5 / self.zoom) {
                    points.push(p);
                }
            }
            Drag::Scale { handle, origin, originals } => {
                let target = scaled(*origin, *handle, snap(p), !ev.modifiers.shift);
                for original in originals.iter() {
                    if let Some(object) = self.ed.object_mut(original.id) {
                        *object = scale_object(original, *origin, target);
                    }
                }
            }
            Drag::Freeform { id, start, base, radius } => {
                let delta = v2(p.x - start.x, p.y - start.y);
                let points = pull_points(base, *start, delta, *radius);
                if let Some(object) = self.ed.object_mut(*id) {
                    object.points = points;
                    object.refit_to_points();
                }
            }
            Drag::Create { start, current } => {
                let raw = snap(p);
                *current = match (ev.modifiers.shift, self.tool) {
                    (true, Tool::Line | Tool::Knife) => constrain_angle(*start, raw),
                    (true, _) => constrain_square(*start, raw),
                    _ => raw,
                };
            }
            Drag::Freehand { points, .. } => {
                let min_step = 1.5 / self.zoom;
                let far = points.last().is_none_or(|l| ((l.x - p.x).powi(2) + (l.y - p.y).powi(2)).sqrt() >= min_step);
                if far {
                    points.push(p);
                }
            }
            Drag::Vertex { id, index } => {
                let (id, index) = (*id, *index);
                if let Some(object) = self.ed.object_mut(id) {
                    object.points[index] = snap(p);
                    object.refit_to_points();
                }
            }
            Drag::Pan { last } => {
                self.pan.x += f32::from(ev.position.x - last.x);
                self.pan.y += f32::from(ev.position.y - last.y);
                *last = ev.position;
            }
        }
        cx.notify();
    }

    pub fn mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(drag) = self.drag.take() else { return };
        match drag {
            Drag::Move { .. } => self.ed.commit("Move"),
            Drag::Resize { .. } => self.ed.commit("Scale"),
            Drag::Vertex { .. } => self.ed.commit("Edit Path"),
            Drag::Marquee { start, current } => self.ed.select_in_rect(Rect::from_corners(start, current)),
            Drag::Create { start, current } => self.finish_create(start, current),
            Drag::Freehand { points, target } => self.finish_freehand(points, target),
            Drag::Crop { start, current } => self.finish_crop_drag(start, current),
            Drag::PixelArea { target, points, mode } => self.finish_pixel_area(target, points, mode),
            Drag::Scale { .. } => self.ed.commit("Scale"),
            Drag::Freeform { .. } => self.ed.commit("Freeform"),
            Drag::Pan { .. } => {}
        }
        cx.notify();
    }

    fn finish_create(&mut self, start: Vec2, current: Vec2) {
        let bounds = Rect::from_corners(start, current);
        if bounds.w < 1.0 && bounds.h < 1.0 {
            return;
        }
        if self.tool == Tool::Knife {
            if self.ed.knife(start, current) > 0 {
                self.ed.commit("Knife");
            } else {
                self.status = "The Knife line did not cross a vector path.".into();
            }
            return;
        }
        let object = match self.tool {
            Tool::Hotspot => Object::new_web(self.ed.alloc_id(), WebKind::Hotspot, bounds),
            Tool::Slice => Object::new_web(self.ed.alloc_id(), WebKind::Slice, bounds),
            Tool::Rect =>self.new_object("Rectangle", Shape::Rect { radius: 0.0 }, bounds, Vec::new()),
            Tool::Ellipse => self.new_object("Ellipse", Shape::Ellipse, bounds, Vec::new()),
            _ => {
                let mut line = self.new_object("Line", Shape::Line, bounds, vec![start, current]);
                line.stroke = line.stroke.or(Some(Stroke { color: Color::BLACK, width: 1.0 }));
                line
            }
        };
        let label = object.name.clone();
        self.add_new(object, &label);
    }

    /// A click without a drag keeps the current area, so a double-click can apply it.
    fn finish_crop_drag(&mut self, start: Vec2, current: Vec2) {
        let dragged = Rect::from_corners(start, current);
        if dragged.w < 1.0 || dragged.h < 1.0 {
            return;
        }
        self.crop_area = self.ed.doc.page().rect().intersect(dragged);
        if self.crop_area.is_some() {
            self.status = "Press Enter or double-click to crop, Esc to cancel".into();
        }
    }

    fn apply_crop(&mut self) {
        let Some(area) = self.crop_area.take() else { return };
        if self.ed.crop_canvas(area) {
            self.ed.commit("Crop Document");
            self.status.clear();
        }
    }

    fn finish_freehand(&mut self, points: Vec<Vec2>, target: Option<u64>) {
        let width = self.freehand_width();
        let name = if self.tool == Tool::Pencil { "Pencil" } else { "Brush" };
        let color = self.stroke.unwrap_or(Color::BLACK);
        if let Some(id) = target {
            let offset = self.stamp.and_then(|s| s.offset).unwrap_or_default();
            let (op, label) = match self.tool {
                Tool::Eraser => (PixelStroke::Erase, "Eraser"),
                Tool::Blur => (PixelStroke::Blur, "Blur"),
                Tool::Stamp => (PixelStroke::Clone(offset), "Rubber Stamp"),
                _ => (PixelStroke::Paint(color), name),
            };
            let mask = self.mask_for(id);
            if let Some(object) = self.ed.object_mut(id) {
                object.stroke_bitmap(&points, width, op, mask.as_deref());
            }
            self.ed.commit(label);
            return;
        }
        if points.len() < 2 {
            return;
        }
        let bounds = Rect::of_points(&points);
        let mut object = self.new_object(name, Shape::Path { closed: false }, bounds, points);
        object.stroke = Some(Stroke { color, width });
        self.add_new(object, name);
    }

    pub fn canvas_scroll(&mut self, ev: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let (dx, dy) = match ev.delta {
            ScrollDelta::Pixels(d) => (f32::from(d.x), f32::from(d.y)),
            ScrollDelta::Lines(d) => (d.x * 40.0, d.y * 40.0),
        };
        if ev.modifiers.control {
            if dy != 0.0 {
                self.step_zoom(dy > 0.0, Some(ev.position));
            }
        } else if ev.modifiers.shift {
            self.pan.x += dy + dx;
        } else {
            self.pan.x += dx;
            self.pan.y += dy;
        }
        cx.notify();
    }

    // ---- Keyboard -------------------------------------------------------------------

    pub fn key_down(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let k = &ev.keystroke;
        let key = k.key.as_str();
        let m = k.modifiers;
        if let Some(cmd) = panel_shortcut(key, m.control, m.shift, m.alt) {
            self.run(cmd, window, cx);
        } else if m.control && key == "v" && (self.field.is_some() || self.text_edit.is_some()) {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                self.insert_typed(text.trim());
            }
        } else if self.field.is_some() {
            self.field_key(key);
        } else if self.text_edit.is_some() && !m.control {
            match key {
                "escape" => self.finish_text(),
                "backspace" => self.edit_text(|t| {
                    t.pop();
                }),
                "enter" => self.edit_text(|t| t.push('\n')),
                _ => {}
            }
        } else if self.size_dialog.is_some() {
            match key {
                "enter" => self.apply_size_dialog(),
                "escape" => self.size_dialog = None,
                _ => {}
            }
        } else if m.control {
            self.ctrl_key(key, m.shift, m.alt, window, cx);
        } else if !m.alt {
            self.plain_key(key, m.shift, window, cx);
        }
        cx.notify();
    }

    pub fn key_up(&mut self, ev: &KeyUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if ev.keystroke.key == "space" {
            self.space_held = false;
            cx.notify();
        }
    }

    fn ctrl_key(&mut self, key: &str, shift: bool, alt: bool, window: &mut Window, cx: &mut Context<Self>) {
        use crate::model::Arrange;
        let cmd = match (key, shift, alt) {
            ("n", false, false) => Cmd::New,
            ("o", false, false) => Cmd::Open,
            ("r", false, false) => Cmd::Import,
            ("s", false, false) => Cmd::Save,
            ("s", true, false) => Cmd::SaveAs,
            ("r", true, false) => Cmd::Export,
            ("z", false, false) => Cmd::Undo,
            ("z", true, false) | ("y", false, false) => Cmd::Redo,
            ("x", false, false) => Cmd::Cut,
            ("c", false, false) => Cmd::Copy,
            ("v", false, false) => Cmd::Paste,
            ("d", false, true) => Cmd::Duplicate,
            ("a", false, false) => Cmd::SelectAll,
            ("d", false, false) => Cmd::Deselect,
            ("=", _, false) | ("+", _, false) => Cmd::ZoomIn,
            ("-", false, false) => Cmd::ZoomOut,
            ("0", false, false) => Cmd::FitAll,
            ("1", false, false) => Cmd::ZoomTo(100),
            ("r", false, true) => Cmd::Rulers,
            ("g", false, true) => Cmd::Grid,
            ("f", false, false) => Cmd::Panel(Panel::Find),
            ("b", false, false) => Cmd::BreakApart,
            ("g", false, false) => Cmd::Group,
            ("g", true, false) => Cmd::Ungroup,
            ("up", true, false) => Cmd::Arrange(Arrange::Front),
            ("up", false, false) => Cmd::Arrange(Arrange::Forward),
            ("down", false, false) => Cmd::Arrange(Arrange::Backward),
            ("down", true, false) => Cmd::Arrange(Arrange::Back),
            _ => return,
        };
        self.run(cmd, window, cx);
    }

    fn plain_key(&mut self, key: &str, shift: bool, window: &mut Window, cx: &mut Context<Self>) {
        let tool = match key {
            "v" | "0" => Some(Tool::Pointer),
            "a" => Some(Tool::Subselect),
            "b" => Some(Tool::Brush),
            // Y and R are shared, so pressing them again cycles between the two tools.
            "y" => Some(if self.tool == Tool::Pencil { Tool::Knife } else { Tool::Pencil }),
            "r" => Some(if self.tool == Tool::Rect { Tool::Blur } else { Tool::Rect }),
            "e" => Some(Tool::Eraser),
            "c" => Some(Tool::Crop),
            "q" => Some(Tool::Scale),
            "m" => Some(Tool::Marquee),
            "l" => Some(Tool::Lasso),
            "w" => Some(Tool::Wand),
            "s" => Some(Tool::Stamp),
            "o" => Some(Tool::Freeform),
            "j" => Some(Tool::Hotspot),
            "k" => Some(Tool::Slice),
            "i" => Some(Tool::Eyedropper),
            "g" => Some(Tool::Bucket),
            "n" => Some(Tool::Line),
            "p" => Some(Tool::Pen),
            "u" => Some(Tool::Ellipse),
            "t" => Some(Tool::Text),
            "h" => Some(Tool::Hand),
            "z" => Some(Tool::Zoom),
            _ => None,
        };
        if let Some(tool) = tool {
            self.set_tool(tool);
            return;
        }
        let nudge = if shift { 10.0 } else { 1.0 };
        match key {
            "space" => self.space_held = true,
            "2" => {
                let visible = self.ed.web_visible();
                self.run(Cmd::ShowWeb(!visible), window, cx);
            }
            "delete" | "backspace" => self.run(Cmd::Clear, window, cx),
            "enter" if self.crop_area.is_some() => self.apply_crop(),
            "enter" => self.finish_pen(false),
            "escape" => {
                self.pen.clear();
                self.crop_area = None;
                self.pixel_sel = None;
                if self.drag.take().is_some() {
                    self.ed.rollback();
                }
                self.close_overlays();
            }
            "left" => self.nudge(-nudge, 0.0),
            "right" => self.nudge(nudge, 0.0),
            "up" => self.nudge(0.0, -nudge),
            "down" => self.nudge(0.0, nudge),
            _ => {}
        }
    }

    fn nudge(&mut self, dx: f32, dy: f32) {
        if !self.ed.selection.is_empty() {
            self.ed.for_selected(|o| o.translate(dx, dy));
            self.ed.commit("Move");
        }
    }

    // ---- Property fields ------------------------------------------------------------

    pub fn field_value(&self, id: FieldId) -> String {
        let number = |v: f32| {
            if v.fract().abs() < 0.005 { format!("{}", v.round() as i64) } else { format!("{v:.1}") }
        };
        let page = self.ed.doc.page();
        let selected = self.ed.selected();
        let first = selected.first().copied();
        let vector = self.first_vector();
        let bounds = self.ed.selection_bounds().unwrap_or_default();
        let hex = |c: Option<Color>| c.map_or(String::new(), Color::hex);
        match id {
            FieldId::X => number(bounds.x),
            FieldId::Y => number(bounds.y),
            FieldId::W => number(bounds.w),
            FieldId::H => number(bounds.h),
            FieldId::Radius => match first.map(|o| &o.shape) {
                Some(Shape::Rect { radius }) => number(*radius),
                _ => String::new(),
            },
            FieldId::TextSize => match first.map(|o| &o.shape) {
                Some(Shape::Text { size, .. }) => number(*size),
                _ => String::new(),
            },
            FieldId::StrokeWidth => number(vector.and_then(|o| o.stroke).map_or(self.stroke_width, |s| s.width)),
            FieldId::BrushSize => number(self.brush_size),
            FieldId::FreeformSize => number(self.freeform_size),
            FieldId::Url => match first.map(|o| &o.shape) {
                Some(Shape::Web { url, .. }) => url.clone(),
                _ => String::new(),
            },
            FieldId::Opacity => number(first.map_or(100.0, |o| o.opacity * 100.0)),
            FieldId::FillHex => hex(vector.map_or(self.fill, |o| o.fill)),
            FieldId::StrokeHex => hex(vector.map_or(self.stroke, |o| o.stroke.map(|s| s.color))),
            FieldId::CanvasHex => hex(page.canvas),
            FieldId::RenamePage(i) => self.ed.doc.pages.get(i).map_or_else(String::new, |p| p.name.clone()),
            FieldId::RenameState(i) => page.states.get(i).map_or_else(String::new, |s| s.name.clone()),
            FieldId::RenameLayer(i) => page.layers.get(i).map_or_else(String::new, |l| l.name.clone()),
            FieldId::RenameObject(id) => self.ed.object(id).map_or_else(String::new, |o| o.name.clone()),
            FieldId::RenameSymbol(id) => self.ed.doc.symbols.iter().find(|s| s.id == id).map_or_else(String::new, |s| s.name.clone()),
            FieldId::StateDelay(i) => page.states.get(i).map_or_else(String::new, |s| s.delay_ms.to_string()),
            FieldId::AutoParam(slot) => match selected.iter().find(|o| matches!(o.shape, Shape::Auto { .. })).map(|o| &o.shape) {
                Some(Shape::Auto { kind, a, b, count }) => {
                    let value = [*a, *b, *count as f32][slot.min(2)];
                    let percent = kind.params()[slot.min(2)].is_some_and(|info| info.max <= 1.0);
                    number(if percent { value * 100.0 } else { value })
                }
                _ => String::new(),
            },
            FieldId::PathAmount => number(self.path_amount),
            FieldId::OptimizeQuality => page.export.quality.to_string(),
            FieldId::OptimizeColors => page.export.colors.to_string(),
            FieldId::Mixer(channel) => {
                let color = self.current_color(self.mixer_target).unwrap_or(Color::BLACK);
                [color.r, color.g, color.b][channel.min(2)].to_string()
            }
            FieldId::FindText => self.find_text.clone(),
            FieldId::ReplaceText => self.replace_text.clone(),
            FieldId::NewUrl => String::new(),
            FieldId::DialogW => self.size_dialog.map_or_else(String::new, |d| d.width.to_string()),
            FieldId::DialogH => self.size_dialog.map_or_else(String::new, |d| d.height.to_string()),
        }
    }

    /// Bitmaps have no fill or stroke, so color controls read the first vector object.
    fn first_vector(&self) -> Option<&Object> {
        self.ed.selected().into_iter().find(|o| o.takes_color())
    }

    pub fn begin_field(&mut self, id: FieldId) {
        self.finish_pending();
        self.keep_overlays = true;
        self.field = Some(FieldEdit { id, buffer: self.field_value(id), fresh: true });
    }

    fn field_key(&mut self, key: &str) {
        match key {
            "enter" | "tab" => self.commit_field(),
            "escape" => self.field = None,
            "backspace" => {
                if let Some(field) = self.field.as_mut() {
                    if field.fresh {
                        field.buffer.clear();
                        field.fresh = false;
                    } else {
                        field.buffer.pop();
                    }
                }
            }
            _ => {}
        }
    }

    /// Receives committed text from the platform input handler.
    fn insert_typed(&mut self, text: &str) {
        if text.is_empty() || text.chars().any(char::is_control) {
            return;
        }
        if let Some(field) = self.field.as_mut() {
            if field.fresh {
                field.buffer.clear();
                field.fresh = false;
            }
            field.buffer.push_str(text);
        } else if self.text_edit.is_some() {
            self.edit_text(|t| t.push_str(text));
        }
    }

    pub fn commit_field(&mut self) {
        let Some(field) = self.field.take() else { return };
        let text = field.buffer.trim();
        let number = text.parse::<f32>().ok().filter(|v| v.is_finite());
        let color = Color::parse_hex(text);
        let cleared = text.is_empty();
        match field.id {
            FieldId::X | FieldId::Y => {
                let (Some(v), Some(bounds)) = (number, self.ed.selection_bounds()) else { return };
                let (dx, dy) = if field.id == FieldId::X { (v - bounds.x, 0.0) } else { (0.0, v - bounds.y) };
                self.ed.for_selected(|o| o.translate(dx, dy));
                self.ed.commit("Move");
            }
            FieldId::W | FieldId::H => {
                let Some(v) = number.filter(|v| *v > 0.0) else { return };
                if self.ed.selection.len() != 1 {
                    self.status = "Width and height apply to a single object".into();
                    return;
                }
                let is_width = field.id == FieldId::W;
                self.ed.for_selected(|o| {
                    let mut b = o.bounds;
                    if is_width {
                        b.w = v;
                    } else {
                        b.h = v;
                    }
                    o.set_bounds(b);
                });
                self.ed.commit("Resize");
            }
            FieldId::Radius => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                self.ed.for_selected(|o| {
                    if let Shape::Rect { radius } = &mut o.shape {
                        *radius = v;
                    }
                });
                self.ed.commit("Roundness");
            }
            FieldId::TextSize => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.ed.for_selected(|o| {
                    if let Shape::Text { content, size } = &mut o.shape {
                        *size = v;
                        let (w, h) = measure_text(content, v);
                        o.bounds.w = w;
                        o.bounds.h = h;
                    }
                });
                self.ed.commit("Text Size");
            }
            FieldId::StrokeWidth => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                if self.first_vector().is_none() {
                    self.stroke_width = v;
                } else {
                    self.ed.for_selected(|o| {
                        if let Some(stroke) = &mut o.stroke {
                            stroke.width = v;
                        }
                    });
                    self.ed.commit("Stroke Width");
                }
            }
            FieldId::BrushSize => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.brush_size = v.min(MAX_BRUSH_SIZE);
            }
            FieldId::FreeformSize => {
                let Some(v) = number.filter(|v| *v >= 2.0) else { return };
                self.freeform_size = v.min(MAX_BRUSH_SIZE);
            }
            FieldId::Url => {
                let link = text.to_string();
                self.ed.for_selected(|o| {
                    if let Shape::Web { url, .. } = &mut o.shape {
                        url.clone_from(&link);
                    }
                });
                self.ed.commit("Hotspot Link");
            }
            FieldId::Opacity => {
                let Some(v) = number else { return };
                let v = v.clamp(0.0, 100.0) / 100.0;
                self.ed.for_selected(|o| o.opacity = v);
                self.ed.commit("Opacity");
            }
            FieldId::FillHex if color.is_some() || cleared => self.apply_color(ColorTarget::Fill, color),
            FieldId::StrokeHex if color.is_some() || cleared => self.apply_color(ColorTarget::Stroke, color),
            FieldId::CanvasHex if color.is_some() || cleared => self.apply_color(ColorTarget::Canvas, color),
            FieldId::RenamePage(i) if !cleared => {
                if let Some(page) = self.ed.doc.pages.get_mut(i) {
                    page.name = text.to_string();
                }
                self.ed.commit("Rename Page");
            }
            FieldId::RenameState(i) if !cleared => {
                if let Some(state) = self.ed.doc.page_mut().states.get_mut(i) {
                    state.name = text.to_string();
                }
                self.ed.commit("Rename State");
            }
            FieldId::RenameLayer(i) if !cleared => {
                if let Some(layer) = self.ed.doc.page_mut().layers.get_mut(i) {
                    layer.name = text.to_string();
                }
                self.ed.commit("Rename Layer");
            }
            FieldId::RenameObject(id) if !cleared => {
                if let Some(object) = self.ed.object_mut(id) {
                    object.name = text.to_string();
                }
                self.ed.commit("Rename Object");
            }
            FieldId::RenameSymbol(id) if !cleared => {
                if let Some(symbol) = self.ed.doc.symbols.iter_mut().find(|s| s.id == id) {
                    symbol.name = text.to_string();
                }
                self.ed.commit("Rename Symbol");
            }
            FieldId::StateDelay(i) => {
                let Some(v) = number else { return };
                if let Some(state) = self.ed.doc.page_mut().states.get_mut(i) {
                    state.delay_ms = (v.round() as u32).clamp(MIN_STATE_DELAY, MAX_STATE_DELAY);
                }
                self.ed.commit("State Delay");
            }
            FieldId::AutoParam(slot) => {
                let Some(v) = number else { return };
                self.ed.for_selected(|o| {
                    let Shape::Auto { kind, a, b, count } = &mut o.shape else { return };
                    let Some(info) = kind.params()[slot.min(2)] else { return };
                    let value = (if info.max <= 1.0 { v / 100.0 } else { v }).clamp(info.min, info.max);
                    match slot {
                        0 => *a = value,
                        1 => *b = value,
                        _ => *count = value.round() as u32,
                    }
                });
                self.ed.commit("Auto Shape");
            }
            FieldId::PathAmount => {
                let Some(v) = number.filter(|v| *v > 0.0) else { return };
                self.path_amount = v;
            }
            FieldId::OptimizeQuality => {
                let Some(v) = number else { return };
                self.set_export(|s| s.quality = v.clamp(1.0, 100.0) as u8);
            }
            FieldId::OptimizeColors => {
                let Some(v) = number else { return };
                self.set_export(|s| s.colors = (v as u16).clamp(MIN_COLORS, MAX_COLORS));
            }
            FieldId::Mixer(channel) => {
                let Some(v) = number else { return };
                self.set_mixer_channel(channel, v.clamp(0.0, 255.0) as u8);
            }
            FieldId::DialogW | FieldId::DialogH => {
                let page = self.ed.doc.page();
                let (page_w, page_h) = (page.width.max(1) as f32, page.height.max(1) as f32);
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                let v = (v.round() as u32).min(MAX_CANVAS_SIZE);
                let Some(dialog) = self.size_dialog.as_mut() else { return };
                let scaled = |value: f32| (value.round() as u32).clamp(1, MAX_CANVAS_SIZE);
                if field.id == FieldId::DialogW {
                    dialog.width = v;
                    if dialog.constrain {
                        dialog.height = scaled(v as f32 * page_h / page_w);
                    }
                } else {
                    dialog.height = v;
                    if dialog.constrain {
                        dialog.width = scaled(v as f32 * page_w / page_h);
                    }
                }
            }
            FieldId::FindText => self.find_text = text.to_string(),
            FieldId::ReplaceText => self.replace_text = text.to_string(),
            FieldId::NewUrl if !cleared => {
                if !self.ed.doc.urls.iter().any(|u| u == text) {
                    self.ed.doc.urls.push(text.to_string());
                    self.ed.commit("Add URL");
                }
            }
            _ => {}
        }
    }

    pub fn current_color(&self, target: ColorTarget) -> Option<Color> {
        let first = self.first_vector();
        match target {
            ColorTarget::Fill => first.map_or(self.fill, |o| o.fill),
            ColorTarget::Stroke => first.map_or(self.stroke, |o| o.stroke.map(|s| s.color)),
            ColorTarget::Canvas => self.ed.doc.page().canvas,
            ColorTarget::Matte => self.ed.doc.page().export.matte,
        }
    }

    /// Sets a color on the selection (or canvas) and as the default for new objects.
    pub fn apply_color(&mut self, target: ColorTarget, color: Option<Color>) {
        match target {
            ColorTarget::Fill => {
                self.fill = color;
                self.ed.for_selected(|o| {
                    if o.takes_color() {
                        o.fill = color;
                    }
                });
                self.ed.commit("Fill");
            }
            ColorTarget::Stroke => {
                self.stroke = color;
                let width = self.stroke_width;
                self.ed.for_selected(|o| {
                    if o.takes_color() {
                        o.stroke = color.map(|color| Stroke { color, width: o.stroke.map_or(width, |s| s.width) });
                    }
                });
                self.ed.commit("Stroke");
            }
            ColorTarget::Canvas => {
                self.ed.doc.page_mut().canvas = color;
                self.ed.commit("Canvas Color");
            }
            ColorTarget::Matte => self.set_export(|s| s.matte = color),
        }
    }

    pub fn swap_colors(&mut self) {
        std::mem::swap(&mut self.stroke, &mut self.fill);
    }

    pub fn default_colors(&mut self) {
        self.stroke = Some(Color::BLACK);
        self.fill = Some(Color::WHITE);
    }
}

/// Computes new bounds for dragging one of the 8 handles, clockwise from top-left.
pub fn resized(origin: Rect, handle: usize, p: Vec2) -> Rect {
    let (mut x0, mut y0, mut x1, mut y1) = (origin.x, origin.y, origin.right(), origin.bottom());
    if matches!(handle, 0 | 6 | 7) {
        x0 = p.x;
    }
    if matches!(handle, 2..=4) {
        x1 = p.x;
    }
    if matches!(handle, 0..=2) {
        y0 = p.y;
    }
    if matches!(handle, 4..=6) {
        y1 = p.y;
    }
    let mut rect = Rect::from_corners(v2(x0, y0), v2(x1, y1));
    rect.w = rect.w.max(1.0);
    rect.h = rect.h.max(1.0);
    rect
}

/// All typed characters arrive here, on every keyboard layout and through IMEs.
/// Composition text is not shown; the committed text is inserted when it finishes.
impl EntityInputHandler for Workspace {
    fn text_for_range(
        &mut self,
        _: Range<usize>,
        _: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        Some(String::new())
    }

    fn selected_text_range(&mut self, _: bool, _: &mut Window, _: &mut Context<Self>) -> Option<UTF16Selection> {
        (self.field.is_some() || self.text_edit.is_some()).then_some(UTF16Selection { range: 0..0, reversed: false })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {}

    fn replace_text_in_range(&mut self, _: Option<Range<usize>>, text: &str, _: &mut Window, cx: &mut Context<Self>) {
        self.insert_typed(text);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _: Option<Range<usize>>,
        _: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
    }

    fn bounds_for_range(
        &mut self,
        _: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(element_bounds)
    }

    fn character_index_for_point(&mut self, _: Point<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        None
    }
}

pub fn app_quit_on_close(cx: &mut App) {
    cx.on_window_closed(|cx| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    fn map() -> ViewMap {
        let bounds = Bounds { origin: point(px(10.), px(20.)), size: size(px(800.), px(600.)) };
        ViewMap::new(bounds, 400.0, 300.0, 2.0, Vec2::default())
    }

    #[test]
    fn view_map_centers_page_and_round_trips() {
        let m = map();
        assert_eq!(m.to_screen(v2(0.0, 0.0)), point(px(10.), px(20.)));
        let back = m.to_doc(m.to_screen(v2(123.0, 45.0)));
        assert!((back.x - 123.0).abs() < 0.01 && (back.y - 45.0).abs() < 0.01);
    }

    #[test]
    fn corner_handle_moves_two_edges_only() {
        let origin = Rect { x: 10.0, y: 10.0, w: 100.0, h: 50.0 };
        assert_eq!(resized(origin, 4, v2(210.0, 110.0)), Rect { x: 10.0, y: 10.0, w: 200.0, h: 100.0 });
        assert_eq!(resized(origin, 0, v2(0.0, 0.0)), Rect { x: 0.0, y: 0.0, w: 110.0, h: 60.0 });
    }

    #[test]
    fn side_handle_moves_one_edge_only() {
        let origin = Rect { x: 10.0, y: 10.0, w: 100.0, h: 50.0 };
        assert_eq!(resized(origin, 3, v2(60.0, 999.0)), Rect { x: 10.0, y: 10.0, w: 50.0, h: 50.0 });
        assert_eq!(resized(origin, 1, v2(999.0, 0.0)), Rect { x: 10.0, y: 0.0, w: 100.0, h: 60.0 });
    }

    #[test]
    fn dragging_past_opposite_edge_flips_without_collapsing() {
        let origin = Rect { x: 10.0, y: 10.0, w: 100.0, h: 50.0 };
        let r = resized(origin, 3, v2(0.0, 0.0));
        assert_eq!((r.x, r.w), (0.0, 10.0));
        assert_eq!(resized(origin, 3, v2(10.0, 0.0)).w, 1.0);
    }

    #[test]
    fn file_type_is_detected_from_content() {
        let bitmap = Bitmap::from_rgba(2, 2, vec![200; 16]).unwrap();
        let png = bitmap.encode_png().unwrap();
        assert!(matches!(parse_file(&png), Ok(Opened::Image(b)) if b == bitmap));

        let json = document_to_json(&Document::new(10, 10)).unwrap();
        assert!(matches!(parse_file(json.as_bytes()), Ok(Opened::Document(_))));

        assert!(parse_file(b"plain text").is_err());
        assert!(parse_file(b"{ not json").is_err());
    }

    #[test]
    fn corner_scale_keeps_proportions_about_the_opposite_corner() {
        let origin = Rect { x: 10.0, y: 10.0, w: 100.0, h: 50.0 };
        assert_eq!(scaled(origin, 4, v2(210.0, 20.0), true), Rect { x: 10.0, y: 10.0, w: 200.0, h: 100.0 });
        assert_eq!(scaled(origin, 0, v2(-90.0, 50.0), true), Rect { x: -90.0, y: -40.0, w: 200.0, h: 100.0 });
        assert_eq!(scaled(origin, 4, v2(210.0, 20.0), false).h, 10.0);
    }

    #[test]
    fn side_scale_stretches_one_axis() {
        let origin = Rect { x: 10.0, y: 10.0, w: 100.0, h: 50.0 };
        assert_eq!(scaled(origin, 3, v2(60.0, 999.0), true), Rect { x: 10.0, y: 10.0, w: 50.0, h: 50.0 });
    }

    #[test]
    fn scaling_moves_each_object_within_the_group() {
        let origin = Rect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 };
        let target = Rect { x: 0.0, y: 0.0, w: 200.0, h: 200.0 };
        let object = Object::new_web(1, WebKind::Slice, Rect { x: 50.0, y: 50.0, w: 10.0, h: 10.0 });
        assert_eq!(scale_object(&object, origin, target).bounds, Rect { x: 100.0, y: 100.0, w: 20.0, h: 20.0 });
    }

    #[test]
    fn modifiers_choose_how_selections_combine() {
        let mut m = gpui::Modifiers::default();
        assert_eq!(selection_mode(m), MaskMode::Replace);
        m.shift = true;
        assert_eq!(selection_mode(m), MaskMode::Add);
        m.alt = true;
        assert_eq!(selection_mode(m), MaskMode::Subtract);
    }

    #[test]
    fn square_and_angle_constraints() {
        assert_eq!(constrain_square(v2(0.0, 0.0), v2(10.0, -4.0)), v2(10.0, -10.0));
        let p = constrain_angle(v2(0.0, 0.0), v2(10.0, 1.0));
        assert!((p.y).abs() < 0.001);
    }
}
