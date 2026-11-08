//! Path panel operations: boolean combines and path alterations.

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::mesh::float::stroke::offset::StrokeOffset;
use i_overlay::mesh::float::style::{LineCap, LineJoin, StrokeStyle};

use crate::model::{bridge, contours, v2, Editor, Object, Rect, Shape, Vec2};

type Contour = Vec<[f64; 2]>;
type Contours = Vec<Contour>;
type Shapes = Vec<Contours>;

/// Curve smoothness of round joins and caps, in radians per segment.
const ROUND_STEP: f64 = 0.2;
/// Outlines below this many vertices stop being simplified.
const MIN_CLOSED_POINTS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathOp {
    Union,
    Intersect,
    Punch,
    Crop,
    Join,
    Split,
    Simplify,
    ExpandStroke,
    Inset,
    Outset,
}

impl PathOp {
    pub fn label(self) -> &'static str {
        match self {
            PathOp::Union => "Union",
            PathOp::Intersect => "Intersect",
            PathOp::Punch => "Punch",
            PathOp::Crop => "Crop",
            PathOp::Join => "Join",
            PathOp::Split => "Split",
            PathOp::Simplify => "Simplify",
            PathOp::ExpandStroke => "Expand Stroke",
            PathOp::Inset => "Inset",
            PathOp::Outset => "Outset",
        }
    }

    /// Combines need two or more objects; the rest work on any selection.
    pub fn needs_two(self) -> bool {
        matches!(self, PathOp::Union | PathOp::Intersect | PathOp::Punch | PathOp::Crop | PathOp::Join)
    }
}

fn to_float(points: &[Vec2]) -> Contour {
    points.iter().map(|p| [p.x as f64, p.y as f64]).collect()
}

fn to_points(contour: &[[f64; 2]]) -> Vec<Vec2> {
    contour.iter().map(|p| v2(p[0] as f32, p[1] as f32)).collect()
}

fn contours_of(object: &Object) -> Contours {
    contours(&object.outline()).iter().map(|c| to_float(c)).collect()
}

fn overlay(subject: &Contours, clip: &Contours, rule: OverlayRule, fill: FillRule) -> Shapes {
    subject.overlay(clip, rule, fill)
}

fn flatten(shapes: Shapes) -> Contours {
    shapes.into_iter().flatten().collect()
}

/// Traces each contour as a stroke and merges the pieces into clean shapes.
fn stroked(contours: &Contours, width: f64, closed: bool, join: LineJoin<f64>) -> Shapes {
    let style = StrokeStyle::new(width)
        .line_join(join)
        .start_cap(LineCap::Round(ROUND_STEP))
        .end_cap(LineCap::Round(ROUND_STEP));
    let mut pieces = Contours::new();
    for contour in contours {
        pieces.extend(flatten(contour.stroke(style.clone(), closed)));
    }
    overlay(&pieces, &Contours::new(), OverlayRule::Union, FillRule::NonZero)
}

/// Turns one resulting shape into a filled path that looks like `template`.
fn result_object(id: u64, template: &Object, points: Vec<Vec2>, closed: bool) -> Object {
    let mut object = template.clone();
    object.id = id;
    object.name = "Path".into();
    object.shape = Shape::Path { closed };
    object.bounds = Rect::of_points(&points);
    object.points = points;
    object
}

fn shape_points(shape: &Contours) -> Vec<Vec2> {
    let mut rings = shape.iter().map(|c| to_points(c));
    let outer = rings.next().unwrap_or_default();
    bridge(outer, rings.collect())
}

/// Ramer-Douglas-Peucker on an open polyline.
fn reduce(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    let mut split = (0, 0.0_f32);
    for (i, p) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let d = crate::model::dist_to_segment(*p, first, last);
        if d > split.1 {
            split = (i, d);
        }
    }
    if split.1 <= tolerance {
        return vec![first, last];
    }
    let mut left = reduce(&points[..=split.0], tolerance);
    left.pop();
    left.extend(reduce(&points[split.0..], tolerance));
    left
}

