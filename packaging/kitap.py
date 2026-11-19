#!/usr/bin/env python3
# This file is part of PisiWorks.
# Copyright (C) 2026 Murat SAPMAZ
#
# PisiWorks is free software: you can redistribute it and/or modify it under the terms of the
# GNU General Public License as published by the Free Software Foundation, either version 3 of
# the License, or (at your option) any later version.
#
# PisiWorks is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
# even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
# General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with PisiWorks. If
# not, see <https://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: GPL-3.0-or-later

"""Builds the PisiAnimasyon book from its chapters.

    packaging/kitap.py [out-dir] [--tek]

The chapters are the files docs/kitap/NN-name.md, in the order of their names, and
their pictures are in docs/kitap/img/. Each chapter becomes a page, and index.html
lists them by part. The book is built twice over: once with pictures only, and once
(the pages named -canli) with the films themselves, playing, where a picture has one:
the films are in docs/kitap/film/, published from PisiAnimasyon. With --tek the pictures are written into the pages themselves,
so that a page is one file that can be sent or opened anywhere.

A chapter is written in a small Markdown of its own:

    @kisim 1. Başlarken       the part the chapter belongs to
    # Title
    > The sentence under the title
    :: 10 dakika | ilk kez açanlar | yalnızca PisiAnimasyon
    ## Heading            ### Smaller heading
    1. A step             (numbered steps, with round numbers)
    - A point             - [ ] A thing to try
    | a | b |             a table; its first row is the head
    ![Caption](name.png "what the picture shows" 600)
    ![Caption](name.png "what it shows" 600){name.html 660x500}
                          a picture that, in the live book, is the film itself
    !goal Title           a box, until the next empty line; also !why !stuck !teacher !note
    ```                   lines shown as they are written

    In a line: **bold**, *leaning*, `as typed`, [[Ctrl]] a key, {{Menu > Item}}
    a thing to click, ((1)) a number as it is on a picture.
"""
import base64
import html
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "docs" / "kitap"

