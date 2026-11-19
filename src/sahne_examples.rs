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

//! The examples that come with PisiAnimasyon: short animations to open, play and take apart.
//!
//! Like PisiWorks' own, they are built here from the commands a user has, so their
//! texts follow the language of the interface and there are no files to install.

use crate::autoshape::AutoKind;
use crate::examples::{add, bold, boxed, centered, finish, gradient, layer, rect, Example, BLUE, INK, ORANGE, PALE, TEAL, WHITE, YELLOW};
use crate::behavior::{Behavior, BehaviorKind};
use crate::i18n::t;
use crate::rules::{Event, EventKind, Rule, Step, StepKind};
use crate::model::{Action, Color, Document, Editor, FillKind, LayerKind, Object, Shape, Span, Stroke, SymbolKind, TextStyle, Tween, Vec2, WebKind};

pub const EXAMPLES: &[Example] = &[
    Example { name: "Pisi: Iron Claw", about: "The showcase: a robot leopard, three stages and a giant rat, made with nothing but behaviors and rules", build: crate::sahne_showcase::iron_claw },
    Example { name: "Bouncing ball", about: "Motion tweens with easing: the ball speeds up as it falls and slows as it rises", build: bouncing_ball },
    Example { name: "Changing shapes", about: "Shape tweens: a square becomes a circle, and the circle a star", build: changing_shapes },
    Example { name: "Along a path", about: "A guide layer: the arrow follows the path and turns to face the way it goes", build: along_a_path },
    Example { name: "Spotlight", about: "A mask layer: a moving circle lets the bright words show", build: spotlight },
    Example { name: "Turning gears", about: "A movie clip with a timeline of its own, placed twice on the stage", build: turning_gears },
    Example { name: "One after another", about: "Layers with times of their own: the car drives for four seconds and stops, then the sun rises, then the words appear", build: one_after_another },
    Example { name: "Stick fight", about: "Pose to pose: figures of lines that turn at their joints, a punch, a kick, and enemies that come and go at their own times", build: stick_fight },
    Example { name: "Behaviors", about: "Motion with no keyframes: every ready behavior on a single frame. Test the movie to walk, jump and drag", build: behaviors },
    Example { name: "Game: catch the apples", about: "Rules and numbers: the arrow keys move the basket, each apple caught is a point, each one dropped costs a life", build: crate::sahne_games::catch_the_apples },
    Example { name: "Game: hit the targets", about: "Rules about clicks and a clock: click the moving targets before the time runs out", build: crate::sahne_games::hit_the_targets },
    Example { name: "Game: platform", about: "Solid ground and platforms, coins that are removed, and a door that opens only when every coin is collected", build: crate::sahne_games::platform_game },
    Example { name: "Game: space shooter", about: "Copies made by a rule: Space fires, each hit makes the aliens faster", build: crate::sahne_games::space_shooter },
    Example { name: "Game: quiz", about: "Rules across scenes: a question on each scene, a point for each right answer, the score at the end", build: crate::sahne_games::quiz },
    Example { name: "Driving a car", about: "A car that gathers speed and slows down by itself, with wheels tied to it that turn as far as it goes", build: crate::sahne_games::driving },
    Example { name: "Two scenes with buttons", about: "Scenes, frame actions that stop the film, and buttons that lead from scene to scene", build: two_scenes },
];

pub(crate) fn start(width: u32, height: u32, first_layer: &str, fps: f32) -> Editor {
    let mut e = Editor::new(Document::new(width, height));
    e.doc.page_mut().layers[0].name = first_layer.to_string();
    e.doc.page_mut().start_timeline(fps);
    e
}

/// Makes every layer of the scene last as long as the longest one, so that what stands still shows throughout.
fn fill_out(e: &mut Editor) {
    e.doc.page_mut().edit_timeline(|timeline| {
        let frames = timeline.frame_count();
        for layer in 0..timeline.layers.len() {
            let own = timeline.layers[layer].last().map_or(0, |span| span.start + span.length);
            if own < frames {
                timeline.insert_frames(layer, own.saturating_sub(1), frames - own);
            }
        }
    });
    e.goto_frame(0);
}

/// A keyframe on a frame of the active layer, with the objects it starts with changed by `change`.
fn keyframe(e: &mut Editor, frame: usize, ids: &[u64], change: impl Fn(&mut crate::model::Object)) {
    e.goto_frame(frame);
    e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, false));
    e.selection = ids.to_vec();
    e.for_selected(change);
    e.selection.clear();
}

fn tween(e: &mut Editor, frame: usize, tween: Tween) {
    e.goto_frame(frame);
    e.on_timeline(|timeline, layer, frame| timeline.set_tween(layer, frame, Some(tween)));
}

fn bouncing_ball() -> Document {
    let mut e = start(480, 320, t("Background"), 24.0);
    let sky = boxed(&mut e, t("Sky"), rect(0.0, 0.0, 480.0, 320.0), 0.0, PALE);
    gradient(&mut e, sky, FillKind::Linear, WHITE, Color::rgb(0xc8, 0xdc, 0xf4), (0.5, 0.0), (0.5, 1.0));
    boxed(&mut e, t("Ground"), rect(0.0, 270.0, 480.0, 50.0), 0.0, TEAL);
    // The shadow grows as the ball comes down and shrinks as it goes up.
    layer(&mut e, t("Shadow"));
    let shadow = add(&mut e, t("Shadow"), Shape::Ellipse, rect(215.0, 276.0, 50.0, 10.0), Some(INK), None);
    e.for_selected(|o| o.opacity = 0.15);
    keyframe(&mut e, 12, &[shadow], |o| {
        o.set_bounds(rect(195.0, 272.0, 90.0, 18.0));
        o.opacity = 0.45;
    });
    keyframe(&mut e, 24, &[shadow], |o| {
        o.set_bounds(rect(215.0, 276.0, 50.0, 10.0));
        o.opacity = 0.15;
    });
    tween(&mut e, 0, Tween { ease: -100.0, ..Tween::default() });
    tween(&mut e, 12, Tween { ease: 100.0, ..Tween::default() });
    e.goto_frame(0);
    layer(&mut e, t("Ball"));
    let ball = add(&mut e, t("Ball"), Shape::Ellipse, rect(210.0, 30.0, 60.0, 60.0), Some(ORANGE), None);
    gradient(&mut e, ball, FillKind::Radial, YELLOW, ORANGE, (0.35, 0.35), (1.0, 0.5));
    // Down to the ground, squashed a little where it lands, and up again.
    keyframe(&mut e, 12, &[ball], |o| o.set_bounds(rect(204.0, 222.0, 72.0, 50.0)));
    keyframe(&mut e, 24, &[ball], |o| o.set_bounds(rect(210.0, 30.0, 60.0, 60.0)));
    tween(&mut e, 0, Tween { ease: -100.0, ..Tween::default() });
    tween(&mut e, 12, Tween { ease: 100.0, ..Tween::default() });
    fill_out(&mut e);
    finish(e)
}

