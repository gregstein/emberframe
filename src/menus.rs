//! Tool palette and menu definitions. An entry with no command is shown disabled.

use crate::bitmap::Filter;
use crate::paths::PathOp;
use crate::model::{Align, Arrange};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pointer,
    Subselect,
    Crop,
    Scale,
    Eraser,
    Marquee,
    Lasso,
    Wand,
    Blur,
    Stamp,
    Freeform,
    Knife,
    Hotspot,
    Slice,
    Rect,
    Ellipse,
    Line,
    Pen,
    Pencil,
    Brush,
    Text,
    Eyedropper,
    Bucket,
    Hand,
    Zoom,
}

pub struct ToolSlot {
    pub label: &'static str,
    pub icon: &'static str,
    pub action: SlotAction,
}

#[derive(Clone, Copy)]
pub enum SlotAction {
    Select(Tool),
    Run(Cmd),
}

const fn on(label: &'static str, icon: &'static str, tool: Tool) -> ToolSlot {
    ToolSlot { label, icon, action: SlotAction::Select(tool) }
}

const fn run(label: &'static str, icon: &'static str, cmd: Cmd) -> ToolSlot {
    ToolSlot { label, icon, action: SlotAction::Run(cmd) }
}

pub const TOOL_SECTIONS: &[(&str, &[ToolSlot])] = &[
    (
        "Select",
        &[
            on("Pointer (V)", "pointer", Tool::Pointer),
            on("Subselection (A)", "subselect", Tool::Subselect),
            on("Scale (Q)", "scale", Tool::Scale),
            on("Crop (C)", "crop", Tool::Crop),
        ],
    ),
    (
        "Bitmap",
        &[
            on("Marquee (M)", "marquee", Tool::Marquee),
            on("Lasso (L)", "lasso", Tool::Lasso),
            on("Magic Wand (W)", "wand", Tool::Wand),
            on("Brush (B)", "brush", Tool::Brush),
            on("Pencil (Y)", "pencil", Tool::Pencil),
            on("Eraser (E)", "eraser", Tool::Eraser),
            on("Blur (R)", "blur", Tool::Blur),
            on("Rubber Stamp (S): Alt+click sets the source", "stamp", Tool::Stamp),
        ],
    ),
    (
        "Vector",
        &[
            on("Line (N)", "line", Tool::Line),
            on("Pen (P)", "pen", Tool::Pen),
            on("Rectangle (R)", "rect", Tool::Rect),
            on("Ellipse (U)", "ellipse", Tool::Ellipse),
            on("Text (T)", "text", Tool::Text),
            on("Freeform (O)", "freeform", Tool::Freeform),
            on("Knife (Y)", "knife", Tool::Knife),
        ],
    ),
    (
        "Web",
        &[
            on("Hotspot (J)", "hotspot", Tool::Hotspot),
            on("Slice (K)", "slice", Tool::Slice),
            run("Hide Slices and Hotspots (2)", "hide", Cmd::ShowWeb(false)),
            run("Show Slices and Hotspots (2)", "show", Cmd::ShowWeb(true)),
        ],
    ),
];

pub const COLOR_TOOLS: &[ToolSlot] = &[on("Eyedropper (I)", "eyedropper", Tool::Eyedropper), on("Paint Bucket (G)", "bucket", Tool::Bucket)];

pub const SCREEN_MODES: &[ToolSlot] = &[
    run("Standard Screen", "screen-standard", Cmd::ScreenMode(ScreenMode::Standard)),
    run("Full Screen with Menus", "screen-menus", Cmd::ScreenMode(ScreenMode::FullWithMenus)),
    run("Full Screen (F4 brings the panels back)", "screen-full", Cmd::ScreenMode(ScreenMode::Full)),
];

pub const VIEW_TOOLS: &[ToolSlot] = &[on("Hand (H)", "hand", Tool::Hand), on("Zoom (Z)", "zoom", Tool::Zoom)];

pub const ZOOM_LEVELS: &[u32] = &[6, 12, 25, 50, 66, 100, 150, 200, 300, 400, 800, 1600, 3200, 6400];

