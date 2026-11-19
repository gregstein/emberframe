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
"""Builds PisiAnimasyon's site: its front page and its book.

    packaging/site.py FOLDER

FOLDER is a checkout of the PisiAnimasyon repository, whose pages GitHub serves. What
is there from an earlier build is replaced; the folder's own .git is left alone. The
front page and the README come from docs/site/, the book from docs/kitap/ by way of
packaging/kitap.py, and the license files from the top of this repository.
"""
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def build(folder):
    folder.mkdir(parents=True, exist_ok=True)
    for old in folder.iterdir():
        if old.name == ".git":
            continue
        shutil.rmtree(old) if old.is_dir() else old.unlink()
    subprocess.run([sys.executable, str(ROOT / "packaging" / "kitap.py"), str(folder / "kitap")], check=True)
    for name in ("index.html", "README.md"):
        shutil.copy(ROOT / "docs" / "site" / name, folder / name)
    for name in ("COPYING", "NOTICE"):
        shutil.copy(ROOT / name, folder / name)
    # The front page's own pictures.
    (folder / "img").mkdir(exist_ok=True)
    shutil.copy(ROOT / "docs" / "screenshots" / "pisiworks-calisma-alani.png", folder / "img" / "pisiworks.png")
    # GitHub is told to serve the files as they are, and not to run them through its own site builder.
    (folder / ".nojekyll").write_text("", encoding="utf-8")
    print(f"site in {folder}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    build(pathlib.Path(sys.argv[1]).resolve())
