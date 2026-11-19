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

//! The written language of PisiAnimasyon: rules as text.
//!
//! It is not a second thing beside the lists of Rules and Numbers: it is the sentences
//! those lists write, typed by hand. A rule is its event on a line ending in a colon and
//! its steps under it, further in:
//!
//! ```text
//! Numbers:
//!     Score = 0
//!
//! When Apple touches Basket:
//!     increase Score by 1
//!     put Apple at the top, at random
//! ```
//!
//! The words are those of the interface's language, so the same game reads in Turkish
//! as `Elma nesnesi Sepet nesnesine değince:`. What the text can say and the lists
//! cannot: a number may be a sum (`Score * 2`, `Hero.x + 10`, `random(1, 6)`), a rule
//! may have several conditions joined by "and", and steps to take "otherwise".
//!
//! Reading is the writing turned round: every sentence [`crate::rules`] can write is
//! tried against a line, and the first that fits gives the rule. So what the program
//! writes it can always read back.

use crate::i18n::{t, tf};
use crate::rules::{choice_name, Command, Condition, Event, EventKind, Pick, Rule, Step, StepKind, Variable, EDGES, KEYS, WAYS};

// ---- Sums ---------------------------------------------------------------------------------

/// A piece of a sum.
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f32),
    /// A name: of a number, or of an object when a place of it is asked for.
    Name(String),
    Sign(char),
}

fn tokens(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut list = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        if c.is_whitespace() {
            at += 1;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(at + 1).is_some_and(|next| next.is_ascii_digit())) {
            let start = at;
            while at < chars.len() && (chars[at].is_ascii_digit() || chars[at] == '.') {
                at += 1;
            }
            let written: String = chars[start..at].iter().collect();
            list.push(Token::Number(written.parse::<f32>().map_err(|_| tf("{} is not a number", &[&written]))?));
        } else if c == '"' {
            // A name with spaces in it is written between quotation marks.
            let end = chars[at + 1..].iter().position(|c| *c == '"').ok_or_else(|| t("a quotation mark is not closed").to_string())?;
            list.push(Token::Name(chars[at + 1..at + 1 + end].iter().collect()));
            at += end + 2;
        } else if c.is_alphabetic() || c == '_' {
            let start = at;
            while at < chars.len() && (chars[at].is_alphanumeric() || chars[at] == '_') {
                at += 1;
            }
            list.push(Token::Name(chars[start..at].iter().collect()));
        } else if "+-*/().,".contains(c) {
            list.push(Token::Sign(c));
            at += 1;
        } else {
            return Err(tf("{} cannot be in a sum", &[&c]));
        }
    }
    Ok(list)
}

/// The words for a number picked at random, in either language.
const RANDOM: [&str; 2] = ["random", "rastgele"];

/// What a sum mentions: the numbers by name, and the objects whose place it asks for.
#[derive(Default)]
struct Mentions {
    numbers: Vec<String>,
    objects: Vec<String>,
}

/// Reads a sum through, to see that it is one. `sum := term (('+' | '-') term)*`, `term := part (('*' | '/') part)*`,
/// `part := number | name | name '.' ('x' | 'y') | random '(' sum ',' sum ')' | '(' sum ')' | '-' part`.
struct Reader<'a> {
    tokens: &'a [Token],
    at: usize,
    found: Mentions,
}

impl Reader<'_> {
    fn sign(&mut self, wanted: char) -> bool {
        let is = self.tokens.get(self.at) == Some(&Token::Sign(wanted));
        if is {
            self.at += 1;
        }
        is
    }

    fn sum(&mut self) -> Result<(), String> {
        self.term()?;
        while self.sign('+') || self.sign('-') {
            self.term()?;
        }
        Ok(())
    }

    fn term(&mut self) -> Result<(), String> {
        self.part()?;
        while self.sign('*') || self.sign('/') {
            self.part()?;
        }
        Ok(())
    }

    fn part(&mut self) -> Result<(), String> {
        match self.tokens.get(self.at).cloned() {
            Some(Token::Number(_)) => self.at += 1,
            Some(Token::Sign('-')) => {
                self.at += 1;
                self.part()?;
            }
            Some(Token::Sign('(')) => {
                self.at += 1;
                self.sum()?;
                if !self.sign(')') {
                    return Err(t("a bracket is not closed").to_string());
                }
            }
            Some(Token::Name(name)) => {
                self.at += 1;
                if RANDOM.contains(&name.as_str()) && self.sign('(') {
                    self.sum()?;
                    if !self.sign(',') {
                        return Err(t("a number at random is written random(from, to)").to_string());
                    }
                    self.sum()?;
                    if !self.sign(')') {
                        return Err(t("a bracket is not closed").to_string());
                    }
                } else if self.sign('.') {
                    match self.tokens.get(self.at) {
                        Some(Token::Name(side)) if side == "x" || side == "y" => self.at += 1,
                        _ => return Err(tf("after {}. comes x or y", &[&name])),
                    }
                    self.found.objects.push(name);
                } else {
                    self.found.numbers.push(name);
                }
            }
            _ => return Err(t("a number or a name is missing").to_string()),
        }
        Ok(())
    }
}

fn mentions(text: &str) -> Result<Mentions, String> {
    let tokens = tokens(text)?;
    let mut reader = Reader { tokens: &tokens, at: 0, found: Mentions::default() };
    reader.sum()?;
    if reader.at < tokens.len() {
        return Err(t("the sum does not end where it should").to_string());
    }
    Ok(reader.found)
}

/// The objects a sum asks the place of. A sum that cannot be read asks for none.
pub fn objects_in(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    let found = mentions(text).map(|found| found.objects).unwrap_or_default();
    // The names are given back as they stand in the text, so that they live as long as it does.
    found.iter().filter_map(|name| text.find(name.as_str()).map(|at| &text[at..at + name.len()])).collect()
}

/// A number as it is written in a rule: plain, or a sum to work out. The sum is kept as text.
#[derive(Clone, Debug, PartialEq)]
pub enum Amount {
    Plain(f32),
    Sum(String),
}

/// Reads what stands where a number belongs. A comma is taken for a decimal point in a plain number.
pub fn amount(text: &str, numbers: &[String]) -> Result<Amount, String> {
    let text = text.trim();
    if let Ok(plain) = text.replace(',', ".").parse::<f32>() {
        if plain.is_finite() {
            return Ok(Amount::Plain(plain));
        }
    }
    let found = mentions(text)?;
    if let Some(unknown) = found.numbers.iter().find(|name| !numbers.contains(name)) {
        return Err(tf("there is no number called {}", &[unknown]));
    }
    Ok(Amount::Sum(text.to_string()))
}

// ---- Fitting a sentence to a line ---------------------------------------------------------

/// Where `mark` next stands in `line` from `from` on, outside brackets and quotation marks.
fn find_outside(line: &str, from: usize, mark: &str) -> Option<usize> {
    let (mut depth, mut quoted) = (0i32, false);
    for (at, c) in line[from..].char_indices() {
        let at = from + at;
        if !quoted && depth == 0 && line[at..].starts_with(mark) {
            return Some(at);
        }
        match c {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            _ => {}
        }
    }
    None
}

/// A text cut at each `mark` that stands outside brackets and quotation marks.
fn split_outside<'a>(text: &'a str, mark: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut from = 0;
    while let Some(at) = find_outside(text, from, mark) {
        parts.push(&text[from..at]);
        from = at + mark.len();
    }
    parts.push(&text[from..]);
    parts
}

