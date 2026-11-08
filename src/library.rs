//! The built-in Common Library. Every item is drawn from code for this application.

use crate::autoshape::AutoKind;
use crate::model::{v2, Color, Object, Rect, Shape, Stroke};
use crate::export::measure_text;

const INK: Color = Color::rgb(0x33, 0x33, 0x33);
const MUTED: Color = Color::rgb(0x88, 0x88, 0x88);
const LINE: Color = Color::rgb(0xbb, 0xbb, 0xbb);
const PAPER: Color = Color::WHITE;
const SHADE: Color = Color::rgb(0xee, 0xee, 0xee);
const BLUE: Color = Color::rgb(0x1e, 0x78, 0xd2);

pub struct CommonItem {
    pub name: &'static str,
    pub category: &'static str,
    build: fn() -> Vec<Object>,
}

impl CommonItem {
    pub fn objects(&self) -> Vec<Object> {
        (self.build)()
    }
}

fn base(shape: Shape, name: &str, bounds: Rect, fill: Option<Color>, stroke: Option<Color>) -> Object {
    Object {
        id: 0,
        name: name.into(),
        shape,
        bounds,
        points: Vec::new(),
        fill,
        stroke: stroke.map(|color| Stroke { color, width: 1.0 }),
        opacity: 1.0,
        visible: true,
        locked: false,
    }
}

fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

fn rect(bounds: Rect, radius: f32, fill: Option<Color>, stroke: Option<Color>) -> Object {
    base(Shape::Rect { radius }, "Rectangle", bounds, fill, stroke)
}

fn ellipse(bounds: Rect, fill: Option<Color>, stroke: Option<Color>) -> Object {
    base(Shape::Ellipse, "Ellipse", bounds, fill, stroke)
}

fn auto(kind: AutoKind, bounds: Rect, fill: Option<Color>, stroke: Option<Color>) -> Object {
    let d = kind.defaults();
    base(Shape::Auto { kind, a: d.a, b: d.b, count: d.count }, kind.label(), bounds, fill, stroke)
}

fn line(a: (f32, f32), b: (f32, f32), color: Color, width: f32) -> Object {
    let points = vec![v2(a.0, a.1), v2(b.0, b.1)];
    let mut object = base(Shape::Line, "Line", Rect::of_points(&points), None, Some(color));
    object.points = points;
    object.stroke = Some(Stroke { color, width });
    object
}

fn poly(points: &[(f32, f32)], fill: Option<Color>, stroke: Option<Color>) -> Object {
    let points: Vec<_> = points.iter().map(|p| v2(p.0, p.1)).collect();
    let mut object = base(Shape::Path { closed: true }, "Path", Rect::of_points(&points), fill, stroke);
    object.points = points;
    object
}

/// Text placed by its top-left corner.
fn text(x: f32, y: f32, content: &str, size: f32, color: Color) -> Object {
    let (w, h) = measure_text(content, size);
    base(Shape::Text { content: content.into(), size }, "Text", r(x, y, w, h), Some(color), None)
}

/// Text centered inside `area`.
fn label(area: Rect, content: &str, size: f32, color: Color) -> Object {
    let (w, h) = measure_text(content, size);
    text(area.x + (area.w - w) / 2.0, area.y + (area.h - h) / 2.0, content, size, color)
}

fn button(fill: Option<Color>, stroke: Option<Color>, ink: Color, radius: f32) -> Vec<Object> {
    let area = r(0.0, 0.0, 120.0, 36.0);
    vec![rect(area, radius, fill, stroke), label(area, "Button", 14.0, ink)]
}

