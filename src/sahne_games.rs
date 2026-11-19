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

//! The games that come with PisiAnimasyon: what rules and behaviors can make, dressed to be worth playing.
//!
//! Each is drawn here from the shapes a student has, grouped into characters, and its sounds are
//! made here as well, so that there is nothing to install. The rules are the few sentences the
//! guide teaches; what is added is the look: characters with faces, scenery that moves, a burst
//! where a blow lands, a sound for each thing that happens.

use crate::autoshape::AutoKind;
use crate::behavior::BehaviorKind as Kind;
use crate::effects::FilterKind;
use crate::examples::{add, bold, centered, filter, gradient, layer, BLUE, INK, ORANGE, PALE, WHITE, YELLOW};
use crate::i18n::{t, tf};
use crate::model::{Color, Document, Editor, FillKind, Rect, Shape, Stroke, TextStyle, Vec2};
use crate::rules::{Condition, EventKind, Rule, StepKind, Variable};
use crate::sahne_examples::{counter, give, last_scene, rule, start, stop_here};

pub(crate) const RED: Color = Color::rgb(0xd8, 0x30, 0x38);
pub(crate) const GREEN: Color = Color::rgb(0x4c, 0xb8, 0x50);
pub(crate) const DARK_GREEN: Color = Color::rgb(0x2c, 0x8a, 0x3c);
pub(crate) const BROWN: Color = Color::rgb(0x9a, 0x64, 0x30);
pub(crate) const DARK_BROWN: Color = Color::rgb(0x6a, 0x40, 0x1c);
pub(crate) const NIGHT: Color = Color::rgb(0x0c, 0x10, 0x28);

pub(crate) fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

pub(crate) fn oval(e: &mut Editor, x: f32, y: f32, w: f32, h: f32, fill: Color) -> u64 {
    add(e, "", Shape::Ellipse, r(x, y, w, h), Some(fill), None)
}

pub(crate) fn block(e: &mut Editor, x: f32, y: f32, w: f32, h: f32, radius: f32, fill: Color) -> u64 {
    add(e, "", Shape::Rect { radius }, r(x, y, w, h), Some(fill), None)
}

/// A closed shape through the points given.
pub(crate) fn poly(e: &mut Editor, points: &[(f32, f32)], fill: Color) -> u64 {
    let points: Vec<Vec2> = points.iter().map(|(x, y)| Vec2 { x: *x, y: *y }).collect();
    let id = add(e, "", Shape::Path { closed: true }, Rect::of_points(&points), Some(fill), None);
    if let Some(object) = e.object_mut(id) {
        object.points = points;
    }
    id
}

pub(crate) fn star(e: &mut Editor, x: f32, y: f32, size: f32, points: u32, depth: f32, fill: Color) -> u64 {
    add(e, "", Shape::Auto { kind: AutoKind::Star, a: depth, b: 0.0, count: points }, r(x, y, size, size), Some(fill), None)
}

/// Lighter at the top, darker at the bottom.
pub(crate) fn shaded(e: &mut Editor, id: u64, top: Color, bottom: Color) -> u64 {
    gradient(e, id, FillKind::Linear, top, bottom, (0.5, 0.0), (0.5, 1.0));
    id
}

/// Lit from the upper left, as a ball is.
pub(crate) fn rounded(e: &mut Editor, id: u64, light: Color, dark: Color) -> u64 {
    gradient(e, id, FillKind::Radial, light, dark, (0.35, 0.3), (1.0, 0.6));
    id
}

pub(crate) fn faint(e: &mut Editor, id: u64, opacity: f32) -> u64 {
    if let Some(object) = e.object_mut(id) {
        object.opacity = opacity;
    }
    id
}

pub(crate) fn outlined(e: &mut Editor, id: u64, color: Color, width: f32) -> u64 {
    if let Some(object) = e.object_mut(id) {
        object.stroke = Some(Stroke { color, width });
    }
    id
}

/// A glow of a color of its own around an object.
pub(crate) fn glowing(e: &mut Editor, id: u64, color: Color) -> u64 {
    filter(e, id, FilterKind::Glow, &[]);
    if let Some(made) = e.object_mut(id).and_then(|object| object.filters.last_mut()) {
        made.color = Some(color);
    }
    id
}

/// Parts made into one object with a name: what a rule knows, and what a behavior moves.
pub(crate) fn grouped(e: &mut Editor, name: &str, parts: Vec<u64>) -> u64 {
    e.selection = parts;
    e.group_selected();
    let id = e.selection.first().copied().unwrap_or_default();
    e.for_selected(|o| o.name = name.to_string());
    id
}

/// Two eyes looking the way `look` says: -1 left, 0 ahead, 1 right.
pub(crate) fn eyes(e: &mut Editor, x: f32, y: f32, gap: f32, size: f32, look: f32) -> Vec<u64> {
    let mut parts = Vec::new();
    for side in [0.0, 1.0] {
        let left = x + side * gap;
        parts.push(oval(e, left, y, size, size * 1.2, WHITE));
        parts.push(oval(e, left + size * 0.3 + look * size * 0.2, y + size * 0.4, size * 0.45, size * 0.55, INK));
    }
    parts
}

// ---- Sounds ---------------------------------------------------------------------------------