/// What stands in the gaps of a sentence, if the line is that sentence: `pattern` has `{}` for each gap.
/// A gap takes as little as it can, and the last one what is left.
fn fit(pattern: &str, line: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = pattern.split("{}").collect();
    let mut rest = line.strip_prefix(parts[0])?;
    let mut gaps = Vec::new();
    for (index, part) in parts.iter().enumerate().skip(1) {
        let last = index == parts.len() - 1;
        let end = if last {
            // The last words must end the line.
            if !rest.ends_with(part) {
                return None;
            }
            rest.len() - part.len()
        } else if part.is_empty() {
            return None;
        } else {
            find_outside(rest, 0, part)?
        };
        let gap = rest[..end].trim();
        if gap.is_empty() {
            return None;
        }
        gaps.push(gap.to_string());
        rest = &rest[end + part.len()..];
    }
    (parts.len() > 1 || rest.is_empty()).then_some(gaps)
}

/// A line with its first letter in the other case: "Hide Apple" for "hide Apple".
fn recased(line: &str) -> String {
    let mut chars = line.chars();
    match chars.next() {
        Some(first) if first.is_lowercase() => first.to_uppercase().chain(chars).collect(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// What a line was found to lack, to be told to its writer.
type Fault = String;

/// What the reading knows: the numbers there are, and what it has found doubtful.
struct Known<'a> {
    numbers: &'a [String],
    /// The commands there are, each with how many values it takes.
    commands: &'a [(String, usize)],
    /// The names of the values of the command being read, which are numbers inside it.
    params: &'a [String],
}

impl Known<'_> {
    /// Every name that counts as a number here: the numbers of the film, and the values of the command being read.
    fn all_numbers(&self) -> Vec<String> {
        self.numbers.iter().chain(self.params.iter()).cloned().collect()
    }

    fn number(&self, name: &str) -> Result<String, Fault> {
        if self.numbers.iter().chain(self.params.iter()).any(|known| known == name) {
            Ok(name.to_string())
        } else {
            Err(tf("there is no number called {}", &[&name]))
        }
    }
}

/// The value a choice is kept as, from its name in the interface's language.
fn choice(pick: Pick, list: &[(&str, &str)], shown: &str) -> Option<String> {
    list.iter().find(|(key, _)| choice_name(pick, key).to_lowercase() == shown.to_lowercase()).map(|(key, _)| key.to_string())
}

/// The sentences an event can be, each with the kind it gives and, for a number held against a value, which way.
const EVENT_FORMS: [(&str, EventKind, &str); 12] = [
    ("When the film starts", EventKind::Start, ""),
    ("When a scene starts", EventKind::Scene, ""),
    ("When the {} key is pressed", EventKind::Key, ""),
    ("When the {} key is released", EventKind::KeyUp, ""),
    ("When {} is clicked", EventKind::Click, ""),
    ("When {} touches {} of the stage", EventKind::Edge, ""),
    ("When {} touches {}", EventKind::Touch, ""),
    ("Every {} seconds", EventKind::Every, ""),
    ("After {} seconds", EventKind::After, ""),
    ("When {} goes above {}", EventKind::Number, ">"),
    ("When {} goes below {}", EventKind::Number, "<"),
    ("When {} becomes {}", EventKind::Number, "="),
];

fn event(line: &str, known: &Known) -> Result<Event, Fault> {
    let mut fault = None;
    for (pattern, kind, how) in EVENT_FORMS {
        let Some(gaps) = fit(t(pattern), line).or_else(|| fit(t(pattern), &recased(line))) else { continue };
        let plain = |text: &str| text.replace(',', ".").parse::<f32>().ok().filter(|n| n.is_finite());
        let made = match kind {
            EventKind::Start | EventKind::Scene => Some(Event::new(kind)),
            EventKind::Key | EventKind::KeyUp => choice(Pick::Key, &KEYS, &gaps[0]).map(|key| Event { kind, a: key, ..Default::default() }),
            EventKind::Click => Some(Event { kind, a: gaps[0].clone(), ..Default::default() }),
            EventKind::Touch => Some(Event { kind, a: gaps[0].clone(), b: gaps[1].clone(), ..Default::default() }),
            EventKind::Edge => choice(Pick::Edge, &EDGES, &gaps[1]).map(|edge| Event { kind, a: gaps[0].clone(), b: edge, ..Default::default() }),
            EventKind::Every | EventKind::After => plain(&gaps[0]).map(|n| Event { kind, n, ..Default::default() }),
            EventKind::Number => match (known.number(&gaps[0]), plain(&gaps[1])) {
                (Ok(name), Some(n)) => Some(Event { kind, a: name, b: how.to_string(), n }),
                (Err(why), Some(_)) => {
                    fault = Some(why);
                    None
                }
                _ => None,
            },
        };
        if let Some(made) = made {
            return Ok(made);
        }
    }
    Err(fault.unwrap_or_else(|| t("this is not an event I know").to_string()))
}

const CONDITION_FORMS: [(&str, &str); 3] = [("{} is more than {}", ">"), ("{} is less than {}", "<"), ("{} is {}", "=")];

fn condition(text: &str, known: &Known) -> Result<Condition, Fault> {
    let mut fault = None;
    for (pattern, how) in CONDITION_FORMS {
        let Some(gaps) = fit(t(pattern), text) else { continue };
        match (known.number(&gaps[0]), amount(&gaps[1], &known.all_numbers())) {
            (Ok(number), Ok(Amount::Plain(value))) => return Ok(Condition { number, how: how.to_string(), value, ..Default::default() }),
            (Ok(number), Ok(Amount::Sum(expr))) => return Ok(Condition { number, how: how.to_string(), expr, ..Default::default() }),
            (Err(why), _) | (_, Err(why)) => fault = fault.or(Some(why)),
        }
    }
    Err(fault.unwrap_or_else(|| t("this is not a condition I know").to_string()))
}

/// The first line of a rule: its event, and after ", if" its conditions joined by "and".
fn head(line: &str, known: &Known) -> Result<(Event, Vec<Condition>), Fault> {
    // The line is split at each place the words between event and conditions stand, until both halves read.
    // Only when no split reads is the whole line taken for an event, since an event's last name would
    // otherwise swallow the conditions.
    let pattern = t("{}, if {}");
    let between = pattern.split("{}").nth(1).unwrap_or(", ");
    let mut from = 0;
    let mut fault: Option<Fault> = None;
    while let Some(at) = find_outside(line, from, between) {
        from = at + between.len();
        let Ok(found) = event(&line[..at], known) else { continue };
        match conditions(&line[at + between.len()..], known) {
            Ok(conditions) => return Ok((found, conditions)),
            Err(why) => fault = Some(why),
        }
    }
    match (event(line, known), fault) {
        (Ok(found), None) => Ok((found, Vec::new())),
        // A line that reads as an event only by taking its conditions for a name has conditions that do not read.
        (Ok(_), Some(why)) | (Err(_), Some(why)) => Err(why),
        (Err(why), None) => Err(why),
    }
}

/// What stands in a gap of a step's sentence.
#[derive(Clone, Copy, PartialEq)]
enum Gap {
    A,
    B,
    X,
    Y,
    /// The values a command is done with, with commas between them.
    Values,
}

/// The sentences a step can be: the kind each gives, what its gaps hold in the order they stand, and
/// whether the first number is taken away ("decrease", "slower").
const STEP_FORMS: [(&str, StepKind, &[Gap], bool); 42] = [
    ("make the camera follow {}", StepKind::Camera, &[Gap::A], false),
    ("let the camera stand still", StepKind::CameraStop, &[], false),
    ("make {} go {} at {}", StepKind::Go, &[Gap::A, Gap::B, Gap::X], false),
    ("make {} slow down and stop", StepKind::Slow, &[Gap::A], false),
    ("make {} stop", StepKind::Halt, &[Gap::A], false),
    ("attach {} to {}", StepKind::Attach, &[Gap::A, Gap::B], false),
    ("let {} go", StepKind::Detach, &[Gap::A], false),
    ("make {} spin {} turns a second", StepKind::Whirl, &[Gap::A, Gap::X], false),
    ("turn {} by {} degrees", StepKind::Turn, &[Gap::A, Gap::X], false),
    ("make {} {} percent of its size", StepKind::Resize, &[Gap::A, Gap::X], false),
    ("make {} {} percent opaque", StepKind::Opacity, &[Gap::A, Gap::X], false),
    ("give {} the behavior {}", StepKind::Give, &[Gap::A, Gap::B], false),
    ("take from {} the behavior {}", StepKind::Take, &[Gap::A, Gap::B], false),
    ("send {} after {} at {}", StepKind::Chase, &[Gap::A, Gap::B, Gap::X], false),
    ("do the command {} with {}", StepKind::Call, &[Gap::A, Gap::Values], false),
    ("do the command {}", StepKind::Call, &[Gap::A], false),
    ("move {} by {} right and {} down", StepKind::Move, &[Gap::A, Gap::X, Gap::Y], false),
    ("put {} somewhere at random", StepKind::Random, &[Gap::A], false),
    ("put {} at the top, at random", StepKind::RandomTop, &[Gap::A], false),
    ("put {} at {}, {}", StepKind::GoTo, &[Gap::A, Gap::X, Gap::Y], false),
    ("show {}", StepKind::Show, &[Gap::A], false),
    ("hide {}", StepKind::Hide, &[Gap::A], false),
    ("make {} jump {} high", StepKind::Jump, &[Gap::A, Gap::X], false),
    ("decrease {} by {}", StepKind::Add, &[Gap::A, Gap::X], true),
    ("increase {} by {}", StepKind::Add, &[Gap::A, Gap::X], false),
    ("make a copy of {} at {}", StepKind::Spawn, &[Gap::A, Gap::B], false),
    ("remove {}", StepKind::Remove, &[Gap::A], false),
    ("make {} blink for a moment", StepKind::Flash, &[Gap::A], false),
    ("make {} slower by {} percent", StepKind::Faster, &[Gap::A, Gap::X], true),
    ("make {} faster by {} percent", StepKind::Faster, &[Gap::A, Gap::X], false),
    ("set {} at random from {} to {}", StepKind::RandomNumber, &[Gap::A, Gap::X, Gap::Y], false),
    ("set {} to {}", StepKind::Set, &[Gap::A, Gap::X], false),
    ("wait {} seconds", StepKind::Wait, &[Gap::X], false),
    ("stop the film", StepKind::StopFilm, &[], false),
    ("play the film", StepKind::PlayFilm, &[], false),
    ("go to frame {} and play", StepKind::GotoFrame, &[Gap::X], false),
    ("go to frame {} and stop", StepKind::GotoFrameStop, &[Gap::X], false),
    ("go to the next scene", StepKind::NextScene, &[], false),
    ("go to scene {}", StepKind::GotoScene, &[Gap::X], false),
    ("play the sound {}", StepKind::Sound, &[Gap::A], false),
    ("stop everything", StepKind::StopAll, &[], false),
    ("start again", StepKind::Restart, &[], false),
];

fn step(line: &str, known: &Known) -> Result<Step, Fault> {
    let mut fault = None;
    'forms: for (pattern, kind, gaps, taken_away) in STEP_FORMS {
        let Some(found) = fit(t(pattern), line).or_else(|| fit(t(pattern), &recased(line))) else { continue };
        let mut made = Step { kind, ..Default::default() };
        for (gap, text) in gaps.iter().zip(&found) {
            match gap {
                Gap::A if kind.needs().0 == Some(Pick::Variable) => match known.number(text) {
                    Ok(name) => made.a = name,
                    Err(why) => {
                        fault = fault.or(Some(why));
                        continue 'forms;
                    }
                },
                Gap::A if kind == StepKind::Call => {
                    // The command must be there, and be given as many values as it takes.
                    let given = if gaps.contains(&Gap::Values) { found.get(1).map_or(0, |values| split_outside(values, ",").len()) } else { 0 };
                    match known.commands.iter().find(|(name, _)| name == text) {
                        Some((_, takes)) if *takes == given => made.a = text.clone(),
                        Some((_, takes)) => {
                            fault = fault.or(Some(tf("the command {} takes {} values, and is given {}", &[text, takes, &given])));
                            continue 'forms;
                        }
                        None => {
                            fault = fault.or(Some(tf("there is no command called {}", &[text])));
                            continue 'forms;
                        }
                    }
                }
                Gap::Values => made.args = split_outside(text, ",").into_iter().map(|value| value.trim().to_string()).collect(),
                Gap::A => made.a = text.clone(),
                // A behavior is written by its name in the menu and kept by its own word.
                Gap::B if kind.needs().1 == Some(Pick::Behavior) => match crate::behavior::KINDS.iter().find(|known| t(known.name()).to_lowercase() == text.to_lowercase()) {
                    Some(known) => made.b = known.key().to_string(),
                    None => {
                        fault = fault.or(Some(tf("there is no behavior called {}", &[text])));
                        continue 'forms;
                    }
                },
                Gap::B if kind.needs().1 == Some(Pick::Way) => match choice(Pick::Way, &WAYS, text) {
                    Some(way) => made.b = way,
                    None => {
                        fault = fault.or(Some(tf("{} is not a way to go: it is to the right, to the left, upwards or downwards", &[text])));
                        continue 'forms;
                    }
                },
                Gap::B => made.b = text.clone(),
                Gap::X | Gap::Y => {
                    let read = match amount(text, &known.all_numbers()) {
                        Ok(read) => read,
                        Err(why) => {
                            fault = fault.or(Some(why));
                            continue 'forms;
                        }
                    };
                    let turn = taken_away && *gap == Gap::X;
                    match (read, *gap) {
                        (Amount::Plain(n), Gap::X) => made.x = if turn { -n } else { n },
                        (Amount::Plain(n), _) => made.y = n,
                        (Amount::Sum(sum), Gap::X) => made.ex = if turn { format!("-({sum})") } else { sum },
                        (Amount::Sum(sum), _) => made.ey = sum,
                    }
                }
            }
        }
        return Ok(made);
    }
    Err(fault.unwrap_or_else(|| t("this is not a step I know").to_string()))
}

