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

//! The user guide, carried in the program and shown as a page in the browser.
//!
//! The guide is written in Markdown, in `docs/`, once for each language. It is built
//! into the program, so that a student has the lessons wherever the program is
//! installed, with no network and no folder to look in. Only the Markdown the guide
//! uses is understood: headings, paragraphs, lists, tables, bold, italic and code.

use crate::i18n::Language;

const TURKISH: &str = include_str!("../docs/guide.tr.md");
const ENGLISH: &str = include_str!("../docs/guide.md");

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// A line's own marks: `code`, **bold** and *italic*.
fn inline(text: &str) -> String {
    let text = escape(text);
    let mut out = String::with_capacity(text.len() + 16);
    let (mut bold, mut italic, mut code) = (false, false, false);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' => {
                out.push_str(if code { "</code>" } else { "<code>" });
                code = !code;
            }
            '*' if !code && chars.peek() == Some(&'*') => {
                chars.next();
                out.push_str(if bold { "</b>" } else { "<b>" });
                bold = !bold;
            }
            '*' if !code => {
                out.push_str(if italic { "</i>" } else { "<i>" });
                italic = !italic;
            }
            _ => out.push(c),
        }
    }
    out
}

/// The words a paragraph opens with in bold, without their full stop: the name of a topic or a lesson.
fn lead(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("**")?;
    let end = rest.find("**")?;
    let name = rest[..end].trim_end_matches('.');
    (!name.is_empty() && name.chars().count() <= 70).then_some(name)
}

/// The guide as a page, and the place in it where the part about animation begins.
pub fn page(language: Language, title: &str, contents: &str) -> (String, String) {
    let source = if language == Language::Turkish { TURKISH } else { ENGLISH };
    let mut body = String::new();
    // The list of what is in the guide: each heading, and under it the topics that open a paragraph in bold.
    let mut list = String::new();
    let (mut count, mut animation, mut in_list, mut open) = (0, String::new(), false, "");
    let mut name = escape(title);
    let close = |body: &mut String, open: &mut &str| {
        if !open.is_empty() {
            body.push_str(&format!("</{open}>\n"));
            *open = "";
        }
    };
    let mut lines = source.lines().peekable();
    let mut fenced = false;
    while let Some(line) = lines.next() {
        let line = line.trim_end();
        // Between two lines of three backticks the text is shown as it is written, line for line: rules in the language.
        if line.trim_start().starts_with("```") {
            close(&mut body, &mut open);
            body.push_str(if fenced { "</pre>\n" } else { "<pre>" });
            fenced = !fenced;
            continue;
        }
        if fenced {
            body.push_str(&escape(line));
            body.push('\n');
            continue;
        }
        if let Some(heading) = line.strip_prefix("## ") {
            close(&mut body, &mut open);
            count += 1;
            let id = format!("b{count}");
            if animation.is_empty() && heading.contains("PisiAnimasyon") {
                animation = id.clone();
            }
            body.push_str(&format!("<h2 id=\"{id}\">{}</h2>\n", inline(heading)));
            if in_list {
                list.push_str("</ul></li>\n");
            }
            list.push_str(&format!("<li><a href=\"#{id}\">{}</a><ul>", inline(heading)));
            in_list = true;
        } else if let Some(heading) = line.strip_prefix("# ") {
            // The guide's own name goes at the top of the page, over the list of what is in it.
            close(&mut body, &mut open);
            name = inline(heading);
        } else if line.starts_with('|') {
            // A table: its first row is the head, its second only rules it off.
            close(&mut body, &mut open);
            let cells = |row: &str| -> Vec<String> { row.trim_matches('|').split('|').map(|cell| inline(cell.trim())).collect() };
            body.push_str("<table>\n<tr>");
            for cell in cells(line) {
                body.push_str(&format!("<th>{cell}</th>"));
            }
            body.push_str("</tr>\n");
            lines.next();
            while let Some(row) = lines.peek().filter(|row| row.starts_with('|')) {
                body.push_str("<tr>");
                for cell in cells(row) {
                    body.push_str(&format!("<td>{cell}</td>"));
                }
                body.push_str("</tr>\n");
                lines.next();
            }
            body.push_str("</table>\n");
        } else if let Some(item) = line.strip_prefix("- ") {
            if open != "ul" {
                close(&mut body, &mut open);
                body.push_str("<ul>\n");
                open = "ul";
            }
            body.push_str(&format!("<li>{}</li>\n", inline(item)));
        } else if let Some((number, item)) = line.split_once(". ").filter(|(number, _)| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit())) {
            let _ = number;
            if open != "ol" {
                close(&mut body, &mut open);
                body.push_str("<ol>\n");
                open = "ol";
            }
            body.push_str(&format!("<li>{}</li>\n", inline(item)));
        } else if line.is_empty() {
            // A blank line ends a list, unless the list goes on after it.
            let goes_on = lines.peek().is_some_and(|next| (open == "ul" && next.starts_with("- ")) || (open == "ol" && next.chars().next().is_some_and(|c| c.is_ascii_digit())));
            if !goes_on {
                close(&mut body, &mut open);
            }
        } else {
            close(&mut body, &mut open);
            match lead(line).filter(|_| in_list) {
                Some(name) => {
                    count += 1;
                    body.push_str(&format!("<p id=\"b{count}\">{}</p>\n", inline(line)));
                    list.push_str(&format!("<li><a href=\"#b{count}\">{}</a></li>", inline(name)));
                }
                None => body.push_str(&format!("<p>{}</p>\n", inline(line))),
            }
        }
    }
    close(&mut body, &mut open);
    if in_list {
        list.push_str("</ul></li>\n");
    }
    let html = PAGE.replace("{title}", &escape(title)).replace("{name}", &name).replace("{contents}", &escape(contents)).replace("{list}", &list).replace("{body}", &body);
    (html, animation)
}

