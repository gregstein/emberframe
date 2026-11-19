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

//! The text being written in Rules as Text: its lines, where the caret is, and what the
//! reading of it has found. A small editor of its own, since the program has no other
//! place where many lines are typed: enough to write, correct and paste rules in.

use crate::rules::Pick;
use crate::script::{Mistake, Opens, Read};

/// How far in a step is written.
const INDENT: &str = "    ";

#[derive(Clone, Debug, Default)]
pub struct ScriptEdit {
    pub lines: Vec<String>,
    /// The line the caret is on, and how many characters of it lie before the caret.
    pub row: usize,
    pub col: usize,
    /// What the text says, or what is wrong with it: read again after every change.
    pub outcome: Option<Result<Read, Vec<Mistake>>>,
    /// Whether the text has been changed since it was last taken into the document.
    pub changed: bool,
    /// The text as it was before each change, the latest last, for taking changes back; and the changes taken
    /// back, for putting them in again. Letters typed one after another count as one change.
    past: Vec<Kept>,
    undone: Vec<Kept>,
    typing: bool,
}

/// The text and the caret at one moment.
#[derive(Clone, Debug, Default)]
struct Kept {
    lines: Vec<String>,
    row: usize,
    col: usize,
}

/// The most changes that can be taken back.
const MOST_KEPT: usize = 400;

/// The names there are to write: what a sentence can be about.
#[derive(Clone, Debug, Default)]
pub struct Names {
    pub objects: Vec<String>,
    pub numbers: Vec<String>,
    pub commands: Vec<String>,
    pub sounds: Vec<String>,
}

/// Something offered as the line is typed: a whole sentence for the line, or the rest of a name.
#[derive(Clone, Debug, PartialEq)]
pub enum Offer {
    /// The sentence the line becomes, with a question mark at each gap still to fill.
    Sentence(String),
    /// A name, to stand in place of the letters of it typed so far.
    Name(String),
}

impl Offer {
    pub fn shown(&self) -> &str {
        match self {
            Offer::Sentence(text) | Offer::Name(text) => text,
        }
    }
}

/// The most offers shown at once.
const MOST_OFFERS: usize = 8;

/// The place in a line, in bytes, of its `col`th character.
fn byte_at(line: &str, col: usize) -> usize {
    line.char_indices().nth(col).map_or(line.len(), |(at, _)| at)
}

impl ScriptEdit {
    pub fn new(text: &str) -> ScriptEdit {
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let mut edit = ScriptEdit { lines, ..Default::default() };
        edit.check();
        edit.changed = false;
        edit
    }

    pub fn text(&self) -> String {
        let mut text = self.lines.join("\n");
        text.push('\n');
        text
    }

    fn now(&self) -> Kept {
        Kept { lines: self.lines.clone(), row: self.row, col: self.col }
    }

    /// Keeps the text as it is, before it is changed. A letter typed straight after another is part of the same change.
    fn keep(&mut self, typing: bool) {
        if !(typing && self.typing) {
            self.past.push(self.now());
            if self.past.len() > MOST_KEPT {
                self.past.remove(0);
            }
        }
        self.undone.clear();
        self.typing = typing;
    }

    /// Takes the last change back. Says whether there was one.
    pub fn undo(&mut self) -> bool {
        let Some(kept) = self.past.pop() else { return false };
        self.undone.push(self.now());
        (self.lines, self.row, self.col, self.typing) = (kept.lines, kept.row, kept.col, false);
        self.check();
        true
    }

    /// Puts back in the change last taken back.
    pub fn redo(&mut self) -> bool {
        let Some(kept) = self.undone.pop() else { return false };
        self.past.push(self.now());
        (self.lines, self.row, self.col, self.typing) = (kept.lines, kept.row, kept.col, false);
        self.check();
        true
    }

    /// Reads the text again.
    pub fn check(&mut self) {
        self.outcome = Some(crate::script::read(&self.text()));
        self.changed = true;
    }

    pub fn mistakes(&self) -> &[Mistake] {
        match &self.outcome {
            Some(Err(mistakes)) => mistakes,
            _ => &[],
        }
    }

