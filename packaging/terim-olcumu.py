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

"""PisiWorks'ün Türkçe metinlerinin başka bir programın metinleriyle ne kadar örtüştüğünü ölçer.

Kullanım:  packaging/terim-olcumu.py KARSILASTIRILACAK_METINLER.txt [--liste]

Karşılaştırılacak dosyada her satırda bir metin bulunur; satır bir sayı ve boşlukla
başlıyorsa (ikili dosyadan çıkarılmış metinlerde olduğu gibi) sayı atlanır. Karşılaştırma
harf büyüklüğüne, "&" kısayol işaretine, sondaki üç noktaya ve iki noktaya bakmaz.

Bu betik hiçbir başka programın metnini içermez; ölçüm, ölçen kişinin kendi elindeki
dosyayla yapılır. --liste, dört ve daha çok sözcüklü ortak metinleri de yazar.
"""
import re
import sys
from pathlib import Path


def norm(text):
    text = re.sub(r"&|\.\.\.\s*$|…\s*$|:\s*$", "", text.split("\t")[0])
    return re.sub(r"\s+", " ", text).strip().lower()


def ours():
    source = (Path(__file__).resolve().parent.parent / "src/i18n/tr.rs").read_text(encoding="utf-8")
    pairs = re.findall(r'\("((?:[^"\\]|\\.)*)", "((?:[^"\\]|\\.)*)"\)', source)
    return [(key, value.replace('\\"', '"')) for key, value in pairs]


def theirs(path):
    found = set()
    for line in Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        head, _, rest = line.partition(" ")
        text = norm(rest if head.isdigit() else line)
        if len(text) >= 2:
            found.add(text)
    return found


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    other = theirs(sys.argv[1])
    mine = ours()
    same = [(key, value) for key, value in mine if norm(value) in other]
    words = lambda value: len(value.split())
    print(f"Türkçe metin sayımız: {len(mine)}")
    print(f"Karşılaştırılan dosyadaki metin sayısı: {len(other)}")
    print(f"Aynı olan: {len(same)} (yüzde {100 * len(same) // max(len(mine), 1)})")
    for name, low, high in [("tek sözcük", 1, 1), ("iki üç sözcük", 2, 3), ("dört-altı sözcük", 4, 6), ("yedi ve daha çok sözcük", 7, 10**6)]:
        print(f"  {name}: {len([1 for _, value in same if low <= words(value) <= high])}")
    longest = max((len(value) for _, value in same), default=0)
    print(f"En uzun ortak metin: {longest} harf")
    if "--liste" in sys.argv:
        for key, value in sorted(same, key=lambda pair: -len(pair[1])):
            if words(value) >= 4:
                print(f"  {value}    <=    {key}")


if __name__ == "__main__":
    main()
