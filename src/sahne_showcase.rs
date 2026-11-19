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

//! Pisi: Iron Claw. The game PisiAnimasyon shows what it can do with.
//!
//! A run-and-gun of our own: Pisi, a robot leopard with a blaster, runs through a city, a
//! factory and a rooftop while robot rats roll in and shoot back, drones drop bombs, and a giant
//! rat waits at the end. It is longer than the other examples, and still nothing but behaviors
//! and rules: every sentence of it can be read in Rules and Numbers.
//!
//! There is no camera in PisiAnimasyon, so the world comes to Pisi: the scenery is strips twice the
//! width of the stage that keep going left, the far ones slowly and the near ones fast, and the
//! enemies are copies made at a gate that stands just off the right edge.

use crate::autoshape::AutoKind;
use crate::behavior::BehaviorKind as Kind;
use crate::examples::{add, bold, centered, layer, BLUE, INK, ORANGE, PALE, WHITE, YELLOW};
use crate::i18n::{t, tf};
use crate::model::{Color, Document, Editor, Rect, Shape, SymbolKind, TextStyle, Vec2, Warp};
use crate::rules::{EventKind, Rule, StepKind, Variable};
use crate::sahne_examples::{counter, give, last_scene, rule, start, stop_here};
use crate::sahne_games::{block, burst, faint, glowing, grouped, oval, placed, poly, rounded, shaded, sky, sound, sounds, star, RED};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 400.0;
/// Where the ground begins.
const GROUND: f32 = 340.0;
/// How fast the world goes by at ground level, in pixels a second.
const PACE: f32 = 150.0;

const GOLD: Color = Color::rgb(0xff, 0xc8, 0x40);
const DEEP_GOLD: Color = Color::rgb(0xc8, 0x80, 0x18);
const SPOT: Color = Color::rgb(0x5a, 0x38, 0x10);
const STEEL: Color = Color::rgb(0x8c, 0x98, 0xb0);
const DARK_STEEL: Color = Color::rgb(0x38, 0x40, 0x58);
const CYAN: Color = Color::rgb(0x40, 0xe0, 0xf8);
const PINK: Color = Color::rgb(0xf0, 0x90, 0xa8);

fn turned(e: &mut Editor, id: u64, around: (f32, f32), degrees: f32) -> u64 {
    if let Some(object) = e.object_mut(id) {
        object.warp(&Warp::rotation(Vec2 { x: around.0, y: around.1 }, degrees.to_radians()));
    }
    id
}

/// Pisi, drawn with its top left near `x`, `y`, on one of the four steps of its run.
fn pisi(e: &mut Editor, x: f32, y: f32, step: usize) -> u64 {
    // How far the near leg swings forward on each step; the far leg does the opposite.
    let swing = [34.0, 0.0, -34.0, 0.0][step % 4];
    let lift = [0.0, -2.0, 0.0, -2.0][step % 4];
    let y = y + lift;
    let mut parts = Vec::new();
    // The tail, which sways as it runs.
    let sway = [0.0, 3.0, 0.0, -3.0][step % 4];
    parts.push(poly(e, &[(x + 12.0, y + 46.0), (x + 2.0, y + 42.0), (x - 6.0, y + 28.0 + sway), (x - 2.0, y + 24.0 + sway), (x + 6.0, y + 36.0), (x + 14.0, y + 40.0)], GOLD));
    parts.push(oval(e, x - 9.0, y + 19.0 + sway, 9.0, 9.0, SPOT));
    // The far leg, behind the body.
    for (hip, angle, shade) in [(x + 17.0, -swing, DARK_STEEL), (x + 29.0, swing, STEEL)] {
        let leg = block(e, hip - 5.0, y + 44.0, 10.0, 18.0, 4.0, shade);
        let boot = block(e, hip - 6.0, y + 58.0, 15.0, 7.0, 3.0, INK);
        parts.push(turned(e, leg, (hip, y + 46.0), -angle));
        parts.push(turned(e, boot, (hip, y + 46.0), -angle));
        if shade == DARK_STEEL {
            // The body goes between the two legs.
            let body = block(e, x + 10.0, y + 24.0, 27.0, 26.0, 10.0, GOLD);
            parts.push(shaded(e, body, GOLD, DEEP_GOLD));
            for (sx, sy, size) in [(14.0, 30.0, 6.0), (27.0, 28.0, 5.0), (17.0, 40.0, 5.0), (29.0, 40.0, 4.0)] {
                parts.push(oval(e, x + sx, y + sy, size, size, SPOT));
            }
            let light = oval(e, x + 20.0, y + 33.0, 8.0, 8.0, CYAN);
            parts.push(glowing(e, light, CYAN));
        }
    }
    // The head: ears, a visor for eyes, a muzzle.
    parts.push(poly(e, &[(x + 9.0, y + 8.0), (x + 11.0, y - 5.0), (x + 20.0, y + 3.0)], DEEP_GOLD));
    parts.push(poly(e, &[(x + 26.0, y + 3.0), (x + 35.0, y - 5.0), (x + 38.0, y + 8.0)], DEEP_GOLD));
    parts.push(poly(e, &[(x + 12.0, y + 5.0), (x + 13.0, y - 1.0), (x + 17.0, y + 3.0)], PINK));
    parts.push(poly(e, &[(x + 29.0, y + 3.0), (x + 33.0, y - 1.0), (x + 35.0, y + 5.0)], PINK));
    let head = block(e, x + 7.0, y + 2.0, 33.0, 25.0, 11.0, GOLD);
    parts.push(shaded(e, head, Color::rgb(0xff, 0xe0, 0x80), GOLD));
    parts.push(oval(e, x + 11.0, y + 5.0, 5.0, 4.0, SPOT));
    parts.push(oval(e, x + 18.0, y + 3.0, 4.0, 3.0, SPOT));
    let visor = block(e, x + 20.0, y + 10.0, 19.0, 7.0, 3.0, CYAN);
    parts.push(glowing(e, visor, CYAN));
    parts.push(oval(e, x + 28.0, y + 16.0, 14.0, 9.0, WHITE));
    parts.push(oval(e, x + 38.0, y + 17.0, 4.0, 4.0, INK));
    // The blaster, held out in front.
    parts.push(oval(e, x + 20.0, y + 28.0, 11.0, 11.0, DEEP_GOLD));
    let barrel = block(e, x + 24.0, y + 30.0, 30.0, 8.0, 3.0, STEEL);
    parts.push(shaded(e, barrel, PALE, DARK_STEEL));
    parts.push(block(e, x + 50.0, y + 28.0, 7.0, 12.0, 2.0, INK));
    parts.push(block(e, x + 32.0, y + 37.0, 6.0, 7.0, 2.0, DARK_STEEL));
    grouped(e, "Pisi", parts)
}