fn changing_shapes() -> Document {
    let mut e = start(480, 300, t("Background"), 12.0);
    boxed(&mut e, t("Background"), rect(0.0, 0.0, 480.0, 300.0), 0.0, INK);
    centered(&mut e, t("Shape tween"), 240.0, 250.0, 18.0, PALE, TextStyle::default());
    layer(&mut e, t("Shape"));
    add(&mut e, t("Square"), Shape::Rect { radius: 0.0 }, rect(60.0, 90.0, 100.0, 100.0), Some(BLUE), None);
    // Each new shape is drawn on a blank keyframe: a shape tween changes one object into another.
    let blank = |e: &mut Editor, frame: usize| {
        e.goto_frame(frame);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, true));
    };
    blank(&mut e, 15);
    add(&mut e, t("Circle"), Shape::Ellipse, rect(180.0, 80.0, 120.0, 120.0), Some(ORANGE), None);
    blank(&mut e, 30);
    add(&mut e, t("Star"), Shape::Auto { kind: AutoKind::Star, a: 0.45, b: 0.0, count: 5 }, rect(310.0, 70.0, 140.0, 140.0), Some(YELLOW), None);
    // The star stays for a while before the film starts over.
    e.on_timeline(|timeline, layer, frame| timeline.insert_frames(layer, frame, 8));
    tween(&mut e, 0, Tween { shape: true, ..Tween::default() });
    tween(&mut e, 15, Tween { shape: true, ease: 60.0, ..Tween::default() });
    fill_out(&mut e);
    finish(e)
}

fn along_a_path() -> Document {
    let mut e = start(520, 320, t("Background"), 24.0);
    boxed(&mut e, t("Background"), rect(0.0, 0.0, 520.0, 320.0), 0.0, PALE);
    layer(&mut e, t("Arrow"));
    let arrow = add(&mut e, t("Arrow"), Shape::Auto { kind: AutoKind::Arrow, a: 0.5, b: 0.5, count: 0 }, rect(20.0, 235.0, 60.0, 30.0), Some(BLUE), None);
    keyframe(&mut e, 47, &[arrow], |o| o.translate(420.0, 0.0));
    tween(&mut e, 0, Tween { orient: true, ..Tween::default() });
    e.goto_frame(0);
    // The path the arrow keeps to: up over a hill and down again. It is a guide, and is not part of the film.
    layer(&mut e, t("Guide"));
    let hill: Vec<Vec2> = (0..=24).map(|step| {
        let part = step as f32 / 24.0;
        Vec2 { x: 50.0 + part * 420.0, y: 250.0 - (part * std::f32::consts::PI).sin() * 170.0 }
    }).collect();
    let path = add(&mut e, t("Path"), Shape::Path { closed: false }, crate::model::Rect::of_points(&hill), None, Some(Stroke { color: INK, width: 1.0 }));
    if let Some(object) = e.object_mut(path) {
        object.points = hill;
    }
    e.doc.page_mut().edit_timeline(|timeline| {
        let top = timeline.layers.len() - 1;
        timeline.set_kind(top, LayerKind::Guide);
    });
    fill_out(&mut e);
    finish(e)
}

fn spotlight() -> Document {
    let mut e = start(520, 200, t("Background"), 24.0);
    boxed(&mut e, t("Background"), rect(0.0, 0.0, 520.0, 200.0), 0.0, INK);
    // The same words twice: dim underneath, bright on the layer that the mask lets through.
    centered(&mut e, "PisiAnimasyon", 260.0, 68.0, 50.0, Color::rgb(0x34, 0x40, 0x5c), bold());
    layer(&mut e, t("Bright"));
    boxed(&mut e, t("Light"), rect(0.0, 0.0, 520.0, 200.0), 0.0, Color::rgb(0x2c, 0x3a, 0x58));
    centered(&mut e, "PisiAnimasyon", 260.0, 68.0, 50.0, YELLOW, bold());
    layer(&mut e, t("Mask"));
    let light = add(&mut e, t("Spot"), Shape::Ellipse, rect(-20.0, 30.0, 140.0, 140.0), Some(WHITE), None);
    keyframe(&mut e, 30, &[light], |o| o.translate(420.0, 0.0));
    keyframe(&mut e, 60, &[light], |o| o.translate(-420.0, 0.0));
    tween(&mut e, 0, Tween { ease: 40.0, ..Tween::default() });
    tween(&mut e, 30, Tween { ease: 40.0, ..Tween::default() });
    e.doc.page_mut().edit_timeline(|timeline| {
        let top = timeline.layers.len() - 1;
        timeline.set_kind(top, LayerKind::Mask);
    });
    fill_out(&mut e);
    finish(e)
}

fn turning_gears() -> Document {
    let mut e = start(480, 300, t("Background"), 24.0);
    boxed(&mut e, t("Background"), rect(0.0, 0.0, 480.0, 300.0), 0.0, PALE);
    layer(&mut e, t("Gears"));
    // One gear, made a movie clip: on its own timeline it turns once round in two seconds.
    let gear = add(&mut e, t("Gear"), Shape::Auto { kind: AutoKind::Gear, a: 0.7, b: 0.0, count: 12 }, rect(90.0, 60.0, 180.0, 180.0), Some(TEAL), None);
    e.selection = vec![gear];
    let Some(symbol) = e.make_symbol() else { return finish(e) };
    let large = e.selection.first().copied();
    if let Some(entry) = e.doc.symbols.iter_mut().find(|s| s.id == symbol) {
        (entry.kind, entry.name) = (SymbolKind::Animation, t("Gear").to_string());
    }
    e.open_clip(symbol);
    let part = e.doc.page().objects().next().map(|o| o.id);
    keyframe(&mut e, 47, &part.into_iter().collect::<Vec<_>>(), |_| {});
    tween(&mut e, 0, Tween { turns: 1, ..Tween::default() });
    e.goto_frame(0);
    e.leave_clip();
    // A second, smaller copy of the same clip beside the first.
    e.place_symbol_at(symbol, Vec2 { x: 345.0, y: 150.0 });
    e.for_selected(|o| o.set_bounds(rect(285.0, 90.0, 120.0, 120.0)));
    e.selection = large.into_iter().collect();
    finish(e)
}

