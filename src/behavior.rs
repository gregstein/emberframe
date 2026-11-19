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

//! Behaviors: ready-made motions an object is given with a click, with no keyframes.
//!
//! This is PisiAnimasyon's own, made for students who are not ready for a timeline full of
//! keyframes, let alone a script. An object keeps a list of behaviors; each is a kind
//! and two numbers. Those that depend only on time (turning, bouncing, going back and
//! forth) are worked out here, for playing inside the program, and again by the script
//! of the published page, by the same rules. Those that answer the mouse or the keyboard
//! work only in the published film, as buttons do.

use serde::{Deserialize, Serialize};

use crate::model::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorKind {
    Spin,
    Bounce,
    BackAndForth,
    UpAndDown,
    Drift,
    Shake,
    Pulse,
    Blink,
    Fade,
    /// Gone after some seconds: for sparks, bursts and anything else that is only to flash by.
    Expire,
    FollowMouse,
    ArrowKeys,
    Gravity,
    Draggable,
    /// Flies in a straight line and bounces off the edges of the stage and off what is solid, as a ball does.
    Ricochet,
    /// Turns as a wheel does: by as much as the way it has gone along the ground.
    Roll,
    /// The view goes along with it: it stays where it is on the screen, and everything else slides past.
    Camera,
    /// Slides past by only a part of how far the view has gone, as what is far away does, and may come back in at the other side.
    Scenery,
    /// Other objects cannot pass through it: what walks or falls stops at it, and stands on it.
    Solid,
}

/// Every kind, in the order the menu lists them: first those that move by themselves, then those the viewer moves.
pub const KINDS: [BehaviorKind; 19] = [
    BehaviorKind::Spin,
    BehaviorKind::Bounce,
    BehaviorKind::BackAndForth,
    BehaviorKind::UpAndDown,
    BehaviorKind::Drift,
    BehaviorKind::Shake,
    BehaviorKind::Pulse,
    BehaviorKind::Blink,
    BehaviorKind::Fade,
    BehaviorKind::Expire,
    BehaviorKind::FollowMouse,
    BehaviorKind::ArrowKeys,
    BehaviorKind::Gravity,
    BehaviorKind::Draggable,
    BehaviorKind::Ricochet,
    BehaviorKind::Roll,
    BehaviorKind::Camera,
    BehaviorKind::Scenery,
    BehaviorKind::Solid,
];

