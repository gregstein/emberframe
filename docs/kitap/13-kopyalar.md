@kisim 3. Oyun kuralları
# Kopyalar: mermi, düşman, patlama
> Bir uzay gemisi yüz mermi atabilir; ama sen yüz mermi çizmezsin. Bir tane çizersin, oyun gerektikçe onun kopyasını çıkarır.
:: ⏱ 25 dakika | 🎯 12. bölümü bitirenler için | 🧰 Örnekler > Oyun: uzay atışı

![Boşluk tuşuna her basışta geminin önünde yeni bir mermi belirir.](13-film.png "Uzay atışı oyunu: gemi, üstünde sarı bir mermi, yukarıda uzaylı" 640){18-uzay.html 640x400}

!goal Bu bölümde öğreneceklerin
**Nesnenin kopyasını oluştur** eylemini, asıl nesneyi gizlemeyi, kopyaları silmeyi ve tuş olayını öğreneceksin.

## Fikir

| Adım | Ne yaparsın |
| 1 | Mermiyi **bir kez** çizer, ona ad ve davranış verirsin. |
| 2 | Film başlarken o asıl mermiyi **gizlersin**. |
| 3 | Tuşa basılınca asıl merminin **kopyasını** geminin olduğu yerde oluşturursun. |
| 4 | Kopya, asıl merminin adını ve davranışlarını taşır: kendi kendine yukarı gider. |
| 5 | İşi biten kopyayı (ekrandan çıkan, hedefi vuran) **silersin**. |

## Örneği incele

1. {{Örnekler > Oyun: uzay atışı}}'nı aç.

![Sahnede asıl nesneler duruyor: ((1)) asıl mermi, ((2)) asıl patlama. Oyunda bunlar görünmez; yalnızca kopyaları görünür.](13-sahne.png "Uzay atışı sahnesi; geminin üstünde mermi ve ortada patlama işaretli" 670)

2. Mermiye tıkla ve {{Davranışlar > Davranış Ayarları…}}'na bak: **Sürekli Git** davranışı var, yönü 90 (yukarı).
3. {{Kurallar > Kurallar ve Sayılar…}}'ı aç.

![Yedi kural. İşaretli satır ateş etme kuralı.](13-kurallar.png "Uzay atışı oyununun yedi kuralı; ikinci kural işaretli")

Kopyalarla ilgili dört kural:

| Kural | Ne yapar |
| Film başlayınca: Mermi nesnesini gizle; Patlama nesnesini gizle | Asılları sakla. |
| Boşluk tuşuna basılınca: Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur; vız sesini çal | Ateş et. |
| Mermi nesnesi sahnenin üst kenarına değince: Mermi nesnesini sil | Kaçan mermiyi temizle. |
| Mermi nesnesi Uzaylı nesnesine değince: … Patlama nesnesinin kopyasını Uzaylı nesnesinin yerinde oluştur … | Vurulan yerde patlama çıkar. |

!why Gizlemek ile silmek arasındaki fark
**Gizle**, nesneyi görünmez yapar ama nesne durur; **göster** ile geri gelir. **Sil**, bir kopyayı tümüyle yok eder. Asıl nesneyi gizlersin (kopyası çıkarılacak, lazım); kopyaları silersin (işleri bitti). Silmezsen ekrandan çıkan yüzlerce mermi birikir ve film yavaşlar.

## Kendin yap: ateş eden kutu

1. Yeni bir filmde altta bir kutu çiz; adı **Gemi**, davranışı **Ok Tuşlarıyla Yürü**.
2. Küçük, ince bir dikdörtgen çiz; adı **Mermi**. Ona **Sürekli Git** ver ve Davranış Ayarları'nda yönünü **90**, hızını **400** yap.
3. Şu üç kuralı kur:

| Olay | Eylem |
| Film başlayınca | Mermi nesnesini gizle |
| Boşluk tuşuna basılınca | Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur |
| Mermi nesnesi sahnenin üst kenarına değince | Mermi nesnesini sil |

4. {{Filmi Test Et}}: yürü ve [[Boşluk]]'a bas.

### Tuş olayını kurmak

{{Yeni Kural}}'dan {{Bir tuşa basılınca}}'yı seçince yanında bir tuş listesi açılır: Boşluk, ok tuşları, Enter, harfler ve rakamlar. Tuşu bırakma anı için ayrı bir olay vardır: {{Bir tuş bırakılınca}}.

## Kendi kendine yok olan kopyalar

Patlama gibi kısa ömürlü şeyleri silmek için kural yazmana gerek yok. Asıl nesneye **Bir Süre Sonra Kaybol** davranışını ver (örneğin 0.4 saniye): her kopya doğduktan o kadar süre sonra kendiliğinden silinir.

| İstediğin | Asıl nesneye ver |
| Patlama | Büyü Küçül + Dön + Bir Süre Sonra Kaybol |
| Yükselen "+1" | İn Çık + Belir Kaybol + Bir Süre Sonra Kaybol |
| Düşen yağmur damlası | Düş ve Zıpla (zıplama hızı 0) |

## Düzenli gelen düşmanlar

Kopyalar yalnızca tuşla çıkmaz. *Her 2 saniyede bir: Düşman nesnesinin kopyasını Kapı nesnesinin yerinde oluştur* kuralı, kapıdan durmadan düşman çıkarır.

!stuck Takıldıysan
- **Kopya çıkmıyor.** Asıl nesnenin ve yer veren nesnenin (Gemi) adlarını kontrol et.
- **Kopya çıkıyor ama kıpırdamıyor.** Davranışı asıl nesneye vermemişsin. Kopya, asıl nesnede ne varsa onu taşır.
- **Asıl mermi de ekranda duruyor.** "Film başlayınca: Mermi nesnesini gizle" kuralı eksik.
- **Film gittikçe yavaşlıyor.** Kopyalar silinmiyor. Kenara değince silen kuralı ya da Bir Süre Sonra Kaybol'u ekle.

## Sıra sende

- [ ] Uzay atışı oyununda mermiyi hızlandır.
- [ ] Ateş eden kutu oyununa yukarıda **Git Gel** yapan bir "Hedef" ekle. Mermi hedefe değince puan artsın ve mermi silinsin.
- [ ] Her 3 saniyede bir sahnenin üstünde bir "Taş" kopyası çıksın ve aşağı düşsün.
- [ ] Zor soru: mermi hedefe değince hedefin olduğu yerde bir patlama çıksın ve kendi kendine kaybolsun.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Asıl nesne | Bir kez çizilen, başta gizlenen, kopyaları çıkarılan nesne. |
| Kopya | Oyun sırasında asıl nesneden çıkarılan, onun adını ve davranışlarını taşıyan nesne. |
| Sil | Bir kopyayı tümüyle yok eden eylem. |

!teacher Öğretmene not
Kopya kavramı, öğrencilerin "her mermiyi ayrı ayrı çizmem mi gerekiyor?" sorusunun yanıtıdır ve bir kez oturunca oyunlar hızla zenginleşir. Asıl nesnenin sahnede görünür durması başta kafa karıştırır; "kalıp" benzetmesi işe yarar: kalıp tezgâhta durur, kurabiyeler ondan çıkar. Asılları sahnenin dışındaki gri alana (çalışma alanına) koymak da mümkündür ve büyük oyunlarda sahneyi temiz tutar. "Film yavaşlıyor" sorunu gerçek bir programlama dersidir: kullanılmayan şeyler temizlenmelidir.
