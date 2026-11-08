//! Original icons, served to GPUI as in-memory SVG assets.
//!
//! Plain icons are one black shape tinted when drawn. Colored icons are a stack of
//! such shapes, which keeps them crisp at every display scale and easy to grey out.

use std::borrow::Cow;

use anyhow::Result;
use gpui::{AssetSource, SharedString};

const INK: u32 = 0x1c1c1c;
const STEEL: u32 = 0x6b7a8c;
const GRAY: u32 = 0x9a9a9a;
const PAPER: u32 = 0xffffff;
const BLUE: u32 = 0x2f6fc4;
const SKY: u32 = 0x8fc0ee;
const GREEN: u32 = 0x2f9e44;
const LIME: u32 = 0x9ad66f;
const RED: u32 = 0xd23c32;
const ORANGE: u32 = 0xe8912d;
const YELLOW: u32 = 0xf2c23e;
const TAN: u32 = 0xc69c6d;
const PINK: u32 = 0xee9fb0;
const PEACH: u32 = 0xf6d3b0;

type Layer = (u32, &'static str);

/// The layers of a colored icon, painted first to last.
pub fn colored(name: &str) -> Option<&'static [Layer]> {
    Some(match name {
        "pointer" => &[(INK, r#"<path fill="black" d="M4 1.5l9 8H8.700l2.300 4.800-1.800.9-2.300-4.900L4 13.500z"/>"#)],
        "subselect" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M4 1.500l9 8H8.700l2.300 4.800-1.800.9-2.300-4.900L4 13.500z"/>"#),
            (INK, r#"<path d="M4 1.500l9 8H8.700l2.300 4.800-1.800.9-2.300-4.900L4 13.500z"/>"#),
        ],
        "scale" => &[
            (SKY, r#"<rect x="3" y="5" width="8" height="8" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="3" y="5" width="8" height="8"/><path d="M8.500 7.500L14 2M10.500 2H14v3.500"/>"#),
        ],
        "crop" => &[(INK, r#"<path stroke-width="1.700" d="M4.500 1v10.500H15M1 4.500h10.500V15"/>"#)],
        "marquee" => &[(INK, r#"<rect x="2.500" y="3.500" width="11" height="9" stroke-dasharray="2 1.600"/>"#)],
        "lasso" => &[
            (INK, r#"<ellipse cx="8" cy="6.500" rx="5.500" ry="4" stroke-dasharray="2 1.400"/>"#),
            (TAN, r#"<path stroke-width="1.500" d="M5 10c-1 1.500 0 3.500 1.500 3.500"/>"#),
        ],
        "wand" => &[
            (INK, r#"<path stroke-width="1.700" d="M2.500 14l7.500-7.500"/>"#),
            (YELLOW, r#"<path stroke-width="1.400" d="M11.500 1.500v3M11.500 7v2M8 4.500H6.500M15 4.500h-1.500M9 2.500l1 1M14 2.500l-1 1"/>"#),
        ],
        "brush" => &[
            (TAN, r#"<path fill="black" d="M14 2l-6.500 6.500-1.500-1.500L12.500.5z" transform="translate(0 .5)"/>"#),
            (BLUE, r#"<path fill="black" stroke="none" d="M6 8.500c-2 .3-2.500 2-2.700 3.200-.1.700-.7 1.300-1.300 1.500 2.500.8 5.600 0 5.900-3z"/>"#),
        ],
        "pencil" => &[
            (YELLOW, r#"<path fill="black" stroke="none" d="M3.300 10.200l7.700-7.700 2.500 2.500-7.700 7.700z"/>"#),
            (PEACH, r#"<path fill="black" stroke="none" d="M2.500 13.500l.8-3.300 2.500 2.500z"/>"#),
            (INK, r#"<path d="M2.500 13.500l.8-3.300 7.700-7.700 2.500 2.500-7.700 7.700z"/>"#),
        ],
        "eraser" => &[
            (PINK, r#"<path fill="black" stroke="none" d="M6 13.500l-3.500-3.500L9.500 3 14 7.500l-6 6z"/>"#),
            (PAPER, r#"<path fill="black" stroke="none" d="M6 13.500l-3.500-3.500 3.500-3.500L10.500 11l-2.500 2.500z"/>"#),
            (INK, r#"<path d="M6 13.500l-3.500-3.500L9.500 3 14 7.500l-6 6zM6 6.500L10.500 11"/>"#),
        ],
        "blur" => &[
            (SKY, r#"<path fill="black" stroke="none" d="M8 2c2.700 3.300 4.200 5.400 4.200 7.400a4.200 4.200 0 01-8.400 0C3.800 7.400 5.300 5.300 8 2z"/>"#),
            (BLUE, r#"<path d="M8 2c2.700 3.300 4.200 5.400 4.200 7.400a4.200 4.200 0 01-8.400 0C3.800 7.400 5.300 5.300 8 2z"/>"#),
        ],
        "stamp" => &[
            (TAN, r#"<path fill="black" stroke="none" d="M6.500 8V6.500a1.500 1.500 0 113 0V8l1 2.500h-5z"/>"#),
            (STEEL, r#"<path fill="black" stroke="none" d="M2.500 10.500h11v2.500h-11z"/>"#),
            (INK, r#"<path d="M6.500 8V6.500a1.500 1.500 0 113 0V8l1 2.500h3v2h-11v-2h3zM3 14.500h10"/>"#),
        ],
        "eyedropper" => &[
            (SKY, r#"<path fill="black" stroke="none" d="M9.500 4.500l2 2-6.500 6.500H3v-2z"/>"#),
            (INK, r#"<path d="M9.500 4.500l2 2-6.500 6.500H3v-2zM9 2.500l4.500 4.500"/>"#),
            (INK, r#"<path fill="black" d="M11 2.500a1.400 1.400 0 012.500 2.500"/>"#),
        ],
        "bucket" => &[
            (STEEL, r#"<path fill="black" stroke="none" d="M7 2.500l6 6-5 5-6-6z"/>"#),
            (ORANGE, r#"<path fill="black" stroke="none" d="M3.500 6.500h8l1.500 2-5 5-6-6z"/>"#),
            (INK, r#"<path d="M7 2.500l6 6-5 5-6-6z"/>"#),
            (BLUE, r#"<path fill="black" d="M14 11.500c0 1-.5 2-1 2s-1-1-1-2 1-2 1-2 1 1 1 2z"/>"#),
        ],
        "line" => &[(INK, r#"<path stroke-width="1.400" d="M3 13L13 3"/>"#)],
        "pen" => &[
            (STEEL, r#"<path fill="black" stroke="none" d="M8 1.500l3.500 7L9 13H7L4.500 8.500z"/>"#),
            (INK, r#"<path d="M8 1.500l3.500 7L9 13H7L4.500 8.500zM8 1.500V7M6.500 14.500h3"/><circle cx="8" cy="8" r="1"/>"#),
        ],
        "rect" => &[
            (STEEL, r#"<rect x="2.500" y="3.500" width="11" height="9" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="2.500" y="3.500" width="11" height="9"/>"#),
        ],
        "ellipse" => &[
            (STEEL, r#"<ellipse cx="8" cy="8" rx="6" ry="4.800" fill="black" stroke="none"/>"#),
            (INK, r#"<ellipse cx="8" cy="8" rx="6" ry="4.800"/>"#),
        ],
        "text" => &[(INK, r#"<path stroke-width="2" d="M3 3.500h10M8 3.500v9.500"/><path d="M5.500 13h5"/>"#)],
        "freeform" => &[
            (INK, r#"<path d="M2.500 12c2-7 4-7 5.500-4s3.500 3 5.500-4"/>"#),
            (BLUE, r#"<circle cx="8" cy="8" r="1.800" fill="black"/>"#),
        ],
        "knife" => &[
            (GRAY, r#"<path fill="black" stroke="none" d="M2 14L10 3l3 2-6 6z"/>"#),
            (INK, r#"<path d="M2 14L10 3l3 2-6 6z"/>"#),
        ],
        "hotspot" => &[
            (SKY, r#"<rect x="2.500" y="3.500" width="11" height="9" fill="black" stroke="none"/>"#),
            (BLUE, r#"<rect x="2.500" y="3.500" width="11" height="9"/>"#),
            (INK, r#"<path fill="black" stroke="none" d="M7 6l5 4.500H9.500l1 2.300-1 .4-1-2.300L7 12.500z"/>"#),
        ],
        "slice" => &[
            (LIME, r#"<rect x="2.500" y="2.500" width="11" height="11" fill="black" stroke="none"/>"#),
            (GREEN, r#"<rect x="2.500" y="2.500" width="11" height="11"/><path d="M2.500 7h11M7 7v6.500"/>"#),
        ],
        "hide" => &[
            (BLUE, r#"<rect x="1.500" y="2.500" width="13" height="11"/>"#),
            (LIME, r#"<rect x="4" y="5" width="8" height="6" fill="black" stroke="none"/>"#),
            (RED, r#"<path stroke-width="1.700" d="M2 14L14 2"/>"#),
        ],
        "show" => &[
            (BLUE, r#"<rect x="1.500" y="2.500" width="13" height="11"/>"#),
            (LIME, r#"<rect x="4" y="5" width="8" height="6" fill="black" stroke="none"/>"#),
            (GREEN, r#"<rect x="4" y="5" width="8" height="6"/>"#),
        ],
        "hand" => &[
            (PEACH, r#"<path fill="black" stroke="none" d="M5 8V3.500a1 1 0 012 0V7m0-4a1 1 0 012 0v4m0-3a1 1 0 012 0v4m0-2a1 1 0 012 0v4c0 3-2 5-4.500 5S5 13 3 9.800c-.6-1 .8-1.900 1.600-1z"/>"#),
            (INK, r#"<path d="M5 8V3.500a1 1 0 012 0V7m0-4a1 1 0 012 0v4m0-3a1 1 0 012 0v4m0-2a1 1 0 012 0v4c0 3-2 5-4.500 5S5 13 3 9.800c-.6-1 .8-1.900 1.600-1z"/>"#),
        ],
        "zoom" => &[
            (SKY, r#"<circle cx="6.500" cy="6.500" r="4.500" fill="black" stroke="none"/>"#),
            (INK, r#"<circle cx="6.500" cy="6.500" r="4.500"/><path stroke-width="2" d="M10 10l4.500 4.500"/>"#),
        ],
        "new" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M3.500 1.500h6l3 3v10h-9z"/>"#),
            (INK, r#"<path d="M3.500 1.500h6l3 3v10h-9zM9.500 1.500v3h3"/>"#),
        ],
        "save" => &[
            (BLUE, r#"<path fill="black" stroke="none" d="M2.500 2.500h9l2 2v9h-11z"/>"#),
            (PAPER, r#"<path fill="black" stroke="none" d="M4.500 8.500h7v5h-7zM5 2.500h5.500V6H5z"/>"#),
            (INK, r#"<path d="M2.500 2.500h9l2 2v9h-11zM5 2.500v3.500h5.500V2.500M4.500 13.500v-5h7v5"/>"#),
        ],
        "open" => &[
            (YELLOW, r#"<path fill="black" stroke="none" d="M1.500 13.500v-10h4.500l1.500 1.500h6v2H4L1.500 13.500z"/>"#),
            (ORANGE, r#"<path fill="black" stroke="none" d="M1.500 13.500l2.500-6.500h11l-2.500 6.500z"/>"#),
            (INK, r#"<path d="M1.500 13.500v-10h4.500l1.500 1.500h6v2M1.500 13.500l2.500-6.500h11l-2.500 6.500z"/>"#),
        ],
        "import" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M6.500 1.500h7v13h-7z"/>"#),
            (INK, r#"<path d="M6.500 4V1.500h7v13h-7V12"/>"#),
            (GREEN, r#"<path stroke-width="1.800" d="M1.500 8h7M6 5l3 3-3 3"/>"#),
        ],
        "export" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M2.500 1.500h7v13h-7z"/>"#),
            (INK, r#"<path d="M9.500 4V1.500h-7v13h7V12"/>"#),
            (GREEN, r#"<path stroke-width="1.800" d="M7 8h7.500M12 5l3 3-3 3"/>"#),
        ],
        "print" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M4.500 1.500h7v5h-7zM4.500 10.500h7v4h-7z"/>"#),
            (GRAY, r#"<path fill="black" stroke="none" d="M2 6.500h12v5h-2.500v-1h-7v1H2z"/>"#),
            (INK, r#"<path d="M4.500 6.500v-5h7v5M4.500 10.500h7v4h-7zM4.500 11.500H2v-5h12v5h-2.500"/>"#),
        ],
        "undo" => &[(INK, r#"<path stroke-width="1.700" d="M5.500 2.500L2.500 5.500l3 3M2.500 5.500h7a3.500 3.500 0 010 7H6"/>"#)],
        "redo" => &[(INK, r#"<path stroke-width="1.700" d="M10.500 2.500l3 3-3 3M13.500 5.500h-7a3.500 3.500 0 000 7H10"/>"#)],
        "cut" => &[
            (GRAY, r#"<path stroke-width="1.400" d="M5.800 10.500L11.500 2M10.200 10.500L4.500 2"/>"#),
            (RED, r#"<circle cx="4.500" cy="12" r="2"/><circle cx="11.500" cy="12" r="2"/>"#),
        ],
        "copy" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M2.500 1.500h8v9h-8zM5.500 5.500h8v9h-8z"/>"#),
            (INK, r#"<rect x="5.500" y="5.500" width="8" height="9"/><path d="M10.500 5.500v-4h-8v9h3"/>"#),
        ],
        "paste" => &[
            (TAN, r#"<path fill="black" stroke="none" d="M2.500 3h11v3H7.500v8.500h-5z"/>"#),
            (PAPER, r#"<rect x="7.500" y="7" width="6.500" height="7.500" fill="black" stroke="none"/>"#),
            (INK, r#"<path d="M5.500 3h-3v11.500h4M10.500 3h3v3M5.500 1.500h5v3h-5z"/><rect x="7.500" y="7" width="6.500" height="7.500"/>"#),
        ],
        "group" => &[
            (SKY, r#"<rect x="3" y="3" width="6" height="6" fill="black" stroke="none"/>"#),
            (ORANGE, r#"<rect x="7" y="7" width="6" height="6" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="1.500" width="13" height="13" stroke-dasharray="2 1.500"/>"#),
        ],
        "ungroup" => &[
            (SKY, r#"<rect x="1.500" y="1.500" width="6" height="6" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="1.500" width="6" height="6"/>"#),
            (ORANGE, r#"<rect x="8.500" y="8.500" width="6" height="6" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="8.500" y="8.500" width="6" height="6"/>"#),
        ],
        "join" => &[
            (SKY, r#"<path fill="black" stroke="none" d="M2 3h7l5 5-5 5H2z"/>"#),
            (INK, r#"<path d="M2 3h7l5 5-5 5H2z"/>"#),
        ],
        "split" => &[
            (SKY, r#"<path fill="black" stroke="none" d="M2 3h4v10H2zM10 3h4v10h-4z"/>"#),
            (INK, r#"<path d="M2 3h4v10H2zM10 3h4v10h-4z"/><path stroke-dasharray="1.500 1.500" d="M8 1v14"/>"#),
        ],
        "arrange-front" => &[
            (GRAY, r#"<rect x="7" y="7" width="7" height="7"/>"#),
            (BLUE, r#"<rect x="2" y="2" width="8" height="8" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="2" y="2" width="8" height="8"/>"#),
        ],
        "arrange-forward" => &[
            (GRAY, r#"<rect x="6" y="6" width="8" height="8" fill="black" stroke="none"/>"#),
            (BLUE, r#"<rect x="2" y="2" width="8" height="8"/>"#),
            (INK, r#"<rect x="6" y="6" width="8" height="8"/>"#),
        ],
        "arrange-backward" => &[
            (BLUE, r#"<rect x="6" y="6" width="8" height="8"/>"#),
            (GRAY, r#"<rect x="2" y="2" width="8" height="8" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="2" y="2" width="8" height="8"/>"#),
        ],
        "arrange-back" => &[
            (BLUE, r#"<rect x="6" y="6" width="8" height="8" fill="black" stroke="none"/>"#),
            (GRAY, r#"<rect x="2" y="2" width="8" height="8"/>"#),
            (INK, r#"<rect x="6" y="6" width="8" height="8"/>"#),
        ],
        "rotate-ccw" => &[
            (STEEL, r#"<rect x="5" y="7" width="8" height="7" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="5" y="7" width="8" height="7"/>"#),
            (GREEN, r#"<path stroke-width="1.500" d="M2 7V4h3M2 4c2.500-2.500 6-2.500 8.500 0"/>"#),
        ],
        "rotate-cw" => &[
            (STEEL, r#"<rect x="3" y="7" width="8" height="7" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="3" y="7" width="8" height="7"/>"#),
            (GREEN, r#"<path stroke-width="1.500" d="M14 7V4h-3M14 4c-2.500-2.500-6-2.500-8.500 0"/>"#),
        ],
        "flip-h" => &[
            (STEEL, r#"<path fill="black" stroke="none" d="M1.500 13V3l5 10z"/>"#),
            (INK, r#"<path d="M1.500 13V3l5 10zM14.500 13V3l-5 10z"/>"#),
            (BLUE, r#"<path stroke-dasharray="1.500 1.500" d="M8 1v14"/>"#),
        ],
        "flip-v" => &[
            (STEEL, r#"<path fill="black" stroke="none" d="M3 1.500h10l-10 5z"/>"#),
            (INK, r#"<path d="M3 1.500h10l-10 5zM3 14.500h10l-10-5z"/>"#),
            (BLUE, r#"<path stroke-dasharray="1.500 1.500" d="M1 8h14"/>"#),
        ],
        "view-original" => &[
            (YELLOW, r#"<path fill="black" stroke="none" d="M2.500 13.500l.8-3.300 7.700-7.700 2.500 2.500-7.700 7.700z"/>"#),
            (ORANGE, r#"<path d="M2.500 13.500l.8-3.300 7.700-7.700 2.500 2.500-7.700 7.700z"/>"#),
        ],
        "view-preview" => &[
            (SKY, r#"<rect x="1.500" y="2.500" width="13" height="11" fill="black" stroke="none"/>"#),
            (GREEN, r#"<path fill="black" stroke="none" d="M1.500 13.500L6 7l3 4 2-2 3.500 4.500z"/>"#),
            (INK, r#"<rect x="1.500" y="2.500" width="13" height="11"/>"#),
        ],
        "view-2up" => &[
            (PAPER, r#"<rect x="1.500" y="2.500" width="13" height="11" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="2.500" width="13" height="11"/><path d="M8 2.500v11"/>"#),
        ],
        "view-4up" => &[
            (PAPER, r#"<rect x="1.500" y="2.500" width="13" height="11" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="2.500" width="13" height="11"/><path d="M8 2.500v11M1.500 8h13"/>"#),
        ],
        "screen-standard" => &[
            (PAPER, r#"<rect x="1.500" y="2.500" width="13" height="11" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="2.500" width="13" height="11"/><path d="M1.500 5h13M5 5v8.500"/>"#),
        ],
        "screen-menus" => &[
            (PAPER, r#"<rect x="1.500" y="2.500" width="13" height="11" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="2.500" width="13" height="11"/><path d="M1.500 5h13"/>"#),
        ],
        "screen-full" => &[
            (STEEL, r#"<rect x="1.500" y="2.500" width="13" height="11" fill="black" stroke="none"/>"#),
            (INK, r#"<rect x="1.500" y="2.500" width="13" height="11"/>"#),
        ],
        "trash" => &[
            (GRAY, r#"<path fill="black" stroke="none" d="M4 5h8l-.8 9.500H4.800z"/>"#),
            (INK, r#"<path d="M4 5h8l-.8 9.500H4.800zM2.500 3.500h11M6.500 3.500V2h3v1.500M6.500 7v5M9.500 7v5"/>"#),
        ],
        "new-layer" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M3.500 1.500h6l3 3v10h-9z"/>"#),
            (INK, r#"<path d="M3.500 1.500h6l3 3v10h-9zM9.500 1.500v3h3"/>"#),
            (GREEN, r#"<path stroke-width="1.600" d="M8 7v5M5.500 9.500h5"/>"#),
        ],
        "folder" => &[
            (PAPER, r#"<path fill="black" stroke="none" d="M1.500 12.500v-9h4.500l1.500 1.500h7v7.500z"/>"#),
            (INK, r#"<path d="M1.500 12.500v-9h4.500l1.500 1.500h7v7.500z"/>"#),
        ],
        _ => return None,
    })
}

/// Single-color icons, tinted when drawn.
fn plain(name: &str) -> Option<&'static str> {
    Some(match name {
        "align-left" => r#"<path d="M2 1.500v13"/><rect x="4" y="3.500" width="9" height="3"/><rect x="4" y="9.500" width="5.500" height="3"/>"#,
        "align-center-x" => r#"<path d="M8 1.500v13"/><rect x="3" y="3.500" width="10" height="3"/><rect x="5" y="9.500" width="6" height="3"/>"#,
        "align-right" => r#"<path d="M14 1.500v13"/><rect x="3" y="3.500" width="9" height="3"/><rect x="6.500" y="9.500" width="5.500" height="3"/>"#,
        "align-top" => r#"<path d="M1.500 2h13"/><rect x="3.500" y="4" width="3" height="9"/><rect x="9.500" y="4" width="3" height="5.500"/>"#,
        "align-center-y" => r#"<path d="M1.500 8h13"/><rect x="3.500" y="3" width="3" height="10"/><rect x="9.500" y="5" width="3" height="6"/>"#,
        "align-bottom" => r#"<path d="M1.500 14h13"/><rect x="3.500" y="3" width="3" height="9"/><rect x="9.500" y="6.500" width="3" height="5.500"/>"#,
        "plus" => r#"<path d="M8 3v10M3 8h10"/>"#,
        "minus" => r#"<path d="M3 8h10"/>"#,
        "eye" => r#"<path d="M1.500 8S4 3.500 8 3.500 14.500 8 14.500 8 12 12.500 8 12.500 1.500 8 1.500 8z"/><circle cx="8" cy="8" r="1.800" fill="black"/>"#,
        "lock" => r#"<rect x="3.500" y="7.500" width="9" height="6.500" fill="black"/><path d="M5.500 7.500V5a2.500 2.500 0 015 0v2.500"/>"#,
        "none" => r#"<path d="M1 15L15 1" stroke-width="2"/>"#,
        "defaults" => r#"<rect x="2" y="2" width="7" height="7" fill="black"/><rect x="7" y="7" width="7" height="7"/>"#,
        "swap" => r#"<path d="M3 6c0-2 1.500-3 3.500-3H12M10 1l2 2-2 2M13 10c0 2-1.500 3-3.500 3H4M6 11l-2 2 2 2"/>"#,
        "chevron" => r#"<path fill="black" d="M6 4l5 4-5 4z"/>"#,
        "chevron-down" => r#"<path fill="black" stroke="none" d="M3 5.500h10L8 11z"/>"#,
        "triangle-open" => r#"<path fill="black" stroke="none" d="M3 5h10l-5 7z"/>"#,
        "triangle-closed" => r#"<path fill="black" stroke="none" d="M5 3l7 5-7 5z"/>"#,
        "check" => r#"<path d="M3 8.500l3.500 3.500L13 4.500" stroke-width="1.800"/>"#,
        "duplicate" => r#"<rect x="2.500" y="2.500" width="8" height="8"/><path d="M13.500 5.500v8h-8"/>"#,
        "play" => r#"<path fill="black" d="M4.500 2.500l9 5.500-9 5.500z"/>"#,
        "stop" => r#"<rect x="4" y="4" width="8" height="8" fill="black"/>"#,
        "first" => r#"<path fill="black" d="M13 3v10L6 8z"/><path d="M3.500 3v10" stroke-width="1.800"/>"#,
        "last" => r#"<path fill="black" d="M3 3v10l7-5z"/><path d="M12.500 3v10" stroke-width="1.800"/>"#,
        "prev" => r#"<path fill="black" d="M11.500 3v10L4.500 8z"/>"#,
        "next" => r#"<path fill="black" d="M4.500 3v10l7-5z"/>"#,
        "onion" => r#"<rect x="2.500" y="2.500" width="7.500" height="7.500" stroke-dasharray="1.600 1.400"/><rect x="6" y="6" width="7.500" height="7.500"/>"#,
        "share" => r#"<path d="M6.500 9.500l3-3M7 5l1.500-1.500a2.500 2.500 0 013.500 3.500L10.500 8.500M9 11l-1.500 1.500a2.500 2.500 0 01-3.500-3.500L5.500 7.500"/>"#,
        "panel-menu" => r#"<path fill="black" stroke="none" d="M2 5h4L4 8zM8 4h6v1.200H8zM8 7.400h6v1.200H8zM8 10.800h6V12H8z"/>"#,
        "close" => r#"<path d="M4 4l8 8M12 4l-8 8" stroke-width="1.400"/>"#,
        "win-min" => r#"<path d="M3 12h8" stroke-width="2.400" stroke-linecap="butt"/>"#,
        "win-max" => r#"<rect x="3" y="3.500" width="10" height="9" stroke-width="1.300"/><path d="M3 5h10" stroke-width="2"/>"#,
        "win-restore" => r#"<rect x="2.500" y="6" width="8" height="7" stroke-width="1.300"/><path d="M2.500 7h8" stroke-width="1.800"/><path d="M5.500 6V3h8v7h-3" stroke-width="1.300"/>"#,
        "win-close" => r#"<path d="M3.500 3.500l9 9M12.500 3.500l-9 9" stroke-width="2.200"/>"#,
        _ => return colored_outline(name),
    })
}

/// Lets a colored icon also be served as one shape, using its last layer.
fn colored_outline(name: &str) -> Option<&'static str> {
    colored(name).and_then(|layers| layers.last()).map(|(_, body)| *body)
}

/// Path of one layer of a colored icon.
pub fn layer_path(name: &str, layer: usize) -> SharedString {
    format!("icons/{name}#{layer}.svg").into()
}

fn lookup(asset: &str) -> Option<&'static str> {
    match asset.split_once('#') {
        Some((name, layer)) => colored(name)?.get(layer.parse::<usize>().ok()?).map(|(_, body)| *body),
        None => plain(asset),
    }
}

/// Serves `icons/<name>.svg` from the tables above.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let name = path.strip_prefix("icons/").and_then(|p| p.strip_suffix(".svg"));
        Ok(name.and_then(lookup).map(|content| {
            Cow::Owned(
                format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" stroke="black" stroke-width="1.100" stroke-linecap="round" stroke-linejoin="round">{content}</svg>"#
                )
                .into_bytes(),
            )
        }))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_are_served_by_index() {
        let layers = colored("pencil").unwrap();
        for index in 0..layers.len() {
            assert!(Assets.load(&layer_path("pencil", index)).unwrap().is_some());
        }
        assert!(Assets.load(&layer_path("pencil", layers.len())).unwrap().is_none());
    }

    #[test]
    fn colored_icons_also_load_as_plain_shapes() {
        assert!(Assets.load("icons/save.svg").unwrap().is_some());
        assert!(Assets.load("icons/no-such-icon.svg").unwrap().is_none());
    }
}