/// A short sound made of notes, each a pitch in hertz and a length in milliseconds, as a WAV file.
/// A pitch of 0 is a burst of noise. Every note dies away, so the sound is a pluck and not a drone.
pub(crate) fn wav(notes: &[(f32, u32)]) -> Vec<u8> {
    const RATE: u32 = 11025;
    let mut samples: Vec<u8> = Vec::new();
    let mut noise: u32 = 0x1234_5678;
    for (pitch, length) in notes {
        let count = RATE * length / 1000;
        for index in 0..count {
            let (time, part) = (index as f32 / RATE as f32, index as f32 / count as f32);
            let wave = if *pitch <= 0.0 {
                noise = noise.wrapping_mul(1_103_515_245).wrapping_add(12345);
                ((noise >> 16) & 0xff) as f32 / 127.5 - 1.0
            } else {
                // A sine with a little of the octave above it, which is brighter than a sine alone.
                let turn = std::f32::consts::TAU * pitch * time;
                turn.sin() * 0.8 + (turn * 2.0).sin() * 0.2
            };
            let loud = (1.0 - part).powf(1.6) * 0.6;
            samples.push((128.0 + wave * loud * 127.0).round().clamp(0.0, 255.0) as u8);
        }
    }
    let size = samples.len() as u32;
    let mut file = Vec::with_capacity(44 + samples.len());
    file.extend_from_slice(b"RIFF");
    file.extend_from_slice(&(36 + size).to_le_bytes());
    file.extend_from_slice(b"WAVEfmt ");
    file.extend_from_slice(&16u32.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&RATE.to_le_bytes());
    file.extend_from_slice(&RATE.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&8u16.to_le_bytes());
    file.extend_from_slice(b"data");
    file.extend_from_slice(&size.to_le_bytes());
    file.extend(samples);
    file
}

/// The sounds the games share, added to the document under the names the rules call them by.
pub(crate) struct Sounds {
    pub ding: &'static str,
    pub zap: &'static str,
    pub boom: &'static str,
    pub oops: &'static str,
    pub win: &'static str,
}

pub(crate) fn sounds(e: &mut Editor) -> Sounds {
    let names = Sounds { ding: t("ding"), zap: t("zap"), boom: t("boom"), oops: t("oops"), win: t("fanfare") };
    e.add_sound(names.ding, "audio/wav", &wav(&[(988.0, 60), (1319.0, 160)]));
    e.add_sound(names.zap, "audio/wav", &wav(&[(880.0, 40), (660.0, 40), (440.0, 50)]));
    e.add_sound(names.boom, "audio/wav", &wav(&[(0.0, 220)]));
    e.add_sound(names.oops, "audio/wav", &wav(&[(330.0, 120), (247.0, 220)]));
    e.add_sound(names.win, "audio/wav", &wav(&[(523.0, 110), (659.0, 110), (784.0, 110), (1047.0, 320)]));
    names
}

/// A step that plays a sound.
pub(crate) fn sound(name: &str) -> (StepKind, &str, f32, f32) {
    (StepKind::Sound, name, 0.0, 0.0)
}

/// A rule whose copies appear where another object is: `at` for each Spawn step, in order.
pub(crate) fn placed(mut made: Rule, at: &[&str]) -> Rule {
    for (step, place) in made.steps.iter_mut().filter(|step| step.kind == StepKind::Spawn).zip(at) {
        step.b = place.to_string();
    }
    made
}

// ---- Scenery --------------------------------------------------------------------------------

pub(crate) fn sky(e: &mut Editor, width: f32, height: f32, top: Color, bottom: Color) {
    let id = block(e, 0.0, 0.0, width, height, 0.0, top);
    shaded(e, id, top, bottom);
    e.for_selected(|o| o.name = t("Sky").to_string());
}

/// A cloud that drifts across the sky and comes back in from the other side.
fn cloud(e: &mut Editor, x: f32, y: f32, size: f32, speed: f32) {
    let parts = vec![oval(e, x, y + size * 0.25, size * 0.6, size * 0.4, WHITE), oval(e, x + size * 0.25, y, size * 0.6, size * 0.55, WHITE), oval(e, x + size * 0.55, y + size * 0.2, size * 0.6, size * 0.45, WHITE)];
    let id = grouped(e, t("Cloud"), parts);
    faint(e, id, 0.9);
    give(e, id, &[(Kind::Drift, speed, 0.0)]);
}

/// A sun whose rays turn. Gives back the rays, which have a behavior and so are an object of their own.
fn sun(e: &mut Editor, x: f32, y: f32, size: f32) -> u64 {
    let rays = star(e, x, y, size, 14, 0.7, Color::rgb(0xff, 0xe0, 0x70));
    give(e, rays, &[(Kind::Spin, 18.0, 0.0)]);
    let disc = oval(e, x + size * 0.2, y + size * 0.2, size * 0.6, size * 0.6, YELLOW);
    rounded(e, disc, Color::rgb(0xff, 0xf0, 0xa0), ORANGE);
    rays
}

fn hills(e: &mut Editor, width: f32, ground: f32) {
    for (x, w, h, color) in [(-80.0, 380.0, 190.0, Color::rgb(0x9c, 0xd4, 0x88)), (width - 360.0, 460.0, 240.0, Color::rgb(0x84, 0xc8, 0x78)), (160.0, 360.0, 130.0, Color::rgb(0x70, 0xbc, 0x68))] {
        oval(e, x, ground - h / 2.0, w, h, color);
    }
}

/// Earth with grass on top: the ground, or a platform. It is one object, to be made solid.
fn turf(e: &mut Editor, name: &str, x: f32, y: f32, w: f32, h: f32) -> u64 {
    let earth = block(e, x, y + 4.0, w, h - 4.0, 5.0, BROWN);
    shaded(e, earth, BROWN, DARK_BROWN);
    let grass = block(e, x - 2.0, y, w + 4.0, 10.0, 5.0, GREEN);
    shaded(e, grass, Color::rgb(0x7c, 0xdc, 0x64), DARK_GREEN);
    grouped(e, name, vec![earth, grass])
}

/// A burst that flashes where something was hit: it swells, fades, and is gone in a third of a second.
pub(crate) fn burst(e: &mut Editor, name: &str, x: f32, y: f32, size: f32, outer: Color, inner: Color) -> u64 {
    let parts = vec![star(e, x, y, size, 10, 0.5, outer), star(e, x + size * 0.22, y + size * 0.22, size * 0.56, 8, 0.45, inner)];
    let id = grouped(e, name, parts);
    give(e, id, &[(Kind::Pulse, 90.0, 0.7), (Kind::Spin, 1.4, 0.0), (Kind::Expire, 0.35, 0.0)]);
    id
}

// ---- The platform game ----------------------------------------------------------------------