/// Pisi as a movie clip that runs in place: four steps, three frames each. Returns the symbol.
fn pisi_clip(e: &mut Editor) -> Option<u64> {
    let first = pisi(e, 100.0, 100.0, 0);
    let drawn = e.object_mut(first)?.bounds;
    e.selection = vec![first];
    let symbol = e.make_symbol()?;
    let copy = e.selection.first().copied();
    if let Some(entry) = e.doc.symbols.iter_mut().find(|s| s.id == symbol) {
        (entry.kind, entry.name) = (SymbolKind::Animation, "Pisi".to_string());
    }
    e.open_clip(symbol);
    // Inside the clip the first step lies where the clip's own page has put it; the others are drawn against it.
    let placed = e.doc.page().objects().map(|o| o.bounds).reduce(Rect::union)?;
    let (x, y) = (placed.x + 100.0 - drawn.x, placed.y + 100.0 - drawn.y);
    for step in 1..4 {
        e.goto_frame(step * 3);
        e.on_timeline(|timeline, layer, frame| timeline.insert_keyframe(layer, frame, true));
        pisi(e, x, y, step);
    }
    e.on_timeline(|timeline, layer, frame| timeline.insert_frames(layer, frame, 2));
    e.goto_frame(0);
    e.leave_clip();
    // The copy left on the stage is taken away: each scene places its own.
    e.selection = copy.into_iter().collect();
    e.delete_selected();
    Some(symbol)
}

/// A robot rat on wheels, with a gun, looking left.
fn rat(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let mut parts = vec![poly(e, &[(x + 38.0, y + 20.0), (x + 52.0, y + 14.0), (x + 54.0, y + 16.0), (x + 40.0, y + 24.0)], PINK)];
    let body = oval(e, x + 8.0, y + 8.0, 34.0, 22.0, STEEL);
    parts.push(shaded(e, body, PALE, DARK_STEEL));
    parts.push(oval(e, x + 12.0, y - 2.0, 12.0, 12.0, PINK));
    let head = oval(e, x, y + 6.0, 22.0, 18.0, STEEL);
    parts.push(shaded(e, head, PALE, DARK_STEEL));
    let eye = oval(e, x + 5.0, y + 11.0, 6.0, 6.0, RED);
    parts.push(glowing(e, eye, RED));
    parts.push(oval(e, x - 2.0, y + 15.0, 5.0, 5.0, INK));
    parts.push(block(e, x - 10.0, y + 20.0, 18.0, 5.0, 2.0, INK));
    for wheel in [x + 10.0, x + 28.0] {
        parts.push(oval(e, wheel, y + 24.0, 12.0, 12.0, INK));
        parts.push(oval(e, wheel + 4.0, y + 28.0, 4.0, 4.0, STEEL));
    }
    grouped(e, name, parts)
}

