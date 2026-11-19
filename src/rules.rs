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

//! Rules: what makes a film a game. "When this happens, do that", picked from lists.
//!
//! PisiAnimasyon's own, the step after behaviors. A rule is an event and the steps to take
//! when it happens; both name the objects they are about by the names the objects have
//! in the document, so one rule about "Apple" holds for every apple. Numbers (a score,
//! lives, a clock) are kept by name as well, and a text that has `{Score}` in it shows
//! the number in the film.
//!
//! Every rule can be read as a sentence, in the language of the interface. Those
//! sentences are the beginning of a written language: the lists only write them for you.
//! The ideas are kept to an object model on purpose (an object, an event on it, a step
//! done to it, a number), so that a programming language can be laid over them later.
//!
//! Rules run in the published film, as buttons do.

use serde::{Deserialize, Serialize};

use crate::i18n::{t, tf};

/// What a rule waits for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// The film starts, or starts again.
    #[default]
    Start,
    /// A scene starts: at the beginning of the film, and whenever the film goes to another scene.
    Scene,
    /// A key goes down: `a` is the key.
    Key,
    /// A key comes up again: `a` is the key.
    KeyUp,
    /// An object is clicked: `a` is its name.
    Click,
    /// An object comes to touch another: `a` and `b` are their names.
    Touch,
    /// An object comes to touch an edge of the stage: `a` is its name, `b` the edge.
    Edge,
    /// Again and again, every `n` seconds.
    Every,
    /// Once, `n` seconds after the film starts.
    After,
    /// A number comes to be, to pass or to fall under `n`: `a` is its name, `b` which of the three.
    Number,
}

pub const EVENTS: [EventKind; 10] = [EventKind::Start, EventKind::Scene, EventKind::Key, EventKind::KeyUp, EventKind::Click, EventKind::Touch, EventKind::Edge, EventKind::Every, EventKind::After, EventKind::Number];

/// What a name in a rule is chosen from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    Object,
    Key,
    Edge,
    /// One of the four ways an object can be sent.
    Way,
    Compare,
    /// One of the three ways of "only if".
    Condition,
    /// A command defined in the written language.
    Command,
    /// One of the ready behaviors, by its name in the menu.
    Behavior,
    Variable,
    Sound,
}