fn one_after_another() -> Document {
    // Twelve frames a second, eight seconds: every layer has its own keyframes, so each part has its own time.
    let mut e = start(520, 300, t("Background"), 12.0);
    let sky = boxed(&mut e, t("Sky"), rect(0.0, 0.0, 520.0, 300.0), 0.0, PALE);
    gradient(&mut e, sky, FillKind::Linear, Color::rgb(0xc8, 0xdc, 0xf4), WHITE, (0.5, 0.0), (0.5, 1.0));
    // A blank keyframe on a later frame: until then the layer shows nothing.
    let blank = |e: &mut Editor, frame: usize| {
        e.goto_frame(frame);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, true));
    };
    // The sun is not there for the first four seconds. Then it rises for two, and stays.
    layer(&mut e, t("Sun"));
    blank(&mut e, 48);
    let sun = add(&mut e, t("Sun"), Shape::Ellipse, rect(380.0, 240.0, 80.0, 80.0), Some(YELLOW), None);
    gradient(&mut e, sun, FillKind::Radial, YELLOW, ORANGE, (0.5, 0.5), (1.0, 0.5));
    keyframe(&mut e, 72, &[sun], |o| o.translate(0.0, -200.0));
    tween(&mut e, 48, Tween { ease: 100.0, ..Tween::default() });
    e.goto_frame(0);
    layer(&mut e, t("Ground"));
    boxed(&mut e, t("Ground"), rect(0.0, 230.0, 520.0, 70.0), 0.0, TEAL);
    // The car drives for the first four seconds. Its last keyframe has no tween: it stands still to the end.
    layer(&mut e, t("Car"));
    let body = boxed(&mut e, t("Car"), rect(20.0, 186.0, 110.0, 34.0), 8.0, BLUE);
    let top = boxed(&mut e, t("Car"), rect(42.0, 162.0, 60.0, 30.0), 8.0, BLUE);
    let back = add(&mut e, t("Wheel"), Shape::Ellipse, rect(34.0, 208.0, 28.0, 28.0), Some(INK), None);
    let front = add(&mut e, t("Wheel"), Shape::Ellipse, rect(90.0, 208.0, 28.0, 28.0), Some(INK), None);
    keyframe(&mut e, 48, &[body, top, back, front], |o| o.translate(250.0, 0.0));
    tween(&mut e, 0, Tween { ease: 100.0, ..Tween::default() });
    e.goto_frame(0);
    // The words come last: from the sixth second they fade in for a second.
    layer(&mut e, t("Words"));
    blank(&mut e, 72);
    let words = centered(&mut e, t("Good morning!"), 200.0, 60.0, 40.0, INK, bold());
    e.selection = vec![words];
    e.for_selected(|o| o.opacity = 0.0);
    keyframe(&mut e, 84, &[words], |o| o.opacity = 1.0);
    tween(&mut e, 72, Tween::default());
    // The film is eight seconds long: the last keyframes last until then.
    e.on_timeline(|timeline, layer, _| timeline.insert_frames(layer, 95, 1));
    fill_out(&mut e);
    finish(e)
}

/// Makes the scene wait on the frame on show: without a Stop the film would go on to the next scene.
pub(crate) fn stop_here(e: &mut Editor) {
    e.on_timeline(|timeline, layer, frame| {
        if let Some(at) = timeline.span_at(layer, frame) {
            timeline.layers[layer][at].action = Action::Stop.text();
        }
    });
}

pub(crate) fn rule(event: (EventKind, &str, &str, f32), steps: &[(StepKind, &str, f32, f32)]) -> Rule {
    Rule {
        event: Event { kind: event.0, a: event.1.to_string(), b: event.2.to_string(), n: event.3 },
        steps: steps.iter().map(|(kind, a, x, y)| Step { kind: *kind, a: a.to_string(), x: *x, y: *y, ..Default::default() }).collect(),
        ..Default::default()
    }
}

pub(crate) fn give(e: &mut Editor, id: u64, list: &[(BehaviorKind, f32, f32)]) {
    e.selection = vec![id];
    e.for_selected(|o| o.behaviors = list.iter().map(|(kind, a, b)| Behavior { kind: *kind, a: *a, b: *b }).collect());
}

/// A text that shows a number: "Score: {Score}".
pub(crate) fn counter(e: &mut Editor, number: &str, x: f32, y: f32, size: f32, color: Color) -> u64 {
    crate::examples::text(e, &format!("{number}: {{{number}}}"), x, y, size, color, bold())
}

/// A scene the game ends on: a word, the score, and a box to click that starts the game again.
pub(crate) fn last_scene(e: &mut Editor, width: f32, height: f32, word: &str, score: &str) {
    e.add_page();
    e.doc.page_mut().layers[0].name = t("Background").to_string();
    let back = boxed(e, t("Background"), rect(0.0, 0.0, width, height), 0.0, INK);
    gradient(e, back, FillKind::Linear, INK, TEAL, (0.5, 0.0), (0.5, 1.0));
    // A star turns slowly behind the words.
    let rays = add(e, t("Star"), Shape::Auto { kind: AutoKind::Star, a: 0.6, b: 0.0, count: 16 }, rect(width / 2.0 - 150.0, height * 0.3 - 150.0, 300.0, 300.0), Some(WHITE), None);
    e.for_selected(|o| o.opacity = 0.08);
    give(e, rays, &[(BehaviorKind::Spin, 24.0, 0.0)]);
    centered(e, word, width / 2.0, height * 0.18, 40.0, WHITE, bold());
    centered(e, score, width / 2.0, height * 0.42, 26.0, YELLOW, bold());
    // The box is what the rule is about; the words on it are only words.
    boxed(e, t("Again"), rect(width / 2.0 - 90.0, height * 0.66, 180.0, 44.0), 10.0, BLUE);
    centered(e, t("Play again"), width / 2.0, height * 0.66 + 10.0, 18.0, WHITE, bold());
    stop_here(e);
}