fn simplify_ring(ring: &[Vec2], closed: bool, tolerance: f32) -> Vec<Vec2> {
    if !closed {
        return reduce(ring, tolerance);
    }
    if ring.len() <= MIN_CLOSED_POINTS {
        return ring.to_vec();
    }
    // Splitting at the far vertex gives the two halves a stable anchor.
    let start = ring[0];
    let far = (0..ring.len())
        .max_by(|a, b| {
            let d = |i: usize| (ring[i].x - start.x).powi(2) + (ring[i].y - start.y).powi(2);
            d(*a).total_cmp(&d(*b))
        })
        .unwrap_or(0);
    let mut first = reduce(&ring[..=far], tolerance);
    let mut second: Vec<Vec2> = ring[far..].to_vec();
    second.push(start);
    let second = reduce(&second, tolerance);
    first.pop();
    first.extend_from_slice(&second[..second.len() - 1]);
    if first.len() < MIN_CLOSED_POINTS { ring.to_vec() } else { first }
}

impl Editor {
    /// Runs a path operation on the selected vector objects and returns how many paths resulted.
    /// `amount` is the Simplify tolerance, or the Inset and Outset distance.
    pub fn path_op(&mut self, op: PathOp, amount: f32) -> Result<usize, &'static str> {
        let selected: Vec<Object> = self.doc.page().objects().filter(|o| self.is_selected(o.id)).cloned().collect();
        if selected.is_empty() {
            return Err("Select one or more vector shapes first");
        }
        if selected.iter().any(|o| !o.reshapable()) {
            return Err("Path operations work on vector shapes, not text, bitmaps, symbols or web areas");
        }
        if op.needs_two() && selected.len() < 2 {
            return Err("Select at least two shapes to combine");
        }
        let results = self.path_results(op, &selected, amount as f64)?;
        if results.is_empty() {
            return Err(match op {
                PathOp::Intersect | PathOp::Crop => "The shapes do not overlap",
                _ => "That would leave nothing behind",
            });
        }
        let objects: Vec<Object> = results
            .into_iter()
            .map(|(template, points, closed)| {
                let id = self.alloc_id();
                result_object(id, &selected[template], points, closed)
            })
            .collect();
        let count = objects.len();
        self.replace_selected(objects);
        Ok(count)
    }

    /// Computes the new paths as (index of the object to copy attributes from, vertices, closed).
    fn path_results(&self, op: PathOp, selected: &[Object], amount: f64) -> Result<Vec<(usize, Vec<Vec2>, bool)>, &'static str> {
        let filled = |shapes: Shapes, template: usize| -> Vec<(usize, Vec<Vec2>, bool)> {
            shapes.iter().map(|shape| (template, shape_points(shape), true)).collect()
        };
        let top = selected.len() - 1;
        Ok(match op {
            PathOp::Union | PathOp::Intersect | PathOp::Join => {
                let rule = match op {
                    PathOp::Union => OverlayRule::Union,
                    PathOp::Intersect => OverlayRule::Intersect,
                    _ => OverlayRule::Xor,
                };
                let mut merged = contours_of(&selected[0]);
                let mut shapes = Shapes::new();
                for object in &selected[1..] {
                    shapes = overlay(&merged, &contours_of(object), rule, FillRule::EvenOdd);
                    merged = flatten(shapes.clone());
                }
                if op == PathOp::Join {
                    let joined = flatten(shapes);
                    if joined.is_empty() { Vec::new() } else { vec![(0, shape_points(&joined), true)] }
                } else {
                    filled(shapes, 0)
                }
            }
            PathOp::Punch | PathOp::Crop => {
                let cutter = contours_of(&selected[top]);
                let rule = if op == PathOp::Punch { OverlayRule::Difference } else { OverlayRule::Intersect };
                (0..top)
                    .flat_map(|i| filled(overlay(&contours_of(&selected[i]), &cutter, rule, FillRule::EvenOdd), i))
                    .collect()
            }
            PathOp::Split => {
                let parts: Vec<_> = selected
                    .iter()
                    .enumerate()
                    .flat_map(|(i, object)| {
                        let closed = object.is_closed();
                        contours(&object.outline()).into_iter().map(move |c| (i, c, closed))
                    })
                    .collect();
                if parts.len() == selected.len() {
                    return Err("Only joined paths and shapes with holes can be split");
                }
                parts
            }
            PathOp::Simplify => selected
                .iter()
                .enumerate()
                .map(|(i, object)| {
                    let closed = object.is_closed();
                    let mut rings = contours(&object.outline()).into_iter().map(|c| simplify_ring(&c, closed, amount as f32));
                    let outer = rings.next().unwrap_or_default();
                    (i, bridge(outer, rings.collect()), closed)
                })
                .collect(),
            PathOp::ExpandStroke => {
                let mut results = Vec::new();
                for (i, object) in selected.iter().enumerate() {
                    let Some(stroke) = object.stroke.filter(|s| s.width > 0.0) else {
                        return Err("Expand Stroke needs objects that have a stroke");
                    };
                    let shapes = stroked(&contours_of(object), stroke.width as f64, object.is_closed(), LineJoin::Round(ROUND_STEP));
                    results.extend(filled(shapes, i));
                }
                results
            }
            PathOp::Inset | PathOp::Outset => {
                let mut results = Vec::new();
                for (i, object) in selected.iter().enumerate() {
                    if !object.is_closed() {
                        return Err("Inset and Outset work on closed shapes");
                    }
                    let shape = contours_of(object);
                    let ring = flatten(stroked(&shape, amount * 2.0, true, LineJoin::Miter(ROUND_STEP)));
                    let rule = if op == PathOp::Inset { OverlayRule::Difference } else { OverlayRule::Union };
                    results.extend(filled(overlay(&shape, &ring, rule, FillRule::EvenOdd), i));
                }
                results
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autoshape::{AutoKind};
    use crate::model::{Color, Document, Stroke};

    fn rect(id: u64, x: f32, y: f32, w: f32, h: f32) -> Object {
        Object {
            id,
            name: format!("r{id}"),
            shape: Shape::Rect { radius: 0.0 },
            bounds: Rect { x, y, w, h },
            points: Vec::new(),
            fill: Some(Color::rgb(10, 20, 30)),
            stroke: Some(Stroke { color: Color::BLACK, width: 4.0 }),
            opacity: 1.0,
            visible: true,
            locked: false,
        }
    }

    fn editor(objects: Vec<Object>) -> Editor {
        let mut doc = Document::new(400, 400);
        doc.next_id = 100;
        doc.pages[0].layers[0].objects = objects;
        let mut ed = Editor::new(doc);
        ed.select_all();
        ed
    }

    fn only(ed: &Editor) -> &Object {
        let objects: Vec<_> = ed.doc.page().objects().collect();
        assert_eq!(objects.len(), 1);
        objects[0]
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.5
    }

    #[test]
    fn union_covers_both_shapes() {
        let mut ed = editor(vec![rect(1, 0.0, 0.0, 100.0, 100.0), rect(2, 50.0, 50.0, 100.0, 100.0)]);
        assert_eq!(ed.path_op(PathOp::Union, 0.0), Ok(1));
        let b = only(&ed).bounds;
        assert!(near(b.x, 0.0) && near(b.right(), 150.0) && near(b.bottom(), 150.0), "{b:?}");
        assert_eq!(ed.selection.len(), 1);
    }

    #[test]
    fn intersect_keeps_the_overlap() {
        let mut ed = editor(vec![rect(1, 0.0, 0.0, 100.0, 100.0), rect(2, 50.0, 50.0, 100.0, 100.0)]);
        ed.path_op(PathOp::Intersect, 0.0).unwrap();
        let b = only(&ed).bounds;
        assert!(near(b.x, 50.0) && near(b.w, 50.0) && near(b.h, 50.0), "{b:?}");
    }

    #[test]
    fn disjoint_intersect_reports_and_leaves_the_shapes() {
        let mut ed = editor(vec![rect(1, 0.0, 0.0, 10.0, 10.0), rect(2, 50.0, 50.0, 10.0, 10.0)]);
        assert_eq!(ed.path_op(PathOp::Intersect, 0.0), Err("The shapes do not overlap"));
        assert_eq!(ed.doc.page().objects().count(), 2);
    }

    #[test]
    fn punch_through_the_middle_makes_a_splittable_hole() {
        let mut ed = editor(vec![rect(1, 0.0, 0.0, 100.0, 100.0), rect(2, 25.0, 25.0, 50.0, 50.0)]);
        ed.path_op(PathOp::Punch, 0.0).unwrap();
        let object = only(&ed);
        assert_eq!(object.fill, Some(Color::rgb(10, 20, 30)));
        assert!(!object.hit(v2(50.0, 50.0), 0.0), "the hole is empty");
        assert!(object.hit(v2(10.0, 50.0), 0.0));
        assert_eq!(ed.path_op(PathOp::Split, 0.0), Ok(2));
    }

    #[test]
    fn crop_trims_lower_objects_to_the_top_one() {
        let mut ed = editor(vec![rect(1, 0.0, 0.0, 100.0, 100.0), rect(2, 60.0, 0.0, 100.0, 40.0)]);
        ed.path_op(PathOp::Crop, 0.0).unwrap();
        let b = only(&ed).bounds;
        assert!(near(b.x, 60.0) && near(b.right(), 100.0) && near(b.h, 40.0), "{b:?}");
    }

    #[test]
    fn join_makes_one_compound_path() {
        let mut ed = editor(vec![rect(1, 0.0, 0.0, 10.0, 10.0), rect(2, 50.0, 50.0, 10.0, 10.0)]);
        assert_eq!(ed.path_op(PathOp::Join, 0.0), Ok(1));
        assert_eq!(ed.path_op(PathOp::Split, 0.0), Ok(2));
    }

    #[test]
    fn expand_stroke_outlines_a_line() {
        let mut line = rect(1, 0.0, 50.0, 100.0, 0.0);
        line.shape = Shape::Line;
        line.points = vec![v2(0.0, 50.0), v2(100.0, 50.0)];
        let mut ed = editor(vec![line]);
        ed.path_op(PathOp::ExpandStroke, 0.0).unwrap();
        let b = only(&ed).bounds;
        assert!(near(b.h, 4.0), "{b:?}");
    }

    #[test]
    fn inset_and_outset_move_the_edges() {
        let mut ed = editor(vec![rect(1, 10.0, 10.0, 100.0, 100.0)]);
        ed.path_op(PathOp::Outset, 5.0).unwrap();
        let b = only(&ed).bounds;
        assert!(near(b.x, 5.0) && near(b.w, 110.0), "{b:?}");
        ed.path_op(PathOp::Inset, 15.0).unwrap();
        let b = only(&ed).bounds;
        assert!(near(b.x, 20.0) && near(b.w, 80.0), "{b:?}");
    }

    #[test]
    fn simplify_removes_redundant_vertices() {
        let mut path = rect(1, 0.0, 0.0, 100.0, 0.0);
        path.shape = Shape::Path { closed: false };
        path.points = (0..=100).map(|x| v2(x as f32, 0.0)).collect();
        let mut ed = editor(vec![path]);
        ed.path_op(PathOp::Simplify, 1.0).unwrap();
        assert_eq!(only(&ed).points.len(), 2);
    }

    #[test]
    fn text_is_refused() {
        let mut text = rect(1, 0.0, 0.0, 10.0, 10.0);
        text.shape = Shape::Text { content: "a".into(), size: 12.0 };
        let mut ed = editor(vec![text]);
        assert!(ed.path_op(PathOp::Simplify, 1.0).is_err());
    }

    #[test]
    fn auto_shapes_can_be_combined() {
        let mut star = rect(1, 0.0, 0.0, 100.0, 100.0);
        let d = AutoKind::Star.defaults();
        star.shape = Shape::Auto { kind: AutoKind::Star, a: d.a, b: d.b, count: d.count };
        let mut ed = editor(vec![star, rect(2, 40.0, 40.0, 100.0, 100.0)]);
        assert_eq!(ed.path_op(PathOp::Union, 0.0), Ok(1));
    }
}
