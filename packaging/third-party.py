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

"""Lists the libraries that are compiled into PisiWorks, with their licenses.

    packaging/third-party.py                 writes docs/THIRD-PARTY.md
    packaging/third-party.py --texts FILE    also writes every library's own license files into FILE

Only what goes into the Linux program is listed: libraries used on other
systems, for building alone or for tests are left out. The texts are taken
from the sources Cargo has downloaded, so the project must have been built once.
"""
import json
import pathlib
import subprocess
import sys

TARGET = "x86_64-unknown-linux-gnu"
root = pathlib.Path(__file__).resolve().parent.parent


def cargo(*args):
    return subprocess.run(["cargo", *args], cwd=root, check=True, capture_output=True, text=True).stdout


def main():
    wanted = set()
    for line in cargo("tree", "--target", TARGET, "-e", "normal", "--prefix", "none", "--format", "{p}").splitlines():
        parts = line.split()
        if len(parts) >= 2 and parts[1].startswith("v"):
            wanted.add((parts[0], parts[1][1:]))
    packages = json.loads(cargo("metadata", "--format-version", "1"))["packages"]
    used = sorted((p for p in packages if (p["name"], p["version"]) in wanted and p["name"] != "pisiworks"), key=lambda p: (p["name"].lower(), p["version"]))

    counts = {}
    for package in used:
        license = (package["license"] or "see its license file").replace("/", " OR ").replace("  ", " ")
        counts[license] = counts.get(license, 0) + 1
    lines = [
        "# Third-party libraries",
        "",
        f"PisiWorks is built from {len(used)} libraries, all of them free software. This list is made by",
        "`packaging/third-party.py` from what Cargo compiles into the Linux program; the libraries used",
        "only on other systems, for building or for tests are not in it. Where a library offers a choice",
        "of licenses, PisiWorks takes it under the most permissive one (MIT or Apache-2.0).",
        "",
        "## By license",
        "",
        "| License | Libraries |",
        "|---|---|",
    ]
    lines += [f"| {license} | {count} |" for license, count in sorted(counts.items(), key=lambda item: -item[1])]
    lines += ["", "## Every library", "", "| Library | Version | License | Source |", "|---|---|---|---|"]
    for package in used:
        source = package.get("repository") or package.get("homepage") or ""
        lines.append(f"| {package['name']} | {package['version']} | {package['license'] or 'see its license file'} | {source} |")
    (root / "docs" / "THIRD-PARTY.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"docs/THIRD-PARTY.md: {len(used)} libraries")

    if "--texts" in sys.argv:
        target = pathlib.Path(sys.argv[sys.argv.index("--texts") + 1])
        missing = []
        with target.open("w", encoding="utf-8") as out:
            out.write("The license files of the libraries compiled into PisiWorks, each as its authors ship it.\n")
            for package in used:
                folder = pathlib.Path(package["manifest_path"]).parent
                files = sorted(f for f in folder.iterdir() if f.is_file() and f.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE")))
                out.write(f"\n\n{'=' * 78}\n{package['name']} {package['version']}  ({package['license']})\n{'=' * 78}\n")
                if not files:
                    missing.append(package["name"])
                    out.write("This library ships no license file of its own; its license is the one named above.\n")
                for file in files:
                    out.write(f"\n--- {file.name} ---\n{file.read_text(encoding='utf-8', errors='replace')}")
        print(f"{target}: {len(used) - len(missing)} with license files, {len(missing)} without ({', '.join(missing[:12])}{'...' if len(missing) > 12 else ''})")


main()