/// What a sentence opens with: its own words, or a gap, and then what the gap holds. A sentence is offered
/// for what has been typed only if the first word typed can be what it opens with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opens {
    Words,
    /// The name of an object, a number, a command, a sound or a key.
    Name(Pick),
    /// A number or a sum.
    Amount,
    /// A condition, which begins with the name of a number.
    Condition,
    /// A name that is being given: that of a new command.
    NewName,
}

/// The sentences that can be written at a place, each with a question mark where a name or a number goes,
/// and with what it opens with: for offering them as they are typed. At the edge of the text these are the
/// events and the headings; further in, the steps.
pub fn offers(edge: bool) -> Vec<(String, Opens)> {
    let shown = |pattern: &str| t(pattern).replace("{}", "?");
    let gap_first = |pattern: &str| t(pattern).starts_with("{}");
    if edge {
        let events = EVENT_FORMS.iter().map(|(pattern, kind, _)| {
            let opens = match kind {
                _ if !gap_first(pattern) => Opens::Words,
                EventKind::Key | EventKind::KeyUp => Opens::Name(Pick::Key),
                EventKind::Every | EventKind::After => Opens::Amount,
                EventKind::Number => Opens::Name(Pick::Variable),
                _ => Opens::Name(Pick::Object),
            };
            (format!("{}:", shown(pattern)), opens)
        });
        let command = (format!("{}:", shown("The command {}")), if gap_first("The command {}") { Opens::NewName } else { Opens::Words });
        events.chain([(format!("{}:", t("Numbers")), Opens::Words), command, (format!("{}:", t("otherwise")), Opens::Words)]).collect()
    } else {
        let blocks = BLOCK_FORMS.iter().map(|(pattern, kind)| {
            let opens = match kind {
                _ if !gap_first(pattern) => Opens::Words,
                StepKind::Repeat => Opens::Amount,
                _ => Opens::Condition,
            };
            (format!("{}:", shown(pattern)), opens)
        });
        let steps = STEP_FORMS.iter().map(|(pattern, kind, gaps, _)| {
            let opens = match (gap_first(pattern), gaps.first(), kind.needs().0) {
                (false, _, _) => Opens::Words,
                (true, Some(Gap::A), Some(pick)) => Opens::Name(pick),
                (true, Some(Gap::B), _) => Opens::Name(Pick::Object),
                _ => Opens::Amount,
            };
            (shown(pattern), opens)
        });
        blocks.chain(steps).chain([(format!("{}:", t("otherwise")), Opens::Words)]).collect()
    }
}