fn behaviors() -> Document {
    use crate::behavior::BehaviorKind as Kind;
    // One frame, and not a keyframe more: everything that moves here has been told to with a click.
    // What falls lands on the bottom edge of the stage, so the stage ends where the ground is.
    let mut e = start(560, 300, t("Background"), 24.0);
    let sky = boxed(&mut e, t("Sky"), rect(0.0, 0.0, 560.0, 300.0), 0.0, PALE);
    gradient(&mut e, sky, FillKind::Linear, Color::rgb(0xbc, 0xdc, 0xf8), WHITE, (0.5, 0.0), (0.5, 1.0));
    centered(&mut e, t("Arrows walk the box, Up jumps. The bee follows the mouse. Drag the star."), 280.0, 42.0, 12.0, INK, TextStyle::default());
    layer(&mut e, t("Behaviors"));
    let sun = add(&mut e, t("Sun"), Shape::Auto { kind: AutoKind::Star, a: 0.6, b: 0.0, count: 12 }, rect(24.0, 20.0, 76.0, 76.0), Some(YELLOW), Some(Stroke { color: ORANGE, width: 2.0 }));
    give(&mut e, sun, &[(Kind::Spin, 8.0, 0.0)]);
    let cloud = boxed(&mut e, t("Cloud"), rect(130.0, 46.0, 110.0, 36.0), 18.0, WHITE);
    give(&mut e, cloud, &[(Kind::Drift, 40.0, 0.0)]);
    let cloud = boxed(&mut e, t("Cloud"), rect(380.0, 92.0, 80.0, 28.0), 14.0, WHITE);
    give(&mut e, cloud, &[(Kind::Drift, 22.0, 0.0)]);
    let title = centered(&mut e, "PisiAnimasyon", 280.0, 8.0, 26.0, BLUE, bold());
    give(&mut e, title, &[(Kind::Fade, 3.0, 0.0)]);
    // A greeting that is gone after three seconds.
    let hello = centered(&mut e, t("Hello!"), 500.0, 120.0, 18.0, TEAL, bold());
    give(&mut e, hello, &[(Kind::Expire, 3.0, 0.0)]);
    let look = centered(&mut e, t("Look!"), 500.0, 150.0, 22.0, ORANGE, bold());
    give(&mut e, look, &[(Kind::Blink, 1.0, 0.0)]);
    let bell = add(&mut e, t("Bell"), Shape::Auto { kind: AutoKind::Polygon, a: 0.0, b: 0.0, count: 6 }, rect(34.0, 236.0, 48.0, 48.0), Some(Color::rgb(0xc8, 0x30, 0x30)), None);
    give(&mut e, bell, &[(Kind::Shake, 3.0, 0.0)]);
    let ball = add(&mut e, t("Ball"), Shape::Ellipse, rect(120.0, 244.0, 56.0, 56.0), Some(ORANGE), None);
    gradient(&mut e, ball, FillKind::Radial, YELLOW, ORANGE, (0.35, 0.35), (1.0, 0.5));
    give(&mut e, ball, &[(Kind::Bounce, 110.0, 0.9)]);
    let arrow = add(&mut e, t("Arrow"), Shape::Auto { kind: AutoKind::Arrow, a: 0.5, b: 0.5, count: 0 }, rect(200.0, 150.0, 60.0, 30.0), Some(BLUE), None);
    give(&mut e, arrow, &[(Kind::BackAndForth, 120.0, 2.0)]);
    let balloon = add(&mut e, t("Balloon"), Shape::Ellipse, rect(420.0, 190.0, 44.0, 56.0), Some(Color::rgb(0xd8, 0x40, 0x60)), None);
    give(&mut e, balloon, &[(Kind::UpAndDown, 50.0, 2.4)]);
    let star = add(&mut e, t("Star"), Shape::Auto { kind: AutoKind::Star, a: 0.45, b: 0.0, count: 5 }, rect(480.0, 230.0, 60.0, 60.0), Some(YELLOW), Some(Stroke { color: ORANGE, width: 2.0 }));
    give(&mut e, star, &[(Kind::Pulse, 25.0, 1.0), (Kind::Draggable, 0.0, 0.0)]);
    let bee = add(&mut e, t("Bee"), Shape::Ellipse, rect(300.0, 110.0, 28.0, 20.0), Some(YELLOW), Some(Stroke { color: INK, width: 3.0 }));
    give(&mut e, bee, &[(Kind::FollowMouse, 220.0, 0.0)]);
    // A marble that never stops: it comes back from every edge, and from the ledge too.
    let marble = add(&mut e, t("Marble"), Shape::Ellipse, rect(250.0, 70.0, 18.0, 18.0), Some(Color::rgb(0x70, 0x40, 0xc0)), None);
    give(&mut e, marble, &[(Kind::Ricochet, 170.0, 35.0)]);
    // A wheel the arrows roll along the ground: it turns as far as it goes.
    let wheel = add(&mut e, t("Wheel"), Shape::Auto { kind: AutoKind::Star, a: 0.75, b: 0.0, count: 8 }, rect(200.0, 268.0, 32.0, 32.0), Some(INK), None);
    give(&mut e, wheel, &[(Kind::ArrowKeys, 160.0, 0.0), (Kind::Roll, 0.0, 0.0)]);
    // A ledge the box can jump onto: it is solid.
    let ledge = boxed(&mut e, t("Ledge"), rect(330.0, 240.0, 80.0, 12.0), 4.0, TEAL);
    give(&mut e, ledge, &[(Kind::Solid, 0.0, 0.0)]);
    let walker = boxed(&mut e, t("Box"), rect(260.0, 258.0, 36.0, 42.0), 8.0, INK);
    give(&mut e, walker, &[(Kind::ArrowKeys, 160.0, 0.0), (Kind::Gravity, 900.0, 420.0)]);
    fill_out(&mut e);
    finish(e)
}

/// How a stick figure stands on one frame. Angles are in degrees. A limb's angle is 0 hanging straight down
/// the body and grows as the limb swings forward, the way the figure faces; an elbow's or a knee's is
/// measured from the limb above it. The first arm and leg are the far ones, drawn behind the body.
#[derive(Clone, Copy)]
struct Pose {
    /// Where the hips are along the ground, and how high above it.
    x: f32,
    up: f32,
    /// How far the whole body leans forward.
    lean: f32,
    arms: [(f32, f32); 2],
    legs: [(f32, f32); 2],
    opacity: f32,
}

const GROUND: f32 = 250.0;
const BODY: f32 = 46.0;
const ARM: f32 = 22.0;
const LEG: f32 = 27.0;
const HEAD: f32 = 11.0;

impl Pose {
    /// Standing on the ground: the hips are as high as the leg that reaches lowest.
    fn standing(x: f32, lean: f32, arms: [(f32, f32); 2], legs: [(f32, f32); 2]) -> Pose {
        let reach = |(hip, knee): (f32, f32)| LEG * ((hip - lean).to_radians().cos() + (hip + knee - lean).to_radians().cos());
        Pose { x, up: reach(legs[0]).max(reach(legs[1])), lean, arms, legs, opacity: 1.0 }
    }

    fn mixed(&self, to: &Pose, part: f32) -> Pose {
        let mix = |a: f32, b: f32| a + (b - a) * part;
        let both = |a: (f32, f32), b: (f32, f32)| (mix(a.0, b.0), mix(a.1, b.1));
        Pose {
            x: mix(self.x, to.x),
            up: mix(self.up, to.up),
            lean: mix(self.lean, to.lean),
            arms: [both(self.arms[0], to.arms[0]), both(self.arms[1], to.arms[1])],
            legs: [both(self.legs[0], to.legs[0]), both(self.legs[1], to.legs[1])],
            opacity: mix(self.opacity, to.opacity),
        }
    }

    /// The figure's lines, each from the joint it turns at to its far end, in the order they are drawn in,
    /// and the middle of the head. `facing` is 1 for a figure that looks right and -1 for one that looks left.
    fn lines(&self, facing: f32) -> ([(Vec2, Vec2); 9], Vec2) {
        let hips = Vec2 { x: self.x, y: GROUND - self.up };
        let reach = |from: Vec2, angle: f32, length: f32| {
            let (sin, cos) = (angle - self.lean).to_radians().sin_cos();
            Vec2 { x: from.x + facing * sin * length, y: from.y + cos * length }
        };
        let neck = reach(hips, 180.0, BODY);
        let limb = |from: Vec2, (upper, lower): (f32, f32), length: f32| {
            let joint = reach(from, upper, length);
            [(from, joint), (joint, reach(joint, upper + lower, length))]
        };
        let (far_arm, near_arm) = (limb(neck, self.arms[0], ARM), limb(neck, self.arms[1], ARM));
        let (far_leg, near_leg) = (limb(hips, self.legs[0], LEG), limb(hips, self.legs[1], LEG));
        ([far_arm[0], far_arm[1], far_leg[0], far_leg[1], (hips, neck), near_leg[0], near_leg[1], near_arm[0], near_arm[1]], reach(neck, 180.0, HEAD + 1.0))
    }
}

/// How often a figure gets a keyframe: on every second frame, as hand-drawn animation is shot "on twos".
/// On the frame between, the motion tween turns each line around its joint.
const ON_TWOS: usize = 2;