impl EventKind {
    /// The name in the list, in English.
    pub const fn name(self) -> &'static str {
        match self {
            EventKind::Start => "When the film starts",
            EventKind::Scene => "When a scene starts",
            EventKind::Key => "When a key is pressed",
            EventKind::KeyUp => "When a key is released",
            EventKind::Click => "When an object is clicked",
            EventKind::Touch => "When an object touches another",
            EventKind::Edge => "When an object touches the edge of the stage",
            EventKind::Every => "Every so many seconds",
            EventKind::After => "After so many seconds",
            EventKind::Number => "When a number reaches a value",
        }
    }

    /// The word the published page's script knows the kind by.
    pub const fn key(self) -> &'static str {
        match self {
            EventKind::Start => "start",
            EventKind::Scene => "scene-start",
            EventKind::Key => "key",
            EventKind::KeyUp => "keyup",
            EventKind::Click => "click",
            EventKind::Touch => "touch",
            EventKind::Edge => "edge",
            EventKind::Every => "every",
            EventKind::After => "after",
            EventKind::Number => "number",
        }
    }

    /// What the kind's first and second names are chosen from, and what its number is called, if it has them.
    pub const fn needs(self) -> (Option<Pick>, Option<Pick>, Option<&'static str>) {
        match self {
            EventKind::Start | EventKind::Scene => (None, None, None),
            EventKind::Key | EventKind::KeyUp => (Some(Pick::Key), None, None),
            EventKind::Click => (Some(Pick::Object), None, None),
            EventKind::Touch => (Some(Pick::Object), Some(Pick::Object), None),
            EventKind::Edge => (Some(Pick::Object), Some(Pick::Edge), None),
            EventKind::Every | EventKind::After => (None, None, Some("seconds")),
            EventKind::Number => (Some(Pick::Variable), Some(Pick::Compare), Some("")),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub kind: EventKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub a: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub b: String,
    #[serde(default)]
    pub n: f32,
}

/// What a rule does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepKind {
    /// Moves the object `a` by `x` to the right and `y` down.
    Move,
    /// Puts the object's top left corner at `x`, `y`.
    GoTo,
    /// Puts the object somewhere on the stage at random.
    Random,
    /// Puts the object at the top of the stage, at a random place along it, and stops its fall.
    RandomTop,
    Show,
    Hide,
    /// Makes the object jump `x` pixels high. It needs Fall and Jump, which is what brings it down again.
    Jump,
    /// Adds `x` to the number `a`.
    #[default]
    Add,
    /// Makes the number `a` be `x`.
    Set,
    StopFilm,
    PlayFilm,
    /// Goes to frame `x` of the scene, counted from 1, and plays on.
    GotoFrame,
    /// Goes to frame `x` and stops there.
    GotoFrameStop,
    /// Goes to scene `x`, counted from 1.
    GotoScene,
    NextScene,
    /// Plays the sound called `a`.
    Sound,
    /// Makes a copy of the object `a` where the object `b` is. The copy has the name and the behaviors of the first.
    Spawn,
    /// Takes the object away for good: a copy is gone, anything else is hidden until the film starts again.
    Remove,
    /// Makes everything the object does by itself faster by `x` percent; a negative number slows it.
    Faster,
    /// Makes the number `a` a whole number picked at random from `x` to `y`.
    RandomNumber,
    /// Makes the object blink for a moment, as a thing that has been hit does.
    Flash,
    /// Turns the object by `x` degrees, clockwise, around its transformation point.
    Turn,
    /// Makes the object `x` percent of the size it was drawn at.
    Resize,
    /// Makes the object `x` percent opaque: 0 cannot be seen, 100 is as it was drawn.
    Opacity,
    /// Gives the object the behavior whose word is `b`, with the numbers that behavior starts with.
    Give,
    /// Takes the behavior whose word is `b` from the object.
    Take,
    /// Keeps the object spinning at `x` turns a second, clockwise; below zero the other way, and at 0 it stops.
    Whirl,
    /// Ties the object to the object `b`: from then on it goes wherever that one goes.
    Attach,
    /// Lets go an object that was tied to another.
    Detach,
    /// Sets the object going the way `b` at `x` pixels a second: it gathers speed, and keeps going.
    Go,
    /// Lets the object that is going lose its speed little by little until it stands still.
    Slow,
    /// Stops the object that is going, at once.
    Halt,
    /// The view goes along with the object from now on.
    Camera,
    /// The view stays where it is.
    CameraStop,
    /// Sends the object after the object `b` at `x` pixels a second; at 0 it stops going after anything.
    Chase,
    /// Waits `x` seconds before the steps that follow it.
    Wait,
    /// Does the command called `a`: steps written once under a name, in the written language.
    Call,
    /// Does the steps of its `body` `x` times over. This and the two after it hold steps of their own, and
    /// are written in the language; the lists show them and leave them as they are.
    Repeat,
    /// Does its `body` if its conditions (`only`) hold, and its `otherwise` if they do not.
    If,
    /// Does its `body` again and again for as long as its conditions hold.
    While,
    /// Stops the film and everything that moves in it.
    StopAll,
    /// Starts the film again from the beginning, with every number as it started.
    Restart,
}

pub const STEPS: [StepKind; 39] = [
    StepKind::Add,
    StepKind::Set,
    StepKind::RandomNumber,
    StepKind::Hide,
    StepKind::Show,
    StepKind::Spawn,
    StepKind::Remove,
    StepKind::Flash,
    StepKind::Faster,
    StepKind::Move,
    StepKind::GoTo,
    StepKind::Random,
    StepKind::RandomTop,
    StepKind::Jump,
    StepKind::Turn,
    StepKind::Whirl,
    StepKind::Resize,
    StepKind::Opacity,
    StepKind::Give,
    StepKind::Take,
    StepKind::Chase,
    StepKind::Go,
    StepKind::Slow,
    StepKind::Halt,
    StepKind::Attach,
    StepKind::Detach,
    StepKind::Camera,
    StepKind::CameraStop,
    StepKind::Wait,
    StepKind::Call,
    StepKind::Sound,
    StepKind::NextScene,
    StepKind::GotoScene,
    StepKind::GotoFrame,
    StepKind::GotoFrameStop,
    StepKind::StopFilm,
    StepKind::PlayFilm,
    StepKind::StopAll,
    StepKind::Restart,
];

