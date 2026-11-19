#!/bin/sh
# This file is part of PisiWorks.
# Copyright (C) 2026 Murat SAPMAZ
# SPDX-License-Identifier: GPL-3.0-or-later

# Gathers every library (crate) the programs are built from into cargo-vendor/, and packs that folder as
# target/deb/pisiworks_VERSION.orig-cargo-vendor.tar.xz, so that the package can be built with no network:
# debian/rules builds from cargo-vendor/ when it is there. Needs a network itself.
#
#   packaging/vendor-tarball.sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$(dirname -- "$here")"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
rm -rf cargo-vendor
cargo vendor --locked cargo-vendor > /dev/null
mkdir -p target/deb
tar --sort=name --owner=0 --group=0 --numeric-owner -cJf "target/deb/pisiworks_${version}.orig-cargo-vendor.tar.xz" cargo-vendor
echo "target/deb/pisiworks_${version}.orig-cargo-vendor.tar.xz"