/// A layer with a stick figure that goes from pose to pose. `poses` name the frames the figure must be in a
/// pose on; between them it gets a keyframe on every second frame, and a motion tween from each to the next.
/// Before its first pose and after its last, unless it is the film's last, the layer is empty.
fn figure(e: &mut Editor, name: &str, color: Color, facing: f32, poses: &[(usize, Pose)], leaves: bool) {
    layer(e, name);
    let ids: Vec<u64> = (0..10).map(|_| e.alloc_id()).collect();
    let names = [t("Arm"), t("Arm"), t("Leg"), t("Leg"), t("Body"), t("Leg"), t("Leg"), t("Arm"), t("Arm")];
    let drawn = |pose: &Pose| -> Vec<Object> {
        let (lines, head) = pose.lines(facing);
        let mut objects: Vec<Object> = lines
            .iter()
            .enumerate()
            .map(|(index, (from, to))| {
                let mut object = Object::new_web(ids[index], WebKind::Slice, crate::model::Rect::of_points(&[*from, *to]));
                (object.name, object.shape, object.fill, object.stroke) = (names[index].to_string(), Shape::Line, None, Some(Stroke { color, width: 6.0 }));
                object.points = vec![*from, *to];
                // Each line turns at the joint it starts from: the shoulder, the elbow, the hip, the knee.
                object.set_pivot_point(Some(*from));
                object.opacity = pose.opacity;
                object
            })
            .collect();
        let mut top = Object::new_web(ids[9], WebKind::Slice, rect(head.x - HEAD, head.y - HEAD, HEAD * 2.0, HEAD * 2.0));
        (top.name, top.shape, top.fill, top.stroke, top.opacity) = (t("Head").to_string(), Shape::Ellipse, Some(color), None, pose.opacity);
        objects.insert(5, top);
        objects
    };
    let (first, last) = (poses[0].0, poses[poses.len() - 1].0);
    let at = |frame: usize| -> Pose {
        let next = poses.iter().position(|(on, _)| *on >= frame).unwrap_or(poses.len() - 1);
        let (to_frame, to) = poses[next];
        if next == 0 || to_frame == frame {
            return to;
        }
        let (from_frame, from) = poses[next - 1];
        // Slow out of a pose and slow into the next.
        let part = (frame - from_frame) as f32 / (to_frame - from_frame) as f32;
        from.mixed(&to, part * part * (3.0 - 2.0 * part))
    };
    let mut spans = Vec::new();
    if first > 0 {
        spans.push(Span { start: 0, length: first, ..Default::default() });
    }
    // A blow is too fast for that: where a limb swings far in two frames, each frame gets a keyframe of its own,
    // or the forearm, which goes straight from keyframe to keyframe, would leave the elbow, which goes round.
    let fast = |a: &Pose, b: &Pose| {
        let swing = |p: (f32, f32), q: (f32, f32)| (p.0 - q.0).abs().max((p.1 - q.1).abs());
        (0..2).any(|side| swing(a.arms[side], b.arms[side]) > 30.0 || swing(a.legs[side], b.legs[side]) > 30.0) || (a.lean - b.lean).abs() > 30.0
    };
    let mut frame = first;
    while frame <= last {
        let length = if frame + ON_TWOS > last || fast(&at(frame), &at(frame + ON_TWOS)) { 1 } else { ON_TWOS };
        spans.push(Span { start: frame, length, objects: drawn(&at(frame)), tween: (frame < last).then(Tween::default), ..Default::default() });
        frame += length;
    }
    if leaves {
        spans.push(Span { start: last + 1, length: 1, ..Default::default() });
    }
    e.doc.page_mut().edit_timeline(|timeline| {
        let top = timeline.layers.len() - 1;
        timeline.layers[top] = spans;
    });
}