/// The hero: a round fellow with a cap, looking right.
fn hero(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let mut parts = vec![block(e, x + 3.0, y + 31.0, 10.0, 7.0, 3.0, INK), block(e, x + 17.0, y + 31.0, 10.0, 7.0, 3.0, INK)];
    let body = block(e, x, y + 6.0, 30.0, 28.0, 12.0, BLUE);
    parts.push(shaded(e, body, Color::rgb(0x5c, 0xb0, 0xf8), Color::rgb(0x14, 0x58, 0xb0)));
    parts.push(oval(e, x + 7.0, y + 19.0, 16.0, 12.0, Color::rgb(0xbc, 0xe0, 0xff)));
    parts.extend(eyes(e, x + 7.0, y + 10.0, 10.0, 7.0, 1.0));
    parts.push(block(e, x + 2.0, y, 26.0, 10.0, 5.0, RED));
    parts.push(block(e, x + 16.0, y + 6.0, 16.0, 4.0, 2.0, RED));
    grouped(e, name, parts)
}

/// A slime that slides along the ground, frowning.
fn slime(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let body = oval(e, x, y, 36.0, 34.0, RED);
    let mut parts = vec![rounded(e, body, Color::rgb(0xff, 0x80, 0x70), Color::rgb(0xa8, 0x18, 0x28)), block(e, x, y + 16.0, 36.0, 10.0, 0.0, Color::rgb(0xb8, 0x20, 0x2c))];
    parts.extend(eyes(e, x + 7.0, y + 6.0, 13.0, 8.0, 0.0));
    parts.push(poly(e, &[(x + 5.0, y + 3.0), (x + 15.0, y + 7.0), (x + 15.0, y + 5.0)], INK));
    parts.push(poly(e, &[(x + 31.0, y + 3.0), (x + 21.0, y + 7.0), (x + 21.0, y + 5.0)], INK));
    grouped(e, name, parts)
}

fn coin(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let rim = oval(e, x, y, 22.0, 22.0, ORANGE);
    let face = oval(e, x + 3.0, y + 3.0, 16.0, 16.0, YELLOW);
    let parts = vec![rim, rounded(e, face, Color::rgb(0xff, 0xf4, 0xa8), Color::rgb(0xf0, 0xb0, 0x20)), star(e, x + 6.0, y + 6.0, 10.0, 5, 0.45, ORANGE)];
    let id = grouped(e, name, parts);
    glowing(e, id, YELLOW);
    give(e, id, &[(Kind::UpAndDown, 6.0, 1.2)]);
    id
}

fn door(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let frame = block(e, x - 4.0, y - 4.0, 46.0, 64.0, 8.0, DARK_BROWN);
    let leaf = block(e, x, y, 38.0, 60.0, 6.0, BROWN);
    let parts = vec![frame, shaded(e, leaf, Color::rgb(0xc8, 0x8c, 0x48), BROWN), block(e, x + 6.0, y + 8.0, 26.0, 20.0, 3.0, DARK_BROWN), block(e, x + 6.0, y + 34.0, 26.0, 20.0, 3.0, DARK_BROWN), oval(e, x + 28.0, y + 28.0, 6.0, 6.0, YELLOW)];
    grouped(e, name, parts)
}

