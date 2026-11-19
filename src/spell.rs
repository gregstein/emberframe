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

//! Spell checking, by way of the spelling dictionaries installed on the system.
//!
//! The words are handed to `enchant-2` (or `hunspell`), which speak the same plain
//! protocol: a word goes in on a line, and a line comes back saying whether it is
//! spelled right and, if not, what it might have been meant to be.

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Result};

use crate::i18n::t;

/// A word of a text: where it starts and ends, in bytes, and the word.
pub type Word = (usize, usize, String);

/// The words of a text. A word is letters, with apostrophes and hyphens inside it; numbers are not words.
pub fn words(text: &str) -> Vec<Word> {
    let mut found = Vec::new();
    let mut start: Option<usize> = None;
    let joins = |c: char| c == '\'' || c == '’' || c == '-';
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let inside = c.is_alphabetic() || (start.is_some() && joins(c) && chars.peek().is_some_and(|(_, next)| next.is_alphabetic()));
        match (inside, start) {
            (true, None) => start = Some(at),
            (false, Some(from)) => {
                found.push((from, at, text[from..at].to_string()));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(from) = start {
        found.push((from, text.len(), text[from..].to_string()));
    }
    found
}

/// Reads what the checker answered for one word: `None` if it is spelled right, otherwise the suggestions, which may be none.
fn parse_answer(line: &str) -> Option<Vec<String>> {
    match line.chars().next() {
        // Found, found by a rule, or found as a compound.
        Some('*' | '+' | '-') | None => None,
        Some('&') => Some(line.split_once(": ").map_or_else(Vec::new, |(_, list)| list.split(", ").map(|word| word.trim().to_string()).filter(|word| !word.is_empty()).collect())),
        // Not found, and nothing to suggest.
        _ => Some(Vec::new()),
    }
}

/// The dictionary to check against for the language the interface is in.
pub fn dictionary() -> &'static str {
    match crate::i18n::language() {
        crate::i18n::Language::Turkish => "tr_TR",
        crate::i18n::Language::English => "en_US",
    }
}

/// Checks words against a dictionary. Gives the misspelled ones, each with what the checker suggests for it.
pub fn check(words: &[String], dictionary: &str) -> Result<HashMap<String, Vec<String>>> {
    if words.is_empty() {
        return Ok(HashMap::new());
    }
    // A caret before a word tells the checker it is a word to check and not a command.
    let input: String = words.iter().map(|word| format!("^{word}\n")).collect();
    let run = |program: &str| -> Option<String> {
        let mut child = Command::new(program).args(["-d", dictionary, "-a"]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
        child.stdin.take()?.write_all(input.as_bytes()).ok()?;
        let out = child.wait_with_output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let Some(output) = run("enchant-2").or_else(|| run("hunspell")) else {
        bail!(t("Spell checking needs enchant-2 or hunspell, and a dictionary for the language."));
    };
    // After its greeting the checker answers each line with one line for the word, then an empty one.
    let mut answers = output.lines().skip(1).filter(|line| !line.is_empty());
    let mut wrong = HashMap::new();
    for word in words {
        let answer = answers.next().ok_or_else(|| anyhow!(t("The spell checker stopped answering.")))?;
        if let Some(suggestions) = parse_answer(answer) {
            wrong.insert(word.clone(), suggestions);
        }
    }
    Ok(wrong)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_split_into_its_words() {
        let found = words("Merhaba dünya, 25 kedi'nin öğle-yemeği!");
        let only: Vec<&str> = found.iter().map(|(_, _, word)| word.as_str()).collect();
        assert_eq!(only, ["Merhaba", "dünya", "kedi'nin", "öğle-yemeği"]);
        // Each word is where it says it is, counted in bytes.
        let text = "Merhaba dünya, 25 kedi'nin öğle-yemeği!";
        assert!(found.iter().all(|(from, to, word)| &text[*from..*to] == word));
        assert!(words("12 34 -- ''").is_empty() && words("").is_empty());
        assert_eq!(words("son")[0], (0, 3, "son".into()));
    }

    #[test]
    fn the_checkers_answers_are_read() {
        assert_eq!(parse_answer("*"), None);
        assert_eq!(parse_answer("+ ROOT"), None);
        assert_eq!(parse_answer("& yanlşı 4 0: yanlı, yanışı, yanılışı, yanaşıl"), Some(vec!["yanlı".into(), "yanışı".into(), "yanılışı".into(), "yanaşıl".into()]));
        assert_eq!(parse_answer("# xqzw 0"), Some(Vec::new()));
        assert!(check(&[], "tr_TR").unwrap().is_empty());
    }
}