fn stick_fight() -> Document {
    let mut e = start(520, 300, t("Background"), 24.0);
    let sky = boxed(&mut e, t("Sky"), rect(0.0, 0.0, 520.0, 300.0), 0.0, PALE);
    gradient(&mut e, sky, FillKind::Linear, Color::rgb(0xf4, 0xe4, 0xc0), WHITE, (0.5, 0.0), (0.5, 1.0));
    boxed(&mut e, t("Ground"), rect(0.0, GROUND + 3.0, 520.0, 50.0), 0.0, Color::rgb(0xc8, 0xb4, 0x90));
    // The poses the figures go through. Everything else is these with other numbers.
    let guard = |x: f32| Pose::standing(x, 8.0, [(30.0, 110.0), (55.0, 95.0)], [(-22.0, -12.0), (28.0, -30.0)]);
    let low = |x: f32| Pose::standing(x, 12.0, [(34.0, 112.0), (58.0, 100.0)], [(-26.0, -16.0), (32.0, -40.0)]);
    let stride = |x: f32, other: bool| {
        let (a, b) = ((48.0, -18.0), (-24.0, -78.0));
        let (c, d) = ((-44.0, 80.0), (46.0, 76.0));
        let pose = Pose::standing(x, 16.0, if other { [d, c] } else { [c, d] }, if other { [b, a] } else { [a, b] });
        Pose { up: pose.up + 4.0, ..pose }
    };
    // Running in from the right edge to where the fight is, a stride every six frames.
    let run_in = |from: usize, to: usize, stop: f32| -> Vec<(usize, Pose)> {
        let strides = (to - from) / 6;
        (0..=strides).map(|step| (from + step * 6, stride(560.0 + (stop - 560.0) * step as f32 / strides as f32, step % 2 == 1))).collect()
    };
    let hurled = |x: f32, up: f32, lean: f32| Pose { x, up, lean, arms: [(70.0, 30.0), (110.0, 20.0)], legs: [(30.0, -30.0), (60.0, -50.0)], opacity: 1.0 };
    let lying = |x: f32, opacity: f32| Pose { x, up: 5.0, lean: -90.0, arms: [(4.0, 6.0), (-185.0, -8.0)], legs: [(-6.0, 0.0), (12.0, -18.0)], opacity };

    // The first enemy runs in, squares up, and is knocked flat by the punch on frame 36.
    let mut first = run_in(0, 24, 256.0);
    first.extend([(30, guard(252.0)), (36, guard(252.0)), (38, hurled(276.0, 62.0, -30.0)), (44, hurled(340.0, 66.0, -70.0)), (50, hurled(400.0, 30.0, -88.0)), (54, lying(420.0, 1.0)), (60, lying(420.0, 1.0)), (68, lying(420.0, 0.0))]);
    figure(&mut e, t("Enemy 1"), Color::rgb(0xc8, 0x30, 0x30), -1.0, &first, true);
    // The second is not there until frame 54. The kick on frame 88 sends it spinning off the stage.
    let mut second = run_in(54, 78, 258.0);
    second.extend([(82, guard(254.0)), (88, guard(254.0)), (90, hurled(285.0, 80.0, -40.0)), (96, hurled(370.0, 125.0, -220.0)), (102, hurled(465.0, 150.0, -400.0)), (108, hurled(590.0, 150.0, -580.0))]);
    figure(&mut e, t("Enemy 2"), Color::rgb(0x30, 0x58, 0xc0), -1.0, &second, true);

    // The hero waits on guard, punches, waits, kicks, and throws both arms up.
    let punch = Pose::standing(190.0, 20.0, [(-30.0, 70.0), (72.0, 0.0)], [(-34.0, -6.0), (36.0, -42.0)]);
    let drawn_back = Pose::standing(156.0, 0.0, [(40.0, 100.0), (-20.0, 130.0)], [(-20.0, -14.0), (26.0, -28.0)]);
    let knee_up = Pose::standing(186.0, -8.0, [(-40.0, 80.0), (40.0, 100.0)], [(-4.0, -6.0), (95.0, -110.0)]);
    let kick = Pose::standing(196.0, -24.0, [(-60.0, 60.0), (30.0, 100.0)], [(-8.0, -4.0), (112.0, -4.0)]);
    let cheer = |up: f32| {
        let pose = Pose::standing(150.0, 0.0, [(-150.0, -20.0), (150.0, 20.0)], [(-14.0, -4.0), (14.0, -6.0)]);
        Pose { up: pose.up + up, ..pose }
    };
    let hero = [
        (0, guard(150.0)),
        (10, low(150.0)),
        (20, guard(150.0)),
        (30, low(150.0)),
        (34, drawn_back),
        (36, punch),
        (40, punch),
        (46, guard(150.0)),
        (56, low(150.0)),
        (66, guard(150.0)),
        (76, low(150.0)),
        (82, low(160.0)),
        (86, knee_up),
        (88, kick),
        (92, kick),
        (98, guard(160.0)),
        (110, Pose::standing(150.0, 0.0, [(-10.0, 14.0), (12.0, 16.0)], [(-12.0, -4.0), (12.0, -6.0)])),
        (118, cheer(0.0)),
        (124, cheer(22.0)),
        (130, cheer(0.0)),
        (136, cheer(22.0)),
        (142, cheer(0.0)),
        (156, cheer(0.0)),
    ];
    figure(&mut e, t("Hero"), INK, 1.0, &hero, false);

    // A star bursts where each blow lands, and is gone four frames later.
    let blank = |e: &mut Editor, frame: usize| {
        e.goto_frame(frame);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, true));
    };
    layer(&mut e, t("Hit"));
    for (frame, x, y) in [(36, 246.0, 150.0), (88, 248.0, 158.0)] {
        blank(&mut e, frame);
        let star = add(&mut e, t("Star"), Shape::Auto { kind: AutoKind::Star, a: 0.45, b: 0.0, count: 8 }, rect(x - 10.0, y - 10.0, 20.0, 20.0), Some(YELLOW), Some(Stroke { color: ORANGE, width: 2.0 }));
        keyframe(&mut e, frame + 4, &[star], |o| {
            o.set_bounds(rect(x - 34.0, y - 34.0, 68.0, 68.0));
            o.opacity = 0.0;
        });
        tween(&mut e, frame, Tween { ease: 100.0, ..Tween::default() });
        blank(&mut e, frame + 5);
    }
    e.goto_frame(0);
    layer(&mut e, t("Words"));
    for (frame, word, x, y) in [(36, t("Pow!"), 270.0, 80.0), (88, t("Wham!"), 280.0, 70.0)] {
        blank(&mut e, frame);
        centered(&mut e, word, x, y, 34.0, ORANGE, bold());
        blank(&mut e, frame + 10);
    }
    blank(&mut e, 130);
    let end = centered(&mut e, t("The end"), 330.0, 110.0, 44.0, INK, bold());
    e.selection = vec![end];
    e.for_selected(|o| o.opacity = 0.0);
    keyframe(&mut e, 144, &[end], |o| o.opacity = 1.0);
    tween(&mut e, 130, Tween::default());
    fill_out(&mut e);
    finish(e)
}