const ITEMS: &[CommonItem] = &[
    CommonItem { name: "Primary Button", category: "Buttons", build: || button(Some(BLUE), None, PAPER, 4.0) },
    CommonItem { name: "Secondary Button", category: "Buttons", build: || button(Some(PAPER), Some(LINE), INK, 4.0) },
    CommonItem { name: "Pill Button", category: "Buttons", build: || button(Some(INK), None, PAPER, 18.0) },
    CommonItem {
        name: "Text Input",
        category: "Forms",
        build: || vec![rect(r(0.0, 0.0, 200.0, 32.0), 2.0, Some(PAPER), Some(LINE)), text(10.0, 8.0, "Placeholder", 13.0, MUTED)],
    },
    CommonItem {
        name: "Dropdown",
        category: "Forms",
        build: || {
            vec![
                rect(r(0.0, 0.0, 200.0, 32.0), 2.0, Some(PAPER), Some(LINE)),
                text(10.0, 8.0, "Choose one", 13.0, INK),
                poly(&[(176.0, 13.0), (188.0, 13.0), (182.0, 20.0)], Some(INK), None),
            ]
        },
    },
    CommonItem {
        name: "Checkbox",
        category: "Forms",
        build: || {
            let mut mark = poly(&[(4.0, 9.0), (8.0, 13.0), (14.0, 5.0)], None, Some(PAPER));
            mark.shape = Shape::Path { closed: false };
            mark.stroke = Some(Stroke { color: PAPER, width: 2.0 });
            vec![rect(r(0.0, 0.0, 18.0, 18.0), 2.0, Some(BLUE), None), mark]
        },
    },
    CommonItem {
        name: "Radio Button",
        category: "Forms",
        build: || vec![ellipse(r(0.0, 0.0, 18.0, 18.0), Some(PAPER), Some(LINE)), ellipse(r(5.0, 5.0, 8.0, 8.0), Some(BLUE), None)],
    },
    CommonItem {
        name: "Browser Window",
        category: "Layout",
        build: || {
            vec![
                rect(r(0.0, 0.0, 320.0, 220.0), 4.0, Some(PAPER), Some(LINE)),
                rect(r(0.0, 0.0, 320.0, 28.0), 4.0, Some(SHADE), Some(LINE)),
                ellipse(r(10.0, 9.0, 10.0, 10.0), Some(LINE), None),
                ellipse(r(26.0, 9.0, 10.0, 10.0), Some(LINE), None),
                ellipse(r(42.0, 9.0, 10.0, 10.0), Some(LINE), None),
                rect(r(64.0, 6.0, 240.0, 16.0), 8.0, Some(PAPER), Some(LINE)),
            ]
        },
    },
    CommonItem {
        name: "Phone Frame",
        category: "Layout",
        build: || {
            vec![
                rect(r(0.0, 0.0, 120.0, 240.0), 14.0, Some(INK), None),
                rect(r(6.0, 18.0, 108.0, 200.0), 2.0, Some(PAPER), None),
                ellipse(r(53.0, 222.0, 14.0, 14.0), None, Some(MUTED)),
            ]
        },
    },
    CommonItem {
        name: "Image Placeholder",
        category: "Layout",
        build: || {
            vec![
                rect(r(0.0, 0.0, 160.0, 120.0), 0.0, Some(SHADE), Some(LINE)),
                line((0.0, 0.0), (160.0, 120.0), LINE, 1.0),
                line((160.0, 0.0), (0.0, 120.0), LINE, 1.0),
            ]
        },
    },
    CommonItem {
        name: "Card",
        category: "Layout",
        build: || {
            vec![
                rect(r(0.0, 0.0, 180.0, 200.0), 6.0, Some(PAPER), Some(LINE)),
                rect(r(10.0, 10.0, 160.0, 90.0), 2.0, Some(SHADE), None),
                text(10.0, 110.0, "Title", 16.0, INK),
                rect(r(10.0, 142.0, 150.0, 6.0), 0.0, Some(SHADE), None),
                rect(r(10.0, 156.0, 120.0, 6.0), 0.0, Some(SHADE), None),
            ]
        },
    },
    CommonItem {
        name: "Callout",
        category: "Layout",
        build: || vec![auto(AutoKind::Bubble, r(0.0, 0.0, 160.0, 90.0), Some(PAPER), Some(LINE)), label(r(0.0, 0.0, 160.0, 70.0), "Note", 14.0, INK)],
    },
    CommonItem {
        name: "Home",
        category: "Icons",
        build: || {
            vec![
                poly(&[(16.0, 2.0), (31.0, 16.0), (1.0, 16.0)], Some(INK), None),
                rect(r(6.0, 16.0, 20.0, 14.0), 0.0, Some(INK), None),
                rect(r(13.0, 21.0, 6.0, 9.0), 0.0, Some(PAPER), None),
            ]
        },
    },
    CommonItem {
        name: "Search",
        category: "Icons",
        build: || {
            let mut lens = ellipse(r(2.0, 2.0, 20.0, 20.0), None, Some(INK));
            lens.stroke = Some(Stroke { color: INK, width: 3.0 });
            vec![lens, line((19.0, 19.0), (30.0, 30.0), INK, 3.0)]
        },
    },
    CommonItem {
        name: "Mail",
        category: "Icons",
        build: || {
            let mut flap = poly(&[(1.0, 6.0), (16.0, 18.0), (31.0, 6.0)], None, Some(INK));
            flap.shape = Shape::Path { closed: false };
            vec![rect(r(1.0, 6.0, 30.0, 20.0), 1.0, Some(PAPER), Some(INK)), flap]
        },
    },
    CommonItem {
        name: "Menu",
        category: "Icons",
        build: || (0..3).map(|i| rect(r(3.0, 7.0 + i as f32 * 8.0, 26.0, 3.0), 0.0, Some(INK), None)).collect(),
    },
    CommonItem {
        name: "Close",
        category: "Icons",
        build: || vec![line((5.0, 5.0), (27.0, 27.0), INK, 3.0), line((27.0, 5.0), (5.0, 27.0), INK, 3.0)],
    },
    CommonItem {
        name: "User",
        category: "Icons",
        build: || vec![ellipse(r(10.0, 2.0, 12.0, 12.0), Some(INK), None), auto(AutoKind::Pie, r(3.0, 15.0, 26.0, 30.0), Some(INK), None)],
    },
    CommonItem { name: "Favorite", category: "Icons", build: || vec![auto(AutoKind::Star, r(0.0, 0.0, 32.0, 32.0), Some(INK), None)] },
    CommonItem {
        name: "Start",
        category: "Flowchart",
        build: || vec![rect(r(0.0, 0.0, 120.0, 44.0), 22.0, Some(PAPER), Some(INK)), label(r(0.0, 0.0, 120.0, 44.0), "Start", 14.0, INK)],
    },
    CommonItem {
        name: "Process",
        category: "Flowchart",
        build: || vec![rect(r(0.0, 0.0, 120.0, 60.0), 0.0, Some(PAPER), Some(INK)), label(r(0.0, 0.0, 120.0, 60.0), "Process", 14.0, INK)],
    },
    CommonItem {
        name: "Decision",
        category: "Flowchart",
        build: || {
            vec![
                poly(&[(70.0, 0.0), (140.0, 45.0), (70.0, 90.0), (0.0, 45.0)], Some(PAPER), Some(INK)),
                label(r(0.0, 0.0, 140.0, 90.0), "Decide", 14.0, INK),
            ]
        },
    },
    CommonItem { name: "Connector", category: "Flowchart", build: || vec![auto(AutoKind::Arrow, r(0.0, 0.0, 80.0, 24.0), Some(INK), None)] },
];

pub fn common_items() -> &'static [CommonItem] {
    ITEMS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_has_drawable_objects_and_a_unique_name() {
        let mut names = std::collections::HashSet::new();
        for item in common_items() {
            assert!(names.insert(item.name), "{}", item.name);
            let objects = item.objects();
            assert!(!objects.is_empty(), "{}", item.name);
            assert!(objects.iter().all(|o| !o.outline().is_empty()), "{}", item.name);
        }
    }
}