/// The words that come just before the name of a behavior, in the sentences that give one and take one.
pub fn behavior_leads() -> Vec<String> {
    STEP_FORMS
        .iter()
        .filter(|(_, kind, _, _)| kind.needs().1 == Some(Pick::Behavior))
        .filter_map(|(pattern, _, _, _)| t(pattern).split("{}").nth(1).map(str::to_string))
        .filter(|lead| !lead.trim().is_empty())
        .collect()
}

/// The names of the keys an event can wait for, as they are written.
pub fn key_names() -> Vec<String> {
    KEYS.iter().map(|(key, _)| choice_name(Pick::Key, key)).collect()
}

/// The numbers and the commands a text sets out, by name, whether or not the rest of it reads.
pub fn declared(text: &str) -> (Vec<String>, Vec<String>) {
    let (mut numbers, mut commands, mut in_numbers) = (Vec::new(), Vec::new(), false);
    for line in lines_of(text) {
        if line.indent == 0 {
            let title = line.text.trim_end_matches(':').trim();
            in_numbers = title.to_lowercase() == t("Numbers").to_lowercase();
            commands.extend(command_head(title).filter(|_| !in_numbers).map(|(name, _)| name));
        } else if in_numbers {
            numbers.extend(line.text.split_once('=').map(|(name, _)| name.trim().to_string()).filter(|name| !name.is_empty()));
        }
    }
    (numbers, commands)
}

/// Every sentence an event can be, in the interface's language, with a question mark where a name or a number goes.
pub fn event_sentences() -> Vec<String> {
    EVENT_FORMS.iter().map(|(pattern, _, _)| t(pattern).replace("{}", "?")).collect()
}

/// Every sentence a step can be, likewise.
pub fn step_sentences() -> Vec<String> {
    let blocks = BLOCK_FORMS.iter().map(|(pattern, _)| format!("{}:", t(pattern).replace("{}", "?")));
    blocks.chain(STEP_FORMS.iter().map(|(pattern, _, _, _)| t(pattern).replace("{}", "?"))).collect()
}

// ---- The whole text ----------------------------------------------------------------------

/// The conditions of a line, joined by "and".
fn conditions(text: &str, known: &Known) -> Result<Vec<Condition>, Fault> {
    let (mut rest, joiner) = (text, t(" and "));
    let mut found = Vec::new();
    loop {
        let end = find_outside(rest, 0, joiner);
        found.push(condition(rest[..end.unwrap_or(rest.len())].trim(), known)?);
        match end {
            Some(end) => rest = &rest[end + joiner.len()..],
            None => return Ok(found),
        }
    }
}

/// The sentences that open a step holding steps of its own. "repeat while" is tried before "repeat ... times"
/// can be mistaken for it, and after it where the words run the other way round.
const BLOCK_FORMS: [(&str, StepKind); 3] = [("repeat {} times", StepKind::Repeat), ("repeat while {}", StepKind::While), ("if {}", StepKind::If)];

/// A line that ends in a colon inside a rule: the step it opens, its body yet to be read.
fn block(title: &str, known: &Known) -> Result<Step, Fault> {
    let mut fault = None;
    for (pattern, kind) in BLOCK_FORMS {
        let Some(gaps) = fit(t(pattern), title).or_else(|| fit(t(pattern), &recased(title))) else { continue };
        if kind == StepKind::Repeat {
            match amount(&gaps[0], &known.all_numbers()) {
                Ok(Amount::Plain(x)) => return Ok(Step { kind, x, ..Default::default() }),
                Ok(Amount::Sum(ex)) => return Ok(Step { kind, ex, ..Default::default() }),
                Err(why) => fault = fault.or(Some(why)),
            }
        } else {
            match conditions(&gaps[0], known) {
                Ok(only) => return Ok(Step { kind, only, ..Default::default() }),
                Err(why) => fault = fault.or(Some(why)),
            }
        }
    }
    Err(fault.unwrap_or_else(|| t("a line that ends with a colon here is \"repeat ... times\", \"repeat while ...\" or \"if ...\"").to_string()))
}

/// What is wrong on a line of the text, counted from 1.
#[derive(Clone, Debug, PartialEq)]
pub struct Mistake {
    pub line: usize,
    pub what: String,
}

/// The rules, numbers and commands a text says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub variables: Vec<Variable>,
    pub commands: Vec<Command>,
    pub rules: Vec<Rule>,
}

/// How far in a step is written, for each level it lies under.
const INDENT: &str = "    ";

fn write_steps(text: &mut String, steps: &[Step], depth: usize) {
    let lead = INDENT.repeat(depth);
    for step in steps {
        if step.is_block() {
            text.push_str(&format!("{lead}{}:\n", step.sentence()));
            write_steps(text, &step.body, depth + 1);
            if !step.otherwise.is_empty() {
                text.push_str(&format!("{lead}{}:\n", t("otherwise")));
                write_steps(text, &step.otherwise, depth + 1);
            }
        } else {
            text.push_str(&format!("{lead}{}\n", step.sentence()));
        }
    }
}

/// The numbers, the commands and the rules as text, in the language of the interface.
pub fn write(variables: &[Variable], commands: &[Command], rules: &[Rule]) -> String {
    let mut text = String::new();
    if !variables.is_empty() {
        text.push_str(&format!("{}:\n", t("Numbers")));
        for variable in variables {
            text.push_str(&format!("{INDENT}{} = {}\n", variable.name, variable.start));
        }
        text.push('\n');
    }
    for command in commands {
        let head = if command.params.is_empty() { tf("The command {}", &[&command.name]) } else { tf("The command {} with {}", &[&command.name, &command.params.join(", ")]) };
        text.push_str(&format!("{head}:\n"));
        write_steps(&mut text, &command.steps, 1);
        text.push('\n');
    }
    for rule in rules {
        text.push_str(&format!("{}:\n", rule.head()));
        write_steps(&mut text, &rule.steps, 1);
        if !rule.otherwise.is_empty() {
            text.push_str(&format!("{}:\n", t("otherwise")));
            write_steps(&mut text, &rule.otherwise, 1);
        }
        text.push('\n');
    }
    text
}