fn two_scenes() -> Document {
    let mut e = start(480, 300, t("Background"), 24.0);
    // A button that leads to a scene: a symbol with an Over look, and an action on its copy.
    let button = |e: &mut Editor, name: &str, x: f32, to_scene: usize| {
        let symbol = e.new_symbol(name, SymbolKind::Button);
        e.edit_symbol(symbol, None, true);
        boxed(e, t("Button"), rect(x, 220.0, 140.0, 40.0), 8.0, BLUE);
        centered(e, name, x + 70.0, 229.0, 17.0, WHITE, bold());
        e.set_symbol_look(1);
        e.copy_previous_look();
        let ids = e.selection.clone();
        for id in ids {
            if let Some(object) = e.object_mut(id) {
                if matches!(object.shape, Shape::Rect { .. }) {
                    object.fill = Some(Color::rgb(0x4a, 0x9d, 0xee));
                }
            }
        }
        e.set_symbol_look(0);
        e.leave_symbol();
        e.for_selected(|o| o.link = Action::Scene(to_scene).text());
    };
    // Each scene stops on its only frame, so the film waits for a button.
    e.doc.page_mut().name = t("Scene 1").to_string();
    let back = boxed(&mut e, t("Background"), rect(0.0, 0.0, 480.0, 300.0), 0.0, PALE);
    gradient(&mut e, back, FillKind::Linear, WHITE, Color::rgb(0xc8, 0xdc, 0xf4), (0.5, 0.0), (0.5, 1.0));
    centered(&mut e, t("The first scene"), 240.0, 80.0, 36.0, INK, bold());
    layer(&mut e, t("Buttons"));
    button(&mut e, t("Next"), 170.0, 2);
    stop_here(&mut e);
    e.add_page();
    e.doc.page_mut().layers[0].name = t("Background").to_string();
    let back = boxed(&mut e, t("Background"), rect(0.0, 0.0, 480.0, 300.0), 0.0, INK);
    gradient(&mut e, back, FillKind::Linear, INK, TEAL, (0.5, 0.0), (0.5, 1.0));
    centered(&mut e, t("The second scene"), 240.0, 80.0, 36.0, WHITE, bold());
    layer(&mut e, t("Buttons"));
    button(&mut e, t("Back"), 170.0, 1);
    stop_here(&mut e);
    e.set_active_page(0);
    finish(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_attached_in_the_menu_is_tied_in_the_published_film() {
        let doc = (EXAMPLES.iter().find(|e| e.name == "Driving a car").unwrap().build)();
        let wheels: Vec<_> = doc.pages[0].objects().filter(|o| o.attached == t("Car")).collect();
        assert_eq!(wheels.len(), 2);
        let html = crate::publish::html(&doc, "x").unwrap();
        // Each wheel says what it is attached to, and the car, which has no behavior, is a picture the page can follow.
        assert_eq!(html.matches("\"a\":\"Car\"").count(), 2);
        assert!(html.contains("\"l\":[[\"camera\",0,0]],\"n\":\"Car\""));
        // The road marks slide as the road does and come back in; the hills slide less.
        assert_eq!(html.matches("[\"scenery\",100,1]").count(), 8);
        assert!(html.contains("[\"scenery\",20,1]") && html.contains("[\"drift\",10,0],[\"scenery\",8,1]"));
        // The words at the top of the stage are a text called what it says: no list offers it. The car and the wheels are offered.
        let names = doc.chosen_names();
        assert!(names.contains(&t("Car").to_string()) && names.contains(&t("Wheel").to_string()));
        assert!(doc.object_names().iter().any(|name| name.starts_with("Hold Right")) && !names.iter().any(|name| name.starts_with("Hold Right")));
        // It is kept with the document.
        let again = crate::model::document_from_json(&crate::model::document_to_json(&doc).unwrap()).unwrap();
        assert_eq!(again.pages[0].objects().filter(|o| o.attached == t("Car")).count(), 2);
    }

    #[test]
    fn every_animation_example_builds_plays_and_is_published() {
        for example in EXAMPLES {
            let doc = (example.build)();
            let page = &doc.pages[0];
            let timeline = page.timeline.as_ref().unwrap_or_else(|| panic!("{} has no timeline", example.name));
            let frames = timeline.frame_count();
            assert_eq!(timeline.current, 0, "{}", example.name);
            // Every layer lasts the whole scene, so nothing drops out halfway.
            assert!(timeline.layers.iter().all(|spans| spans.last().is_some_and(|span| span.start + span.length == frames)), "{}", example.name);
            for frame in [0, frames / 2, frames - 1] {
                crate::export::render_page(&page.at_frame(frame).played()).unwrap();
            }
            let html = crate::publish::html(&doc, example.name).unwrap();
            assert!(html.contains("scenes: [["), "{}", example.name);
            let again = crate::model::document_from_json(&crate::model::document_to_json(&doc).unwrap()).unwrap();
            assert_eq!(again.pages.len(), doc.pages.len(), "{}", example.name);
        }
    }

    /// Writes frames of an example as pictures, to look at them:
    /// `EXAMPLE="Stick fight" FRAMES=0,36,78 cargo test -- --ignored example_frames`.
    #[test]
    #[ignore]
    fn example_frames() {
        let name = std::env::var("EXAMPLE").unwrap_or_else(|_| "Stick fight".to_string());
        let doc = (EXAMPLES.iter().find(|e| e.name == name).unwrap().build)();
        // The published page as well, to open in a browser.
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/target/example.html"), crate::publish::html(&doc, &name).unwrap()).unwrap();
        let frames = std::env::var("FRAMES").unwrap_or_else(|_| "0".to_string());
        // PAGE picks another page than the first: a later scene, or the page of a movie clip.
        let page = std::env::var("PAGE").ok().and_then(|p| p.parse::<usize>().ok()).unwrap_or(0).min(doc.pages.len() - 1);
        for frame in frames.split(',').filter_map(|f| f.trim().parse::<usize>().ok()) {
            let picture = crate::export::render_page(&doc.pages[page].at_frame(frame).played()).unwrap();
            picture.save_png(format!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/example-{:03}.png"), frame)).unwrap();
        }
    }

    #[test]
    fn the_examples_menu_lists_every_example_under_its_own_name() {
        let listed: Vec<(&str, u8)> = crate::menus::SAHNE_EXTRA_MENUS[0]
            .1
            .iter()
            .filter_map(|entry| match entry {
                crate::menus::Entry::Item(label, _, Some(crate::menus::Cmd::SahneExample(which))) => Some((*label, *which)),
                _ => None,
            })
            .collect();
        let wanted: Vec<(&str, u8)> = EXAMPLES.iter().enumerate().map(|(index, example)| (example.name, index as u8)).collect();
        assert_eq!(listed, wanted);
    }

    #[test]
    fn the_animation_examples_show_what_they_say() {
        let find = |name: &str| (EXAMPLES.iter().find(|e| e.name == name).unwrap().build)();
        // The ball is lowest in the middle of the film, and back where it began at the end.
        let doc = find("Bouncing ball");
        let ball_top = |frame: usize| doc.pages[0].at_frame(frame).layers[2].objects[0].bounds.y;
        assert!(ball_top(12) > 200.0 && ball_top(0) == ball_top(24) && ball_top(6) < 130.0, "falling, it is still slow a quarter of the way through");
        // Halfway between the square and the circle the shape is a path.
        let doc = find("Changing shapes");
        assert!(matches!(doc.pages[0].at_frame(7).layers[1].objects[0].shape, Shape::Path { closed: true }));
        // On the hill the arrow is high up and turned; the guide is not in the finished film.
        let doc = find("Along a path");
        let middle = doc.pages[0].at_frame(24);
        assert!(middle.layers[1].objects[0].bounds.center().y < 120.0);
        assert_eq!(middle.played().layers[2].objects.len(), 0);
        // The bright words show only through the mask.
        let doc = find("Spotlight");
        assert!(doc.pages[0].played().layers[1].objects.iter().all(|o| o.mask.is_some()));
        // The gear is one movie clip of 48 frames, placed twice.
        let doc = find("Turning gears");
        assert_eq!((doc.symbols.len(), doc.scenes().len(), doc.pages.len()), (1, 1, 2));
        assert_eq!(doc.pages[1].timeline.as_ref().unwrap().frame_count(), 48);
        assert_eq!(doc.pages[0].objects().filter(|o| matches!(o.shape, Shape::Instance { .. })).count(), 2);
        // Each layer keeps its own time: the car is done after four seconds, when the sun first shows; the words come at six.
        let doc = find("One after another");
        let page = &doc.pages[0];
        assert_eq!(page.timeline.as_ref().unwrap().frame_count(), 96);
        let (sun, car, words) = (|frame: usize| page.at_frame(frame).layers[1].objects.clone(), |frame: usize| page.at_frame(frame).layers[3].objects[0].bounds.x, |frame: usize| page.at_frame(frame).layers[4].objects.clone());
        assert!(sun(47).is_empty() && sun(48).len() == 1 && sun(72)[0].bounds.y < sun(60)[0].bounds.y && sun(95)[0].bounds.y == sun(72)[0].bounds.y);
        assert!(car(0) < car(24) && car(24) < car(48) && car(48) == car(95));
        assert!(words(71).is_empty() && words(72)[0].opacity == 0.0 && words(78)[0].opacity > 0.3 && words(95)[0].opacity == 1.0);
        // One frame, every kind of behavior, and each object with any is a picture of its own that the page moves.
        let doc = find("Behaviors");
        let page = &doc.pages[0];
        assert_eq!(page.timeline.as_ref().unwrap().frame_count(), 1);
        let used: std::collections::HashSet<_> = page.objects().flat_map(|o| o.behaviors.iter().map(|b| b.kind)).collect();
        // All but the two that move the view, which the car example shows: here they would carry the stage away.
        assert_eq!(used.len(), crate::behavior::KINDS.len() - 2);
        assert!(!used.contains(&crate::behavior::BehaviorKind::Camera) && !used.contains(&crate::behavior::BehaviorKind::Scenery));
        let html = crate::publish::html(&doc, "x").unwrap();
        assert_eq!(html.matches("[\"m\",[").count(), page.objects().filter(|o| !o.behaviors.is_empty()).count());
        assert!(html.contains("\"l\":[[\"keys\",160,0],[\"gravity\",900,420]]") && html.contains("[\"spin\",8,0]"));
        // Half a second into playing, the ball is off the ground and the sun has turned; the box waits for the keys.
        let (rest, playing) = (page.clone(), page.behaved(11));
        let named = |page: &crate::model::Page, name: &str| page.objects().find(|o| o.name == name).unwrap().clone();
        assert!(named(&playing, t("Ball")).bounds.y < named(&rest, t("Ball")).bounds.y - 80.0);
        assert_ne!(named(&playing, t("Sun")).outline(), named(&rest, t("Sun")).outline());
        assert_eq!(named(&playing, t("Box")).bounds, named(&rest, t("Box")).bounds);
        // The games: their rules read as sentences, the objects they are about are pictures of their own
        // with their names, and the texts that show numbers are written by the page.
        crate::i18n::set_language(crate::i18n::Language::English);
        let doc = find("Game: catch the apples");
        assert_eq!(doc.rules[1].sentence(), "When Apple touches Basket: increase Score by 1; make a copy of Plus at Basket; put Apple at the top, at random; make Apple faster by 4 percent; play the sound ding");
        assert_eq!(doc.rules[3].sentence(), "When Lives goes below 1: go to scene 2");
        let html = crate::publish::html(&doc, "x").unwrap();
        assert_eq!((html.matches("\"n\":\"Apple\"").count(), html.matches("\"n\":\"Basket\"").count(), html.matches("\"n\":\"Again\"").count()), (3, 1, 1));
        assert!(html.contains("numbers: {\"Score\":0,\"Lives\":5}") && html.contains("{\"e\":[\"edge\",\"Apple\",\"bottom\",0],\"c\":null,\"s\":[[\"add\",\"Lives\",-1,0,\"\"],[\"top\",\"Apple\",0,0,\"\"],[\"sound\",\"oops\",3,0,\"\"]]}"));
        assert_eq!(html.matches("[\"t\",").count(), 3, "the score and the lives on the first scene, the score on the last");
        assert!(html.contains("\"text\":\"Score: {Score}\""));
        // Every game carries the five sounds made for it, and its characters are groups: one picture each.
        assert_eq!((doc.sounds.len(), html.matches("data:audio/wav;base64,").count()), (5, 5));
        assert!(doc.pages[0].objects().filter(|o| o.name == "Apple").all(|o| matches!(o.shape, Shape::Group { .. })));
        let doc = find("Game: hit the targets");
        assert_eq!(doc.rules[2].sentence(), "Every 1 seconds: decrease Time by 1");
        assert_eq!(crate::publish::html(&doc, "x").unwrap().matches("\"n\":\"Target\"").count(), 4);
        // The platform game: three solid platforms and the ground, a door that waits for every coin, a sparkle for each coin taken.
        let doc = find("Game: platform");
        assert_eq!(doc.rules[3].sentence(), "When Hero touches Door, if Score is more than 2: play the sound fanfare; go to scene 2");
        assert_eq!(doc.rules[1].sentence(), "When Hero touches Coin: increase Score by 1; make a copy of Sparkle at Coin; remove Coin; play the sound ding");
        let html = crate::publish::html(&doc, "x").unwrap();
        assert_eq!((html.matches("[\"solid\",0,0]").count(), html.matches("\"n\":\"Coin\"").count(), html.matches("\"n\":\"Hero\"").count()), (4, 3, 1));
        assert!(html.contains("\"c\":[[\"Score\",\">\",2]]") && html.contains("[\"expire\",0.35,0]") && doc.scenes().len() == 3);
        // The shooter: every shot is a copy of the one bullet, made where the ship is; every hit a copy of the one explosion.
        let doc = find("Game: space shooter");
        assert_eq!(doc.rules[1].sentence(), "When the Space key is pressed: make a copy of Bullet at Ship; play the sound zap");
        assert_eq!(doc.rules[3].sentence(), "When Bullet touches Alien: increase Score by 1; make a copy of Explosion at Alien; remove Bullet; put Alien at the top, at random; make Alien faster by 6 percent; play the sound boom");
        let html = crate::publish::html(&doc, "x").unwrap();
        assert!(html.contains("[\"spawn\",\"Bullet\",0,0,\"Ship\"]") && html.contains("[\"spawn\",\"Explosion\",0,0,\"Alien\"]"));
        assert_eq!(html.matches("\"n\":\"Alien\"").count(), 5);
        let doc = find("Game: quiz");
        assert_eq!((doc.scenes().len(), doc.rules.len()), (4, 3));
        let html = crate::publish::html(&doc, "x").unwrap();
        assert_eq!((html.matches("\"n\":\"Right answer\"").count(), html.matches("\"n\":\"Wrong answer\"").count()), (3, 6));
        assert_eq!(html.matches("\"a\":\"action:stop\"").count(), 4, "every scene waits for its answer");
        // The fight: on a frame between two keyframes every forearm still starts where its upper arm ends, and every
        // shin where its thigh ends, because each line turns at its joint. The enemies are there only for their part.
        let doc = find("Stick fight");
        let page = &doc.pages[0];
        for frame in [9, 35, 37, 87, 89, 95, 121] {
            for layer in 1..=3 {
                let lines: Vec<Vec<Vec2>> = page.at_frame(frame).layers[layer].objects.iter().filter(|o| matches!(o.shape, Shape::Line | Shape::Path { .. })).map(|o| o.points.clone()).collect();
                for (upper, lower) in [(0, 1), (2, 3), (5, 6), (7, 8)] {
                    if let (Some(a), Some(b)) = (lines.get(upper), lines.get(lower)) {
                        let gap = (a[1].x - b[0].x).hypot(a[1].y - b[0].y);
                        assert!(gap < 2.5, "frame {frame}, layer {layer}: a joint has come apart by {gap}");
                        let length = (a[1].x - a[0].x).hypot(a[1].y - a[0].y);
                        assert!((length - if upper == 0 || upper == 7 { ARM } else { LEG }).abs() < 0.5, "frame {frame}: a limb has changed length to {length}");
                    }
                }
            }
        }
        let there = |layer: usize, frame: usize| !page.at_frame(frame).layers[layer].objects.is_empty();
        assert!(there(1, 0) && there(1, 60) && !there(1, 80) && !there(2, 10) && there(2, 70) && !there(2, 120) && there(3, 0) && there(3, 155));
        let fist = |frame: usize| page.at_frame(frame).layers[3].objects[9].points[1].x;
        assert!(fist(36) > fist(30) + 25.0 && fist(36) > fist(46) + 25.0, "the punch reaches furthest on frame 36");
        // Two scenes, each stopping on its frame, each with a button that leads to the other.
        let doc = find("Two scenes with buttons");
        assert_eq!(doc.scenes().len(), 2);
        let html = crate::publish::html(&doc, "x").unwrap();
        assert!(html.contains("\"action:scene:2\"") && html.contains("\"action:scene:1\"") && html.matches("\"a\":\"action:stop\"").count() == 2);
    }
}