CSS = """
:root { --ink: #1c2438; --dim: #5a6478; --line: #d5dbe6; --blue: #1e78d2; --orange: #e85a14; --green: #2c8a3c; --paper: #fff; --back: #eef1f6; }
* { box-sizing: border-box; }
body { margin: 0; background: var(--back); color: var(--ink); font: 18px/1.65 "Noto Sans", "Liberation Sans", "DejaVu Sans", Arial, sans-serif; }
main { max-width: 840px; margin: 0 auto; background: var(--paper); padding: 28px 48px 80px; }
nav.top, nav.bottom { display: flex; justify-content: space-between; gap: 12px; font-size: 15px; }
nav.top { border-bottom: 1px solid var(--line); padding-bottom: 10px; margin-bottom: 28px; }
nav.bottom { border-top: 1px solid var(--line); padding-top: 14px; margin-top: 48px; }
a { color: var(--blue); text-decoration: none; }
a:hover { text-decoration: underline; }
.chapter { color: var(--blue); font-weight: 700; letter-spacing: .06em; text-transform: uppercase; font-size: 15px; margin: 0; }
h1 { font-size: 36px; line-height: 1.2; margin: 4px 0 8px; }
.lead { font-size: 20px; color: var(--dim); margin: 0 0 24px; }
h2 { font-size: 24px; margin: 44px 0 8px; padding-top: 14px; border-top: 3px solid var(--blue); }
h3 { font-size: 19px; margin: 28px 0 4px; }
p { margin: 10px 0; }
.facts { display: flex; flex-wrap: wrap; gap: 10px; margin: 18px 0 8px; padding: 0; list-style: none; }
.facts li { background: var(--back); border-radius: 999px; padding: 4px 14px; font-size: 15px; }
.box { border-radius: 10px; padding: 14px 18px; margin: 20px 0; border-left: 6px solid; }
.box > b:first-child { display: block; margin-bottom: 4px; }
.box p { margin: 6px 0; }
.goal { background: #e9f3ff; border-color: var(--blue); }
.why { background: #eaf7ec; border-color: var(--green); }
.stuck { background: #fff1e8; border-color: var(--orange); }
.teacher { background: #f3f0fa; border-color: #7040c0; font-size: 16px; }
.note { background: #f4f6fa; border-color: var(--dim); }
ol.steps { counter-reset: step; list-style: none; padding: 0; margin: 14px 0; }
ol.steps > li { counter-increment: step; position: relative; padding: 6px 0 6px 52px; margin: 10px 0; }
ol.steps > li::before { content: counter(step); position: absolute; left: 0; top: 4px; width: 36px; height: 36px; border-radius: 50%; background: var(--blue); color: #fff; font-weight: 700; display: flex; align-items: center; justify-content: center; }
figure { margin: 16px 0 22px; }
figure img { display: block; max-width: 100%; height: auto; border: 1px solid var(--line); border-radius: 8px; box-shadow: 0 2px 8px rgba(28, 36, 56, .12); }
figcaption { font-size: 15px; color: var(--dim); margin-top: 6px; }
figure.live iframe { display: block; width: 100%; height: auto; border: 2px solid var(--blue); border-radius: 8px; background: #222; box-shadow: 0 2px 8px rgba(28, 36, 56, .12); }
figure.live .play { display: block; color: var(--blue); font-weight: 700; }
nav.top .switch { font-size: 14px; }
.n { display: inline-flex; align-items: center; justify-content: center; width: 22px; height: 22px; border-radius: 50%; background: var(--orange); color: #fff; font-size: 13px; font-weight: 700; vertical-align: 2px; }
kbd { font: inherit; font-size: 15px; background: #f4f6fa; border: 1px solid var(--line); border-bottom-width: 3px; border-radius: 6px; padding: 0 7px; white-space: nowrap; }
code { background: #eef2f7; padding: 1px 5px; border-radius: 4px; font: 15px "DejaVu Sans Mono", "Liberation Mono", monospace; }
pre { background: #1c2438; color: #f4f6fa; padding: 14px 18px; border-radius: 8px; font: 15px/1.55 "DejaVu Sans Mono", "Liberation Mono", monospace; overflow-x: auto; }
.menu { font-weight: 700; }
table { border-collapse: collapse; width: 100%; margin: 12px 0; font-size: 16px; }
th, td { border: 1px solid var(--line); padding: 7px 10px; text-align: left; vertical-align: top; }
th { background: var(--back); }
ul.check { list-style: none; padding-left: 0; }
ul.check li { padding-left: 30px; position: relative; margin: 6px 0; }
ul.check li::before { content: "\\2610"; position: absolute; left: 4px; color: var(--blue); font-size: 20px; line-height: 1.3; }
.parts h2 { border-top: none; padding-top: 0; margin-top: 30px; }
.parts ol { padding-left: 0; list-style: none; }
.parts li { margin: 6px 0; padding: 8px 12px; border: 1px solid var(--line); border-radius: 8px; }
.parts li span { color: var(--dim); font-size: 15px; display: block; }
@media (max-width: 640px) { main { padding: 20px 18px 60px; } h1 { font-size: 28px; } }
@media print { body { background: #fff; font-size: 12pt; } main { max-width: none; padding: 0; } nav { display: none !important; } h2 { break-after: avoid; } figure, .box, ol.steps > li { break-inside: avoid; } }
"""

BOXES = {"goal", "why", "stuck", "teacher", "note"}