pub fn platform_game() -> Document {
    let (score, lives, hero_name, coin_name, enemy, door_name, spark) = (t("Score"), t("Lives"), t("Hero"), t("Coin"), t("Enemy"), t("Door"), t("Sparkle"));
    let (width, height, ground) = (640.0, 400.0, 360.0);
    let mut e = start(640, 400, t("Background"), 30.0);
    let sfx = sounds(&mut e);
    sky(&mut e, width, height, Color::rgb(0x7c, 0xc4, 0xf8), Color::rgb(0xe4, 0xf4, 0xff));
    sun(&mut e, 520.0, 24.0, 84.0);
    hills(&mut e, width, ground);
    cloud(&mut e, 60.0, 50.0, 90.0, 14.0);
    cloud(&mut e, 330.0, 90.0, 70.0, 9.0);
    centered(&mut e, t("Collect the three coins, then go to the door. Arrows walk, Up jumps."), 320.0, 40.0, 13.0, INK, TextStyle::default());
    counter(&mut e, score, 14.0, 8.0, 20.0, INK);
    counter(&mut e, lives, 490.0, 8.0, 20.0, RED);
    layer(&mut e, t("Game"));
    // The ground and the platforms are solid: the hero stands on them and bumps its head on them.
    let id = turf(&mut e, t("Ground"), -10.0, ground, width + 20.0, 60.0);
    give(&mut e, id, &[(Kind::Solid, 0.0, 0.0)]);
    for (x, y, w) in [(150.0, 290.0, 130.0), (350.0, 225.0, 130.0), (500.0, 155.0, 120.0)] {
        let id = turf(&mut e, t("Platform"), x, y, w, 20.0);
        give(&mut e, id, &[(Kind::Solid, 0.0, 0.0)]);
        coin(&mut e, coin_name, x + w / 2.0 - 11.0, y - 40.0);
    }
    door(&mut e, door_name, 590.0, 300.0);
    let id = slime(&mut e, enemy, 290.0, 334.0);
    give(&mut e, id, &[(Kind::BackAndForth, 200.0, 3.2)]);
    // One sparkle is drawn and hidden; a copy flashes wherever a coin is taken.
    burst(&mut e, spark, 20.0, 60.0, 34.0, YELLOW, WHITE);
    let id = hero(&mut e, hero_name, 30.0, 322.0);
    give(&mut e, id, &[(Kind::ArrowKeys, 190.0, 0.0), (Kind::Gravity, 1100.0, 480.0)]);
    stop_here(&mut e);
    last_scene(&mut e, width, height, t("Well done"), &format!("{score}: {{{score}}}"));
    last_scene(&mut e, width, height, t("Game over"), &format!("{score}: {{{score}}}"));
    e.set_active_page(0);
    e.doc.variables = vec![Variable { name: score.to_string(), start: 0.0 }, Variable { name: lives.to_string(), start: 3.0 }];
    // The door lets the hero through only with every coin: a rule with "only if".
    let mut open = rule((EventKind::Touch, hero_name, door_name, 0.0), &[sound(sfx.win), (StepKind::GotoScene, "", 2.0, 0.0)]);
    open.when = Some(Condition { number: score.to_string(), how: ">".to_string(), value: 2.0, ..Default::default() });
    e.doc.rules = vec![
        rule((EventKind::Start, "", "", 0.0), &[(StepKind::Hide, spark, 0.0, 0.0)]),
        placed(rule((EventKind::Touch, hero_name, coin_name, 0.0), &[(StepKind::Add, score, 1.0, 0.0), (StepKind::Spawn, spark, 0.0, 0.0), (StepKind::Remove, coin_name, 0.0, 0.0), sound(sfx.ding)]), &[coin_name]),
        rule((EventKind::Touch, hero_name, enemy, 0.0), &[(StepKind::Add, lives, -1.0, 0.0), (StepKind::GoTo, hero_name, 30.0, 322.0), sound(sfx.oops)]),
        open,
        rule((EventKind::Number, lives, "<", 1.0), &[(StepKind::GotoScene, "", 3.0, 0.0)]),
        rule((EventKind::Click, t("Again"), "", 0.0), &[(StepKind::Restart, "", 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}

// ---- The space shooter ----------------------------------------------------------------------

fn ship(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let flame = poly(e, &[(x + 17.0, y + 38.0), (x + 24.0, y + 48.0), (x + 31.0, y + 38.0)], ORANGE);
    let wings = poly(e, &[(x, y + 40.0), (x + 24.0, y + 14.0), (x + 48.0, y + 40.0), (x + 34.0, y + 36.0), (x + 14.0, y + 36.0)], Color::rgb(0x2c, 0x60, 0xc0));
    let body = poly(e, &[(x + 24.0, y), (x + 34.0, y + 22.0), (x + 32.0, y + 40.0), (x + 16.0, y + 40.0), (x + 14.0, y + 22.0)], PALE);
    let parts = vec![flame, wings, shaded(e, body, WHITE, Color::rgb(0x9c, 0xb0, 0xd0)), oval(e, x + 19.0, y + 14.0, 10.0, 14.0, Color::rgb(0x40, 0xd0, 0xf0)), block(e, x + 2.0, y + 30.0, 5.0, 12.0, 2.0, RED), block(e, x + 41.0, y + 30.0, 5.0, 12.0, 2.0, RED)];
    grouped(e, name, parts)
}

/// A flying saucer with a creature under its dome.
fn saucer(e: &mut Editor, name: &str, x: f32, y: f32, dome: Color) -> u64 {
    let glass = oval(e, x + 11.0, y, 26.0, 24.0, dome);
    let mut parts = vec![rounded(e, glass, WHITE, dome)];
    parts.extend(eyes(e, x + 16.0, y + 6.0, 9.0, 6.0, 0.0));
    let hull = oval(e, x, y + 12.0, 48.0, 18.0, Color::rgb(0xa0, 0xa8, 0xc0));
    parts.push(shaded(e, hull, Color::rgb(0xe0, 0xe4, 0xf0), Color::rgb(0x60, 0x68, 0x88)));
    for place in 0..4 {
        parts.push(oval(e, x + 7.0 + place as f32 * 10.0, y + 19.0, 5.0, 5.0, YELLOW));
    }
    grouped(e, name, parts)
}

pub fn space_shooter() -> Document {
    let (score, lives, ship_name, bullet, alien, blast) = (t("Score"), t("Lives"), t("Ship"), t("Bullet"), t("Alien"), t("Explosion"));
    let (width, height) = (640.0, 400.0);
    let mut e = start(640, 400, t("Background"), 30.0);
    let sfx = sounds(&mut e);
    sky(&mut e, width, height, NIGHT, Color::rgb(0x24, 0x1c, 0x50));
    // A planet far away, and stars that stream past at three speeds: the nearer, the faster.
    let planet = oval(&mut e, 470.0, 40.0, 110.0, 110.0, Color::rgb(0x8c, 0x5c, 0xc8));
    rounded(&mut e, planet, Color::rgb(0xd0, 0xa8, 0xf8), Color::rgb(0x40, 0x24, 0x78));
    for index in 0..36u32 {
        // Scattered by a rule of thumb, so that the stars are the same each time the example is opened.
        let (x, y) = ((index * 97 % 64) as f32 * 10.0 + 3.0, (index * 53 % 40) as f32 * 10.0 + 2.0);
        let near = index % 3;
        let size = 1.5 + near as f32;
        let id = oval(&mut e, x, y, size, size, WHITE);
        faint(&mut e, id, 0.5 + near as f32 * 0.25);
        give(&mut e, id, &[(Kind::Drift, 14.0 + near as f32 * 22.0, 270.0)]);
    }
    centered(&mut e, t("Arrows move the ship, Space fires"), 320.0, 40.0, 13.0, PALE, TextStyle::default());
    counter(&mut e, score, 14.0, 8.0, 20.0, WHITE);
    counter(&mut e, lives, 490.0, 8.0, 20.0, YELLOW);
    layer(&mut e, t("Game"));
    // The saucers come down one after another; each starts higher than the one before.
    let domes = [GREEN, Color::rgb(0x40, 0xc8, 0xe8), Color::rgb(0xf0, 0x70, 0xb0), ORANGE, GREEN];
    for (place, x) in [40.0, 170.0, 300.0, 430.0, 560.0].into_iter().enumerate() {
        let id = saucer(&mut e, alien, x, 50.0 - place as f32 * 90.0, domes[place]);
        give(&mut e, id, &[(Kind::Gravity, 18.0, 0.0)]);
    }
    // One bullet and one explosion are drawn, and hidden when the film starts: every shot and every blast is a copy.
    let id = block(&mut e, 317.0, 320.0, 6.0, 18.0, 3.0, YELLOW);
    e.for_selected(|o| o.name = bullet.to_string());
    glowing(&mut e, id, ORANGE);
    give(&mut e, id, &[(Kind::Drift, 460.0, 90.0)]);
    burst(&mut e, blast, 300.0, 180.0, 56.0, ORANGE, YELLOW);
    // Falling keeps the ship on the bottom edge, so the arrows only move it sideways.
    let id = ship(&mut e, ship_name, 296.0, 352.0);
    give(&mut e, id, &[(Kind::ArrowKeys, 320.0, 0.0), (Kind::Gravity, 900.0, 0.0)]);
    stop_here(&mut e);
    last_scene(&mut e, width, height, t("Game over"), &format!("{score}: {{{score}}}"));
    e.set_active_page(0);
    e.doc.variables = vec![Variable { name: score.to_string(), start: 0.0 }, Variable { name: lives.to_string(), start: 3.0 }];
    e.doc.rules = vec![
        rule((EventKind::Start, "", "", 0.0), &[(StepKind::Hide, bullet, 0.0, 0.0), (StepKind::Hide, blast, 0.0, 0.0)]),
        placed(rule((EventKind::Key, " ", "", 0.0), &[(StepKind::Spawn, bullet, 0.0, 0.0), sound(sfx.zap)]), &[ship_name]),
        rule((EventKind::Edge, bullet, "top", 0.0), &[(StepKind::Remove, bullet, 0.0, 0.0)]),
        placed(
            rule((EventKind::Touch, bullet, alien, 0.0), &[(StepKind::Add, score, 1.0, 0.0), (StepKind::Spawn, blast, 0.0, 0.0), (StepKind::Remove, bullet, 0.0, 0.0), (StepKind::RandomTop, alien, 0.0, 0.0), (StepKind::Faster, alien, 6.0, 0.0), sound(sfx.boom)]),
            &[alien],
        ),
        rule((EventKind::Edge, alien, "bottom", 0.0), &[(StepKind::Add, lives, -1.0, 0.0), (StepKind::RandomTop, alien, 0.0, 0.0), sound(sfx.oops)]),
        rule((EventKind::Number, lives, "<", 1.0), &[(StepKind::GotoScene, "", 2.0, 0.0)]),
        rule((EventKind::Click, t("Again"), "", 0.0), &[(StepKind::Restart, "", 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}

// ---- Catch the apples -----------------------------------------------------------------------

fn apple(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let stem = block(e, x + 15.0, y, 4.0, 9.0, 2.0, DARK_BROWN);
    let leaf = oval(e, x + 18.0, y, 14.0, 7.0, GREEN);
    let fruit = oval(e, x, y + 6.0, 34.0, 32.0, RED);
    let shine = oval(e, x + 6.0, y + 11.0, 9.0, 7.0, WHITE);
    let parts = vec![stem, leaf, rounded(e, fruit, Color::rgb(0xff, 0x70, 0x60), Color::rgb(0xa0, 0x10, 0x20)), faint(e, shine, 0.6)];
    grouped(e, name, parts)
}

fn basket(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let body = poly(e, &[(x, y + 8.0), (x + 100.0, y + 8.0), (x + 88.0, y + 46.0), (x + 12.0, y + 46.0)], BROWN);
    let mut parts = vec![shaded(e, body, Color::rgb(0xd0, 0x98, 0x50), DARK_BROWN)];
    for place in 0..5 {
        parts.push(block(e, x + 14.0 + place as f32 * 17.0, y + 12.0, 4.0, 32.0, 2.0, DARK_BROWN));
    }
    parts.push(block(e, x + 6.0, y + 24.0, 88.0, 4.0, 2.0, DARK_BROWN));
    parts.push(block(e, x - 4.0, y, 108.0, 12.0, 6.0, Color::rgb(0xd8, 0xa0, 0x58)));
    grouped(e, name, parts)
}

fn tree(e: &mut Editor, x: f32, ground: f32) {
    let trunk = block(e, x + 62.0, ground - 150.0, 30.0, 160.0, 6.0, BROWN);
    shaded(e, trunk, BROWN, DARK_BROWN);
    for (dx, dy, size, color) in [(0.0, -250.0, 110.0, DARK_GREEN), (50.0, -280.0, 120.0, GREEN), (20.0, -200.0, 130.0, GREEN), (-10.0, -220.0, 90.0, DARK_GREEN)] {
        oval(e, x + dx, ground + dy, size, size * 0.85, color);
    }
    for (dx, dy) in [(30.0, -220.0), (90.0, -250.0), (110.0, -180.0), (50.0, -160.0)] {
        oval(e, x + dx, ground + dy, 12.0, 12.0, RED);
    }
}

pub fn catch_the_apples() -> Document {
    let (score, lives, apple_name, basket_name, plus) = (t("Score"), t("Lives"), t("Apple"), t("Basket"), t("Plus"));
    let (width, height, ground) = (640.0, 400.0, 356.0);
    let mut e = start(640, 400, t("Background"), 30.0);
    let sfx = sounds(&mut e);
    sky(&mut e, width, height, Color::rgb(0x8c, 0xcc, 0xf8), Color::rgb(0xf0, 0xf8, 0xff));
    sun(&mut e, 30.0, 20.0, 80.0);
    hills(&mut e, width, ground);
    tree(&mut e, 470.0, ground);
    cloud(&mut e, 200.0, 40.0, 90.0, 12.0);
    cloud(&mut e, 420.0, 80.0, 60.0, 7.0);
    let grass = block(&mut e, -10.0, ground, width + 20.0, 60.0, 0.0, GREEN);
    shaded(&mut e, grass, Color::rgb(0x7c, 0xdc, 0x64), DARK_GREEN);
    centered(&mut e, t("Catch the apples with the arrow keys"), 320.0, 40.0, 13.0, INK, TextStyle::default());
    counter(&mut e, score, 14.0, 8.0, 20.0, INK);
    counter(&mut e, lives, 490.0, 8.0, 20.0, RED);
    layer(&mut e, t("Game"));
    // The apples fall by themselves; each starts higher up than the one before, so they come one after another.
    for (x, y) in [(110.0, 2.0), (330.0, -170.0), (520.0, -340.0)] {
        let id = apple(&mut e, apple_name, x, y);
        give(&mut e, id, &[(Kind::Gravity, 60.0, 0.0)]);
    }
    // A "+1" that rises and fades where the basket is: one is drawn and hidden, each catch shows a copy.
    let id = centered(&mut e, "+1", 320.0, 200.0, 26.0, ORANGE, bold());
    e.selection = vec![id];
    e.for_selected(|o| o.name = plus.to_string());
    give(&mut e, id, &[(Kind::UpAndDown, 60.0, 1.2), (Kind::Fade, 1.2, 0.0), (Kind::Expire, 0.6, 0.0)]);
    // The basket walks with the arrow keys. Falling keeps it on the ground, so the up arrow does not lift it.
    let id = basket(&mut e, basket_name, 270.0, 354.0);
    give(&mut e, id, &[(Kind::ArrowKeys, 300.0, 0.0), (Kind::Gravity, 900.0, 0.0)]);
    stop_here(&mut e);
    last_scene(&mut e, width, height, t("Game over"), &format!("{score}: {{{score}}}"));
    e.set_active_page(0);
    e.doc.variables = vec![Variable { name: score.to_string(), start: 0.0 }, Variable { name: lives.to_string(), start: 5.0 }];
    e.doc.rules = vec![
        rule((EventKind::Start, "", "", 0.0), &[(StepKind::Hide, plus, 0.0, 0.0)]),
        // Each apple caught falls a little faster the next time: the game gets harder as it goes.
        placed(
            rule((EventKind::Touch, apple_name, basket_name, 0.0), &[(StepKind::Add, score, 1.0, 0.0), (StepKind::Spawn, plus, 0.0, 0.0), (StepKind::RandomTop, apple_name, 0.0, 0.0), (StepKind::Faster, apple_name, 4.0, 0.0), sound(sfx.ding)]),
            &[basket_name],
        ),
        rule((EventKind::Edge, apple_name, "bottom", 0.0), &[(StepKind::Add, lives, -1.0, 0.0), (StepKind::RandomTop, apple_name, 0.0, 0.0), sound(sfx.oops)]),
        rule((EventKind::Number, lives, "<", 1.0), &[(StepKind::GotoScene, "", 2.0, 0.0)]),
        rule((EventKind::Click, t("Again"), "", 0.0), &[(StepKind::Restart, "", 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}

// ---- Hit the targets ------------------------------------------------------------------------

fn target(e: &mut Editor, name: &str, x: f32, y: f32, size: f32) -> u64 {
    let mut parts = Vec::new();
    for (ring, color) in [RED, WHITE, RED, WHITE, RED].into_iter().enumerate() {
        let inset = ring as f32 * size / 10.0;
        parts.push(oval(e, x + inset, y + inset, size - inset * 2.0, size - inset * 2.0, color));
    }
    let id = grouped(e, name, parts);
    outlined(e, id, INK, 0.0);
    id
}

pub fn hit_the_targets() -> Document {
    let (score, time, target_name, pop) = (t("Score"), t("Time"), t("Target"), t("Pop"));
    let (width, height) = (640.0, 400.0);
    let mut e = start(640, 400, t("Background"), 30.0);
    let sfx = sounds(&mut e);
    sky(&mut e, width, height, Color::rgb(0x2c, 0x1c, 0x58), Color::rgb(0xd0, 0x58, 0x78));
    // A fairground at dusk: a striped tent, and lights that blink along its edge.
    for stripe in 0..8 {
        let x = stripe as f32 * 80.0;
        poly(&mut e, &[(x, 400.0), (x + 80.0, 400.0), (x + 60.0, 310.0), (x + 20.0, 310.0)], if stripe % 2 == 0 { RED } else { PALE });
    }
    block(&mut e, 0.0, 300.0, width, 14.0, 0.0, DARK_BROWN);
    for lamp in 0..16 {
        let id = oval(&mut e, 14.0 + lamp as f32 * 40.0, 302.0, 10.0, 10.0, YELLOW);
        glowing(&mut e, id, YELLOW);
        give(&mut e, id, &[(Kind::Blink, 0.6 + (lamp % 3) as f32 * 0.3, 0.0)]);
    }
    centered(&mut e, t("Click the targets before the time runs out"), 320.0, 40.0, 13.0, PALE, TextStyle::default());
    counter(&mut e, score, 14.0, 8.0, 20.0, WHITE);
    counter(&mut e, time, 490.0, 8.0, 20.0, YELLOW);
    layer(&mut e, t("Game"));
    // Four targets, each moving its own way.
    for (x, y, size, moves) in [
        (60.0, 80.0, 64.0, (Kind::BackAndForth, 180.0, 2.2)),
        (420.0, 150.0, 56.0, (Kind::UpAndDown, 80.0, 1.6)),
        (220.0, 220.0, 50.0, (Kind::Drift, 130.0, 0.0)),
        (520.0, 70.0, 44.0, (Kind::Drift, 90.0, 200.0)),
    ] {
        let id = target(&mut e, target_name, x, y, size);
        give(&mut e, id, &[moves]);
    }
    burst(&mut e, pop, 300.0, 180.0, 60.0, WHITE, YELLOW);
    stop_here(&mut e);
    last_scene(&mut e, width, height, t("Time is up"), &format!("{score}: {{{score}}}"));
    e.set_active_page(0);
    e.doc.variables = vec![Variable { name: score.to_string(), start: 0.0 }, Variable { name: time.to_string(), start: 20.0 }];
    e.doc.rules = vec![
        rule((EventKind::Start, "", "", 0.0), &[(StepKind::Hide, pop, 0.0, 0.0)]),
        placed(
            rule((EventKind::Click, target_name, "", 0.0), &[(StepKind::Add, score, 1.0, 0.0), (StepKind::Spawn, pop, 0.0, 0.0), (StepKind::Random, target_name, 0.0, 0.0), (StepKind::Faster, target_name, 8.0, 0.0), sound(sfx.boom)]),
            &[target_name],
        ),
        rule((EventKind::Every, "", "", 1.0), &[(StepKind::Add, time, -1.0, 0.0)]),
        rule((EventKind::Number, time, "<", 1.0), &[sound(sfx.win), (StepKind::GotoScene, "", 2.0, 0.0)]),
        rule((EventKind::Click, t("Again"), "", 0.0), &[(StepKind::Restart, "", 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}

// ---- The quiz -------------------------------------------------------------------------------

pub fn quiz() -> Document {
    let (score, right, wrong) = (t("Score"), t("Right answer"), t("Wrong answer"));
    let questions = [
        (t("How many legs does a spider have?"), ["6", "8", "10"], 1),
        (t("How many days are there in a week?"), ["5", "9", "7"], 2),
        (t("How many sides does a triangle have?"), ["3", "4", "5"], 0),
    ];
    let cards = [Color::rgb(0xf0, 0x60, 0x50), Color::rgb(0x40, 0xa8, 0xe8), Color::rgb(0x58, 0xc0, 0x60)];
    let (width, height) = (640.0, 400.0);
    let mut e = start(640, 400, t("Background"), 30.0);
    let sfx = sounds(&mut e);
    for (index, (question, answers, correct)) in questions.iter().enumerate() {
        if index > 0 {
            e.add_page();
            e.doc.page_mut().layers[0].name = t("Background").to_string();
        }
        e.doc.page_mut().name = tf("Question {}", &[&(index + 1)]);
        sky(&mut e, width, height, Color::rgb(0x34, 0x2c, 0x78), Color::rgb(0x18, 0x90, 0xa8));
        // Bubbles rise behind the question, slowly.
        for bubble in 0..9u32 {
            let size = 20.0 + (bubble * 37 % 50) as f32;
            let id = oval(&mut e, (bubble * 83 % 600) as f32, (bubble * 131 % 360) as f32, size, size, WHITE);
            faint(&mut e, id, 0.12);
            give(&mut e, id, &[(Kind::Drift, 12.0 + (bubble % 4) as f32 * 8.0, 90.0)]);
        }
        counter(&mut e, score, 14.0, 8.0, 18.0, WHITE);
        let badge = block(&mut e, 250.0, 46.0, 140.0, 32.0, 16.0, YELLOW);
        shaded(&mut e, badge, YELLOW, ORANGE);
        centered(&mut e, &tf("Question {}", &[&(index + 1)]), 320.0, 52.0, 17.0, INK, bold());
        centered(&mut e, question, 320.0, 112.0, 28.0, WHITE, bold());
        // Each answer is a card with its words on it. The cards are named Right answer or Wrong answer: that is all the rules need.
        for (place, answer) in answers.iter().enumerate() {
            let x = 95.0 + place as f32 * 160.0;
            let shadow = block(&mut e, x + 4.0, 226.0, 130.0, 90.0, 16.0, INK);
            faint(&mut e, shadow, 0.35);
            let id = block(&mut e, x, 220.0, 130.0, 90.0, 16.0, cards[place]);
            e.for_selected(|o| o.name = if place == *correct { right } else { wrong }.to_string());
            shaded(&mut e, id, cards[place], INK);
            gradient(&mut e, id, FillKind::Linear, cards[place], Color::rgb(cards[place].r / 2, cards[place].g / 2, cards[place].b / 2), (0.5, 0.0), (0.5, 1.4));
            give(&mut e, id, &[(Kind::UpAndDown, 6.0, 1.6 + place as f32 * 0.3)]);
            centered(&mut e, answer, x + 65.0, 240.0, 40.0, WHITE, bold());
        }
        stop_here(&mut e);
    }
    last_scene(&mut e, width, height, t("Well done"), &format!("{score}: {{{score}}} / {}", questions.len()));
    e.set_active_page(0);
    e.doc.variables = vec![Variable { name: score.to_string(), start: 0.0 }];
    e.doc.rules = vec![
        rule((EventKind::Click, right, "", 0.0), &[(StepKind::Add, score, 1.0, 0.0), sound(sfx.ding), (StepKind::NextScene, "", 0.0, 0.0)]),
        rule((EventKind::Click, wrong, "", 0.0), &[sound(sfx.oops), (StepKind::NextScene, "", 0.0, 0.0)]),
        rule((EventKind::Click, t("Again"), "", 0.0), &[(StepKind::Restart, "", 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sound_made_here_is_a_wav_file_of_the_right_length() {
        // A tenth of a second at 11025 samples a second, one byte each, after a header of 44 bytes.
        let file = wav(&[(440.0, 100)]);
        assert_eq!((&file[0..4], &file[8..12], file.len()), (&b"RIFF"[..], &b"WAVE"[..], 44 + 1102));
        assert_eq!(u32::from_le_bytes([file[40], file[41], file[42], file[43]]), 1102);
        // It starts loud and dies away.
        let loud = |range: std::ops::Range<usize>| file[range].iter().map(|s| (*s as i32 - 128).abs()).max().unwrap();
        assert!(loud(44..300) > 40 && loud(1000..1146) < 12);
    }
}

// ---- Driving a car --------------------------------------------------------------------------

/// A wheel whose turning shows: a tyre, a hub and spokes.
fn wheel(e: &mut Editor, name: &str, x: f32, y: f32, size: f32) -> u64 {
    let tyre = oval(e, x, y, size, size, Color::rgb(0x26, 0x28, 0x30));
    let rim = oval(e, x + size * 0.2, y + size * 0.2, size * 0.6, size * 0.6, Color::rgb(0xd0, 0xd4, 0xdc));
    let across = block(e, x + size * 0.2, y + size * 0.45, size * 0.6, size * 0.1, 0.0, Color::rgb(0x60, 0x64, 0x70));
    let down = block(e, x + size * 0.45, y + size * 0.2, size * 0.1, size * 0.6, 0.0, Color::rgb(0x60, 0x64, 0x70));
    let hub = oval(e, x + size * 0.38, y + size * 0.38, size * 0.24, size * 0.24, Color::rgb(0x26, 0x28, 0x30));
    grouped(e, name, vec![tyre, rim, across, down, hub])
}

/// A car to drive: the keys set it going, letting go lets it slow down, and its wheels turn as far as it goes.
pub fn driving() -> Document {
    let (car_name, wheel_name) = (t("Car"), t("Wheel"));
    let (width, height, road) = (720.0, 360.0, 290.0);
    let mut e = start(720, 360, t("Background"), 30.0);
    sky(&mut e, width, height, Color::rgb(0x8c, 0xcc, 0xf8), Color::rgb(0xf0, 0xf8, 0xff));
    // The rays turn, so they are an object the view would carry off: sliding by nothing keeps them on the screen.
    let rays = sun(&mut e, 600.0, 20.0, 80.0);
    e.selection = vec![rays];
    e.for_selected(|o| o.behaviors.push(crate::behavior::Behavior { kind: Kind::Scenery, a: 0.0, b: 0.0 }));
    // The sky and the sun stay where they are. What is far slides a little as the car goes, what is near
    // slides more, and each comes back in at the other side: the road never ends.
    let far = |e: &mut Editor, percent: f32| e.for_selected(|o| o.behaviors.push(crate::behavior::Behavior { kind: Kind::Scenery, a: percent, b: 1.0 }));
    for (x, w, h, color, percent) in [(-80.0, 380.0, 190.0, Color::rgb(0x9c, 0xd4, 0x88), 20.0), (width - 360.0, 460.0, 240.0, Color::rgb(0x84, 0xc8, 0x78), 30.0), (160.0, 360.0, 130.0, Color::rgb(0x70, 0xbc, 0x68), 45.0)] {
        let id = oval(&mut e, x, road - h / 2.0, w, h, color);
        e.selection = vec![id];
        e.for_selected(|o| o.name = t("Hill").to_string());
        far(&mut e, percent);
    }
    cloud(&mut e, 120.0, 40.0, 90.0, 10.0);
    far(&mut e, 8.0);
    cloud(&mut e, 380.0, 70.0, 60.0, 6.0);
    far(&mut e, 5.0);
    let tarmac = block(&mut e, -10.0, road, width + 20.0, 80.0, 0.0, Color::rgb(0x50, 0x54, 0x60));
    shaded(&mut e, tarmac, Color::rgb(0x64, 0x68, 0x74), Color::rgb(0x3c, 0x40, 0x4c));
    // Five marks, a stage and one mark apart from the first to the first again, so that they stay evenly spaced as they come round.
    for x in [0.0, 160.0, 320.0, 480.0, 640.0] {
        let id = block(&mut e, x, road + 34.0, 80.0, 6.0, 3.0, WHITE);
        e.selection = vec![id];
        e.for_selected(|o| o.name = t("Road mark").to_string());
        far(&mut e, 100.0);
    }
    // Bushes by the road, as near as the road is.
    for x in [90.0, 420.0, 610.0] {
        let id = oval(&mut e, x, road - 22.0, 70.0, 34.0, DARK_GREEN);
        shaded(&mut e, id, GREEN, DARK_GREEN);
        e.selection = vec![id];
        e.for_selected(|o| o.name = t("Bush").to_string());
        far(&mut e, 100.0);
    }
    centered(&mut e, t("Hold Right or Left to drive. Let go, and the car slows down by itself."), 360.0, 14.0, 13.0, INK, TextStyle::default());
    layer(&mut e, t("Game"));
    // The car is one object, its wheels two more: a wheel inside the car's group could not turn.
    let (x, y) = (250.0, road - 84.0);
    let body = block(&mut e, x, y + 30.0, 190.0, 40.0, 14.0, RED);
    shaded(&mut e, body, Color::rgb(0xf0, 0x50, 0x50), Color::rgb(0xa8, 0x20, 0x28));
    let cabin = block(&mut e, x + 40.0, y, 100.0, 40.0, 16.0, RED);
    let front = block(&mut e, x + 94.0, y + 8.0, 38.0, 24.0, 6.0, Color::rgb(0xc8, 0xe8, 0xff));
    let back = block(&mut e, x + 50.0, y + 8.0, 38.0, 24.0, 6.0, Color::rgb(0xc8, 0xe8, 0xff));
    let lamp = oval(&mut e, x + 176.0, y + 40.0, 12.0, 12.0, YELLOW);
    let car = grouped(&mut e, car_name, vec![cabin, body, front, back, lamp]);
    // The view goes along with the car: it stays where it is on the screen, and the road slides under it.
    give(&mut e, car, &[(Kind::Camera, 0.0, 0.0)]);
    for at in [x + 22.0, x + 124.0] {
        let id = wheel(&mut e, wheel_name, at, road - 40.0, 44.0);
        give(&mut e, id, &[(Kind::Roll, 0.0, 0.0)]);
        // Behaviors > Attach to Another Object: the wheel goes wherever the car goes.
        e.for_selected(|o| o.attached = car_name.to_string());
    }
    let with = |mut made: Rule, b: &str| {
        made.steps[0].b = b.to_string();
        made
    };
    e.doc.rules = vec![
        with(rule((EventKind::Key, "ArrowRight", "", 0.0), &[(StepKind::Go, car_name, 320.0, 0.0)]), "right"),
        rule((EventKind::KeyUp, "ArrowRight", "", 0.0), &[(StepKind::Slow, car_name, 0.0, 0.0)]),
        with(rule((EventKind::Key, "ArrowLeft", "", 0.0), &[(StepKind::Go, car_name, 320.0, 0.0)]), "left"),
        rule((EventKind::KeyUp, "ArrowLeft", "", 0.0), &[(StepKind::Slow, car_name, 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}