impl StepKind {
    pub const fn name(self) -> &'static str {
        match self {
            StepKind::Move => "Move an object",
            StepKind::GoTo => "Put an object at a place",
            StepKind::Random => "Put an object somewhere at random",
            StepKind::RandomTop => "Put an object at the top, at random",
            StepKind::Show => "Show an object",
            StepKind::Hide => "Hide an object",
            StepKind::Jump => "Make an object jump",
            StepKind::Add => "Change a number",
            StepKind::Set => "Set a number",
            StepKind::StopFilm => "Stop the film",
            StepKind::PlayFilm => "Play the film",
            StepKind::GotoFrame => "Go to a frame and play",
            StepKind::GotoFrameStop => "Go to a frame and stop",
            StepKind::GotoScene => "Go to a scene",
            StepKind::NextScene => "Go to the next scene",
            StepKind::Sound => "Play a sound",
            StepKind::Spawn => "Make a copy of an object",
            StepKind::Remove => "Remove an object",
            StepKind::Faster => "Make an object faster",
            StepKind::RandomNumber => "Set a number at random",
            StepKind::Flash => "Make an object blink for a moment",
            StepKind::Turn => "Turn an object",
            StepKind::Resize => "Resize an object",
            StepKind::Opacity => "Make an object see-through",
            StepKind::Give => "Give an object a behavior",
            StepKind::Take => "Take a behavior from an object",
            StepKind::Whirl => "Keep an object spinning",
            StepKind::Camera => "Make the camera follow an object",
            StepKind::CameraStop => "Let the camera stand still",
            StepKind::Chase => "Send an object after another",
            StepKind::Go => "Set an object going",
            StepKind::Slow => "Let an object slow down and stop",
            StepKind::Halt => "Stop an object",
            StepKind::Attach => "Attach an object to another",
            StepKind::Detach => "Let an attached object go",
            StepKind::Wait => "Wait",
            StepKind::Call => "Do a command",
            StepKind::Repeat => "Repeat",
            StepKind::If => "If",
            StepKind::While => "Repeat while",
            StepKind::StopAll => "Stop everything",
            StepKind::Restart => "Start again",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            StepKind::Move => "move",
            StepKind::GoTo => "goto",
            StepKind::Random => "random",
            StepKind::RandomTop => "top",
            StepKind::Show => "show",
            StepKind::Hide => "hide",
            StepKind::Jump => "jump",
            StepKind::Add => "add",
            StepKind::Set => "set",
            StepKind::StopFilm => "stop",
            StepKind::PlayFilm => "play",
            StepKind::GotoFrame => "frame",
            StepKind::GotoFrameStop => "frame-stop",
            StepKind::GotoScene => "scene",
            StepKind::NextScene => "next",
            StepKind::Sound => "sound",
            StepKind::Spawn => "spawn",
            StepKind::Remove => "remove",
            StepKind::Faster => "faster",
            StepKind::RandomNumber => "rnd",
            StepKind::Flash => "flash",
            StepKind::Turn => "turn",
            StepKind::Resize => "size",
            StepKind::Opacity => "alpha",
            StepKind::Give => "give",
            StepKind::Take => "take",
            StepKind::Whirl => "whirl",
            StepKind::Camera => "camera",
            StepKind::CameraStop => "camera-stop",
            StepKind::Chase => "chase",
            StepKind::Go => "go",
            StepKind::Slow => "slow",
            StepKind::Halt => "stand",
            StepKind::Attach => "attach",
            StepKind::Detach => "detach",
            StepKind::Wait => "wait",
            StepKind::Call => "call",
            StepKind::Repeat => "repeat",
            StepKind::If => "if",
            StepKind::While => "while",
            StepKind::StopAll => "halt",
            StepKind::Restart => "restart",
        }
    }

    /// What the kind's first and second names are chosen from, and what its two numbers are called and start as, if it has them.
    pub const fn needs(self) -> (Option<Pick>, Option<Pick>, Option<(&'static str, f32)>, Option<(&'static str, f32)>) {
        match self {
            StepKind::Move => (Some(Pick::Object), None, Some(("Rightwards:", 10.0)), Some(("Down:", 0.0))),
            StepKind::GoTo => (Some(Pick::Object), None, Some(("X:", 0.0)), Some(("Y:", 0.0))),
            StepKind::Random | StepKind::RandomTop | StepKind::Show | StepKind::Hide | StepKind::Remove | StepKind::Flash => (Some(Pick::Object), None, None, None),
            StepKind::Spawn => (Some(Pick::Object), Some(Pick::Object), None, None),
            StepKind::Jump => (Some(Pick::Object), None, Some(("Height:", 100.0)), None),
            StepKind::Faster => (Some(Pick::Object), None, Some(("Percent:", 10.0)), None),
            StepKind::Add => (Some(Pick::Variable), None, Some(("By:", 1.0)), None),
            StepKind::Set => (Some(Pick::Variable), None, Some(("Value:", 0.0)), None),
            StepKind::RandomNumber => (Some(Pick::Variable), None, Some(("At least:", 1.0)), Some(("Up to:", 6.0))),
            StepKind::GotoFrame | StepKind::GotoFrameStop => (None, None, Some(("Frame:", 1.0)), None),
            StepKind::GotoScene => (None, None, Some(("Scene:", 1.0)), None),
            StepKind::Turn => (Some(Pick::Object), None, Some(("Degrees:", 90.0)), None),
            StepKind::Resize => (Some(Pick::Object), None, Some(("Percent:", 150.0)), None),
            StepKind::Opacity => (Some(Pick::Object), None, Some(("Percent:", 50.0)), None),
            StepKind::Give | StepKind::Take => (Some(Pick::Object), Some(Pick::Behavior), None, None),
            StepKind::Whirl => (Some(Pick::Object), None, Some(("Turns a second:", 1.0)), None),
            StepKind::Chase => (Some(Pick::Object), Some(Pick::Object), Some(("Speed:", 100.0)), None),
            StepKind::Go => (Some(Pick::Object), Some(Pick::Way), Some(("Speed:", 250.0)), None),
            StepKind::Slow | StepKind::Halt | StepKind::Detach | StepKind::Camera => (Some(Pick::Object), None, None, None),
            StepKind::CameraStop => (None, None, None, None),
            StepKind::Attach => (Some(Pick::Object), Some(Pick::Object), None, None),
            StepKind::Wait => (None, None, Some(("Seconds:", 1.0)), None),
            StepKind::Call => (Some(Pick::Command), None, None, None),
            StepKind::Repeat => (None, None, Some(("Times:", 3.0)), None),
            StepKind::If | StepKind::While => (None, None, None, None),
            StepKind::Sound => (Some(Pick::Sound), None, None, None),
            StepKind::StopFilm | StepKind::PlayFilm | StepKind::NextScene | StepKind::StopAll | StepKind::Restart => (None, None, None, None),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub kind: StepKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub a: String,
    /// A second name, for the kinds that have one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub b: String,
    /// The first and the second number written as a sum to work out, such as `Score * 2` or `Hero.x + 10`,
    /// where they are not plain numbers. See [`crate::script`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ex: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ey: String,
    /// The steps a step holds, for the kinds that hold steps: what is repeated, or done if the conditions hold.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<Step>,
    /// The conditions of an "if" or a "repeat while".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<Condition>,
    /// What an "if" does when its conditions do not hold.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub otherwise: Vec<Step>,
    /// The values a command is done with, each as it is written: a number, a sum, or the name of an object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub event: Event,
    /// What must also be so for the rule to be followed, if anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Condition>,
    /// Further things that must be so as well. The lists set one condition; the written language any number.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also: Vec<Condition>,
    #[serde(default)]
    pub steps: Vec<Step>,
    /// What to do when the event happens and the conditions do not hold.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub otherwise: Vec<Step>,
}

/// A command: steps written once under a name, to be done from any rule with "do the command".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    /// The names of the values the command is given when it is done. Inside it each is a number to
    /// reckon with, or stands for the object it was given the name of.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<String>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

/// Every step in a list of steps, with those the steps hold, each before what it holds.
pub fn every_step(steps: &[Step]) -> Vec<&Step> {
    let mut all = Vec::new();
    for step in steps {
        all.push(step);
        all.extend(every_step(&step.body));
        all.extend(every_step(&step.otherwise));
    }
    all
}

/// The names of the objects some steps are about, those of the steps they hold among them.
pub fn objects_of(steps: &[Step]) -> Vec<&str> {
    let mut names = Vec::new();
    for step in every_step(steps) {
        let (a, b, _, _) = step.kind.needs();
        if a == Some(Pick::Object) {
            names.push(step.a.as_str());
        }
        if b == Some(Pick::Object) {
            names.push(step.b.as_str());
        }
        // And the objects whose places its sums ask for: `Hero.x`.
        let sums = [step.ex.as_str(), step.ey.as_str()].into_iter().chain(step.only.iter().map(|only| only.expr.as_str())).chain(step.args.iter().map(String::as_str));
        names.extend(sums.flat_map(crate::script::objects_in));
        // A value given to a command may be the name of an object: whatever is not a figure is taken for one here.
        names.extend(step.args.iter().map(|arg| arg.trim_matches('"')).filter(|arg| arg.replace(',', ".").parse::<f32>().is_err()));
    }
    names.retain(|name| !name.is_empty());
    names
}

/// Steps as the script of the published page reads them. A step that holds steps carries them after its own
/// five parts: its body, its conditions, and what to do otherwise. A sound is given by its place in `sounds`.
pub fn steps_text(steps: &[Step], sounds: &[String]) -> String {
    let list: Vec<String> = steps
        .iter()
        .map(|step| {
            // A number that is a sum goes as its text, for the script to work out when the step is taken.
            let x = if step.kind == StepKind::Sound {
                sounds.iter().position(|name| *name == step.a).map_or(-1.0, |at| at as f32).to_string()
            } else if step.ex.is_empty() {
                step.x.to_string()
            } else {
                quoted(&step.ex)
            };
            let y = if step.ey.is_empty() { step.y.to_string() } else { quoted(&step.ey) };
            let own = format!("\"{}\",{},{x},{y},{}", step.kind.key(), quoted(&step.a), quoted(&step.b));
            if step.kind == StepKind::Call {
                // The values the command is given go after its own five parts, each as it is written.
                let args: Vec<String> = step.args.iter().map(|arg| quoted(arg)).collect();
                format!("[{own},[{}]]", args.join(","))
            } else if matches!(step.kind, StepKind::Repeat | StepKind::If | StepKind::While) {
                let only: Vec<String> = step.only.iter().map(Condition::text).collect();
                format!("[{own},[{}],[{}],[{}]]", steps_text(&step.body, sounds), only.join(","), steps_text(&step.otherwise, sounds))
            } else {
                format!("[{own}]")
            }
        })
        .collect();
    list.join(",")
}

/// "Only if": a number held against a value, as [`COMPARES`] has the three ways.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub number: String,
    pub how: String,
    pub value: f32,
    /// The value written as a sum to work out, where it is not a plain number.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub expr: String,
}

