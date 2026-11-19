#!/bin/sh
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

# Puts the files of the package under a folder, as they lie on a system: the two programs, their menu entries,
# icons, guides, the PisiAnimasyon book and the notices. Both packaging/build-deb.sh and debian/rules use it, so
# that the package is the same however it is built. The programs must have been built (cargo build --release).
#
#   packaging/install-files.sh DESTDIR
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$(dirname -- "$here")"
dest=${1:?"usage: packaging/install-files.sh DESTDIR"}
case "$dest" in /*) ;; *) dest="$(pwd)/$dest" ;; esac

install -Dm755 target/release/pisiworks "$dest/usr/bin/pisiworks"
install -Dm644 packaging/pisiworks.desktop "$dest/usr/share/applications/pisiworks.desktop"
# PisiAnimasyon, the animation program built from the same code, comes in the same package.
install -Dm755 target/release/pisianimasyon "$dest/usr/bin/pisianimasyon"
install -Dm644 packaging/pisianimasyon.desktop "$dest/usr/share/applications/pisianimasyon.desktop"
install -Dm644 packaging/pisiworks.xml "$dest/usr/share/mime/packages/pisiworks.xml"
install -Dm644 packaging/pisiworks.svg "$dest/usr/share/icons/hicolor/scalable/apps/pisiworks.svg"
for size in 16 32 48 64 128 256; do
    install -Dm644 "packaging/icons/pisiworks-$size.png" "$dest/usr/share/icons/hicolor/${size}x${size}/apps/pisiworks.png"
    install -Dm644 "packaging/icons/pisianimasyon-$size.png" "$dest/usr/share/icons/hicolor/${size}x${size}/apps/pisianimasyon.png"
done
install -Dm644 docs/guide.tr.md "$dest/usr/share/doc/pisiworks/kilavuz.md"
install -Dm644 docs/guide.md "$dest/usr/share/doc/pisiworks/guide.md"
# The PisiAnimasyon book: its chapters as pages, with their pictures. Help opens it in PisiAnimasyon.
python3 packaging/kitap.py target/kitap
for page in target/kitap/*.html; do
    install -Dm644 "$page" "$dest/usr/share/doc/pisiworks/kitap/$(basename "$page")"
done
for picture in target/kitap/img/*.png; do
    install -Dm644 "$picture" "$dest/usr/share/doc/pisiworks/kitap/img/$(basename "$picture")"
done
# The films the live book plays in its pages.
for film in target/kitap/film/*.html; do
    install -Dm644 "$film" "$dest/usr/share/doc/pisiworks/kitap/film/$(basename "$film")"
done
install -Dm644 examples/solstice-landing.pisi "$dest/usr/share/doc/pisiworks/examples/solstice-landing.pisi"

install -Dm644 NOTICE "$dest/usr/share/doc/pisiworks/NOTICE"

# The notices of the libraries compiled into the program: the list, and every
# library's own license files as its authors ship them.
python3 packaging/third-party.py --texts "$dest/usr/share/doc/pisiworks/third-party-licenses.txt"
gzip -9n "$dest/usr/share/doc/pisiworks/third-party-licenses.txt"
chmod 644 "$dest/usr/share/doc/pisiworks/third-party-licenses.txt.gz"
install -Dm644 docs/THIRD-PARTY.md "$dest/usr/share/doc/pisiworks/THIRD-PARTY.md"
install -Dm644 docs/lisans-ve-telif.md "$dest/usr/share/doc/pisiworks/lisans-ve-telif.md"