#[derive(Clone, Copy, Debug)]
pub enum Cmd {
    New,
    Open,
    Save,
    SaveAs,
    Import,
    Export,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Clear,
    Duplicate,
    ZoomIn,
    ZoomOut,
    ZoomTo(u32),
    FitAll,
    Rulers,
    Grid,
    SelectAll,
    Deselect,
    Arrange(Arrange),
    Align(Align),
    FlipH,
    FlipV,
    Filter(Filter),
    ShowWeb(bool),
    AddLayer,
    AddPage,
    About,
    Panel(Panel),
    HidePanels,
    ConvertToSymbol,
    BreakApart,
    PathOp(PathOp),
    Group,
    Ungroup,
    Rotate { clockwise: bool },
    CanvasSize,
    ImageSize,
    FitCanvas,
    ScreenMode(ScreenMode),
    FirstState,
    PreviousState,
    NextState,
    LastState,
    PlayStates,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenMode {
    Standard,
    FullWithMenus,
    Full,
}

/// Every window listed in the Window menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Toolbar,
    Tools,
    Properties,
    Optimize,
    History,
    Align,
    Pages,
    States,
    Layers,
    Styles,
    Path,
    ColorPalette,
    Swatches,
    AutoShapeProperties,
    AutoShapes,
    DocumentLibrary,
    CommonLibrary,
    Mixer,
    Info,
    Url,
    Find,
    SpecialCharacters,
    ImageEditing,
}

impl Panel {
    pub fn title(self) -> &'static str {
        match self {
            Panel::Toolbar => "Main Toolbar",
            Panel::Tools => "Tools",
            Panel::Properties => "Properties",
            Panel::Optimize => "Optimize",
            Panel::History => "History",
            Panel::Align => "Align",
            Panel::Pages => "Pages",
            Panel::States => "States",
            Panel::Layers => "Layers",
            Panel::Styles => "Styles",
            Panel::Path => "Path",
            Panel::ColorPalette => "Color Palette",
            Panel::Swatches => "Swatches",
            Panel::AutoShapeProperties => "Auto Shape Properties",
            Panel::AutoShapes => "Auto Shapes",
            Panel::DocumentLibrary => "Document Library",
            Panel::CommonLibrary => "Common Library",
            Panel::Mixer => "Color Mixer",
            Panel::Info => "Info",
            Panel::Url => "URL",
            Panel::Find => "Find",
            Panel::SpecialCharacters => "Special Characters",
            Panel::ImageEditing => "Image Editing",
        }
    }
}

/// Tab groups of the right-hand dock, top to bottom.
pub const DOCK_GROUPS: [&[Panel]; 7] = [
    &[Panel::Optimize, Panel::History, Panel::Align],
    &[Panel::Pages, Panel::States, Panel::Layers],
    &[Panel::Styles, Panel::ColorPalette, Panel::Swatches],
    &[Panel::SpecialCharacters, Panel::ImageEditing, Panel::Path, Panel::AutoShapes],
    &[Panel::DocumentLibrary, Panel::CommonLibrary],
    &[Panel::Mixer, Panel::Info, Panel::Url],
    &[Panel::AutoShapeProperties, Panel::Find],
];

/// Window-menu shortcuts. These work even while a field is being edited.
pub fn panel_shortcut(key: &str, control: bool, shift: bool, alt: bool) -> Option<Cmd> {
    let panel = match (key, control, shift, alt) {
        ("f4", false, false, false) => return Some(Cmd::HidePanels),
        ("f8", false, false, false) => return Some(Cmd::ConvertToSymbol),
        ("f2", true, false, false) => Panel::Tools,
        ("f3", true, false, false) => Panel::Properties,
        ("f6", false, false, false) => Panel::Optimize,
        ("f2", false, false, false) => Panel::Layers,
        ("f5", false, false, false) => Panel::Pages,
        ("f2", false, true, false) => Panel::States,
        ("f10", false, true, false) => Panel::History,
        ("f11", true, false, false) => Panel::Styles,
        ("f11", false, false, false) => Panel::DocumentLibrary,
        ("f7", false, false, false) => Panel::CommonLibrary,
        ("f10", false, true, true) => Panel::Url,
        ("f9", false, true, false) => Panel::Mixer,
        ("f9", true, false, false) => Panel::Swatches,
        ("f12", false, true, true) => Panel::Info,
        _ => return None,
    };
    Some(Cmd::Panel(panel))
}