/// A line of the text that says something: its number, how far in it is written, and what it says.
struct Line<'a> {
    number: usize,
    indent: usize,
    text: &'a str,
}

fn lines_of(text: &str) -> Vec<Line<'_>> {
    text.lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let said = raw.trim();
            if said.is_empty() || said.starts_with('#') || said.starts_with("//") {
                return None;
            }
            // A tab counts for a step's worth of spaces.
            let indent = raw.chars().take_while(|c| c.is_whitespace()).map(|c| if c == '\t' { INDENT.len() } else { 1 }).sum();
            Some(Line { number: index + 1, indent, text: said })
        })
        .collect()
}

/// The command a line at the edge defines, if it defines one: its name, and the names of the values it takes.
/// Each name is one word.
fn command_head(title: &str) -> Option<(String, Vec<String>)> {
    let one_word = |name: &str| !name.is_empty() && !name.contains(char::is_whitespace);
    let with = t("The command {} with {}");
    if let Some(gaps) = fit(with, title).or_else(|| fit(with, &recased(title))) {
        let params: Vec<String> = split_outside(&gaps[1], ",").into_iter().map(|name| name.trim().to_string()).collect();
        if one_word(&gaps[0]) && params.iter().all(|name| one_word(name)) {
            return Some((gaps[0].clone(), params));
        }
    }
    let gaps = fit(t("The command {}"), title).or_else(|| fit(t("The command {}"), &recased(title)))?;
    one_word(&gaps[0]).then(|| (gaps[0].clone(), Vec::new()))
}

/// Reads the steps written further in than `parent`, from line `at` on, each with the steps it holds.
fn steps_under(lines: &[Line], at: &mut usize, parent: usize, known: &Known, mistakes: &mut Vec<Mistake>) -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    let Some(level) = lines.get(*at).map(|line| line.indent).filter(|indent| *indent > parent) else { return steps };
    while let Some(line) = lines.get(*at).filter(|line| line.indent > parent) {
        *at += 1;
        if line.indent != level {
            mistakes.push(Mistake { line: line.number, what: t("this line is not as far in as the lines it belongs with").to_string() });
            continue;
        }
        let Some(title) = line.text.strip_suffix(':').map(str::trim) else {
            match step(line.text, known) {
                Ok(step) => steps.push(step),
                Err(what) => mistakes.push(Mistake { line: line.number, what }),
            }
            continue;
        };
        if title.to_lowercase() == t("otherwise") {
            // "otherwise" belongs to the "if" just above it.
            let rest = steps_under(lines, at, level, known, mistakes);
            match steps.last_mut().filter(|last| last.kind == StepKind::If && last.otherwise.is_empty()) {
                Some(last) => last.otherwise = rest,
                None => mistakes.push(Mistake { line: line.number, what: t("\"otherwise\" comes after the steps of a rule or of an \"if\"").to_string() }),
            }
            continue;
        }
        let opened = block(title, known);
        let body = steps_under(lines, at, level, known, mistakes);
        match opened {
            Ok(mut step) => {
                if body.is_empty() {
                    mistakes.push(Mistake { line: line.number, what: t("the steps under this line are missing: they are written further in").to_string() });
                }
                step.body = body;
                steps.push(step);
            }
            Err(what) => mistakes.push(Mistake { line: line.number, what }),
        }
    }
    steps
}

/// Reads a text. Every line that cannot be read is told of, and nothing is taken unless all of it reads.
pub fn read(text: &str) -> Result<Read, Vec<Mistake>> {
    let mut found = Read::default();
    let mut mistakes = Vec::new();
    let lines = lines_of(text);
    let is_numbers = |title: &str| title.to_lowercase() == t("Numbers").to_lowercase();
    // The numbers and the names of the commands are gathered first, so that a rule may speak of one written further down.
    let mut names = Vec::new();
    let mut in_numbers = false;
    for line in &lines {
        if line.indent == 0 {
            let title = line.text.trim_end_matches(':').trim();
            in_numbers = is_numbers(title);
            if let Some((name, params)) = command_head(title).filter(|_| !in_numbers) {
                if names.iter().any(|(known, _)| *known == name) {
                    mistakes.push(Mistake { line: line.number, what: tf("the command {} is written twice", &[&name]) });
                }
                names.push((name, params.len()));
            }
        } else if in_numbers {
            match line.text.split_once('=') {
                Some((name, start)) if !name.trim().is_empty() && !name.trim().contains(char::is_whitespace) => match start.trim().replace(',', ".").parse::<f32>() {
                    Ok(start) if !found.variables.iter().any(|v| v.name == name.trim()) => found.variables.push(Variable { name: name.trim().to_string(), start }),
                    Ok(_) => mistakes.push(Mistake { line: line.number, what: tf("the number {} is written twice", &[&name.trim()]) }),
                    Err(_) => mistakes.push(Mistake { line: line.number, what: tf("{} is not a number", &[&start.trim()]) }),
                },
                _ => mistakes.push(Mistake { line: line.number, what: t("a number is written as: Name = 0, the name one word").to_string() }),
            }
        }
    }
    let numbers: Vec<String> = found.variables.iter().map(|v| v.name.clone()).collect();
    let known = Known { numbers: &numbers, commands: &names, params: &[] };
    let mut at = 0;
    // Whether the last thing read at the edge was a rule, which an "otherwise" may follow.
    let mut after_rule = false;
    while let Some(line) = lines.get(at) {
        at += 1;
        if line.indent > 0 {
            // Steps under a line that could not be read are passed over: the line has been told of.
            if mistakes.is_empty() {
                mistakes.push(Mistake { line: line.number, what: t("a step comes under an event").to_string() });
            }
            continue;
        }
        let Some(title) = line.text.strip_suffix(':').map(str::trim) else {
            mistakes.push(Mistake { line: line.number, what: t("a line that starts at the edge is an event and ends with a colon; steps are written further in").to_string() });
            after_rule = false;
            while lines.get(at).is_some_and(|next| next.indent > 0) {
                at += 1;
            }
            continue;
        };
        if is_numbers(title) {
            while lines.get(at).is_some_and(|next| next.indent > 0) {
                at += 1;
            }
            after_rule = false;
        } else if title.to_lowercase() == t("otherwise") {
            let rest = steps_under(&lines, &mut at, 0, &known, &mut mistakes);
            match found.rules.last_mut().filter(|rule| after_rule && rule.otherwise.is_empty()) {
                Some(rule) => rule.otherwise = rest,
                None => mistakes.push(Mistake { line: line.number, what: t("\"otherwise\" comes after the steps of a rule or of an \"if\"").to_string() }),
            }
            after_rule = false;
        } else if let Some((name, params)) = command_head(title) {
            // Inside the command the names of its values are numbers to reckon with.
            let inside = Known { numbers: &numbers, commands: &names, params: &params };
            let steps = steps_under(&lines, &mut at, 0, &inside, &mut mistakes);
            if !found.commands.iter().any(|c| c.name == name) {
                found.commands.push(Command { name, params, steps });
            }
            after_rule = false;
        } else {
            let opened = head(title, &known);
            let steps = steps_under(&lines, &mut at, 0, &known, &mut mistakes);
            match opened {
                Ok((event, mut conditions)) => {
                    let when = if conditions.is_empty() { None } else { Some(conditions.remove(0)) };
                    found.rules.push(Rule { event, when, also: conditions, steps, ..Default::default() });
                    after_rule = true;
                }
                Err(what) => {
                    mistakes.push(Mistake { line: line.number, what });
                    after_rule = false;
                }
            }
        }
    }
    if mistakes.is_empty() {
        Ok(found)
    } else {
        mistakes.sort_by_key(|mistake| mistake.line);
        Err(mistakes)
    }
}

