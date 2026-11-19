@kisim 1. Başlarken
# Çizmek, boyamak, adlandırmak
> Kutulardan sıkıldın. Bu bölümde daire, çizgi ve yazı çizecek, birkaç şekli tek bir karakterde birleştirecek ve ona bir ad vereceksin.
:: ⏱ 20 dakika | 🎯 1. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Bölümün sonunda: üç şekilden yapılmış, tek parça gibi taşınan ve adı olan bir surat.](03-surat.png "Turuncu daire, iki siyah göz ve çizgi ağızdan oluşan surat" 390)

!goal Bu bölümde öğreneceklerin
Elips, çizgi ve yazı araçlarını kullanmayı, şekilleri seçip taşımayı, birkaç şekli **gruplayarak** tek bir nesne yapmayı ve nesneye **ad** vermeyi öğreneceksin. Ad, ileride kuralların o nesneyi tanıması için gerekecek.

## Adım adım

### 1. Elips aracını bul

Bazı araçların köşesinde küçük bir üçgen vardır: o aracın altında başka araçlar saklıdır.

1. {{Dikdörtgen}} aracına farenin tuşunu **basılı tut**. Küçük bir liste açılır.
2. Listeden {{Elips}}'e tıkla. Artık araç kutusunda dikdörtgenin yerinde elips görünür.

![Dikdörtgen aracına basılı tutunca açılan liste.](03-arac-listesi.png "Araç listesi: Dikdörtgen, Elips, Çokgen; Elips işaretli" 420)

### 2. Suratı çiz

1. {{Elips}} aracıyla ((1)) sahneye büyükçe bir daire çiz ((2)). Tam daire olması için sürüklerken [[Shift]] tuşunu basılı tut.
2. {{Stiller}} panelinde turuncuya tıkla ((3)).
3. Dairenin içine iki küçük daire daha çiz ve ikisini de siyaha boya. Bunlar gözler.
4. {{Çizgi}} aracını al (araçlarda, kalemin solunda) ve gözlerin altına kısa bir çizgi çek. Bu da ağız.

![Elips aracı, çizilen daire ve turuncu stil.](03-elips.png "Elips aracı seçili, sahnede turuncu daire, Stiller panelinde turuncu işaretli")

!stuck Takıldıysan
- **Yanlış yere çizdim.** [[Ctrl]]+[[Z]] son yaptığını geri alır. İstediğin kadar geri gidebilirsin.
- **Şekli taşımak istiyorum.** En üstteki siyah ok aracını (İşaretçi) al, şekle tıkla ve sürükle.
- **Şekli silmek istiyorum.** İşaretçiyle seç, [[Delete]] tuşuna bas.

### 3. Parçaları tek nesne yap: grupla

Şu an surat dört ayrı şekil. Birini sürüklersen gözler yerinde kalır. Onları birleştirelim.

1. Siyah ok aracını (İşaretçi) al.
2. Suratın sol üstündeki boş bir yerden sağ altına doğru sürükle: bir seçim kutusu çizilir ve içine giren bütün şekiller seçilir.
3. [[Ctrl]]+[[G]] tuşlarına bas (ya da {{Değiştir > Grupla}}).

!why Ne oldu?
Dört şekil artık tek bir **grup**. Tek tıkla seçilir, tek parça taşınır, davranış da kural da ona tek nesne gibi davranır. Parçaları yeniden ayırmak için {{Değiştir > Grubu Çöz}} yeter. Grubun içindeki bir parçayı düzeltmek için gruba çift tıkla; işin bitince boş bir yere çift tıkla ya da [[Esc]]'e bas.

### 4. Nesneye ad ver

1. Grup seçiliyken pencerenin altındaki {{Özellikler}} paneline bak.
2. {{Ad}} kutusuna tıkla, **Surat** yaz ve [[Enter]]'a bas.

![Özellikler panelinde Ad kutusu. Solda nesnenin türü yazar: burada "Grup".](03-ad.png "Özellikler paneli; Ad kutusunda surat yazıyor ve kutu işaretli")

!why Ad neden önemli?
Şimdilik fark etmez; ama 3. kısımda "**Surat** nesnesine tıklanınca…" gibi kurallar yazacağız. Kural, nesneyi adından tanır. Aynı adı birkaç nesneye verebilirsin: üç elmanın adı da "Elma" olursa bir kural üçü için de geçerli olur.

### 5. Yazı ekle

1. Araçlardan {{Metin}} aracını (büyük **T**) al.
2. Sahnede yazının başlamasını istediğin yere tıkla ve yaz: **Merhaba**.
3. Bitince [[Esc]]'e bas.

![Yazı da bir nesnedir: taşınır, boyanır, davranış alır.](03-yazi.png "Sahnede merhaba yazısı ve sağ üstte surat" 670)

## Seçmenin dört yolu

| Ne yapmak istiyorsun | Nasıl |
| Bir şekli seçmek | İşaretçiyle şekle tıkla. |
| Birkaç şekli seçmek | [[Shift]] basılıyken sırayla tıkla, ya da boş yerden sürükleyip kutu içine al. |
| Hepsini seçmek | [[Ctrl]]+[[A]] |
| Seçimi kaldırmak | Boş bir yere tıkla. |

## Sıra sende

- [ ] Surata bir şapka çiz (bir dikdörtgen yeter), gruba çift tıklayıp içine gir ve şapkayı yerine oturt.
- [ ] Surata **Zıpla** davranışını ver ve [[Enter]]'a bas: bütün surat tek parça zıplıyor mu?
- [ ] "Merhaba" yazısına **Yanıp Sön** ver.
- [ ] Zor soru: gruplamadan önce yalnızca gözlere **Dön** verseydin ne olurdu? Dene, sonra [[Ctrl]]+[[Z]] ile geri al.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Araç listesi | Bir araca basılı tutunca açılan, altındaki öbür araçlar. |
| İşaretçi | Seçmek ve taşımak için kullanılan siyah ok aracı. |
| Grup | Tek nesne gibi davranan birkaç şekil. [[Ctrl]]+[[G]] |
| Ad | Nesnenin Özellikler panelindeki adı; kurallar nesneyi bununla tanır. |

!teacher Öğretmene not
Bu bölüm bir ders saatini doldurur. Gruplama, sonraki bütün bölümlerin temelidir: davranışlar ve kurallar grubun kendisine verilir, içindeki parçalara değil. Öğrencilerin en sık takıldığı yer, grubun içine çift tıklayıp girdikten sonra çıkamamaktır; [[Esc]] ya da boş yere çift tıklamak çıkarır. Ad verme adımını atlamayın: 3. kısımdaki kuralların hepsi buna dayanır. Serbest çalışma için öğrencilere kendi karakterlerini çizdirin; bu karakter kitabın sonuna kadar kullanılabilir.
