@kisim 3. Oyun kuralları
# İlk kural: tıklanınca
> Davranış "hep şunu yap" der. Kural ise "şu olunca şunu yap" der. Oyunu oyun yapan budur. İlk kuralın: kutuya tıklanınca kutu dönsün.
:: ⏱ 15 dakika | 🎯 3. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Kutuya her tıkladığında kutu 45 derece döner.](10-film.png "Tarayıcıda 45 derece dönmüş mavi kutu" 400){10-ilk-kural.html 660x500}

!goal Bu bölümde öğreneceklerin
Bir kuralın iki parçasını (**olay** ve **eylem**), Kurallar penceresini kullanmayı ve kuralın cümle olarak nasıl okunduğunu öğreneceksin. Hiçbir şey yazmayacaksın: her şey listelerden seçilir.

## Kural nedir?

Her kural tek bir cümledir ve iki parçası vardır:

| Parça | Ne söyler | Örnek |
| Olay | **Ne olunca?** | Kutu nesnesine tıklanınca |
| Eylem | **Ne yapılsın?** | Kutu nesnesini 45 derece döndür |

İkisi iki nokta ile birleşir: *Kutu nesnesine tıklanınca: Kutu nesnesini 45 derece döndür.* Bütün oyunlar böyle cümlelerden oluşur.

## Hazırlık

1. Yeni bir filmde mavi bir kutu çiz.
2. Özellikler panelindeki {{Ad}} kutusuna **Kutu** yaz ve [[Enter]]'a bas.

!note Ad vermeyi unutma
Kural, nesneyi adından tanır. Adı olmayan bir nesne kural listelerinde görünmez. (Ad vermeyi 3. bölümde görmüştük.)

## Adım adım

### 1. Kurallar penceresini aç

1. Sağ üstteki {{Kurallar}} menüsünü aç ve {{Kurallar ve Sayılar…}}'a tıkla.

![Kurallar menüsü.](10-menu.png "Kurallar menüsü; Kurallar ve Sayılar işaretli" 360)

### 2. Olayı seç

1. {{Yeni Kural}} kutusuna tıkla. Olayların listesi açılır.
2. {{Bir nesneye tıklanınca}}'yı seç.

![Olaylar: bir kuralı başlatabilecek on şey.](10-olaylar.png "Yeni Kural listesi açık; Bir nesneye tıklanınca işaretli")

Pencere değişir: üstte kuralın cümlesi belirir, altında kuralın parçaları.

![((1)) Olay. ((2)) Olayın hangi nesneyle ilgili olduğu. ((3)) Eylem eklemek için liste.](10-kural.png "Kural satırları: Olay, Koşul, Eylem")

### 3. Eylemi seç

1. {{Eylem Ekle}} kutusuna tıkla ((3)). Eylemlerin listesi açılır.
2. {{Nesneyi döndür}}'ü seç.

![Eylemler listesi uzundur; kaydırarak hepsini görebilirsin.](10-eylemler.png "Eylem listesi; Nesneyi döndür işaretli" 380)

### 4. Sayıyı ayarla ve dene

1. {{Derece}} kutusuna tıkla ((2)), **45** yaz ve [[Enter]]'a bas.
2. Üstteki cümleyi oku ((1)): *Kutu nesnesine tıklanınca: Kutu nesnesini 45 derece döndür.*
3. {{Filmi Test Et}}'e tıkla ((3)) ve tarayıcıda kutuya tıkla. Her tıklamada biraz daha döner.

![Kural tamam. Üstteki mavi satır kuralın cümlesidir.](10-bitti.png "Kurallar penceresi; kural cümlesi, Derece kutusu ve Filmi Test Et işaretli")

!why Ne oldu?
PisiAnimasyon'a bir kural verdin. Film oynarken sürekli bekler: kutuya tıklandı mı? Tıklanınca eylemi bir kez yapar ve beklemeye devam eder. Sen hiçbir kod yazmadın; iki listeden iki şey seçtin ve bir sayı girdin.

## Bir kurala birkaç eylem

Bir kuralın birden çok eylemi olabilir; sırayla yapılır. {{Eylem Ekle}} kutusu her eylemden sonra yeniden belirir.

1. {{Eylem Ekle}}'den {{Nesneyi bir an yanıp söndür}}'ü seç.

Cümle uzar: *Kutu nesnesine tıklanınca: Kutu nesnesini 45 derece döndür, Kutu nesnesini bir an yanıp söndür.* Bir eylemi silmek için yanındaki {{Kaldır}} düğmesine tıkla.

## Sık kullanılan olaylar

| Olay | Ne zaman olur | Bölüm |
| Film başlayınca | Film açılır açılmaz, bir kez | 13 |
| Bir tuşa basılınca | Seçtiğin tuşa basılınca | 13 |
| Bir nesneye tıklanınca | Nesneye fareyle tıklanınca | bu bölüm |
| Bir nesne ötekine değince | İki nesne birbirine dokununca | 12 |
| Bir nesne sahnenin kenarına değince | Nesne kenara ulaşınca | 12 |
| Her şu kadar saniyede bir | Durmadan, düzenli aralıkla | 11 |
| Bir sayı bir değere gelince | Puan 10 olunca, can bitince | 11 |

!stuck Takıldıysan
- **Nesne listesi boş.** Nesnene ad vermemişsin. Pencereyi kapat, nesneyi seç, Özellikler'de Ad kutusunu doldur.
- **Tıklıyorum, bir şey olmuyor.** Programın içinde değil, tarayıcıda tıklamalısın. Kurallar yalnızca **Filmi Test Et** ile çalışır.
- **Yanlış kural yaptım.** Kuralın satırı seçiliyken {{Kuralı Sil}}'e tıkla.

## Sıra sende

- [ ] Dereceyi 45 yerine 90 yap. Dört tıklamada kutu bir tam tur atıyor mu?
- [ ] İkinci bir kural ekle: {{Bir tuşa basılınca}} → Boşluk → {{Nesneyi gizle}} → Kutu. Sonra üçüncü bir kural: Enter tuşuna basılınca kutuyu göster.
- [ ] Kutuya tıklanınca kutu rastgele bir yere gitsin ({{Nesneyi rastgele bir yere koy}}). Yakalaması zor mu?
- [ ] Zor soru: iki nesne çiz, "Düğme" ve "Lamba". Düğmeye tıklanınca lamba görünsün. Kural hangi nesnenin olayı, hangi nesnenin eylemi?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Kural | "Şu olunca şunu yap" diyen cümle. |
| Olay | Kuralı başlatan şey: tıklama, tuş, değme, süre. |
| Eylem | Olay olunca yapılan şey: döndür, gizle, taşı, puan ver. |
| Kurallar ve Sayılar | Kuralları listelerden seçerek kurduğun pencere. |

!teacher Öğretmene not
Bu bölüm, programlamanın temelindeki "olay olunca eylem" düşüncesini kod yazdırmadan öğretir. Öğrencilerden kuralı her seferinde yüksek sesle, cümle olarak okumalarını isteyin; pencerenin üstündeki mavi satır bunun için vardır. Kural cümleleri 5. kısımda klavyeden yazılacak olan cümlelerle harfi harfine aynıdır; yani burada öğrenilen dil ileride de geçerlidir. Son "zor soru" olayın bir nesneyle, eylemin başka bir nesneyle ilgili olabileceğini kavratır; birçok öğrenci ilk başta ikisinin aynı nesne olması gerektiğini sanır.