/// How many letters must be changed, put in or taken out to make one word of another.
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.to_lowercase().chars().collect(), b.to_lowercase().chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut last = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let now = row[j + 1];
            row[j + 1] = if x == y { last } else { 1 + last.min(now).min(row[j]) };
            last = now;
        }
    }
    row[b.len()]
}

/// The name that was most likely meant by one that is not there: one that differs from it by a letter or
/// two, or only in its capitals.
pub fn meant<'a>(name: &str, known: &'a [String]) -> Option<&'a String> {
    known.iter().map(|other| (distance(name, other), other)).filter(|(far, other)| *far <= 2 && *far < other.chars().count()).min_by_key(|(far, _)| *far).map(|(_, other)| other)
}

/// The names in the rules that nothing in the document answers to: objects and sounds. They do no
/// harm, but a rule about a thing that is not there does nothing.
pub fn strangers(rules: &[Rule], commands: &[Command], objects: &[String], sounds: &[String]) -> Vec<String> {
    // Inside a command, the name of a value it takes stands for whatever it is given: it is no object of the document.
    let named = rules.iter().flat_map(|rule| rule.objects()).chain(commands.iter().flat_map(|command| crate::rules::objects_of(&command.steps).into_iter().filter(|name| !command.params.iter().any(|param| param == name))));
    let mut missing: Vec<String> = named.filter(|name| !objects.iter().any(|known| known == name)).map(str::to_string).collect();
    let lists = rules.iter().flat_map(|rule| [&rule.steps, &rule.otherwise]).chain(commands.iter().map(|command| &command.steps));
    let played = lists.flat_map(|steps| crate::rules::every_step(steps)).filter(|step| step.kind == StepKind::Sound).map(|step| step.a.clone());
    missing.extend(played.filter(|name| !sounds.contains(name)));
    missing.sort();
    missing.dedup();
    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{set_language, Language};

    #[test]
    fn a_sum_is_read_or_told_what_is_wrong_with_it() {
        set_language(Language::English);
        let numbers = vec!["Score".to_string(), "Lives".to_string()];
        assert_eq!(amount("12", &numbers), Ok(Amount::Plain(12.0)));
        assert_eq!(amount("0,5", &numbers), Ok(Amount::Plain(0.5)));
        assert_eq!(amount("Score * 2 + 1", &numbers), Ok(Amount::Sum("Score * 2 + 1".to_string())));
        assert_eq!(amount("Hero.x + 10", &numbers), Ok(Amount::Sum("Hero.x + 10".to_string())));
        assert_eq!(amount("\"Giant Rat\".y - (Lives + 1)", &numbers), Ok(Amount::Sum("\"Giant Rat\".y - (Lives + 1)".to_string())));
        assert_eq!(amount("random(1, 6)", &numbers), Ok(Amount::Sum("random(1, 6)".to_string())));
        assert_eq!(amount("Points + 1", &numbers), Err("there is no number called Points".to_string()));
        assert_eq!(amount("Score +", &numbers), Err("a number or a name is missing".to_string()));
        assert_eq!(amount("(Score", &numbers), Err("a bracket is not closed".to_string()));
        assert_eq!(amount("Hero.width", &numbers), Err("after Hero. comes x or y".to_string()));
        assert_eq!(objects_in("Hero.x + \"Giant Rat\".y - Score"), vec!["Hero", "Giant Rat"]);
    }

    #[test]
    fn a_sentence_fits_a_line_or_does_not() {
        assert_eq!(fit("When {} touches {}", "When Apple touches Basket"), Some(vec!["Apple".to_string(), "Basket".to_string()]));
        assert_eq!(fit("put {} at {}, {}", "put Hero at random(1, 6), Score * 2"), Some(vec!["Hero".to_string(), "random(1, 6)".to_string(), "Score * 2".to_string()]));
        assert_eq!(fit("stop the film", "stop the film"), Some(Vec::new()));
        assert_eq!(fit("stop the film", "stop the film now"), None);
        assert_eq!(fit("show {}", "hide Apple"), None);
    }

    const GAME: &str = "\
# A game in a few lines.
Numbers:
    Score = 0
    Lives = 3

When Apple touches Basket:
    increase Score by 1
    put Apple at the top, at random

When Hero touches Door, if Score is more than 2 and Lives is more than 0:
    play the sound fanfare
    wait 1 seconds
    go to scene 2
otherwise:
    put Hero at Hero.x - 40, 262

When the Space key is pressed:
    make a copy of Bullet at Ship
    set Score to Score * 2 + random(1, 6)
";

    #[test]
    fn a_text_is_read_into_rules_and_written_back_the_same() {
        set_language(Language::English);
        let found = read(GAME).unwrap();
        assert_eq!(found.variables, vec![Variable { name: "Score".into(), start: 0.0 }, Variable { name: "Lives".into(), start: 3.0 }]);
        assert_eq!(found.rules.len(), 3);
        // The door: two conditions, three steps, and one to take otherwise, with a sum that asks where the hero is.
        let door = &found.rules[1];
        assert_eq!((door.event.kind, door.event.a.as_str(), door.event.b.as_str()), (EventKind::Touch, "Hero", "Door"));
        assert_eq!(door.conditions().map(|only| (only.number.as_str(), only.how.as_str(), only.value)).collect::<Vec<_>>(), vec![("Score", ">", 2.0), ("Lives", ">", 0.0)]);
        assert_eq!(door.steps.iter().map(|step| step.kind).collect::<Vec<_>>(), vec![StepKind::Sound, StepKind::Wait, StepKind::GotoScene]);
        assert_eq!((door.otherwise[0].kind, door.otherwise[0].ex.as_str(), door.otherwise[0].y), (StepKind::GoTo, "Hero.x - 40", 262.0));
        assert_eq!(found.rules[2].steps[1].ex, "Score * 2 + random(1, 6)");
        // Written out and read again, it is the same; so is the writing, but for the remark at the top.
        let written = write(&found.variables, &found.commands, &found.rules);
        assert_eq!(read(&written).unwrap(), found);
        assert_eq!(written.trim(), GAME.lines().skip(1).collect::<Vec<_>>().join("\n").trim());
        // The page is told of the conditions, of what to do otherwise, and of the sums as they are written.
        let page = door.text(&["fanfare".to_string()]);
        assert!(page.contains("\"c\":[[\"Score\",\">\",2],[\"Lives\",\">\",0]]") && page.contains("\"o\":[[\"goto\",\"Hero\",\"Hero.x - 40\",262,\"\"]]") && page.contains("[\"wait\",\"\",1,0,\"\"]"), "{page}");
        assert_eq!(strangers(&found.rules, &found.commands, &["Apple".to_string(), "Basket".to_string(), "Hero".to_string()], &[]), vec!["Bullet", "Door", "Ship", "fanfare"]);
    }

    const PROGRAM: &str = "\
Numbers:
    Score = 0
    Shots = 3

The command Burst:
    make a copy of Explosion at Enemy
    play the sound boom

When the Space key is pressed:
    repeat Shots times:
        make a copy of Bullet at Ship
        wait 0.2 seconds
    if Score is more than 10 and Shots is less than 5:
        do the command Burst
        repeat while Score is more than 0:
            decrease Score by 3
    otherwise:
        play the sound zap
    increase Shots by 1
otherwise:
    stop everything
";

    #[test]
    fn loops_conditions_and_commands_nest_and_read_back() {
        set_language(Language::English);
        let found = read(PROGRAM).unwrap();
        assert_eq!(found.commands.iter().map(|c| (c.name.as_str(), c.steps.len())).collect::<Vec<_>>(), vec![("Burst", 2)]);
        let rule = &found.rules[0];
        // Three steps at the first level: the loop, the "if" with its "otherwise", and the last one.
        assert_eq!(rule.steps.iter().map(|step| step.kind).collect::<Vec<_>>(), vec![StepKind::Repeat, StepKind::If, StepKind::Add]);
        assert_eq!((rule.steps[0].ex.as_str(), rule.steps[0].body.len()), ("Shots", 2));
        let choice = &rule.steps[1];
        assert_eq!((choice.only.len(), choice.body.len(), choice.otherwise.len()), (2, 2, 1));
        assert_eq!((choice.body[0].kind, choice.body[0].a.as_str(), choice.body[1].kind, choice.body[1].body[0].x), (StepKind::Call, "Burst", StepKind::While, -3.0));
        // The "otherwise" at the edge is the rule's own.
        assert_eq!(rule.otherwise.iter().map(|step| step.kind).collect::<Vec<_>>(), vec![StepKind::StopAll]);
        // Written out it is the same text, and the page gets each block with what it holds.
        assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), PROGRAM.trim());
        let page = rule.text(&["boom".to_string(), "zap".to_string()]);
        assert!(page.contains("[\"repeat\",\"\",\"Shots\",0,\"\",[[\"spawn\",\"Bullet\",0,0,\"Ship\"],[\"wait\",\"\",0.2,0,\"\"]],[],[]]"), "{page}");
        assert!(page.contains("[\"while\",\"\",0,0,\"\",[[\"add\",\"Score\",-3,0,\"\"]],[[\"Score\",\">\",0]],[]]") && page.contains("[\"call\",\"Burst\",0,0,\"\",[]]"), "{page}");
        // The objects of what is nested and of the commands are known, for the page to keep them apart.
        assert_eq!(rule.objects(), vec!["Bullet", "Ship"]);
        assert_eq!(strangers(&found.rules, &found.commands, &[], &[]), vec!["Bullet", "Enemy", "Explosion", "Ship", "boom", "zap"]);
        // On one line, as a list shows it.
        assert_eq!(rule.steps[0].told(), "repeat Shots times [make a copy of Bullet at Ship; wait 0.2 seconds]");
    }

    const WITH_VALUES: &str = "\
Numbers:
    Score = 0