/// A drone with one red eye.
fn drone(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let mut parts = vec![block(e, x, y, 46.0, 4.0, 2.0, PALE), block(e, x + 21.0, y + 2.0, 4.0, 8.0, 0.0, DARK_STEEL)];
    let body = oval(e, x + 7.0, y + 8.0, 32.0, 20.0, BLUE);
    parts.push(rounded(e, body, Color::rgb(0x70, 0xb0, 0xf0), Color::rgb(0x18, 0x30, 0x70)));
    let eye = oval(e, x + 10.0, y + 13.0, 9.0, 9.0, RED);
    parts.push(glowing(e, eye, RED));
    parts.push(block(e, x + 14.0, y + 26.0, 4.0, 7.0, 1.0, DARK_STEEL));
    parts.push(block(e, x + 28.0, y + 26.0, 4.0, 7.0, 1.0, DARK_STEEL));
    grouped(e, name, parts)
}

fn heart(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let parts = vec![oval(e, x, y, 14.0, 14.0, RED), oval(e, x + 10.0, y, 14.0, 14.0, RED), poly(e, &[(x + 1.0, y + 9.0), (x + 23.0, y + 9.0), (x + 12.0, y + 24.0)], RED), oval(e, x + 4.0, y + 3.0, 5.0, 4.0, WHITE)];
    let id = grouped(e, name, parts);
    glowing(e, id, PINK)
}

fn turret(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let base = poly(e, &[(x + 6.0, y + 28.0), (x + 50.0, y + 28.0), (x + 56.0, y + 52.0), (x, y + 52.0)], DARK_STEEL);
    let dome = oval(e, x + 12.0, y + 6.0, 34.0, 34.0, STEEL);
    let eye = oval(e, x + 18.0, y + 16.0, 9.0, 9.0, RED);
    let parts = vec![block(e, x - 22.0, y + 18.0, 38.0, 9.0, 3.0, INK), base, shaded(e, dome, PALE, DARK_STEEL), glowing(e, eye, RED), block(e, x + 8.0, y + 44.0, 40.0, 4.0, 0.0, YELLOW)];
    grouped(e, name, parts)
}

/// The giant rat: a machine as tall as half the stage.
fn giant(e: &mut Editor, name: &str, x: f32, y: f32) -> u64 {
    let mut parts = vec![block(e, x + 10.0, y + 150.0, 150.0, 30.0, 15.0, INK)];
    for wheel in 0..5 {
        parts.push(oval(e, x + 16.0 + wheel as f32 * 28.0, y + 155.0, 20.0, 20.0, STEEL));
    }
    let body = block(e, x + 20.0, y + 60.0, 130.0, 100.0, 26.0, Color::rgb(0x90, 0x38, 0x40));
    parts.push(shaded(e, body, Color::rgb(0xc8, 0x58, 0x58), Color::rgb(0x50, 0x18, 0x28)));
    for stripe in 0..5 {
        parts.push(poly(e, &[(x + 30.0 + stripe as f32 * 24.0, y + 150.0), (x + 42.0 + stripe as f32 * 24.0, y + 150.0), (x + 34.0 + stripe as f32 * 24.0, y + 138.0), (x + 22.0 + stripe as f32 * 24.0, y + 138.0)], YELLOW));
    }
    parts.push(oval(e, x + 70.0, y - 4.0, 44.0, 44.0, PINK));
    parts.push(oval(e, x + 108.0, y + 4.0, 40.0, 40.0, PINK));
    let head = oval(e, x, y + 20.0, 120.0, 86.0, STEEL);
    parts.push(shaded(e, head, PALE, DARK_STEEL));
    for (ex, ey) in [(22.0, 44.0), (58.0, 40.0)] {
        let eye = oval(e, x + ex, y + ey, 22.0, 18.0, RED);
        parts.push(glowing(e, eye, RED));
        parts.push(oval(e, x + ex + 4.0, y + ey + 5.0, 8.0, 8.0, YELLOW));
    }
    parts.push(oval(e, x - 8.0, y + 62.0, 18.0, 16.0, INK));
    for tooth in 0..4 {
        parts.push(poly(e, &[(x + 16.0 + tooth as f32 * 14.0, y + 88.0), (x + 28.0 + tooth as f32 * 14.0, y + 88.0), (x + 22.0 + tooth as f32 * 14.0, y + 100.0)], WHITE));
    }
    let barrel = block(e, x - 46.0, y + 112.0, 86.0, 18.0, 6.0, DARK_STEEL);
    parts.push(shaded(e, barrel, STEEL, INK));
    parts.push(block(e, x - 54.0, y + 108.0, 12.0, 26.0, 3.0, INK));
    grouped(e, name, parts)
}

/// The symbols the characters of the game are copies of. They are in the library, one of each for the whole
/// game: a rat changed there is changed in every scene.
struct Cast {
    pisi: u64,
    rat: u64,
    drone: u64,
    shot: u64,
    bomb: u64,
    heart: u64,
    bullet: u64,
    blast: u64,
    turret: u64,
    giant: u64,
}