impl BehaviorKind {
    /// The name in the menu, in English; the interface translates it.
    pub const fn name(self) -> &'static str {
        match self {
            BehaviorKind::Spin => "Spin",
            BehaviorKind::Bounce => "Bounce",
            BehaviorKind::BackAndForth => "Go Back and Forth",
            BehaviorKind::UpAndDown => "Go Up and Down",
            BehaviorKind::Drift => "Keep Going",
            BehaviorKind::Shake => "Shake",
            BehaviorKind::Pulse => "Grow and Shrink",
            BehaviorKind::Blink => "Blink",
            BehaviorKind::Fade => "Fade In and Out",
            BehaviorKind::Expire => "Disappear After a While",
            BehaviorKind::FollowMouse => "Follow the Mouse",
            BehaviorKind::ArrowKeys => "Walk with the Arrow Keys",
            BehaviorKind::Gravity => "Fall and Jump",
            BehaviorKind::Draggable => "Can Be Dragged",
            BehaviorKind::Ricochet => "Bounce off the Edges",
            BehaviorKind::Roll => "Roll Like a Wheel",
            BehaviorKind::Camera => "Camera Follows Me",
            BehaviorKind::Scenery => "Slide Like Scenery",
            BehaviorKind::Solid => "Is Solid",
        }
    }

    /// The word the published page's script knows the kind by.
    pub const fn key(self) -> &'static str {
        match self {
            BehaviorKind::Spin => "spin",
            BehaviorKind::Bounce => "bounce",
            BehaviorKind::BackAndForth => "swing",
            BehaviorKind::UpAndDown => "rise",
            BehaviorKind::Drift => "drift",
            BehaviorKind::Shake => "shake",
            BehaviorKind::Pulse => "pulse",
            BehaviorKind::Blink => "blink",
            BehaviorKind::Fade => "fade",
            BehaviorKind::Expire => "expire",
            BehaviorKind::FollowMouse => "mouse",
            BehaviorKind::ArrowKeys => "keys",
            BehaviorKind::Gravity => "gravity",
            BehaviorKind::Draggable => "drag",
            BehaviorKind::Ricochet => "ricochet",
            BehaviorKind::Roll => "roll",
            BehaviorKind::Camera => "camera",
            BehaviorKind::Scenery => "scenery",
            BehaviorKind::Solid => "solid",
        }
    }

    /// What the kind's two numbers are, in English, and what they start as. A kind with one number has no second name.
    pub const fn settings(self) -> ((&'static str, f32), Option<(&'static str, f32)>) {
        match self {
            BehaviorKind::Spin => (("Seconds for one turn:", 2.0), Some(("Counterclockwise (1: yes, 0: no):", 0.0))),
            BehaviorKind::Bounce => (("Height (pixels):", 60.0), Some(("Seconds for one bounce:", 0.8))),
            BehaviorKind::BackAndForth => (("Distance (pixels):", 120.0), Some(("Seconds there and back:", 2.0))),
            BehaviorKind::UpAndDown => (("Distance (pixels):", 60.0), Some(("Seconds there and back:", 2.0))),
            BehaviorKind::Drift => (("Speed (pixels a second):", 60.0), Some(("Direction (degrees):", 0.0))),
            BehaviorKind::Shake => (("Strength (pixels):", 3.0), None),
            BehaviorKind::Pulse => (("Growth (percent):", 25.0), Some(("Seconds for one beat:", 1.0))),
            BehaviorKind::Blink => (("Seconds for one blink:", 1.0), None),
            BehaviorKind::Fade => (("Seconds there and back:", 2.0), None),
            BehaviorKind::Expire => (("Seconds:", 0.5), None),
            BehaviorKind::FollowMouse => (("Speed (pixels a second):", 200.0), None),
            BehaviorKind::ArrowKeys => (("Speed (pixels a second):", 160.0), None),
            BehaviorKind::Gravity => (("Pull (pixels a second, each second):", 900.0), Some(("Jump speed (pixels a second):", 420.0))),
            BehaviorKind::Ricochet => (("Speed (pixels a second):", 220.0), Some(("Direction (degrees):", 50.0))),
            BehaviorKind::Scenery => (("Slides by (percent: 100 as the road, 0 stays on the screen):", 40.0), Some(("Comes back in at the other side (1: yes, 0: no):", 1.0))),
            BehaviorKind::Draggable | BehaviorKind::Roll | BehaviorKind::Camera | BehaviorKind::Solid => (("", 0.0), None),
        }
    }

    /// Whether the kind waits for the viewer's mouse or keyboard, and so shows only in the published film.
    pub const fn interactive(self) -> bool {
        matches!(self, BehaviorKind::FollowMouse | BehaviorKind::ArrowKeys | BehaviorKind::Gravity | BehaviorKind::Draggable | BehaviorKind::Ricochet | BehaviorKind::Roll | BehaviorKind::Camera | BehaviorKind::Scenery | BehaviorKind::Solid)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Behavior {
    pub kind: BehaviorKind,
    /// The kind's first number, and its second: see [`BehaviorKind::settings`].
    pub a: f32,
    pub b: f32,
}

impl Behavior {
    pub fn new(kind: BehaviorKind) -> Behavior {
        let (first, second) = kind.settings();
        Behavior { kind, a: first.1, b: second.map_or(0.0, |s| s.1) }
    }

    /// What the script of the published page reads.
    pub fn text(&self) -> String {
        format!("[\"{}\",{},{}]", self.kind.key(), self.a, self.b)
    }
}

/// What behaviors have done to an object by some moment: moved it, turned it and resized it
/// around its transformation point, and let it through by a part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moved {
    pub dx: f32,
    pub dy: f32,
    /// Clockwise, in radians.
    pub angle: f32,
    pub scale: f32,
    pub alpha: f32,
}

/// A length of time that is never nothing, so that nothing is divided by it.
fn span(seconds: f32) -> f32 {
    if seconds.abs() < 0.01 { 0.01 } else { seconds }
}

/// Where a box that left one edge of the stage comes back in from the other: `at` kept between `-size` and `room`.
fn wrapped(at: f32, size: f32, room: f32) -> f32 {
    (at + size).rem_euclid(room + size) - size
}

