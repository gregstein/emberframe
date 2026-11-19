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

use crate::blend::BlendMode;
use crate::effects::{FilterKind, LiveFilter};
use crate::i18n::{t, tc, tf};
use crate::settings::Settings;
use crate::bitmap::{Bitmap, Mask, MaskMode, FILL_TOLERANCE};
use crate::canvas_view::{encoded_texture, handle_rects, make_texture, texture_key, Textures};
use crate::autoshape::AutoKind;
use crate::export::{export_web, measure_text};
use crate::library::common_items;
use crate::menus::{panel_shortcut, Cmd, GuideCmd, Panel, ScreenMode, Tool, DOCK_GROUPS, ZOOM_LEVELS};
use crate::model::{snap_offset, snap_places, FillKind, FillStyle};
use crate::theme::RULER_SIZE;
use crate::optimize::{comparison_settings, encode, encoded_palette, export_page, format_size, settings_label, ExportSettings, MAX_COLORS, MIN_COLORS};
use crate::paths::PathOp;
use crate::model::SymbolKind;
use crate::model::{
    document_from_json, document_to_json, flatten_curve, pull_points, subdivide, v2, Color, Document, Editor, Handles,
    Object, PixelStroke, Rect, Shape, Stroke, Vec2, Warp, WebKind,
};
use crate::theme::{app_name, document_extension, is_sahne, FILE_EXTENSION, LEGACY_EXTENSION, SAHNE_EXTENSION};

/// Screen pixels the pointer must travel before a press counts as pulling a curve handle.
const HANDLE_DRAG_START: f32 = 3.0;
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
    /// The tint that shows the JPEG mask.
    JpegMask,
    /// The color of the live filter being edited.
    Filter,
    /// The color of one of the effects in the Photoshop Live Effects being edited.
    PsEffect(usize),
    /// The color of the gradient stop picked in Photoshop Live Effects.
    GradientStop,
    /// The colors of the Replace Color tool: the one to change, and what it becomes.
    ReplaceFrom,
    ReplaceTo,
    /// The color guides are drawn in.
    Guides,
    Grid,
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
    /// The list of live filters to add.
    AddFilter,
    /// A setting of the filter being edited that is a choice from a list, by its place among the settings.
    FilterChoice(usize),
    /// The fonts installed on this system, for text.
    TextFont,
    Paper,
    Orientation,
    /// Which look of the button being edited is on show.
    ButtonLook,
    BatchScale,
    TextOrientation,
    GifLoop,
    GifWizardLoop,
    /// Rules and Numbers: the kinds of event, to add a rule; the kind of the rule on show; its first or second name.
    RuleNew,
    RuleEvent,
    RuleEventPick(bool),
    /// The kinds of step, to add one; the kind of a step of the rule on show; a step's name.
    RuleStepAdd,
    RuleStep(usize),
    RuleStepPick(usize, bool),
    /// "Only if": the number it is about, or none; and which way the number is held against the value.
    RuleIf,
    RuleIfHow,
    /// Rules as Text: the sentences an event can be, and those a step can be, to put one in.
    ScriptEvent,
    ScriptStep,
    /// The action of the frame on show, or, with `true`, of the selected button.
    Action(bool),
    FrameSound,
    ArrowStyle,
    AutoMask,
    Printer,
    PrintRange,
    /// What an object is filled with: a color, a gradient or a pattern.
    FillKind,
    FillPattern,
    ToneRange,
    PolygonShape,
    FillEdge,
    FillTexture,
    PathOrientation,
    /// The gradients to start from, in Photoshop Live Effects.
    GradientPreset,
    /// How the mask of the selection works.
    MaskKind,
    /// The blend mode of the selection.
    Blend,
    /// Whether a symbol, given by its id, is a graphic or a button.
    SymbolKind(u64),
    /// What the Numeric Transform dialog changes.
    NumericMode,
    /// The tools behind the Scale button.
    /// The tools behind a shared button, by the number of their group.
    ToolGroup(usize),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NumericMode {
    /// Width and height as percentages of what they are.
    Scale,
    /// Width and height in pixels.
    Resize,
    /// An angle in degrees, clockwise.
    Rotate,
}

impl NumericMode {
    pub const ALL: [NumericMode; 3] = [NumericMode::Scale, NumericMode::Resize, NumericMode::Rotate];

    pub fn label(self) -> &'static str {
        match self {
            NumericMode::Scale => t("Scale"),
            NumericMode::Resize => t("Resize"),
            NumericMode::Rotate => t("Rotate"),
        }
    }
}

/// The Numeric Transform dialog while it is open.
/// Text > Check Spelling while it runs: the misspelled words with what the checker suggests for
/// each, what has been set aside, and what the word on show is to be changed to.
pub struct SpellSession {
    pub wrong: std::collections::HashMap<String, Vec<String>>,
    /// Words to pass over wherever they are, and single places to pass over: a text and where in it the word starts.
    pub ignored: std::collections::HashSet<String>,
    pub skipped: std::collections::HashSet<(u64, usize)>,
    pub replacement: String,
}

/// File > Image Preview: the export settings beside the image they give, before it is saved.
pub struct ImagePreview {
    /// The part of the page to export; all of it when there is none.
    pub area: Option<Rect>,
    /// The settings to go back to on Cancel.
    pub before: ExportSettings,
    /// The image as it will be exported, with its format and size, and the revision of the document it was made from.
    pub pane: Option<PreviewPane>,
    pub made_from: Option<u64>,
    /// The size of the exported image in pixels.
    pub size: (u32, u32),
}

/// What Properties offers for the tool in hand, beyond the size of its brush.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolOptions {
    /// Dodge and Burn: how far a pixel goes toward white or black, in percent, and the tones worked on most.
    pub exposure: f32,
    pub range: crate::bitmap::ToneRange,
    /// Smudge: how strongly it rubs in, in percent, and whether the brush starts loaded with the stroke color.
    pub pressure: f32,
    pub smudge_color: bool,
    /// Replace Color: the color to change, or none to take it from where the stroke starts; what it becomes; and how near a color must be.
    pub replace_from: Option<Color>,
    pub replace_to: Color,
    pub replace_tolerance: f32,
    /// Red Eye Removal: how readily a pixel counts as red, and how much red is taken away, in percent.
    pub red_tolerance: f32,
    pub red_strength: f32,
    /// Polygon: a star instead, and the number of sides or points.
    pub star: bool,
    pub sides: f32,
    /// Vector Path and Redraw Path: how closely the path follows the hand, from 1 to 10.
    pub precision: f32,
    /// The Path Scrubbers: how fast they change the stroke, in percent.
    pub rate: f32,
    /// How unlike the pixel clicked a pixel may be and still be taken by the Magic Wand, or filled by the Paint Bucket.
    pub wand_tolerance: f32,
    pub bucket_tolerance: f32,
}

impl Default for ToolOptions {
    fn default() -> Self {
        Self {
            exposure: 50.0,
            range: Default::default(),
            pressure: 50.0,
            smudge_color: false,
            replace_from: None,
            replace_to: Color::rgb(255, 0, 0),
            replace_tolerance: 32.0,
            red_tolerance: 50.0,
            red_strength: 75.0,
            star: false,
            sides: 5.0,
            precision: 5.0,
            rate: 20.0,
            wand_tolerance: FILL_TOLERANCE as f32,
            bucket_tolerance: FILL_TOLERANCE as f32,
        }
    }
}

impl ToolOptions {
    /// How far a fitted curve may stray from the points drawn, for the precision set.
    pub fn fit_tolerance(&self) -> f32 {
        (11.0 - self.precision.clamp(1.0, 10.0)) * 0.3
    }
}

/// File > Batch Process: the files, and what is done to each.
#[derive(Clone, Debug, Default)]
pub struct BatchDialog {
    pub files: Vec<PathBuf>,
    /// 0 no scaling, 1 to a size, 2 to fit an area, 3 to a percentage.
    pub scale: usize,
    pub width: u32,
    pub height: u32,
    pub percent: f32,
    /// Export every file with the Optimize settings of the document on show, not with its own.
    pub use_current: bool,
    pub prefix: String,
    pub suffix: String,
    /// Where the results go; beside each original if not set.
    pub output: Option<PathBuf>,
}

pub const BATCH_SCALES: [&str; 4] = ["No Scaling", "Scale to Size", "Scale to Fit Area", "Scale to Percentage"];

/// Make a GIF: the picture files, how long each stays on show, and how many times the animation plays.
#[derive(Clone, Debug)]
pub struct GifDialog {
    pub files: Vec<PathBuf>,
    pub delay_ms: u32,
    pub loops: u16,
}

/// The Print dialog: where the page is to go, and how many times.
#[derive(Clone, Debug)]
pub struct PrintDialog {
    pub printers: Vec<String>,
    /// The printer picked; one past the last of them is a PDF file.
    pub chosen: usize,
    pub copies: u32,
    /// Which pages of the document to print, and for a range the first and last, counted from 1.
    pub range: crate::print::PageRange,
    pub from: u32,
    pub to: u32,
    /// Printer options: on both sides of the paper, and in shades of grey.
    pub two_sided: bool,
    pub grayscale: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct NumericDialog {
    pub mode: NumericMode,
    /// The selection box the numbers apply to.
    pub bounds: Rect,
    /// The point that stays where it is: the center point of the transform box if
    /// it was dragged somewhere, otherwise the middle of the selection.
    pub center: Vec2,
    /// Percentages for Scale.
    pub percent: (f32, f32),
    /// Pixels for Resize.
    pub size: (f32, f32),
    /// Degrees for Rotate.
    pub angle: f32,
    pub constrain: bool,
    /// Whether stroke widths grow and shrink with the artwork.
    pub scale_attributes: bool,
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
    OptimizeSelective,
    /// A number of the live filter being edited, by its place among the settings.
    FilterValue(usize),
    /// The number of pixels in the dialog of a marquee command.
    MarqueeAmount,
    /// A margin in Page Setup: left, right, top, bottom.
    PrintMargin(usize),
    PrintCopies,
    PrintFrom,
    PrintTo,
    /// How large a pattern fill is drawn, in percent.
    FillScale,
    /// The numbers among the options of the tool in hand.
    ToolExposure,
    ToolPressure,
    ReplaceTolerance,
    RedTolerance,
    RedStrength,
    PolygonSides,
    PathPrecision,
    ScrubRate,
    WandTolerance,
    BucketTolerance,
    FillFeather,
    TextureAmount,
    /// How far along its path text on a path starts.
    PathOffset,
    /// What Check Spelling changes the word on show to.
    SpellChange,
    NewSymbolName,
    TemplateName,
    CommandName,
    ArrowSize,
    FrameSize,
    TweenSteps,
    GifDelay,
    FrameRate,
    /// The frame on show, counted from 1: typing a number goes there.
    CurrentFrame,
    /// The time of the frame on show, in seconds: typing a time goes there.
    CurrentSecond,
    /// The number or address an action takes: of the frame's action, or of the selected button's.
    ActionParam(bool),
    TweenEase,
    TweenTurns,
    /// A number of Twist and Fade: steps, spacing, direction, rotation, scale, the opacity faded to.
    Twist(usize),
    /// Rules and Numbers: the number of the event on show; the first or the second number of one of its steps.
    RuleNumber,
    /// The value "only if" holds its number against.
    RuleIfValue,
    RuleStep(usize, bool),
    /// The name of a number the rules keep, or what it starts as.
    VariableName(usize),
    VariableStart(usize),
    /// A number of a behavior in Behavior Settings: which behavior of the list, and whether its second number.
    Behavior(usize, bool),
    /// Reverse State Range: the first state, or the last.
    ReverseState(bool),
    /// A text of Batch Process: 0 width, 1 height, 2 percentage, 3 prefix, 4 suffix.
    Batch(usize),
    /// A number of the Animate dialog: states, move, direction, scaling, opacity from and to, rotation.
    Animate(usize),
    PrefUndo,
    PrefPick,
    PrefSnap,
    PrefGridX,
    PrefGridY,
    GuidePosition,
    /// Where the picked gradient stop is, in percent, and how opaque it is.
    GradientPlace,
    GradientOpacity,
    OptimizeSmoothing,
    OptimizeColors,
    Mixer(usize),
    FindText,
    ReplaceText,
    NewUrl,
    DialogW,
    DialogH,
    /// The first and second number of the Numeric Transform dialog.
    NumericA,
    NumericB,
}

pub struct FieldEdit {
    pub id: FieldId,
    pub buffer: String,
    /// The next typed character replaces the whole value.
    pub fresh: bool,
}

/// Something being dragged out of a panel to be dropped on the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelItem {
    AutoShape(AutoKind),
    /// An entry of the Common Library, by its place in the list.
    Common(usize),
    /// A symbol of the Document Library.
    Symbol(u64),
}

/// The pink Fireworks lays over a JPEG mask unless another color is chosen.
pub const JPEG_MASK_COLOR: Color = Color::rgb(255, 51, 153);

/// Screen pixels the pointer may wander during a press that still counts as a click.
const CLICK_SLOP: f32 = 4.0;

/// What a drag does with the Scale, Skew and Distort tools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformKind {
    /// Scale tool, any handle.
    Scale,
    /// Any of the three tools, dragging outside the box: turn around the center point.
    Rotate,
    /// Any of the three tools, dragging inside the box.
    Move,
    /// Skew tool, a side handle: that side slides along itself.
    Skew,
    /// Skew tool, a corner handle: the corner and its neighbour move apart or together.
    Perspective,
    /// Distort tool, a corner handle: that corner moves alone.
    Distort,
    /// Distort tool, a side handle: both corners of that side move.
    DistortSide,
    /// Dragging the center point that rotation turns around.
    Pivot,
}

/// A transform being tried out with the Scale, Skew or Distort tool. As in Fireworks,
/// the artwork only previews it until Enter or a double-click applies it, and Esc
/// puts everything back.
pub struct TransformSession {
    /// The selection box when the session began.
    pub origin: Rect,
    /// Everything done so far, as one map from that box.
    pub warp: Warp,
    /// The selected objects as they were.
    originals: Vec<Object>,
    /// The point rotation turns around, once it has been dragged off the middle.
    pub pivot: Option<Vec2>,
}

impl TransformSession {
    /// The corners of the box as they are now, clockwise from the top-left.
    pub fn quad(&self) -> [Vec2; 4] {
        rect_corners(self.origin).map(|corner| self.warp.apply(corner))
    }

    pub fn pivot(&self) -> Vec2 {
        self.pivot.unwrap_or_else(|| quad_center(self.quad()))
    }
}

pub fn rect_corners(r: Rect) -> [Vec2; 4] {
    [v2(r.x, r.y), v2(r.right(), r.y), v2(r.right(), r.bottom()), v2(r.x, r.bottom())]
}

pub fn quad_center(quad: [Vec2; 4]) -> Vec2 {
    v2(quad.iter().map(|p| p.x).sum::<f32>() / 4.0, quad.iter().map(|p| p.y).sum::<f32>() / 4.0)
}

/// The eight handles of a transform box: its corners and the middles of its sides,
/// clockwise from the top-left like [`handle_rects`].
pub fn quad_handles(quad: [Vec2; 4]) -> [Vec2; 8] {
    let mid = |a: Vec2, b: Vec2| v2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    [quad[0], mid(quad[0], quad[1]), quad[1], mid(quad[1], quad[2]), quad[2], mid(quad[2], quad[3]), quad[3], mid(quad[3], quad[0])]
}

/// The upright rectangle a box is, if it has only been moved and scaled.
fn upright(quad: [Vec2; 4]) -> Option<Rect> {
    let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
    let [a, b, c, d] = quad;
    (close(a.y, b.y) && close(b.x, c.x) && close(c.y, d.y) && close(d.x, a.x) && b.x > a.x && d.y > a.y)
        .then(|| Rect { x: a.x, y: a.y, w: b.x - a.x, h: d.y - a.y })
}

/// `bounds` scaled by `sx` and `sy` so that `center` stays where it is.
fn scaled_about(bounds: Rect, center: Vec2, sx: f32, sy: f32) -> Rect {
    Rect { x: center.x - (center.x - bounds.x) * sx, y: center.y - (center.y - bounds.y) * sy, w: bounds.w * sx, h: bounds.h * sy }
}

fn is_transform_tool(tool: Tool) -> bool {
    matches!(tool, Tool::Scale | Tool::Skew | Tool::Distort)
}

/// Rotation with Shift held snaps to steps of this many degrees.
const ROTATE_STEP_DEGREES: f32 = 15.0;
/// How close, in screen pixels, the pointer must be to a transform handle to take it.
const TRANSFORM_REACH: f32 = 5.0;
/// How long the shared tool button is held before its tools are listed.
const TOOL_HOLD_MS: u64 = 350;
const TOOL_FLYOUT_OFFSET: f32 = 24.0;