pub enum Entry {
    Item(&'static str, &'static str, Option<Cmd>),
    Sep,
    Sub(&'static str, &'static [Entry]),
}

use Entry::{Item, Sep, Sub};

const fn cmd(label: &'static str, keys: &'static str, c: Cmd) -> Entry {
    Item(label, keys, Some(c))
}

const fn na(label: &'static str) -> Entry {
    Item(label, "", None)
}

pub const MENUS: &[(&str, &[Entry])] = &[
    (
        "File",
        &[
            cmd("New", "Ctrl+N", Cmd::New),
            na("New from Template..."),
            cmd("Open...", "Ctrl+O", Cmd::Open),
            Sep,
            cmd("Save", "Ctrl+S", Cmd::Save),
            cmd("Save As...", "Ctrl+Shift+S", Cmd::SaveAs),
            na("Revert"),
            Sep,
            cmd("Import...", "Ctrl+R", Cmd::Import),
            cmd("Export...", "Ctrl+Shift+R", Cmd::Export),
            na("Image Preview..."),
            Sep,
            na("Batch Process..."),
            Sep,
            na("Page Setup..."),
            na("Print..."),
            Sep,
            cmd("Exit", "Alt+F4", Cmd::Exit),
        ],
    ),
    (
        "Edit",
        &[
            cmd("Undo", "Ctrl+Z", Cmd::Undo),
            cmd("Redo", "Ctrl+Y", Cmd::Redo),
            Sep,
            Sub(
                "Insert",
                &[
                    na("New Button..."),
                    na("New Symbol..."),
                    Sep,
                    na("Hotspot"),
                    na("Rectangular Slice"),
                    Sep,
                    na("Empty Bitmap"),
                    Sep,
                    cmd("Layer", "", Cmd::AddLayer),
                    na("State"),
                    cmd("Page", "", Cmd::AddPage),
                ],
            ),
            cmd("Find and Replace...", "Ctrl+F", Cmd::Panel(Panel::Find)),
            Sep,
            cmd("Cut", "Ctrl+X", Cmd::Cut),
            cmd("Copy", "Ctrl+C", Cmd::Copy),
            cmd("Paste", "Ctrl+V", Cmd::Paste),
            cmd("Clear", "Delete", Cmd::Clear),
            Sep,
            na("Paste as Mask"),
            na("Paste Inside"),
            na("Paste Attributes"),
            Sep,
            cmd("Duplicate", "Ctrl+Alt+D", Cmd::Duplicate),
            na("Clone"),
            Sep,
            na("Preferences..."),
            na("Keyboard Shortcuts..."),
        ],
    ),
    (
        "View",
        &[
            cmd("Zoom In", "Ctrl+=", Cmd::ZoomIn),
            cmd("Zoom Out", "Ctrl+-", Cmd::ZoomOut),
            Sub(
                "Magnification",
                &[
                    cmd("6%", "", Cmd::ZoomTo(6)),
                    cmd("12%", "", Cmd::ZoomTo(12)),
                    cmd("25%", "", Cmd::ZoomTo(25)),
                    cmd("50%", "", Cmd::ZoomTo(50)),
                    cmd("66%", "", Cmd::ZoomTo(66)),
                    cmd("100%", "Ctrl+1", Cmd::ZoomTo(100)),
                    cmd("150%", "", Cmd::ZoomTo(150)),
                    cmd("200%", "", Cmd::ZoomTo(200)),
                    cmd("300%", "", Cmd::ZoomTo(300)),
                    cmd("400%", "", Cmd::ZoomTo(400)),
                    cmd("800%", "", Cmd::ZoomTo(800)),
                    cmd("1600%", "", Cmd::ZoomTo(1600)),
                    cmd("3200%", "", Cmd::ZoomTo(3200)),
                    cmd("6400%", "", Cmd::ZoomTo(6400)),
                ],
            ),
            Sep,
            na("Fit Selection"),
            cmd("Fit All", "Ctrl+0", Cmd::FitAll),
            Sep,
            cmd("Rulers", "Ctrl+Alt+R", Cmd::Rulers),
            cmd("Show Grid", "Ctrl+Alt+G", Cmd::Grid),
            na("Guides"),
            na("Smart Guides"),
            na("Slice Guides"),
        ],
    ),
    (
        "Select",
        &[
            cmd("Select All", "Ctrl+A", Cmd::SelectAll),
            cmd("Deselect", "Ctrl+D", Cmd::Deselect),
            na("Superselect"),
            na("Subselect"),
            Sep,
            na("Select Similar"),
            na("Select Inverse"),
            na("Feather..."),
            Sep,
            na("Expand Marquee..."),
            na("Contract Marquee..."),
            na("Border Marquee..."),
            na("Smooth Marquee..."),
        ],
    ),
    (
        "Modify",
        &[
            Sub(
                "Canvas",
                &[
                    cmd("Image Size...", "", Cmd::ImageSize),
                    cmd("Canvas Size...", "", Cmd::CanvasSize),
                    na("Trim Canvas"),
                    cmd("Fit Canvas", "", Cmd::FitCanvas),
                ],
            ),
            na("Animation"),
            Sub(
                "Symbol",
                &[cmd("Convert to Symbol", "F8", Cmd::ConvertToSymbol), cmd("Break Apart", "Ctrl+B", Cmd::BreakApart)],
            ),
            na("Mask"),
            Sep,
            na("Flatten Selection"),
            na("Merge Down"),
            na("Flatten Layers"),
            Sep,
            Sub(
                "Transform",
                &[
                    na("Free Transform"),
                    na("Numeric Transform..."),
                    Sep,
                    cmd("Rotate 90 CW", "", Cmd::Rotate { clockwise: true }),
                    cmd("Rotate 90 CCW", "", Cmd::Rotate { clockwise: false }),
                    Sep,
                    cmd("Flip Horizontal", "", Cmd::FlipH),
                    cmd("Flip Vertical", "", Cmd::FlipV),
                ],
            ),
            Sub(
                "Arrange",
                &[
                    cmd("Bring to Front", "Ctrl+Shift+Up", Cmd::Arrange(Arrange::Front)),
                    cmd("Bring Forward", "Ctrl+Up", Cmd::Arrange(Arrange::Forward)),
                    cmd("Send Backward", "Ctrl+Down", Cmd::Arrange(Arrange::Backward)),
                    cmd("Send to Back", "Ctrl+Shift+Down", Cmd::Arrange(Arrange::Back)),
                ],
            ),
            Sub(
                "Align",
                &[
                    cmd("Left", "", Cmd::Align(Align::Left)),
                    cmd("Center Vertical", "", Cmd::Align(Align::CenterX)),
                    cmd("Right", "", Cmd::Align(Align::Right)),
                    Sep,
                    cmd("Top", "", Cmd::Align(Align::Top)),
                    cmd("Center Horizontal", "", Cmd::Align(Align::CenterY)),
                    cmd("Bottom", "", Cmd::Align(Align::Bottom)),
                ],
            ),
            Sep,
            Sub(
                "Combine Paths",
                &[
                    cmd("Union", "", Cmd::PathOp(PathOp::Union)),
                    cmd("Intersect", "", Cmd::PathOp(PathOp::Intersect)),
                    cmd("Punch", "", Cmd::PathOp(PathOp::Punch)),
                    cmd("Crop", "", Cmd::PathOp(PathOp::Crop)),
                    cmd("Join", "", Cmd::PathOp(PathOp::Join)),
                    cmd("Split", "", Cmd::PathOp(PathOp::Split)),
                ],
            ),
            Sub(
                "Alter Path",
                &[
                    cmd("Simplify", "", Cmd::PathOp(PathOp::Simplify)),
                    cmd("Expand Stroke", "", Cmd::PathOp(PathOp::ExpandStroke)),
                    cmd("Inset", "", Cmd::PathOp(PathOp::Inset)),
                    cmd("Outset", "", Cmd::PathOp(PathOp::Outset)),
                ],
            ),
            Sep,
            cmd("Group", "Ctrl+G", Cmd::Group),
            cmd("Ungroup", "Ctrl+Shift+G", Cmd::Ungroup),
        ],
    ),
    ("Text", &[na("Font"), na("Size"), na("Style"), na("Align"), Sep, na("Attach to Path"), na("Convert to Paths"), Sep, na("Check Spelling...")]),
    ("Commands", &[na("Manage Saved Commands..."), na("Run Script...")]),
    (
        "Filters",
        &[
            na("Repeat Filter"),
            Sep,
            Sub(
                "Adjust Color",
                &[
                    cmd("Brighten", "", Cmd::Filter(Filter::Brighten)),
                    cmd("Darken", "", Cmd::Filter(Filter::Darken)),
                    cmd("Grayscale", "", Cmd::Filter(Filter::Grayscale)),
                    cmd("Invert", "", Cmd::Filter(Filter::Invert)),
                ],
            ),
            Sub(
                "Blur and Sharpen",
                &[cmd("Gaussian Blur", "", Cmd::Filter(Filter::Blur)), cmd("Sharpen", "", Cmd::Filter(Filter::Sharpen))],
            ),
        ],
    ),
    (
        "Window",
        &[
            cmd("Hide Panels", "F4", Cmd::HidePanels),
            Sep,
            cmd("Main Toolbar", "", Cmd::Panel(Panel::Toolbar)),
            cmd("Tools", "Ctrl+F2", Cmd::Panel(Panel::Tools)),
            cmd("Properties", "Ctrl+F3", Cmd::Panel(Panel::Properties)),
            Sep,
            cmd("Optimize", "F6", Cmd::Panel(Panel::Optimize)),
            cmd("Layers", "F2", Cmd::Panel(Panel::Layers)),
            cmd("Pages", "F5", Cmd::Panel(Panel::Pages)),
            cmd("States", "Shift+F2", Cmd::Panel(Panel::States)),
            cmd("History", "Shift+F10", Cmd::Panel(Panel::History)),
            cmd("Auto Shapes", "", Cmd::Panel(Panel::AutoShapes)),
            cmd("Styles", "Ctrl+F11", Cmd::Panel(Panel::Styles)),
            cmd("Document Library", "F11", Cmd::Panel(Panel::DocumentLibrary)),
            cmd("Common Library", "F7", Cmd::Panel(Panel::CommonLibrary)),
            cmd("URL", "Alt+Shift+F10", Cmd::Panel(Panel::Url)),
            Sep,
            cmd("Color Mixer", "Shift+F9", Cmd::Panel(Panel::Mixer)),
            cmd("Swatches", "Ctrl+F9", Cmd::Panel(Panel::Swatches)),
            cmd("Color Palette", "", Cmd::Panel(Panel::ColorPalette)),
            cmd("Info", "Alt+Shift+F12", Cmd::Panel(Panel::Info)),
            cmd("Find", "Ctrl+F", Cmd::Panel(Panel::Find)),
            cmd("Align", "", Cmd::Panel(Panel::Align)),
            cmd("Auto Shape Properties", "", Cmd::Panel(Panel::AutoShapeProperties)),
            cmd("Path", "", Cmd::Panel(Panel::Path)),
            cmd("Special Characters", "", Cmd::Panel(Panel::SpecialCharacters)),
            cmd("Image Editing", "", Cmd::Panel(Panel::ImageEditing)),
        ],
    ),
    ("Help", &[cmd("About Emberframe", "", Cmd::About)]),
];