const PAGE: &str = r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
  body { margin: 0; font: 16px/1.6 "Noto Sans", "Liberation Sans", "DejaVu Sans", Arial, sans-serif; color: #1c2438; background: #f4f6fa; }
  main { max-width: 860px; margin: 0 auto; padding: 24px 28px 80px; background: #fff; }
  h1 { font-size: 30px; margin: 8px 0 16px; }
  h2 { font-size: 22px; margin: 40px 0 10px; padding-top: 12px; border-top: 2px solid #1e78d2; }
  p, li { margin: 8px 0; }
  code { background: #eef2f7; padding: 1px 5px; border-radius: 4px; font-size: 14px; }
  pre { background: #1c2438; color: #f4f6fa; padding: 12px 16px; border-radius: 8px; font: 14px/1.5 "DejaVu Sans Mono", "Liberation Mono", monospace; overflow-x: auto; }
  table { border-collapse: collapse; margin: 12px 0; width: 100%; }
  th, td { border: 1px solid #c8d0dc; padding: 5px 9px; text-align: left; vertical-align: top; }
  th { background: #eef2f7; }
  nav { background: #eef2f7; border-radius: 8px; padding: 12px 18px; margin: 16px 0 28px; }
  nav > b { font-size: 18px; }
  nav > ul { columns: 2; column-gap: 28px; padding-left: 18px; margin: 8px 0 0; }
  nav li { margin: 2px 0; break-inside: avoid; }
  nav ul ul { padding-left: 16px; font-size: 14px; }
  a { color: #1e78d2; text-decoration: none; }
  a:hover { text-decoration: underline; }
  :target { background: #fff6c8; }
</style>
</head>
<body>
<main>
<h1>{name}</h1>
<nav><b>{contents}</b>
<ul>
{list}</ul>
</nav>
{body}</main>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes the guide out, to look at it in a browser: `cargo test -- --ignored guide_page`.
    #[test]
    #[ignore]
    fn guide_page() {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/target/kilavuz.html"), page(Language::Turkish, "Kullanım Kılavuzu", "İçindekiler").0).unwrap();
    }

    #[test]
    fn the_guide_becomes_a_page_with_its_lessons_listed() {
        for language in [Language::Turkish, Language::English] {
            let (html, animation) = page(language, "Guide", "Contents");
            // Every heading is in the list and can be gone to; the part about animation is found.
            assert!(html.matches("<h2 id=").count() > 20 && html.contains(&format!("<h2 id=\"{animation}\">")), "{animation}");
            assert!(!animation.is_empty());
            // Tables, lists and numbered steps are there, and no mark of the Markdown is left over.
            assert!(html.contains("<table>") && html.contains("<ol>") && html.contains("<ul>") && html.contains("<code>"));
            assert!(!html.contains("**") && !html.contains("\n|") && !html.contains("<p>- "), "Markdown left in the page");
            // The marks are balanced: what is opened is closed.
            assert!(html.contains("<pre>") && !html.contains("```"));
            for tag in ["b", "i", "code", "ul", "ol", "table", "p", "pre"] {
                assert_eq!(html.matches(&format!("<{tag}>")).count() + html.matches(&format!("<{tag} ")).count(), html.matches(&format!("</{tag}>")).count(), "{tag}");
            }
        }
        // The lessons are in the list by name.
        let (html, _) = page(Language::Turkish, "x", "y");
        for lesson in ["Ders: elma yakala", "Ders: platform oyunu", "Ders: uzay atışı", "Vitrin oyunu: Pisi: Demir Pençe", "Oyunu canlandırmak"] {
            assert!(html.contains(&format!("\">{lesson}</a>")), "{lesson}");
        }
    }

    #[test]
    fn a_line_keeps_its_bold_italic_and_code() {
        assert_eq!(inline("**Bold** and *lean*, `a*b` <tag>"), "<b>Bold</b> and <i>lean</i>, <code>a*b</code> &lt;tag&gt;");
        assert_eq!(lead("**Lesson: catch the apples** (the example"), Some("Lesson: catch the apples"));
        assert_eq!(lead("**Turning at a joint.** A motion tween"), Some("Turning at a joint"));
        assert_eq!(lead("A plain line"), None);
    }
}