pub enum Drag {
    /// Moving the selection: where the mouse took hold, the box of the selection
    /// then, and how far it has been moved since, which snapping can make differ from the mouse.
    Move { start: Vec2, origin: Option<Rect>, moved: Vec2 },
    /// A Path Scrubber at work on these paths: making their strokes heavier, or lighter.
    Scrub { ids: Vec<u64>, add: bool },
    /// Reshape Area: pushing the points of these paths that lie within a radius of the mouse.
    Reshape { ids: Vec<u64>, last: Vec2, radius: f32 },
    /// 9-slice scaling by a handle of the selection box, from the box and the objects as they were.
    NineSlice { handle: usize, origin: Rect, originals: Vec<Object> },
    /// Dragging one of the four cuts of 9-slice scaling: the two upright ones, then the two level ones.
    NineGuide(usize),
    /// Dragging a handle of a gradient fill: its end, or else its start.
    FillHandle { end: bool, width: bool },
    /// The end of an animation's path is being dragged.
    MotionEnd,
    /// A shape hint is being dragged: its number, and whether it is the end of the pair.
    Hint { number: usize, end: bool },
    /// The start of an animation's path is being dragged: where its end is, and where the start was last.
    MotionStart { end: Vec2, last: Vec2 },
    /// Dragging a guide: one from the page, or with no index a new one out of a ruler.
    Guide { horizontal: bool, index: Option<usize>, at: f32 },
    Resize { handle: usize, origin: Rect },
    Marquee { start: Vec2, current: Vec2 },
    Create { start: Vec2, current: Vec2 },
    /// `target` is the bitmap being painted; `None` draws a vector path.
    Freehand { points: Vec<Vec2>, target: Option<u64> },
    Crop { start: Vec2, current: Vec2 },
    /// A Marquee or Lasso drag. Marquee keeps just the two corners.
    PixelArea { target: u64, points: Vec<Vec2>, mode: MaskMode },
    /// One drag of a transform session. `from` is the session's map when the drag began.
    Transform { kind: TransformKind, handle: usize, start: Vec2, from: Warp, pivot: Vec2 },
    Freeform { id: u64, start: Vec2, base: Vec<Vec2>, radius: f32 },
    Vertex { id: u64, index: usize },
    /// Pulling the curve handles out of the vertex the Pen just placed.
    PenHandle,
    /// Dragging one curve handle of a vertex with Subselection. `fresh` pulls a new
    /// pair out of a corner, and leaves a corner behind if the pointer never moves.
    /// `pen` marks a press of the Pen on a vertex, which acts as a click if nothing is pulled.
    Handle { id: u64, index: usize, after: bool, fresh: bool, moved: bool, pen: bool },
    Pan { last: Point<Pixels> },
    /// Dragging a pixel selection. With `pixels` the selected pixels travel with the
    /// marquee, leaving a hole unless `copy` is set; without it only the marquee moves.
    /// `before` is the bitmap as the drag found it, `piece` the selected pixels and
    /// `at` where they sat.
    Pixels { target: u64, before: Arc<Bitmap>, piece: Bitmap, at: (i64, i64), mask: Arc<Mask>, start: Vec2, pixels: bool, copy: bool },
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
    /// The options of the retouching tools, as Properties shows them while the tool is in hand.
    pub tool_options: ToolOptions,
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
    /// Curve handles of the vertices in `pen`, one each.
    pub pen_handles: Vec<Handles>,
    /// The vertices picked with Subselection, each as its path and its index.
    pub point_selection: Vec<(u64, usize)>,
    /// The open path the Pen is carrying on, which `pen` then holds a copy of.
    pen_resume: Option<u64>,
    /// A panel item picked up with the mouse, and where it was pressed.
    pub panel_drag: Option<(PanelItem, Point<Pixels>)>,
    /// A question is on screen. Its buttons sit over the canvas, and a click on one
    /// must not also act on the artwork behind it.
    awaiting_answer: bool,
    pub transform: Option<TransformSession>,
    /// Which of Scale, Skew and Distort the shared tool button shows.
    pub transform_tool: Tool,
    /// The tool each shared button shows: the one of its group used last.
    pub group_tools: Vec<Tool>,
    /// The Polygon Lasso under way: the bitmap it selects on, its corners so far, and how it combines with the selection.
    pub polygon_lasso: Option<(u64, Vec<Vec2>, MaskMode)>,
    /// The corners clicked so far of a polygon hotspot or slice.
    pub polygon_web: Option<Vec<Vec2>>,
    /// The cuts of 9-slice scaling, as parts of the selection box: two upright, then two level.
    pub nine_guides: [f32; 4],
    /// Counts presses of that button, so a hold that ended is not mistaken for one still held.
    tool_hold: u64,
    pub cursor_doc: Vec2,
    /// Where the pointer is in the window.
    pub cursor_screen: Point<Pixels>,
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
    previews_key: Option<(u64, ViewMode, Option<(u64, usize)>)>,
    stale_previews: Vec<Arc<RenderImage>>,
    /// The rendered page shown on the canvas while it has blend modes, with the revision it was rendered at.
    pub page_raster: Option<(u64, Arc<RenderImage>)>,
    pub size_dialog: Option<SizeDialog>,
    pub numeric_dialog: Option<NumericDialog>,
    /// The Selective JPEG Settings dialog, holding the settings to go back to on Cancel.
    pub jpeg_dialog: Option<(ExportSettings, Option<Color>)>,
    /// The live filter whose settings are open: its place in the list, and the list to go back to on Cancel.
    pub filter_dialog: Option<(usize, Vec<LiveFilter>)>,
    /// A live filter being dragged in the list: where it is now, where the mouse took hold, and whether it has moved.
    pub filter_drag: Option<(usize, Point<Pixels>, bool)>,
    /// How many pixels of the object have each level of red, green and blue, as the
    /// filter being edited receives them. Levels draws it and Auto reads it.
    pub filter_histogram: Option<Rc<[[u32; 256]; 3]>>,
    /// The point of the curve being dragged in the Curves dialog.
    pub curve_drag: Option<usize>,
    /// The point of the curve last touched, whose levels the dialog shows.
    pub curve_point: Option<usize>,
    pub curve_bounds: Rc<Cell<Bounds<Pixels>>>,
    /// The effect whose settings show in the Photoshop Live Effects dialog.
    pub ps_effect: usize,
    /// What File > Page Setup is set to.
    pub print_setup: crate::print::PrintSetup,
    /// Page Setup while it is open, holding what to go back to on Cancel.
    pub page_setup_dialog: Option<crate::print::PrintSetup>,
    /// New Symbol: the name and the kind asked for.
    pub symbol_dialog: Option<(String, SymbolKind)>,
    /// New from Template: the user's own templates, and which entry of the list is picked.
    pub template_dialog: Option<(Vec<(String, PathBuf)>, usize)>,
    /// Save as Template: the name asked for.
    pub template_name: Option<String>,
    pub batch_dialog: Option<BatchDialog>,
    /// The saved commands, as the Commands menu lists them.
    pub saved_commands: Vec<(String, PathBuf)>,
    /// The menu commands that changed the document, each with the place and the name its step has in the history.
    pub recorded: Vec<(usize, String, String)>,
    /// Manage Saved Commands: the command picked in the list.
    pub commands_dialog: Option<Option<usize>>,
    /// The name asked for: for the steps being saved as a command, or, with a place in the list, for a saved command being renamed.
    pub command_name: Option<(String, Option<usize>)>,
    /// Reverse State Range: the first and the last state, counted from 1.
    pub reverse_dialog: Option<(u32, u32)>,
    /// Add Arrowheads: whether the start and the end get one, and how long the heads are.
    pub arrow_dialog: Option<(bool, bool, f32, usize)>,
    /// Add Picture Frame: how wide the frame is.
    pub frame_dialog: Option<f32>,
    /// Auto Vector Mask: which of the masks is picked.
    pub mask_dialog: Option<usize>,
    pub twist_dialog: Option<crate::model::TwistSettings>,
    /// Tween Instances: how many copies in between, and whether the run is dealt out to the states.
    pub tween_dialog: Option<(u32, bool)>,
    pub gif_dialog: Option<GifDialog>,
    /// Whether the timeline is being played.
    pub frames_playing: bool,
    /// How many frames the film has been playing for, which is the time its behaviors go by.
    pub behavior_tick: usize,
    /// Whether the picture of the page now on the canvas was made with the behaviors at work.
    raster_behaved: bool,
    /// Rules as Text while it is open: the text being written, and where its lines are scrolled to.
    pub script: Option<crate::script_edit::ScriptEdit>,
    pub script_scroll: gpui::ScrollHandle,
    /// Which of the things offered as a line is typed is picked, and whether the offers have been waved away for now.
    pub script_pick: usize,
    pub script_quiet: bool,
    /// Whether Rules and Numbers is open, and which rule it shows.
    pub rules_dialog: bool,
    pub rule_picked: usize,
    /// Whether Behavior Settings is open.
    pub behavior_dialog: bool,
    /// Behaviors > Attach to Another Object: the list of objects to attach the selection to.
    pub attach_dialog: bool,
    /// Whether playing in the program starts over at the end. Off, as in Flash, it plays once and stops on the last frame.
    pub loop_playback: bool,
    /// Where the frames of the timeline are scrolled to.
    pub timeline_scroll: gpui::ScrollHandle,
    /// The menu a right-click opened: where the click was, and what the menu offers.
    pub context_menu: Option<(Point<Pixels>, &'static [crate::menus::Entry])>,
    /// In Preview, the button under the mouse and the look it shows for it: 1 Over, 2 Down.
    pub preview_button: Option<(u64, usize)>,
    /// Animate Selection and its Settings: the settings as edited so far.
    pub animate_dialog: Option<crate::model::AnimationSettings>,
    pub print_dialog: Option<PrintDialog>,
    pub image_preview: Option<ImagePreview>,
    /// What Edit > Preferences sets, and while its dialog is open what to go back to on Cancel.
    pub prefs: crate::settings::Preferences,
    pub prefs_dialog: Option<crate::settings::Preferences>,
    /// The keyboard shortcuts the user has changed, by the English menu name of the command.
    pub shortcuts: std::collections::HashMap<String, String>,
    /// Edit > Keyboard Shortcuts while it is open: the command picked in the list, and whether the next key press is its new shortcut.
    pub shortcut_dialog: Option<(Option<usize>, bool)>,
    pub spell: Option<SpellSession>,
    /// The dialog that edits a gradient or pattern fill, holding the fill to go back to on Cancel.
    pub fill_dialog: Option<Arc<FillStyle>>,
    /// The bitmap filter last used from the Filters menu, for Repeat Filter.
    pub last_filter: Option<crate::bitmap::Filter>,
    /// The dialog of a marquee command that is open, and the number of pixels in it.
    pub marquee_dialog: Option<(crate::menus::MarqueeOp, f32)>,
    /// The stop picked in the gradient of Photoshop Live Effects, and the one being dragged.
    pub gradient_stop: Option<usize>,
    pub gradient_drag: Option<usize>,
    /// Whether the stop picked is one of the opacity stops above the gradient, not one of the color stops under it.
    pub gradient_opacity: bool,
    pub gradient_bounds: Rc<Cell<Bounds<Pixels>>>,
    /// With Preview off in a filter dialog, the filter as edited so far; the object
    /// meanwhile shows the filters it had before the dialog opened.
    pub filter_draft: Option<LiveFilter>,
    /// The eyedropper of Levels or Curves waiting for a click on the object: 0 black, 1 gray, 2 white.
    pub filter_dropper: Option<usize>,
    /// The slider of Levels being dragged: black, gamma and white on the way in, then black and white on the way out.
    pub level_drag: Option<usize>,
    /// Where the two rows of Levels sliders are on screen: the way in, then the way out.
    pub level_bounds: [Rc<Cell<Bounds<Pixels>>>; 2],
    /// The JPEG mask as rectangles to tint, kept until the mask changes.
    jpeg_tint: std::cell::RefCell<Option<(usize, Arc<Vec<Rect>>)>>,
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
    /// How the next text made with the Text tool is set.
    pub text_style: crate::model::TextStyle,
    /// The size of the next text made with the Text tool.
    pub text_size: f32,
    pub show_guides: bool,
    pub lock_guides: bool,
    pub snap_guides: bool,
    pub show_smart_guides: bool,
    pub snap_smart_guides: bool,
    pub show_slice_guides: bool,
    /// The smart guides the selection being moved is lined up on: whether each runs across, and where it is.
    pub smart_lines: Vec<(bool, f32)>,
    /// The color guides are drawn in.
    pub guide_color: Color,
    /// Move Guide: the guide that was double-clicked (whether it runs across, and which), and the place typed for it.
    pub guide_dialog: Option<(bool, usize, f32)>,
    /// View > Guides > Edit Guides, while it is open.
    pub guides_dialog: bool,
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
        Shape::Rect { .. } => t("Rectangle"),
        Shape::Ellipse => t("Ellipse"),
        Shape::Line => t("Line"),
        Shape::Path { .. } => t("Path"),
        Shape::Text { .. } => t("Text"),
        Shape::Image { .. } => t("Bitmap"),
        Shape::Web { kind: WebKind::Hotspot, .. } => t("Hotspot"),
        Shape::Web { kind: WebKind::Slice, .. } => t("Slice"),
        Shape::Auto { kind, .. } => kind.label(),
        Shape::Instance { .. } => t("Symbol"),
        Shape::Group { .. } => tc("object", "Group"),
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
    // Transformed text is scaled through its placement, which set_bounds has already done.
    if object.placement.is_none() {
        if let Shape::Text { size, .. } = &mut object.shape {
            *size = (*size * sy).max(1.0);
            object.refit_text();
        }
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

/// The developer's site, and where the code of PisiWorks and PisiAnimasyon is: About shows them, and its buttons open them.
pub const SITE: &str = "https://muratsapmaz.web.app/";
pub const SOURCE: &str = "https://github.com/MuratSAPMAZ/Pisiworks";
/// The site the two programs share, with the book.
pub const HOME: &str = "https://muratsapmaz.github.io/PisiAnimasyon/";

/// What Help > About says: what the program is and who makes it, then whose work it stands on.
pub fn about_text() -> String {
    about_text_in(crate::i18n::language(), is_sahne())
}

/// What About says in a language, in PisiAnimasyon or in PisiWorks: the same but for what the program is.
fn about_text_in(language: crate::i18n::Language, sahne: bool) -> String {
    let t = |text: &'static str| language.translate(text);
    [
        t("Version {}").replace("{}", env!("CARGO_PKG_VERSION")),
        String::new(),
        // The box is narrow and cuts off at both ends what does not fit, an address above all, which it cannot break:
        // every line is short enough to be read whole.
        if sahne { t("An animation program for Pardus,") } else { t("A Fireworks-like design program") }.to_string(),
        if sahne { t("built on PisiWorks.") } else { t("for Pardus, in Turkish.") }.to_string(),
        String::new(),
        t("Developed by Murat SAPMAZ").to_string(),
        SITE.to_string(),
        String::new(),
        HOME.trim_start_matches("https://").trim_end_matches('/').to_string(),
        SOURCE.trim_start_matches("https://").to_string(),
        String::new(),
        t("Thanks to Med Amine Belbachir:").to_string(),
        t("Built on his Emberframe (MIT).").to_string(),
        "github.com/gregstein/emberframe".to_string(),
        String::new(),
        t("Free software: GNU GPL 3 or later.").to_string(),
        t("It comes with no warranty.").to_string(),
        t("For schools, free of charge;").to_string(),
        t("it has no commercial aim.").to_string(),
        t("Not affiliated with Adobe.").to_string(),
    ]
    .join("\n")
}

/// An animation made of picture files, one state for each picture, in the order
/// given. An animated GIF among them gives all its pictures. Says what went wrong otherwise.
pub fn animation_from_files(files: &[PathBuf], delay_ms: u32, loops: u16) -> Result<Document, String> {
    let mut frames = Vec::new();
    for path in files {
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let bytes = std::fs::read(path).map_err(|error| format!("{name}: {error}"))?;
        match crate::optimize::gif_frames(&bytes) {
            Some((pictures, _)) => frames.extend(pictures.into_iter().map(|(picture, _)| (picture, delay_ms))),
            None => frames.push((Bitmap::decode(&bytes).map_err(|error| format!("{name}: {error}"))?, delay_ms)),
        }
    }
    if frames.len() < 2 {
        return Err(t("An animation needs at least two pictures").to_string());
    }
    Document::from_frames(t("Bitmap"), frames, loops).ok_or_else(|| t("An animation needs at least two pictures").to_string())
}

/// How wide a frame is in the timeline, in pixels.
pub const TIMELINE_CELL: f32 = 11.0;

/// Where the PisiAnimasyon book is, if it is there: beside the program as the package installs it, or, when the
/// program is run from its sources, where `packaging/kitap.py` builds it.
fn book_index() -> Option<std::path::PathBuf> {
    let beside = std::env::current_exe().ok().and_then(|exe| Some(exe.parent()?.parent()?.join("share/doc/pisiworks/kitap")));
    let folders = [beside, Some("/usr/share/doc/pisiworks/kitap".into()), Some(concat!(env!("CARGO_MANIFEST_DIR"), "/target/kitap").into())];
    // The book with its films playing is opened where it is there; the one with pictures only is a link away in it.
    folders.into_iter().flatten().flat_map(|folder: std::path::PathBuf| [folder.join("index-canli.html"), folder.join("index.html")]).find(|place| place.is_file())
}

/// A way to put right at once what a piece of advice in Rules as Text is about.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptFix {
    /// Give every object of this name the behavior Fall and Jump.
    Falls(String),
    /// Write the second name wherever the first stands in the text.
    Rename(String, String),
}

/// The frame rate a new animation starts with, as Flash's does.
pub const DEFAULT_FPS: f32 = 24.0;

/// A document as the running program works on it: in the animation program every page is a scene with a timeline.
fn staged(mut doc: Document) -> Document {
    if is_sahne() {
        for page in &mut doc.pages {
            page.start_timeline(DEFAULT_FPS);
        }
    }
    doc
}

/// The names of the heads Add Arrowheads offers.
pub const ARROW_STYLES: [&str; 5] = ["Triangle", "Lines", "Diamond", "Circle", "Square"];

/// The outline of an arrowhead whose point is at `tip`, on a path that arrives from
/// `from`: whether it is a closed shape, and its corners. `size` is its length.
pub fn arrow_head(style: usize, tip: Vec2, from: Vec2, size: f32) -> (bool, Vec<Vec2>) {
    let (dx, dy) = (tip.x - from.x, tip.y - from.y);
    let length = dx.hypot(dy).max(0.001);
    let (ux, uy) = (dx / length, dy / length);
    // A place given by how far back from the point it is and how far to the side.
    let at = |back: f32, side: f32| v2(tip.x - ux * back - uy * side, tip.y - uy * back + ux * side);
    let half = size * 0.4;
    match style {
        1 => (false, vec![at(size, half), tip, at(size, -half)]),
        2 => (true, vec![tip, at(size / 2.0, half), at(size, 0.0), at(size / 2.0, -half)]),
        3 => (true, (0..24).map(|step| {
            let angle = step as f32 / 24.0 * std::f32::consts::TAU;
            at(size / 2.0 - angle.cos() * size / 2.0, angle.sin() * size / 2.0)
        }).collect()),
        4 => (true, vec![at(0.0, half), at(size * 0.8, half), at(size * 0.8, -half), at(0.0, -half)]),
        // The point of the head is the end of the path; its back is four fifths as wide as it is long.
        _ => (true, vec![tip, at(size, half), at(size, -half)]),
    }
}

enum Opened {
    Document(Document),
    Image(Bitmap),
    /// An animated GIF, its pictures laid out as states.
    Animation(Document),
    /// A Fireworks PNG read with its layers.
    Fireworks(Document, crate::fwpng::Report),
    /// A Fireworks PNG with this many objects that could not be read, opened as its flat picture.
    Flat(Bitmap, usize),
}

/// Identifies a file by content, so a misnamed file still opens correctly.
fn parse_file(bytes: &[u8]) -> Result<Opened> {
    if bytes.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{') {
        let text = std::str::from_utf8(bytes).context(t("not an Emberframe document"))?;
        return document_from_json(text).map(Opened::Document);
    }
    // An animated GIF opens as Fireworks opens it: a state for each of its pictures.
    if let Some((frames, loops)) = crate::optimize::gif_frames(bytes) {
        if let Some(document) = Document::from_frames(t("Bitmap"), frames, loops) {
            return Ok(Opened::Animation(document));
        }
    }
    // A Fireworks PNG opens with its layers if all of it can be read, and as the flat picture it carries if not.
    match crate::fwpng::read(bytes) {
        Ok(Some((document, report))) if report.complete() => return Ok(Opened::Fireworks(document, report)),
        Ok(Some((_, report))) => return Bitmap::decode(bytes).map(|bitmap| Opened::Flat(bitmap, report.unread_objects)),
        _ => {}
    }
    Bitmap::decode(bytes).map(Opened::Image)
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map_or_else(|| t("Untitled").into(), |s| s.to_string_lossy().into_owned())
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            ed: Editor::new(staged(Document::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))),
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
            pen_handles: Vec::new(),
            point_selection: Vec::new(),
            pen_resume: None,
            awaiting_answer: false,
            panel_drag: None,
            transform: None,
            tool_options: ToolOptions::default(),
            transform_tool: Tool::Scale,
            group_tools: crate::menus::TOOL_GROUPS.iter().filter_map(|group| match group.first().map(|slot| slot.action) { Some(crate::menus::SlotAction::Select(tool)) => Some(tool), _ => None }).collect(),
            polygon_lasso: None,
            polygon_web: None,
            nine_guides: [1.0 / 3.0, 2.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0],
            tool_hold: 0,
            cursor_doc: Vec2::default(),
            cursor_screen: Point::default(),
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
            // Flash keeps layers in the timeline and has no states: those two panels start closed in the animation program.
            hidden_panels: if is_sahne() { vec![Panel::Layers, Panel::States] } else { Vec::new() },
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
            page_raster: None,
            size_dialog: None,
            numeric_dialog: None,
            jpeg_dialog: None,
            filter_dialog: None,
            filter_drag: None,
            filter_histogram: None,
            curve_drag: None,
            curve_point: None,
            curve_bounds: Rc::new(Cell::new(Bounds::default())),
            ps_effect: 0,
            print_setup: Settings::load().print,
            page_setup_dialog: None,
            symbol_dialog: None,
            template_dialog: None,
            template_name: None,
            batch_dialog: None,
            saved_commands: crate::commands::saved(),
            recorded: Vec::new(),
            commands_dialog: None,
            command_name: None,
            reverse_dialog: None,
            arrow_dialog: None,
            frame_dialog: None,
            mask_dialog: None,
            twist_dialog: None,
            tween_dialog: None,
            gif_dialog: None,
            frames_playing: false,
            behavior_tick: 0,
            raster_behaved: false,
            behavior_dialog: false,
            attach_dialog: false,
            script: None,
            script_scroll: gpui::ScrollHandle::new(),
            script_pick: 0,
            script_quiet: false,
            rules_dialog: false,
            rule_picked: 0,
            loop_playback: false,
            timeline_scroll: gpui::ScrollHandle::new(),
            context_menu: None,
            preview_button: None,
            animate_dialog: None,
            print_dialog: None,
            image_preview: None,
            prefs: Settings::load().preferences,
            prefs_dialog: None,
            shortcuts: Settings::load().shortcuts,
            shortcut_dialog: None,
            spell: None,
            fill_dialog: None,
            last_filter: None,
            marquee_dialog: None,
            gradient_stop: None,
            gradient_drag: None,
            gradient_opacity: false,
            gradient_bounds: Rc::new(Cell::new(Bounds::default())),
            filter_draft: None,
            filter_dropper: None,
            level_drag: None,
            level_bounds: [Rc::new(Cell::new(Bounds::default())), Rc::new(Cell::new(Bounds::default()))],
            jpeg_tint: std::cell::RefCell::new(None),
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
            text_style: Default::default(),
            text_size: 24.0,
            show_guides: true,
            lock_guides: false,
            snap_guides: true,
            show_smart_guides: true,
            snap_smart_guides: true,
            show_slice_guides: true,
            smart_lines: Vec::new(),
            guide_color: Color::rgb(0, 192, 0),
            guide_dialog: None,
            guides_dialog: false,
            space_held: false,
            status: String::new(),
            keep_overlays: false,
            last_title: String::new(),
        }
    }

    pub fn title(&self) -> String {
        let dirty = if self.ed.is_dirty() { "*" } else { "" };
        format!("{}{dirty} - {}", self.document_name(), app_name())
    }

    pub fn document_name(&self) -> String {
        self.path
            .as_ref()
            .or(self.source_image.as_ref())
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("{}.{}", t("Untitled"), document_extension()))
    }

    /// Name offered by Save As; an opened image becomes a document beside it.
    fn save_name(&self) -> String {
        match (&self.path, &self.source_image) {
            (None, Some(source)) => format!("{}.{}", file_stem(source), document_extension()),
            _ => self.document_name(),
        }
    }

    /// Stroke width of the active freehand tool, in document units.
    pub fn freehand_width(&self) -> f32 {
        match self.tool {
            Tool::Pencil => 1.0,
            // A path drawn freehand takes the stroke width, like any other vector.
            Tool::VectorPath => self.stroke_width.max(1.0),
            _ => self.brush_size,
        }
    }

    /// The tool a key of a shared button goes to: the next of its group if one of
    /// them is in hand, otherwise the one used last.
    fn group_key(&self, first: Tool) -> Tool {
        let Some(group) = crate::menus::tool_group(first) else { return first };
        let tools: Vec<Tool> = crate::menus::TOOL_GROUPS[group].iter().filter_map(|slot| match slot.action {
            crate::menus::SlotAction::Select(tool) => Some(tool),
            _ => None,
        }).collect();
        match tools.iter().position(|tool| *tool == self.tool) {
            Some(at) => tools[(at + 1) % tools.len()],
            None => self.group_tools.get(group).copied().unwrap_or(first),
        }
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
        self.ed.sync_mask_edit();
        self.ed.sync_open_groups();
        // A page with blend modes is rendered whole, as it will export. It is rendered
        // again when the document changes, and on every frame of an edit in progress.
        let editing = self.drag.is_some() || self.transform.is_some() || self.text_edit.is_some() || self.field.is_some();
        // While the film plays, behaviors move things: the page is drawn anew on each frame with them at work.
        let behaving = self.frames_playing && self.ed.doc.page().has_behaviors();
        let wanted = self.ed.needs_compositing() || behaving;
        let stale = behaving || self.raster_behaved || self.page_raster.as_ref().is_none_or(|(revision, _)| *revision != self.ed.revision || editing);
        if !wanted || stale {
            if let Some((_, image)) = self.page_raster.take() {
                self.stale_previews.push(image);
            }
        }
        if wanted && self.page_raster.is_none() {
            // A group that is open is drawn whole, so that its own mask and filters still show.
            // In an animation, masks whose layers are locked show at work on the stage, as in Flash.
            let closed = if self.ed.doc.page().shows_locked_masks() {
                Some(self.ed.doc.page().with_locked_masks())
            } else {
                self.ed.doc.page().open_groups_need_drawing().then(|| self.ed.doc.page().with_groups_closed())
            };
            let moving = behaving.then(|| closed.as_ref().unwrap_or(self.ed.doc.page()).behaved(self.behavior_tick));
            self.raster_behaved = behaving;
            self.page_raster = crate::export::render_page(moving.as_ref().or(closed.as_ref()).unwrap_or(self.ed.doc.page()))
                .ok()
                .and_then(|pixmap| crate::canvas_view::pixmap_texture(&pixmap))
                .map(|image| (self.ed.revision, image));
        }
        for image in self.stale_previews.drain(..) {
            cx.drop_image(image, Some(window));
        }
    }

    /// The live filters of the selection.
    pub fn current_filters(&self) -> Vec<LiveFilter> {
        self.ed.selected().first().map_or_else(Vec::new, |o| o.filters.clone())
    }

    /// The live filters as the list in Properties shows them: with Preview off in a
    /// filter's dialog the object does not have that filter's new settings, but the list still names it.
    pub fn listed_filters(&self) -> Vec<LiveFilter> {
        match (&self.filter_dialog, &self.filter_draft) {
            (Some((index, before)), Some(draft)) => crate::effects::placed(before, *index, draft.clone()),
            _ => self.current_filters(),
        }
    }

    /// Changes the live filters of every selected object.
    pub fn change_filters(&mut self, change: impl Fn(&mut Vec<LiveFilter>)) {
        self.ed.for_selected(|o| change(&mut o.filters));
        self.ed.commit(t("Live Filters"));
    }

    /// Adds a filter to the selection and opens its settings, if it has any.
    pub fn add_filter(&mut self, kind: FilterKind) {
        let before = self.current_filters();
        self.change_filters(|filters| filters.push(LiveFilter::new(kind)));
        if !kind.settings().is_empty() || kind.start_color().is_some() {
            self.show_filter_dialog(before.len(), before);
        }
    }

    pub fn open_filter(&mut self, index: usize) {
        let filters = self.current_filters();
        if filters.get(index).is_some_and(|f| !f.kind.settings().is_empty() || f.kind.start_color().is_some()) {
            self.show_filter_dialog(index, filters);
        }
    }

    fn show_filter_dialog(&mut self, index: usize, before: Vec<LiveFilter>) {
        self.filter_dialog = Some((index, before));
        (self.curve_drag, self.curve_point, self.ps_effect) = (None, None, 0);
        (self.filter_draft, self.filter_dropper, self.level_drag) = (None, None, None);
        (self.gradient_stop, self.gradient_drag) = (None, None);
        self.filter_histogram = self.histogram_before(index).map(Rc::new);
    }

    fn histogram_before(&self, index: usize) -> Option<[[u32; 256]; 3]> {
        histogram_before(self.ed.selected().first()?, index)
    }

    /// Takes hold of a filter in the list, to drag it to another place or, with no drag, to open it.
    pub fn begin_filter_drag(&mut self, index: usize, at: Point<Pixels>) {
        self.finish_pending();
        self.filter_drag = Some((index, at, false));
    }

    /// Moves the dragged filter past its neighbors, one for each row the mouse has crossed.
    fn drag_filter(&mut self, at: Point<Pixels>) {
        let Some((index, held, moved)) = self.filter_drag else { return };
        let (to, held) = dragged_place(index, held, at, self.current_filters().len());
        if to != index {
            self.ed.for_selected(|o| {
                if index.max(to) < o.filters.len() {
                    let filter = o.filters.remove(index);
                    o.filters.insert(to, filter);
                }
            });
        }
        self.filter_drag = Some((to, held, moved || to != index));
    }

    fn finish_filter_drag(&mut self) {
        match self.filter_drag.take() {
            Some((_, _, true)) => self.ed.commit(t("Live Filters")),
            Some((index, _, false)) => self.open_filter(index),
            None => {}
        }
    }

    /// The channel the Levels or Curves dialog is showing: 0 for all, then red, green and blue.
    pub fn filter_channel(&self) -> usize {
        self.edited_filter().map_or(0, |(_, filter)| match filter.kind {
            FilterKind::Levels => filter.value(5).round() as usize,
            _ => filter.value(0).round() as usize,
        })
    }

    /// Changes the filter being edited: on the selection, or, with Preview off, the
    /// copy kept aside. `commit` makes it a step in the history; dragging leaves that to the release.
    fn edit_filter(&mut self, commit: bool, change: impl Fn(&mut LiveFilter)) {
        let Some(index) = self.filter_dialog.as_ref().map(|(index, _)| *index) else { return };
        if let Some(draft) = self.filter_draft.as_mut() {
            change(draft);
            return;
        }
        self.ed.for_selected(|o| {
            if let Some(filter) = o.filters.get_mut(index) {
                change(filter);
            }
        });
        if commit {
            self.ed.commit(t("Live Filters"));
        }
    }

    fn touch_edited_filter(&mut self, change: impl Fn(&mut LiveFilter)) {
        self.edit_filter(false, change);
    }

    /// Whether the object shows the filter while its dialog is open.
    pub fn filter_preview(&self) -> bool {
        self.filter_draft.is_none()
    }

    /// The Preview box: off, the object goes back to how it was before the dialog,
    /// and the settings are kept aside until it is on again or the dialog closes.
    pub fn toggle_filter_preview(&mut self) {
        self.commit_field();
        let Some((index, before)) = self.filter_dialog.clone() else { return };
        match self.filter_draft.take() {
            Some(draft) => {
                let filters = crate::effects::placed(&before, index, draft);
                self.ed.for_selected(|o| o.filters = filters.clone());
            }
            None => {
                self.filter_draft = self.edited_filter().map(|(_, filter)| filter);
                self.ed.for_selected(|o| o.filters = before.clone());
            }
        }
    }

    /// Arms one of the eyedroppers, or puts it down again.
    pub fn pick_filter_dropper(&mut self, dropper: usize) {
        self.commit_field();
        self.filter_dropper = if self.filter_dropper == Some(dropper) { None } else { Some(dropper) };
    }

    /// A click on the canvas with an eyedropper armed: the color of the object
    /// there, as the filter receives it, becomes black, gray or white.
    pub fn use_filter_dropper(&mut self, at: Point<Pixels>) -> bool {
        let Some(dropper) = self.filter_dropper else { return false };
        let Some((index, _)) = self.filter_dialog.as_ref().map(|(index, before)| (*index, before.len())) else { return false };
        let p = self.view_map().to_doc(at);
        let sample = self.ed.selected().first().and_then(|object| sample_before(object, index, p));
        if let Some(sample) = sample {
            self.edit_filter(true, |filter| filter.pick_point(dropper, sample));
            (self.filter_dropper, self.curve_point) = (None, None);
        }
        true
    }

    /// The gradient of the effect on show in Photoshop Live Effects, by its number there.
    fn shown_filter_gradient(&self) -> Option<usize> {
        let (_, filter) = self.edited_filter()?;
        crate::pseffects::gradient_of(&filter, self.ps_effect)
    }

    /// The stops of the gradient being edited, of the kind picked last (color or
    /// opacity): the fill's while its dialog is open, otherwise that of the effect
    /// on show in Photoshop Live Effects.
    pub fn shown_stops(&self) -> Option<Vec<crate::pseffects::Stop>> {
        self.shown_track(self.gradient_opacity)
    }

    /// The color stops of the gradient being edited, or its opacity stops.
    pub fn shown_track(&self, opacity: bool) -> Option<Vec<crate::pseffects::Stop>> {
        if self.fill_dialog.is_some() {
            let style = self.current_fill_style().filter(|style| style.kind != FillKind::Pattern)?;
            return Some(if opacity { style.opacity_stops } else { style.stops });
        }
        let gradient = self.shown_filter_gradient()?;
        Some(self.edited_filter()?.1.gradient_stops(gradient, opacity))
    }

    /// Writes the stops of the gradient being edited back to where they came from.
    fn write_stops(&mut self, stops: Vec<crate::pseffects::Stop>, commit: bool) {
        let opacity = self.gradient_opacity;
        if self.fill_dialog.is_some() {
            self.change_fill_style(commit, |style| if opacity { style.opacity_stops = stops.clone() } else { style.stops = stops.clone() });
        } else if let Some(gradient) = self.shown_filter_gradient() {
            self.edit_filter(commit, |filter| filter.set_gradient_stops(gradient, opacity, stops.clone()));
        }
    }

    /// The gradient stop that is picked.
    pub fn picked_stop(&self) -> Option<crate::pseffects::Stop> {
        self.shown_stops()?.get(self.gradient_stop?).copied()
    }

    /// Changes the picked stop of the gradient on show.
    fn change_picked_stop(&mut self, change: impl Fn(&mut crate::pseffects::Stop)) {
        let (Some(mut stops), Some(index)) = (self.shown_stops(), self.gradient_stop) else { return };
        let Some(stop) = stops.get_mut(index) else { return };
        change(stop);
        // Typing a place can carry the stop past its neighbors.
        let (stops, held) = crate::pseffects::stop_dragged(stops.clone(), index, stops[index].0, false);
        self.gradient_stop = held;
        self.write_stops(stops, true);
    }

    /// Where a place on screen is along the gradient, from 0 to 1, how far a stop
    /// reaches, whether it is above the gradient, where the opacity stops are, and
    /// whether it is well away from the row of stops being worked on.
    fn gradient_place(&self, at: Point<Pixels>) -> (f32, f32, bool, bool) {
        let bounds = self.gradient_bounds.get();
        let width = f32::from(bounds.size.width).max(1.0);
        let (above, below) = (f32::from(bounds.origin.y - at.y), f32::from(at.y - bounds.origin.y) - f32::from(bounds.size.height));
        let away = if self.gradient_opacity { !(-25.0..=45.0).contains(&above) } else { !(-45.0..=30.0).contains(&below) };
        (f32::from(at.x - bounds.origin.x) / width, 6.0 / width, above > 0.0, away)
    }

    /// A press beside the gradient: above it works on the opacity stops, under it
    /// on the color stops. It picks the stop there, or adds one.
    pub fn press_gradient(&mut self, at: Point<Pixels>) {
        self.commit_field();
        let (place, reach, above, _) = self.gradient_place(at);
        self.gradient_opacity = above;
        let Some(stops) = self.shown_stops() else { return };
        let (stops, index) = if above && stops.is_empty() {
            // The first opacity stop is opaque: it changes nothing until it is set.
            (vec![(place.clamp(0.0, 1.0), Color::BLACK)], 0)
        } else {
            crate::pseffects::stop_pressed(stops, place, reach)
        };
        self.write_stops(stops, false);
        (self.gradient_stop, self.gradient_drag) = (Some(index), Some(index));
    }

    /// Moves the held gradient stop to the mouse; pulled well away from the gradient, it is taken off.
    /// The last two color stops stay; the opacity stops can all go.
    pub fn drag_gradient(&mut self, at: Point<Pixels>) -> bool {
        let (Some(index), Some(mut stops)) = (self.gradient_drag, self.shown_stops()) else { return false };
        let (place, _, _, away) = self.gradient_place(at);
        let (stops, held) = if self.gradient_opacity && away && index < stops.len() {
            stops.remove(index);
            (stops, None)
        } else {
            crate::pseffects::stop_dragged(stops, index, place, away)
        };
        (self.gradient_stop, self.gradient_drag) = (held, held);
        self.write_stops(stops, false);
        true
    }

    /// Replaces the gradient on show with one of the ready-made ones.
    pub fn use_gradient_preset(&mut self, which: usize) {
        // A ready-made gradient brings its own opacity with its colors.
        self.gradient_stop = None;
        self.gradient_opacity = true;
        self.write_stops(Vec::new(), false);
        self.gradient_opacity = false;
        self.write_stops(crate::pseffects::preset(which), true);
    }

    /// The gradient or pattern the first selected object is filled with.
    pub fn current_fill_style(&self) -> Option<FillStyle> {
        self.ed.selected().iter().find(|o| o.takes_color()).and_then(|o| o.fill_style.as_deref().cloned())
    }

    /// Changes the gradient or pattern fill of every selected object that has one.
    pub fn change_fill_style(&mut self, commit: bool, change: impl Fn(&mut FillStyle)) {
        self.ed.for_selected(|o| {
            if let Some(style) = o.fill_style.as_mut() {
                change(Arc::make_mut(style));
            }
        });
        if commit {
            self.ed.commit(t("Fill"));
        }
    }

    /// The Fill category list: one color, one of the gradients, or a pattern.
    /// A new gradient runs from the object's fill color to white, or to black from a light color.
    pub fn set_fill_kind(&mut self, kind: Option<FillKind>) {
        let fallback = self.fill.unwrap_or(Color::BLACK);
        self.ed.for_selected(|o| {
            if !o.takes_color() {
                return;
            }
            o.fill_style = kind.map(|kind| {
                let mut style = o.fill_style.as_deref().cloned().unwrap_or_else(|| {
                    let from = o.fill.unwrap_or(fallback);
                    let light = from.r as u32 + from.g as u32 + from.b as u32 > 600;
                    FillStyle::new(kind, from, if light { Color::BLACK } else { Color::rgb(255, 255, 255) })
                });
                if style.kind != kind {
                    (style.start, style.end) = kind.start_handles();
                    style.kind = kind;
                }
                Arc::new(style)
            });
            // Something must be there to fill with: a fill of "none" would hide the gradient.
            if kind.is_some() && o.fill.is_none() {
                o.fill = Some(fallback);
            }
        });
        self.ed.commit(t("Fill"));
    }

    /// Opens the dialog that edits the gradient or pattern of the selection.
    pub fn open_fill_dialog(&mut self) {
        self.finish_pending();
        if let Some(style) = self.current_fill_style() {
            self.fill_dialog = Some(Arc::new(style));
            (self.gradient_stop, self.gradient_drag) = (None, None);
        }
    }

    /// Closes the fill dialog, putting the fill back as it was unless `keep` is set.
    pub fn close_fill_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some(before) = self.fill_dialog.take() else { return };
        (self.gradient_stop, self.gradient_drag) = (None, None);
        if !keep {
            self.ed.for_selected(|o| {
                if o.fill_style.is_some() {
                    o.fill_style = Some(before.clone());
                }
            });
            self.ed.commit(t("Fill"));
        }
    }

    /// The two handles of the gradient of the one selected object, on the page: where it starts and where it ends.
    pub fn fill_handles(&self) -> Option<(Vec2, Vec2, Option<Vec2>)> {
        if self.tool != Tool::Pointer || self.ed.selection.len() != 1 {
            return None;
        }
        let object = self.ed.selected().into_iter().next()?;
        let style = object.fill_style.as_deref().filter(|style| !matches!(style.kind, FillKind::Pattern | FillKind::Contour | FillKind::WebDither))?;
        let (start, end) = style.handles(object.bounds);
        // An Ellipse or Rectangle gradient has a third handle, for how wide it is.
        Some((start, end, style.width_handle(object.bounds)))
    }

    /// The edge and texture of the first selected vector's fill.
    pub fn current_fill_look(&self) -> crate::model::FillLook {
        self.ed.selected().iter().find(|o| o.takes_color()).map_or_else(Default::default, |o| o.fill_look)
    }

    /// Changes the edge or the texture of the fill of the selected vectors.
    pub fn change_fill_look(&mut self, change: impl Fn(&mut crate::model::FillLook)) {
        self.ed.for_selected(|o| {
            if o.takes_color() {
                change(&mut o.fill_look);
            }
        });
        self.ed.commit(t("Fill"));
    }

    /// The five numbers of the Levels channel on show.
    fn shown_levels(&self) -> Option<[f32; 5]> {
        let (_, filter) = self.edited_filter()?;
        let at = crate::effects::levels_start(self.filter_channel());
        Some([0, 1, 2, 3, 4].map(|offset| filter.value(at + offset)))
    }

    /// A press on one of the two rows of Levels sliders takes hold of the nearest slider there.
    pub fn press_level(&mut self, row: usize, at: Point<Pixels>) {
        self.commit_field();
        let Some(values) = self.shown_levels() else { return };
        let bounds = self.level_bounds[row].get();
        let level = f32::from(at.x - bounds.origin.x) / f32::from(bounds.size.width).max(1.0) * 255.0;
        let middle = values[0] + (values[2] - values[0]) * crate::effects::gamma_place(values[1]);
        let sliders: &[(usize, f32)] = if row == 0 { &[(0, values[0]), (1, middle), (2, values[2])] } else { &[(3, values[3]), (4, values[4])] };
        self.level_drag = sliders.iter().min_by(|a, b| (a.1 - level).abs().total_cmp(&(b.1 - level).abs())).map(|(slider, _)| *slider);
        self.drag_level(at);
    }

    /// Moves the held Levels slider to the mouse.
    pub fn drag_level(&mut self, at: Point<Pixels>) -> bool {
        let (Some(slider), Some(values)) = (self.level_drag, self.shown_levels()) else { return false };
        let bounds = self.level_bounds[if slider < 3 { 0 } else { 1 }].get();
        let level = f32::from(at.x - bounds.origin.x) / f32::from(bounds.size.width).max(1.0) * 255.0;
        let moved = crate::effects::levels_dragged(values, slider, level);
        let start = crate::effects::levels_start(self.filter_channel());
        self.edit_filter(false, |filter| {
            for (offset, value) in moved.into_iter().enumerate() {
                filter.set_value(start + offset, value);
            }
        });
        true
    }

    /// Where a place on screen falls in the curve box, in levels, and the size of the box.
    fn curve_level(&self, at: Point<Pixels>) -> ((f32, f32), (f32, f32)) {
        let bounds = self.curve_bounds.get();
        let (width, height) = (f32::from(bounds.size.width).max(1.0), f32::from(bounds.size.height).max(1.0));
        ((f32::from(at.x - bounds.origin.x) / width * 255.0, 255.0 - f32::from(at.y - bounds.origin.y) / height * 255.0), (width, height))
    }

    /// A press in the curve box: takes hold of the point there, or puts a new one on the curve.
    pub fn press_curve(&mut self, at: Point<Pixels>) {
        self.commit_field();
        let Some((_, filter)) = self.edited_filter() else { return };
        let channel = self.filter_channel();
        let (level, (width, _)) = self.curve_level(at);
        let (points, index) = crate::effects::curve_pressed(filter.curve(channel), level, 7.0 * 255.0 / width);
        self.touch_edited_filter(|f| f.set_curve(channel, points.clone()));
        (self.curve_drag, self.curve_point) = (Some(index), Some(index));
    }

    /// Moves the held point of the curve. It stays between its neighbors, and one
    /// dragged well out of the box is taken off the curve, unless it is an end.
    pub fn drag_curve(&mut self, at: Point<Pixels>) -> bool {
        let Some(index) = self.curve_drag else { return false };
        let Some((_, filter)) = self.edited_filter() else { return false };
        let channel = self.filter_channel();
        let (level, (width, _)) = self.curve_level(at);
        let (points, kept) = crate::effects::curve_dragged(filter.curve(channel), index, level, 24.0 * 255.0 / width);
        if !kept {
            (self.curve_drag, self.curve_point) = (None, None);
        }
        self.touch_edited_filter(|f| f.set_curve(channel, points.clone()));
        true
    }

    /// Lets go of a point of the curve or a slider of Levels.
    pub fn release_curve(&mut self) {
        let held = [self.curve_drag.take(), self.level_drag.take(), self.gradient_drag.take()];
        if held.iter().any(Option::is_some) && self.filter_draft.is_none() {
            self.ed.commit(t("Live Filters"));
        }
    }

    /// The Auto button of Levels and Curves: each color is stretched from its
    /// darkest level in the object to its lightest, as Auto Levels does.
    pub fn auto_filter(&mut self) {
        self.commit_field();
        let Some(histogram) = self.filter_histogram.clone() else { return };
        self.edit_filter(true, |filter| filter.stretch(&histogram));
        self.curve_point = None;
    }

    /// The filter whose settings are open.
    pub fn edited_filter(&self) -> Option<(usize, LiveFilter)> {
        let (index, _) = self.filter_dialog.as_ref()?;
        match &self.filter_draft {
            Some(draft) => Some((*index, draft.clone())),
            None => Some((*index, self.current_filters().get(*index)?.clone())),
        }
    }

    pub fn set_filter_value(&mut self, setting: usize, value: f32) {
        self.edit_filter(true, |filter| filter.set_value(setting, value));
    }

    /// Closes the filter settings, putting the filters back as they were unless `keep` is set.
    pub fn close_filter_dialog(&mut self, keep: bool) {
        self.commit_field();
        let draft = self.filter_draft.take();
        if let Some((index, before)) = self.filter_dialog.take() {
            match (keep, draft) {
                // With Preview off the object never got the settings, so it gets them now.
                (true, Some(draft)) => {
                    let filters = crate::effects::placed(&before, index, draft);
                    self.change_filters(|current| *current = filters.clone());
                }
                (true, None) => {}
                (false, _) => self.change_filters(|filters| *filters = before.clone()),
            }
        }
        (self.filter_histogram, self.curve_drag, self.curve_point) = (None, None, None);
        (self.filter_dropper, self.level_drag) = (None, None);
        (self.gradient_stop, self.gradient_drag) = (None, None);
    }

    /// The blend mode the selection has, or Normal with nothing selected.
    pub fn current_blend(&self) -> BlendMode {
        self.ed.selected().first().map_or(BlendMode::Normal, |o| o.blend)
    }

    pub fn set_blend(&mut self, mode: BlendMode) {
        self.ed.for_selected(|o| o.blend = mode);
        self.ed.commit(t("Blend Mode"));
    }

    pub fn view_map(&self) -> ViewMap {
        let page = self.ed.doc.page();
        ViewMap::new(self.canvas_bounds.get(), page.width as f32, page.height as f32, self.zoom, self.pan)
    }

    pub fn active_tool(&self) -> Tool {
        if self.space_held { Tool::Hand } else { self.tool }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        // Moving between Scale, Skew and Distort carries on with the same transform.
        let session = if is_transform_tool(tool) && is_transform_tool(self.tool) { self.transform.take() } else { None };
        self.finish_pending();
        self.transform = session;
        self.crop_area = None;
        self.tool = tool;
        self.point_selection.clear();
        self.polygon_lasso = None;
        self.polygon_web = None;
        if is_transform_tool(tool) {
            self.transform_tool = tool;
        }
        if let Some(slot) = crate::menus::tool_group(tool).and_then(|group| self.group_tools.get_mut(group)) {
            *slot = tool;
        }
        let hint = match tool {
            Tool::Scale => Some(t("Drag a handle to scale, or outside the box to rotate.")),
            Tool::Skew => Some(t("Drag a side to skew or a corner for perspective, or outside the box to rotate.")),
            Tool::Distort => Some(t("Drag a handle to distort, or outside the box to rotate.")),
            _ => None,
        };
        if let Some(hint) = hint {
            self.status = hint.into();
        }
    }

    /// Commits any in-progress field, text or pen edit.
    pub fn finish_pending(&mut self) {
        self.commit_field();
        self.finish_text();
        self.finish_pen(false);
        self.apply_transform();
    }

    /// The shared tool button was pressed. If it is still down after a moment, its tools are listed.
    pub fn begin_tool_hold(&mut self, position: Point<Pixels>, group: usize, cx: &mut Context<Self>) {
        self.tool_hold += 1;
        let token = self.tool_hold;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(TOOL_HOLD_MS)).await;
            this.update(cx, |this, cx| {
                if this.tool_hold == token {
                    // Beside the button, so letting go over the button picks nothing.
                    let beside = point(position.x + px(TOOL_FLYOUT_OFFSET), position.y - px(10.));
                    this.open_combo = Some((Combo::ToolGroup(group), beside));
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn end_tool_hold(&mut self) {
        self.tool_hold += 1;
    }

    /// Makes a previewed transform permanent, as Enter and a double-click do.
    pub fn apply_transform(&mut self) {
        let Some(session) = self.transform.take() else { return };
        // In PisiAnimasyon the point the box turns around stays with the object, as Flash's transformation point
        // does: a motion tween turns the object around it.
        let kept = self.kept_pivot(&session);
        if session.warp == Warp::identity() && kept.is_none() {
            return;
        }
        if let Some((id, point)) = kept {
            if let Some(object) = self.ed.object_mut(id) {
                object.set_pivot_point(Some(point));
            }
        }
        self.ed.commit(t("Transform"));
        let skipped = session.originals.iter().filter(|o| !o.warpable()).count();
        self.report_transform(skipped);
    }

    /// The object a session's center point is to stay with, and where the point is.
    fn kept_pivot(&self, session: &TransformSession) -> Option<(u64, Vec2)> {
        match (&session.originals[..], session.pivot) {
            ([only], Some(point)) if crate::theme::is_sahne() => Some((only.id, point)),
            _ => None,
        }
    }

    /// Where the one selected object's transformation point is, in PisiAnimasyon, if it has one of its own.
    fn own_pivot(&self) -> Option<Vec2> {
        match &self.ed.selected()[..] {
            [only] if crate::theme::is_sahne() => only.pivot_point(),
            _ => None,
        }
    }

    /// Drops a previewed transform and puts the artwork back, as Esc does.
    pub fn cancel_transform(&mut self) -> bool {
        if self.transform.take().is_none() {
            return false;
        }
        self.ed.rollback();
        true
    }

    /// The transform box to draw: the session's, or the upright selection box before one begins.
    pub fn transform_box(&self) -> Option<([Vec2; 4], Vec2)> {
        if !is_transform_tool(self.tool) {
            return None;
        }
        match &self.transform {
            Some(session) => Some((session.quad(), session.pivot())),
            None => self.ed.selection_bounds().map(|bounds| (rect_corners(bounds), self.own_pivot().unwrap_or(bounds.center()))),
        }
    }

    /// Redraws the selection from the untouched originals through the session's map.
    fn preview_transform(&mut self) {
        let Some(session) = &self.transform else { return };
        let (origin, warp) = (session.origin, session.warp);
        // A box that was only moved and scaled keeps rectangles, bitmaps and text as they are.
        let target = upright(session.quad());
        for original in &session.originals {
            let next = match target {
                Some(target) => Some(scale_object(original, origin, target)),
                None => {
                    let mut warped = original.clone();
                    warped.warp(&warp).then_some(warped)
                }
            };
            if let (Some(next), Some(object)) = (next, self.ed.object_mut(original.id)) {
                *object = next;
            }
        }
    }

    pub fn close_overlays(&mut self) {
        self.open_menu = None;
        self.open_sub = None;
        self.color_popup = None;
        self.open_combo = None;
        self.context_menu = None;
    }

    /// Opens a menu where the mouse was right-clicked.
    pub fn open_context_menu(&mut self, at: Point<Pixels>, entries: &'static [crate::menus::Entry], window: &Window) {
        self.finish_pending();
        self.close_overlays();
        self.frames_playing = false;
        // The menu is kept inside the window: near the right or bottom edge it opens leftward or upward.
        let view = window.viewport_size();
        let tall = entries.iter().map(|entry| if matches!(entry, crate::menus::Entry::Sep) { 9.0 } else { 20.0 }).sum::<f32>() + 10.0;
        let x = f32::from(at.x).min(f32::from(view.width) - 244.0).max(0.0);
        let y = f32::from(at.y).min(f32::from(view.height) - tall - 4.0).max(0.0);
        self.context_menu = Some((point(px(x), px(y)), entries));
    }

    /// A right-click on the canvas: what is under the pointer is selected, unless it already is, and the menu for it opens.
    pub fn canvas_context(&mut self, ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.awaiting_answer || self.view_mode != ViewMode::Original {
            return;
        }
        window.focus(&self.focus);
        self.finish_pending();
        let p = self.view_map().to_doc(ev.position);
        match self.ed.doc.page().hit_test(p, self.tolerance()) {
            Some(id) if !self.ed.is_selected(id) => self.ed.selection = vec![id],
            Some(_) => {}
            None => self.ed.selection.clear(),
        }
        self.open_context_menu(ev.position, if is_sahne() { crate::menus::STAGE_MENU } else { crate::menus::CANVAS_MENU }, window);
        cx.notify();
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
            Cmd::SnapToGrid => self.prefs.snap_to_grid,
            Cmd::TextFont(index) => crate::fonts::families().get(index as usize).is_some_and(|name| {
                let style = self.shown_text_style();
                *name == style.font || (style.font.is_empty() && name == crate::fonts::family())
            }),
            Cmd::TextSize(size) => self.shown_text_size() == size as f32,
            Cmd::TextStyle(style) => {
                let shown = self.shown_text_style();
                match style {
                    crate::menus::TextStyleCmd::Plain => !shown.bold && !shown.italic && !shown.underline,
                    crate::menus::TextStyleCmd::Bold => shown.bold,
                    crate::menus::TextStyleCmd::Italic => shown.italic,
                    crate::menus::TextStyleCmd::Underline => shown.underline,
                }
            }
            Cmd::TextAlign(align) => !self.shown_text_style().orientation.is_vertical() && self.shown_text_style().align == align,
            Cmd::TextAlignDown(align) => self.shown_text_style().orientation.is_vertical() && self.shown_text_style().align == align,
            Cmd::TextOrientation(orientation) => self.shown_text_style().orientation == orientation,
            Cmd::Frame(crate::menus::FrameCmd::GuideLayer) => self.active_layer_kind() == crate::model::LayerKind::Guide,
            Cmd::Frame(crate::menus::FrameCmd::MaskLayer) => self.active_layer_kind() == crate::model::LayerKind::Mask,
            Cmd::Frame(crate::menus::FrameCmd::LoopPlayback) => self.loop_playback,
            Cmd::Behavior(crate::menus::BehaviorCmd::Toggle(kind)) => !self.ed.selection.is_empty() && self.ed.selected().iter().all(|o| o.behaviors.iter().any(|b| b.kind == kind)),
            Cmd::Frame(crate::menus::FrameCmd::BindLayer) => {
                let page = self.ed.doc.page();
                page.timeline.as_ref().is_some_and(|timeline| timeline.is_bound(page.active_layer))
            }
            Cmd::PathOrientation(orientation) => self.ed.has_text_on_path() && self.shown_text_style().on_path == orientation,
            Cmd::Guides(GuideCmd::Show) => self.show_guides,
            Cmd::Guides(GuideCmd::Lock) => self.lock_guides,
            Cmd::Guides(GuideCmd::Snap) => self.snap_guides,
            Cmd::Guides(GuideCmd::ShowSmart) => self.show_smart_guides,
            Cmd::Guides(GuideCmd::SnapSmart) => self.snap_smart_guides,
            Cmd::Guides(GuideCmd::Slice) => self.show_slice_guides,
            Cmd::Language(language) => crate::i18n::language() == language,
            // Ticked while the mask is switched off.
            Cmd::MaskToggle => self.ed.selected().iter().any(|o| o.mask.as_ref().is_some_and(|mask| !mask.enabled)),
            _ => false,
        }
    }

    pub fn set_export(&mut self, change: impl FnOnce(&mut ExportSettings)) {
        change(&mut self.ed.doc.page_mut().export);
        self.ed.commit(t("Optimize"));
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
        let page = self.ed.export_page();
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
        let key = (self.ed.revision, self.view_mode, self.preview_button);
        if self.previews_key == Some(key) {
            return;
        }
        self.previews_key = Some(key);
        let mut page = self.ed.export_page();
        // In Preview a button answers the mouse as it will in a browser.
        if let Some((id, look)) = self.preview_button {
            page = page.with_button_look(id, look);
        }
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

    /// Keeps the pixel selection as the area a JPEG writes at its selective quality.
    fn save_jpeg_mask(&mut self) {
        let Some(selection) = self.active_pixels() else { return };
        let mask = selection.mask.clone();
        let Some(object) = self.ed.object(selection.object).cloned() else { return };
        let page = self.ed.doc.page();
        // The selection is in the pixels of its bitmap, and the mask in those of the page.
        let page_mask = Mask::from_fn(page.width, page.height, |x, y| {
            object.pixel_at(v2(x as f32 + 0.5, y as f32 + 0.5)).is_some_and(|(px, py)| mask.is_set(px, py))
        });
        if page_mask.is_empty() {
            return;
        }
        self.ed.doc.page_mut().jpeg_mask = Some(Arc::new(page_mask));
        self.ed.commit(t("Save Selection as JPEG Mask"));
    }

    /// Selects, on the selected bitmap, the pixels under the page's JPEG mask.
    fn restore_jpeg_mask(&mut self) {
        let Some(page_mask) = self.ed.doc.page().jpeg_mask.clone() else { return };
        let Some(object) = self.ed.selected().into_iter().find(|o| o.is_image()).cloned() else { return };
        let Some((width, height)) = object.bitmap().map(|b| (b.width(), b.height())) else { return };
        let (page_width, page_height) = page_mask.size();
        let mask = Mask::from_fn(width, height, |x, y| {
            object.pixel_to_doc(v2(x as f32 + 0.5, y as f32 + 0.5)).is_some_and(|p| {
                p.x >= 0.0 && p.y >= 0.0 && p.x < page_width as f32 && p.y < page_height as f32 && page_mask.is_set(p.x as u32, p.y as u32)
            })
        });
        self.set_pixel_selection(object.id, Some(mask), MaskMode::Replace);
    }

    /// The JPEG mask of the page as document rectangles, for the tint that shows where it is.
    pub fn jpeg_mask_rects(&self) -> Option<Arc<Vec<Rect>>> {
        let page = self.ed.doc.page();
        let mask = page.jpeg_mask.as_ref().filter(|mask| mask.size() == (page.width, page.height))?;
        let key = Arc::as_ptr(mask) as usize;
        let mut cache = self.jpeg_tint.borrow_mut();
        if cache.as_ref().is_none_or(|(cached, _)| *cached != key) {
            let rects = mask.rects().into_iter().map(|(x, y, w, h)| Rect { x: x as f32, y: y as f32, w: w as f32, h: h as f32 }).collect();
            *cache = Some((key, Arc::new(rects)));
        }
        cache.as_ref().map(|(_, rects)| rects.clone())
    }

    /// Closes the Selective JPEG Settings dialog, putting the settings back unless `keep` is set.
    pub fn close_jpeg_dialog(&mut self, keep: bool) {
        self.commit_field();
        if let Some((settings, color)) = self.jpeg_dialog.take().filter(|_| !keep) {
            let page = self.ed.doc.page();
            if settings != page.export || color != page.jpeg_mask_color {
                self.ed.doc.page_mut().jpeg_mask_color = color;
                self.set_export(|current| *current = settings);
            }
        }
    }

    fn open_numeric_dialog(&mut self, center_point: Option<Vec2>) {
        let Some(bounds) = self.ed.selection_bounds() else { return };
        self.numeric_dialog = Some(NumericDialog {
            mode: NumericMode::Scale,
            bounds,
            center: center_point.unwrap_or_else(|| bounds.center()),
            percent: (100.0, 100.0),
            size: (bounds.w, bounds.h),
            angle: 0.0,
            constrain: true,
            scale_attributes: true,
        });
    }

    /// Applies the Numeric Transform dialog around its center point and closes it.
    pub fn apply_numeric_dialog(&mut self) {
        self.commit_field();
        let Some(dialog) = self.numeric_dialog.take() else { return };
        let (bounds, center) = (dialog.bounds, dialog.center);
        let mut skipped = 0;
        if dialog.mode == NumericMode::Rotate {
            let warp = Warp::rotation(center, dialog.angle.to_radians());
            self.ed.for_selected(|o| {
                if !o.warp(&warp) {
                    skipped += 1;
                }
            });
        } else {
            let (w, h) = match dialog.mode {
                NumericMode::Scale => (bounds.w * dialog.percent.0 / 100.0, bounds.h * dialog.percent.1 / 100.0),
                _ => dialog.size,
            };
            let factor = |from: f32, to: f32| if from > 0.0 { to / from } else { 1.0 };
            let target = scaled_about(bounds, center, factor(bounds.w, w), factor(bounds.h, h));
            let growth = (factor(bounds.w, w) + factor(bounds.h, h)) / 2.0;
            self.ed.for_selected(|o| {
                *o = scale_object(o, bounds, target);
                if dialog.scale_attributes {
                    if let Some(stroke) = &mut o.stroke {
                        stroke.width *= growth;
                    }
                }
            });
        }
        self.ed.commit(t("Transform"));
        self.report_transform(skipped);
    }

    /// Applies the open size dialog and closes it.
    pub fn apply_size_dialog(&mut self) {
        self.commit_field();
        let Some(dialog) = self.size_dialog.take() else { return };
        match dialog.kind {
            SizeDialogKind::Canvas => {
                self.ed.resize_canvas(dialog.width, dialog.height);
                self.ed.commit(t("Canvas Size"));
            }
            SizeDialogKind::Image => {
                self.ed.resize_image(dialog.width, dialog.height);
                self.ed.commit(t("Image Size"));
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
        // In an animation the neighbours are the frames before and after the one on show.
        if let Some(timeline) = &page.timeline {
            let frames = [timeline.current.checked_sub(1), Some(timeline.current + 1).filter(|next| *next < timeline.frame_count())];
            return frames
                .into_iter()
                .flatten()
                .flat_map(|frame| (0..page.layers.len()).filter(|layer| page.layers[*layer].visible).flat_map(move |layer| timeline.objects_at(layer, frame)))
                .filter(|o| o.visible && !o.is_web())
                .map(|mut ghost| {
                    ghost.opacity *= ONION_OPACITY;
                    ghost
                })
                .collect();
        }
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
        self.frames_playing = false;
    }

    /// What the active layer is for in the timeline.
    pub fn active_layer_kind(&self) -> crate::model::LayerKind {
        let page = self.ed.doc.page();
        page.timeline.as_ref().map_or_else(Default::default, |timeline| timeline.kind(page.active_layer))
    }

    /// The tween of the keyframe the frame on show belongs to, on the active layer.
    pub fn current_tween(&self) -> Option<crate::model::Tween> {
        let page = self.ed.doc.page();
        let timeline = page.timeline.as_ref()?;
        let span = timeline.span_at(page.active_layer, timeline.current)?;
        timeline.layers[page.active_layer][span].tween.clone()
    }

    /// The shape hints to show on the stage: on the first keyframe of a shape tween the
    /// places the hints start at, on the keyframe it ends in the places they go to.
    /// Each comes with its number, and whether it is an end.
    pub fn shape_hints(&self) -> Vec<(Vec2, usize, bool)> {
        let page = self.ed.doc.page();
        let Some(timeline) = page.timeline.as_ref().filter(|_| is_sahne() && self.view_mode == ViewMode::Original) else { return Vec::new() };
        let (layer, frame) = (page.active_layer, timeline.current);
        let Some(index) = timeline.span_at(layer, frame) else { return Vec::new() };
        let spans = &timeline.layers[layer];
        let of = |span: &crate::model::Span, end: bool| span.tween.as_ref().filter(|tween| tween.shape).map_or_else(Vec::new, |tween| tween.hints.iter().enumerate().map(|(i, (a, b))| (if end { *b } else { *a }, i, end)).collect());
        if spans[index].start != frame {
            return Vec::new();
        }
        let mut shown = of(&spans[index], false);
        if index > 0 {
            shown.extend(of(&spans[index - 1], true));
        }
        shown
    }

    /// Moves a shape hint to the point of its shape's outline nearest `place`.
    fn move_hint(&mut self, number: usize, end: bool, place: Vec2) {
        let outline = self.ed.doc.page().layers.get(self.ed.doc.page().active_layer).and_then(|layer| layer.objects.first()).map(|o| o.outline()).unwrap_or_default();
        let dense = crate::model::outline_points(&outline, 256);
        let Some(snapped) = dense.into_iter().min_by(|a, b| (a.x - place.x).hypot(a.y - place.y).total_cmp(&(b.x - place.x).hypot(b.y - place.y))) else { return };
        let page = self.ed.doc.page_mut();
        let layer = page.active_layer;
        let Some(timeline) = page.timeline.as_mut() else { return };
        let Some(index) = timeline.span_at(layer, timeline.current) else { return };
        let span = if end { index.checked_sub(1) } else { Some(index) };
        if let Some(hint) = span.and_then(|span| timeline.layers[layer][span].tween.as_mut()).and_then(|tween| tween.hints.get_mut(number)) {
            if end {
                hint.1 = snapped;
            } else {
                hint.0 = snapped;
            }
        }
    }

    /// The frame on show and the number of frames of the scene, if the page has a timeline.
    pub fn frame_position(&self) -> Option<(usize, usize)> {
        self.ed.doc.page().timeline.as_ref().map(|timeline| (timeline.current, timeline.frame_count()))
    }

    /// Shows a frame of the scene, and makes a layer the active one if one is named.
    pub fn show_frame(&mut self, frame: usize, layer: Option<usize>) {
        self.finish_pending();
        self.frames_playing = false;
        if let Some(layer) = layer {
            self.ed.set_active_layer(layer);
        }
        self.ed.goto_frame(frame);
        self.reveal_frame(frame);
    }

    /// Scrolls the timeline, if need be, so that a frame is in sight.
    pub fn reveal_frame(&mut self, frame: usize) {
        let (offset, view) = (self.timeline_scroll.offset(), self.timeline_scroll.bounds().size.width);
        let (left, view) = (-f32::from(offset.x), f32::from(view));
        let at = frame as f32 * TIMELINE_CELL;
        if view > 0.0 && (at < left || at + TIMELINE_CELL > left + view) {
            self.timeline_scroll.set_offset(point(px(-(at - view / 2.0).max(0.0)), offset.y));
        }
    }

    fn run_frame(&mut self, cmd: crate::menus::FrameCmd, window: &mut Window, cx: &mut Context<Self>) {
        use crate::menus::FrameCmd;
        let Some((current, count)) = self.frame_position() else { return };
        let done = |this: &mut Self, label: &str| this.ed.commit(label);
        match cmd {
            FrameCmd::InsertFrame => {
                self.ed.on_timeline(|timeline, layer, frame| timeline.insert_frames(layer, frame, 1));
                done(self, t("Insert Frame"));
            }
            FrameCmd::RemoveFrame => {
                self.ed.on_timeline(|timeline, layer, frame| timeline.remove_frame(layer, frame));
                done(self, t("Remove Frames"));
            }
            FrameCmd::Keyframe | FrameCmd::BlankKeyframe => {
                let blank = cmd == FrameCmd::BlankKeyframe;
                self.ed.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, blank));
                done(self, if blank { t("Insert Blank Keyframe") } else { t("Insert Keyframe") });
            }
            FrameCmd::ClearKeyframe => {
                self.ed.on_timeline(|timeline, layer, frame| timeline.clear_keyframe(layer, frame));
                done(self, t("Clear Keyframe"));
            }
            FrameCmd::GuideLayer | FrameCmd::MaskLayer => {
                use crate::model::LayerKind;
                let wanted = if cmd == FrameCmd::GuideLayer { LayerKind::Guide } else { LayerKind::Mask };
                // Chosen again, the layer goes back to being an ordinary one.
                self.ed.on_timeline(|timeline, layer, _| timeline.set_kind(layer, if timeline.kind(layer) == wanted { LayerKind::Normal } else { wanted }));
                done(self, if cmd == FrameCmd::GuideLayer { t("Guide Layer") } else { t("Mask Layer") });
            }
            FrameCmd::BindLayer => {
                let tied = self.ed.on_timeline(|timeline, layer, _| {
                    let now = !timeline.is_bound(layer);
                    timeline.set_bound(layer, now)
                });
                if tied == Some(false) {
                    self.status = t("Only a layer right under a guide, a mask or a layer tied to one can be tied").into();
                }
                done(self, t("Tie to Guide or Mask"));
            }
            FrameCmd::AddHint | FrameCmd::RemoveHints => {
                let adding = cmd == FrameCmd::AddHint;
                // A hint starts at the first point of each of the two shapes; it is then dragged to its place on each keyframe.
                let added = self.ed.on_timeline(|timeline, layer, frame| {
                    let index = timeline.span_at(layer, frame)?;
                    let spans = &timeline.layers[layer];
                    let start = spans[index].objects.first().map(|o| o.outline()).and_then(|line| line.first().copied())?;
                    let end = spans.get(index + 1).and_then(|next| next.objects.first()).map(|o| o.outline()).and_then(|line| line.first().copied())?;
                    let tween = timeline.layers[layer][index].tween.as_mut().filter(|tween| tween.shape)?;
                    if adding {
                        if tween.hints.len() < 26 {
                            tween.hints.push((start, end));
                        }
                    } else {
                        tween.hints.clear();
                    }
                    Some(())
                });
                if added.flatten().is_none() {
                    self.status = t("Shape hints belong to a shape tween: go to its first keyframe").into();
                }
                done(self, if adding { t("Add Shape Hint") } else { t("Remove All Hints") });
            }
            FrameCmd::Tween | FrameCmd::ShapeTween | FrameCmd::RemoveTween => {
                let tween = (cmd != FrameCmd::RemoveTween).then(|| crate::model::Tween { shape: cmd == FrameCmd::ShapeTween, ..Default::default() });
                // A tween needs a keyframe to go to: without one after it, there is nothing to move toward.
                let has_next = self.ed.on_timeline(|timeline, layer, frame| {
                    let next = timeline.span_at(layer, frame).is_some_and(|index| index + 1 < timeline.layers[layer].len());
                    timeline.set_tween(layer, frame, tween.clone());
                    next
                });
                if tween.is_some() && has_next == Some(false) {
                    self.status = t("The tween will show once a keyframe follows this one: go to a later frame and insert a keyframe (F6)").into();
                }
                done(self, match cmd {
                    FrameCmd::Tween => t("Create Motion Tween"),
                    FrameCmd::ShapeTween => t("Create Shape Tween"),
                    _ => t("Remove Tween"),
                });
            }
            FrameCmd::Play => {
                if self.frames_playing {
                    self.frames_playing = false;
                } else if count < 2 && self.ed.doc.page().has_behaviors() {
                    // A scene of one frame still has its behaviors to show.
                    self.frames_playing = true;
                    self.behavior_tick = 0;
                    self.play_token += 1;
                    self.schedule_timeline(self.play_token, cx);
                } else if count < 2 {
                    self.status = t("The scene has only one frame: insert frames (F5) or a keyframe (F6) to animate").into();
                } else {
                    // From the last frame there is nowhere to play to: it starts from the first again.
                    if current + 1 >= count {
                        self.ed.goto_frame(0);
                        self.reveal_frame(0);
                    }
                    self.frames_playing = true;
                    self.behavior_tick = 0;
                    self.play_token += 1;
                    self.schedule_timeline(self.play_token, cx);
                }
            }
            FrameCmd::LoopPlayback => self.loop_playback = !self.loop_playback,
            FrameCmd::Rewind => self.show_frame(0, None),
            FrameCmd::Forward => self.show_frame((current + 1).min(count.saturating_sub(1)), None),
            FrameCmd::Back => self.show_frame(current.saturating_sub(1), None),
            FrameCmd::PublishHtml => self.publish_html(window, cx),
            FrameCmd::TestMovie => self.test_movie(),
            FrameCmd::DeleteLayer => {
                if self.ed.delete_layer() {
                    done(self, t("Delete Layer"));
                }
            }
            FrameCmd::ImportSound => self.pick_file(window, cx, |this, path| this.import_sound(&path)),
        }
    }

    fn schedule_timeline(&mut self, token: u64, cx: &mut Context<Self>) {
        let fps = self.ed.doc.page().timeline.as_ref().map_or(DEFAULT_FPS, |timeline| timeline.fps).clamp(1.0, 120.0);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs_f32(1.0 / fps)).await;
            this.update(cx, |this, cx| {
                if this.frames_playing && this.play_token == token {
                    this.behavior_tick += 1;
                    if let Some((current, count)) = this.frame_position() {
                        // At the end it stops, as Flash does, unless Loop Playback is on. Behaviors have no end:
                        // where there are any, the last frame stays on show with them at work until Stop.
                        let ended = current + 1 >= count && !this.loop_playback;
                        if ended && !this.ed.doc.page().has_behaviors() {
                            this.frames_playing = false;
                            cx.notify();
                            return;
                        }
                        if !ended && count > 1 {
                            let next = if current + 1 >= count { 0 } else { current + 1 };
                            this.ed.goto_frame(next);
                            this.reveal_frame(next);
                            for sound in this.sounds_starting(next) {
                                this.play_sound(sound);
                            }
                        }
                    }
                    this.schedule_timeline(token, cx);
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// The whole animation as it is to be published: every scene and clip with the work on its frame on show written into its timeline.
    fn movie(&mut self) -> Document {
        self.ed.sync_clip();
        let mut movie = self.ed.doc.clone();
        for page in &mut movie.pages {
            page.store_frame();
        }
        movie
    }

    /// Control > Test Movie: publishes the animation to a file of the program's own and opens it in the browser, as Flash's Test Movie opens the finished film.
    /// Help > User Guide: writes the guide out as a page and opens it in the browser. PisiAnimasyon opens it at the
    /// part about animation, where the lessons are.
    fn open_guide(&mut self) {
        // PisiAnimasyon has a book of its own, with pictures, that comes with the package: lessons from the first click on.
        // It is in Turkish; where it is not installed, or in another language, the guide built into the program is shown.
        if crate::theme::is_sahne() && crate::i18n::language() == crate::i18n::Language::Turkish {
            if let Some(book) = book_index() {
                self.status = match std::process::Command::new("xdg-open").arg(&book).spawn() {
                    Ok(_) => t("The guide is open in the browser").to_string(),
                    Err(error) => tf("Export failed: {}", &[&error]),
                };
                return;
            }
        }
        let (html, animation) = crate::guide::page(crate::i18n::language(), t("User Guide"), t("Contents"));
        let folder = dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("pisiworks");
        let path = folder.join(if crate::i18n::language() == crate::i18n::Language::Turkish { "kilavuz.html" } else { "guide.html" });
        let written = std::fs::create_dir_all(&folder).and_then(|()| std::fs::write(&path, html));
        let place = if crate::theme::is_sahne() && !animation.is_empty() { format!("file://{}#{animation}", path.display()) } else { format!("file://{}", path.display()) };
        self.status = match written.and_then(|()| std::process::Command::new("xdg-open").arg(&place).spawn().map(|_| ())) {
            Ok(()) => t("The guide is open in the browser").to_string(),
            Err(error) => tf("Export failed: {}", &[&error]),
        };
    }

    fn test_movie(&mut self) {
        // What is being transformed is taken as it looks: the film is of what is on the stage.
        self.apply_transform();
        let movie = self.movie();
        let folder = dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("pisianimasyon");
        let path = folder.join("test.html");
        let written = std::fs::create_dir_all(&folder).map_err(anyhow::Error::from).and_then(|()| crate::publish::html(&movie, &self.document_name())).and_then(|html| std::fs::write(&path, html).map_err(anyhow::Error::from));
        self.status = match written.and_then(|()| std::process::Command::new("xdg-open").arg(&path).spawn().map(|_| ()).map_err(anyhow::Error::from)) {
            Ok(()) => t("The movie is open in the browser").to_string(),
            Err(error) => tf("Export failed: {}", &[&format!("{error:#}")]),
        };
    }

    /// Timeline > Import Sound: adds a sound file to the animation and starts it on the keyframe the frame on show belongs to.
    pub fn import_sound(&mut self, path: &Path) {
        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let mime = match extension.as_str() {
            "mp3" => "audio/mpeg",
            "ogg" | "oga" => "audio/ogg",
            "wav" => "audio/wav",
            _ => {
                self.status = t("Sounds can be MP3, OGG or WAV files").into();
                return;
            }
        };
        match std::fs::read(path) {
            Ok(bytes) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let id = self.ed.add_sound(&name, mime, &bytes);
                self.set_frame_sound(Some(id));
                self.status = tf("Imported {}", &[&name]);
            }
            Err(error) => self.status = tf("Could not import file: {}", &[&error]),
        }
    }

    /// The keyframe the frame on show belongs to, on the active layer.
    pub fn current_span(&self) -> Option<&crate::model::Span> {
        let page = self.ed.doc.page();
        let timeline = page.timeline.as_ref()?;
        let span = timeline.span_at(page.active_layer, timeline.current)?;
        timeline.layers[page.active_layer].get(span)
    }

    /// Sets what the keyframe the frame on show belongs to does when the animation reaches it.
    pub fn set_frame_action(&mut self, action: Option<crate::model::Action>) {
        let text = action.map_or_else(String::new, |action| action.text());
        self.ed.on_timeline(|timeline, layer, frame| {
            if let Some(at) = timeline.span_at(layer, frame) {
                timeline.layers[layer][at].action = text;
            }
        });
        self.ed.commit(t("Frame Action"));
    }

    /// Sets the sound that starts on the keyframe the frame on show belongs to.
    pub fn set_frame_sound(&mut self, sound: Option<u64>) {
        self.ed.on_timeline(|timeline, layer, frame| {
            if let Some(at) = timeline.span_at(layer, frame) {
                timeline.layers[layer][at].sound = sound;
            }
        });
        self.ed.commit(t("Frame Sound"));
    }

    /// The action of the selected button, if one copy of a button symbol is selected.
    pub fn button_action(&self) -> Option<Option<crate::model::Action>> {
        let selected = self.ed.selected();
        let [object] = selected.as_slice() else { return None };
        let Shape::Instance { symbol, .. } = object.shape else { return None };
        self.ed.doc.symbols.iter().any(|s| s.id == symbol && s.kind == SymbolKind::Button).then(|| crate::model::Action::parse(&object.link))
    }

    pub fn set_button_action(&mut self, action: Option<crate::model::Action>) {
        let text = action.map_or_else(String::new, |action| action.text());
        self.ed.for_selected(|o| o.link.clone_from(&text));
        self.ed.commit(t("Button Action"));
    }

    /// Plays a sound of the animation while the timeline plays in the program, with whichever player the system has.
    fn play_sound(&self, id: u64) {
        use base64::Engine;
        let Some(sound) = self.ed.doc.sounds.iter().find(|s| s.id == id) else { return };
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(sound.data.as_bytes()) else { return };
        let extension = match sound.mime.as_str() {
            "audio/mpeg" => "mp3",
            "audio/ogg" => "ogg",
            _ => "wav",
        };
        let folder = dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("pisianimasyon");
        let file = folder.join(format!("ses-{id}.{extension}"));
        if std::fs::create_dir_all(&folder).is_err() || (!file.exists() && std::fs::write(&file, bytes).is_err()) {
            return;
        }
        let players: [(&str, &[&str]); 4] = [("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]), ("mpv", &["--no-video", "--really-quiet"]), ("paplay", &[]), ("aplay", &["-q"])];
        for (program, arguments) in players {
            let started = std::process::Command::new(program).args(arguments).arg(&file).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
            if started.is_ok() {
                break;
            }
        }
    }

    /// The sounds that start on a frame: those of the keyframes that begin there.
    fn sounds_starting(&self, frame: usize) -> Vec<u64> {
        self.ed.doc.page().timeline.as_ref().map_or_else(Vec::new, |timeline| timeline.layers.iter().filter_map(|spans| spans.iter().find(|span| span.start == frame)?.sound).collect())
    }

    /// Hands the document to the other of the two programs: it is written to a file of the
    /// program's own and the other program is started on it. Nothing here changes.
    fn open_in_other(&mut self) {
        self.ed.sync_clip();
        let (other, extension) = if is_sahne() { ("pisiworks", FILE_EXTENSION) } else { ("pisianimasyon", SAHNE_EXTENSION) };
        let document = self.ed.for_other_program(is_sahne());
        let folder = dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("pisiworks").join("aktarim");
        let stem = self.path.as_ref().or(self.source_image.as_ref()).map_or_else(|| t("Untitled").to_string(), |p| file_stem(p));
        let path = folder.join(format!("{}.{extension}", crate::templates::file_name(&stem)));
        // The other program is looked for beside this one first, then wherever the system finds it.
        let beside = std::env::current_exe().ok().map(|exe| exe.with_file_name(other)).filter(|exe| exe.exists());
        let program = beside.unwrap_or_else(|| PathBuf::from(other));
        let started = std::fs::create_dir_all(&folder)
            .map_err(anyhow::Error::from)
            .and_then(|()| document_to_json(&document))
            .and_then(|json| std::fs::write(&path, json).map_err(anyhow::Error::from))
            .and_then(|()| std::process::Command::new(&program).arg(&path).spawn().map(|_| ()).map_err(anyhow::Error::from));
        self.status = match started {
            Ok(()) => tf("Opened in {} as a copy: {}", &[&if is_sahne() { "PisiWorks" } else { "PisiAnimasyon" }, &path.display()]),
            Err(error) => tf("Could not open the other program: {}", &[&format!("{error:#}")]),
        };
    }

    /// Control > Publish as HTML5: asks for a file and writes the scene as a page that plays in a browser.
    fn publish_html(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.ed.leave_symbol_for_good();
        self.ed.end_mask_edit();
        self.ed.close_groups();
        let movie = self.movie();
        let stem = self.path.as_ref().map_or_else(|| t("Untitled").to_string(), |p| file_stem(p));
        let picked = cx.prompt_for_new_path(&Self::default_dir(), Some(&format!("{stem}.html")));
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut path))) = picked.await else { return };
            if path.extension().is_none() {
                path.set_extension("html");
            }
            this.update(cx, |this, cx| {
                let written = crate::publish::html(&movie, &stem).and_then(|html| std::fs::write(&path, html).map_err(anyhow::Error::from));
                this.status = match written {
                    Ok(()) => tf("Exported {}", &[&path.display()]),
                    Err(error) => tf("Export failed: {}", &[&format!("{error:#}")]),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn toggle_play(&mut self, cx: &mut Context<Self>) {
        if self.playing {
            self.playing = false;
            return;
        }
        if self.ed.doc.page().states.iter().filter(|s| s.exported).count() < 2 {
            self.status = t("Add a second state to play an animation").into();
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
        self.ed.commit(t("Apply Style"));
    }

    fn run_path_op(&mut self, op: PathOp) {
        match self.ed.path_op(op, self.path_amount) {
            Ok(_) => self.ed.commit(op.label()),
            Err(message) => self.status = message.to_string(),
        }
    }

    /// Picks up a panel item. It is placed when the mouse is let go: where it is
    /// dropped on the canvas, or in the middle of the page after a plain click.
    pub fn begin_panel_drag(&mut self, item: PanelItem, at: Point<Pixels>) {
        self.panel_drag = Some((item, at));
    }

    /// The size the item will have on the canvas, for the outline shown while it is dragged.
    pub fn panel_item_size(&self, item: PanelItem) -> (f32, f32) {
        match item {
            PanelItem::AutoShape(_) => (AUTO_SHAPE_SIZE, AUTO_SHAPE_SIZE),
            PanelItem::Common(index) => common_items()
                .get(index)
                .and_then(|entry| entry.objects().iter().map(|o| o.bounds).reduce(Rect::union))
                .map_or((0.0, 0.0), |b| (b.w, b.h)),
            PanelItem::Symbol(id) => self.ed.symbol_size(id).unwrap_or((0.0, 0.0)),
        }
    }

    /// Whether the pointer is over the canvas with a panel item in hand, and where in the document.
    pub fn panel_drop_point(&self, at: Point<Pixels>) -> Option<Vec2> {
        let (_, pressed) = self.panel_drag?;
        let moved = f32::from(at.x - pressed.x).abs().max(f32::from(at.y - pressed.y).abs()) > CLICK_SLOP;
        (moved && self.canvas_bounds.get().contains(&at)).then(|| snap(self.view_map().to_doc(at)))
    }

    /// Lets go of a panel item.
    fn finish_panel_drag(&mut self, at: Point<Pixels>) {
        let drop = self.panel_drop_point(at);
        let Some((item, pressed)) = self.panel_drag.take() else { return };
        let clicked = f32::from(at.x - pressed.x).abs().max(f32::from(at.y - pressed.y).abs()) <= CLICK_SLOP;
        let center = match (drop, clicked, item) {
            (Some(point), _, _) => point,
            // A click on a symbol of the Document Library only picks it; Insert places it.
            (None, true, PanelItem::AutoShape(_) | PanelItem::Common(_)) => self.ed.doc.page().rect().center(),
            _ => return,
        };
        match item {
            PanelItem::AutoShape(kind) => self.insert_auto_shape_at(kind, center),
            PanelItem::Common(index) => {
                let Some(entry) = common_items().get(index) else { return };
                // The buttons of the Common Library arrive as button symbols.
                let kind = if entry.category == "Buttons" { SymbolKind::Button } else { SymbolKind::Graphic };
                let id = self.ed.add_library_symbol(t(entry.name), &entry.objects(), kind);
                self.symbol_selection = Some(id);
                self.place_symbol_at(id, center);
            }
            PanelItem::Symbol(id) => self.place_symbol_at(id, center),
        }
    }

    fn insert_auto_shape_at(&mut self, kind: AutoKind, center: Vec2) {
        let d = kind.defaults();
        let bounds = Rect {
            x: (center.x - AUTO_SHAPE_SIZE / 2.0).round(),
            y: (center.y - AUTO_SHAPE_SIZE / 2.0).round(),
            w: AUTO_SHAPE_SIZE,
            h: AUTO_SHAPE_SIZE,
        };
        let object = self.new_object(kind.label(), Shape::Auto { kind, a: d.a, b: d.b, count: d.count }, bounds, Vec::new());
        self.add_new(object, t("Auto Shape"));
    }

    pub fn place_symbol(&mut self, id: u64) {
        self.place_symbol_at(id, self.ed.doc.page().rect().center());
    }

    fn place_symbol_at(&mut self, id: u64, center: Vec2) {
        if self.ed.place_symbol_at(id, center) {
            self.ed.commit(t("Insert Symbol"));
        } else {
            self.status = t("All layers are locked").into();
        }
    }

    /// Makes an empty symbol and opens it, so that what is drawn next becomes the symbol.
    fn new_symbol(&mut self, name: &str, kind: SymbolKind) {
        self.ed.leave_symbol();
        let id = self.ed.new_symbol(name, kind);
        self.symbol_selection = Some(id);
        // A new movie clip opens with its own empty timeline; its first copy is placed when the edit ends.
        if is_sahne() && kind == SymbolKind::Animation {
            self.ed.open_clip(id);
            self.ed.commit(t("New Symbol"));
            self.status = t("Draw the movie clip on its own timeline, then go back to the scene").into();
            return;
        }
        self.ed.edit_symbol(id, None, true);
        self.ed.commit(if kind == SymbolKind::Button { t("New Button") } else { t("New Symbol") });
        self.set_view_mode(ViewMode::Original);
        self.status = t("Draw the symbol, then double-click an empty spot to finish").into();
    }

    /// Edit Symbol: opens the selected copy where it lies, or ends the edit that is going on.
    pub fn toggle_symbol_edit(&mut self) {
        self.frames_playing = false;
        // What is being transformed is taken as it looks before the symbol is entered or left.
        self.apply_transform();
        // In the animation program a movie clip is edited on a page of its own, with its own timeline.
        if let Some(symbol) = self.ed.editing_clip() {
            self.ed.leave_clip();
            // A clip made from nothing gets its first copy now, in the middle of the scene.
            let placed = self.ed.doc.pages.iter().any(|page| page.objects().any(|o| matches!(o.shape, Shape::Instance { symbol: of, .. } if of == symbol)));
            if !placed {
                let middle = self.ed.doc.page().rect().center();
                self.ed.place_symbol_at(symbol, middle);
            }
            self.ed.commit(t("Edit Symbol"));
            return;
        }
        let clip = self.ed.selected().iter().find_map(|o| match o.shape {
            Shape::Instance { symbol, .. } if is_sahne() && self.ed.doc.symbols.iter().any(|s| s.id == symbol && s.kind == SymbolKind::Animation) => Some(symbol),
            _ => None,
        });
        if let Some(symbol) = clip.filter(|_| self.ed.editing_symbol().is_none()) {
            if self.ed.open_clip(symbol) {
                self.ed.commit(t("Edit Symbol"));
                self.status = t("Editing the movie clip: the timeline is its own. The button above the stage goes back to the scene").into();
            }
            return;
        }
        if self.ed.leave_symbol() || self.ed.edit_selected_symbol() {
            self.ed.commit(t("Edit Symbol"));
            if self.ed.editing_symbol().is_some() {
                self.status = t("Editing the symbol. To finish: double-click an empty place, press Esc, or use the button above the stage").into();
            }
        }
    }

    /// The path of the animation the selection belongs to: the middle of its symbol in each state.
    pub fn motion_path(&self) -> Vec<Vec2> {
        if self.view_mode != ViewMode::Original {
            return Vec::new();
        }
        self.ed.selected_animation().map_or_else(Vec::new, |index| self.ed.animation_path(index))
    }

    /// Asks for files to add to the batch.
    pub fn add_batch_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: true, prompt: None });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else { return };
            this.update(cx, |this, cx| {
                if let Some(dialog) = this.batch_dialog.as_mut() {
                    for path in paths {
                        if !dialog.files.contains(&path) {
                            dialog.files.push(path);
                        }
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Asks for the folder the results of the batch go to.
    pub fn pick_batch_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions { files: false, directories: true, multiple: false, prompt: None });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else { return };
            this.update(cx, |this, cx| {
                if let Some(dialog) = this.batch_dialog.as_mut() {
                    dialog.output = paths.pop();
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Runs the batch over its files and says how it went.
    pub fn run_batch(&mut self) {
        self.commit_field();
        let Some(dialog) = self.batch_dialog.take() else { return };
        let scale = match dialog.scale {
            1 => crate::batch::Scale::ToSize(dialog.width, dialog.height),
            2 => crate::batch::Scale::FitArea(dialog.width, dialog.height),
            3 => crate::batch::Scale::Percent(dialog.percent),
            _ => crate::batch::Scale::None,
        };
        let batch = crate::batch::Batch {
            scale,
            settings: dialog.use_current.then(|| self.ed.doc.page().export),
            prefix: crate::templates::name_part(&dialog.prefix),
            suffix: crate::templates::name_part(&dialog.suffix),
            output: dialog.output.clone(),
        };
        let open = |bytes: &[u8], path: &Path| match parse_file(bytes)? {
            Opened::Document(document) => Ok((document, true)),
            Opened::Fireworks(document, _) | Opened::Animation(document) => Ok((document, false)),
            Opened::Image(bitmap) | Opened::Flat(bitmap, _) => Ok((Document::from_bitmap(file_stem(path), bitmap), false)),
        };
        let mut failed = Vec::new();
        for path in &dialog.files {
            if let Err(error) = crate::batch::process(path, &batch, open) {
                failed.push(format!("{}: {error}", path.file_name().unwrap_or_default().to_string_lossy()));
            }
        }
        let done = dialog.files.len() - failed.len();
        self.status = match failed.first() {
            None => tf("Batch finished: {} file(s) processed", &[&done]),
            Some(first) => tf("Batch finished: {} processed, {} failed ({})", &[&done, &failed.len(), first]),
        };
    }

    /// Starts a new, unnamed document from the template picked in New from Template.
    pub fn close_template_dialog(&mut self, keep: bool) {
        let Some((saved, picked)) = self.template_dialog.take() else { return };
        if !keep {
            return;
        }
        let built_in = crate::templates::BUILT_IN;
        if let Some(template) = built_in.get(picked) {
            self.load_document(template.document(), None);
            self.fit_if_larger_than_view();
            self.status = tf("New document from the template {}", &[&t(template.name)]);
        } else if let Some((_, path)) = saved.get(picked - built_in.len()) {
            self.open_as_template(path.clone());
        }
    }

    /// Opens a document as the start of a new one: it has no file of its own, so saving asks for a name.
    pub fn open_as_template(&mut self, path: PathBuf) {
        self.template_dialog = None;
        self.open_file(path.clone());
        if self.path.as_deref() == Some(path.as_path()) {
            self.path = None;
            self.status = tf("New document from the template {}", &[&file_stem(&path)]);
        }
    }

    /// Closes Save as Template; kept, the document is written into the templates folder under the name.
    pub fn close_template_name(&mut self, keep: bool) {
        self.commit_field();
        let Some(name) = self.template_name.take() else { return };
        if !keep {
            return;
        }
        self.ed.leave_symbol_for_good();
        self.ed.end_mask_edit();
        self.ed.close_groups();
        self.status = match document_to_json(&self.ed.doc).and_then(|json| crate::templates::save(&name, &json)) {
            Ok(path) => tf("Template saved as {}", &[&path.display()]),
            Err(error) => tf("Could not save: {}", &[&error]),
        };
    }

    /// Closes New Symbol; kept, the symbol is made and opened for drawing.
    pub fn close_symbol_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some((name, kind)) = self.symbol_dialog.take() else { return };
        if keep {
            self.new_symbol(&name, kind);
        }
    }

    /// Closes Animate; kept, the selection is animated with the settings, or its animation changed to them.
    pub fn close_animate_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some(settings) = self.animate_dialog.take() else { return };
        if keep && self.ed.animate_selection(settings) {
            self.ed.commit(t("Animate Selection"));
        }
    }

    fn make_symbol(&mut self) {
        match self.ed.make_symbol() {
            Some(id) => {
                self.symbol_selection = Some(id);
                self.pixel_sel = None;
                self.ed.commit(t("Convert to Symbol"));
            }
            None => self.status = t("Select objects to make a symbol from").into(),
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
        self.ed.commit(t("Hotspot Link"));
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
            self.status = t("Type what to find first").into();
            return;
        }
        let colors = match self.find_mode {
            FindMode::Text => None,
            FindMode::Color => match (Color::parse_hex(&find), Color::parse_hex(&change).or(if replace { None } else { Some(Color::BLACK) })) {
                (Some(from), Some(to)) => Some((from, to)),
                _ => {
                    self.status = t("Enter colors as hex, such as #FF0000").into();
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
            self.status = tf("Found {} on this page", &[&count]);
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
                } else if let Shape::Text { content, .. } = &mut object.shape {
                    *content = content.replace(&find, &change);
                    object.refit_text();
                }
            }
        }
        self.ed.commit(t("Replace"));
        self.status = tf("Replaced {}", &[&count]);
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
            if let Shape::Text { content, .. } = &mut o.shape {
                content.push(ch);
                o.refit_text();
                changed = true;
            }
        });
        if changed {
            self.ed.commit(t("Special Character"));
        } else {
            self.status = t("Select a text object, or start typing with the Text tool").into();
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
            Cmd::ConvertToSymbol | Cmd::Animate => has_selection,
            Cmd::Behavior(crate::menus::BehaviorCmd::Toggle(_) | crate::menus::BehaviorCmd::Attach) => has_selection,
            Cmd::Behavior(_) => self.ed.selected().iter().any(|o| !o.behaviors.is_empty()),
            Cmd::TextAlignDown(_) => self.shown_text_style().orientation.is_vertical(),
            Cmd::SaveSteps => !self.steps_to_save().is_empty(),
            Cmd::Ready(ready) => match ready {
                crate::menus::ReadyCmd::Grayscale | crate::menus::ReadyCmd::Sepia | crate::menus::ReadyCmd::AddShadow | crate::menus::ReadyCmd::AutoVectorMask | crate::menus::ReadyCmd::TwistAndFade => has_selection,
                crate::menus::ReadyCmd::AddArrowheads => self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Line | Shape::Path { closed: false })),
                crate::menus::ReadyCmd::AddPictureFrame => true,
                crate::menus::ReadyCmd::DistributeToLayers => self.ed.selection.len() >= 2,
                crate::menus::ReadyCmd::HideOtherLayers | crate::menus::ReadyCmd::LockOtherLayers => self.ed.doc.page().layers.len() >= 2,
                crate::menus::ReadyCmd::ReverseStates | crate::menus::ReadyCmd::ReverseStateRange => self.ed.doc.page().states.len() >= 2,
            },
            Cmd::Frame(crate::menus::FrameCmd::DeleteLayer) => self.ed.doc.page().layers.len() > 1,
            // A right-click menu greys what does not fit the frame it was opened on.
            Cmd::Frame(crate::menus::FrameCmd::RemoveTween) => self.current_tween().is_some(),
            Cmd::Frame(crate::menus::FrameCmd::ClearKeyframe) => self.frame_position().is_some_and(|(frame, _)| frame > 0) && self.current_span().is_some_and(|span| self.frame_position().is_some_and(|(frame, _)| span.start == frame)),
            Cmd::Frame(crate::menus::FrameCmd::AddHint | crate::menus::FrameCmd::RemoveHints) => self.current_tween().is_some_and(|tween| tween.shape),
            Cmd::EditSymbol => self.ed.can_edit_symbol() || self.ed.editing_symbol().is_some() || self.ed.editing_clip().is_some(),
            Cmd::TweenInstances => self.ed.can_tween(),
            Cmd::AnimateSettings | Cmd::RemoveAnimation => self.ed.selected_animation().is_some(),
            Cmd::BreakApart | Cmd::Ungroup => self.ed.selected().iter().any(|o| o.is_instance()),
            Cmd::Group => self.ed.selection.len() >= 2,
            Cmd::Rotate { .. } => has_selection,
            Cmd::Rotate180 | Cmd::NumericTransform => has_selection,
            Cmd::MaskAll(_) => has_selection,
            Cmd::MaskSelection(_) => self.active_pixels().is_some(),
            Cmd::PasteAsMask | Cmd::PasteInside => has_selection && self.ed.has_clipboard(),
            Cmd::GroupAsMask => self.ed.selection.len() >= 2,
            Cmd::MaskToggle | Cmd::MaskDelete => self.ed.selected().iter().any(|o| o.mask.is_some()),
            Cmd::JpegMaskSave => self.active_pixels().is_some(),
            Cmd::JpegMaskRestore => self.ed.doc.page().jpeg_mask.is_some() && self.ed.selected().iter().any(|o| o.is_image()),
            Cmd::JpegMaskRemove => self.ed.doc.page().jpeg_mask.is_some(),
            Cmd::RemoveTransform => self.ed.selected().iter().any(|o| o.placement.is_some()),
            Cmd::TextToPaths => self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Text { .. })),
            Cmd::FirstState | Cmd::PreviousState | Cmd::NextState | Cmd::LastState => self.ed.doc.page().states.len() > 1,
            Cmd::PathOp(op) => self.ed.selection.len() >= if op.needs_two() { 2 } else { 1 },
            Cmd::Guides(GuideCmd::Clear) => !self.ed.doc.page().guides.is_empty(),
            Cmd::AttachToPath => {
                let selected = self.ed.selected();
                selected.iter().any(|o| matches!(o.shape, Shape::Text { .. })) && selected.iter().any(|o| o.reshapable() && !matches!(o.shape, Shape::Text { .. }))
            }
            Cmd::DetachFromPath | Cmd::ReversePathText | Cmd::PathOrientation(_) => self.ed.has_text_on_path(),
            Cmd::Subselect => self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Group { .. })),
            Cmd::Superselect => self.ed.can_superselect(),
            Cmd::Revert => (self.path.is_some() || self.source_image.is_some()) && self.ed.is_dirty(),
            Cmd::Clone | Cmd::FitSelection => has_selection,
            Cmd::PasteAttributes => has_selection && self.ed.has_clipboard(),
            Cmd::InsertWeb(_) | Cmd::FlattenSelection => self.ed.selected().iter().any(|o| !o.is_web()),
            Cmd::SelectSimilar | Cmd::Marquee(_) => self.active_pixels().is_some(),
            Cmd::MergeDown => self.ed.can_merge_down(),
            Cmd::RepeatFilter => self.last_filter.is_some() && self.ed.has_selected_bitmap(),
            _ => true,
        }
    }

    /// Runs a command and, if it is one of the menus and changed the document, notes it as a step that can be saved and played again.
    pub fn run(&mut self, cmd: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        let before = (self.ed.history_pos, self.ed.history.len());
        self.run_command(cmd, window, cx);
        let after = (self.ed.history_pos, self.ed.history.len());
        if after != before && after.0 > 0 && !matches!(cmd, Cmd::Undo | Cmd::Redo | Cmd::RunSaved(_) | Cmd::RunScript) {
            if let Some(step) = crate::commands::step_of(cmd) {
                self.recorded.push((after.0, self.ed.history[after.0].0.clone(), step));
            }
        }
    }

    /// The recorded steps that are still part of the document's history, oldest first.
    pub fn steps_to_save(&self) -> Vec<String> {
        self.recorded
            .iter()
            .filter(|(at, label, _)| *at <= self.ed.history_pos && self.ed.history.get(*at).is_some_and(|(name, _)| name == label))
            .map(|(_, _, step)| step.clone())
            .collect()
    }

    /// Plays the steps of a command one after another. Steps that are not menu commands, or cannot run now, are passed over.
    pub fn play_steps(&mut self, steps: &[String], window: &mut Window, cx: &mut Context<Self>) {
        let mut done = 0;
        for step in steps {
            let Some(cmd) = crate::commands::command_of(step).filter(|cmd| !matches!(cmd, Cmd::RunScript | Cmd::ManageCommands | Cmd::Exit | Cmd::New | Cmd::Open | Cmd::NewFromTemplate)) else { continue };
            if self.is_enabled(cmd) {
                self.run(cmd, window, cx);
                done += 1;
            }
        }
        self.status = tf("Command played: {} of {} step(s)", &[&done, &steps.len()]);
    }

    fn play_file(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        match crate::commands::read(path) {
            Ok(steps) => self.play_steps(&steps, window, cx),
            Err(error) => self.status = error.to_string(),
        }
    }

    /// Asks for a command file and plays it.
    fn run_script(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: None });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else { return };
            let Some(path) = paths.pop() else { return };
            this.update_in(cx, |this, window, cx| {
                this.play_file(&path, window, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Closes the box that asks for a command's name; kept, the steps are saved under it, or the command renamed to it.
    pub fn close_command_name(&mut self, keep: bool) {
        self.commit_field();
        let Some((name, renaming)) = self.command_name.take() else { return };
        if !keep {
            return;
        }
        let result = match renaming.and_then(|index| self.saved_commands.get(index)) {
            Some((_, path)) => crate::commands::rename(path, &name),
            None => crate::commands::save(&name, &self.steps_to_save()),
        };
        match result {
            Ok(path) => self.status = tf("Command saved as {}", &[&path.display()]),
            Err(error) => self.status = error.to_string(),
        }
        self.saved_commands = crate::commands::saved();
    }

    /// Deletes the saved command picked in Manage Saved Commands.
    pub fn delete_saved_command(&mut self) {
        let Some(Some(picked)) = self.commands_dialog else { return };
        if let Some((_, path)) = self.saved_commands.get(picked) {
            if let Err(error) = std::fs::remove_file(path) {
                self.status = error.to_string();
            }
        }
        self.saved_commands = crate::commands::saved();
        self.commands_dialog = Some(None);
    }

    /// The Commands menu, with the saved commands at its foot.
    pub fn commands_menu(&self, fixed: &'static [crate::menus::Entry]) -> &'static [crate::menus::Entry] {
        let names: Vec<String> = self.saved_commands.iter().map(|(name, _)| name.clone()).collect();
        crate::commands::menu(fixed, &names)
    }

    fn run_ready(&mut self, ready: crate::menus::ReadyCmd) {
        use crate::menus::ReadyCmd;
        match ready {
            // Fireworks does both with a Hue/Saturation live filter, which stays editable.
            ReadyCmd::Grayscale | ReadyCmd::Sepia => {
                let sepia = ready == ReadyCmd::Sepia;
                self.change_filters(|filters| {
                    let mut filter = LiveFilter::new(FilterKind::HueSaturation);
                    filter.values = if sepia { vec![30.0, -50.0, 0.0, 1.0] } else { vec![0.0, -100.0, 0.0, 0.0] };
                    filters.push(filter);
                });
            }
            ReadyCmd::DistributeToLayers => {
                if self.ed.distribute_to_layers() > 0 {
                    self.ed.commit(t("Distribute to Layers"));
                }
            }
            ReadyCmd::HideOtherLayers | ReadyCmd::LockOtherLayers => {
                let hide = ready == ReadyCmd::HideOtherLayers;
                if self.ed.set_other_layers(hide) {
                    self.ed.commit(if hide { t("Hide Other Layers") } else { t("Lock Other Layers") });
                }
            }
            ReadyCmd::ReverseStates => {
                let last = self.ed.doc.page().states.len() - 1;
                if self.ed.reverse_states(0, last) {
                    self.ed.commit(t("Reverse All States"));
                }
            }
            ReadyCmd::AddArrowheads => self.arrow_dialog = Some((false, true, 12.0, 0)),
            ReadyCmd::AddPictureFrame => self.frame_dialog = Some(10.0),
            ReadyCmd::AutoVectorMask => self.mask_dialog = Some(0),
            ReadyCmd::TwistAndFade => self.twist_dialog = Some(Default::default()),
            ReadyCmd::AddShadow => {
                if self.ed.add_shadows() > 0 {
                    self.ed.commit(t("Add Shadow"));
                }
            }
            ReadyCmd::ReverseStateRange => self.reverse_dialog = Some((1, self.ed.doc.page().states.len() as u32)),
        }
    }

    /// Closes Add Arrowheads; kept, each selected open path gets a head at the ends asked for, in the color of its stroke.
    pub fn close_arrow_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some((at_start, at_end, size, style)) = self.arrow_dialog.take() else { return };
        if !keep || !(at_start || at_end) {
            return;
        }
        let paths: Vec<(Vec<Vec2>, Stroke)> = self
            .ed
            .selected()
            .into_iter()
            .filter(|o| matches!(o.shape, Shape::Line | Shape::Path { closed: false }) && o.points.len() >= 2)
            .map(|o| (o.points.clone(), o.stroke.unwrap_or(Stroke { color: Color::BLACK, width: 1.0 })))
            .collect();
        let before = self.ed.selection.clone();
        let mut heads = Vec::new();
        for (points, stroke) in paths {
            let last = points.len() - 1;
            let ends = [(at_start, points[0], points[1]), (at_end, points[last], points[last - 1])];
            for (_, tip, from) in ends.into_iter().filter(|(wanted, _, _)| *wanted) {
                let (closed, corners) = arrow_head(style, tip, from, size);
                let mut head = self.new_object(t("Arrowhead"), Shape::Path { closed }, Rect::of_points(&corners), corners);
                // A head of two lines is drawn as the path is; the others are filled with its color.
                (head.fill, head.stroke) = if closed { (Some(stroke.color), None) } else { (None, Some(stroke)) };
                heads.push(head);
            }
        }
        if heads.is_empty() {
            self.status = t("Select an open path to add arrowheads to").into();
            return;
        }
        for head in heads {
            self.ed.add_object(head);
        }
        self.ed.selection = before;
        self.ed.commit(t("Add Arrowheads"));
    }

    /// Asks for picture files to add to Make a GIF.
    pub fn add_gif_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: true, prompt: None });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else { return };
            // The pictures follow one another by name, as numbered files should.
            paths.sort();
            this.update(cx, |this, cx| {
                if let Some(dialog) = this.gif_dialog.as_mut() {
                    dialog.files.extend(paths);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The animation Make a GIF has been asked for, as a document with a state for each picture.
    fn gif_document(&mut self) -> Option<Document> {
        self.commit_field();
        let dialog = self.gif_dialog.clone()?;
        match animation_from_files(&dialog.files, dialog.delay_ms, dialog.loops) {
            Ok(document) => Some(document),
            Err(error) => {
                self.status = error;
                None
            }
        }
    }

    /// Make a GIF, opened as a document to go on working with.
    pub fn open_gif_document(&mut self) {
        if let Some(document) = self.gif_document() {
            let states = document.pages[0].states.len();
            self.gif_dialog = None;
            self.load_document(document, None);
            self.fit_if_larger_than_view();
            self.status = tf("New animation of {} states. Export saves it as an animated GIF.", &[&states]);
        }
    }

    /// Make a GIF, saved straight to a file.
    pub fn save_gif(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(document) = self.gif_document() else { return };
        let page = Editor::new(document).export_page();
        let picked = cx.prompt_for_new_path(&Self::default_dir(), Some("animasyon.gif"));
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut path))) = picked.await else { return };
            if path.extension().is_none() {
                path.set_extension("gif");
            }
            this.update(cx, |this, cx| {
                let written = crate::optimize::encode(&page, &page.export).and_then(|bytes| std::fs::write(&path, bytes).map_err(anyhow::Error::from));
                this.status = match written {
                    Ok(()) => {
                        this.gif_dialog = None;
                        tf("Exported {}", &[&path.display()])
                    }
                    Err(error) => tf("Export failed: {}", &[&format!("{error:#}")]),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Closes Tween Instances; kept, the copies in between are made.
    pub fn close_tween_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some((steps, distribute)) = self.tween_dialog.take() else { return };
        if keep && self.ed.tween_instances(steps, distribute) > 0 {
            self.ed.commit(t("Tween Instances"));
        }
    }

    /// Closes Auto Vector Mask; kept, the selection gets the mask picked.
    pub fn close_mask_dialog(&mut self, keep: bool) {
        let Some(which) = self.mask_dialog.take() else { return };
        if keep && self.ed.auto_vector_mask(which) {
            self.ed.commit(t("Auto Vector Mask"));
        }
    }

    /// Rules > Rules as Text: the rules and the numbers written out, to be read and written by hand.
    pub fn open_script(&mut self) {
        self.commit_field();
        self.rules_dialog = false;
        self.script = Some(crate::script_edit::ScriptEdit::new(&crate::script::write(&self.ed.doc.variables, &self.ed.doc.commands, &self.ed.doc.rules)));
    }

    /// What is offered for the line being written in Rules as Text: sentences and names. The names are those of
    /// the document's objects and sounds, and of the numbers and commands the text itself sets out.
    pub fn script_offers(&self) -> Vec<crate::script_edit::Offer> {
        let Some(script) = self.script.as_ref().filter(|_| !self.script_quiet) else { return Vec::new() };
        let (numbers, commands) = crate::script::declared(&script.text());
        let names = crate::script_edit::Names { objects: self.ed.doc.chosen_names(), numbers, commands, sounds: self.ed.doc.sounds.iter().map(|s| s.name.clone()).collect() };
        script.offers(&names)
    }

    /// What in a text that reads will still not do what its writer hopes, each as a sentence to show: a name
    /// that nothing answers to, with the name that was most likely meant; a jump asked of what cannot fall.
    pub fn script_advice(&self, read: &crate::script::Read) -> Vec<(String, Option<ScriptFix>)> {
        use crate::behavior::BehaviorKind;
        use crate::rules::StepKind;
        let doc = &self.ed.doc;
        let (objects, sounds) = (doc.object_names(), doc.sounds.iter().map(|s| s.name.clone()).collect::<Vec<_>>());
        let known: Vec<String> = objects.iter().chain(&sounds).cloned().collect();
        // Where there is a way to put a thing right at once, the advice carries it: a button beside it does it.
        let mut advice: Vec<(String, Option<ScriptFix>)> = crate::script::strangers(&read.rules, &read.commands, &objects, &sounds)
            .into_iter()
            .map(|name| match crate::script::meant(&name, &known) {
                Some(near) => (tf("Nothing in the document is called {}. Did you mean {}?", &[&name, near]), Some(ScriptFix::Rename(name.clone(), near.clone()))),
                None if objects.is_empty() => (tf("Nothing in the document is called {}: give an object that name in Properties", &[&name]), None),
                None => (tf("Nothing in the document is called {}", &[&name]), None),
            })
            .collect();
        // A jump needs something to pull the object down again.
        let lists = read.rules.iter().flat_map(|rule| [&rule.steps, &rule.otherwise]).chain(read.commands.iter().map(|command| &command.steps));
        let jumps: Vec<&crate::rules::Step> = lists.flat_map(|steps| crate::rules::every_step(steps)).filter(|step| step.kind == StepKind::Jump).collect();
        // A jump of a few pixels cannot be seen: the number is a height, and a figure's height is tens of pixels.
        for step in jumps.iter().filter(|step| step.ex.is_empty() && step.x < 15.0) {
            advice.push((tf("A jump of {} pixels is too small to see: try 100", &[&crate::rules::Step::first(step)]), None));
        }
        let mut jumpers: Vec<&str> = jumps.iter().map(|step| step.a.as_str()).collect();
        jumpers.sort();
        jumpers.dedup();
        for name in jumpers {
            let falls = |o: &crate::model::Object| o.behaviors.iter().any(|b| b.kind == BehaviorKind::Gravity);
            let mut named = doc.pages.iter().flat_map(|page| page.objects()).filter(|o| o.name == name).peekable();
            if named.peek().is_some() && !named.any(falls) {
                advice.push((tf("{} cannot jump yet: it needs the behavior Fall and Jump", &[&name]), Some(ScriptFix::Falls(name.to_string()))));
            }
        }
        advice
    }

    /// Puts right what a piece of advice in Rules as Text was about.
    pub fn apply_fix(&mut self, fix: ScriptFix) {
        match fix {
            ScriptFix::Falls(name) => {
                // Every object of that name, on every scene and every keyframe, is given Fall and Jump.
                use crate::behavior::{Behavior, BehaviorKind};
                let give = |object: &mut crate::model::Object| {
                    if object.name == name && !object.behaviors.iter().any(|b| b.kind == BehaviorKind::Gravity) {
                        object.behaviors.push(Behavior::new(BehaviorKind::Gravity));
                    }
                };
                for page in &mut self.ed.doc.pages {
                    page.edit_timeline(|timeline| timeline.layers.iter_mut().flatten().flat_map(|span| span.objects.iter_mut()).for_each(&give));
                    page.layers.iter_mut().flat_map(|layer| layer.objects.iter_mut()).for_each(&give);
                }
                self.ed.commit(t("Fall and Jump"));
                self.status = tf("{} was given the behavior Fall and Jump", &[&name]);
            }
            ScriptFix::Rename(from, to) => {
                if let Some(script) = self.script.as_mut() {
                    script.replace_name(&from, &to);
                }
            }
        }
    }

    /// Takes one of the offers into the line.
    pub fn accept_offer(&mut self, index: usize) {
        let offers = self.script_offers();
        if let (Some(offer), Some(script)) = (offers.get(index), self.script.as_mut()) {
            script.accept(offer);
        }
        self.script_pick = 0;
    }

    /// Takes the text into the document, if all of it reads. Says whether it did.
    pub fn apply_script(&mut self) -> bool {
        let Some(script) = self.script.as_mut() else { return false };
        match script.outcome.clone() {
            Some(Ok(read)) => {
                script.changed = false;
                self.status = tf("Applied: {} rules and {} numbers", &[&read.rules.len(), &read.variables.len()]);
                (self.ed.doc.rules, self.ed.doc.variables, self.ed.doc.commands) = (read.rules, read.variables, read.commands);
                self.rule_picked = 0;
                self.ed.commit(t("Rules as Text"));
                true
            }
            _ => {
                self.status = t("The text has lines that cannot be read: they are marked in red").into();
                false
            }
        }
    }

    /// A key pressed while Rules as Text is open.
    fn script_key(&mut self, key: &str, control: bool, shift: bool, window: &mut Window, cx: &mut Context<Self>) {
        // While sentences or names are offered, the arrows pick among them, Tab takes the one picked and
        // Esc waves them away; otherwise the keys are the editor's own.
        let offered = self.script_offers().len();
        if offered > 0 && !control {
            match key {
                "down" => {
                    self.script_pick = (self.script_pick + 1) % offered;
                    return;
                }
                "up" => {
                    self.script_pick = (self.script_pick + offered - 1) % offered;
                    return;
                }
                "tab" => {
                    self.accept_offer(self.script_pick.min(offered - 1));
                    return;
                }
                "escape" => {
                    self.script_quiet = true;
                    return;
                }
                _ => {}
            }
        }
        // Any other key starts the offering afresh.
        (self.script_pick, self.script_quiet) = (0, false);
        let Some(script) = self.script.as_mut() else { return };
        match (key, control) {
            ("escape", _) => self.script = None,
            ("enter", true) => {
                // Ctrl+Enter: take the text and see the film, as Test Movie does anywhere else.
                if self.apply_script() {
                    self.run(Cmd::Frame(crate::menus::FrameCmd::TestMovie), window, cx);
                }
            }
            ("s", true) => {
                self.apply_script();
            }
            ("v", true) => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    script.insert(&text);
                }
            }
            // Ctrl+Z takes a change back; Ctrl+Y and Ctrl+Shift+Z put it in again.
            ("z", true) if !shift => {
                script.undo();
            }
            ("y" | "z", true) => {
                script.redo();
            }
            (_, true) => {}
            ("enter", _) => script.newline(),
            ("backspace", _) => script.backspace(),
            ("delete", _) => script.delete(),
            // With nothing offered, Tab goes to the next gap of the line; with no gap, it is a step's worth of spaces.
            ("tab", _) => {
                if !script.next_gap() {
                    script.insert("\t");
                }
            }
            ("left", _) => script.step(0, -1),
            ("right", _) => script.step(0, 1),
            ("up", _) => script.step(-1, 0),
            ("down", _) => script.step(1, 0),
            ("pageup", _) => script.step(-12, 0),
            ("pagedown", _) => script.step(12, 0),
            ("home", _) => script.home(),
            ("end", _) => script.end(),
            _ => {}
        }
        // The line the caret is on is kept in sight.
        if let Some(script) = self.script.as_ref() {
            self.script_scroll.scroll_to_item(script.row);
        }
    }

    /// What a name in a rule can be chosen from: the value kept, and how the list shows it.
    pub fn rule_choices(&self, pick: crate::rules::Pick) -> Vec<(String, String)> {
        use crate::rules::{choice_name, Pick, COMPARES, CONDITIONS, EDGES, KEYS, WAYS};
        let listed = |list: &[(&str, &str)]| list.iter().map(|(key, _)| (key.to_string(), choice_name(pick, key))).collect();
        let own = |names: Vec<String>| names.into_iter().map(|name| (name.clone(), name)).collect();
        match pick {
            Pick::Object => own(self.ed.doc.chosen_names()),
            Pick::Variable => own(self.ed.doc.variables.iter().map(|v| v.name.clone()).collect()),
            Pick::Sound => own(self.ed.doc.sounds.iter().map(|s| s.name.clone()).collect()),
            Pick::Command => own(self.ed.doc.commands.iter().map(|c| c.name.clone()).collect()),
            Pick::Behavior => crate::behavior::KINDS.iter().map(|kind| (kind.key().to_string(), t(kind.name()).to_string())).collect(),
            Pick::Key => listed(&KEYS),
            Pick::Edge => listed(&EDGES),
            Pick::Way => listed(&WAYS),
            Pick::Compare => listed(&COMPARES),
            Pick::Condition => listed(&CONDITIONS),
        }
    }

    /// The `place`th of a pick's choices, or the last, or nothing if there are none: what a new rule starts with.
    fn first_choice(&self, pick: Option<crate::rules::Pick>, place: usize) -> String {
        let choices = pick.map_or_else(Vec::new, |pick| self.rule_choices(pick));
        choices.get(place).or(choices.last()).map_or_else(String::new, |(value, _)| value.clone())
    }

    /// An event of a kind, about the first things there are to be about.
    fn fresh_event(&self, kind: crate::rules::EventKind) -> crate::rules::Event {
        let (a, b, _) = kind.needs();
        let mut event = crate::rules::Event::new(kind);
        (event.a, event.b) = (self.first_choice(a, 0), self.first_choice(b, if a == b { 1 } else { 0 }));
        event
    }

    fn fresh_step(&self, kind: crate::rules::StepKind) -> crate::rules::Step {
        let (a, b, _, _) = kind.needs();
        let mut step = crate::rules::Step::new(kind);
        (step.a, step.b) = (self.first_choice(a, 0), self.first_choice(b, if a == b { 1 } else { 0 }));
        step
    }

    /// Changes the rule on show in Rules and Numbers.
    pub fn edit_rule(&mut self, change: impl FnOnce(&mut crate::rules::Rule)) {
        if let Some(rule) = self.ed.doc.rules.get_mut(self.rule_picked) {
            change(rule);
            self.ed.commit(t("Rules"));
        }
    }

    pub fn add_rule(&mut self, kind: crate::rules::EventKind) {
        self.commit_field();
        let event = self.fresh_event(kind);
        self.ed.doc.rules.push(crate::rules::Rule { event, ..Default::default() });
        self.rule_picked = self.ed.doc.rules.len() - 1;
        self.ed.commit(t("New Rule"));
    }

    pub fn remove_rule(&mut self) {
        self.commit_field();
        if self.rule_picked < self.ed.doc.rules.len() {
            self.ed.doc.rules.remove(self.rule_picked);
            self.rule_picked = self.rule_picked.min(self.ed.doc.rules.len().saturating_sub(1));
            self.ed.commit(t("Delete Rule"));
        }
    }

    pub fn set_rule_event(&mut self, kind: crate::rules::EventKind) {
        let event = self.fresh_event(kind);
        self.edit_rule(|rule| rule.event = event);
    }

    pub fn add_rule_step(&mut self, kind: crate::rules::StepKind) {
        self.commit_field();
        let step = self.fresh_step(kind);
        self.edit_rule(|rule| rule.steps.push(step));
    }

    pub fn set_rule_step(&mut self, index: usize, kind: crate::rules::StepKind) {
        let step = self.fresh_step(kind);
        self.edit_rule(|rule| {
            if let Some(old) = rule.steps.get_mut(index) {
                *old = step;
            }
        });
    }

    pub fn remove_rule_step(&mut self, index: usize) {
        self.commit_field();
        self.edit_rule(|rule| {
            if index < rule.steps.len() {
                rule.steps.remove(index);
            }
        });
    }

    /// A new number, with a name no other has.
    pub fn add_variable(&mut self) {
        self.commit_field();
        let taken = |name: &str| self.ed.doc.variables.iter().any(|v| v.name == name);
        let name = [t("Score"), t("Lives"), t("Time")].into_iter().map(str::to_string).find(|name| !taken(name)).unwrap_or_else(|| {
            (1..).map(|n| tf("Number {}", &[&n])).find(|name| !taken(name)).unwrap_or_default()
        });
        self.ed.doc.variables.push(crate::rules::Variable { name, start: 0.0 });
        self.ed.commit(t("New Number"));
    }

    pub fn remove_variable(&mut self, index: usize) {
        self.commit_field();
        if index < self.ed.doc.variables.len() {
            self.ed.doc.variables.remove(index);
            self.ed.commit(t("Delete Number"));
        }
    }

    /// What the first selected object has been given, which is what Behavior Settings lists.
    pub fn shown_behaviors(&self) -> Vec<crate::behavior::Behavior> {
        self.ed.selected().first().map_or_else(Vec::new, |o| o.behaviors.clone())
    }

    fn run_behavior(&mut self, cmd: crate::menus::BehaviorCmd) {
        use crate::behavior::Behavior;
        use crate::menus::BehaviorCmd;
        match cmd {
            BehaviorCmd::Toggle(kind) => {
                // Chosen again when everything selected has it, the behavior is taken away.
                let all = self.ed.selected().iter().all(|o| o.behaviors.iter().any(|b| b.kind == kind));
                self.ed.for_selected(|o| {
                    if all {
                        o.behaviors.retain(|b| b.kind != kind);
                    } else if !o.behaviors.iter().any(|b| b.kind == kind) {
                        o.behaviors.push(Behavior::new(kind));
                    }
                });
                self.ed.commit(t(kind.name()));
                self.status = if all {
                    t("The behavior was removed").into()
                } else if kind.interactive() {
                    t("Added. This behavior answers the mouse or the keyboard: see it with Test Movie (Ctrl+Enter)").into()
                } else {
                    t("Added. Press Enter to see it; its numbers are in Behaviors > Behavior Settings").into()
                };
            }
            BehaviorCmd::Attach => self.attach_dialog = true,
            BehaviorCmd::Settings => self.behavior_dialog = true,
            BehaviorCmd::Clear => {
                self.ed.for_selected(|o| o.behaviors.clear());
                self.ed.commit(t("Remove All Behaviors"));
                self.behavior_dialog = false;
            }
        }
    }

    /// Takes one behavior away from the selection, as the button beside it in Behavior Settings does.
    pub fn remove_behavior(&mut self, kind: crate::behavior::BehaviorKind) {
        self.commit_field();
        self.ed.for_selected(|o| o.behaviors.retain(|b| b.kind != kind));
        self.ed.commit(t("Remove Behavior"));
        if self.shown_behaviors().is_empty() {
            self.behavior_dialog = false;
        }
    }

    /// The objects the selection can be attached to: every name on the stage that is not the selection's own.
    pub fn attach_choices(&self) -> Vec<String> {
        let own: Vec<&str> = self.ed.selected().iter().map(|o| o.name.as_str()).collect();
        // Only what is on this scene can be gone along with; and not what is itself attached, by however
        // many steps, to the selection: the two would hold each other.
        let page = self.ed.doc.page();
        let here: Vec<&Object> = page.objects().collect();
        let leads_back = |name: &str| {
            let (mut at, mut steps) = (name.to_string(), 0);
            loop {
                if own.contains(&at.as_str()) {
                    return true;
                }
                match here.iter().find(|o| o.name == at && !o.attached.is_empty()) {
                    Some(next) if steps < 16 => (at, steps) = (next.attached.clone(), steps + 1),
                    _ => return false,
                }
            }
        };
        self.ed.doc.chosen_names().into_iter().filter(|name| here.iter().any(|o| o.name == *name) && !leads_back(name)).collect()
    }

    /// Attaches the selection to the object of a name, or, with no name, lets it go.
    pub fn attach_selection(&mut self, to: &str) {
        self.ed.for_selected(|o| o.attached = to.to_string());
        self.ed.commit(t("Attach to Another Object"));
        self.attach_dialog = false;
        self.status = if to.is_empty() { t("The object is attached to nothing now").into() } else { tf("Attached to {}: it goes wherever that goes. See it with Test Movie (Ctrl+Enter)", &[&to]).into() };
    }

    pub fn close_behavior_dialog(&mut self) {
        self.commit_field();
        self.behavior_dialog = false;
    }

    /// Closes Twist and Fade; kept, the copies are made.
    pub fn close_twist_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some(settings) = self.twist_dialog.take() else { return };
        if keep && self.ed.twist_and_fade(settings) > 0 {
            self.ed.commit(t("Twist and Fade"));
        }
    }

    /// Closes Add Picture Frame; kept, a frame of the width asked for is laid round the canvas on a locked layer of its own.
    pub fn close_frame_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some(width) = self.frame_dialog.take() else { return };
        if !keep {
            return;
        }
        self.ed.leave_symbol();
        let area = self.ed.doc.page().rect();
        let thickness = (width / area.w.min(area.h)).clamp(0.02, 0.49);
        let top = self.ed.doc.page().layers.len() - 1;
        self.ed.doc.page_mut().active_layer = top;
        self.ed.add_layer();
        let mut frame = self.new_object(t("Frame"), Shape::Auto { kind: AutoKind::Frame, a: thickness, b: 0.0, count: 0 }, area, Vec::new());
        (frame.fill, frame.stroke) = (Some(Color::rgb(0x8b, 0x5a, 0x2b)), None);
        frame.filters.push(LiveFilter::new(FilterKind::InnerBevel));
        let page = self.ed.doc.page_mut();
        let layer = &mut page.layers[page.active_layer];
        layer.name = t("Frame").into();
        layer.objects.push(frame);
        layer.locked = true;
        self.ed.selection.clear();
        self.ed.commit(t("Add Picture Frame"));
    }

    /// Closes Reverse State Range; kept, the states from the first to the last asked for change places.
    pub fn close_reverse_dialog(&mut self, keep: bool) {
        self.commit_field();
        let Some((from, to)) = self.reverse_dialog.take() else { return };
        if keep && self.ed.reverse_states(from.saturating_sub(1) as usize, to.saturating_sub(1) as usize) {
            self.ed.commit(t("Reverse State Range"));
        }
    }

    fn run_command(&mut self, cmd: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        self.close_overlays();
        // Finishing the pending transform forgets its center point, which Numeric Transform turns around.
        let center_point = self.transform.as_ref().and_then(|session| session.pivot);
        self.finish_pending();
        if !self.is_enabled(cmd) {
            return;
        }
        // Play is also Stop: it has to find the timeline still playing to stop it.
        if !matches!(cmd, Cmd::Panel(_) | Cmd::HidePanels | Cmd::Frame(crate::menus::FrameCmd::Play)) {
            self.stop_playing();
        }
        let pixels = self.active_pixels().map(|s| (s.object, s.mask.clone()));
        match cmd {
            Cmd::New | Cmd::Open | Cmd::Exit | Cmd::NewFromTemplate | Cmd::Example(_) | Cmd::SahneExample(_) => self.confirm_discard(cmd, window, cx),
            Cmd::OpenInOther => self.open_in_other(),
            Cmd::Behavior(behavior) => self.run_behavior(behavior),
            Cmd::Rules => self.rules_dialog = true,
            Cmd::Script => self.open_script(),
            Cmd::Guide => self.open_guide(),
            Cmd::ManageCommands => {
                self.saved_commands = crate::commands::saved();
                self.commands_dialog = Some(None);
            }
            Cmd::RunScript => self.run_script(window, cx),
            Cmd::RunSaved(index) => {
                if let Some((_, path)) = self.saved_commands.get(index as usize).cloned() {
                    self.play_file(&path, window, cx);
                }
            }
            Cmd::SaveSteps => self.command_name = Some((tf("Command {}", &[&(self.saved_commands.len() + 1)]), None)),
            Cmd::Ready(ready) => self.run_ready(ready),
            Cmd::BatchProcess => self.batch_dialog = Some(BatchDialog { width: 640, height: 480, percent: 50.0, ..Default::default() }),
            Cmd::SaveAsTemplate => self.template_name = Some(self.path.as_deref().map_or_else(|| t("Template").to_string(), file_stem)),
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
                self.ed.commit(t("Cut"));
            }
            Cmd::Copy => match &pixels {
                Some((id, mask)) => self.ed.copy_pixels(*id, mask),
                None => self.ed.copy(),
            },
            Cmd::Paste => {
                self.pixel_sel = None;
                self.ed.paste(PASTE_OFFSET);
                self.ed.commit(t("Paste"));
            }
            Cmd::Clear => {
                match &pixels {
                    Some((id, mask)) => self.ed.clear_pixels(*id, mask),
                    None => self.ed.delete_selected(),
                }
                self.ed.commit(t("Clear"));
            }
            Cmd::Duplicate => {
                self.ed.duplicate(PASTE_OFFSET);
                self.ed.commit(t("Duplicate"));
            }
            Cmd::ZoomIn => self.step_zoom(true, None),
            Cmd::ZoomOut => self.step_zoom(false, None),
            Cmd::ZoomTo(percent) => self.zoom_about(percent as f32 / 100.0, None),
            Cmd::FitAll => self.fit_all(),
            Cmd::Rulers => self.show_rulers = !self.show_rulers,
            Cmd::Grid => self.show_grid = !self.show_grid,
            Cmd::TextFont(index) => {
                if let Some(name) = crate::fonts::families().get(index as usize) {
                    self.change_text_style(|style| style.font = name.clone());
                }
            }
            Cmd::TextSize(size) => self.set_text_size(size as f32),
            Cmd::TextSizeStep(larger) => {
                let (now, sizes) = (self.shown_text_size(), crate::menus::TEXT_SIZES);
                let next = if larger { sizes.iter().find(|s| **s as f32 > now + 0.01) } else { sizes.iter().rev().find(|s| (**s as f32) < now - 0.01) };
                if let Some(size) = next {
                    self.set_text_size(*size as f32);
                }
            }
            Cmd::TextStyle(what) => self.change_text_style(|style| match what {
                crate::menus::TextStyleCmd::Plain => (style.bold, style.italic, style.underline) = (false, false, false),
                crate::menus::TextStyleCmd::Bold => style.bold = !style.bold,
                crate::menus::TextStyleCmd::Italic => style.italic = !style.italic,
                crate::menus::TextStyleCmd::Underline => style.underline = !style.underline,
            }),
            Cmd::TextAlign(align) | Cmd::TextAlignDown(align) => self.change_text_style(|style| style.align = align),
            Cmd::TextOrientation(orientation) => self.change_text_style(|style| style.orientation = orientation),
            Cmd::Guides(GuideCmd::Edit) => self.guides_dialog = true,
            Cmd::Guides(GuideCmd::Show) => self.show_guides = !self.show_guides,
            Cmd::Guides(GuideCmd::Lock) => self.lock_guides = !self.lock_guides,
            Cmd::Guides(GuideCmd::Snap) => self.snap_guides = !self.snap_guides,
            Cmd::Guides(GuideCmd::ShowSmart) => self.show_smart_guides = !self.show_smart_guides,
            Cmd::Guides(GuideCmd::SnapSmart) => self.snap_smart_guides = !self.snap_smart_guides,
            Cmd::Guides(GuideCmd::Slice) => self.show_slice_guides = !self.show_slice_guides,
            Cmd::Guides(GuideCmd::Clear) => {
                self.ed.doc.page_mut().guides = Default::default();
                self.ed.commit(t("Clear Guides"));
            }
            Cmd::SelectAll => self.ed.select_all(),
            // The first Deselect drops the pixel selection, the next the object.
            Cmd::Deselect => {
                if self.pixel_sel.take().is_none() {
                    self.ed.selection.clear();
                }
            }
            Cmd::Arrange(op) => {
                self.ed.arrange(op);
                self.ed.commit(t("Arrange"));
            }
            Cmd::Align(op) => {
                self.ed.align(op);
                self.ed.commit(t("Align"));
            }
            Cmd::FlipH | Cmd::FlipV => {
                self.ed.for_selected(|o| o.flip(matches!(cmd, Cmd::FlipH)));
                self.ed.commit(t("Flip"));
            }
            Cmd::Filter(filter) => {
                self.ed.apply_filter(filter, pixels.as_ref().map(|(id, mask)| (*id, mask.as_ref())));
                self.ed.commit(filter.label());
                self.last_filter = Some(filter);
            }
            Cmd::RepeatFilter => {
                if let Some(filter) = self.last_filter {
                    self.ed.apply_filter(filter, pixels.as_ref().map(|(id, mask)| (*id, mask.as_ref())));
                    self.ed.commit(filter.label());
                }
            }
            Cmd::Revert => self.confirm_revert(window, cx),
            Cmd::Clone => {
                // A copy right on top of the original, selected in its place.
                self.ed.duplicate(0.0);
                self.ed.commit(t("Clone"));
            }
            Cmd::PasteAttributes => {
                if self.ed.paste_attributes() {
                    self.ed.commit(t("Paste Attributes"));
                }
            }
            Cmd::InsertWeb(kind) => {
                if self.ed.selected().iter().filter(|o| !o.is_web()).count() > 1 {
                    self.insert_web_asking(kind, window, cx);
                } else {
                    self.insert_web(kind, true);
                }
            }
            Cmd::InsertBitmap => {
                if self.ed.insert_empty_bitmap() {
                    self.ed.commit(t("Empty Bitmap"));
                } else {
                    self.status = t("All layers are locked").into();
                    self.ed.rollback();
                }
            }
            Cmd::AddState => {
                self.ed.selection.clear();
                self.ed.doc.page_mut().add_state();
                self.ed.commit(t("New State"));
            }
            Cmd::FitSelection => self.fit_selection(),
            Cmd::SelectSimilar => {
                if let Some((id, mask)) = &pixels {
                    let similar = self.ed.object(*id).and_then(|o| o.bitmap()).and_then(|bitmap| bitmap.similar(mask, self.tool_options.wand_tolerance as u8));
                    self.set_pixel_selection(*id, similar, MaskMode::Replace);
                }
            }
            Cmd::SelectInverse => match &pixels {
                Some((id, mask)) => self.set_pixel_selection(*id, Some(mask.inverted()), MaskMode::Replace),
                // Without a marquee it is the objects that change places: those not selected become the selection.
                None => {
                    let page = self.ed.doc.page();
                    let others: Vec<u64> =
                        page.layers.iter().filter(|l| l.visible && !l.locked).flat_map(|l| &l.objects).filter(|o| o.visible && !self.ed.is_selected(o.id)).map(|o| o.id).collect();
                    self.ed.selection = others;
                }
            },
            Cmd::Marquee(op) => self.marquee_dialog = Some((op, 10.0)),
            Cmd::TrimCanvas => {
                if self.ed.trim_canvas() {
                    self.pixel_sel = None;
                    self.ed.commit(t("Trim Canvas"));
                } else {
                    self.status = t("There is nothing to trim away").into();
                }
            }
            Cmd::FlattenSelection | Cmd::MergeDown | Cmd::FlattenLayers => {
                let (done, label) = match cmd {
                    Cmd::FlattenSelection => (self.ed.flatten_selection(), t("Flatten Selection")),
                    Cmd::MergeDown => (self.ed.merge_down(), t("Merge Down")),
                    _ => (self.ed.flatten_layers(), t("Flatten Layers")),
                };
                if done {
                    self.pixel_sel = None;
                    self.ed.commit(label);
                } else {
                    self.status = t("There is nothing to flatten").into();
                    self.ed.rollback();
                }
            }
            Cmd::ShowWeb(visible) => {
                self.ed.set_web_visible(visible);
                self.ed.commit(if visible { t("Show Slices") } else { t("Hide Slices") });
            }
            Cmd::AddLayer => {
                self.ed.add_layer();
                self.ed.commit(t("New Layer"));
            }
            Cmd::AddPage => {
                self.ed.add_page();
                self.ed.commit(t("New Page"));
            }
            Cmd::Panel(panel) => self.toggle_panel(panel),
            Cmd::HidePanels => self.panels_off = !self.panels_off,
            Cmd::Language(language) => {
                crate::i18n::set_language(language);
                let mut settings = Settings::load();
                settings.set_language(language);
                if let Err(error) = settings.save() {
                    self.status = tf("Could not save: {}", &[&error]);
                }
            }
            Cmd::ConvertToSymbol => self.make_symbol(),
            Cmd::NewSymbol => self.symbol_dialog = Some((tf("Symbol {}", &[&(self.ed.doc.symbols.len() + 1)]), SymbolKind::Graphic)),
            Cmd::NewButton => self.new_symbol(t("Button"), SymbolKind::Button),
            Cmd::EditSymbol => self.toggle_symbol_edit(),
            Cmd::Frame(frame) => self.run_frame(frame, window, cx),
            Cmd::GifWizard => self.gif_dialog = Some(GifDialog { files: Vec::new(), delay_ms: 500, loops: 0 }),
            Cmd::TweenInstances => self.tween_dialog = Some((10, true)),
            Cmd::Animate | Cmd::AnimateSettings => {
                let settings = self.ed.selected_animation().map(|index| self.ed.doc.page().animations[index].settings);
                self.animate_dialog = Some(settings.unwrap_or_default());
            }
            Cmd::RemoveAnimation => {
                if self.ed.remove_animation() {
                    self.ed.commit(t("Remove Animation"));
                }
            }
            Cmd::BreakApart => {
                self.ed.break_apart();
                self.ed.commit(t("Break Apart"));
            }
            Cmd::PathOp(op) => self.run_path_op(op),
            Cmd::Group => {
                if self.ed.group_selected() {
                    self.ed.commit(t("Group"));
                }
            }
            Cmd::ImagePreview => self.open_image_preview(None),
            Cmd::Preferences => self.prefs_dialog = Some(self.prefs),
            Cmd::KeyboardShortcuts => self.shortcut_dialog = Some((None, false)),
            Cmd::SnapToGrid => {
                self.prefs.snap_to_grid = !self.prefs.snap_to_grid;
                self.save_preferences();
            }
            Cmd::CheckSpelling => self.check_spelling(),
            Cmd::AttachToPath => {
                if self.ed.attach_text_to_path() {
                    self.ed.commit(t("Attach to Path"));
                }
            }
            Cmd::DetachFromPath => {
                if self.ed.detach_text_from_path() {
                    self.ed.commit(t("Detach from Path"));
                }
            }
            Cmd::ReversePathText => {
                if self.ed.reverse_text_path() {
                    self.ed.commit(t("Reverse Direction"));
                }
            }
            Cmd::PathOrientation(orientation) => self.change_text_style(|style| style.on_path = orientation),
            Cmd::PageSetup => self.page_setup_dialog = Some(self.print_setup),
            Cmd::Print => {
                let (printers, default) = crate::print::printers();
                // The default printer if there is one, else the first; with none set up, a PDF file.
                let chosen = default.and_then(|name| printers.iter().position(|p| *p == name)).unwrap_or(0);
                let pages = self.ed.doc.pages.len() as u32;
                self.print_dialog = Some(PrintDialog { printers, chosen, copies: 1, range: Default::default(), from: 1, to: pages, two_sided: false, grayscale: false });
            }
            Cmd::Subselect => {
                if self.ed.subselect() {
                    self.ed.commit(t("Subselect"));
                }
            }
            Cmd::Superselect => {
                if self.ed.superselect() {
                    self.ed.commit(t("Superselect"));
                }
            }
            Cmd::Ungroup => {
                self.ed.break_apart();
                self.ed.commit(t("Ungroup"));
            }
            Cmd::Rotate { clockwise } => {
                let skipped = self.ed.rotate_selected(clockwise);
                self.ed.commit(t("Rotate"));
                self.report_transform(skipped);
            }
            Cmd::Rotate180 => {
                let skipped = self.ed.rotate_selected(true).max(self.ed.rotate_selected(true));
                self.ed.commit(t("Rotate"));
                self.report_transform(skipped);
            }
            Cmd::RemoveTransform => {
                self.ed.for_selected(|o| {
                    o.remove_transform();
                });
                self.ed.commit(t("Remove Transformations"));
            }
            Cmd::FreeTransform => self.set_tool(Tool::Scale),
            Cmd::NumericTransform => self.open_numeric_dialog(center_point),
            Cmd::MaskAll(reveal) => {
                if self.ed.add_bitmap_mask(reveal, None) {
                    self.ed.commit(if reveal { t("Reveal All") } else { t("Hide All") });
                }
            }
            Cmd::MaskSelection(reveal) => {
                if let Some((object, mask)) = pixels {
                    self.ed.selection = vec![object];
                    if self.ed.add_bitmap_mask(reveal, Some(&mask)) {
                        self.pixel_sel = None;
                        self.ed.commit(if reveal { t("Reveal Selection") } else { t("Hide Selection") });
                    }
                }
            }
            Cmd::PasteAsMask => self.paste_as_mask_asking(window, cx),
            Cmd::PasteInside => {
                if self.ed.paste_inside() {
                    self.ed.commit(t("Paste Inside"));
                }
            }
            Cmd::GroupAsMask => {
                if self.ed.group_as_mask() {
                    self.ed.commit(t("Group as Mask"));
                }
            }
            Cmd::MaskToggle => {
                self.ed.toggle_masks();
                self.ed.commit(t("Disable Mask"));
            }
            Cmd::MaskDelete => self.confirm_delete_mask(window, cx),
            Cmd::JpegMaskSave => self.save_jpeg_mask(),
            Cmd::JpegMaskRestore => self.restore_jpeg_mask(),
            Cmd::JpegMaskRemove => {
                self.ed.doc.page_mut().jpeg_mask = None;
                self.ed.commit(t("Remove JPEG Mask"));
            }
            Cmd::JpegSettings => self.jpeg_dialog = Some((self.ed.doc.page().export, self.ed.doc.page().jpeg_mask_color)),
            Cmd::SelectTool(tool) => self.set_tool(tool),
            Cmd::TextToPaths => {
                self.ed.for_selected(|o| {
                    o.text_to_paths();
                });
                self.ed.commit(t("Convert to Paths"));
            }
            Cmd::CanvasSize => self.open_size_dialog(SizeDialogKind::Canvas),
            Cmd::ImageSize => self.open_size_dialog(SizeDialogKind::Image),
            Cmd::FitCanvas => {
                if self.ed.fit_canvas() {
                    self.ed.commit(t("Fit Canvas"));
                } else {
                    self.status = t("There is no artwork to fit the canvas to").into();
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
                // The addresses cannot be clicked in the box, and are easily mistyped: its buttons open them.
                let answer = window.prompt(PromptLevel::Info, app_name(), Some(&about_text()), &[t("OK"), t("Site and book"), t("Developer's site"), t("Source code (GitHub)")], cx);
                self.awaiting_answer = true;
                cx.spawn_in(window, async move |this, cx| {
                    let answer = answer.await;
                    this.update_in(cx, |this, _, cx| {
                        this.awaiting_answer = false;
                        if let Ok(choice @ 1..=3) = answer {
                            let place = [HOME, SITE, SOURCE][choice - 1];
                            if let Err(error) = std::process::Command::new("xdg-open").arg(place).spawn() {
                                this.status = tf("Could not open the browser: {}", &[&error]);
                            }
                        }
                        cx.notify();
                    })
                    .ok();
                })
                .detach();
            }
        }
        cx.notify();
    }

    /// How the selected text is set, or without one, how the next text will be.
    pub fn shown_text_style(&self) -> crate::model::TextStyle {
        self.ed.selected().iter().find_map(|o| match &o.shape {
            Shape::Text { style, .. } => Some(style.clone()),
            _ => None,
        })
        .unwrap_or_else(|| self.text_style.clone())
    }

    fn shown_text_size(&self) -> f32 {
        self.ed.selected().iter().find_map(|o| match &o.shape {
            Shape::Text { size, .. } => Some(*size),
            _ => None,
        })
        .unwrap_or(self.text_size)
    }

    /// Changes how the selected text is set, and how the next text made will be.
    /// A toggle goes by the first selected text, so several end up alike.
    pub fn change_text_style(&mut self, change: impl Fn(&mut crate::model::TextStyle)) {
        let mut style = self.shown_text_style();
        change(&mut style);
        self.text_style = style.clone();
        let mut changed = false;
        self.ed.for_selected(|o| {
            if let Shape::Text { style: own, .. } = &mut o.shape {
                *own = style.clone();
                o.refit_text();
            }
        });
        changed |= self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Text { .. }));
        if changed {
            self.ed.commit(t("Text"));
        }
    }

    fn set_text_size(&mut self, to: f32) {
        self.text_size = to;
        let any = self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Text { .. }));
        self.ed.for_selected(|o| {
            if let Shape::Text { size, .. } = &mut o.shape {
                *size = to;
                o.refit_text();
            }
        });
        if any {
            self.ed.commit(t("Text Size"));
        }
    }

    /// Closes Page Setup. Kept, the setup is remembered for the next run; otherwise it goes back to what it was.
    pub fn close_page_setup(&mut self, keep: bool) {
        self.commit_field();
        let Some(before) = self.page_setup_dialog.take() else { return };
        if !keep {
            self.print_setup = before;
            return;
        }
        let mut settings = Settings::load();
        settings.print = self.print_setup;
        if let Err(error) = settings.save() {
            self.status = error.to_string();
        }
    }

    /// The page as it prints: laid out on the paper as a PDF. A see-through canvas prints as white paper.
    fn printed_pages(&mut self, pages: &[usize]) -> anyhow::Result<Vec<u8>> {
        self.ed.leave_symbol_for_good();
        self.ed.end_mask_edit();
        self.ed.close_groups();
        let mut pictures = Vec::new();
        for index in pages {
            let pixmap = crate::export::render_page(&self.ed.export_page_at(*index))?;
            let rgb: Vec<u8> = pixmap.pixels().iter().flat_map(|px| [px.red(), px.green(), px.blue()].map(|c| c.saturating_add(255 - px.alpha()))).collect();
            pictures.push(crate::print::Picture { rgb, width: pixmap.width(), height: pixmap.height() });
        }
        crate::print::pdf_of(&pictures, &self.print_setup)
    }

    /// OK in the Print dialog: the page goes to the printer picked, or into a PDF file.
    pub fn apply_print_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_field();
        let Some(dialog) = self.print_dialog.take() else { return };
        let pages = crate::print::pages_in(dialog.range, self.ed.doc.active_page, self.ed.doc.pages.len(), dialog.from, dialog.to);
        let pdf = match self.printed_pages(&pages) {
            Ok(pdf) => pdf,
            Err(error) => {
                self.status = error.to_string();
                return;
            }
        };
        let stem = self.path.as_ref().or(self.source_image.as_ref()).map_or_else(|| t("Untitled").into(), |p| file_stem(p));
        let page = self.ed.doc.page();
        let too_big = self.print_setup.overflows(page.width, page.height);
        if let Some(printer) = dialog.printers.get(dialog.chosen) {
            self.status = match crate::print::send(&pdf, printer, dialog.copies, &self.print_setup, &stem, crate::print::Options { two_sided: dialog.two_sided, grayscale: dialog.grayscale }) {
                Ok(_) if too_big => tf("Sent to {}. The page is larger than the paper, so it is printed on several sheets.", &[printer]),
                Ok(_) => tf("Sent to {}", &[printer]),
                Err(error) => tf("Could not print: {}", &[&error]),
            };
            return;
        }
        let picked = cx.prompt_for_new_path(&Self::default_dir(), Some(&format!("{stem}.pdf")));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(mut path))) = picked.await {
                if path.extension().is_none() {
                    path.set_extension("pdf");
                }
                this.update(cx, |this, cx| {
                    this.status = match std::fs::write(&path, &pdf) {
                        Ok(()) => tf("Saved {}", &[&path.file_name().unwrap_or_default().to_string_lossy()]),
                        Err(error) => tf("could not write {}", &[&format!("{}: {error}", path.display())]),
                    };
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// Writes the preferences and the changed shortcuts to the settings file, and puts the number of undo steps to work.
    pub(crate) fn save_preferences(&mut self) {
        self.ed.history_limit = self.prefs.undo_steps.clamp(2, 1000) as usize + 1;
        let mut settings = Settings::load();
        (settings.preferences, settings.shortcuts) = (self.prefs, self.shortcuts.clone());
        if let Err(error) = settings.save() {
            self.status = error.to_string();
        }
    }

    /// Closes Preferences. Kept, they are remembered for the next run; otherwise they go back to what they were.
    pub fn close_preferences(&mut self, keep: bool) {
        self.commit_field();
        let Some(before) = self.prefs_dialog.take() else { return };
        if keep {
            self.save_preferences();
        } else {
            self.prefs = before;
        }
    }

    /// The keys a menu command has now: those the user gave it, or those it came with.
    pub fn shortcut_of(&self, name: &str, keys: &'static str) -> String {
        self.shortcuts.get(name).cloned().unwrap_or_else(|| keys.to_string())
    }

    /// What a key press does once the user's own shortcuts are taken into account:
    /// the command given those keys, or nothing at all if the keys belonged to a
    /// command that has since been given others. `None` leaves the press to the built-in shortcuts.
    fn custom_shortcut(&self, pressed: &str) -> Option<Option<Cmd>> {
        if self.shortcuts.is_empty() || pressed.is_empty() {
            return None;
        }
        let commands = crate::menus::menu_commands();
        if let Some((_, _, _, cmd)) = commands.iter().find(|(_, name, _, _)| self.shortcuts.get(*name).is_some_and(|keys| keys.eq_ignore_ascii_case(pressed))) {
            return Some(Some(*cmd));
        }
        commands.iter().any(|(_, name, keys, _)| keys.eq_ignore_ascii_case(pressed) && self.shortcuts.contains_key(*name)).then_some(None)
    }

    /// Gives the command picked in Keyboard Shortcuts new keys, none, or the ones it came with.
    pub fn set_shortcut(&mut self, keys: Option<String>) {
        let Some((Some(picked), _)) = self.shortcut_dialog else { return };
        let Some((_, name, _, _)) = crate::menus::menu_commands().get(picked).copied() else { return };
        match keys {
            Some(keys) => {
                // Keys can open one command only: any other that had them loses them.
                if !keys.is_empty() {
                    for (_, other, default, _) in crate::menus::menu_commands() {
                        if other != name && self.shortcut_of(other, default).eq_ignore_ascii_case(&keys) {
                            self.shortcuts.insert(other.to_string(), String::new());
                        }
                    }
                }
                self.shortcuts.insert(name.to_string(), keys);
            }
            None => {
                self.shortcuts.remove(name);
            }
        }
        self.shortcut_dialog = Some((Some(picked), false));
        self.save_preferences();
    }

    /// The texts Check Spelling goes through: the selected ones, or with none selected every text on the page.
    fn spell_targets(&self) -> Vec<u64> {
        let is_text = |o: &&Object| matches!(o.shape, Shape::Text { .. });
        let selected: Vec<u64> = self.ed.selected().into_iter().filter(is_text).map(|o| o.id).collect();
        if !selected.is_empty() {
            return selected;
        }
        self.ed.doc.page().objects().filter(is_text).map(|o| o.id).collect()
    }

    fn text_of(&self, id: u64) -> Option<&str> {
        match &self.ed.object(id)?.shape {
            Shape::Text { content, .. } => Some(content.as_str()),
            _ => None,
        }
    }

    /// The next misspelled word that has not been dealt with: its text, where it starts and ends there, and the word.
    pub fn spell_current(&self) -> Option<(u64, usize, usize, String)> {
        let session = self.spell.as_ref()?;
        self.spell_targets().into_iter().find_map(|id| {
            crate::spell::words(self.text_of(id)?)
                .into_iter()
                .find(|(from, _, word)| session.wrong.contains_key(word) && !session.ignored.contains(word) && !session.skipped.contains(&(id, *from)))
                .map(|(from, to, word)| (id, from, to, word))
        })
    }

    /// Text > Check Spelling: checks every word of the texts and opens the dialog at the first one not found.
    pub fn check_spelling(&mut self) {
        self.finish_pending();
        let personal = Settings::load().personal_words;
        let mut words: Vec<String> = self.spell_targets().into_iter().filter_map(|id| self.text_of(id)).flat_map(crate::spell::words).map(|(_, _, word)| word).collect();
        words.sort();
        words.dedup();
        words.retain(|word| !personal.contains(word));
        match crate::spell::check(&words, crate::spell::dictionary()) {
            Ok(wrong) => {
                self.spell = Some(SpellSession { wrong, ignored: Default::default(), skipped: Default::default(), replacement: String::new() });
                self.spell_advance();
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    /// Goes on to the next word, offering the checker's first suggestion for it, or finishes when none is left.
    fn spell_advance(&mut self) {
        self.field = None;
        match self.spell_current() {
            Some((id, _, _, word)) => {
                self.ed.selection = vec![id];
                if let Some(session) = self.spell.as_mut() {
                    session.replacement = session.wrong.get(&word).and_then(|list| list.first()).cloned().unwrap_or(word);
                }
            }
            None => {
                self.spell = None;
                self.status = t("Spelling check complete.").into();
            }
        }
    }

    /// Ignore passes over this one place; Ignore All passes over the word wherever it is.
    pub fn spell_ignore(&mut self, everywhere: bool) {
        self.commit_field();
        let Some((id, from, _, word)) = self.spell_current() else { return };
        if let Some(session) = self.spell.as_mut() {
            if everywhere {
                session.ignored.insert(word);
            } else {
                session.skipped.insert((id, from));
            }
        }
        self.spell_advance();
    }

    /// Add to Personal: the word is right from now on, in every document.
    pub fn spell_add(&mut self) {
        self.commit_field();
        let Some((_, _, _, word)) = self.spell_current() else { return };
        let mut settings = Settings::load();
        if !settings.personal_words.contains(&word) {
            settings.personal_words.push(word.clone());
            if let Err(error) = settings.save() {
                self.status = error.to_string();
            }
        }
        if let Some(session) = self.spell.as_mut() {
            session.ignored.insert(word);
        }
        self.spell_advance();
    }

    /// Change puts the replacement in place of the word on show; Change All in place of the word wherever it is.
    pub fn spell_change(&mut self, everywhere: bool) {
        self.commit_field();
        let (Some((id, from, to, word)), Some(replacement)) = (self.spell_current(), self.spell.as_ref().map(|session| session.replacement.clone())) else { return };
        let targets = if everywhere { self.spell_targets() } else { vec![id] };
        for target in targets {
            let Some(object) = self.ed.object_mut(target) else { continue };
            let Shape::Text { content, .. } = &mut object.shape else { continue };
            // From the end of the text backward, so the places still to change do not move.
            let places: Vec<(usize, usize)> = match everywhere {
                true => crate::spell::words(content).into_iter().filter(|(_, _, found)| *found == word).map(|(a, b, _)| (a, b)).rev().collect(),
                false => vec![(from, to)],
            };
            for (a, b) in places {
                content.replace_range(a..b, &replacement);
            }
            object.refit_text();
        }
        self.ed.commit(t("Check Spelling"));
        // What it was changed to is taken as right, even if the checker does not know it either.
        if let Some(session) = self.spell.as_mut() {
            session.wrong.remove(&replacement);
        }
        self.spell_advance();
    }

    /// Fireworks' Revert: asks, then opens the file again as it was last saved.
    fn confirm_revert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone().or_else(|| self.source_image.clone()) else { return };
        let question = t("Revert to the last saved version? The changes made since will be lost.");
        let answer = window.prompt(PromptLevel::Warning, question, None, &[t("Revert"), t("Cancel")], cx);
        self.awaiting_answer = true;
        cx.spawn_in(window, async move |this, cx| {
            let answer = answer.await;
            this.update_in(cx, |this, _, cx| {
                this.awaiting_answer = false;
                if let Ok(0) = answer {
                    this.open_file(path);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn insert_web(&mut self, kind: WebKind, single: bool) {
        if self.ed.insert_web(kind, single) {
            self.ed.commit(if kind == WebKind::Hotspot { t("Hotspot") } else { t("Slice") });
        } else {
            self.status = t("All layers are locked").into();
            self.ed.rollback();
        }
    }

    /// With several objects selected, Fireworks asks whether they get one hotspot or slice together, or one each.
    fn insert_web_asking(&mut self, kind: WebKind, window: &mut Window, cx: &mut Context<Self>) {
        let question = if kind == WebKind::Hotspot { t("Multiple items are selected. Do you want to create one hotspot, or more than one?") } else { t("Multiple items are selected. Do you want to create one slice, or more than one?") };
        let answer = window.prompt(PromptLevel::Info, question, None, &[t("Single"), t("Multiple"), t("Cancel")], cx);
        self.awaiting_answer = true;
        cx.spawn_in(window, async move |this, cx| {
            let answer = answer.await;
            this.update_in(cx, |this, _, cx| {
                this.awaiting_answer = false;
                if let Ok(choice @ (0 | 1)) = answer {
                    this.insert_web(kind, choice == 0);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Zooms and scrolls so the selection fills the view.
    fn fit_selection(&mut self) {
        let Some(area) = self.ed.selection_bounds() else { return };
        let (bounds, page) = (self.canvas_bounds.get(), self.ed.doc.page());
        let margin = 48.0;
        let fit_w = (f32::from(bounds.size.width) - margin) / area.w.max(1.0);
        let fit_h = (f32::from(bounds.size.height) - margin) / area.h.max(1.0);
        self.zoom = fit_w.min(fit_h).clamp(MIN_ZOOM, MAX_ZOOM);
        // The view is centered on the page; panning by how far the selection is from the middle of it centers the selection.
        let center = area.center();
        self.pan = v2((page.width as f32 / 2.0 - center.x) * self.zoom, (page.height as f32 / 2.0 - center.y) * self.zoom);
    }

    /// OK in Move Guide: the guide goes to the place typed.
    pub fn apply_guide_dialog(&mut self) {
        self.commit_field();
        let Some((horizontal, index, place)) = self.guide_dialog.take() else { return };
        if let Some(guide) = self.ed.doc.page_mut().guides.list_mut(horizontal).get_mut(index) {
            *guide = place.round();
            self.ed.commit(t("Guides"));
        }
    }

    /// OK in the dialog of a marquee command: the marquee is grown, shrunk, turned into a band along its edge, or evened out.
    pub fn apply_marquee_dialog(&mut self) {
        self.commit_field();
        let Some((op, amount)) = self.marquee_dialog.take() else { return };
        let Some((id, mask)) = self.active_pixels().map(|s| (s.object, s.mask.clone())) else { return };
        let by = amount.round().clamp(1.0, 1000.0) as u32;
        let changed = match op {
            crate::menus::MarqueeOp::Expand => mask.expanded(by),
            crate::menus::MarqueeOp::Contract => mask.contracted(by),
            crate::menus::MarqueeOp::Border => mask.border(by),
            crate::menus::MarqueeOp::Smooth => mask.smoothed(by),
            crate::menus::MarqueeOp::Feather => mask.feathered(by as f32),
        };
        if changed.is_empty() {
            self.status = t("Nothing would be left of the marquee").into();
            return;
        }
        self.set_pixel_selection(id, Some(changed), MaskMode::Replace);
    }

    fn confirm_discard(&mut self, cmd: Cmd, window: &mut Window, cx: &mut Context<Self>) {
        if !self.ed.is_dirty() {
            self.perform_destructive(cmd, window, cx);
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            t("Discard unsaved changes?"),
            Some(t("Your changes to this document will be lost.")),
            &[t("Discard"), t("Cancel")],
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
            Cmd::NewFromTemplate => self.template_dialog = Some((crate::templates::saved(), 0)),
            Cmd::SahneExample(which) => {
                if let Some(example) = crate::sahne_examples::EXAMPLES.get(which as usize) {
                    self.load_document((example.build)(), None);
                    self.fit_if_larger_than_view();
                    self.status = format!("{}: {}", t(example.name), t(example.about));
                }
            }
            Cmd::Example(which) => {
                if let Some(example) = crate::examples::EXAMPLES.get(which as usize) {
                    // An example opens as a new document of the user's own, with no file behind it.
                    self.load_document((example.build)(), None);
                    self.fit_if_larger_than_view();
                    self.status = format!("{}: {}", t(example.name), t(example.about));
                }
            }
            Cmd::Exit => cx.quit(),
            _ => {}
        }
    }

    fn load_document(&mut self, doc: Document, path: Option<PathBuf>) {
        self.recorded.clear();
        // The picture kept of the page is of the old document: a new one counts its changes from nothing again.
        if let Some((_, image)) = self.page_raster.take() {
            self.stale_previews.push(image);
        }
        let doc = staged(doc);
        self.ed = Editor::new(doc);
        self.path = path;
        self.source_image = None;
        self.crop_area = None;
        self.pixel_sel = None;
        self.stamp = None;
        self.pan = Vec2::default();
        self.zoom = 1.0;
        self.pen.clear();
        self.pen_handles.clear();
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

    pub(crate) fn pick_file(
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

    pub fn open_file(&mut self, path: PathBuf) {
        let opened = std::fs::read(&path).map_err(anyhow::Error::from).and_then(|bytes| parse_file(&bytes));
        match opened {
            Ok(Opened::Document(doc)) => {
                self.status = tf("Opened {}", &[&path.file_name().unwrap_or_default().to_string_lossy()]);
                self.load_document(doc, Some(path));
            }
            Ok(Opened::Animation(doc)) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                self.status = tf("Opened {} as an animation of {} states", &[&name, &doc.pages[0].states.len()]);
                self.load_document(doc, None);
                self.source_image = Some(path);
                self.fit_if_larger_than_view();
            }
            Ok(Opened::Fireworks(doc, report)) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                self.status = tf("Opened {} with its Fireworks layers: {} layer(s), {} bitmap(s)", &[&name, &report.layers, &report.bitmaps]);
                if report.unread_effects > 0 {
                    self.status = format!("{}. {}", self.status, tf("{} effect(s) could not be read", &[&report.unread_effects]));
                }
                // Like any picture, it becomes a new document: saving it never writes over the Fireworks file.
                self.load_document(doc, None);
                self.source_image = Some(path);
                self.fit_if_larger_than_view();
            }
            Ok(opened @ (Opened::Image(_) | Opened::Flat(..))) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let bitmap = match opened {
                    Opened::Flat(bitmap, unread) => {
                        self.status = tf("Opened {} as a flat picture: {} Fireworks object(s) of kinds that cannot be read (paths, text, groups)", &[&name, &unread]);
                        bitmap
                    }
                    Opened::Image(bitmap) => {
                        self.status = tf("Opened {}", &[&name]);
                        bitmap
                    }
                    _ => return,
                };
                let doc = Document::from_bitmap(file_stem(&path), bitmap);
                let id = doc.next_id - 1;
                self.load_document(doc, None);
                self.ed.selection = vec![id];
                self.source_image = Some(path);
                self.fit_if_larger_than_view();
            }
            Err(error) => self.status = tf("Could not open file: {}", &[&error]),
        }
    }

    /// Adds an image to the current document, centered on the canvas.
    fn import_file(&mut self, path: &Path) {
        // A document of either program is brought in with its objects and the symbols they use.
        if let Ok(Opened::Document(other)) = std::fs::read(path).map_err(anyhow::Error::from).and_then(|bytes| parse_file(&bytes)) {
            let count = self.ed.import_document(&other);
            self.ed.commit(t("Import"));
            self.status = tf("Imported {} object(s) from {}", &[&count, &path.file_name().unwrap_or_default().to_string_lossy()]);
            return;
        }
        let decoded = std::fs::read(path).map_err(anyhow::Error::from).and_then(|bytes| Bitmap::decode(&bytes));
        match decoded {
            Ok(bitmap) => {
                let page = self.ed.doc.page();
                let x = ((page.width as f32 - bitmap.width() as f32) / 2.0).round();
                let y = ((page.height as f32 - bitmap.height() as f32) / 2.0).round();
                self.status = tf("Imported {}", &[&path.display()]);
                let object = Object::new_bitmap(self.ed.alloc_id(), file_stem(path), bitmap, x, y);
                self.add_new(object, t("Import"));
            }
            Err(error) => self.status = tf("Could not import file: {}", &[&error]),
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
        ["USERPROFILE", "HOME"]
            .iter()
            .find_map(|name| std::env::var_os(name))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn write_document(&mut self, mut path: PathBuf) {
        self.ed.leave_symbol_for_good();
        self.ed.end_mask_edit();
        self.ed.close_groups();
        // Saving under an image name writes a flattened copy, and the editable
        // document stays unsaved. JSON must never land in a file named .png.
        // An animation is written with the work on the frame on show in its timeline.
        for page in &mut self.ed.doc.pages {
            page.store_frame();
        }
        if path.extension().is_some_and(|e| !e.eq_ignore_ascii_case(FILE_EXTENSION) && !e.eq_ignore_ascii_case(LEGACY_EXTENSION) && !e.eq_ignore_ascii_case(SAHNE_EXTENSION)) {
            self.export_to(&path);
            return;
        }
        if path.extension().is_none() {
            path.set_extension(document_extension());
        }
        let result = document_to_json(&self.ed.doc)
            .and_then(|json| std::fs::write(&path, json).map_err(anyhow::Error::from));
        match result {
            Ok(()) => {
                self.ed.mark_saved();
                self.status = tf("Saved {}", &[&path.display()]);
                self.path = Some(path);
            }
            Err(error) => self.status = tf("Could not save: {}", &[&error]),
        }
    }

    fn export_to(&mut self, path: &Path) {
        self.ed.leave_symbol_for_good();
        self.ed.end_mask_edit();
        self.ed.close_groups();
        let page = &self.ed.export_page();
        self.status = match export_page(page, path).and_then(|()| export_web(page, path)) {
            Ok(0) => tf("Exported {}", &[&path.display()]),
            Ok(extra) => tf("Exported {} and {} web file(s) beside it", &[&path.display(), &extra]),
            Err(error) => tf("Export failed: {}", &[&format!("{error:#}")]),
        };
    }

    pub fn export(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let stem = self.path.as_ref().or(self.source_image.as_ref()).map_or_else(|| t("Untitled").into(), |p| file_stem(p));
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
        if self.awaiting_answer {
            return;
        }
        self.commit_field();
        window.focus(&self.focus);
        self.stop_playing();
        // In Preview a press on a button shows its Down look.
        if self.watch_preview_button(self.view_map().to_doc(ev.position), true) && self.preview_button.is_some() {
            cx.notify();
            return;
        }
        // The comparison views show finished images, so dragging only pans them.
        if self.view_mode != ViewMode::Original {
            self.drag = Some(Drag::Pan { last: ev.position });
            cx.notify();
            return;
        }
        let p = self.view_map().to_doc(ev.position);
        let tool = self.active_tool();
        if self.guide_down(ev.position, p, tool, ev.click_count) {
            cx.notify();
            return;
        }
        if self.text_edit.is_some() {
            self.finish_text();
            if tool != Tool::Text {
                cx.notify();
                return;
            }
        }
        match tool {
            Tool::SelectBehind => self.select_behind(p),
            Tool::PolygonHotspot | Tool::PolygonSlice => self.polygon_web_down(ev, p),
            Tool::CircleHotspot => {
                let start = self.guide_snapped(p);
                self.drag = Some(Drag::Create { start, current: start });
            }
            Tool::RedrawPath => self.drag = Some(Drag::Freehand { points: vec![p], target: None }),
            Tool::ReshapeArea => self.reshape_down(p),
            Tool::ScrubAdd | Tool::ScrubSubtract => self.scrub_down(p, tool == Tool::ScrubAdd),
            Tool::NineSlice => self.nine_slice_down(ev, p),
            Tool::PolygonLasso => self.polygon_lasso_down(ev, p),
            Tool::RedEye => match self.pixel_target(p) {
                Some(target) => self.drag = Some(Drag::PixelArea { target, points: vec![p], mode: MaskMode::Replace }),
                None => self.status = t("Red Eye Removal works on bitmaps.").into(),
            },
            Tool::Sharpen | Tool::Dodge | Tool::Burn | Tool::Smudge | Tool::ReplaceColor => match self.ed.doc.page().bitmap_at(p, &self.ed.selection) {
                Some(id) => self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) }),
                None => self.status = t("This tool works on bitmaps. Open or import an image first.").into(),
            },
            Tool::VectorPath => self.drag = Some(Drag::Freehand { points: vec![p], target: None }),
            Tool::Polygon | Tool::Gradient => {
                let start = self.guide_snapped(p);
                // A gradient is dragged over the object under the mouse, if nothing is selected yet.
                if tool == Tool::Gradient && self.ed.selection.is_empty() {
                    self.ed.selection = self.ed.doc.page().hit_test(p, self.tolerance()).into_iter().collect();
                }
                self.drag = Some(Drag::Create { start, current: start });
            }
            Tool::Pointer => self.pointer_down(ev, p),
            Tool::Scale | Tool::Skew | Tool::Distort => self.transform_down(ev, p),
            Tool::Subselect => self.subselect_down(p, ev.modifiers.alt, ev.modifiers.shift, window, cx),
            // A press inside the selection drags the marquee, and with Ctrl the pixels in it.
            Tool::Marquee | Tool::OvalMarquee | Tool::Lasso
                if !ev.modifiers.shift && !ev.modifiers.alt && self.pixel_drag_down(p, ev.modifiers.control, false) => {}
            Tool::Marquee | Tool::OvalMarquee | Tool::Lasso => match self.pixel_target(p) {
                Some(target) => {
                    self.drag = Some(Drag::PixelArea { target, points: vec![p], mode: selection_mode(ev.modifiers) });
                }
                None => self.status = t("Selections work on bitmaps. Open or import an image first.").into(),
            },
            Tool::Wand => self.wand_click(ev, p),
            Tool::Blur => match self.ed.doc.page().bitmap_at(p, &self.ed.selection) {
                Some(id) => self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) }),
                None => self.status = t("The Blur tool works on bitmaps.").into(),
            },
            Tool::Stamp => self.stamp_down(ev, p),
            Tool::Freeform => self.freeform_down(p),
            Tool::Rect | Tool::Ellipse | Tool::Line | Tool::Knife | Tool::Hotspot | Tool::Slice => {
                let start = self.guide_snapped(p);
                self.drag = Some(Drag::Create { start, current: start });
            }
            Tool::Pen => self.pen_down(p, ev.click_count, window, cx),
            Tool::Pencil | Tool::Brush => {
                let target = self.ed.doc.page().bitmap_at(p, &self.ed.selection);
                self.drag = Some(Drag::Freehand { points: vec![p], target });
            }
            Tool::Eraser => match self.ed.doc.page().bitmap_at(p, &self.ed.selection) {
                Some(id) => self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) }),
                None => self.status = t("The Eraser works on bitmaps. Open or import an image first.").into(),
            },
            Tool::Crop | Tool::ExportArea => {
                if ev.click_count >= 2 && self.crop_area.is_some() {
                    self.apply_crop(window, cx);
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
        self.prefs.pick_distance.max(1.0) / self.zoom
    }

    /// How near something has to come to snap, in pixels of the page.
    fn snap_reach(&self) -> f32 {
        self.prefs.snap_distance.max(1.0) / self.zoom
    }

    /// Starts dragging the pixel selection if `p` is inside it.
    fn pixel_drag_down(&mut self, p: Vec2, pixels: bool, copy: bool) -> bool {
        let Some(selection) = self.active_pixels() else { return false };
        let (target, mask) = (selection.object, selection.mask.clone());
        let Some(object) = self.ed.object(target) else { return false };
        let Shape::Image { bitmap } = &object.shape else { return false };
        if !object.pixel_at(p).is_some_and(|(x, y)| mask.is_set(x, y)) {
            return false;
        }
        let Some((piece, left, top)) = bitmap.extract(&mask) else { return false };
        self.drag = Some(Drag::Pixels { target, before: bitmap.clone(), piece, at: (left as i64, top as i64), mask, start: p, pixels, copy });
        true
    }

    /// Moves the pixel selection, and the pixels in it if `pixels` is set, by whole pixels of its bitmap.
    #[allow(clippy::too_many_arguments)]
    fn move_pixel_selection(&mut self, target: u64, before: &Arc<Bitmap>, piece: &Bitmap, at: (i64, i64), mask: &Arc<Mask>, by: (i64, i64), pixels: bool, copy: bool) {
        if pixels {
            let mut moved = (**before).clone();
            if !copy {
                moved.clear(mask);
            }
            moved.overlay(piece, at.0 + by.0, at.1 + by.1);
            if let Some(object) = self.ed.object_mut(target) {
                object.shape = Shape::Image { bitmap: Arc::new(moved) };
            }
        }
        let shifted = mask.shifted(by.0, by.1);
        self.pixel_sel = Some(PixelSelection { object: target, edges: Arc::new(shifted.edges()), mask: Arc::new(shifted) });
    }

    /// A press on a ruler pulls a new guide out of it, and a press with the Pointer on a guide takes hold of that one.
    fn guide_down(&mut self, at: Point<Pixels>, p: Vec2, tool: Tool, clicks: usize) -> bool {
        if !self.show_guides {
            return false;
        }
        let bounds = self.canvas_bounds.get();
        let ruler = px(RULER_SIZE);
        if self.show_rulers && bounds.contains(&at) && (at.y < bounds.origin.y + ruler || at.x < bounds.origin.x + ruler) {
            // The ruler along the top gives guides that run across; the one down the side, guides that run down.
            let horizontal = at.y < bounds.origin.y + ruler;
            self.drag = Some(Drag::Guide { horizontal, index: None, at: if horizontal { p.y } else { p.x }.round() });
            return true;
        }
        if tool != Tool::Pointer || self.lock_guides {
            return false;
        }
        let Some((horizontal, index)) = self.ed.doc.page().guides.near(p, 3.0 / self.zoom) else { return false };
        // A double-click on a guide asks for its place in numbers.
        if clicks >= 2 {
            let guides = &self.ed.doc.page().guides;
            let place = if horizontal { guides.horizontal[index] } else { guides.vertical[index] };
            self.guide_dialog = Some((horizontal, index, place));
            return true;
        }
        self.drag = Some(Drag::Guide { horizontal, index: Some(index), at: if horizontal { p.y } else { p.x }.round() });
        true
    }

    /// Every line to draw across the view: the page's guides, the one being dragged,
    /// the guides along the edges of the slices, and the smart guides of the move under way.
    pub fn guide_lines(&self) -> Vec<(bool, f32, u32)> {
        let guide_color = (self.guide_color.r as u32) << 16 | (self.guide_color.g as u32) << 8 | self.guide_color.b as u32;
        #[allow(non_snake_case)]
        let GUIDE = guide_color;
        const SLICE_GUIDE: u32 = 0xff4040;
        const SMART_GUIDE: u32 = 0xff00ff;
        let page = self.ed.doc.page();
        let mut lines = Vec::new();
        if self.show_slice_guides {
            for slice in page.objects().filter(|o| o.visible && matches!(o.shape, Shape::Web { kind: WebKind::Slice, .. })) {
                let b = slice.bounds;
                lines.extend([(false, b.x, SLICE_GUIDE), (false, b.right(), SLICE_GUIDE), (true, b.y, SLICE_GUIDE), (true, b.bottom(), SLICE_GUIDE)]);
            }
        }
        if self.show_guides {
            // The guide being dragged is drawn where the mouse has it, not where it was.
            let held = match self.drag {
                Some(Drag::Guide { horizontal, index, at }) => Some((horizontal, index, at)),
                _ => None,
            };
            for horizontal in [true, false] {
                let list = if horizontal { &page.guides.horizontal } else { &page.guides.vertical };
                lines.extend(list.iter().enumerate().filter(|(index, _)| held.is_none_or(|(h, held, _)| h != horizontal || held != Some(*index))).map(|(_, at)| (horizontal, *at, GUIDE)));
            }
            if let Some((horizontal, _, at)) = held {
                lines.push((horizontal, at, GUIDE));
            }
        }
        lines.extend(self.smart_lines.iter().map(|(horizontal, at)| (*horizontal, *at, SMART_GUIDE)));

        lines
    }

    /// A point moved onto a guide, if it is within reach of one and snapping is on.
    fn guide_snapped(&self, p: Vec2) -> Vec2 {
        let p = snap(p);
        // The grid comes first: with Snap to Grid on, a point within reach of a grid line goes onto it.
        let p = if self.show_grid && self.prefs.snap_to_grid {
            let (across, down) = (self.prefs.grid.0.max(1.0), self.prefs.grid.1.max(1.0));
            let near = |v: f32, step: f32| {
                let line = (v / step).round() * step;
                if (line - v).abs() <= self.snap_reach() { line } else { v }
            };
            v2(near(p.x, across), near(p.y, down))
        } else {
            p
        };
        if !(self.show_guides && self.snap_guides) {
            return p;
        }
        let (guides, reach) = (&self.ed.doc.page().guides, self.snap_reach());
        v2(p.x + snap_offset(&[p.x], &guides.vertical, reach).unwrap_or(0.0), p.y + snap_offset(&[p.y], &guides.horizontal, reach).unwrap_or(0.0))
    }

    /// How much further a selection whose box would be `moved` has to go to line up
    /// with a guide or, failing that, with another object; and the smart guides it then lies on.
    fn move_snap(&self, moved: Rect) -> (Vec2, Vec<(bool, f32)>) {
        let (page, reach) = (self.ed.doc.page(), self.snap_reach());
        let (xs, ys) = snap_places(moved);
        let guides = (self.show_guides && self.snap_guides).then_some(&page.guides);
        let smart = self.snap_smart_guides.then(|| page.smart_targets(&self.ed.selection));
        let mut lines = Vec::new();
        let mut along = |places: &[f32; 3], guides: Option<&Vec<f32>>, smart: Option<&Vec<f32>>, horizontal: bool| -> f32 {
            if let Some(offset) = guides.and_then(|guides| snap_offset(places, guides, reach)) {
                return offset;
            }
            let Some(offset) = smart.and_then(|targets| snap_offset(places, targets, reach)) else { return 0.0 };
            // Every place that now lies on a target gets its line drawn.
            let targets = smart.map_or(&[][..], |targets| &targets[..]);
            for place in places.iter().map(|place| place + offset) {
                if targets.iter().any(|target| (target - place).abs() < 0.01) && !lines.contains(&(horizontal, place)) {
                    lines.push((horizontal, place));
                }
            }
            offset
        };
        let dx = along(&xs, guides.map(|g| &g.vertical), smart.as_ref().map(|s| &s.0), false);
        let dy = along(&ys, guides.map(|g| &g.horizontal), smart.as_ref().map(|s| &s.1), true);
        (v2(dx, dy), if self.show_smart_guides { lines } else { Vec::new() })
    }

    fn pointer_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        // The handles of a gradient come first: they may lie outside the object.
        if let Some((start, end, width)) = self.fill_handles() {
            let reach = 6.0 / self.zoom;
            let near = |handle: Vec2| (handle.x - p.x).abs() <= reach && (handle.y - p.y).abs() <= reach;
            if width.is_some_and(near) {
                self.drag = Some(Drag::FillHandle { end: false, width: true });
                return;
            }
            if near(end) || near(start) {
                self.drag = Some(Drag::FillHandle { end: near(end), width: false });
                return;
            }
        }
        // A shape hint is dragged along the outline of its shape.
        let reach = 7.0 / self.zoom;
        if let Some((_, number, end)) = self.shape_hints().into_iter().rev().find(|(at, _, _)| (at.x - p.x).abs() <= reach && (at.y - p.y).abs() <= reach) {
            self.drag = Some(Drag::Hint { number, end });
            return;
        }
        // The red end of an animation's path sets how far and which way the animation goes.
        if let Some(end) = self.motion_path().last().copied().filter(|_| self.motion_path().len() > 1) {
            let reach = 6.0 / self.zoom;
            if (end.x - p.x).abs() <= reach && (end.y - p.y).abs() <= reach {
                self.drag = Some(Drag::MotionEnd);
                return;
            }
            // And the green start moves the place it sets out from, leaving the end where it is.
            if let Some(start) = self.motion_path().first().copied().filter(|start| (start.x - p.x).abs() <= reach && (start.y - p.y).abs() <= reach) {
                self.drag = Some(Drag::MotionStart { end, last: start });
                return;
            }
        }
        // Inside a pixel selection the Pointer moves the pixels, and Alt leaves a copy behind.
        if self.tool == Tool::Pointer && self.pixel_drag_down(p, true, ev.modifiers.alt) {
            return;
        }
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
            // A double-click on a copy of a symbol opens the symbol where it lies.
            Some(id) if ev.click_count >= 2 && self.ed.object(id).is_some_and(|o| matches!(o.shape, Shape::Instance { .. })) => {
                self.ed.selection = vec![id];
                self.toggle_symbol_edit();
            }
            // In PisiAnimasyon a double-click on a group goes inside it, as in Flash: its parts can then be picked and
            // changed one by one, and a click beside them puts the group together again.
            Some(id) if ev.click_count >= 2 && crate::theme::is_sahne() && self.ed.object(id).is_some_and(|o| matches!(o.shape, Shape::Group { .. })) => {
                self.ed.selection = vec![id];
                if self.ed.subselect() {
                    if let Some(part) = self.ed.doc.page().hit_test(p, self.tolerance()) {
                        self.ed.selection = vec![part];
                    }
                    self.status = t("Inside the group: pick a part to change it. A click beside the group closes it").into();
                }
            }
            // And one beside the symbol being edited ends the edit.
            None if ev.click_count >= 2 && self.ed.editing_symbol().is_some() => self.toggle_symbol_edit(),
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
                self.drag = Some(Drag::Move { start: p, origin: self.ed.selection_bounds(), moved: Vec2::default() });
            }
            None => {
                if !ev.modifiers.shift {
                    self.ed.selection.clear();
                }
                self.drag = Some(Drag::Marquee { start: p, current: p });
            }
        }
    }

    /// A press with the Scale, Skew or Distort tool.
    fn transform_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        // A session belongs to the selection it began with.
        let selection: Vec<u64> = self.ed.selected().iter().map(|o| o.id).collect();
        if self.transform.as_ref().is_some_and(|s| s.originals.iter().map(|o| o.id).ne(selection.iter().copied())) {
            self.apply_transform();
        }
        let Some((quad, pivot)) = self.transform_box() else { return self.pointer_down(ev, p) };
        let reach = TRANSFORM_REACH / self.zoom;
        let near = |q: Vec2| (q.x - p.x).abs() <= reach && (q.y - p.y).abs() <= reach;
        let inside = crate::model::point_in_polygon(p, &quad);
        let handle = quad_handles(quad).iter().position(|h| near(*h));
        let (kind, handle) = if near(pivot) {
            (TransformKind::Pivot, 0)
        } else if let Some(handle) = handle {
            // Handles run clockwise from the top-left, so the corners are the even ones.
            let corner = handle % 2 == 0;
            let kind = match self.tool {
                Tool::Skew if corner => TransformKind::Perspective,
                Tool::Skew => TransformKind::Skew,
                Tool::Distort if corner => TransformKind::Distort,
                Tool::Distort => TransformKind::DistortSide,
                _ => TransformKind::Scale,
            };
            (kind, handle)
        } else if inside {
            if ev.click_count >= 2 {
                self.apply_transform();
                return;
            }
            (TransformKind::Move, 0)
        } else if ev.click_count >= 2 && self.ed.editing_symbol().is_some() && self.ed.doc.page().hit_test(p, self.tolerance()).is_none() {
            // A double-click on an empty place ends the editing of a symbol with these tools too: what was
            // being transformed is applied first.
            self.toggle_symbol_edit();
            return;
        } else if self.transform.is_none() && self.ed.doc.page().hit_test(p, self.tolerance()).is_some() {
            // Before a transform has begun, a press on other artwork still selects it.
            return self.pointer_down(ev, p);
        } else {
            (TransformKind::Rotate, 0)
        };
        if self.transform.is_none() {
            let Some(origin) = self.ed.selection_bounds() else { return };
            let originals = self.ed.selected().into_iter().cloned().collect();
            let pivot = self.own_pivot();
            self.transform = Some(TransformSession { origin, warp: Warp::identity(), originals, pivot });
        }
        let from = self.transform.as_ref().map_or(Warp::identity(), |s| s.warp);
        self.drag = Some(Drag::Transform { kind, handle, start: p, from, pivot });
    }

    /// Tells the user what a transform had to leave alone.
    fn report_transform(&mut self, skipped: usize) {
        if skipped > 0 {
            self.status = tf("{} web object(s) could not be transformed", &[&skipped]);
        }
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
        if self.tool == Tool::RedEye {
            let [a, b] = pixels.as_slice() else { return };
            let (a, b) = (*a, *b);
            let mut changed = 0;
            if let Some(object) = self.ed.object_mut(target) {
                let (tolerance, strength) = (self.tool_options.red_tolerance / 100.0, self.tool_options.red_strength / 100.0);
                object.edit_pixels(None, |bitmap| changed = bitmap.remove_red_eye(a, b, tolerance, strength));
            }
            if changed > 0 {
                self.ed.commit(t("Red Eye Removal"));
            } else {
                self.status = t("No red eye was found in that area").into();
            }
            return;
        }
        let mask = match (self.tool, pixels.as_slice()) {
            (Tool::Marquee, [a, b]) => Some(Mask::rect(w, h, *a, *b)),
            (Tool::OvalMarquee, [a, b]) => Some(Mask::polygon(w, h, &oval_points(*a, *b))),
            (Tool::PolygonLasso, [_, _, _, ..]) => Some(Mask::polygon(w, h, &pixels)),
            (Tool::Lasso, [_, _, _, ..]) => Some(Mask::polygon(w, h, &pixels)),
            _ => None,
        };
        self.set_pixel_selection(target, mask.filter(|m| !m.is_empty()), mode);
    }

    /// A click with the Polygon Lasso: the first starts it, each further one adds a
    /// corner, and a double-click or a click back on the first corner closes it into a selection.
    fn polygon_lasso_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        let Some((target, mut points, mode)) = self.polygon_lasso.take() else {
            match self.pixel_target(p) {
                Some(target) => self.polygon_lasso = Some((target, vec![p], selection_mode(ev.modifiers))),
                None => self.status = t("Selections work on bitmaps. Open or import an image first.").into(),
            }
            return;
        };
        let reach = 6.0 / self.zoom;
        let at_start = points.first().is_some_and(|first| (first.x - p.x).abs() <= reach && (first.y - p.y).abs() <= reach);
        if ev.click_count >= 2 || (at_start && points.len() >= 3) {
            if !at_start && ev.click_count < 2 {
                points.push(p);
            }
            self.finish_pixel_area(target, points, mode);
            return;
        }
        points.push(p);
        self.polygon_lasso = Some((target, points, mode));
    }

    /// Fireworks' Select Behind: each click picks the next object down under the mouse, and after the lowest the topmost again.
    fn select_behind(&mut self, p: Vec2) {
        let hits = self.ed.doc.page().hits_at(p, self.tolerance());
        let current = hits.iter().position(|id| self.ed.is_selected(*id));
        self.ed.selection = match current {
            Some(at) => vec![hits[(at + 1) % hits.len()]],
            None => hits.first().copied().into_iter().collect(),
        };
    }

    fn wand_click(&mut self, ev: &MouseDownEvent, p: Vec2) {
        let Some(id) = self.ed.doc.page().bitmap_at(p, &self.ed.selection) else {
            self.status = t("The Magic Wand works on bitmaps.").into();
            return;
        };
        let Some(object) = self.ed.object(id) else { return };
        let region = object.pixel_at(p).and_then(|(x, y)| object.bitmap()?.region(x, y, self.tool_options.wand_tolerance as u8));
        self.set_pixel_selection(id, region, selection_mode(ev.modifiers));
    }

    fn stamp_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        let Some(id) = self.ed.doc.page().bitmap_at(p, &self.ed.selection) else {
            self.status = t("The Rubber Stamp works on bitmaps.").into();
            return;
        };
        if ev.modifiers.alt {
            self.stamp = Some(Stamp { object: id, source: p, offset: None });
            self.status = t("Clone source set. Drag to paint.").into();
            return;
        }
        match self.stamp.as_mut().filter(|s| s.object == id) {
            Some(stamp) => {
                let source = stamp.source;
                stamp.offset.get_or_insert(v2(source.x - p.x, source.y - p.y));
                self.drag = Some(Drag::Freehand { points: vec![p], target: Some(id) });
            }
            None => self.status = t("Alt+click this image to set the clone source first.").into(),
        }
    }

    fn freeform_down(&mut self, p: Vec2) {
        let radius = self.freeform_size / 2.0;
        let Some(id) = self.ed.doc.page().nearest_path(p, radius, &self.ed.selection) else {
            self.status = t("Press near a vector path or shape to reshape it.").into();
            return;
        };
        let Some(object) = self.ed.object_mut(id) else { return };
        if !object.to_path() {
            return;
        }
        object.flatten();
        let base = subdivide(&object.points, object.is_closed(), (radius * FREEFORM_DETAIL).max(1.0));
        object.points = base.clone();
        self.ed.selection = vec![id];
        self.drag = Some(Drag::Freeform { id, start: p, base, radius });
    }

    /// The picked vertices that still exist, on paths that are still selected.
    pub fn picked_points(&self) -> Vec<(u64, usize)> {
        self.point_selection
            .iter()
            .copied()
            .filter(|(id, index)| self.ed.is_selected(*id) && self.ed.object(*id).is_some_and(|o| *index < o.points.len()))
            .collect()
    }

    /// The curve handles on show: those of each picked vertex, and the one handle on
    /// either neighbour that shapes a segment it shares. Each is its path, its vertex,
    /// whether it is the outgoing handle, the vertex and the handle's own position.
    pub fn visible_handles(&self) -> Vec<(u64, usize, bool, Vec2, Vec2)> {
        let mut out: Vec<(u64, usize, bool, Vec2, Vec2)> = Vec::new();
        let mut add = |object: &Object, index: usize, after: bool| {
            let handles = object.handles_at(index);
            let offset = if after { handles.after } else { handles.before };
            if offset != Vec2::default() && !out.iter().any(|h| (h.0, h.1, h.2) == (object.id, index, after)) {
                let anchor = object.points[index];
                out.push((object.id, index, after, anchor, v2(anchor.x + offset.x, anchor.y + offset.y)));
            }
        };
        for (id, index) in self.picked_points() {
            let Some(object) = self.ed.object(id) else { continue };
            let count = object.points.len();
            add(object, index, false);
            add(object, index, true);
            if index > 0 || object.is_closed() {
                add(object, (index + count - 1) % count, true);
            }
            if index + 1 < count || object.is_closed() {
                add(object, (index + 1) % count, false);
            }
        }
        out
    }

    /// The vertex of a selected path under `p`. An ellipse or an auto shape shows the
    /// vertices it would have as a path, and taking one makes it that path. A rectangle
    /// shows them too but is left as it is, and `None` is returned after asking the
    /// user whether to ungroup it, as Fireworks does.
    fn path_point_at(&mut self, p: Vec2, window: &mut Window, cx: &mut Context<Self>) -> Option<(u64, usize)> {
        let reach = TRANSFORM_REACH / self.zoom;
        let (id, index, rectangle) = self.ed.selected().into_iter().find_map(|object| {
            let index = object.path_points().iter().position(|v| (v.x - p.x).abs() <= reach && (v.y - p.y).abs() <= reach)?;
            Some((object.id, index, matches!(object.shape, Shape::Rect { .. })))
        })?;
        if rectangle {
            self.confirm_rectangle_to_path(id, window, cx);
            return None;
        }
        if let Some(object) = self.ed.object_mut(id) {
            object.to_path();
        }
        Some((id, index))
    }

    /// Paste as Mask, with the two questions Fireworks asks on the way: whether to
    /// replace a mask that is already there or add to it, and whether vectors are to
    /// become a bitmap mask or stay vectors.
    fn paste_as_mask_asking(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let existing = self.ed.selected().iter().any(|o| o.mask.is_some());
        let vectors = self.ed.clipboard_vectors();
        cx.spawn_in(window, async move |this, cx| {
            // Shows one question and waits for its answer: the number of the button, or None for Cancel.
            let ask = async |question: String, first: &'static str, second: &'static str, cx: &mut gpui::AsyncWindowContext| {
                let asked = this.update_in(cx, |this, window, cx| {
                    this.awaiting_answer = true;
                    window.prompt(PromptLevel::Info, &question, None, &[first, second, t("Cancel")], cx)
                });
                let answer = match asked {
                    Ok(answer) => answer.await.ok(),
                    Err(_) => None,
                };
                this.update(cx, |this, _| this.awaiting_answer = false).ok();
                answer.filter(|choice| *choice < 2)
            };
            let mut add = false;
            if existing {
                let question = t("A mask already exists. Do you want to replace the mask or add to the existing mask?").to_string();
                match ask(question, t("Replace"), t("Add"), cx).await {
                    Some(choice) => add = choice == 1,
                    None => return,
                }
            }
            let mut as_bitmap = false;
            if vectors > 0 {
                let (question, keep) = if vectors == 1 {
                    (t("You are pasting a vector as a mask. Do you want to make a bitmap mask or create a mask that will remain a vector object?"), t("Vector"))
                } else {
                    (t("You are pasting multiple objects as a mask. Do you want to make a bitmap mask or create a mask containing editable vectors?"), t("Vectors"))
                };
                match ask(question.to_string(), t("Bitmap"), keep, cx).await {
                    Some(choice) => as_bitmap = choice == 0,
                    None => return,
                }
            }
            this.update_in(cx, |this, _, cx| {
                if this.ed.paste_as_mask(as_bitmap, add) {
                    this.ed.commit(t("Paste as Mask"));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Asks, as Fireworks does, whether a mask is to be applied to its object before it is removed.
    fn confirm_delete_mask(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let question = format!(
            "{} {}",
            t("Apply mask before removing?"),
            t("Applying a mask converts the object and mask into a single bitmap image.")
        );
        let answer = window.prompt(PromptLevel::Warning, &question, None, &[t("Apply"), t("Discard"), t("Cancel")], cx);
        self.awaiting_answer = true;
        cx.spawn_in(window, async move |this, cx| {
            let answer = answer.await;
            this.update_in(cx, |this, _, cx| {
                this.awaiting_answer = false;
                if let Ok(choice @ (0 | 1)) = answer {
                    this.ed.delete_masks(choice == 0);
                    this.ed.commit(t("Delete Mask"));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Starts editing the mask of the object with this id.
    pub fn edit_mask_of(&mut self, id: u64) {
        self.finish_pending();
        self.ed.selection = vec![id];
        if self.ed.edit_mask() {
            self.status = t("Editing the mask. Select something else to finish.").into();
        }
    }

    /// The mask of the selected object, for the mask settings in Properties.
    pub fn current_mask(&self) -> Option<Arc<crate::model::ObjectMask>> {
        self.ed.selected().first().and_then(|o| o.mask.clone())
    }

    /// Changes the masks of the selected objects.
    pub fn change_masks(&mut self, change: impl Fn(&mut crate::model::ObjectMask)) {
        self.ed.for_selected(|o| {
            if let Some(mask) = &mut o.mask {
                change(Arc::make_mut(mask));
            }
        });
        self.ed.commit(t("Mask"));
    }

    /// Asks before a rectangle gives up its roundness setting to have its vertices edited.
    fn confirm_rectangle_to_path(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        // One text, because the prompt wraps its message but cuts a long detail line short.
        let question = format!("{} {}", t("To edit a rectangle's points, it must be ungrouped."), t("OK to ungroup and turn into a vector?"));
        let answer = window.prompt(PromptLevel::Warning, &question, None, &[t("OK"), t("Cancel")], cx);
        self.awaiting_answer = true;
        cx.spawn_in(window, async move |this, cx| {
            let agreed = answer.await == Ok(0);
            this.update_in(cx, |this, _, cx| {
                this.awaiting_answer = false;
                if let Some(object) = this.ed.object_mut(id).filter(|_| agreed) {
                    object.to_path();
                    this.ed.commit(t("Convert to Path"));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Whether a rectangle's vertex is under `p`, which stops a press from doing anything else while the user is asked.
    fn rectangle_point_at(&self, p: Vec2) -> bool {
        let reach = TRANSFORM_REACH / self.zoom;
        self.ed.selected().into_iter().any(|object| {
            matches!(object.shape, Shape::Rect { .. }) && object.path_points().iter().any(|v| (v.x - p.x).abs() <= reach && (v.y - p.y).abs() <= reach)
        })
    }

    /// A press with Subselection: on a curve handle or a vertex of a selected path it
    /// starts dragging it, and anywhere else it selects what is there. Shift adds a
    /// vertex to those picked, and Alt pulls a new pair of handles out of one.
    fn subselect_down(&mut self, p: Vec2, alt: bool, shift: bool, window: &mut Window, cx: &mut Context<Self>) {
        // A click on a group picks the member under the mouse out of it, as in Fireworks.
        let hit = self.ed.doc.page().hit_test(p, self.tolerance());
        if let Some(group) = hit.filter(|id| self.ed.object(*id).is_some_and(|o| matches!(o.shape, Shape::Group { .. }))) {
            self.ed.selection = vec![group];
            if self.ed.subselect() {
                let member = self.ed.doc.page().hit_test(p, self.tolerance()).filter(|id| self.ed.is_selected(*id));
                if let Some(member) = member {
                    self.ed.selection = vec![member];
                }
                self.point_selection.clear();
                self.ed.commit(t("Subselect"));
                return;
            }
        }
        let reach = TRANSFORM_REACH / self.zoom;
        let near = |v: Vec2| ((v.x - p.x).powi(2) + (v.y - p.y).powi(2)).sqrt() <= reach;
        // Handles first: a short one sits almost on top of its vertex.
        if let Some((id, index, after, ..)) = self.visible_handles().into_iter().find(|h| near(h.4)) {
            self.drag = Some(Drag::Handle { id, index, after, fresh: false, moved: false, pen: false });
            return;
        }
        let on_rectangle = self.rectangle_point_at(p);
        if let Some((id, index)) = self.path_point_at(p, window, cx) {
            let picked = self.picked_points().contains(&(id, index));
            if shift && picked {
                self.point_selection.retain(|point| *point != (id, index));
                return;
            }
            if shift {
                self.point_selection.push((id, index));
            } else if !picked {
                self.point_selection = vec![(id, index)];
            }
            self.drag = Some(if alt {
                Drag::Handle { id, index, after: true, fresh: true, moved: false, pen: false }
            } else {
                Drag::Vertex { id, index }
            });
            return;
        }
        if on_rectangle {
            return;
        }
        self.point_selection.clear();
        match self.ed.doc.page().hit_test(p, self.tolerance()) {
            Some(id) => self.ed.selection = vec![id],
            // On nothing, a box is dragged out to pick the vertices inside it.
            None => self.drag = Some(Drag::Marquee { start: p, current: p }),
        }
    }

    /// Ends a box dragged out with Subselection: it picks the vertices of the selected
    /// paths that lie inside, or selects the objects inside when there are none.
    fn finish_point_marquee(&mut self, area: Rect) {
        let picked: Vec<(u64, usize)> = self
            .ed
            .selected()
            .into_iter()
            .filter(|o| matches!(o.shape, Shape::Line | Shape::Path { .. }))
            .flat_map(|o| o.points.iter().enumerate().filter(|(_, p)| area.contains(**p)).map(|(index, _)| (o.id, index)).collect::<Vec<_>>())
            .collect();
        if picked.is_empty() {
            self.ed.select_in_rect(area);
        }
        self.point_selection = picked;
    }

    /// Removes the picked vertices, as Delete does with Subselection. A path left
    /// with fewer than two vertices goes with them.
    fn delete_picked_points(&mut self) -> bool {
        let mut picked = self.picked_points();
        if picked.is_empty() {
            return false;
        }
        // From the far end, so earlier indices stay good.
        picked.sort_by(|a, b| b.cmp(a));
        let mut emptied = Vec::new();
        for (id, index) in picked {
            if let Some(object) = self.ed.object_mut(id) {
                object.to_path();
                if !object.remove_point(index) && !emptied.contains(&id) {
                    emptied.push(id);
                }
            }
        }
        if !emptied.is_empty() {
            self.ed.selection = emptied;
            self.ed.delete_selected();
        }
        self.point_selection.clear();
        self.ed.commit(t("Edit Path"));
        true
    }

    /// A press with the Pen. On a selected path it edits the path: on a vertex it
    /// waits to see whether handles are pulled out, and on a segment it adds a vertex.
    /// Anywhere else it places the next vertex of the path being drawn.
    fn pen_down(&mut self, p: Vec2, clicks: usize, window: &mut Window, cx: &mut Context<Self>) {
        let reach = TRANSFORM_REACH / self.zoom;
        if self.pen.is_empty() {
            if self.rectangle_point_at(p) {
                self.path_point_at(p, window, cx);
                return;
            }
            if let Some((id, index)) = self.path_point_at(p, window, cx) {
                self.drag = Some(Drag::Handle { id, index, after: true, fresh: true, moved: false, pen: true });
                return;
            }
            let on_segment = self.ed.selected().into_iter().filter(|o| matches!(o.shape, Shape::Line | Shape::Path { .. })).find_map(|object| {
                object.nearest_segment(p, reach).map(|(segment, along)| (object.id, segment, along))
            });
            if let Some((id, segment, along)) = on_segment {
                if let Some(object) = self.ed.object_mut(id) {
                    object.to_path();
                    let index = object.insert_point(segment, along);
                    self.point_selection = vec![(id, index)];
                    self.ed.commit(t("Edit Path"));
                }
                return;
            }
        } else {
            // Ending on the loose end of another open path joins the two.
            let near = |v: &Vec2| (v.x - p.x).abs() <= reach && (v.y - p.y).abs() <= reach;
            let end = self
                .ed
                .selected()
                .into_iter()
                .filter(|o| Some(o.id) != self.pen_resume && matches!(o.shape, Shape::Line | Shape::Path { closed: false }))
                .find_map(|o| [0, o.points.len().saturating_sub(1)].into_iter().find(|i| o.points.get(*i).is_some_and(near)).map(|i| (o.clone(), i)));
            if let Some((other, index)) = end {
                let id = other.id;
                let (mut points, mut handles) = (other.points.clone(), other.handles.clone());
                handles.resize(points.len(), Handles::default());
                if index != 0 {
                    points.reverse();
                    handles.reverse();
                    handles.iter_mut().for_each(|h| *h = Handles { before: h.after, after: h.before });
                }
                self.pen.extend(points);
                self.pen_handles.extend(handles);
                self.ed.selection = vec![id];
                self.ed.delete_selected();
                self.finish_pen(false);
                return;
            }
        }
        self.pen_click(p, clicks);
    }

    /// A click of the Pen on a vertex of a selected path, with nothing pulled out of it.
    /// The loose end of an open path is picked up to draw on from; a curve point
    /// becomes a corner; a corner is removed.
    fn pen_point_click(&mut self, id: u64, index: usize) {
        let Some(object) = self.ed.object(id) else { return };
        let count = object.points.len();
        if !object.is_closed() && (index == 0 || index + 1 == count) {
            let (mut points, mut handles) = (object.points.clone(), object.handles.clone());
            handles.resize(count, Handles::default());
            // The Pen adds to the end of its list, so the picked end has to be last.
            if index == 0 {
                points.reverse();
                handles.reverse();
                handles.iter_mut().for_each(|h| *h = Handles { before: h.after, after: h.before });
            }
            (self.pen, self.pen_handles, self.pen_resume) = (points, handles, Some(id));
            return;
        }
        let curved = !object.handles_at(index).is_corner();
        let Some(object) = self.ed.object_mut(id) else { return };
        object.to_path();
        if curved {
            object.set_handles(index, Handles::default());
            object.refit_to_points();
        } else {
            object.remove_point(index);
            self.point_selection.clear();
        }
        self.ed.commit(t("Edit Path"));
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
                self.status = tf("Picked {}", &[&color.hex()]);
            }
            return;
        }
        if let Some(color) = object.fill.or(object.stroke.map(|s| s.color)) {
            self.fill = Some(color);
            self.status = tf("Picked {}", &[&color.hex()]);
        }
    }

    fn bucket(&mut self, p: Vec2) {
        let fill = self.fill;
        let Some(id) = self.ed.doc.page().art_at(p, self.tolerance()) else { return };
        if self.ed.object(id).is_some_and(Object::is_image) {
            let Some(color) = fill else {
                self.status = t("Choose a fill color first").into();
                return;
            };
            let mask = self.mask_for(id);
            let tolerance = self.tool_options.bucket_tolerance as u8;
            let Some(object) = self.ed.object_mut(id) else { return };
            let Some((x, y)) = object.pixel_at(p) else { return };
            object.edit_pixels(mask.as_deref(), |bitmap| {
                bitmap.flood_fill(x, y, color, tolerance);
            });
            self.ed.commit(t("Paint Bucket"));
            return;
        }
        if let Some(object) = self.ed.object_mut(id) {
            object.fill = fill;
        }
        self.ed.commit(t("Paint Bucket"));
    }

    fn new_object(&mut self, name: &str, shape: Shape, bounds: Rect, points: Vec<Vec2>) -> Object {
        let is_stroke_only = matches!(shape, Shape::Line | Shape::Path { closed: false } | Shape::Auto { kind: AutoKind::Spiral, .. });
        Object {
            id: self.ed.alloc_id(),
            name: name.to_string(),
            shape,
            bounds,
            points,
            handles: Vec::new(),
            placement: None,
            pivot: None,

            behaviors: Vec::new(),
            attached: String::new(),
            blend: crate::blend::BlendMode::Normal,
            mask: None,
            mask_of: None,
            fill_style: None,
            stroke_profile: Vec::new(),
            fill_look: Default::default(),
            link: String::new(),
            filters: Vec::new(),
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
            self.status = t("All layers are locked").into();
            self.ed.rollback();
        }
    }

    fn pen_click(&mut self, p: Vec2, clicks: usize) {
        let p = self.guide_snapped(p);
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
        self.pen_handles.push(Handles::default());
        // Dragging before the button is released turns the corner into a curve.
        self.drag = Some(Drag::PenHandle);
    }

    pub fn finish_pen(&mut self, closed: bool) {
        let points = std::mem::take(&mut self.pen);
        let handles = std::mem::take(&mut self.pen_handles);
        if points.len() < 2 {
            self.pen_resume = None;
            return;
        }
        // A path that was picked up by its loose end is carried on, not drawn again.
        if let Some(object) = self.pen_resume.take().and_then(|id| self.ed.object_mut(id)) {
            object.shape = Shape::Path { closed };
            (object.points, object.handles) = (points, handles);
            if !object.is_curved() {
                object.handles.clear();
            }
            object.refit_to_points();
            self.ed.commit(t("Pen"));
            return;
        }
        let bounds = Rect::of_points(&flatten_curve(&points, &handles, closed));
        let mut object = self.new_object(t("Path"), Shape::Path { closed }, bounds, points);
        object.handles = handles;
        if !object.is_curved() {
            object.handles.clear();
        }
        self.add_new(object, t("Pen"));
    }

    fn begin_text(&mut self, p: Vec2) {
        let size = self.text_size;
        let (w, h) = measure_text("", size);
        let bounds = Rect { x: p.x, y: p.y, w, h };
        let mut object = self.new_object(t("Text"), Shape::Text { content: String::new(), size, style: self.text_style.clone() }, bounds, Vec::new());
        object.fill = self.stroke.or(self.fill).or(Some(Color::BLACK));
        object.stroke = None;
        let id = object.id;
        if self.ed.add_object(object) {
            self.text_edit = Some(id);
        } else {
            self.status = t("All layers are locked").into();
        }
    }

    fn edit_text(&mut self, edit: impl FnOnce(&mut String)) {
        let Some(id) = self.text_edit else { return };
        let Some(object) = self.ed.object_mut(id) else { return };
        if let Shape::Text { content, .. } = &mut object.shape {
            edit(content);
            object.refit_text();
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
            self.ed.commit(t("Text"));
        }
    }

    pub fn mouse_move(&mut self, ev: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let map = self.view_map();
        let p = map.to_doc(ev.position);
        self.cursor_doc = p;
        self.cursor_screen = ev.position;
        if self.drag.is_none() && self.watch_preview_button(p, ev.pressed_button == Some(gpui::MouseButton::Left)) {
            cx.notify();
        }
        if self.filter_drag.is_some() {
            self.drag_filter(ev.position);
            cx.notify();
            return;
        }
        if matches!(self.drag, Some(Drag::Scrub { .. })) {
            self.scrub_at(p);
            cx.notify();
            return;
        }
        // Where the mouse is once guides have had their say, for the drags that draw and resize.
        let on_guides = self.guide_snapped(p);
        if let Some(Drag::Move { start, origin, moved }) = self.drag {
            let free = v2((p.x - start.x).round(), (p.y - start.y).round());
            let (extra, lines) = origin.map_or((Vec2::default(), Vec::new()), |origin| self.move_snap(Rect { x: origin.x + free.x, y: origin.y + free.y, ..origin }));
            let total = v2(free.x + extra.x, free.y + extra.y);
            let (dx, dy) = (total.x - moved.x, total.y - moved.y);
            if dx != 0.0 || dy != 0.0 {
                self.ed.for_selected(|o| o.translate(dx, dy));
            }
            self.smart_lines = lines;
            self.drag = Some(Drag::Move { start, origin, moved: total });
            cx.notify();
            return;
        }
        let Some(drag) = self.drag.as_mut() else {
            if !self.pen.is_empty() || self.panel_showing(Panel::Info) || self.panel_drag.is_some() || self.polygon_lasso.is_some() || self.polygon_web.is_some() {
                cx.notify();
            }
            return;
        };
        match drag {
            Drag::Move { .. } => {}
            Drag::Guide { horizontal, at, .. } => *at = if *horizontal { p.y } else { p.x }.round(),
            Drag::Scrub { .. } => {}
            Drag::Reshape { ids, last, radius } => {
                let (delta, from, radius) = (v2(p.x - last.x, p.y - last.y), *last, *radius);
                *last = p;
                for id in ids.clone() {
                    if let Some(object) = self.ed.object_mut(id) {
                        object.points = pull_points(&object.points, from, delta, radius);
                        object.refit_to_points();
                    }
                }
            }
            Drag::NineSlice { handle, origin, originals } => {
                let (target, origin, guides) = (resized(*origin, *handle, on_guides), *origin, self.nine_guides);
                for original in originals.clone() {
                    let mut scaled = original.clone();
                    scaled.nine_slice(origin, target, guides);
                    if let Some(object) = self.ed.object_mut(original.id) {
                        *object = scaled;
                    }
                }
            }
            Drag::NineGuide(index) => {
                let index = *index;
                if let Some(b) = self.ed.selection_bounds() {
                    let part = if index < 2 { (p.x - b.x) / b.w.max(1.0) } else { (p.y - b.y) / b.h.max(1.0) };
                    self.nine_guides[index] = part.clamp(0.0, 1.0);
                }
            }
            Drag::MotionStart { end, last } => {
                let (end, from) = (*end, *last);
                *last = on_guides;
                if let Some(index) = self.ed.selected_animation() {
                    self.ed.shift_animation(index, on_guides.x - from.x, on_guides.y - from.y, false);
                    let mut settings = self.ed.doc.page().animations[index].settings;
                    let (dx, dy) = (end.x - on_guides.x, end.y - on_guides.y);
                    settings.distance = (dx * dx + dy * dy).sqrt().round();
                    settings.direction = (-dy).atan2(dx).to_degrees().rem_euclid(360.0).round();
                    self.ed.set_animation(settings);
                }
            }
            Drag::Hint { number, end } => {
                let (number, end) = (*number, *end);
                self.move_hint(number, end, p);
            }
            Drag::MotionEnd => {
                if let (Some(index), Some(start)) = (self.ed.selected_animation(), self.motion_path().first().copied()) {
                    let mut settings = self.ed.doc.page().animations[index].settings;
                    let (dx, dy) = (on_guides.x - start.x, on_guides.y - start.y);
                    settings.distance = (dx * dx + dy * dy).sqrt().round();
                    settings.direction = (-dy).atan2(dx).to_degrees().rem_euclid(360.0).round();
                    self.ed.set_animation(settings);
                }
            }
            Drag::FillHandle { end, width } => {
                let (end, width) = (*end, *width);
                self.ed.for_selected(|o| {
                    let bounds = o.bounds;
                    if let Some(style) = o.fill_style.as_mut() {
                        match width {
                            true => Arc::make_mut(style).set_width_handle(on_guides, bounds),
                            false => Arc::make_mut(style).set_handle(end, on_guides, bounds),
                        }
                    }
                });
            }
            Drag::Resize { handle, origin } => {
                let target = resized(*origin, *handle, on_guides);
                self.ed.for_selected(|o| o.set_bounds(target));
            }
            Drag::Marquee { current, .. } => *current = p,
            Drag::Crop { current, .. } => *current = on_guides,
            Drag::PixelArea { points, .. } => {
                if matches!(self.tool, Tool::Marquee | Tool::OvalMarquee | Tool::RedEye) {
                    points.truncate(1);
                    points.push(p);
                } else if points.last().is_none_or(|l| (l.x - p.x).abs() + (l.y - p.y).abs() >= 1.5 / self.zoom) {
                    points.push(p);
                }
            }
            Drag::Transform { kind, handle, start, from, pivot } => {
                let (kind, handle, start, from, pivot) = (*kind, *handle, *start, *from, *pivot);
                let Some(session) = self.transform.as_mut() else { return };
                let origin = session.origin;
                // Handle drags are worked out in the box's own upright space, so a box
                // that has been turned still scales and slants along its own sides.
                let local = from.inverse().map(|inverse| (inverse.apply(start), inverse.apply(p)));
                let (dx, dy) = local.map_or((0.0, 0.0), |(a, b)| (b.x - a.x, b.y - a.y));
                let in_box = |change: Option<Warp>| change.map(|change| change.then(&from));
                let next = match kind {
                    TransformKind::Pivot => {
                        session.pivot = Some(p);
                        None
                    }
                    TransformKind::Move => Some(from.then(&Warp::translation((p.x - start.x).round(), (p.y - start.y).round()))),
                    TransformKind::Rotate => {
                        let angle_to = |q: Vec2| (q.y - pivot.y).atan2(q.x - pivot.x);
                        let mut angle = angle_to(p) - angle_to(start);
                        if ev.modifiers.shift {
                            let step = ROTATE_STEP_DEGREES.to_radians();
                            angle = (angle / step).round() * step;
                        }
                        Some(from.then(&Warp::rotation(pivot, angle)))
                    }
                    TransformKind::Scale => {
                        in_box(local.map(|(_, at)| Warp::fit(origin, scaled(origin, handle, at, !ev.modifiers.shift))))
                    }
                    // The dragged side slides along itself, and the opposite side stays put.
                    TransformKind::Skew => in_box(match handle {
                        1 => (origin.h > 0.0).then(|| Warp::shear_x(origin.bottom(), dx / -origin.h)),
                        5 => (origin.h > 0.0).then(|| Warp::shear_x(origin.y, dx / origin.h)),
                        3 => (origin.w > 0.0).then(|| Warp::shear_y(origin.x, dy / origin.w)),
                        _ => (origin.w > 0.0).then(|| Warp::shear_y(origin.right(), dy / -origin.w)),
                    }),
                    TransformKind::Perspective => {
                        // The neighbour along the dragged direction mirrors the corner,
                        // so that side of the box grows or shrinks about its middle.
                        let mut corners = rect_corners(origin);
                        let corner = handle / 2;
                        if dx.abs() >= dy.abs() {
                            corners[corner].x += dx;
                            corners[[1, 0, 3, 2][corner]].x -= dx;
                        } else {
                            corners[corner].y += dy;
                            corners[[3, 2, 1, 0][corner]].y -= dy;
                        }
                        in_box(Warp::quad(origin, corners))
                    }
                    TransformKind::Distort | TransformKind::DistortSide => {
                        let mut corners = rect_corners(origin).map(|corner| from.apply(corner));
                        let corner = handle / 2;
                        if kind == TransformKind::Distort {
                            corners[corner] = p;
                        } else {
                            for index in [corner, (corner + 1) % 4] {
                                corners[index] = v2(corners[index].x + p.x - start.x, corners[index].y + p.y - start.y);
                            }
                        }
                        Warp::quad(origin, corners)
                    }
                };
                // A box pulled inside out has no sensible result, so the last good one stays.
                let sound = |warp: &Warp| {
                    origin.w <= 0.0 || origin.h <= 0.0 || Warp::quad(origin, rect_corners(origin).map(|c| warp.apply(c))).is_some()
                };
                if let Some(next) = next.filter(sound) {
                    session.warp = next;
                    self.preview_transform();
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
                let raw = on_guides;
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
                // Every picked vertex of the path moves with the one under the pointer.
                let others: Vec<usize> = self.point_selection.iter().filter(|(path, other)| *path == id && *other != index).map(|(_, other)| *other).collect();
                if let Some(object) = self.ed.object_mut(id) {
                    object.to_path();
                    let Some(from) = object.points.get(index).copied() else { return };
                    let to = on_guides;
                    object.points[index] = to;
                    for other in others {
                        if let Some(point) = object.points.get_mut(other) {
                            *point = v2(point.x + to.x - from.x, point.y + to.y - from.y);
                        }
                    }
                    object.refit_to_points();
                }
            }
            Drag::Pixels { target, before, piece, at, mask, start, pixels, copy } => {
                let (target, at, pixels, copy) = (*target, *at, *pixels, *copy);
                let scale = self.ed.object(target).and_then(|o| o.pixel_scale()).unwrap_or((1.0, 1.0));
                let by = (((p.x - start.x) * scale.0).round() as i64, ((p.y - start.y) * scale.1).round() as i64);
                let (before, piece, mask) = (before.clone(), piece.clone(), mask.clone());
                self.move_pixel_selection(target, &before, &piece, at, &mask, by, pixels, copy);
            }
            Drag::PenHandle => {
                if let (Some(anchor), Some(handles)) = (self.pen.last(), self.pen_handles.last_mut()) {
                    let pull = v2(p.x - anchor.x, p.y - anchor.y);
                    // A click always wobbles a little, and that must stay a corner.
                    let dragged = (pull.x.powi(2) + pull.y.powi(2)).sqrt() * self.zoom >= HANDLE_DRAG_START;
                    *handles = if dragged { Handles::smooth(pull) } else { Handles::default() };
                }
            }
            Drag::Handle { id, index, after, fresh, moved, .. } => {
                let (id, index, after) = (*id, *index, *after);
                let Some(object) = self.ed.object_mut(id) else { return };
                let Some(anchor) = object.points.get(index).copied() else { return };
                let pull = v2(p.x - anchor.x, p.y - anchor.y);
                let length = |v: Vec2| (v.x.powi(2) + v.y.powi(2)).sqrt();
                if !*moved && length(pull) * self.zoom < HANDLE_DRAG_START {
                    return;
                }
                *moved = true;
                // A line has no room for handles until it is a path.
                object.to_path();
                let mut handles = object.handles_at(index);
                if *fresh {
                    handles = Handles::smooth(pull);
                } else {
                    let (dragged, other) =
                        if after { (&mut handles.after, &mut handles.before) } else { (&mut handles.before, &mut handles.after) };
                    // A smooth vertex stays smooth: the far handle keeps its length and
                    // turns to face away. Alt lets the two sides part into a cusp.
                    let cross = dragged.x * other.y - dragged.y * other.x;
                    let dot = dragged.x * other.x + dragged.y * other.y;
                    let was_smooth = dot < 0.0 && cross.abs() <= 0.02 * length(*dragged) * length(*other);
                    *dragged = pull;
                    if was_smooth && !ev.modifiers.alt && length(pull) > 0.0 {
                        let scale = -length(*other) / length(pull);
                        *other = v2(pull.x * scale, pull.y * scale);
                    }
                }
                object.set_handles(index, handles);
                object.refit_to_points();
            }
            Drag::Pan { last } => {
                self.pan.x += f32::from(ev.position.x - last.x);
                self.pan.y += f32::from(ev.position.y - last.y);
                *last = ev.position;
            }
        }
        cx.notify();
    }

    /// Keeps track, in Preview, of the button under the mouse. Returns whether anything changed.
    fn watch_preview_button(&mut self, p: Vec2, pressed: bool) -> bool {
        let over = if self.view_mode == ViewMode::Preview {
            let has_looks = |symbol: u64| self.ed.doc.symbols.iter().any(|s| s.id == symbol && s.kind == SymbolKind::Button && s.looks.iter().any(|look| !look.is_empty()));
            let page = self.ed.doc.page();
            let hit = page.objects().filter(|o| o.visible && o.bounds.contains(p) && matches!(o.shape, Shape::Instance { symbol, .. } if has_looks(symbol))).last();
            hit.map(|o| (o.id, if pressed { 2 } else { 1 }))
        } else {
            None
        };
        let changed = over != self.preview_button;
        self.preview_button = over;
        changed
    }

    pub fn mouse_up(&mut self, ev: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.watch_preview_button(self.view_map().to_doc(ev.position), false) {
            cx.notify();
        }
        if self.panel_drag.is_some() {
            self.finish_panel_drag(ev.position);
            cx.notify();
            return;
        }
        if self.filter_drag.is_some() {
            self.finish_filter_drag();
            cx.notify();
            return;
        }
        let Some(drag) = self.drag.take() else { return };
        match drag {
            Drag::Move { moved, .. } => {
                self.smart_lines.clear();
                // A copy of an animation takes the whole animation with it, as in Fireworks.
                if let Some(index) = self.ed.selected_animation().filter(|_| moved.x != 0.0 || moved.y != 0.0) {
                    self.ed.shift_animation(index, moved.x, moved.y, true);
                }
                self.ed.commit(t("Move"));
            }
            Drag::FillHandle { .. } => self.ed.commit(t("Fill")),
            Drag::MotionEnd | Drag::MotionStart { .. } => self.ed.commit(t("Animate Selection")),
            Drag::Hint { .. } => self.ed.commit(t("Shape Hint")),
            Drag::Reshape { ids, .. } => {
                // Pushing works on many short pieces, and the result is a curved path again.
                for id in ids {
                    if let Some(object) = self.ed.object_mut(id) {
                        let closed = object.is_closed();
                        (object.points, object.handles) = crate::fit::fit_curves(&object.points, closed, crate::fit::TOLERANCE);
                        object.refit_to_points();
                    }
                }
                self.ed.commit(t("Reshape Area"));
            }
            Drag::NineSlice { origin, .. } => {
                // The cuts stay the same distance from the edges, so the next scaling keeps the same corners.
                if let Some(now) = self.ed.selection_bounds() {
                    let g = self.nine_guides;
                    let keep = |near: f32, far: f32, old: f32, new: f32| (near * old / new.max(1.0), 1.0 - (1.0 - far) * old / new.max(1.0));
                    let ((x0, x1), (y0, y1)) = (keep(g[0], g[1], origin.w, now.w), keep(g[2], g[3], origin.h, now.h));
                    if x0 < x1 && y0 < y1 {
                        self.nine_guides = [x0, x1, y0, y1];
                    }
                }
                self.ed.commit(t("9-Slice Scaling"));
            }
            Drag::NineGuide(_) => {}
            Drag::Scrub { .. } => self.ed.commit(t("Path Scrubber")),
            Drag::Guide { horizontal, index, at } => {
                // Let go over a ruler or off the canvas, a guide is thrown away.
                let bounds = self.canvas_bounds.get();
                let ruler = if self.show_rulers { px(RULER_SIZE) } else { px(0.) };
                let kept = bounds.contains(&ev.position) && ev.position.x >= bounds.origin.x + ruler && ev.position.y >= bounds.origin.y + ruler;
                let guides = self.ed.doc.page_mut().guides.list_mut(horizontal);
                match (index, kept) {
                    (Some(index), true) if index < guides.len() => guides[index] = at,
                    (Some(index), false) if index < guides.len() => {
                        guides.remove(index);
                    }
                    (None, true) => guides.push(at),
                    _ => {}
                }
                self.ed.commit(t("Guides"));
            }
            Drag::Resize { .. } => self.ed.commit(t("Scale")),
            Drag::Vertex { .. } => self.ed.commit(t("Edit Path")),
            Drag::PenHandle => {}
            Drag::Pixels { pixels, .. } => {
                if pixels {
                    self.ed.commit(t("Move"));
                }
            }
            Drag::Handle { id, index, moved: false, pen: true, .. } => self.pen_point_click(id, index),
            Drag::Handle { id, index, fresh, moved, .. } => {
                // Alt+click without a drag turns the vertex back into a corner.
                if fresh && !moved {
                    if let Some(object) = self.ed.object_mut(id) {
                        object.set_handles(index, Handles::default());
                        object.refit_to_points();
                    }
                }
                self.ed.commit(t("Edit Path"));
            }
            Drag::Marquee { start, current } if self.tool == Tool::Subselect => self.finish_point_marquee(Rect::from_corners(start, current)),
            Drag::Marquee { start, current } => self.ed.select_in_rect(Rect::from_corners(start, current)),
            Drag::Create { start, current } => self.finish_create(start, current),
            Drag::Freehand { points, target } => self.finish_freehand(points, target),
            Drag::Crop { start, current } => self.finish_crop_drag(start, current),
            Drag::PixelArea { target, points, mode } => self.finish_pixel_area(target, points, mode),
            Drag::Transform { kind, .. } => {
                if kind != TransformKind::Pivot {
                    self.status = t("Press Enter or double-click to apply, Esc to cancel").into();
                }
            }
            Drag::Freeform { id, .. } => {
                // Pulling works on many short pieces, and the result is a curved path again.
                if let Some(object) = self.ed.object_mut(id) {
                    let closed = object.is_closed();
                    (object.points, object.handles) = crate::fit::fit_curves(&object.points, closed, crate::fit::TOLERANCE);
                    object.refit_to_points();
                }
                self.ed.commit(t("Freeform"));
            }
            Drag::Pan { .. } => {}
        }
        cx.notify();
    }

    /// The Gradient tool: the selected vectors are filled with a gradient that
    /// starts where the drag began and ends where it was let go.
    fn drag_gradient_fill(&mut self, start: Vec2, end: Vec2) {
        let fallback = self.fill.unwrap_or(Color::BLACK);
        let mut any = false;
        self.ed.for_selected(|o| {
            if !o.takes_color() || matches!(o.shape, Shape::Line) {
                return;
            }
            let mut style = o.fill_style.as_deref().cloned().filter(|style| !matches!(style.kind, FillKind::Pattern | FillKind::WebDither)).unwrap_or_else(|| {
                let from = o.fill.unwrap_or(fallback);
                let light = from.r as u32 + from.g as u32 + from.b as u32 > 600;
                FillStyle::new(FillKind::Linear, from, if light { Color::BLACK } else { Color::rgb(255, 255, 255) })
            });
            style.set_handle(false, start, o.bounds);
            style.set_handle(true, end, o.bounds);
            o.fill_style = Some(Arc::new(style));
            o.fill = o.fill.or(Some(fallback));
            any = true;
        });
        if any {
            self.ed.commit(t("Fill"));
        } else {
            self.status = t("Select a vector object to fill with a gradient").into();
        }
    }

    fn finish_create(&mut self, start: Vec2, current: Vec2) {
        let bounds = Rect::from_corners(start, current);
        if bounds.w < 1.0 && bounds.h < 1.0 {
            return;
        }
        if self.tool == Tool::Knife {
            if self.ed.knife(start, current) > 0 {
                self.ed.commit(t("Knife"));
            } else {
                self.status = t("The Knife line did not cross a vector path.").into();
            }
            return;
        }
        if self.tool == Tool::Gradient {
            self.drag_gradient_fill(start, current);
            return;
        }
        let object = match self.tool {
            // A polygon is drawn from its middle outward, as in Fireworks.
            Tool::CircleHotspot => {
                let radius = (current.x - start.x).hypot(current.y - start.y);
                Object::new_web_circle(self.ed.alloc_id(), Rect { x: start.x - radius, y: start.y - radius, w: radius * 2.0, h: radius * 2.0 })
            }
            Tool::Polygon => {
                let radius = (current.x - start.x).hypot(current.y - start.y);
                let around = Rect { x: start.x - radius, y: start.y - radius, w: radius * 2.0, h: radius * 2.0 };
                // The sides, or the points of a star, come from the tool's options.
                let kind = if self.tool_options.star { AutoKind::Star } else { AutoKind::Polygon };
                let d = kind.defaults();
                let count = self.tool_options.sides.round().clamp(3.0, 64.0) as u32;
                self.new_object(kind.label(), Shape::Auto { kind, a: d.a, b: d.b, count }, around, Vec::new())
            }
            Tool::Hotspot => Object::new_web(self.ed.alloc_id(), WebKind::Hotspot, bounds),
            Tool::Slice => Object::new_web(self.ed.alloc_id(), WebKind::Slice, bounds),
            Tool::Rect =>self.new_object(t("Rectangle"), Shape::Rect { radius: 0.0 }, bounds, Vec::new()),
            Tool::Ellipse => self.new_object(t("Ellipse"), Shape::Ellipse, bounds, Vec::new()),
            _ => {
                let mut line = self.new_object(t("Line"), Shape::Line, bounds, vec![start, current]);
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
            self.status = t("Press Enter or double-click to crop, Esc to cancel").into();
        }
    }

    fn apply_crop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(area) = self.crop_area.take() else { return };
        // The same box, drawn with Export Area, is saved as an image instead of cut out of the canvas.
        if self.tool == Tool::ExportArea {
            self.export_area(area, window, cx);
            return;
        }
        if self.ed.crop_canvas(area) {
            self.ed.commit(t("Crop Document"));
            self.status.clear();
        }
    }

    /// Fireworks' Export Area: opens Image Preview for that part of the page alone.
    fn export_area(&mut self, area: Rect, _: &mut Window, _: &mut Context<Self>) {
        self.open_image_preview(Some(area));
    }

    /// The page as it is exported, or with an area only that part of it.
    fn export_source(&mut self, area: Option<Rect>) -> crate::model::Page {
        self.ed.end_mask_edit();
        self.ed.close_groups();
        let mut page = self.ed.export_page();
        if let Some(area) = area {
            let (x, y) = (area.x.round(), area.y.round());
            (page.width, page.height) = (area.w.round().max(1.0) as u32, area.h.round().max(1.0) as u32);
            page.all_objects_mut().for_each(|o| o.translate(-x, -y));
            // The guides and the JPEG mask belong to the whole page.
            page.jpeg_mask = None;
        }
        page
    }

    pub fn open_image_preview(&mut self, area: Option<Rect>) {
        self.finish_pending();
        let page = self.ed.doc.page();
        let size = area.map_or((page.width, page.height), |a| (a.w.round().max(1.0) as u32, a.h.round().max(1.0) as u32));
        self.image_preview = Some(ImagePreview { area, before: page.export, pane: None, made_from: None, size });
    }

    /// Encodes the image for Image Preview again whenever the settings or the page have changed.
    pub fn refresh_image_preview(&mut self, cx: &mut Context<Self>) {
        let revision = self.ed.revision;
        let Some(area) = self.image_preview.as_ref().filter(|preview| preview.made_from != Some(revision)).map(|preview| preview.area) else { return };
        if let Some(preview) = self.image_preview.as_mut() {
            preview.made_from = Some(revision);
        }
        let page = self.export_source(area);
        cx.spawn(async move |this, cx| {
            let pane = cx
                .background_executor()
                .spawn(async move {
                    let bytes = encode(&page, &page.export).ok()?;
                    let image = encoded_texture(&bytes)?;
                    Some(PreviewPane { image, caption: format!("{}   {}", settings_label(&page.export), format_size(bytes.len())) })
                })
                .await;
            this.update(cx, |this, cx| {
                // Only the newest image is shown; an older one that arrives late is dropped.
                match this.image_preview.as_mut().filter(|preview| preview.made_from == Some(revision)) {
                    Some(preview) => this.stale_previews.extend(std::mem::replace(&mut preview.pane, pane).map(|old| old.image)),
                    None => this.stale_previews.extend(pane.map(|new| new.image)),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Closes Image Preview. Kept, the settings stay as they were set there; otherwise they go back.
    pub fn close_image_preview(&mut self, keep: bool) {
        self.commit_field();
        let Some(preview) = self.image_preview.take() else { return };
        self.stale_previews.extend(preview.pane.map(|pane| pane.image));
        if !keep && self.ed.doc.page().export != preview.before {
            self.set_export(|settings| *settings = preview.before);
        }
    }

    /// Export in Image Preview: asks for a file and saves the image shown.
    pub fn export_image_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_field();
        let Some(area) = self.image_preview.as_ref().map(|preview| preview.area) else { return };
        let page = self.export_source(area);
        let whole = area.is_none();
        let stem = self.path.as_ref().or(self.source_image.as_ref()).map_or_else(|| t("Untitled").into(), |p| file_stem(p));
        let extension = page.export.format.extension();
        let picked = cx.prompt_for_new_path(&Self::default_dir(), Some(&format!("{stem}.{extension}")));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(mut path))) = picked.await {
                if path.extension().is_none() {
                    path.set_extension(extension);
                }
                this.update(cx, |this, cx| {
                    // The whole page takes its slices and image map with it; a part of it is one image.
                    let written = export_page(&page, &path).and_then(|()| if whole { export_web(&page, &path) } else { Ok(0) });
                    this.status = match written {
                        Ok(0) => tf("Exported {}", &[&path.display()]),
                        Ok(extra) => tf("Exported {} and {} web file(s) beside it", &[&path.display(), &extra]),
                        Err(error) => tf("Export failed: {}", &[&format!("{error:#}")]),
                    };
                    this.close_image_preview(true);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// A click with the Polygon Hotspot or Polygon Slice tool: each adds a corner,
    /// and a double-click or a click back on the first corner closes the area.
    fn polygon_web_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        let p = self.guide_snapped(p);
        let mut points = self.polygon_web.take().unwrap_or_default();
        let reach = 6.0 / self.zoom;
        let at_start = points.first().is_some_and(|first| (first.x - p.x).abs() <= reach && (first.y - p.y).abs() <= reach);
        let closing = ev.click_count >= 2 || (at_start && points.len() >= 3);
        if !closing {
            points.push(p);
            self.polygon_web = Some(points);
            return;
        }
        if !at_start && points.last() != Some(&p) {
            points.push(p);
        }
        if points.len() < 3 {
            return;
        }
        let kind = if self.tool == Tool::PolygonSlice { WebKind::Slice } else { WebKind::Hotspot };
        let object = Object::new_web_polygon(self.ed.alloc_id(), kind, &points);
        let label = object.name.clone();
        self.add_new(object, &label);
    }

    /// The paths Reshape Area and Redraw Path work on: the selected ones, or else the one nearest the mouse.
    fn paths_near(&self, p: Vec2, reach: f32) -> Vec<u64> {
        let selected: Vec<u64> = self.ed.selected().iter().filter(|o| o.reshapable()).map(|o| o.id).collect();
        if !selected.is_empty() {
            return selected;
        }
        self.ed.doc.page().nearest_path(p, reach, &self.ed.selection).into_iter().collect()
    }

    /// A press with a Path Scrubber: the stroked paths under it are readied, and the stroke changes where the mouse is.
    fn scrub_down(&mut self, p: Vec2, add: bool) {
        let radius = self.freeform_size / 2.0;
        let ids: Vec<u64> = self.paths_near(p, radius).into_iter().filter(|id| self.ed.object_mut(*id).is_some_and(|o| o.prepare_scrub())).collect();
        if ids.is_empty() {
            self.status = t("Press on a path that has a stroke to change how heavy it is.").into();
            self.ed.rollback();
            return;
        }
        self.ed.selection = ids.clone();
        self.drag = Some(Drag::Scrub { ids, add });
        self.scrub_at(p);
    }

    /// One touch of the Path Scrubber at a place, as strong as its rate.
    fn scrub_at(&mut self, p: Vec2) {
        let Some(Drag::Scrub { ids, add }) = &self.drag else { return };
        let (ids, add) = (ids.clone(), *add);
        let (radius, amount) = (self.freeform_size / 2.0, self.tool_options.rate / 100.0 * 0.25 * if add { 1.0 } else { -1.0 });
        for id in ids {
            if let Some(object) = self.ed.object_mut(id) {
                object.scrub(p, radius, amount);
            }
        }
    }

    /// Reshape Area: every point of the paths within the circle is pushed along with the mouse.
    fn reshape_down(&mut self, p: Vec2) {
        let radius = self.freeform_size / 2.0;
        let ids = self.paths_near(p, radius);
        if ids.is_empty() {
            self.status = t("Press near a vector path or shape to reshape it.").into();
            return;
        }
        for id in &ids {
            let Some(object) = self.ed.object_mut(*id) else { continue };
            if object.to_path() {
                object.flatten();
                object.points = subdivide(&object.points, object.is_closed(), (radius * FREEFORM_DETAIL).max(1.0));
            }
        }
        self.ed.selection = ids.clone();
        self.drag = Some(Drag::Reshape { ids, last: p, radius });
    }

    /// Redraw Path: the stroke replaces the part of the path it was drawn along.
    fn redraw(&mut self, stroke: Vec<Vec2>) {
        let Some(first) = stroke.first().copied() else { return };
        let reach = 8.0 / self.zoom;
        let tolerance = self.tool_options.fit_tolerance();
        let done = self.paths_near(first, reach).into_iter().any(|id| {
            let Some(object) = self.ed.object_mut(id) else { return false };
            if !object.to_path() {
                return false;
            }
            let mut dense = object.clone();
            dense.flatten();
            let closed = dense.is_closed();
            let points = subdivide(&dense.points, closed, 3.0);
            let Some(redrawn) = crate::model::redraw_path(&points, closed, &stroke, reach) else { return false };
            (object.points, object.handles) = crate::fit::fit_curves(&redrawn, closed, tolerance);
            object.refit_to_points();
            true
        });
        if done {
            self.ed.commit(t("Redraw Path"));
        } else {
            self.status = t("Start the stroke on a path to redraw part of it.").into();
            self.ed.rollback();
        }
    }

    /// The four cuts of 9-slice scaling on the page, for the selection as it is: two places across, then two down.
    pub fn nine_slice_lines(&self) -> Option<[f32; 4]> {
        if self.tool != Tool::NineSlice {
            return None;
        }
        let b = self.ed.selection_bounds()?;
        let g = self.nine_guides;
        Some([b.x + b.w * g[0], b.x + b.w * g[1], b.y + b.h * g[2], b.y + b.h * g[3]])
    }

    /// A press with the 9-slice scaling tool: on a handle of the selection it
    /// scales, on one of the cuts it moves that cut, and elsewhere it selects.
    fn nine_slice_down(&mut self, ev: &MouseDownEvent, p: Vec2) {
        if let (Some(bounds), Some(lines)) = (self.ed.selection_bounds(), self.nine_slice_lines()) {
            let map = self.view_map();
            if let Some(handle) = handle_rects(bounds, map).iter().position(|r| r.contains(&ev.position)) {
                let originals = self.ed.selected().into_iter().cloned().collect();
                self.drag = Some(Drag::NineSlice { handle, origin: bounds, originals });
                return;
            }
            let reach = 4.0 / self.zoom;
            let inside = p.x >= bounds.x - reach && p.x <= bounds.right() + reach && p.y >= bounds.y - reach && p.y <= bounds.bottom() + reach;
            let near = (0..4).find(|index| (if *index < 2 { p.x } else { p.y } - lines[*index]).abs() <= reach);
            if let Some(index) = near.filter(|_| inside) {
                self.drag = Some(Drag::NineGuide(index));
                return;
            }
        }
        self.ed.selection = self.ed.doc.page().hit_test(p, self.tolerance()).into_iter().collect();
    }

    fn finish_freehand(&mut self, points: Vec<Vec2>, target: Option<u64>) {
        if self.tool == Tool::RedrawPath {
            self.redraw(points);
            return;
        }
        let width = self.freehand_width();
        let name = match self.tool {
            Tool::Pencil => t("Pencil"),
            Tool::VectorPath => t("Path"),
            _ => t("Brush"),
        };
        let color = self.stroke.unwrap_or(Color::BLACK);
        if let Some(id) = target {
            let offset = self.stamp.and_then(|s| s.offset).unwrap_or_default();
            let options = self.tool_options;
            // With no color to change picked, Replace Color changes the one the stroke starts on.
            let under = points.first().and_then(|p| self.ed.object(id)?.pixel_at(*p)).and_then(|(x, y)| self.ed.object(id)?.bitmap()?.pixel(x, y)).unwrap_or([0; 4]);
            let (op, label) = match self.tool {
                Tool::Eraser => (PixelStroke::Erase, t("Eraser")),
                Tool::Blur => (PixelStroke::Blur, t("Blur")),
                Tool::Sharpen => (PixelStroke::Sharpen, t("Sharpen")),
                Tool::Dodge => (PixelStroke::Dodge(options.exposure / 100.0, options.range), t("Dodge")),
                Tool::Burn => (PixelStroke::Burn(options.exposure / 100.0, options.range), t("Burn")),
                Tool::Smudge => (PixelStroke::Smudge(options.pressure / 100.0, options.smudge_color.then_some(color)), t("Smudge")),
                Tool::ReplaceColor => {
                    let from = options.replace_from.map_or(under, |c| [c.r, c.g, c.b, 255]);
                    (PixelStroke::Replace(from, options.replace_to, options.replace_tolerance.round().clamp(0.0, 255.0) as u8), t("Replace Color"))
                }
                Tool::Stamp => (PixelStroke::Clone(offset), t("Rubber Stamp")),
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
        // A Vector Path is smoothed into curves, as closely as its precision asks.
        if self.tool == Tool::VectorPath {
            (object.points, object.handles) = crate::fit::fit_curves(&object.points, false, self.tool_options.fit_tolerance());
            object.refit_to_points();
        }
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
        let pressed = crate::menus::key_name(key, m.control, m.alt, m.shift);
        // Keyboard Shortcuts is waiting for the new keys of a command: this press is them.
        if let Some((picked, capturing)) = self.shortcut_dialog {
            match (capturing, key) {
                (false, "escape") => self.shortcut_dialog = None,
                (true, "escape") => self.shortcut_dialog = Some((picked, false)),
                (true, "shift" | "control" | "alt" | "platform" | "function") => {}
                (true, _) => self.set_shortcut(Some(pressed)),
                _ => {}
            }
            cx.notify();
            return;
        }
        // Rules as Text takes every key while it is open and no field or list is.
        if self.script.is_some() && self.field.is_none() && self.open_combo.is_none() {
            self.script_key(key, m.control, m.shift, window, cx);
            cx.notify();
            return;
        }
        let typing = self.field.is_some() || self.text_edit.is_some();
        if is_sahne() && !typing && m.control && !m.alt && !m.shift && key == "enter" {
            self.run(Cmd::Frame(crate::menus::FrameCmd::TestMovie), window, cx);
            cx.notify();
            return;
        }
        // The animation program's own keys, where Flash has them.
        if is_sahne() && !typing && !m.control && !m.alt {
            use crate::menus::FrameCmd;
            let frame = match (key, m.shift) {
                ("f5", false) => Some(FrameCmd::InsertFrame),
                ("f5", true) => Some(FrameCmd::RemoveFrame),
                ("f6", false) => Some(FrameCmd::Keyframe),
                ("f6", true) => Some(FrameCmd::ClearKeyframe),
                ("f7", false) => Some(FrameCmd::BlankKeyframe),
                (".", false) => Some(FrameCmd::Forward),
                (",", false) => Some(FrameCmd::Back),
                _ => None,
            };
            if let Some(frame) = frame {
                self.run(Cmd::Frame(frame), window, cx);
                cx.notify();
                return;
            }
        }
        // The user's own shortcuts come before the built-in ones, except while something is being typed.
        if let Some(action) = self.custom_shortcut(&pressed).filter(|_| !typing || m.control || m.alt) {
            if let Some(cmd) = action {
                self.run(cmd, window, cx);
            }
            cx.notify();
            return;
        }
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
        } else if self.filter_dialog.is_some() {
            match key {
                "enter" => self.close_filter_dialog(true),
                "escape" => self.close_filter_dialog(false),
                _ => {}
            }
        } else if self.jpeg_dialog.is_some() {
            match key {
                "enter" => self.close_jpeg_dialog(true),
                "escape" => self.close_jpeg_dialog(false),
                _ => {}
            }
        } else if self.command_name.is_some() && matches!(key, "enter" | "escape") {
            self.close_command_name(key == "enter");
        } else if self.tween_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_tween_dialog(key == "enter");
        } else if self.mask_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_mask_dialog(key == "enter");
        } else if self.rules_dialog && matches!(key, "enter" | "escape") {
            // Enter only takes what was typed; Esc closes.
            self.commit_field();
            self.rules_dialog = key != "escape";
        } else if self.attach_dialog && matches!(key, "enter" | "escape") {
            self.attach_dialog = false;
        } else if self.behavior_dialog && matches!(key, "enter" | "escape") {
            self.close_behavior_dialog();
        } else if self.twist_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_twist_dialog(key == "enter");
        } else if self.arrow_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_arrow_dialog(key == "enter");
        } else if self.frame_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_frame_dialog(key == "enter");
        } else if self.reverse_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_reverse_dialog(key == "enter");
        } else if self.commands_dialog.is_some() && key == "escape" {
            self.commands_dialog = None;
        } else if self.template_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_template_dialog(key == "enter");
        } else if self.template_name.is_some() && matches!(key, "enter" | "escape") {
            self.close_template_name(key == "enter");
        } else if self.symbol_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_symbol_dialog(key == "enter");
        } else if self.animate_dialog.is_some() && matches!(key, "enter" | "escape") {
            self.close_animate_dialog(key == "enter");
        } else if self.prefs_dialog.is_some() {
            match key {
                "enter" => self.close_preferences(true),
                "escape" => self.close_preferences(false),
                _ => {}
            }
        } else if self.spell.is_some() {
            match key {
                "enter" => self.spell_change(false),
                "escape" => {
                    self.field = None;
                    self.spell = None;
                }
                _ => {}
            }
        } else if self.image_preview.is_some() {
            match key {
                "enter" => self.export_image_preview(window, cx),
                "escape" => self.close_image_preview(false),
                _ => {}
            }
        } else if self.fill_dialog.is_some() {
            match key {
                "enter" => self.close_fill_dialog(true),
                "escape" => self.close_fill_dialog(false),
                _ => {}
            }
        } else if self.page_setup_dialog.is_some() {
            match key {
                "enter" => self.close_page_setup(true),
                "escape" => self.close_page_setup(false),
                _ => {}
            }
        } else if self.print_dialog.is_some() {
            match key {
                "enter" => self.apply_print_dialog(window, cx),
                "escape" => self.print_dialog = None,
                _ => {}
            }
        } else if self.guide_dialog.is_some() {
            match key {
                "enter" => self.apply_guide_dialog(),
                "escape" => self.guide_dialog = None,
                _ => {}
            }
        } else if self.guides_dialog {
            if matches!(key, "enter" | "escape") {
                self.guides_dialog = false;
            }
        } else if self.marquee_dialog.is_some() {
            match key {
                "enter" => self.apply_marquee_dialog(),
                "escape" => self.marquee_dialog = None,
                _ => {}
            }
        } else if self.numeric_dialog.is_some() {
            match key {
                "enter" => self.apply_numeric_dialog(),
                "escape" => self.numeric_dialog = None,
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
            ("x", true, false) => Cmd::ImagePreview,
            ("c", false, false) => Cmd::Copy,
            ("v", false, false) => Cmd::Paste,
            ("d", false, true) => Cmd::Duplicate,
            ("d", true, false) => Cmd::Clone,
            ("g", true, true) => Cmd::SnapToGrid,
            ("y", true, false) => Cmd::AttachToPath,
            ("h", true, false) if is_sahne() => Cmd::Frame(crate::menus::FrameCmd::AddHint),
            ("t", true, true) => Cmd::TweenInstances,
            ("p", false, false) => Cmd::Print,
            (";", false, false) => Cmd::Guides(GuideCmd::Show),
            (";", false, true) => Cmd::Guides(GuideCmd::Lock),
            (";" | ":", true, false) => Cmd::Guides(GuideCmd::Snap),
            (";" | ":", true, true) => Cmd::Guides(GuideCmd::Slice),
            ("\\", false, false) => Cmd::Guides(GuideCmd::ShowSmart),
            ("v", true, true) => Cmd::PasteAttributes,
            ("0", false, true) => Cmd::FitSelection,
            ("i", true, false) => Cmd::SelectInverse,
            ("z", true, true) => Cmd::FlattenSelection,
            ("e", false, false) => Cmd::MergeDown,
            ("f", true, true) => Cmd::RepeatFilter,
            ("u", true, false) => Cmd::InsertWeb(WebKind::Hotspot),
            ("a", false, false) => Cmd::SelectAll,
            ("d", false, false) => Cmd::Deselect,
            ("=", _, false) | ("+", _, false) => Cmd::ZoomIn,
            ("-", false, false) => Cmd::ZoomOut,
            ("0", false, false) => Cmd::FitAll,
            ("1", false, false) => Cmd::ZoomTo(100),
            ("r", false, true) => Cmd::Rulers,
            ("g", false, true) => Cmd::Grid,
            ("f", false, false) => Cmd::Panel(Panel::Find),
            // On text Ctrl+B is bold, as in Fireworks; on a symbol it breaks it apart.
            ("b", false, false) if self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Text { .. })) => Cmd::TextStyle(crate::menus::TextStyleCmd::Bold),
            ("b", false, false) => Cmd::BreakApart,
            ("i", false, false) => Cmd::TextStyle(crate::menus::TextStyleCmd::Italic),
            // On text Ctrl+U underlines; otherwise it opens Preferences, as in Fireworks.
            ("u", false, false) if self.ed.selected().iter().any(|o| matches!(o.shape, Shape::Text { .. })) => Cmd::TextStyle(crate::menus::TextStyleCmd::Underline),
            ("u", false, false) => Cmd::Preferences,
            ("," | "<", true, false) => Cmd::TextSizeStep(false),
            ("." | ">", true, false) => Cmd::TextSizeStep(true),
            ("l", true, true) => Cmd::TextAlign(crate::model::TextAlign::Left),
            ("c", true, true) => Cmd::TextAlign(crate::model::TextAlign::Center),
            ("r", true, true) => Cmd::TextAlign(crate::model::TextAlign::Right),
            ("j", true, true) => Cmd::TextAlign(crate::model::TextAlign::Justify),
            ("s", true, true) => Cmd::TextAlign(crate::model::TextAlign::Stretch),
            ("g", false, false) => Cmd::Group,
            ("t", false, false) => Cmd::FreeTransform,
            ("t", true, false) => Cmd::NumericTransform,
            ("v", true, false) => Cmd::PasteInside,
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
            "v" | "0" => Some(self.group_key(Tool::Pointer)),
            "a" => Some(Tool::Subselect),
            "b" => Some(Tool::Brush),
            // Y and R are shared, so pressing them again cycles between the two tools.
            "y" => Some(if self.tool == Tool::Pencil { Tool::Knife } else { Tool::Pencil }),
            "r" => Some(self.group_key(Tool::Blur)),
            "e" => Some(Tool::Eraser),
            "c" => Some(self.group_key(Tool::Crop)),
            // Q walks through the three transform tools, as it does in Fireworks.
            "q" => Some(match self.tool {
                Tool::Scale => Tool::Skew,
                Tool::Skew => Tool::Distort,
                Tool::Distort => Tool::NineSlice,
                _ => Tool::Scale,
            }),
            "m" => Some(self.group_key(Tool::Marquee)),
            "l" => Some(self.group_key(Tool::Lasso)),
            "w" => Some(Tool::Wand),
            "s" => Some(self.group_key(Tool::Stamp)),
            "o" => Some(self.group_key(Tool::Freeform)),
            "j" => Some(self.group_key(Tool::Hotspot)),
            "k" => Some(self.group_key(Tool::Slice)),
            "i" => Some(Tool::Eyedropper),
            "g" => Some(self.group_key(Tool::Bucket)),
            "n" => Some(Tool::Line),
            "p" => Some(self.group_key(Tool::Pen)),
            "u" => Some(self.group_key(Tool::Rect)),
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
            "delete" | "backspace" if self.tool == Tool::Subselect && self.delete_picked_points() => {}
            "delete" | "backspace" => self.run(Cmd::Clear, window, cx),
            "enter" if self.crop_area.is_some() => self.apply_crop(window, cx),
            "enter" if self.transform.is_some() => self.apply_transform(),
            // With nothing waiting to be applied, Enter plays the timeline, as in Flash.
            "f1" => self.run(Cmd::Guide, window, cx),
            "enter" if is_sahne() && self.pen.is_empty() => self.run(Cmd::Frame(crate::menus::FrameCmd::Play), window, cx),
            "enter" => self.finish_pen(false),
            "escape" => {
                let busy = !self.pen.is_empty() || self.crop_area.is_some() || self.pixel_sel.is_some() || self.polygon_lasso.is_some() || self.polygon_web.is_some();
                self.polygon_lasso = None;
                self.polygon_web = None;
                self.pen.clear();
                self.pen_handles.clear();
                self.pen_resume = None;
                self.crop_area = None;
                self.pixel_sel = None;
                let working = busy;
                if self.drag.take().is_some() | self.cancel_transform() {
                    self.ed.rollback();
                } else if !working && is_sahne() && (self.ed.editing_symbol().is_some() || self.ed.editing_clip().is_some()) {
                    // With nothing else to give up, Esc leaves the symbol or the movie clip being edited, as in Flash.
                    self.toggle_symbol_edit();
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
        // With a pixel selection the arrow keys move the selected pixels, not the whole bitmap.
        if let Some(selection) = self.active_pixels() {
            let (target, mask) = (selection.object, selection.mask.clone());
            let bitmap = self.ed.object(target).and_then(|o| match &o.shape {
                Shape::Image { bitmap } => Some(bitmap.clone()),
                _ => None,
            });
            if let Some((before, (piece, left, top))) = bitmap.and_then(|b| b.extract(&mask).map(|piece| (b, piece))) {
                self.move_pixel_selection(target, &before, &piece, (left as i64, top as i64), &mask, (dx as i64, dy as i64), true, false);
                self.ed.commit(t("Move"));
                return;
            }
        }
        if !self.ed.selection.is_empty() {
            self.ed.for_selected(|o| o.translate(dx, dy));
            self.ed.commit(t("Move"));
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
            FieldId::ToolExposure => number(self.tool_options.exposure),
            FieldId::ToolPressure => number(self.tool_options.pressure),
            FieldId::ReplaceTolerance => number(self.tool_options.replace_tolerance),
            FieldId::RedTolerance => number(self.tool_options.red_tolerance),
            FieldId::RedStrength => number(self.tool_options.red_strength),
            FieldId::PolygonSides => number(self.tool_options.sides),
            FieldId::PathPrecision => number(self.tool_options.precision),
            FieldId::ScrubRate => number(self.tool_options.rate),
            FieldId::WandTolerance => number(self.tool_options.wand_tolerance),
            FieldId::FillFeather => number(self.current_fill_look().feather),
            FieldId::PathOffset => number(self.shown_text_style().path_offset),
            FieldId::TemplateName => self.template_name.clone().unwrap_or_default(),
            FieldId::RuleNumber => self.ed.doc.rules.get(self.rule_picked).map_or_else(String::new, |rule| number(rule.event.n)),
            FieldId::RuleIfValue => self.ed.doc.rules.get(self.rule_picked).and_then(|rule| rule.when.as_ref()).map_or_else(String::new, |only| if only.expr.is_empty() { number(only.value) } else { only.expr.clone() }),
            FieldId::RuleStep(which, second) => self.ed.doc.rules.get(self.rule_picked).and_then(|rule| rule.steps.get(which)).map_or_else(String::new, |step| if second { step.second() } else { step.first() }),
            FieldId::VariableName(which) => self.ed.doc.variables.get(which).map_or_else(String::new, |v| v.name.clone()),
            FieldId::VariableStart(which) => self.ed.doc.variables.get(which).map_or_else(String::new, |v| number(v.start)),
            FieldId::Behavior(which, second) => self.shown_behaviors().get(which).map_or_else(String::new, |b| number(if second { b.b } else { b.a })),
            FieldId::Twist(which) => self.twist_dialog.map_or_else(String::new, |s| number([s.steps as f32, s.spacing, s.direction, s.rotation, s.scale, s.fade_to][which.min(5)])),
            FieldId::ActionParam(button) => {
                let action = if button { self.button_action().flatten() } else { self.current_span().and_then(|span| crate::model::Action::parse(&span.action)) };
                match action {
                    Some(crate::model::Action::GotoPlay(n) | crate::model::Action::GotoStop(n) | crate::model::Action::Scene(n)) => n.to_string(),
                    Some(crate::model::Action::Url(address)) => address,
                    _ => String::new(),
                }
            }
            FieldId::TweenEase => self.current_tween().map_or_else(String::new, |tween| number(tween.ease)),
            FieldId::TweenTurns => self.current_tween().map_or_else(String::new, |tween| tween.turns.to_string()),
            FieldId::CurrentFrame => self.frame_position().map_or_else(String::new, |(frame, _)| (frame + 1).to_string()),
            FieldId::CurrentSecond => self.ed.doc.page().timeline.as_ref().map_or_else(String::new, |timeline| format!("{:.1}", timeline.current as f32 / timeline.fps.max(1.0))),
            FieldId::FrameRate => self.ed.doc.page().timeline.as_ref().map_or_else(String::new, |timeline| number(timeline.fps)),
            FieldId::GifDelay => self.gif_dialog.as_ref().map_or_else(String::new, |d| d.delay_ms.to_string()),
            FieldId::TweenSteps => self.tween_dialog.map_or_else(String::new, |d| d.0.to_string()),
            FieldId::ArrowSize => self.arrow_dialog.map_or_else(String::new, |d| number(d.2)),
            FieldId::FrameSize => self.frame_dialog.map_or_else(String::new, number),
            FieldId::CommandName => self.command_name.as_ref().map_or_else(String::new, |(name, _)| name.clone()),
            FieldId::ReverseState(last) => self.reverse_dialog.map_or_else(String::new, |(from, to)| if last { to } else { from }.to_string()),
            FieldId::Batch(which) => self.batch_dialog.as_ref().map_or_else(String::new, |d| match which {
                0 => d.width.to_string(),
                1 => d.height.to_string(),
                2 => number(d.percent),
                3 => d.prefix.clone(),
                _ => d.suffix.clone(),
            }),
            FieldId::NewSymbolName => self.symbol_dialog.as_ref().map_or_else(String::new, |(name, _)| name.clone()),
            FieldId::Animate(which) => self.animate_dialog.map_or_else(String::new, |s| number([s.states as f32, s.distance, s.direction, s.scale, s.opacity_from, s.opacity_to, s.rotation][which.min(6)])),
            FieldId::PrefUndo => self.prefs.undo_steps.to_string(),
            FieldId::PrefPick => number(self.prefs.pick_distance),
            FieldId::PrefSnap => number(self.prefs.snap_distance),
            FieldId::PrefGridX => number(self.prefs.grid.0),
            FieldId::PrefGridY => number(self.prefs.grid.1),
            FieldId::SpellChange => self.spell.as_ref().map_or_else(String::new, |session| session.replacement.clone()),
            FieldId::TextureAmount => number(self.current_fill_look().texture_amount),
            FieldId::GuidePosition => self.guide_dialog.map_or_else(String::new, |(_, _, place)| number(place)),
            FieldId::BucketTolerance => number(self.tool_options.bucket_tolerance),
            FieldId::FreeformSize => number(self.freeform_size),
            FieldId::Url => match first.map(|o| &o.shape) {
                Some(Shape::Web { url, .. }) => url.clone(),
                _ => first.map_or_else(String::new, |o| o.link.clone()),
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
            FieldId::OptimizeSelective => page.export.selective_quality.to_string(),
            FieldId::FilterValue(setting) => self.edited_filter().map_or_else(String::new, |(_, filter)| number(filter.value(setting))),
            FieldId::MarqueeAmount => self.marquee_dialog.map_or_else(String::new, |(_, amount)| number(amount)),
            FieldId::PrintMargin(side) => number(self.print_setup.margins[side.min(3)]),
            FieldId::FillScale => self.current_fill_style().map_or_else(String::new, |style| number(style.scale)),
            FieldId::PrintCopies => self.print_dialog.as_ref().map_or_else(String::new, |dialog| dialog.copies.to_string()),
            FieldId::PrintFrom => self.print_dialog.as_ref().map_or_else(String::new, |dialog| dialog.from.to_string()),
            FieldId::PrintTo => self.print_dialog.as_ref().map_or_else(String::new, |dialog| dialog.to.to_string()),
            FieldId::GradientPlace => self.picked_stop().map_or_else(String::new, |stop| number((stop.0 * 100.0).round())),
            FieldId::GradientOpacity => self.picked_stop().map_or_else(String::new, |stop| number((stop.1.a as f32 / 2.55).round())),
            FieldId::OptimizeSmoothing => page.export.smoothing.to_string(),
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
            FieldId::NumericA | FieldId::NumericB => self.numeric_dialog.map_or_else(String::new, |d| {
                let (a, b) = match d.mode {
                    NumericMode::Scale => d.percent,
                    NumericMode::Resize => d.size,
                    NumericMode::Rotate => (d.angle, d.angle),
                };
                number(if id == FieldId::NumericA { a } else { b })
            }),
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
        } else if let Some(script) = self.script.as_mut() {
            script.insert(text);
            (self.script_pick, self.script_quiet) = (0, false);
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
                self.ed.commit(t("Move"));
            }
            FieldId::W | FieldId::H => {
                let Some(v) = number.filter(|v| *v > 0.0) else { return };
                if self.ed.selection.len() != 1 {
                    self.status = t("Width and height apply to a single object").into();
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
                self.ed.commit(t("Resize"));
            }
            FieldId::Radius => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                self.ed.for_selected(|o| {
                    if let Shape::Rect { radius } = &mut o.shape {
                        *radius = v;
                    }
                });
                self.ed.for_selected(Object::refit_placed);
                self.ed.commit(t("Roundness"));
            }
            FieldId::TextSize => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.ed.for_selected(|o| {
                    if let Shape::Text { size, .. } = &mut o.shape {
                        *size = v;
                        o.refit_text();
                    }
                });
                self.ed.commit(t("Text Size"));
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
                    self.ed.commit(t("Stroke Width"));
                }
            }
            FieldId::BrushSize => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.brush_size = v.min(MAX_BRUSH_SIZE);
            }
            FieldId::ToolExposure | FieldId::ToolPressure | FieldId::RedTolerance | FieldId::RedStrength => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                let v = v.min(100.0);
                match field.id {
                    FieldId::ToolExposure => self.tool_options.exposure = v.max(1.0),
                    FieldId::ToolPressure => self.tool_options.pressure = v.max(1.0),
                    FieldId::RedTolerance => self.tool_options.red_tolerance = v,
                    _ => self.tool_options.red_strength = v,
                }
            }
            FieldId::ReplaceTolerance | FieldId::WandTolerance | FieldId::BucketTolerance => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                let v = v.round().min(255.0);
                match field.id {
                    FieldId::ReplaceTolerance => self.tool_options.replace_tolerance = v,
                    FieldId::WandTolerance => self.tool_options.wand_tolerance = v,
                    _ => self.tool_options.bucket_tolerance = v,
                }
            }
            FieldId::PolygonSides => {
                let Some(v) = number.filter(|v| *v >= 3.0) else { return };
                self.tool_options.sides = v.round().min(64.0);
            }
            FieldId::Batch(which) => {
                let Some(dialog) = self.batch_dialog.as_mut() else { return };
                let size = number.filter(|v| *v >= 1.0);
                match (which, size) {
                    (0, Some(v)) => dialog.width = v.round().min(30000.0) as u32,
                    (1, Some(v)) => dialog.height = v.round().min(30000.0) as u32,
                    (2, Some(v)) => dialog.percent = v.min(10000.0),
                    (3, _) => dialog.prefix = text.to_string(),
                    (4, _) => dialog.suffix = text.to_string(),
                    _ => {}
                }
            }
            FieldId::RuleNumber => {
                let Some(v) = number else { return };
                self.edit_rule(|rule| rule.event.n = v);
            }
            FieldId::RuleIfValue => {
                let names: Vec<String> = self.ed.doc.variables.iter().map(|v| v.name.clone()).collect();
                let read = match crate::script::amount(text, &names) {
                    Ok(read) => read,
                    Err(why) => {
                        self.status = why;
                        return;
                    }
                };
                self.edit_rule(|rule| {
                    if let Some(only) = rule.when.as_mut() {
                        (only.value, only.expr) = match read {
                            crate::script::Amount::Plain(v) => (v, String::new()),
                            crate::script::Amount::Sum(sum) => (0.0, sum),
                        };
                    }
                });
            }
            FieldId::RuleStep(which, second) => {
                // A plain number, or a sum such as Score * 2: what the written language takes, the lists take too.
                let names: Vec<String> = self.ed.doc.variables.iter().map(|v| v.name.clone()).collect();
                let read = match crate::script::amount(text, &names) {
                    Ok(read) => read,
                    Err(why) => {
                        self.status = why;
                        return;
                    }
                };
                self.edit_rule(|rule| {
                    if let Some(step) = rule.steps.get_mut(which) {
                        let (plain, sum) = match read {
                            crate::script::Amount::Plain(v) => (v, String::new()),
                            crate::script::Amount::Sum(sum) => (0.0, sum),
                        };
                        if second {
                            (step.y, step.ey) = (plain, sum);
                        } else {
                            (step.x, step.ex) = (plain, sum);
                        }
                    }
                });
            }
            FieldId::VariableName(which) => {
                // A name is one word with no braces, and no other number's name. The rules that are about the number follow it.
                let name: String = text.chars().filter(|c| !c.is_whitespace() && !matches!(c, '{' | '}')).collect();
                let Some(old) = self.ed.doc.variables.get(which).map(|v| v.name.clone()) else { return };
                if name.is_empty() || name == old || self.ed.doc.variables.iter().any(|v| v.name == name) {
                    return;
                }
                use crate::rules::Pick;
                for rule in &mut self.ed.doc.rules {
                    if rule.event.kind.needs().0 == Some(Pick::Variable) && rule.event.a == old {
                        rule.event.a = name.clone();
                    }
                    if let Some(only) = rule.when.as_mut().filter(|only| only.number == old) {
                        only.number = name.clone();
                    }
                    for step in rule.steps.iter_mut().filter(|step| step.kind.needs().0 == Some(Pick::Variable) && step.a == old) {
                        step.a = name.clone();
                    }
                }
                self.ed.doc.variables[which].name = name;
                self.ed.commit(t("Rename Number"));
            }
            FieldId::VariableStart(which) => {
                let (Some(v), Some(variable)) = (number, self.ed.doc.variables.get_mut(which)) else { return };
                variable.start = v;
                self.ed.commit(t("Rules"));
            }
            FieldId::Behavior(which, second) => {
                let (Some(v), Some(shown)) = (number, self.shown_behaviors().get(which).copied()) else { return };
                let v = v.clamp(-100000.0, 100000.0);
                // The number goes to that kind of behavior on everything selected.
                self.ed.for_selected(|o| {
                    for behavior in o.behaviors.iter_mut().filter(|b| b.kind == shown.kind) {
                        if second {
                            behavior.b = v;
                        } else {
                            behavior.a = v;
                        }
                    }
                });
                self.ed.commit(t("Behavior Settings"));
            }
            FieldId::Twist(which) => {
                let (Some(v), Some(s)) = (number, self.twist_dialog.as_mut()) else { return };
                match which {
                    0 => s.steps = v.round().clamp(1.0, 200.0) as u32,
                    1 => s.spacing = v.clamp(-2000.0, 2000.0),
                    2 => s.direction = v.rem_euclid(360.0),
                    3 => s.rotation = v.clamp(-360.0, 360.0),
                    4 => s.scale = v.clamp(1.0, 1000.0),
                    _ => s.fade_to = v.clamp(0.0, 100.0),
                }
            }
            FieldId::ActionParam(button) => {
                use crate::model::Action;
                let before = if button { self.button_action().flatten() } else { self.current_span().and_then(|span| Action::parse(&span.action)) };
                let whole = number.filter(|v| *v >= 1.0).map(|v| v.round() as usize);
                let action = match (before, whole) {
                    (Some(Action::GotoPlay(_)), Some(n)) => Action::GotoPlay(n),
                    (Some(Action::GotoStop(_)), Some(n)) => Action::GotoStop(n),
                    (Some(Action::Scene(_)), Some(n)) => Action::Scene(n),
                    (Some(Action::Url(_)), _) if !cleared => Action::Url(text.to_string()),
                    _ => return,
                };
                if button {
                    self.set_button_action(Some(action));
                } else {
                    self.set_frame_action(Some(action));
                }
            }
            FieldId::TweenEase | FieldId::TweenTurns => {
                let (Some(v), Some(mut tween)) = (number, self.current_tween()) else { return };
                if field.id == FieldId::TweenEase {
                    tween.ease = v.clamp(-100.0, 100.0);
                } else {
                    tween.turns = v.round().clamp(-50.0, 50.0) as i32;
                }
                self.ed.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(tween)));
                self.ed.commit(t("Tween"));
            }
            FieldId::CurrentFrame => {
                if let Some(v) = number.filter(|v| *v >= 1.0) {
                    self.show_frame((v.round() as usize - 1).min(crate::model::MAX_FRAMES - 1), None);
                }
            }
            FieldId::CurrentSecond => {
                let fps = self.ed.doc.page().timeline.as_ref().map_or(DEFAULT_FPS, |timeline| timeline.fps);
                if let Some(v) = number.or_else(|| text.replace(',', ".").parse::<f32>().ok()).filter(|v| *v >= 0.0) {
                    self.show_frame(((v * fps).round() as usize).min(crate::model::MAX_FRAMES - 1), None);
                }
            }
            FieldId::FrameRate => {
                if let (Some(v), Some(timeline)) = (number.filter(|v| *v >= 1.0), self.ed.doc.page_mut().timeline.as_mut()) {
                    timeline.fps = v.min(120.0);
                    self.ed.commit(t("Frame Rate"));
                }
            }
            FieldId::GifDelay => {
                if let (Some(v), Some(dialog)) = (number.filter(|v| *v >= 10.0), self.gif_dialog.as_mut()) {
                    dialog.delay_ms = v.round().min(600_000.0) as u32;
                }
            }
            FieldId::TweenSteps => {
                if let (Some(v), Some(dialog)) = (number.filter(|v| *v >= 1.0), self.tween_dialog.as_mut()) {
                    dialog.0 = v.round().min(500.0) as u32;
                }
            }
            FieldId::ArrowSize => {
                if let (Some(v), Some(dialog)) = (number.filter(|v| *v >= 1.0), self.arrow_dialog.as_mut()) {
                    dialog.2 = v.min(500.0);
                }
            }
            FieldId::FrameSize => {
                if let (Some(v), Some(width)) = (number.filter(|v| *v >= 1.0), self.frame_dialog.as_mut()) {
                    *width = v.min(2000.0);
                }
            }
            FieldId::CommandName => {
                if let Some((name, _)) = self.command_name.as_mut().filter(|_| !cleared) {
                    *name = text.to_string();
                }
            }
            FieldId::ReverseState(last) => {
                let states = self.ed.doc.page().states.len() as f32;
                let (Some(v), Some(range)) = (number.filter(|v| *v >= 1.0), self.reverse_dialog.as_mut()) else { return };
                let v = v.round().min(states) as u32;
                if last {
                    range.1 = v;
                } else {
                    range.0 = v;
                }
            }
            FieldId::TemplateName => {
                if let Some(name) = self.template_name.as_mut().filter(|_| !cleared) {
                    *name = text.to_string();
                }
            }
            FieldId::NewSymbolName => {
                if let Some((name, _)) = self.symbol_dialog.as_mut().filter(|_| !cleared) {
                    *name = text.to_string();
                }
            }
            FieldId::Animate(which) => {
                let (Some(v), Some(s)) = (number.filter(|v| *v >= 0.0), self.animate_dialog.as_mut()) else { return };
                match which {
                    0 => s.states = v.round().clamp(1.0, 250.0) as u32,
                    1 => s.distance = v.min(10000.0),
                    2 => s.direction = v % 360.0,
                    3 => s.scale = v.min(25000.0),
                    4 => s.opacity_from = v.min(100.0),
                    5 => s.opacity_to = v.min(100.0),
                    _ => s.rotation = v.min(36000.0),
                }
            }
            FieldId::PrefUndo | FieldId::PrefPick | FieldId::PrefSnap | FieldId::PrefGridX | FieldId::PrefGridY => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                match field.id {
                    FieldId::PrefUndo => self.prefs.undo_steps = v.round().clamp(1.0, 1000.0) as u32,
                    FieldId::PrefPick => self.prefs.pick_distance = v.min(10.0),
                    FieldId::PrefSnap => self.prefs.snap_distance = v.min(10.0),
                    FieldId::PrefGridX => self.prefs.grid.0 = v.min(2000.0),
                    _ => self.prefs.grid.1 = v.min(2000.0),
                }
            }
            FieldId::SpellChange => {
                if let Some(session) = self.spell.as_mut() {
                    session.replacement = text.to_string();
                }
            }
            FieldId::PathOffset => {
                let Some(v) = number else { return };
                self.change_text_style(|style| style.path_offset = v);
            }
            FieldId::FillFeather => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                self.change_fill_look(|look| look.feather = v.min(100.0));
            }
            FieldId::TextureAmount => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                self.change_fill_look(|look| look.texture_amount = v.min(100.0));
            }
            FieldId::GuidePosition => {
                let (Some(v), Some(dialog)) = (number, self.guide_dialog.as_mut()) else { return };
                dialog.2 = v;
            }
            FieldId::ScrubRate => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.tool_options.rate = v.min(100.0);
            }
            FieldId::PathPrecision => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.tool_options.precision = v.round().min(10.0);
            }
            FieldId::FreeformSize => {
                let Some(v) = number.filter(|v| *v >= 2.0) else { return };
                self.freeform_size = v.min(MAX_BRUSH_SIZE);
            }
            FieldId::Url => {
                let link = text.to_string();
                self.ed.for_selected(|o| match &mut o.shape {
                    Shape::Web { url, .. } => url.clone_from(&link),
                    // A button carries its link itself.
                    Shape::Instance { .. } => o.link.clone_from(&link),
                    _ => {}
                });
                self.ed.commit(t("Hotspot Link"));
            }
            FieldId::Opacity => {
                let Some(v) = number else { return };
                let v = v.clamp(0.0, 100.0) / 100.0;
                self.ed.for_selected(|o| o.opacity = v);
                self.ed.commit(t("Opacity"));
            }
            FieldId::FillHex if color.is_some() || cleared => self.apply_color(ColorTarget::Fill, color),
            FieldId::StrokeHex if color.is_some() || cleared => self.apply_color(ColorTarget::Stroke, color),
            FieldId::CanvasHex if color.is_some() || cleared => self.apply_color(ColorTarget::Canvas, color),
            FieldId::RenamePage(i) if !cleared => {
                if let Some(page) = self.ed.doc.pages.get_mut(i) {
                    page.name = text.to_string();
                }
                self.ed.commit(t("Rename Page"));
            }
            FieldId::RenameState(i) if !cleared => {
                if let Some(state) = self.ed.doc.page_mut().states.get_mut(i) {
                    state.name = text.to_string();
                }
                self.ed.commit(t("Rename State"));
            }
            FieldId::RenameLayer(i) if !cleared => {
                if let Some(layer) = self.ed.doc.page_mut().layers.get_mut(i) {
                    layer.name = text.to_string();
                }
                self.ed.commit(t("Rename Layer"));
            }
            FieldId::RenameObject(id) if !cleared => {
                if let Some(object) = self.ed.object_mut(id) {
                    object.name = text.to_string();
                }
                // In an animation the object is on other keyframes too: it has one name on all of them.
                let name = text.to_string();
                self.ed.doc.page_mut().edit_timeline(|timeline| {
                    for object in timeline.layers.iter_mut().flatten().flat_map(|span| span.objects.iter_mut()).filter(|o| o.id == id) {
                        object.name = name.clone();
                    }
                });
                self.ed.commit(t("Rename Object"));
            }
            FieldId::RenameSymbol(id) if !cleared => {
                if let Some(symbol) = self.ed.doc.symbols.iter_mut().find(|s| s.id == id) {
                    symbol.name = text.to_string();
                }
                self.ed.commit(t("Rename Symbol"));
            }
            FieldId::StateDelay(i) => {
                let Some(v) = number else { return };
                if let Some(state) = self.ed.doc.page_mut().states.get_mut(i) {
                    state.delay_ms = (v.round() as u32).clamp(MIN_STATE_DELAY, MAX_STATE_DELAY);
                }
                self.ed.commit(t("State Delay"));
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
                    o.refit_placed();
                });
                self.ed.commit(t("Auto Shape"));
            }
            FieldId::PathAmount => {
                let Some(v) = number.filter(|v| *v > 0.0) else { return };
                self.path_amount = v;
            }
            FieldId::OptimizeQuality => {
                let Some(v) = number else { return };
                self.set_export(|s| s.quality = v.clamp(1.0, 100.0) as u8);
            }
            FieldId::FilterValue(setting) => {
                let Some(v) = number else { return };
                self.set_filter_value(setting, v);
            }
            FieldId::PrintMargin(side) => {
                let Some(v) = number.filter(|v| *v >= 0.0) else { return };
                self.print_setup.margins[side.min(3)] = v.min(100.0);
            }
            FieldId::FillScale => {
                let Some(v) = number.filter(|v| *v >= 1.0) else { return };
                self.change_fill_style(true, |style| style.scale = v.min(1000.0));
            }
            FieldId::PrintFrom | FieldId::PrintTo => {
                let pages = self.ed.doc.pages.len() as f32;
                let (Some(v), Some(dialog)) = (number.filter(|v| *v >= 1.0), self.print_dialog.as_mut()) else { return };
                let page = v.round().min(pages) as u32;
                if field.id == FieldId::PrintFrom { dialog.from = page } else { dialog.to = page }
            }
            FieldId::PrintCopies => {
                let (Some(v), Some(dialog)) = (number.filter(|v| *v >= 1.0), self.print_dialog.as_mut()) else { return };
                dialog.copies = v.round().min(999.0) as u32;
            }
            FieldId::MarqueeAmount => {
                let (Some(v), Some(dialog)) = (number.filter(|v| *v >= 1.0), self.marquee_dialog.as_mut()) else { return };
                dialog.1 = v.round().min(1000.0);
            }
            FieldId::GradientPlace => {
                let Some(v) = number else { return };
                self.change_picked_stop(|stop| stop.0 = (v / 100.0).clamp(0.0, 1.0));
            }
            FieldId::GradientOpacity => {
                let Some(v) = number else { return };
                self.change_picked_stop(|stop| stop.1.a = (v.clamp(0.0, 100.0) * 2.55).round() as u8);
            }
            FieldId::OptimizeSelective => {
                let Some(v) = number else { return };
                self.set_export(|s| s.selective_quality = v.clamp(1.0, 100.0) as u8);
            }
            FieldId::OptimizeSmoothing => {
                let Some(v) = number else { return };
                self.set_export(|s| s.smoothing = v.clamp(0.0, crate::optimize::MAX_SMOOTHING as f32) as u8);
            }
            FieldId::OptimizeColors => {
                let Some(v) = number else { return };
                self.set_export(|s| s.colors = (v as u16).clamp(MIN_COLORS, MAX_COLORS));
            }
            FieldId::Mixer(channel) => {
                let Some(v) = number else { return };
                self.set_mixer_channel(channel, v.clamp(0.0, 255.0) as u8);
            }
            FieldId::NumericA | FieldId::NumericB => {
                let (Some(v), Some(dialog)) = (number, self.numeric_dialog.as_mut()) else { return };
                let first = field.id == FieldId::NumericA;
                match dialog.mode {
                    NumericMode::Rotate => dialog.angle = v,
                    NumericMode::Scale if v > 0.0 => {
                        dialog.percent = if dialog.constrain { (v, v) } else if first { (v, dialog.percent.1) } else { (dialog.percent.0, v) };
                    }
                    NumericMode::Resize if v > 0.0 => {
                        let (w, h) = (dialog.bounds.w, dialog.bounds.h);
                        dialog.size = match (first, dialog.constrain && w > 0.0 && h > 0.0) {
                            (true, true) => (v, v * h / w),
                            (false, true) => (v * w / h, v),
                            (true, false) => (v, dialog.size.1),
                            (false, false) => (dialog.size.0, v),
                        };
                    }
                    _ => {}
                }
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
                    self.ed.commit(t("Add URL"));
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
            ColorTarget::JpegMask => Some(self.ed.doc.page().jpeg_mask_color.unwrap_or(JPEG_MASK_COLOR)),
            ColorTarget::Filter => self.edited_filter().and_then(|(_, filter)| filter.color.or(filter.kind.start_color())),
            ColorTarget::PsEffect(effect) => self.edited_filter().map(|(_, filter)| filter.ps_color(effect)),
            ColorTarget::GradientStop => self.picked_stop().map(|stop| Color { a: 255, ..stop.1 }),
            ColorTarget::ReplaceFrom => self.tool_options.replace_from,
            ColorTarget::Guides => Some(self.guide_color),
            ColorTarget::Grid => Some(self.prefs.grid_color),
            ColorTarget::ReplaceTo => Some(self.tool_options.replace_to),
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
                self.ed.commit(t("Fill"));
            }
            ColorTarget::Stroke => {
                self.stroke = color;
                let width = self.stroke_width;
                self.ed.for_selected(|o| {
                    if o.takes_color() {
                        o.stroke = color.map(|color| Stroke { color, width: o.stroke.map_or(width, |s| s.width) });
                    }
                });
                self.ed.commit(t("Stroke"));
            }
            ColorTarget::Canvas => {
                self.ed.doc.page_mut().canvas = color;
                self.ed.commit(t("Canvas Color"));
            }
            ColorTarget::Matte => self.set_export(|s| s.matte = color),
            ColorTarget::Filter => self.edit_filter(true, |filter| filter.color = color),
            ColorTarget::PsEffect(effect) => {
                // An effect always has a color: "none" leaves it as it was.
                if let Some(color) = color {
                    self.edit_filter(true, |filter| filter.set_ps_color(effect, color));
                }
            }
            // "None" for the color to change means: the one the stroke starts on.
            ColorTarget::ReplaceFrom => self.tool_options.replace_from = color,
            ColorTarget::Guides => self.guide_color = color.unwrap_or(self.guide_color),
            ColorTarget::Grid => self.prefs.grid_color = color.unwrap_or(self.prefs.grid_color),
            ColorTarget::ReplaceTo => self.tool_options.replace_to = color.unwrap_or(self.tool_options.replace_to),
            ColorTarget::GradientStop => {
                // The stop keeps how opaque it is; only its color changes.
                if let Some(color) = color {
                    self.change_picked_stop(|stop| stop.1 = Color { a: stop.1.a, ..color });
                }
            }
            ColorTarget::JpegMask => {
                self.ed.doc.page_mut().jpeg_mask_color = color;
                self.ed.commit(t("Optimize"));
            }
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

/// The outline of the oval inside the box between two corners, as the corners of a many-sided polygon.
fn oval_points(a: Vec2, b: Vec2) -> Vec<Vec2> {
    let (cx, cy, rx, ry) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.x - b.x).abs() / 2.0, (a.y - b.y).abs() / 2.0);
    (0..96).map(|step| step as f32 / 96.0 * std::f32::consts::TAU).map(|turn| v2(cx + rx * turn.cos(), cy + ry * turn.sin())).collect()
}

/// Where a filter held at `held` in the list is once the mouse is at `at`: one place
/// further for each whole row crossed, with the hold moved along so the rest of the drag counts from there.
fn dragged_place(mut index: usize, mut held: Point<Pixels>, at: Point<Pixels>, count: usize) -> (usize, Point<Pixels>) {
    const ROW: f32 = 17.0;
    loop {
        let dy = f32::from(at.y - held.y);
        if dy >= ROW && index + 1 < count {
            (index, held.y) = (index + 1, held.y + px(ROW));
        } else if dy <= -ROW && index > 0 {
            (index, held.y) = (index - 1, held.y - px(ROW));
        } else {
            return (index, held);
        }
    }
}

/// The red, green and blue of an object at a place on the page, from 0 to 255, as it
/// looks with only the filters before the one at `index`. Nothing where it is clear.
fn sample_before(object: &Object, index: usize, at: Vec2) -> Option<[f32; 3]> {
    let mut object = object.clone();
    object.filters.truncate(index);
    let (bitmap, area) = crate::export::render_object(&object)?;
    let (x, y) = ((at.x - area.x).floor(), (at.y - area.y).floor());
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let [r, g, b, a] = bitmap.pixel(x as u32, y as u32)?;
    (a > 0).then_some([r as f32, g as f32, b as f32])
}

/// Counts how many pixels of an object have each level of red, green and blue, as it
/// looks with only the filters before the one at `index`.
fn histogram_before(object: &Object, index: usize) -> Option<[[u32; 256]; 3]> {
    let mut object = object.clone();
    object.filters.truncate(index);
    let (bitmap, _) = crate::export::render_object(&object)?;
    let mut counts = [[0u32; 256]; 3];
    for px in bitmap.rgba().chunks_exact(4).filter(|px| px[3] > 0) {
        for channel in 0..3 {
            counts[channel][px[channel] as usize] += 1;
        }
    }
    Some(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moved_or_scaled_box_is_still_upright() {
        let origin = Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0 };
        assert_eq!(upright(rect_corners(origin)), Some(origin));
        let grown = Warp::fit(origin, Rect { x: 0.0, y: 0.0, w: 200.0, h: 25.0 });
        assert_eq!(upright(rect_corners(origin).map(|c| grown.apply(c))), Some(Rect { x: 0.0, y: 0.0, w: 200.0, h: 25.0 }));
        // Turned, slanted or flipped boxes are not.
        let turned = Warp::rotation(origin.center(), 0.3);
        assert_eq!(upright(rect_corners(origin).map(|c| turned.apply(c))), None);
        let slanted = Warp::shear_x(origin.bottom(), 0.5);
        assert_eq!(upright(rect_corners(origin).map(|c| slanted.apply(c))), None);
        let half_turn = Warp::rotation(origin.center(), std::f32::consts::PI);
        assert_eq!(upright(rect_corners(origin).map(|c| half_turn.apply(c))), None);
    }

    #[test]
    fn numeric_scaling_keeps_the_center_point_still() {
        let bounds = Rect { x: 100.0, y: 100.0, w: 200.0, h: 100.0 };
        // Around the middle, the box shrinks toward the middle.
        assert_eq!(scaled_about(bounds, bounds.center(), 0.5, 0.5), Rect { x: 150.0, y: 125.0, w: 100.0, h: 50.0 });
        // Around its top-left corner, that corner does not move.
        assert_eq!(scaled_about(bounds, v2(100.0, 100.0), 0.5, 2.0), Rect { x: 100.0, y: 100.0, w: 100.0, h: 200.0 });
        // Around a point outside the box, the box moves toward the point as it shrinks.
        assert_eq!(scaled_about(bounds, v2(0.0, 0.0), 0.5, 0.5), Rect { x: 50.0, y: 50.0, w: 100.0, h: 50.0 });
    }

    #[test]
    fn transform_handles_sit_on_corners_and_side_middles() {
        let quad = rect_corners(Rect { x: 0.0, y: 0.0, w: 10.0, h: 4.0 });
        let handles = quad_handles(quad);
        assert_eq!(handles[0], v2(0.0, 0.0));
        assert_eq!(handles[1], v2(5.0, 0.0));
        assert_eq!(handles[3], v2(10.0, 2.0));
        assert_eq!(handles[7], v2(0.0, 2.0));
        assert_eq!(quad_center(quad), v2(5.0, 2.0));
    }

    #[test]
    fn a_session_box_follows_its_map_and_turns_around_its_middle() {
        let origin = Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 };
        let mut session = TransformSession { origin, warp: Warp::translation(5.0, 0.0), originals: Vec::new(), pivot: None };
        assert_eq!(session.quad()[0], v2(5.0, 0.0));
        assert_eq!(session.pivot(), v2(10.0, 5.0));
        session.pivot = Some(v2(1.0, 2.0));
        assert_eq!(session.pivot(), v2(1.0, 2.0));
    }
    use gpui::{point, px, size};

    #[test]
    fn a_dragged_filter_moves_one_place_for_each_row_crossed() {
        let at = |y: f32| point(px(100.), px(y));
        // Less than a row moves nothing.
        assert_eq!(dragged_place(0, at(100.), at(110.), 3), (0, at(100.)));
        // Two rows down, the first of three ends up last, and the hold moves with it.
        assert_eq!(dragged_place(0, at(100.), at(136.), 3), (2, at(134.)));
        // Further down there is nowhere to go; back up a row it changes place again.
        assert_eq!(dragged_place(2, at(134.), at(300.), 3), (2, at(134.)));
        assert_eq!(dragged_place(2, at(134.), at(117.), 3), (1, at(117.)));
        assert_eq!(dragged_place(1, at(117.), at(0.), 3), (0, at(100.)));
    }

    #[test]
    fn levels_are_counted_from_the_object_before_the_filter() {
        let pixels = crate::bitmap::Bitmap::from_rgba(40, 30, [200, 100, 50, 255].repeat(40 * 30)).expect("the sizes agree");
        let mut object = Object::new_bitmap(1, "Bitmap", pixels, 10.0, 10.0);
        object.filters = vec![LiveFilter::new(FilterKind::Invert), LiveFilter::new(FilterKind::Levels)];
        // The Levels filter, second in the list, receives the inverted fill.
        let counts = histogram_before(&object, 1).expect("the object draws");
        assert!(counts[0][55] > 1000 && counts[1][155] > 1000 && counts[2][205] > 1000, "{:?}", counts[0].iter().position(|n| *n > 0));
        // The first filter receives the fill as it is.
        let counts = histogram_before(&object, 0).expect("the object draws");
        assert!(counts[0][200] > 1000 && counts[0][55] == 0);
        // An eyedropper picks the same colors, and nothing beside the object.
        assert_eq!(sample_before(&object, 1, v2(20.5, 20.5)), Some([55.0, 155.0, 205.0]));
        assert_eq!(sample_before(&object, 0, v2(49.0, 39.0)), Some([200.0, 100.0, 50.0]));
        assert_eq!(sample_before(&object, 0, v2(5.0, 20.0)), None);
        assert_eq!(sample_before(&object, 0, v2(60.0, 20.0)), None);
    }

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
    fn about_fits_its_box_in_both_languages_and_says_where_the_site_and_the_code_are() {
        use crate::i18n::Language;
        for language in [Language::English, Language::Turkish] {
            for sahne in [false, true] {
                let about = about_text_in(language, sahne);
                assert!(about.contains("https://muratsapmaz.web.app/") && about.contains("github.com/MuratSAPMAZ/Pisiworks") && about.contains("muratsapmaz.github.io/PisiAnimasyon") && about.contains("Med Amine Belbachir") && about.contains("github.com/gregstein/emberframe"), "{about}");
                // The box cuts off what is wider than it: every line is short enough to be read whole.
                assert!(about.lines().all(|line| line.chars().count() <= 36), "{about}");
            }
        }
        assert_eq!(about_text_in(Language::Turkish, true).lines().nth(2), Some("Pardus için animasyon programı;"));
        // Its buttons open the site and the code, where they really are.
        assert_eq!((SITE, SOURCE, HOME), ("https://muratsapmaz.web.app/", "https://github.com/MuratSAPMAZ/Pisiworks", "https://muratsapmaz.github.io/PisiAnimasyon/"));
    }

    #[test]
    fn file_type_is_detected_from_content() {
        let bitmap = Bitmap::from_rgba(2, 2, vec![200; 16]).unwrap();
        let png = bitmap.encode_png().unwrap();
        assert!(matches!(parse_file(&png), Ok(Opened::Image(b)) if b == bitmap));
        // About names the program's own maker and thanks the author of what it is built on.
        let about = about_text();
        assert!(about.contains("Murat SAPMAZ") && about.contains("github.com/MuratSAPMAZ/Pisiworks"));
        assert!(about.contains("Med Amine Belbachir") && about.contains("github.com/gregstein/emberframe"));
        // Make a GIF: two pictures of different sizes become two states on a canvas that holds the larger, played three times.
        let dir = std::env::temp_dir().join(format!("pisiworks-gif-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (small, large) = (dir.join("1.png"), dir.join("2.png"));
        std::fs::write(&small, Bitmap::from_rgba(2, 2, vec![255; 16]).unwrap().encode_png().unwrap()).unwrap();
        std::fs::write(&large, Bitmap::from_rgba(6, 4, vec![9; 96]).unwrap().encode_png().unwrap()).unwrap();
        let animation = animation_from_files(&[small.clone(), large], 200, 3).unwrap();
        let page = &animation.pages[0];
        assert_eq!((page.width, page.height, page.states.len(), page.gif_loops, page.states[1].delay_ms), (6, 4, 2, 3, 200));
        assert_eq!(page.objects().next().unwrap().bounds, Rect { x: 2.0, y: 1.0, w: 2.0, h: 2.0 }, "the smaller picture is in the middle");
        let page = Editor::new(animation).export_page();
        let (frames, loops) = crate::optimize::gif_frames(&crate::optimize::encode(&page, &page.export).unwrap()).unwrap();
        assert_eq!((frames.len(), loops), (2, 3));
        assert!(animation_from_files(&[small.clone()], 200, 0).is_err() && animation_from_files(&[small, dir.join("none.png")], 200, 0).is_err());
        std::fs::remove_dir_all(&dir).ok();
        // Arrowheads point where the path ends, whichever way it runs.
        let (closed, triangle) = arrow_head(0, v2(10.0, 0.0), v2(0.0, 0.0), 10.0);
        assert!(closed);
        assert_eq!(triangle, [v2(10.0, 0.0), v2(0.0, 4.0), v2(0.0, -4.0)]);
        let (closed, lines) = arrow_head(1, v2(0.0, 10.0), v2(0.0, 0.0), 10.0);
        assert!(!closed && lines[1] == v2(0.0, 10.0) && (lines[0].y - 0.0).abs() < 0.001 && (lines[0].x + 4.0).abs() < 0.001);
        let (_, circle) = arrow_head(3, v2(10.0, 0.0), v2(0.0, 0.0), 10.0);
        assert!(circle.iter().all(|p| ((p.x - 5.0).hypot(p.y) - 5.0).abs() < 0.001));

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
