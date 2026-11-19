@kisim 4. Oyun dersleri
# Oyun dersi: hedef vurma
> Panayırdaki atış oyunu. Hedefler hareket eder, sen fareyle tıklarsın. Süre 20 saniye.
:: ⏱ 35 dakika | 🎯 3. kısmı bitirenler için | 🧰 Örnekler > Oyun: hedef vurma

![Bitmiş oyun.](16-film.png "Hedef vurma: dört hareketli hedef, puan ve geri sayan süre" 640){16-hedef.html 640x400}

!goal Bu derste kullanacakların
Tıklama olayı, geri sayan bir saat (her saniyede bir), her hedefe ayrı bir davranış, rastgele yere koyma ve patlama kopyası.

## Oyunun parçaları

| Nesne | Adı | Davranışı | Not |
| Hedef 1 | Hedef | Git Gel: uzaklık 180, süre 2.2 | Sağa sola gider gelir. |
| Hedef 2 | Hedef | İn Çık: 80; 1.6 | Yukarı aşağı. |
| Hedef 3 | Hedef | Sürekli Git: hız 130, yön 0 | Sahneyi boydan boya geçer. |
| Hedef 4 | Hedef | Sürekli Git: hız 90, yön 200 | Çapraz gider. |
| Patlama | Pat | Büyü Küçül (90; 0.7), Dön (1.4), Bir Süre Sonra Kaybol (0.35) | Vurulan yerde çıkar. |
| Yazılar | | | `Puan: {Puan}` ve `Süre: {Süre}` |

Dört hedefin adı aynı, davranışı farklı. Kural hepsi için tek; ama her biri başka türlü kaçar.

Sayılar: **Puan** (0), **Süre** (20).

## Kurallar

```
Film başlayınca:
    Pat nesnesini gizle

Hedef nesnesine tıklanınca:
    Puan sayısını 1 artır
    Pat nesnesinin kopyasını Hedef nesnesinin yerinde oluştur
    Hedef nesnesini rastgele bir yere koy
    Hedef nesnesini yüzde 8 hızlandır
    bum sesini çal

Her 1 saniyede bir:
    Süre sayısını 1 azalt

Süre sayısı 1 değerinin altına inince:
    zafer sesini çal
    2. sahneye git

Yeniden nesnesine tıklanınca:
    baştan başla
```

## Yapım sırası

1. **Tek hedef.** Bir hedef çiz (iç içe üç daire; gruplayıp adını "Hedef" koy), ona Git Gel ver. Test et.
2. **Tıklama kuralı.** Puan sayısı ve yazısı; "Hedef nesnesine tıklanınca: Puan sayısını 1 artır; Hedef nesnesini rastgele bir yere koy". Test et.
3. **Saat.** Süre sayısı ve yazısı; "Her 1 saniyede bir: Süre sayısını 1 azalt". Test et: saat geri sayıyor mu?
4. **Bitiş.** İkinci sahne ("Süre doldu"), Durdur kare eylemleri, son iki kural.
5. **Öbür hedefler.** Hedefi üç kez kopyala; her kopyanın davranışını kaldırıp başka bir davranış ver.
6. **Süsler.** Patlama, hızlanma, sesler.

!why Hedef iç içe dairelerden oluşuyor; hangisine tıklamalıyım?
Hiçbirine ayrı ayrı. Daireleri **grupla** ve adı gruba ver: tıklama, grubun herhangi bir yerine gelince kural çalışır. Gruplamazsan kural yalnızca adı "Hedef" olan tek daire için çalışır.

## Oyunu sen yönet

| Değiştir | Ne olur |
| Süre 20 → 10 | Telaşlı bir oyun. |
| Hızlanma yüzde 8 → 20 | Birkaç vuruştan sonra hedefler yakalanmaz olur. |
| Hedefleri küçült | Nişan almak zorlaşır. |
| Git Gel'in süresi 2.2 → 0.8 | Hedef çok hızlı gider gelir. |

## Sıra sende

- [ ] Beşinci bir hedef ekle ve ona **Kenarlardan Sek** ver.
- [ ] Küçük bir "Altın hedef" ekle: 5 puan versin.
- [ ] Tıklanmaması gereken bir "Bomba" ekle: tıklanınca süre 5 saniye azalsın.
- [ ] Zor soru: her vuruş süreye 1 saniye eklesin. Oyun ne kadar sürebilir?

!teacher Öğretmene not
Bu oyun klavye gerektirmediği için en küçük öğrencilerle bile yapılabilir ve fare becerisini de geliştirir. Dersin kavramsal özü geri sayan saattir: "her şu kadar saniyede bir" olayı ile bir sayının birleşimi. Aynı kalıp sonraki bütün oyunlarda yeniden kullanılır. Hedefin gruplanması konusu (kutudaki soru) 3. bölümdeki gruplama dersinin pratikteki karşılığıdır; gruplamayı atlayan öğrenciler "yalnızca ortasına tıklayınca çalışıyor" diye gelir.
