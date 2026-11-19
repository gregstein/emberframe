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

//! The written language in Turkish. The language of the interface is one for the whole
//! program, so what is tried in Turkish is tried here, in a run of its own.

use pisiworks::i18n::{set_language, Language};
use pisiworks::script::{read, strangers, write};

#[test]
fn every_game_reads_back_from_its_own_turkish_text() {
    set_language(Language::Turkish);
    let mut games = 0;
    for example in pisiworks::sahne_examples::EXAMPLES {
        let doc = (example.build)();
        if doc.rules.is_empty() {
            continue;
        }
        games += 1;
        let written = write(&doc.variables, &doc.commands, &doc.rules);
        let found = read(&written).unwrap_or_else(|mistakes| panic!("{}: {mistakes:?}\n{written}", example.name));
        assert_eq!(found.rules, doc.rules, "{}\n{written}", example.name);
        assert_eq!(found.variables, doc.variables, "{}", example.name);
        let sounds: Vec<String> = doc.sounds.iter().map(|s| s.name.clone()).collect();
        assert!(strangers(&doc.rules, &doc.commands, &doc.object_names(), &sounds).is_empty(), "{}", example.name);
    }
    assert_eq!(games, 7, "the showcase, the five small games and the car");
}

#[test]
fn a_game_written_in_turkish_is_read() {
    set_language(Language::Turkish);
    let text = "\
Sayılar:
    Puan = 0
    Can = 3

Elma nesnesi Sepet nesnesine değince:
    Puan sayısını 1 artır
    Elma nesnesini üstte rastgele bir yere koy

Kahraman nesnesi Kapı nesnesine değince, Puan sayısı 2 değerinden büyükse ve Can sayısı 0 değerinden büyükse:
    zafer sesini çal
    1 saniye bekle
    2. sahneye git
değilse:
    Kahraman nesnesini Kahraman.x - 40, 262 konumuna koy

Boşluk tuşuna basılınca:
    Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur
    Puan sayısını Puan * 2 + rastgele(1, 6) yap
";
    let found = read(text).unwrap();
    assert_eq!((found.variables.len(), found.rules.len()), (2, 3));
    let door = &found.rules[1];
    assert_eq!(door.conditions().count(), 2);
    assert_eq!((door.steps.len(), door.otherwise.len(), door.otherwise[0].ex.as_str()), (3, 1, "Kahraman.x - 40"));
    assert_eq!(found.rules[2].steps[1].ex, "Puan * 2 + rastgele(1, 6)");
    // Written out again it is the same text.
    assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
    // What is wrong is told in Turkish, with its line.
    let told = read("Elma dans edince:\n    Elma nesnesini gizle\n").unwrap_err();
    assert_eq!((told[0].line, told[0].what.as_str()), (1, "bu, bildiğim bir olay değil"));
    let told = read("Film başlayınca:\n    Puan sayısını 1 artır\n").unwrap_err();
    assert_eq!((told[0].line, told[0].what.as_str()), (2, "Puan adında bir sayı yok"));
}

#[test]
fn loops_and_commands_read_in_turkish() {
    set_language(Language::Turkish);
    let text = "\
Sayılar:
    Puan = 0
    Atış = 3

Patlat komutu:
    Patlama nesnesinin kopyasını Düşman nesnesinin yerinde oluştur
    bum sesini çal

Boşluk tuşuna basılınca:
    Atış kez yinele:
        Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur
        0.2 saniye bekle
    eğer Puan sayısı 10 değerinden büyükse:
        Patlat komutunu yap
        Puan sayısı 0 değerinden büyükse yinele:
            Puan sayısını 3 azalt
    değilse:
        vız sesini çal
";
    let found = read(text).unwrap();
    assert_eq!((found.commands[0].name.as_str(), found.rules[0].steps.len()), ("Patlat", 2));
    let choice = &found.rules[0].steps[1];
    assert_eq!((choice.body.len(), choice.otherwise.len(), choice.body[1].body.len()), (2, 1, 1));
    assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
    let told = read("Film başlayınca:\n    Zıpla komutunu yap\n").unwrap_err();
    assert_eq!((told[0].line, told[0].what.as_str()), (2, "Zıpla adında bir komut yok"));
}

#[test]
fn what_is_typed_first_decides_what_is_offered() {
    use pisiworks::script_edit::{Names, ScriptEdit};
    set_language(Language::Turkish);
    let names = Names { objects: vec!["Elma".into(), "Sepet".into()], numbers: vec!["Puan".into()], commands: vec!["Patlat".into()], sounds: vec!["bum".into()] };
    let offered = |line: &str| {
        let mut edit = ScriptEdit::new("Film başlayınca:");
        edit.end();
        edit.newline();
        edit.insert(line);
        edit.offers(&names).iter().map(|offer| offer.shown().to_string()).collect::<Vec<_>>()
    };
    // For an object, the steps about objects, in its own capitals however it was typed.
    let apple = offered("elma");
    assert!(apple.iter().filter(|s| s.contains(' ')).all(|sentence| sentence.starts_with("Elma nesnesi")), "{apple:?}");
    // For a number, those about numbers and those that open with an amount; none about objects.
    let score = offered("puan");
    assert!(score.contains(&"Puan sayısını ? artır".to_string()) && score.contains(&"Puan kez yinele:".to_string()) && !score.iter().any(|s| s.contains("nesnesini")), "{score:?}");
    // For a command, the one sentence that does it; for a figure, what opens with one; for a sound, playing it.
    assert!(offered("patlat").contains(&"Patlat komutunu yap".to_string()) && offered("patlat").iter().filter(|s| s.contains(' ')).all(|s| s.starts_with("Patlat komutunu")));
    assert!(offered("3").contains(&"3 kez yinele:".to_string()) && offered("3").contains(&"3 saniye bekle".to_string()));
    assert_eq!(offered("bum"), vec!["bum sesini çal"]);
    // Further into a sentence, what still fits it; and the beginning of a name brings the name.
    assert!(offered("Elma nesnesini g").contains(&"Elma nesnesini gizle".to_string()));
    assert_eq!(offered("Puan sayısını Pu"), vec!["Puan"]);
    assert!(!offered("Puan sayısını Puan").contains(&"Puan".to_string()), "a name that is whole is not offered again");
    assert_eq!(offered("Elma nesnesi Se")[0], "Sepet");
    // At the edge, the events: a key's name brings the event of that key.
    let mut edit = ScriptEdit::new("");
    edit.insert("boşluk");
    assert_eq!(edit.offers(&names)[0].shown(), "Boşluk tuşuna basılınca:");
}