impl Condition {
    /// The condition as the sentence says it: "Score is more than 2".
    pub fn phrase(&self) -> String {
        let value = if self.expr.is_empty() { plain(self.value) } else { self.expr.clone() };
        match self.how.as_str() {
            ">" => tf("{} is more than {}", &[&self.number, &value]),
            "<" => tf("{} is less than {}", &[&self.number, &value]),
            _ => tf("{} is {}", &[&self.number, &value]),
        }
    }

    fn text(&self) -> String {
        let value = if self.expr.is_empty() { self.value.to_string() } else { quoted(&self.expr) };
        format!("[{},{},{value}]", quoted(&self.number), quoted(&self.how))
    }
}

/// The three ways as "only if" says them: the word the script knows each by, and its name in English.
pub const CONDITIONS: [(&str, &str); 3] = [("=", "is"), (">", "is more than"), ("<", "is less than")];

/// A number the film keeps, such as a score: its name, and what it is when the film starts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    #[serde(default)]
    pub start: f32,
}

/// The keys a rule can wait for: what the browser calls the key, and its name in English.
pub const KEYS: [(&str, &str); 42] = [
    (" ", "Space"),
    ("ArrowUp", "Up Arrow"),
    ("ArrowDown", "Down Arrow"),
    ("ArrowLeft", "Left Arrow"),
    ("ArrowRight", "Right Arrow"),
    ("Enter", "Enter"),
    ("a", "A"),
    ("b", "B"),
    ("c", "C"),
    ("d", "D"),
    ("e", "E"),
    ("f", "F"),
    ("g", "G"),
    ("h", "H"),
    ("i", "I"),
    ("j", "J"),
    ("k", "K"),
    ("l", "L"),
    ("m", "M"),
    ("n", "N"),
    ("o", "O"),
    ("p", "P"),
    ("q", "Q"),
    ("r", "R"),
    ("s", "S"),
    ("t", "T"),
    ("u", "U"),
    ("v", "V"),
    ("w", "W"),
    ("x", "X"),
    ("y", "Y"),
    ("z", "Z"),
    ("0", "0"),
    ("1", "1"),
    ("2", "2"),
    ("3", "3"),
    ("4", "4"),
    ("5", "5"),
    ("6", "6"),
    ("7", "7"),
    ("8", "8"),
    ("9", "9"),
];