/// Makes a symbol of what has just been drawn and takes the drawing off the stage: the scenes place their own copies.
fn symbol_of(e: &mut Editor, id: u64, name: &str) -> Option<u64> {
    e.selection = vec![id];
    let symbol = e.make_symbol()?;
    if let Some(entry) = e.doc.symbols.iter_mut().find(|s| s.id == symbol) {
        entry.name = name.to_string();
    }
    e.delete_selected();
    Some(symbol)
}

fn cast(e: &mut Editor) -> Option<Cast> {
    let pisi = pisi_clip(e)?;
    let id = rat(e, "", 100.0, 100.0);
    let rat = symbol_of(e, id, t("Rat"))?;
    let id = drone(e, "", 100.0, 100.0);
    let drone = symbol_of(e, id, t("Drone"))?;
    let id = oval(e, 100.0, 100.0, 14.0, 9.0, Color::rgb(0xff, 0x70, 0x50));
    glowing(e, id, RED);
    let shot = symbol_of(e, id, t("Shot"))?;
    let parts = vec![oval(e, 100.0, 108.0, 16.0, 18.0, INK), star(e, 104.0, 100.0, 10.0, 6, 0.5, ORANGE)];
    let id = grouped(e, "", parts);
    let bomb = symbol_of(e, id, t("Bomb"))?;
    let id = heart(e, "", 100.0, 100.0);
    let heart = symbol_of(e, id, t("Heart"))?;
    let id = block(e, 100.0, 100.0, 16.0, 5.0, 2.0, Color::rgb(0xc8, 0xff, 0xff));
    glowing(e, id, CYAN);
    let bullet = symbol_of(e, id, t("Bullet"))?;
    let id = burst(e, "", 100.0, 100.0, 50.0, ORANGE, YELLOW);
    let blast = symbol_of(e, id, t("Explosion"))?;
    let id = turret(e, "", 100.0, 100.0);
    let turret = symbol_of(e, id, t("Turret"))?;
    let id = giant(e, "", 100.0, 100.0);
    let giant = symbol_of(e, id, t("Giant Rat"))?;
    Some(Cast { pisi, rat, drone, shot, bomb, heart, bullet, blast, turret, giant })
}

/// A copy of a symbol with its top left corner at `x`, `y`, and a name for the rules to know it by.
fn put(e: &mut Editor, symbol: u64, name: &str, x: f32, y: f32) -> u64 {
    if !e.place_symbol_at(symbol, Vec2 { x, y }) {
        return 0;
    }
    e.for_selected(|o| {
        o.name = name.to_string();
        o.translate(x - o.bounds.x, y - o.bounds.y);
    });
    e.selection.first().copied().unwrap_or_default()
}

/// A strip of scenery as wide as the stage that keeps going left, and a second one after it, so that
/// there is never a gap: when one has left by the left edge it comes in again from the right.
fn strip(e: &mut Editor, speed: f32, draw: impl Fn(&mut Editor, f32) -> Vec<u64>) {
    for start in [0.0, WIDTH] {
        // A span of nothing as wide as the stage keeps every strip the same width, whatever is drawn on it.
        let span = block(e, start, GROUND - 2.0, WIDTH, 2.0, 0.0, WHITE);
        let mut parts = vec![faint(e, span, 0.0)];
        parts.extend(draw(e, start));
        let id = grouped(e, t("Scenery"), parts);
        give(e, id, &[(Kind::Drift, speed, 180.0)]);
    }
}

/// Houses against the sky: their widths and heights come from `seed`, so each strip is its own town.
fn skyline(e: &mut Editor, from: f32, seed: u32, tall: f32, wall: Color, lit: Option<Color>) -> Vec<u64> {
    let mut parts = Vec::new();
    let mut x = from;
    let mut turn = seed;
    while x < from + WIDTH - 20.0 {
        turn = turn.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let width = (40 + (turn >> 16) % 50) as f32;
        let width = width.min(from + WIDTH - x);
        let height = tall * (0.45 + ((turn >> 8) % 55) as f32 / 100.0);
        parts.push(block(e, x, GROUND - height, width - 4.0, height, 2.0, wall));
        if let Some(lit) = lit {
            for row in 0..((height - 16.0) / 22.0) as u32 {
                for column in 0..((width - 14.0) / 16.0) as u32 {
                    if (turn >> (row + column * 3)) & 3 != 0 {
                        parts.push(block(e, x + 8.0 + column as f32 * 16.0, GROUND - height + 10.0 + row as f32 * 22.0, 8.0, 12.0, 1.0, lit));
                    }
                }
            }
        }
        x += width;
    }
    parts
}

/// What a scene of the game is set in.
#[derive(Clone, Copy, PartialEq)]
enum Place {
    City,
    Factory,
    Roof,
    Lair,
}