def inline(text):
    text = html.escape(text, quote=False)
    text = re.sub(r"`([^`]+)`", r"<code>\1</code>", text)
    text = re.sub(r"\[\[([^\]]+)\]\]", r"<kbd>\1</kbd>", text)
    text = re.sub(r"\{\{([^}]+)\}\}", r'<span class="menu">\1</span>', text)
    text = re.sub(r"\(\((\d+)\)\)", r'<span class="n">\1</span>', text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", text)
    text = re.sub(r"\*([^*]+)\*", r"<i>\1</i>", text)
    return text


class Chapter:
    def __init__(self, path, embed, live=False):
        self.path, self.embed, self.live = path, embed, live
        self.part, self.title, self.lead = "", path.stem, ""
        self.body = self.read(path.read_text(encoding="utf-8").splitlines())

    def picture(self, name):
        file = SOURCE / "img" / name
        if not file.exists():
            sys.exit(f"{self.path.name}: there is no picture {name}")
        if self.embed:
            return "data:image/png;base64," + base64.b64encode(file.read_bytes()).decode()
        return "img/" + name

    def read(self, lines):
        out, open_list, at = [], "", 0

        def close():
            nonlocal open_list
            if open_list:
                out.append("</ol>" if open_list == "ol" else "</ul>")
                open_list = ""

        def start(kind, cls=""):
            nonlocal open_list
            want = kind + cls
            if open_list != kind or getattr(start, "last", "") != want:
                close()
                out.append(f'<{kind} class="{cls}">' if cls else f"<{kind}>")
                open_list, start.last = kind, want

        while at < len(lines):
            line = lines[at].rstrip()
            at += 1
            if line.startswith("```"):
                close()
                kept = []
                while at < len(lines) and not lines[at].startswith("```"):
                    kept.append(html.escape(lines[at], quote=False))
                    at += 1
                at += 1
                out.append("<pre>" + "\n".join(kept) + "</pre>")
            elif line.startswith("@kisim "):
                self.part = line[7:].strip()
            elif line.startswith("# "):
                self.title = line[2:].strip()
            elif line.startswith("> "):
                self.lead = line[2:].strip()
            elif line.startswith(":: "):
                close()
                out.append('<ul class="facts">' + "".join(f"<li>{inline(part.strip())}</li>" for part in line[3:].split("|")) + "</ul>")
            elif line.startswith("### "):
                close()
                out.append(f"<h3>{inline(line[4:])}</h3>")
            elif line.startswith("## "):
                close()
                out.append(f"<h2>{inline(line[3:])}</h2>")
            elif line.startswith("!["):
                close()
                found = re.match(r'!\[(.*)\]\((\S+)(?: "([^"]*)")?(?: (\d+))?\)(?:\{(\S+) (\d+)x(\d+)\})?$', line)
                if not found:
                    sys.exit(f"{self.path.name}: a picture is written wrongly: {line}")
                caption, name, alt, width, film, wide, high = found.groups()
                size = f' width="{width}"' if width else ""
                if film and not (SOURCE / "film" / film).exists():
                    sys.exit(f"{self.path.name}: there is no film {film}")
                if film and self.live:
                    # The film itself, in a frame of its own shape; a click gives it the keyboard.
                    out.append(f'<figure class="live"><iframe src="film/{film}" title="{html.escape(alt or caption)}" style="aspect-ratio: {wide} / {high}; max-width: {wide}px" loading="lazy"></iframe><figcaption>{inline(caption)} <span class="play">▶ Bu bir resim değil, filmin kendisi: üstüne tıkla ve oyna.</span></figcaption></figure>')
                    continue
                out.append(f'<figure><img src="{self.picture(name)}" alt="{html.escape(alt or caption)}"{size}><figcaption>{inline(caption)}</figcaption></figure>')
            elif line.startswith("!") and line[1:].split(" ", 1)[0] in BOXES:
                close()
                kind, _, title = line[1:].partition(" ")
                inner = []
                while at < len(lines) and lines[at].strip():
                    inner.append(lines[at].rstrip())
                    at += 1
                out.append(f'<div class="box {kind}"><b>{inline(title)}</b>' + Chapter.block(inner) + "</div>")
            elif line.startswith("|"):
                close()
                rows = [line]
                while at < len(lines) and lines[at].startswith("|"):
                    rows.append(lines[at].rstrip())
                    at += 1
                cells = lambda row: [inline(cell.strip()) for cell in row.strip("|").split("|")]
                table = "<table><tr>" + "".join(f"<th>{c}</th>" for c in cells(rows[0])) + "</tr>"
                for row in rows[1:]:
                    if set(row) <= set("|-: "):
                        continue
                    table += "<tr>" + "".join(f"<td>{c}</td>" for c in cells(row)) + "</tr>"
                out.append(table + "</table>")
            elif line.startswith("- [ ] "):
                start("ul", "check")
                out.append(f"<li>{inline(line[6:])}</li>")
            elif line.startswith("- "):
                start("ul")
                out.append(f"<li>{inline(line[2:])}</li>")
            elif re.match(r"\d+\. ", line):
                start("ol", "steps")
                out.append(f"<li>{inline(line.split('. ', 1)[1])}</li>")
            elif not line:
                close()
            else:
                close()
                out.append(f"<p>{inline(line)}</p>")
        close()
        return "\n".join(out)

    @staticmethod
    def block(lines):
        """The inside of a box: paragraphs and plain points."""
        out, in_list = [], False
        for line in lines:
            if line.startswith("- "):
                if not in_list:
                    out.append("<ul>")
                    in_list = True
                out.append(f"<li>{inline(line[2:])}</li>")
            else:
                if in_list:
                    out.append("</ul>")
                    in_list = False
                out.append(f"<p>{inline(line)}</p>")
        if in_list:
            out.append("</ul>")
        return "".join(out)


def page(title, body):
    return f'<!doctype html>\n<html lang="tr">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width, initial-scale=1">\n<title>{html.escape(title)}</title>\n<style>{CSS}</style>\n</head>\n<body>\n<main>\n{body}\n</main>\n</body>\n</html>\n'


def edition(out, embed, live):
    """One way of the book: with pictures only, or with the films playing. The pages of the second are named -canli."""
    mark = "-canli" if live else ""
    files = sorted(SOURCE.glob("[0-9]*.md"))
    chapters = [Chapter(file, embed, live) for file in files]
    other = "Resimli sürüm" if live else "Canlı örnekli sürüm"
    switch = lambda stem: "" if embed else f'<a class="switch" href="{stem}{"" if live else "-canli"}.html">{other}</a>'
    for number, chapter in enumerate(chapters, 1):
        before = chapters[number - 2] if number > 1 else None
        after = chapters[number] if number < len(chapters) else None
        link = lambda to, text: f'<a href="{to.path.stem}{mark}.html">{text}</a>' if to else "<span></span>"
        top = f'<nav class="top"><a href="index{mark}.html">PisiAnimasyon Kitabı</a><span>{html.escape(chapter.part)}</span>{switch(chapter.path.stem)}</nav>'
        bottom = '<nav class="bottom">' + link(before, f"← {html.escape(before.title)}" if before else "") + link(after, f"Sıradaki: {html.escape(after.title)} →" if after else "") + "</nav>"
        head = f'<p class="chapter">Bölüm {number}</p><h1>{inline(chapter.title)}</h1>' + (f'<p class="lead">{inline(chapter.lead)}</p>' if chapter.lead else "")
        (out / f"{chapter.path.stem}{mark}.html").write_text(page(f"{chapter.title} – PisiAnimasyon Kitabı", top + head + chapter.body + bottom), encoding="utf-8")
    # The contents: the chapters under their parts.
    about = "Animasyon ve oyun yapmayı adım adım, resimlerle anlatan kitap. Her bölüm bir ders saatine sığar ve bir önceki bölümün bıraktığı yerden devam eder."
    note = "Bu sürümde örnekler resim değil, filmin kendisidir: üstüne tıklayıp oynayabilirsin. Yazdırmak için resimli sürümü kullan." if live else "Bu sürümde örnekler resimdir; yazdırmaya uygundur. Canlı örnekli sürümde aynı örnekler oynanabilir."
    listing, part = [f'<nav class="top"><span>PisiAnimasyon</span>{switch("index")}</nav><h1>PisiAnimasyon Kitabı</h1><p class="lead">{about}</p><div class="box note"><b>{"Canlı örnekli sürüm" if live else "Resimli sürüm"}</b><p>{note}</p></div>', '<div class="parts">'], None
    for number, chapter in enumerate(chapters, 1):
        if chapter.part != part:
            if part is not None:
                listing.append("</ol>")
            part = chapter.part
            listing.append(f"<h2>{html.escape(part)}</h2><ol>")
        listing.append(f'<li><a href="{chapter.path.stem}{mark}.html"><b>{number}. {html.escape(chapter.title)}</b></a><span>{inline(chapter.lead)}</span></li>')
    listing.append("</ol></div>")
    (out / f"index{mark}.html").write_text(page("PisiAnimasyon Kitabı", "\n".join(listing)), encoding="utf-8")
    return len(chapters)


def build(out, embed):
    out.mkdir(parents=True, exist_ok=True)
    count = edition(out, embed, False)
    if not embed:
        # The live book stands beside the other and shares its pictures; a page that is one file has no films.
        edition(out, embed, True)
        for folder, kind in (("img", "*.png"), ("film", "*.html")):
            (out / folder).mkdir(exist_ok=True)
            for file in (SOURCE / folder).glob(kind):
                (out / folder / file.name).write_bytes(file.read_bytes())
    print(f"{count} chapters in {out}")


if __name__ == "__main__":
    args = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
    build(pathlib.Path(args[0]) if args else ROOT / "target" / "kitap", "--tek" in sys.argv)