/// The edges of the stage: the word the script knows each by, and its name in English.
pub const EDGES: [(&str, &str); 5] = [("any", "any edge"), ("bottom", "the bottom edge"), ("top", "the top edge"), ("left", "the left edge"), ("right", "the right edge")];

/// The four ways an object can be sent: the word the script knows each by, and its name in English.
pub const WAYS: [(&str, &str); 4] = [("right", "to the right"), ("left", "to the left"), ("up", "upwards"), ("down", "downwards")];

/// The ways a number is held against a value.
pub const COMPARES: [(&str, &str); 3] = [("=", "becomes"), (">", "goes above"), ("<", "goes below")];

/// The name in the interface's language of a choice that is not an object's or a number's own name.
pub fn choice_name(pick: Pick, value: &str) -> String {
    let list: &[(&str, &str)] = match pick {
        Pick::Key => &KEYS,
        Pick::Edge => &EDGES,
        Pick::Way => &WAYS,
        Pick::Compare => &COMPARES,
        Pick::Condition => &CONDITIONS,
        Pick::Object | Pick::Variable | Pick::Sound | Pick::Command => return value.to_string(),
        Pick::Behavior => return crate::behavior::KINDS.iter().find(|kind| kind.key() == value).map_or_else(|| value.to_string(), |kind| t(kind.name()).to_string()),
    };
    list.iter().find(|(key, _)| *key == value).map_or_else(|| value.to_string(), |(_, name)| if name.len() == 1 { name.to_string() } else { t(name).to_string() })
}