/// What the behaviors that depend on time alone have done after `tick` frames at `fps` frames a second,
/// to an object whose box is `bounds` on a stage of the given size. The script of the published page
/// follows the same rules; change both together.
pub fn moved(behaviors: &[Behavior], tick: usize, fps: f32, bounds: Rect, stage: (f32, f32)) -> Moved {
    use std::f32::consts::{PI, TAU};
    let t = tick as f32 / fps.max(1.0);
    let mut moved = Moved { dx: 0.0, dy: 0.0, angle: 0.0, scale: 1.0, alpha: 1.0 };
    for behavior in behaviors {
        let (a, b) = (behavior.a, behavior.b);
        match behavior.kind {
            BehaviorKind::Spin => moved.angle += TAU * t / span(a) * if b != 0.0 { -1.0 } else { 1.0 },
            BehaviorKind::Bounce => moved.dy -= a * (PI * t / span(b)).sin().abs(),
            BehaviorKind::BackAndForth => moved.dx += a * (1.0 - (TAU * t / span(b)).cos()) / 2.0,
            BehaviorKind::UpAndDown => moved.dy -= a * (1.0 - (TAU * t / span(b)).cos()) / 2.0,
            BehaviorKind::Drift => {
                let (sin, cos) = b.to_radians().sin_cos();
                moved.dx += wrapped(bounds.x + a * cos * t, bounds.w, stage.0) - bounds.x;
                moved.dy += wrapped(bounds.y - a * sin * t, bounds.h, stage.1) - bounds.y;
            }
            BehaviorKind::Shake => {
                let n = tick as f32;
                moved.dx += a * (n * 12.9898).sin();
                moved.dy += a * (n * 78.233).cos();
            }
            BehaviorKind::Pulse => moved.scale *= 1.0 + a / 100.0 * (1.0 - (TAU * t / span(b)).cos()) / 2.0,
            BehaviorKind::Blink => {
                if t.rem_euclid(span(a).abs()) >= span(a).abs() / 2.0 {
                    moved.alpha = 0.0;
                }
            }
            BehaviorKind::Fade => moved.alpha *= (1.0 + (TAU * t / span(a)).cos()) / 2.0,
            BehaviorKind::Expire => {
                if t >= a {
                    moved.alpha = 0.0;
                }
            }
            BehaviorKind::FollowMouse | BehaviorKind::ArrowKeys | BehaviorKind::Gravity | BehaviorKind::Draggable | BehaviorKind::Ricochet | BehaviorKind::Roll | BehaviorKind::Camera | BehaviorKind::Scenery | BehaviorKind::Solid => {}
        }
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: Rect = Rect { x: 100.0, y: 50.0, w: 40.0, h: 20.0 };

    fn after(kind: BehaviorKind, a: f32, b: f32, tick: usize) -> Moved {
        moved(&[Behavior { kind, a, b }], tick, 10.0, BOX, (400.0, 300.0))
    }

    #[test]
    fn each_behavior_does_what_its_name_says() {
        // One turn in two seconds: a quarter of a turn after half a second.
        assert!((after(BehaviorKind::Spin, 2.0, 0.0, 5).angle - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
        assert!((after(BehaviorKind::Spin, 2.0, 1.0, 5).angle + std::f32::consts::FRAC_PI_2).abs() < 1e-4, "the other way round");
        // A bounce of 60 pixels in a second is highest after half a second and down again after one.
        assert!((after(BehaviorKind::Bounce, 60.0, 1.0, 5).dy + 60.0).abs() < 1e-3 && after(BehaviorKind::Bounce, 60.0, 1.0, 10).dy.abs() < 1e-3);
        // There and back in two seconds: there after one, back after two.
        assert!((after(BehaviorKind::BackAndForth, 120.0, 2.0, 10).dx - 120.0).abs() < 1e-3 && after(BehaviorKind::BackAndForth, 120.0, 2.0, 20).dx.abs() < 1e-3);
        assert!((after(BehaviorKind::UpAndDown, 60.0, 2.0, 10).dy + 60.0).abs() < 1e-3);
        // A quarter bigger at the middle of the beat.
        assert!((after(BehaviorKind::Pulse, 25.0, 1.0, 5).scale - 1.25).abs() < 1e-4);
        // Seen for the first half of a blink and not for the second.
        assert_eq!((after(BehaviorKind::Blink, 1.0, 0.0, 2).alpha, after(BehaviorKind::Blink, 1.0, 0.0, 7).alpha), (1.0, 0.0));
        assert!(after(BehaviorKind::Fade, 2.0, 0.0, 10).alpha.abs() < 1e-4);
        // There for half a second, then gone.
        assert_eq!((after(BehaviorKind::Expire, 0.5, 0.0, 4).alpha, after(BehaviorKind::Expire, 0.5, 0.0, 5).alpha), (1.0, 0.0));
        assert!(after(BehaviorKind::Shake, 3.0, 0.0, 3).dx.abs() <= 3.0);
        // Those that wait for the viewer do nothing by themselves.
        assert_eq!(after(BehaviorKind::ArrowKeys, 160.0, 0.0, 9), Moved { dx: 0.0, dy: 0.0, angle: 0.0, scale: 1.0, alpha: 1.0 });
    }

    #[test]
    fn what_keeps_going_comes_back_in_from_the_other_side() {
        // To the right at 100 pixels a second: 100 further after a second, and after leaving the
        // right edge of a stage 400 wide it comes in again from just outside the left edge.
        assert!((after(BehaviorKind::Drift, 100.0, 0.0, 10).dx - 100.0).abs() < 1e-3);
        let left = BOX.x + after(BehaviorKind::Drift, 100.0, 0.0, 31).dx;
        assert!((-40.0..0.0).contains(&left), "{left}");
        // Up is 90 degrees.
        assert!((after(BehaviorKind::Drift, 100.0, 90.0, 2).dy + 20.0).abs() < 1e-2);
    }

    #[test]
    fn behaviors_add_up_and_every_kind_has_a_name_and_a_word() {
        let both = moved(&[Behavior::new(BehaviorKind::Spin), Behavior::new(BehaviorKind::Bounce)], 4, 10.0, BOX, (400.0, 300.0));
        assert!(both.angle > 0.0 && both.dy < 0.0);
        let mut keys: Vec<&str> = KINDS.iter().map(|kind| kind.key()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), KINDS.len());
        assert_eq!(Behavior::new(BehaviorKind::Bounce).text(), "[\"bounce\",60,0.8]");
    }
}