    fn length(&self) -> usize {
        self.lines[self.row].chars().count()
    }

    /// Puts typed or pasted text in at the caret. A tab is a step's worth of spaces.
    pub fn insert(&mut self, text: &str) {
        // A word typed letter by letter is one change; a space, or anything pasted, begins another.
        let mut letters = text.chars();
        self.keep(matches!((letters.next(), letters.next()), (Some(letter), None) if !letter.is_whitespace()));
        for (index, piece) in text.replace('\r', "").replace('\t', INDENT).split('\n').enumerate() {
            if index > 0 {
                self.split();
            }
            let at = byte_at(&self.lines[self.row], self.col);
            self.lines[self.row].insert_str(at, piece);
            self.col += piece.chars().count();
        }
        self.check();
    }

    fn split(&mut self) {
        let at = byte_at(&self.lines[self.row], self.col);
        let rest = self.lines[self.row].split_off(at);
        self.lines.insert(self.row + 1, rest);
        self.row += 1;
        self.col = 0;
    }

    /// Enter: a new line. Under an event it starts further in, where its steps go; under a step, as far in as the step.
    pub fn newline(&mut self) {
        self.keep(false);
        let line = &self.lines[self.row];
        let before: String = line.chars().take(self.col).collect();
        // As far in as the line it comes from, and a step further under a line that opens steps.
        let mut lead: String = before.chars().take_while(|c| *c == ' ').collect();
        if before.trim_end().ends_with(':') {
            lead.push_str(INDENT);
        }
        self.split();
        self.lines[self.row].insert_str(0, &lead);
        self.col = lead.chars().count();
        self.check();
    }

    pub fn backspace(&mut self) {
        self.keep(false);
        if self.col > 0 {
            // At the start of a step, the whole of its indent goes at once.
            let before: String = self.lines[self.row].chars().take(self.col).collect();
            let take = if before.chars().all(|c| c == ' ') && self.col % INDENT.len() == 0 { INDENT.len() } else { 1 };
            let (from, to) = (byte_at(&self.lines[self.row], self.col - take), byte_at(&self.lines[self.row], self.col));
            self.lines[self.row].replace_range(from..to, "");
            self.col -= take;
        } else if self.row > 0 {
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.length();
            self.lines[self.row].push_str(&line);
        }
        self.check();
    }