#[test]
fn a_command_takes_values_in_turkish() {
    set_language(Language::Turkish);
    let text = "\
Sayılar:
    Puan = 0

Ödül komutu, Kaç, Kim ile:
    Puan sayısını Kaç * 2 artır
    Kim nesnesini bir an yanıp söndür

Altın nesnesine tıklanınca:
    Ödül komutunu 10, Altın ile yap
";
    let found = read(text).unwrap();
    assert_eq!((found.commands[0].name.as_str(), found.commands[0].params.len()), ("Ödül", 2));
    assert_eq!(found.rules[0].steps[0].args, vec!["10", "Altın"]);
    assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
    let told = read("Ödül komutu, Kaç ile:\n    filmi durdur\n\nFilm başlayınca:\n    Ödül komutunu yap\n").unwrap_err();
    assert_eq!((told[0].line, told[0].what.as_str()), (5, "Ödül komutu 1 değer alır, 0 değer verilmiş"));
}

#[test]
fn turning_and_giving_behaviors_read_in_turkish() {
    set_language(Language::Turkish);
    let text = "\
Düğme nesnesine tıklanınca:
    Elips nesnesini 45 derece döndür
    Elips nesnesi saniyede 2 tur dönsün
    Elips nesnesini yüzde 150 boyuta getir
    Elips nesnesini yüzde 40 opak yap
    Elips nesnesine Dön davranışını ver
    Top nesnesine Kenarlardan Sek davranışını ver
    Top nesnesinden Düş ve Zıpla davranışını al
    Kedi nesnesi Fare nesnesini 120 hızla izlesin
";
    let found = read(text).unwrap();
    let steps = &found.rules[0].steps;
    assert_eq!(steps.iter().map(|s| s.b.as_str()).collect::<Vec<_>>(), vec!["", "", "", "", "spin", "ricochet", "gravity", "Fare"]);
    assert_eq!((steps[0].x, steps[1].x, steps[2].x, steps[3].x, steps[7].x), (45.0, 2.0, 150.0, 40.0, 120.0));
    assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
    let told = read("Film başlayınca:\n    Top nesnesine Uç davranışını ver\n").unwrap_err();
    assert_eq!((told[0].line, told[0].what.as_str()), (2, "Uç adında bir davranış yok"));
}

#[test]
fn a_car_is_driven_in_turkish() {
    set_language(Language::Turkish);
    let text = "\
Film başlayınca:
    Tekerlek nesnesini Araba nesnesine bağla

Sağ Ok tuşuna basılınca:
    Araba nesnesi sağa 300 hızla gitsin

Sağ Ok tuşu bırakılınca:
    Araba nesnesi yavaşlayarak dursun

Boşluk tuşuna basılınca:
    Araba nesnesi dursun
    Tekerlek nesnesini serbest bırak
";
    let found = read(text).unwrap();
    assert_eq!(found.rules.iter().map(|rule| rule.event.kind.key()).collect::<Vec<_>>(), vec!["start", "key", "keyup", "key"]);
    assert_eq!(found.rules.iter().flat_map(|rule| &rule.steps).map(|step| step.kind.key()).collect::<Vec<_>>(), vec!["attach", "go", "slow", "stand", "detach"]);
    assert_eq!((found.rules[1].steps[0].b.as_str(), found.rules[1].steps[0].x), ("right", 300.0));
    assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
    let told = read("Film başlayınca:\n    Araba nesnesi ileri 300 hızla gitsin\n").unwrap_err();
    assert_eq!((told[0].line, told[0].what.as_str()), (2, "ileri bir yön değil: sağa, sola, yukarı ya da aşağı yazılır"));
}

#[test]
fn the_camera_is_told_in_turkish() {
    set_language(Language::Turkish);
    let text = "\
Film başlayınca:
    kamera Araba nesnesini izlesin
    Bulut nesnesine Arka Plan Gibi Kay davranışını ver

Boşluk tuşuna basılınca:
    kamera yerinde dursun
";
    let found = read(text).unwrap();
    assert_eq!(found.rules.iter().flat_map(|rule| &rule.steps).map(|step| (step.kind.key(), step.a.as_str(), step.b.as_str())).collect::<Vec<_>>(), vec![("camera", "Araba", ""), ("give", "Bulut", "scenery"), ("camera-stop", "", "")]);
    assert_eq!(write(&found.variables, &found.commands, &found.rules).trim(), text.trim());
}