The command Award with Points, Who:
    increase Score by Points * 2
    make Who blink for a moment
    if Points is more than 5:
        do the command Award with Points - 5, Who

When Coin is clicked:
    do the command Award with 10, Coin
    do the command Award with Score + random(1, 3), \"Giant Rat\"
";

    #[test]
    fn a_command_is_given_values_by_name() {
        set_language(Language::English);
        let found = read(WITH_VALUES).unwrap();
        let award = &found.commands[0];
        assert_eq!((award.name.as_str(), award.params.clone()), ("Award", vec!["Points".to_string(), "Who".to_string()]));
        // Inside the command its values are numbers to reckon with; a value that is no number names an object.
        assert_eq!(award.steps[0].ex, "Points * 2");
        assert_eq!(award.steps[1].a, "Who");
        assert_eq!(award.steps[2].body[0].args, vec!["Points - 5", "Who"]);
        let calls = &found.rules[0].steps;
        assert_eq!((calls[0].args.clone(), calls[1].args.clone()), (vec!["10".to_string(), "Coin".to_string()], vec!["Score + random(1, 3)".to_string(), "\"Giant Rat\"".to_string()]));
        assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), WITH_VALUES.trim());
        // The page gets the values as they are written, and the objects named among them are kept apart for the rules.
        assert!(found.rules[0].text(&[]).contains("[\"call\",\"Award\",0,0,\"\",[\"10\",\"Coin\"]]"));
        let named = found.rules[0].objects();
        assert!(named.contains(&"Coin") && named.contains(&"Giant Rat"), "{named:?}");
        // As many values as the command takes, no more and no fewer; and outside it its values are not numbers.
        let told = |text: &str| read(text).unwrap_err().into_iter().map(|m| (m.line, m.what)).collect::<Vec<_>>();
        assert_eq!(told("The command Award with Points:\n    stop the film\n\nWhen the film starts:\n    do the command Award\n"), vec![(5, "the command Award takes 1 values, and is given 0".to_string())]);
        assert_eq!(told("The command Award with Points:\n    stop the film\n\nWhen the film starts:\n    do the command Award with 1, 2\n")[0].0, 5);
        assert_eq!(told("The command Award with Points:\n    stop the film\n\nWhen the film starts:\n    wait Points seconds\n"), vec![(5, "there is no number called Points".to_string())]);
    }

    #[test]
    fn a_name_that_is_not_there_is_guessed_at() {
        let known = vec!["Top".to_string(), "Sepet".to_string(), "Kahraman".to_string()];
        assert_eq!(meant("kTop", &known), Some(&known[0]));
        assert_eq!(meant("top", &known), Some(&known[0]));
        assert_eq!(meant("Kahrman", &known), Some(&known[2]));
        assert_eq!(meant("Elma", &known), None);
        assert_eq!(meant("Xy", &known), None, "a short name is not guessed from one as short");
    }

    #[test]
    fn what_is_wrong_in_a_block_is_told() {
        set_language(Language::English);
        let told = |text: &str| read(text).unwrap_err().into_iter().map(|m| (m.line, m.what)).collect::<Vec<_>>();
        assert_eq!(told("When the film starts:\n    do the command Dance\n"), vec![(2, "there is no command called Dance".to_string())]);
        assert_eq!(told("When the film starts:\n    repeat 3 times:\n    stop the film\n"), vec![(2, "the steps under this line are missing: they are written further in".to_string())]);
        assert_eq!(told("When the film starts:\n    dance 3 times:\n        stop the film\n")[0].0, 2);
        assert_eq!(told("When the film starts:\n    stop the film\n      play the film\n"), vec![(3, "this line is not as far in as the lines it belongs with".to_string())]);
        assert_eq!(told("When the film starts:\n    stop the film\n    otherwise:\n        play the film\n")[0], (3, "\"otherwise\" comes after the steps of a rule or of an \"if\"".to_string()));
        // A command may be used above where it is written, and may use itself.
        let found = read("When the film starts:\n    do the command Loop\n\nThe command Loop:\n    wait 1 seconds\n    do the command Loop\n").unwrap();
        assert_eq!((found.rules.len(), found.commands[0].steps.len()), (1, 2));
    }

    /// Writes a page whose rules use what only the written language has, to try in a browser:
    /// `cargo test -- --ignored language_page`.
    #[test]
    #[ignore]
    fn language_page() {
        set_language(Language::English);
        let mut e = crate::model::Editor::new(crate::model::Document::new(300, 200));
        e.doc.page_mut().start_timeline(20.0);
        let mut object = crate::model::Object::new_web(e.alloc_id(), crate::model::WebKind::Slice, crate::model::Rect { x: 20.0, y: 50.0, w: 40.0, h: 40.0 });
        (object.name, object.shape, object.fill) = ("Box".to_string(), crate::model::Shape::Rect { radius: 4.0 }, Some(crate::model::Color::rgb(30, 120, 210)));
        e.add_object(object);
        // A ring that falls and can jump, and a copy of a button symbol, as a student would have them.
        let mut ring = crate::model::Object::new_web(e.alloc_id(), crate::model::WebKind::Slice, crate::model::Rect { x: 200.0, y: 100.0, w: 40.0, h: 40.0 });
        (ring.name, ring.shape, ring.fill) = ("Ring".to_string(), crate::model::Shape::Ellipse, Some(crate::model::Color::rgb(210, 50, 50)));
        ring.behaviors = vec![crate::behavior::Behavior::new(crate::behavior::BehaviorKind::Gravity)];
        e.add_object(ring);
        let symbol = e.new_symbol("Button", crate::model::SymbolKind::Button);
        e.edit_symbol(symbol, None, true);
        let mut face = crate::model::Object::new_web(e.alloc_id(), crate::model::WebKind::Slice, crate::model::Rect { x: 100.0, y: 20.0, w: 80.0, h: 24.0 });
        (face.shape, face.fill) = (crate::model::Shape::Rect { radius: 12.0 }, Some(crate::model::Color::rgb(40, 40, 40)));
        e.add_object(face);
        e.leave_symbol();
        e.for_selected(|o| o.name = "Button".to_string());
        // What the page is made from can be given from outside, to try a text as it was written by hand.
        let text = std::env::var("RULES").unwrap_or_else(|_| "When Box is clicked:\n    make Box jump 80 high\n".to_string());
        let found = read(&text).unwrap();
        (e.doc.rules, e.doc.variables, e.doc.commands) = (found.rules, found.variables, found.commands);
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/target/dil.html"), crate::publish::html(&e.doc, "dil").unwrap()).unwrap();
    }

    #[test]
    fn steps_that_turn_resize_and_give_behaviors_are_read_and_written() {
        set_language(Language::English);
        let text = "\
When Button is clicked:
    turn Box by 45 degrees
    make Box spin -2 turns a second
    make Box 150 percent of its size
    make Box 40 percent opaque
    give Box the behavior Spin
    take from Ring the behavior Fall and Jump
    send Ring after Box at 120
";
        let found = read(text).unwrap();
        let steps = &found.rules[0].steps;
        assert_eq!(steps.iter().map(|s| s.kind).collect::<Vec<_>>(), vec![StepKind::Turn, StepKind::Whirl, StepKind::Resize, StepKind::Opacity, StepKind::Give, StepKind::Take, StepKind::Chase]);
        // A behavior is kept by its own word, whatever the language it was written in.
        assert_eq!((steps[4].b.as_str(), steps[5].b.as_str(), steps[6].b.as_str(), steps[6].x, steps[1].x), ("spin", "gravity", "Box", 120.0, -2.0));
        assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
        // Both objects of "send after" are objects the page must be able to move.
        assert_eq!(crate::rules::objects_of(steps).iter().filter(|name| **name == "Box").count(), 6);
        let told = read("When Box is clicked:\n    give Box the behavior Dance\n").unwrap_err();
        assert_eq!((told[0].line, told[0].what.as_str()), (2, "there is no behavior called Dance"));
        // The page carries the steps, and the numbers a behavior given by a rule starts with.
        let mut doc = crate::model::Document::new(100, 100);
        doc.page_mut().start_timeline(20.0);
        doc.rules = found.rules;
        let page = crate::publish::html(&doc, "x").unwrap();
        assert!(page.contains(r#"["give","Box",0,0,"spin"]"#) && page.contains(r#""spin":[2,0]"#) && page.contains(r#"["whirl","Box",-2,0,""]"#) && page.contains(r#""ricochet":[220,50]"#), "{}", &page[page.find("rules:").unwrap()..][..600]);
    }

    #[test]
    fn a_car_is_set_going_slowed_and_stopped() {
        set_language(Language::English);
        let text = "\
When the film starts:
    attach Wheel to Car

When the Right Arrow key is pressed:
    make Car go to the right at 320

When the Right Arrow key is released:
    make Car slow down and stop
    make Car stop
    let Wheel go
";
        let found = read(text).unwrap();
        let kinds: Vec<StepKind> = found.rules.iter().flat_map(|rule| &rule.steps).map(|step| step.kind).collect();
        assert_eq!(kinds, vec![StepKind::Attach, StepKind::Go, StepKind::Slow, StepKind::Halt, StepKind::Detach]);
        assert_eq!((found.rules[2].event.kind, found.rules[2].steps[0].a.as_str(), found.rules[1].steps[0].b.as_str()), (EventKind::KeyUp, "Car", "right"));
        assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
    }

    #[test]
    fn a_value_of_a_command_is_not_taken_for_a_missing_object() {
        set_language(Language::English);
        let found = read("The command Turn with Who, Times:\n    repeat Times times:\n        turn Who by 45 degrees\n\nWhen Box is clicked:\n    do the command Turn with Box, 8\n").unwrap();
        assert!(strangers(&found.rules, &found.commands, &["Box".to_string()], &[]).is_empty());
        // What is not there is still told: the object the command is given, and one named outright inside it.
        assert_eq!(strangers(&found.rules, &found.commands, &[], &[]), vec!["Box".to_string()]);
    }

    #[test]
    fn what_cannot_be_read_is_told_with_its_line() {
        set_language(Language::English);
        let told = |text: &str| read(text).unwrap_err().into_iter().map(|m| (m.line, m.what)).collect::<Vec<_>>();
        assert_eq!(told("When Apple dances:\n    hide Apple\n"), vec![(1, "this is not an event I know".to_string())]);
        assert_eq!(told("When Apple is clicked:\n    make Apple sing\n"), vec![(2, "this is not a step I know".to_string())]);
        assert_eq!(told("When Apple is clicked:\n    increase Score by 1\n"), vec![(2, "there is no number called Score".to_string())]);
        assert_eq!(told("When Apple is clicked\n"), vec![(1, "a line that starts at the edge is an event and ends with a colon; steps are written further in".to_string())]);
        assert_eq!(told("Numbers:\n    Score = lots\n"), vec![(2, "lots is not a number".to_string())]);
        assert_eq!(told("otherwise:\n    hide Apple\n"), vec![(1, "\"otherwise\" comes after the steps of a rule or of an \"if\"".to_string())]);
        // Capitals at the start of a step do no harm, and neither do tabs or a decimal comma.
        let found = read("Numbers:\n\tTime = 2,5\n\nEvery 0,5 seconds:\n\tDecrease Time by 0,5\n").unwrap();
        assert_eq!((found.variables[0].start, found.rules[0].event.n, found.rules[0].steps[0].x), (2.5, 0.5, -0.5));
    }

    #[test]
    fn every_game_that_comes_with_the_program_reads_back_from_its_own_text() {
        // In English here; `tests/turkce.rs` does the same in Turkish, in a run of its own, since the language
        // is one for the whole program and the tests beside this one read English.
        for language in [Language::English] {
            set_language(language);
            for example in crate::sahne_examples::EXAMPLES {
                let doc = (example.build)();
                if doc.rules.is_empty() {
                    continue;
                }
                let written = write(&doc.variables, &doc.commands, &doc.rules);
                let found = read(&written).unwrap_or_else(|mistakes| panic!("{} ({language:?}): {mistakes:?}\n{written}", example.name));
                assert_eq!(found.rules, doc.rules, "{} ({language:?})\n{written}", example.name);
                assert_eq!(found.variables, doc.variables, "{}", example.name);
                assert!(strangers(&doc.rules, &doc.commands, &doc.object_names(), &doc.sounds.iter().map(|s| s.name.clone()).collect::<Vec<_>>()).is_empty(), "{}", example.name);
            }
        }
        set_language(Language::English);
    }
}