    pub fn delete(&mut self) {
        self.keep(false);
        if self.col < self.length() {
            let (from, to) = (byte_at(&self.lines[self.row], self.col), byte_at(&self.lines[self.row], self.col + 1));
            self.lines[self.row].replace_range(from..to, "");
        } else if self.row + 1 < self.lines.len() {
            let line = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&line);
        }
        self.check();
    }

    /// Moves the caret by lines and characters; over the end of a line it goes on to the next.
    pub fn step(&mut self, rows: isize, cols: isize) {
        self.typing = false;
        if rows != 0 {
            self.row = (self.row as isize + rows).clamp(0, self.lines.len() as isize - 1) as usize;
            self.col = self.col.min(self.length());
        }
        if cols < 0 {
            if self.col > 0 {
                self.col -= 1;
            } else if self.row > 0 {
                self.row -= 1;
                self.col = self.length();
            }
        } else if cols > 0 {
            if self.col < self.length() {
                self.col += 1;
            } else if self.row + 1 < self.lines.len() {
                self.row += 1;
                self.col = 0;
            }
        }
    }

    pub fn home(&mut self) {
        self.col = 0;
    }

    pub fn end(&mut self) {
        self.col = self.length();
    }

    /// Puts the caret on a line, as near the character asked for as the line has.
    pub fn place(&mut self, row: usize, col: usize) {
        self.typing = false;
        self.row = row.min(self.lines.len() - 1);
        self.col = col.min(self.length());
    }

    /// What can be offered for the line the caret is at the end of: the sentences that begin as the line does,
    /// and the names that begin as its last word does. Nothing is offered in the middle of a line or on an empty one.
    pub fn offers(&self, names: &Names) -> Vec<Offer> {
        let line = &self.lines[self.row];
        let before: String = line.chars().take(self.col).collect();
        let after: String = line.chars().skip(self.col).collect();
        let typed = before.trim_start();
        // In the middle of a line only names are offered, and only at the end of the word being typed:
        // a sentence would take the place of what is already written after it.
        let inside = !after.trim().is_empty();
        if typed.is_empty() || (inside && after.chars().next().is_some_and(|c| c.is_alphanumeric() || c == '_')) {
            return Vec::new();
        }
        let edge = !before.starts_with(' ');
        let low = typed.to_lowercase();
        let mut found = Vec::new();
        // The first word says what the sentence can be: a number's name brings the sentences about numbers, an
        // object's those about objects, a figure those that open with an amount. A name is known whatever
        // its capitals, and is written the way it is kept.
        let first = typed.split(' ').next().unwrap_or_default();
        let known = |list: &[String]| list.iter().find(|name| name.to_lowercase() == first.to_lowercase()).cloned();
        let keys = crate::script::key_names();
        let figure = first.replace(',', ".").parse::<f32>().is_ok();
        let (number, command, object, sound, key) = (known(&names.numbers), known(&names.commands), known(&names.objects), known(&names.sounds), known(&keys));
        let fits = |opens: Opens| match opens {
            Opens::Words => true,
            Opens::Amount => figure || number.is_some(),
            Opens::Condition => number.is_some(),
            Opens::Name(Pick::Variable) => number.is_some(),
            Opens::Name(Pick::Command) => command.is_some(),
            Opens::Name(Pick::Sound) => sound.is_some(),
            Opens::Name(Pick::Key) => key.is_some(),
            // A word that is nothing yet may be an object still to be drawn, or a command still to be written.
            Opens::Name(_) => object.is_some() || (!figure && number.is_none() && command.is_none() && key.is_none() && sound.is_none()),
            Opens::NewName => !figure && number.is_none() && object.is_none(),
        };
        let written = number.clone().or(command.clone()).or(object.clone()).or(sound.clone()).or(key.clone()).unwrap_or_else(|| first.to_string());
        for (pattern, opens) in crate::script::offers(edge).into_iter().filter(|_| !inside) {
            let sentence = if pattern.starts_with('?') {
                if !fits(opens) {
                    continue;
                }
                pattern.replacen('?', &written, 1)
            } else {
                pattern
            };
            let lowered = sentence.to_lowercase();
            if lowered.starts_with(&low) && lowered != low && lowered.trim_end_matches(':') != low {
                found.push(Offer::Sentence(sentence));
            }
        }
        // The word being typed, if it is the beginning of a name. A name already whole is not offered again.
        let word: String = typed.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<Vec<_>>().into_iter().rev().collect();
        // Where the sentence wants a behavior, the behaviors are offered, even before a letter of one is typed.
        let before_word = &typed[..typed.len() - word.len()];
        if !edge && crate::script::behavior_leads().iter().any(|lead| before_word.ends_with(lead.as_str())) {
            let partial = word.to_lowercase();
            let fitting = crate::behavior::KINDS.iter().map(|kind| crate::i18n::t(kind.name()).to_string()).filter(|name| name.to_lowercase().starts_with(&partial) && *name != word);
            found.splice(0..0, fitting.map(Offer::Name));
        } else if !word.is_empty() {
            let partial = word.to_lowercase();
            let mut fitting: Vec<&String> = names.objects.iter().chain(&names.numbers).chain(&names.commands).chain(&names.sounds).filter(|name| name.to_lowercase().starts_with(&partial) && **name != word).collect();
            fitting.sort();
            fitting.dedup();
            // Inside a sentence the names come first, and so they do while the first word is still the
            // beginning of a name and not yet one; once it is a name, the sentences about it come first.
            let offered = fitting.into_iter().map(|name| Offer::Name(name.clone()));
            let nothing_yet = number.is_none() && command.is_none() && object.is_none() && sound.is_none() && key.is_none();
            if typed.contains(' ') || nothing_yet {
                found.splice(0..0, offered);
            } else {
                found.extend(offered);
            }
        }
        found.truncate(MOST_OFFERS);
        found
    }

    /// Takes an offer. A sentence becomes the line, and the caret goes to its first gap; a name takes the place of its beginning.
    pub fn accept(&mut self, offer: &Offer) {
        self.keep(false);
        match offer {
            Offer::Sentence(sentence) => {
                let lead: String = self.lines[self.row].chars().take_while(|c| *c == ' ').collect();
                self.lines[self.row] = format!("{lead}{sentence}");
                self.col = 0;
                if !self.next_gap() {
                    // A sentence with no gap is whole: after a line that opens steps the caret may as well go on.
                    self.col = self.length();
                }
            }
            Offer::Name(name) => {
                let before: Vec<char> = self.lines[self.row].chars().take(self.col).collect();
                let typed = before.iter().rev().take_while(|c| c.is_alphanumeric() || **c == '_').count();
                let (from, to) = (byte_at(&self.lines[self.row], self.col - typed), byte_at(&self.lines[self.row], self.col));
                self.lines[self.row].replace_range(from..to, name);
                self.col = self.col - typed + name.chars().count();
            }
        }
        self.check();
    }

    /// Goes to the next gap of the line, a question mark, and takes the mark away so that the name can be typed. Says whether there was one.
    pub fn next_gap(&mut self) -> bool {
        let line = &self.lines[self.row];
        let after = line.chars().enumerate().skip(self.col).find(|(_, c)| *c == '?').map(|(at, _)| at);
        let Some(at) = after.or_else(|| line.chars().position(|c| c == '?')) else { return false };
        // Going to a gap is part of taking the sentence it is in: it is not kept as a change of its own.
        self.typing = false;
        let (from, to) = (byte_at(line, at), byte_at(line, at + 1));
        self.lines[self.row].replace_range(from..to, "");
        self.col = at;
        self.check();
        true
    }

    /// Writes one name in place of another wherever it stands as a word of its own. Says how many times it did.
    pub fn replace_name(&mut self, from: &str, to: &str) -> usize {
        if from.is_empty() {
            return 0;
        }
        self.keep(false);
        let part_of_word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        let mut done = 0;
        for line in &mut self.lines {
            let mut out = String::with_capacity(line.len());
            let mut rest = line.as_str();
            while let Some(at) = rest.find(from) {
                let (before, after) = (rest[..at].chars().next_back(), rest[at + from.len()..].chars().next());
                out.push_str(&rest[..at]);
                if part_of_word(before) || part_of_word(after) {
                    out.push_str(from);
                } else {
                    out.push_str(to);
                    done += 1;
                }
                rest = &rest[at + from.len()..];
            }
            out.push_str(rest);
            *line = out;
        }
        self.col = self.col.min(self.length());
        self.check();
        done
    }

    /// Adds a sentence as a line of its own under the caret's line: an event at the edge with a blank
    /// line before it, a step further in. The caret is left at its end.
    pub fn add_sentence(&mut self, sentence: &str, event: bool) {
        self.keep(false);
        let line = if event { format!("{sentence}:") } else { format!("{INDENT}{sentence}") };
        // An empty line is written into; otherwise the sentence goes under the line the caret is on.
        if self.lines[self.row].trim().is_empty() {
            self.lines[self.row] = line;
        } else {
            if event {
                self.lines.insert(self.row + 1, String::new());
                self.row += 1;
            }
            self.lines.insert(self.row + 1, line);
            self.row += 1;
        }
        self.col = self.length();
        self.check();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{set_language, Language};

    #[test]
    fn typing_a_rule_line_by_line_gives_a_rule() {
        set_language(Language::English);
        let mut edit = ScriptEdit::new("");
        edit.insert("When Apple is clicked:");
        // Enter after an event goes further in, where its step is written; Enter after a step stays as far in.
        edit.newline();
        assert_eq!((edit.row, edit.col, edit.lines[1].as_str()), (1, 4, "    "));
        edit.insert("hide Apple");
        edit.newline();
        edit.insert("stop evrything");
        assert_eq!(edit.mistakes().iter().map(|m| m.line).collect::<Vec<_>>(), vec![3]);
        // The caret goes back to the slip and it is put right.
        for _ in 0.."rything".len() {
            edit.step(0, -1);
        }
        edit.insert("e");
        assert!(edit.mistakes().is_empty());
        let found = edit.outcome.clone().unwrap().unwrap();
        assert_eq!(found.rules[0].sentence(), "When Apple is clicked: hide Apple; stop everything");
        // Backspace at the start of a step takes its indent at once, then joins it to the line above.
        edit.home();
        for _ in 0..4 {
            edit.step(0, 1);
        }
        edit.backspace();
        assert_eq!((edit.col, edit.lines[2].as_str()), (0, "stop everything"));
        edit.backspace();
        assert_eq!((edit.row, edit.lines.len(), edit.lines[1].as_str()), (1, 2, "    hide Applestop everything"));
    }

    #[test]
    fn sentences_and_names_are_offered_as_a_line_is_typed() {
        set_language(Language::English);
        let names = Names { objects: vec!["Apple".into(), "Basket".into()], numbers: vec!["Score".into()], commands: vec!["Burst".into()], sounds: vec!["boom".into()] };
        let shown = |edit: &ScriptEdit| edit.offers(&names).iter().map(|offer| offer.shown().to_string()).collect::<Vec<_>>();
        // At the edge the events that begin as the line does are offered.
        let mut edit = ScriptEdit::new("");
        edit.insert("when the f");
        assert_eq!(shown(&edit), vec!["When the film starts:"]);
        // Taking it writes the line; with no gap to fill, the caret is at its end.
        let offer = edit.offers(&names)[0].clone();
        edit.accept(&offer);
        assert_eq!((edit.lines[0].as_str(), edit.col), ("When the film starts:", 21));
        assert!(edit.offers(&names).is_empty(), "a line that is whole is offered nothing");
        // Under it, further in, the steps; a gap is a question mark, and Tab goes from gap to gap.
        edit.newline();
        edit.insert("put");
        assert_eq!(shown(&edit), vec!["put ? somewhere at random", "put ? at the top, at random", "put ? at ?, ?"]);
        let offer = edit.offers(&names)[2].clone();
        edit.accept(&offer);
        assert_eq!((edit.lines[1].as_str(), edit.col), ("    put  at ?, ?", 8));
        // In a gap the beginning of a name brings the name.
        edit.insert("Ba");
        assert_eq!(shown(&edit), vec!["Basket"], "in the middle of a line the name is offered, and no sentence");
        let offer = edit.offers(&names)[0].clone();
        edit.accept(&offer);
        assert_eq!((edit.lines[1].as_str(), edit.col), ("    put Basket at ?, ?", 14));
        // Not inside a word, though: there the rest of the word is already written.
        edit.col = 6;
        assert_eq!(edit.offers(&names), Vec::new());
        edit.end();
        edit.insert(" Sc");
        assert_eq!(shown(&edit), vec!["Score"]);
        let offer = edit.offers(&names)[0].clone();
        edit.accept(&offer);
        assert!(edit.lines[1].ends_with(" Score"));
        // Where a behavior is wanted the behaviors are offered: all of them at first, then those that begin as typed.
        let mut edit = ScriptEdit::new("When the film starts:");
        edit.end();
        edit.newline();
        edit.insert("give Apple the behavior ");
        assert_eq!(shown(&edit)[..2], ["Spin", "Bounce"]);
        edit.insert("ro");
        assert_eq!(shown(&edit), vec!["Roll Like a Wheel"]);
        let offer = edit.offers(&names)[0].clone();
        edit.accept(&offer);
        assert_eq!(edit.lines[1], "    give Apple the behavior Roll Like a Wheel");
        // What the first word is decides what is offered: `tests/turkce.rs` tries that, where sentences open with a name.
        let english = |line: &str| {
            let mut edit = ScriptEdit::new("When the film starts:");
            edit.end();
            edit.newline();
            edit.insert(line);
            edit.offers(&names).iter().map(|offer| offer.shown().to_string()).collect::<Vec<_>>()
        };
        assert_eq!(english("do"), vec!["do the command ? with ?", "do the command ?"]);
        assert!(english("rep").contains(&"repeat ? times:".to_string()) && english("rep").contains(&"repeat while ?:".to_string()));
        // A gap is gone to and its mark taken away.
        let mut edit = ScriptEdit::new("    put Apple at ?, ?");
        assert!(edit.next_gap());
        assert_eq!((edit.lines[0].as_str(), edit.col), ("    put Apple at , ?", 17));
        edit.insert("10");
        assert!(edit.next_gap());
        edit.insert("20");
        assert_eq!(edit.lines[0], "    put Apple at 10, 20");
        assert!(!edit.next_gap(), "no gap is left");
    }

    #[test]
    fn changes_are_taken_back_and_put_in_again() {
        set_language(Language::English);
        let mut edit = ScriptEdit::new("When the film starts:");
        edit.end();
        edit.newline();
        for letter in ["h", "i", "d", "e"] {
            edit.insert(letter);
        }
        edit.insert(" ");
        for letter in ["A", "p", "p", "l", "e"] {
            edit.insert(letter);
        }
        assert_eq!(edit.lines[1], "    hide Apple");
        // A word comes back out whole, then the space, then the word before it, then the new line.
        assert!(edit.undo());
        assert_eq!(edit.lines[1], "    hide ");
        edit.undo();
        edit.undo();
        assert_eq!((edit.lines[1].as_str(), edit.col), ("    ", 4));
        edit.undo();
        assert_eq!((edit.lines.len(), edit.row, edit.col), (1, 0, 21));
        assert!(!edit.undo(), "nothing more to take back");
        // And goes back in, change by change, until the text is as it was.
        while edit.redo() {}
        assert_eq!(edit.lines, vec!["When the film starts:", "    hide Apple"]);
        // Something typed after taking back leaves nothing to put in again.
        edit.undo();
        edit.insert("x");
        assert!(!edit.redo());
        // A paste and a backspace are each one change.
        let mut edit = ScriptEdit::new("");
        edit.insert("one\ntwo");
        edit.backspace();
        edit.undo();
        assert_eq!(edit.lines, vec!["one", "two"]);
        edit.undo();
        assert_eq!(edit.lines, vec![""]);
    }

    #[test]
    fn a_name_is_put_right_everywhere_it_stands_alone() {
        set_language(Language::English);
        let mut edit = ScriptEdit::new("When kTop is clicked:\n    hide kTop\n    show kTopper\n    put kTop at kTop.x + 1, 5");
        assert_eq!(edit.replace_name("kTop", "Top"), 4);
        assert_eq!(edit.lines, vec!["When Top is clicked:", "    hide Top", "    show kTopper", "    put Top at Top.x + 1, 5"]);
        // It is one change, to be taken back at once.
        edit.undo();
        assert_eq!(edit.lines[1], "    hide kTop");
    }

    #[test]
    fn pasting_and_adding_sentences_keeps_the_lines_right() {
        set_language(Language::English);
        let mut edit = ScriptEdit::new("");
        edit.insert("Numbers:\n\tScore = 0\n");
        assert_eq!(edit.lines, vec!["Numbers:", "    Score = 0", ""]);
        edit.add_sentence("When Apple is clicked", true);
        edit.add_sentence("increase Score by 1", false);
        assert_eq!(edit.text(), "Numbers:\n    Score = 0\nWhen Apple is clicked:\n    increase Score by 1\n");
        // A second event gets a blank line before it.
        edit.add_sentence("When the film starts", true);
        assert_eq!(&edit.lines[4..], ["", "When the film starts:"]);
        assert!(edit.mistakes().is_empty() && edit.changed);
        // Letters that take more than a byte are one character to the caret.
        let mut edit = ScriptEdit::new("Şişe");
        edit.end();
        edit.backspace();
        edit.step(0, -1);
        edit.insert("ğ");
        assert_eq!((edit.lines[0].as_str(), edit.col), ("Şiğş", 3));
        edit.delete();
        assert_eq!(edit.lines[0], "Şiğ");
    }
}