/// The sky, the scenery that goes by and the ground of a place; and, on a second layer, everything the
/// rules are about. `cast` has the symbols the characters are copies of.
fn scene(e: &mut Editor, cast: &Cast, place: Place, title: &str) {
    let (score, lives) = (t("Score"), t("Lives"));
    match place {
        Place::City => {
            sky(e, WIDTH, HEIGHT, Color::rgb(0x2c, 0x20, 0x60), Color::rgb(0xf4, 0x88, 0x60));
            let moon = oval(e, 500.0, 40.0, 70.0, 70.0, PALE);
            rounded(e, moon, WHITE, Color::rgb(0xd0, 0xc0, 0xe0));
            strip(e, 12.0, |e, from| skyline(e, from, 7, 190.0, Color::rgb(0x58, 0x38, 0x78), None));
            strip(e, 34.0, |e, from| skyline(e, from, 21, 130.0, Color::rgb(0x30, 0x20, 0x50), Some(YELLOW)));
        }
        Place::Factory => {
            sky(e, WIDTH, HEIGHT, Color::rgb(0x50, 0x28, 0x20), Color::rgb(0xf8, 0xb0, 0x50));
            strip(e, 14.0, |e, from| {
                // Chimneys, each with its smoke.
                let mut parts = Vec::new();
                for (x, height) in [(60.0, 210.0), (250.0, 170.0), (470.0, 230.0)] {
                    parts.push(block(e, from + x, GROUND - height, 34.0, height, 3.0, Color::rgb(0x70, 0x40, 0x30)));
                    parts.push(block(e, from + x - 4.0, GROUND - height, 42.0, 12.0, 3.0, Color::rgb(0x48, 0x28, 0x20)));
                    for puff in 0..3 {
                        let smoke = oval(e, from + x - 6.0 + puff as f32 * 14.0, GROUND - height - 34.0 - puff as f32 * 18.0, 40.0, 28.0, PALE);
                        parts.push(faint(e, smoke, 0.35));
                    }
                }
                parts
            });
            strip(e, 40.0, |e, from| {
                // Tanks and the pipes between them.
                let mut parts = vec![block(e, from, GROUND - 62.0, WIDTH, 10.0, 0.0, Color::rgb(0x40, 0x30, 0x38))];
                for x in [30.0, 210.0, 380.0, 540.0] {
                    let tank = block(e, from + x, GROUND - 110.0, 80.0, 110.0, 18.0, Color::rgb(0x58, 0x48, 0x50));
                    parts.push(shaded(e, tank, Color::rgb(0x88, 0x70, 0x70), Color::rgb(0x30, 0x24, 0x30)));
                    parts.push(block(e, from + x + 10.0, GROUND - 80.0, 60.0, 6.0, 0.0, YELLOW));
                }
                parts
            });
        }
        Place::Roof => {
            sky(e, WIDTH, HEIGHT, Color::rgb(0x08, 0x0c, 0x28), Color::rgb(0x30, 0x40, 0x80));
            for index in 0..40u32 {
                let size = 1.5 + (index % 3) as f32;
                let dot = oval(e, (index * 97 % 64) as f32 * 10.0 + 3.0, (index * 53 % 26) as f32 * 10.0 + 4.0, size, size, WHITE);
                faint(e, dot, 0.5 + (index % 3) as f32 * 0.2);
            }
            let moon = oval(e, 80.0, 30.0, 120.0, 120.0, PALE);
            rounded(e, moon, WHITE, Color::rgb(0xb0, 0xb8, 0xd8));
            strip(e, 10.0, |e, from| skyline(e, from, 40, 110.0, Color::rgb(0x18, 0x20, 0x48), Some(Color::rgb(0xf0, 0xd0, 0x70))));
            strip(e, 60.0, |e, from| {
                // Aerials and water tanks along the roof's edge.
                let mut parts = Vec::new();
                for x in [70.0, 300.0, 520.0] {
                    parts.push(block(e, from + x, GROUND - 90.0, 4.0, 90.0, 0.0, DARK_STEEL));
                    parts.push(block(e, from + x - 14.0, GROUND - 80.0, 32.0, 3.0, 0.0, DARK_STEEL));
                    parts.push(block(e, from + x - 9.0, GROUND - 64.0, 22.0, 3.0, 0.0, DARK_STEEL));
                    let lamp = oval(e, from + x - 2.0, GROUND - 96.0, 8.0, 8.0, RED);
                    parts.push(glowing(e, lamp, RED));
                }
                parts
            });
        }
        Place::Lair => {
            sky(e, WIDTH, HEIGHT, Color::rgb(0x30, 0x08, 0x18), Color::rgb(0xc0, 0x38, 0x30));
            // Warning lights that blink along the wall.
            for lamp in 0..8 {
                let id = oval(e, 30.0 + lamp as f32 * 80.0, 70.0, 14.0, 14.0, YELLOW);
                glowing(e, id, ORANGE);
                give(e, id, &[(Kind::Blink, 0.5 + (lamp % 2) as f32 * 0.25, 0.0)]);
            }
            for pillar in [40.0, 250.0, 460.0] {
                let id = block(e, pillar, 90.0, 60.0, 250.0, 4.0, Color::rgb(0x40, 0x14, 0x20));
                shaded(e, id, Color::rgb(0x60, 0x20, 0x28), Color::rgb(0x20, 0x08, 0x10));
            }
        }
    }
    centered(e, title, WIDTH / 2.0, 10.0, 16.0, WHITE, bold());
    centered(e, t("Arrows walk, Up jumps, Space fires"), WIDTH / 2.0, 34.0, 12.0, PALE, TextStyle::default());
    counter(e, score, 14.0, 8.0, 20.0, WHITE);
    counter(e, lives, 490.0, 8.0, 20.0, YELLOW);
    if place == Place::Lair {
        counter(e, t("Armor"), 14.0, 34.0, 18.0, PINK);
    }
    layer(e, t("Game"));
    // The ground is solid. The marks on it keep pace with the world.
    let slab = block(e, -10.0, GROUND, WIDTH + 20.0, HEIGHT - GROUND + 10.0, 0.0, DARK_STEEL);
    shaded(e, slab, Color::rgb(0x58, 0x60, 0x78), Color::rgb(0x18, 0x1c, 0x2c));
    e.for_selected(|o| o.name = t("Ground").to_string());
    give(e, slab, &[(Kind::Solid, 0.0, 0.0)]);
    if place != Place::Lair {
        strip(e, PACE, |e, from| (0..8).map(|mark| block(e, from + 20.0 + mark as f32 * 80.0, GROUND + 26.0, 44.0, 6.0, 2.0, YELLOW)).collect());
    }
    // The gates stand off the right edge and above the top one: where the rules make copies appear.
    let gate = |e: &mut Editor, name: &str, x: f32, y: f32| {
        let id = add(e, name, Shape::Auto { kind: AutoKind::Star, a: 0.5, b: 0.0, count: 4 }, Rect { x: x - 8.0, y: y - 8.0, w: 16.0, h: 16.0 }, Some(PINK), None);
        e.selection = vec![id];
    };
    if place != Place::Lair {
        gate(e, t("Gate"), WIDTH + 40.0, GROUND - 18.0);
    }
    if matches!(place, Place::Factory | Place::Roof) {
        gate(e, t("Sky gate"), WIDTH + 40.0, 150.0);
    }
    gate(e, t("Heart gate"), 430.0, -30.0);
    // What copies are made from waits off the left edge, and is hidden when the scene starts. Each is a copy of
    // a symbol, the same one in every scene: change the symbol, and the character is changed everywhere.
    let id = put(e, cast.rat, t("Rat"), -150.0, 60.0);
    give(e, id, &[(Kind::Drift, PACE + 20.0, 180.0)]);
    let id = put(e, cast.drone, t("Drone"), -150.0, 110.0);
    give(e, id, &[(Kind::Drift, 120.0, 180.0), (Kind::UpAndDown, 36.0, 1.8)]);
    let id = put(e, cast.shot, t("Shot"), -150.0, 160.0);
    give(e, id, &[(Kind::Drift, 240.0, 180.0)]);
    let id = put(e, cast.bomb, t("Bomb"), -150.0, 180.0);
    give(e, id, &[(Kind::Gravity, 320.0, 0.0)]);
    let id = put(e, cast.heart, t("Heart"), -150.0, 220.0);
    give(e, id, &[(Kind::Gravity, 70.0, 0.0)]);
    let id = put(e, cast.bullet, t("Bullet"), -150.0, 260.0);
    give(e, id, &[(Kind::Drift, 620.0, 0.0)]);
    let id = put(e, cast.blast, t("Explosion"), -160.0, 280.0);
    give(e, id, &[(Kind::Pulse, 90.0, 0.7), (Kind::Spin, 1.4, 0.0), (Kind::Expire, 0.35, 0.0)]);
    match place {
        Place::Roof => {
            // A gun turret on a crate. The crate is solid: Pisi can jump onto it.
            let crate_box = block(e, 520.0, GROUND - 40.0, 70.0, 40.0, 4.0, Color::rgb(0x70, 0x50, 0x30));
            shaded(e, crate_box, Color::rgb(0x98, 0x70, 0x40), Color::rgb(0x48, 0x30, 0x18));
            give(e, crate_box, &[(Kind::Solid, 0.0, 0.0)]);
            put(e, cast.turret, t("Turret"), 504.0, GROUND - 92.0);
        }
        Place::Lair => {
            let id = put(e, cast.giant, t("Giant Rat"), 386.0, GROUND - 190.0);
            give(e, id, &[(Kind::BackAndForth, -110.0, 4.6), (Kind::UpAndDown, 14.0, 1.4)]);
        }
        Place::City | Place::Factory => {}
    }
    // Pisi stands on the ground, a little in from the left edge.
    if e.place_symbol_at(cast.pisi, Vec2 { x: 110.0, y: GROUND - 40.0 }) {
        e.for_selected(|o| {
            o.name = "Pisi".to_string();
            o.translate(0.0, GROUND - o.bounds.bottom());
        });
        if let Some(id) = e.selection.first().copied() {
            give(e, id, &[(Kind::ArrowKeys, 190.0, 0.0), (Kind::Gravity, 1200.0, 520.0)]);
        }
    }
    stop_here(e);
}

