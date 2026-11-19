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

# Builds a .deb package of PisiWorks for Pardus and other Debian-based systems.
#
#   packaging/build-deb.sh
#
# The package lands in target/deb/. It needs cargo, dpkg-deb, strip and objdump.
# The maintainer written into the package is taken from DEB_MAINTAINER, or
# failing that from git's user.name and user.email:
#
#   DEB_MAINTAINER="Name <mail@example.org>" packaging/build-deb.sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(dirname -- "$here")
cd "$root"

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
arch=$(dpkg --print-architecture)
maintainer=${DEB_MAINTAINER:-"$(git config user.name || true) <$(git config user.email || true)>"}
if [ "$maintainer" = " <>" ]; then
    echo "Set DEB_MAINTAINER=\"Name <mail>\" or git's user.name and user.email." >&2
    exit 1
fi

cargo build --release

stage="target/deb/pisiworks_${version}_${arch}"
rm -rf "$stage"
packaging/install-files.sh "$stage"
strip --strip-unneeded "$stage/usr/bin/pisiworks" "$stage/usr/bin/pisianimasyon"

# The license, in the form Debian keeps it in: the work as a whole under the GNU GPL, whose text Debian
# and Pardus keep in one place for every package, and what came from Emberframe under the MIT license.
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: PisiWorks"
    echo "Upstream-Contact: Murat SAPMAZ <https://muratsapmaz.web.app/>"
    echo "Source: https://github.com/MuratSAPMAZ/PisiWorks"
    echo "Comment: PisiWorks is built on Emberframe, https://github.com/gregstein/emberframe,"
    echo " whose code stays under the MIT license of its author. A film or a page that the"
    echo " programs write may be conveyed under any terms. Additional terms under section 7"
    echo " of the GPL keep the author's attribution and the origin, and reserve the names"
    echo " PisiWorks and PisiAnimasyon to unmodified copies: see NOTICE."
    echo
    echo "Files: *"
    echo "Copyright: 2026 Murat SAPMAZ"
    echo "           2026 Med Amine Belbachir (https://medamineweb.com)"
    echo "License: GPL-3+ and Expat"
    echo
    echo "License: GPL-3+"
    echo " This program is free software: you can redistribute it and/or modify it under the"
    echo " terms of the GNU General Public License as published by the Free Software"
    echo " Foundation, either version 3 of the License, or (at your option) any later version."
    echo " ."
    echo " This program is distributed in the hope that it will be useful, but WITHOUT ANY"
    echo " WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A"
    echo " PARTICULAR PURPOSE. See the GNU General Public License for more details."
    echo " ."
    echo " On Debian and Pardus systems the full text of the GNU General Public License"
    echo " version 3 is in /usr/share/common-licenses/GPL-3."
    echo
    echo "License: Expat"
    sed -e '1,/^$/d' -e '/^Copyright/,/^$/d' -e 's/^$/./' -e 's/^/ /' LICENSES/MIT.txt
} > "$stage/usr/share/doc/pisiworks/copyright"
chmod 644 "$stage/usr/share/doc/pisiworks/copyright"

# The oldest C library the program runs with is the newest one it asks for a symbol from.
glibc=$(objdump -T "$stage/usr/bin/pisiworks" | grep -o 'GLIBC_[0-9.]*' | sed 's/GLIBC_//' | sort -V | tail -n 1)
installed=$(du -sk "$stage/usr" | cut -f1)
mkdir -p "$stage/DEBIAN"
cat > "$stage/DEBIAN/control" <<CONTROL
Package: pisiworks
Version: $version
Section: graphics
Priority: optional
Architecture: $arch
Maintainer: $maintainer
Installed-Size: $installed
Depends: libc6 (>= $glibc), libgcc-s1, libxcb1, libxkbcommon0, libxkbcommon-x11-0, libvulkan1
Recommends: mesa-vulkan-drivers, libwayland-client0, fonts-noto-core | fonts-dejavu-core, cups-client, zenity, enchant-2 | hunspell, hunspell-tr
Homepage: https://github.com/MuratSAPMAZ/PisiWorks
Description: Fireworks-like screen design editor with a Turkish interface
 PisiWorks draws vector shapes and edits bitmaps in one docked workspace,
 with pages, layers, states, symbols, live filters, masks and export for
 the web, in the manner of Adobe Fireworks. Its menus and commands are in
 Turkish or English.
 .
 The package also holds PisiAnimasyon, an animation program built on the same
 code: a timeline with keyframes, motion and shape tweens, guide and mask
 layers, published as an HTML5 page.
 .
 It is a fork of Emberframe and contains no Adobe code or artwork.
CONTROL

( cd "$stage" && find usr -type f -exec md5sum {} + > DEBIAN/md5sums )
dpkg-deb --root-owner-group --build "$stage" "target/deb/pisiworks_${version}_${arch}.deb"
echo "target/deb/pisiworks_${version}_${arch}.deb"