/// A number as a sentence has it: whole where it is whole.
fn plain(n: f32) -> String {
    if n.fract() == 0.0 { format!("{}", n as i64) } else { format!("{n}") }
}

impl Event {
    pub fn new(kind: EventKind) -> Event {
        Event { kind, a: String::new(), b: String::new(), n: if matches!(kind, EventKind::Every | EventKind::After) { 1.0 } else { 0.0 } }
    }

    pub fn sentence(&self) -> String {
        let (a, n) = (&self.a, plain(self.n));
        match self.kind {
            EventKind::Start => t("When the film starts").to_string(),
            EventKind::Scene => t("When a scene starts").to_string(),
            EventKind::Key => tf("When the {} key is pressed", &[&choice_name(Pick::Key, a)]),
            EventKind::KeyUp => tf("When the {} key is released", &[&choice_name(Pick::Key, a)]),
            EventKind::Click => tf("When {} is clicked", &[a]),
            EventKind::Touch => tf("When {} touches {}", &[a, &self.b]),
            EventKind::Edge => tf("When {} touches {} of the stage", &[a, &choice_name(Pick::Edge, &self.b)]),
            EventKind::Every => tf("Every {} seconds", &[&n]),
            EventKind::After => tf("After {} seconds", &[&n]),
            EventKind::Number => match self.b.as_str() {
                ">" => tf("When {} goes above {}", &[a, &n]),
                "<" => tf("When {} goes below {}", &[a, &n]),
                _ => tf("When {} becomes {}", &[a, &n]),
            },
        }
    }

    fn text(&self) -> String {
        format!("[\"{}\",{},{},{}]", self.kind.key(), quoted(&self.a), quoted(&self.b), self.n)
    }
}

fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
}

impl Step {
    pub fn new(kind: StepKind) -> Step {
        let (_, _, x, y) = kind.needs();
        Step { kind, x: x.map_or(0.0, |x| x.1), y: y.map_or(0.0, |y| y.1), ..Default::default() }
    }

    /// The first number as it is written: the sum, if it is one.
    pub fn first(&self) -> String {
        if self.ex.is_empty() { plain(self.x) } else { self.ex.clone() }
    }

    pub fn second(&self) -> String {
        if self.ey.is_empty() { plain(self.y) } else { self.ey.clone() }
    }

    fn told_conditions(&self) -> String {
        let told: Vec<String> = self.only.iter().map(Condition::phrase).collect();
        told.join(t(" and "))
    }

    /// Whether the step holds steps of its own.
    pub fn is_block(&self) -> bool {
        matches!(self.kind, StepKind::Repeat | StepKind::If | StepKind::While)
    }

    /// The step on one line, what it holds between brackets: how a list shows it.
    pub fn told(&self) -> String {
        if !self.is_block() {
            return self.sentence();
        }
        let inside = |steps: &[Step]| steps.iter().map(Step::told).collect::<Vec<_>>().join("; ");
        let rest = if self.otherwise.is_empty() { String::new() } else { format!(" {} [{}]", t("otherwise"), inside(&self.otherwise)) };
        format!("{} [{}]{rest}", self.sentence(), inside(&self.body))
    }