/// A rule that happens every so many seconds.
fn every(seconds: f32, steps: &[(StepKind, &str, f32, f32)]) -> Rule {
    rule((EventKind::Every, "", "", seconds), steps)
}

pub fn iron_claw() -> Document {
    let mut e = start(640, 400, t("Background"), 30.0);
    let sfx = sounds(&mut e);
    let Some(cast) = cast(&mut e) else { return crate::examples::finish(e) };
    let places = [(Place::City, t("City")), (Place::Factory, t("Factory")), (Place::Roof, t("Rooftop"))];
    for (index, (place, name)) in places.iter().enumerate() {
        if index > 0 {
            e.add_page();
            e.doc.page_mut().layers[0].name = t("Background").to_string();
        }
        e.doc.page_mut().name = name.to_string();
        scene(&mut e, &cast, *place, &tf("Stage {}: {}", &[&(index + 1), name]));
    }
    e.add_page();
    e.doc.page_mut().layers[0].name = t("Background").to_string();
    e.doc.page_mut().name = t("Giant Rat").to_string();
    scene(&mut e, &cast, Place::Lair, t("The Giant Rat"));
    let score = t("Score");
    last_scene(&mut e, WIDTH, HEIGHT, t("Pisi wins!"), &format!("{score}: {{{score}}}"));
    last_scene(&mut e, WIDTH, HEIGHT, t("Game over"), &format!("{score}: {{{score}}}"));
    e.set_active_page(0);

    let (lives, armor) = (t("Lives"), t("Armor"));
    let (rat, drone, shot, bomb, heart, bullet, blast) = (t("Rat"), t("Drone"), t("Shot"), t("Bomb"), t("Heart"), t("Bullet"), t("Explosion"));
    let (gate, sky_gate, heart_gate, ground, turret, giant) = (t("Gate"), t("Sky gate"), t("Heart gate"), t("Ground"), t("Turret"), t("Giant Rat"));
    e.doc.variables = vec![Variable { name: score.to_string(), start: 0.0 }, Variable { name: lives.to_string(), start: 6.0 }, Variable { name: armor.to_string(), start: 30.0 }];
    let hide = |name| (StepKind::Hide, name, 0.0, 0.0);
    let copy = |name| (StepKind::Spawn, name, 0.0, 0.0);
    let remove = |name| (StepKind::Remove, name, 0.0, 0.0);
    let add_to = |name, by: f32| (StepKind::Add, name, by, 0.0);
    let flash = |name| (StepKind::Flash, name, 0.0, 0.0);
    let touch = |a, b, steps: &[(StepKind, &str, f32, f32)]| rule((EventKind::Touch, a, b, 0.0), steps);
    let edge = |a, side, steps: &[(StepKind, &str, f32, f32)]| rule((EventKind::Edge, a, side, 0.0), steps);
    let reaches = |value: f32, steps: &[(StepKind, &str, f32, f32)]| rule((EventKind::Number, score, "=", value), steps);
    let to_scene = |number: f32| (StepKind::GotoScene, "", number, 0.0);
    e.doc.rules = vec![
        // What copies are made from is hidden as each scene starts.
        rule((EventKind::Scene, "", "", 0.0), &[hide(rat), hide(drone), hide(shot), hide(bomb), hide(heart), hide(bullet), hide(blast)]),
        // Pisi's blaster.
        placed(rule((EventKind::Key, " ", "", 0.0), &[copy(bullet), sound(sfx.zap)]), &["Pisi"]),
        edge(bullet, "right", &[remove(bullet)]),
        // The rats: they roll in from the gate, shoot, and blow up when hit.
        placed(every(2.6, &[copy(rat)]), &[gate]),
        placed(every(2.3, &[copy(shot)]), &[rat]),
        placed(touch(bullet, rat, &[add_to(score, 1.0), copy(blast), remove(bullet), remove(rat), sound(sfx.boom)]), &[rat]),
        edge(rat, "left", &[remove(rat)]),
        edge(shot, "left", &[remove(shot)]),
        touch(shot, "Pisi", &[add_to(lives, -1.0), remove(shot), flash("Pisi"), sound(sfx.oops)]),
        placed(touch(rat, "Pisi", &[add_to(lives, -1.0), copy(blast), remove(rat), flash("Pisi"), sound(sfx.oops)]), &[rat]),
        // The drones: they fly in from the sky gate and drop bombs.
        placed(every(4.2, &[copy(drone)]), &[sky_gate]),
        placed(every(2.9, &[copy(bomb)]), &[drone]),
        placed(touch(bullet, drone, &[add_to(score, 1.0), copy(blast), remove(bullet), remove(drone), sound(sfx.boom)]), &[drone]),
        edge(drone, "left", &[remove(drone)]),
        placed(touch(bomb, ground, &[copy(blast), remove(bomb)]), &[bomb]),
        placed(touch(bomb, "Pisi", &[add_to(lives, -1.0), copy(blast), remove(bomb), flash("Pisi"), sound(sfx.oops)]), &[bomb]),
        // A heart falls now and then, and is worth a life.
        placed(every(11.0, &[copy(heart)]), &[heart_gate]),
        touch("Pisi", heart, &[add_to(lives, 1.0), remove(heart), sound(sfx.ding)]),
        // The turret on the rooftop.
        placed(every(1.9, &[copy(shot)]), &[turret]),
        placed(touch(bullet, turret, &[add_to(score, 1.0), copy(blast), remove(bullet), remove(turret), sound(sfx.boom)]), &[turret]),
        // Enough enemies down, and the next stage begins.
        reaches(10.0, &[sound(sfx.win), to_scene(2.0)]),
        reaches(25.0, &[sound(sfx.win), to_scene(3.0)]),
        reaches(45.0, &[sound(sfx.win), to_scene(4.0)]),
        // The giant rat: it shoots, and every hit takes some of its armor.
        placed(every(1.3, &[copy(shot)]), &[giant]),
        placed(touch(bullet, giant, &[add_to(armor, -1.0), copy(blast), remove(bullet), flash(giant), sound(sfx.boom)]), &[bullet]),
        rule((EventKind::Number, armor, "<", 1.0), &[sound(sfx.win), to_scene(5.0)]),
        rule((EventKind::Number, lives, "<", 1.0), &[to_scene(6.0)]),
        rule((EventKind::Click, t("Again"), "", 0.0), &[(StepKind::Restart, "", 0.0, 0.0)]),
    ];
    crate::examples::finish(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_showcase_is_six_scenes_of_behaviors_and_rules() {
        crate::i18n::set_language(crate::i18n::Language::English);
        let doc = iron_claw();
        // Three stages, the giant rat, and the two ways it can end.
        assert_eq!(doc.scenes().len(), 6);
        // Pisi is a movie clip of twelve frames, four steps of three, with a copy on each of the four scenes it is played on.
        let clip = doc.symbols.iter().find(|s| s.name == "Pisi").unwrap();
        assert_eq!(clip.kind, SymbolKind::Animation);
        assert_eq!(doc.pages[doc.clip_page(clip.id).unwrap()].timeline.as_ref().unwrap().frame_count(), 12);
        let on_stage = |page: &crate::model::Page| page.objects().filter(|o| o.name == "Pisi" && matches!(o.shape, Shape::Instance { .. })).count();
        assert_eq!(doc.scenes().iter().map(|index| on_stage(&doc.pages[*index])).collect::<Vec<_>>(), vec![1, 1, 1, 1, 0, 0]);
        // The whole game is sentences.
        assert_eq!(doc.rules.len(), 28);
        assert_eq!(doc.rules[1].sentence(), "When the Space key is pressed: make a copy of Bullet at Pisi; play the sound zap");
        assert_eq!(doc.rules[4].sentence(), "Every 2.3 seconds: make a copy of Shot at Rat");
        assert_eq!(doc.rules[8].sentence(), "When Shot touches Pisi: decrease Lives by 1; remove Shot; make Pisi blink for a moment; play the sound oops");
        assert_eq!(doc.rules[20].sentence(), "When Score becomes 10: play the sound fanfare; go to scene 2");
        // The page has Pisi as a clip with its name and behaviors, the scenery as strips that keep going left, and the scene rule.
        let html = crate::publish::html(&doc, "x").unwrap();
        assert!(html.contains("\"l\":[[\"keys\",190,0],[\"gravity\",1200,520]],\"n\":\"Pisi\""));
        assert!(html.contains("{\"e\":[\"scene-start\",\"\",\"\",0],\"c\":null,\"s\":[[\"hide\",\"Rat\""));
        assert!(html.matches("[\"drift\",150,180]").count() >= 6, "the marks on the ground of three stages, two strips each");
        // The giant rat is only in its lair, and the turret only on the rooftop.
        let named = |scene: usize, name: &str| doc.pages[doc.scenes()[scene]].objects().any(|o| o.name == name);
        assert!(named(3, "Giant Rat") && !named(0, "Giant Rat") && named(2, "Turret") && !named(1, "Turret") && named(1, "Sky gate") && !named(0, "Sky gate"));
    }
}
