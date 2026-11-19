@kisim 6. Çizim ve karakter
# Karakter çizmek
> Mavi bir kutu iş görür; ama gözleri ve şapkası olan bir kahraman oyunu sevdirir. Birkaç basit şekilden bir karakter yapmayı öğreneceksin.
:: ⏱ 35 dakika | 🎯 3. bölümü bitirenler için | 🧰 Örnekler > Oyun: platform

![Platform oyununun kahramanı seçili. ((1)) Sahnede tek nesne gibi durur. ((2)) Özellikler'de türü **Grup**, adı **Kahraman**.](25-kahraman.png "Platform oyunu sahnesi; kahraman seçili, Özellikler'de Ad kutusu işaretli")

!goal Bu bölümde öğreneceklerin
Bir karakteri basit şekillerden kurmayı, grubun içine girip parçalarını düzeltmeyi, hazır şekilleri (otomatik şekiller) kullanmayı ve karakterini oyuna yerleştirmeyi öğreneceksin.

## Bir karakter neden gruptur?

Davranışlar ve kurallar **tek bir nesneyle** çalışır. Karakterin gövdesi, gözleri ve şapkası ayrı ayrı nesneler olursa "Ok Tuşlarıyla Yürü" yalnızca birine verilir, öbürleri yerinde kalır. Parçaları gruplayınca hepsi tek nesne olur: birlikte yürür, birlikte zıplar, tek adı olur.

## Hazır bir karakteri incele

1. {{Örnekler > Oyun: platform}}'u aç.
2. Kahramana bir kez tıkla: tek nesne olarak seçilir.
3. Kahramana **çift tıkla**: grubun içine girersin.

![Grubun içindesin: artık parçalar tek tek seçilir. Alttaki durum satırı nasıl çıkacağını söyler.](25-icinde.png "Grubun içinde bir göz seçili; durum satırında Grubun içindesiniz yazısı işaretli")

4. Parçalara tek tek tıkla: gövde, göz, şapka. Her biri sıradan bir şekil.
5. Çıkmak için grubun dışında boş bir yere çift tıkla ya da [[Esc]]'e bas.

## Kendi karakterin: beş şekilde bir robot

1. **Gövde.** {{Dikdörtgen}} aracıyla dikine bir dikdörtgen çiz. Özellikler'deki {{Yuvarlaklık}} kutusuna 8 yaz: köşeler yuvarlanır. Rengini seç.
2. **Kafa.** Gövdenin üstüne daha küçük bir dikdörtgen.
3. **Gözler.** {{Elips}} aracıyla iki küçük beyaz daire; içlerine daha küçük iki siyah daire.
4. **Anten.** {{Çizgi}} aracıyla kafanın üstünden yukarı kısa bir çizgi; ucuna küçük bir daire.
5. Hepsini seç (boş yerden sürükleyerek kutu içine al) ve [[Ctrl]]+[[G]].
6. Özellikler'de {{Ad}}: **Robot**.

!why Küçük çiz, büyük düşün
Oyun karakterleri küçüktür: 30–50 piksel. Bu boyutta ince ayrıntılar görünmez; büyük, yalın şekiller ve güçlü renkler görünür. İyi bir karakter, küçültüldüğünde bile tanınır: belirgin bir şapka, büyük gözler, tek bir ana renk.

## Kolaylaştıran araçlar

| Araç | Nerede | Ne işe yarar |
| Otomatik Şekiller | Sağdaki panel | Hazır ok, yıldız, çokgen, dişli. Sahneye sürükle. |
| Yuvarlaklık | Özellikler (dikdörtgen seçiliyken) | Köşeleri yuvarlar. |
| Kontur | Özellikler | Şeklin kenar çizgisi: rengi ve kalınlığı. |
| Degrade dolgu | Özellikler'deki dolgu türü listesi | Tek renk yerine iki renk arasında geçiş: şekle hacim verir. |
| Hizala | Üstteki araç çubuğu | Seçili şekilleri ortalar, eşit aralar. |
| Kopyala, yapıştır | [[Ctrl]]+[[C]], [[Ctrl]]+[[V]] | İkinci göz, ikinci teker. |

### Sıra: önde, arkada

Sonradan çizilen şekil öne gelir. Göz gövdenin arkasında kaldıysa: gözü seç ve {{Değiştir > Yerleştir > En Öne Getir}}.

### Aynalamak

Karakterin sola baksın istiyorsan: seç ve {{Değiştir > Dönüştür > Yatay Çevir}}.

## Karakteri oyuna koymak

1. Karakteri seç, kopyala ([[Ctrl]]+[[C]]).
2. Oyun filmini aç ve yapıştır ([[Ctrl]]+[[V]]).
3. Eski kahramanı sil. Yeni karaktere **aynı adı** ("Kahraman") ve aynı davranışları ver.

Kuralları değiştirmene gerek yok: kurallar adı tanır, çizimi değil.

!stuck Takıldıysan
- **Karakterin bir parçası yerinde kalıyor.** O parça grubun dışında kalmış. Grubu çöz ([[Ctrl]]+[[Shift]]+[[G]]), hepsini yeniden seç ve grupla.
- **Grubun içinden çıkamıyorum.** [[Esc]], ya da grubun dışında boş bir yere çift tıkla.
- **Karakter platformun içine gömülüyor.** Çarpışma, karakterin çevresindeki dikdörtgen kutuya göre hesaplanır. Anten gibi dışarı taşan parçalar kutuyu büyütür; karakterini derli toplu çiz.

## Sıra sende

- [ ] Robotuna kollar ve bacaklar ekle (grubun içine girerek).
- [ ] Robotu platform oyununun kahramanı yap.
- [ ] Bir düşman çiz: tek gözlü, dişli bir yaratık. Platform oyunundaki "Düşman"ın yerine koy.
- [ ] Zor soru: aynı karakterin üç rengini yap (kırmızı, mavi, yeşil). En hızlı yolu ne?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Grubun içine girmek | Gruba çift tıklayıp parçalarını tek tek düzenlemek. |
| Otomatik şekil | Hazır, ayarlanabilir şekil: yıldız, ok, çokgen. |
| Yuvarlaklık | Dikdörtgenin köşelerinin yuvarlanma miktarı. |

!teacher Öğretmene not
Bu bölüm görsel sanatlar dersiyle birleştirilebilir. Öğrencilere önce kâğıda, yalnızca daire ve dikdörtgenlerden oluşan karakter taslakları çizdirin; sonra programa geçsinler. "Küçük çiz, büyük düşün" ilkesi tasarımın temelidir ve iyi bir sınıf tartışması konusudur: sevdikleri oyun karakterlerinin siluetlerini tahtaya çizip tanınıp tanınmadığına bakabilirsiniz. Karakterin kurallar değişmeden başka bir oyuna taşınabilmesi ("kurallar adı tanır") bu kitabın tekrar eden fikridir.