    pub fn sentence(&self) -> String {
        let (a, x, y) = (&self.a, self.first(), self.second());
        // Taking away reads as taking away only where the number is a plain one.
        let down = self.ex.is_empty() && self.x < 0.0;
        match self.kind {
            StepKind::Move => tf("move {} by {} right and {} down", &[a, &x, &y]),
            StepKind::GoTo => tf("put {} at {}, {}", &[a, &x, &y]),
            StepKind::Random => tf("put {} somewhere at random", &[a]),
            StepKind::RandomTop => tf("put {} at the top, at random", &[a]),
            StepKind::Show => tf("show {}", &[a]),
            StepKind::Hide => tf("hide {}", &[a]),
            StepKind::Jump => tf("make {} jump {} high", &[a, &x]),
            StepKind::Add if down => tf("decrease {} by {}", &[a, &plain(-self.x)]),
            StepKind::Add => tf("increase {} by {}", &[a, &x]),
            StepKind::Spawn => tf("make a copy of {} at {}", &[a, &self.b]),
            StepKind::Remove => tf("remove {}", &[a]),
            StepKind::Flash => tf("make {} blink for a moment", &[a]),
            StepKind::Turn => tf("turn {} by {} degrees", &[a, &x]),
            StepKind::Resize => tf("make {} {} percent of its size", &[a, &x]),
            StepKind::Opacity => tf("make {} {} percent opaque", &[a, &x]),
            StepKind::Give => tf("give {} the behavior {}", &[a, &choice_name(Pick::Behavior, &self.b)]),
            StepKind::Take => tf("take from {} the behavior {}", &[a, &choice_name(Pick::Behavior, &self.b)]),
            StepKind::Whirl => tf("make {} spin {} turns a second", &[a, &x]),
            StepKind::Camera => tf("make the camera follow {}", &[a]),
            StepKind::CameraStop => t("let the camera stand still").to_string(),
            StepKind::Chase => tf("send {} after {} at {}", &[a, &self.b, &x]),
            StepKind::Go => tf("make {} go {} at {}", &[a, &choice_name(Pick::Way, &self.b), &x]),
            StepKind::Slow => tf("make {} slow down and stop", &[a]),
            StepKind::Halt => tf("make {} stop", &[a]),
            StepKind::Attach => tf("attach {} to {}", &[a, &self.b]),
            StepKind::Detach => tf("let {} go", &[a]),
            StepKind::Wait => tf("wait {} seconds", &[&x]),
            StepKind::Call if !self.args.is_empty() => tf("do the command {} with {}", &[a, &self.args.join(", ")]),
            StepKind::Call => tf("do the command {}", &[a]),
            StepKind::Repeat => tf("repeat {} times", &[&x]),
            StepKind::If => tf("if {}", &[&self.told_conditions()]),
            StepKind::While => tf("repeat while {}", &[&self.told_conditions()]),
            StepKind::Faster if down => tf("make {} slower by {} percent", &[a, &plain(-self.x)]),
            StepKind::Faster => tf("make {} faster by {} percent", &[a, &x]),
            StepKind::RandomNumber => tf("set {} at random from {} to {}", &[a, &x, &y]),
            StepKind::Set => tf("set {} to {}", &[a, &x]),
            StepKind::StopFilm => t("stop the film").to_string(),
            StepKind::PlayFilm => t("play the film").to_string(),
            StepKind::GotoFrame => tf("go to frame {} and play", &[&x]),
            StepKind::GotoFrameStop => tf("go to frame {} and stop", &[&x]),
            StepKind::GotoScene => tf("go to scene {}", &[&x]),
            StepKind::NextScene => t("go to the next scene").to_string(),
            StepKind::Sound => tf("play the sound {}", &[a]),
            StepKind::StopAll => t("stop everything").to_string(),
            StepKind::Restart => t("start again").to_string(),
        }
    }
}

impl Rule {
    pub fn new() -> Rule {
        Rule::default()
    }

    /// Every condition of the rule, the one the lists set first.
    pub fn conditions(&self) -> impl Iterator<Item = &Condition> {
        self.when.iter().chain(self.also.iter())
    }

    /// The event and what must also be so, as the first line of the rule in the written language.
    pub fn head(&self) -> String {
        let event = self.event.sentence();
        let conditions: Vec<String> = self.conditions().map(Condition::phrase).collect();
        if conditions.is_empty() {
            event
        } else {
            tf("{}, if {}", &[&event, &conditions.join(t(" and "))])
        }
    }

    /// The rule as one sentence: the event, what must also be so, then the steps.
    pub fn sentence(&self) -> String {
        let steps: Vec<String> = self.steps.iter().map(Step::told).collect();
        let head = self.head();
        let rest: Vec<String> = self.otherwise.iter().map(Step::told).collect();
        let rest = if rest.is_empty() { String::new() } else { format!(" — {}: {}", t("otherwise"), rest.join("; ")) };
        if steps.is_empty() {
            format!("{head}: …{rest}")
        } else {
            format!("{head}: {}{rest}", steps.join("; "))
        }
    }

    /// The names of the objects the rule is about.
    pub fn objects(&self) -> Vec<&str> {
        let (a, b, _) = self.event.kind.needs();
        let mut names = Vec::new();
        if a == Some(Pick::Object) {
            names.push(self.event.a.as_str());
        }
        if b == Some(Pick::Object) {
            names.push(self.event.b.as_str());
        }
        names.extend(objects_of(&self.steps));
        names.extend(objects_of(&self.otherwise));
        names.extend(self.conditions().flat_map(|only| crate::script::objects_in(&only.expr)));
        names.retain(|name| !name.is_empty());
        names
    }

    /// What the script of the published page reads. A sound is given by its place in `sounds`.
    pub fn text(&self, sounds: &[String]) -> String {
        let written = |steps: &[Step]| steps_text(steps, sounds);
        let conditions: Vec<String> = self.conditions().map(Condition::text).collect();
        let only = if conditions.is_empty() { "null".to_string() } else { format!("[{}]", conditions.join(",")) };
        let otherwise = if self.otherwise.is_empty() { String::new() } else { format!(",\"o\":[{}]", written(&self.otherwise)) };
        format!("{{\"e\":{},\"c\":{only},\"s\":[{}]{otherwise}}}", self.event.text(), written(&self.steps))
    }
}

/// The names of numbers that a text shows: every `{name}` in it that is the name of one.
pub fn shown_in<'a>(content: &str, variables: &'a [Variable]) -> Vec<&'a str> {
    variables.iter().filter(|variable| content.contains(&format!("{{{}}}", variable.name))).map(|variable| variable.name.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> Rule {
        Rule {
            event: Event { kind: EventKind::Touch, a: "Elma".into(), b: "Sepet".into(), n: 0.0 },
            steps: vec![
                Step { kind: StepKind::Add, a: "Puan".into(), x: 1.0, ..Default::default() },
                Step { kind: StepKind::RandomTop, a: "Elma".into(), ..Default::default() },
                Step { kind: StepKind::Sound, a: "zil".into(), ..Default::default() },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn a_rule_reads_as_a_sentence_and_names_its_objects() {
        crate::i18n::set_language(crate::i18n::Language::English);
        let rule = rule();
        assert_eq!(rule.sentence(), "When Elma touches Sepet: increase Puan by 1; put Elma at the top, at random; play the sound zil");
        // "Only if" comes between the event and the steps; taking away reads as taking away; a copy is made at another object.
        let fire = Rule {
            event: Event { kind: EventKind::Key, a: " ".into(), b: String::new(), n: 0.0 },
            when: Some(Condition { number: "Mermi".into(), how: ">".into(), ..Default::default() }),
            steps: vec![Step { kind: StepKind::Spawn, a: "Ok".into(), b: "Gemi".into(), ..Default::default() }, Step { kind: StepKind::Add, a: "Mermi".into(), x: -1.0, ..Default::default() }],
            ..Default::default()
        };
        assert_eq!(fire.sentence(), "When the Space key is pressed, if Mermi is more than 0: make a copy of Ok at Gemi; decrease Mermi by 1");
        assert_eq!(fire.objects(), vec!["Ok", "Gemi"]);
        assert_eq!(fire.text(&[]), "{\"e\":[\"key\",\" \",\"\",0],\"c\":[[\"Mermi\",\">\",0]],\"s\":[[\"spawn\",\"Ok\",0,0,\"Gemi\"],[\"add\",\"Mermi\",-1,0,\"\"]]}");
        assert_eq!(rule.objects(), vec!["Elma", "Sepet", "Elma"]);
        let key = Event { kind: EventKind::Key, a: " ".into(), b: String::new(), n: 0.0 };
        assert_eq!(key.sentence(), "When the Space key is pressed");
        let number = Event { kind: EventKind::Number, a: "Can".into(), b: "<".into(), n: 1.0 };
        assert_eq!(number.sentence(), "When Can goes below 1");
        assert_eq!(Rule::new().sentence(), "When the film starts: …");
    }

    #[test]
    fn the_page_gets_a_rule_with_its_sound_by_number() {
        let text = rule().text(&["davul".to_string(), "zil".to_string()]);
        assert_eq!(text, "{\"e\":[\"touch\",\"Elma\",\"Sepet\",0],\"c\":null,\"s\":[[\"add\",\"Puan\",1,0,\"\"],[\"top\",\"Elma\",0,0,\"\"],[\"sound\",\"zil\",1,0,\"\"]]}");
    }

    #[test]
    fn every_kind_has_its_own_word_and_a_text_tells_its_numbers() {
        let mut words: Vec<&str> = EVENTS.iter().map(|kind| kind.key()).chain(STEPS.iter().map(|kind| kind.key())).collect();
        let count = words.len();
        words.sort();
        words.dedup();
        assert_eq!(words.len(), count);
        let numbers = [Variable { name: "Puan".into(), start: 0.0 }, Variable { name: "Can".into(), start: 3.0 }];
        assert_eq!(shown_in("Puan: {Puan}", &numbers), vec!["Puan"]);
        assert!(shown_in("Puan", &numbers).is_empty());
    }
}
