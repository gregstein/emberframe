@kisim 2. Hareket
# Hareket ara doldurma
> Geçen bölümde top bir yerden öbürüne atlıyordu. Şimdi aradaki kareleri PisiAnimasyon'a çizdireceksin: top kayarak gidecek.
:: ⏱ 20 dakika | 🎯 6. bölümü bitirenler için | 🧰 6. bölümde yaptığın film

!goal Bu bölümde öğreneceklerin
İki anahtar kare arasına **hareket ara doldurması** koymayı, hareketi yavaş başlatıp yavaş bitirmeyi (yavaşlama) ve nesneyi yol boyunca döndürmeyi öğreneceksin.

## Ara doldurma nedir?

Sen iki anahtar kare verirsin: başlangıç ve bitiş. PisiAnimasyon aradaki bütün kareleri kendisi hesaplar. Topun 1. karede solda, 12. karede yukarıda olduğunu biliyorsa 6. karede yolun ortasında olması gerektiğini de bilir. Buna **ara doldurma** denir. Yalnızca yer değil; boyut, dönüş ve saydamlık da böyle doldurulur.

## Adım adım

### 1. İlk iki anahtar karenin arasını doldur

1. 6. bölümdeki filmi aç. (Yoksa: bir top çiz, 12. ve 24. karelerde [[F6]]'ya basıp topu başka yerlere taşı.)
2. Zaman çizelgesinde **1.** kareye tıkla.
3. {{Zaman Çizelgesi}} menüsünü aç ve {{Hareket Ara Doldurması Oluştur}}'a tıkla.

![Zaman Çizelgesi menüsü.](07-menu.png "Zaman Çizelgesi menüsü; Hareket Ara Doldurması Oluştur işaretli" 470)

1'den 12'ye kadar olan kareler maviye döner: aralarında ara doldurma var demektir.

### 2. İkinci aralığı da doldur

1. **12.** kareye tıkla.
2. Yine {{Zaman Çizelgesi > Hareket Ara Doldurması Oluştur}}.

### 3. Bak ve oynat

1. Zaman çizelgesinde 6. kareye tıkla: top yolun ortasında ((2)). Onu oraya sen koymadın.
2. [[Enter]]'a bas. Top artık kayarak gidiyor.

![Aynı üç anahtar kare, bu kez aralarında ara doldurma var: top kayarak gider.](07-film.png "Oynayan film: kırmızı top yolun üstünde" 660){07-ara-doldurma.html 660x500}

![((1)) Mavi kareler ara doldurmayı gösterir. ((2)) 6. karede top yolun ortasında. ((3)) Ara doldurmanın ayarları burada belirir.](07-ara-doldurma.png "Zaman çizelgesinde mavi ara doldurma; top iki anahtar karenin ortasında; Yavaşlama ve Tur kutuları")

!why Ne oldu?
Ara doldurma, bulunduğun anahtar kareden bir sonraki anahtar kareye kadar olan aralığa konur. Bu yüzden önce 1. kareye, sonra 12. kareye tıklayıp iki kez oluşturdun: iki aralık, iki ara doldurma.

## Hareketi güzelleştirmek

Ara doldurmalı bir kareye tıkladığında zaman çizelgesinin altında üç ayar belirir ((3)).

| Ayar | Ne yapar | Dene |
| Yavaşlama | Hareketin hızını değiştirir. **100**: hızlı başlar, yavaşlayarak durur. **−100**: yavaş başlar, hızlanır. **0**: hep aynı hız. | Yukarı çıkan topa 100, aşağı inen topa −100 yaz: gerçek bir top gibi olur. |
| Tur | Nesne yol boyunca kaç tam tur dönsün. | Yuvarlanan bir tekerlek için 2 yaz. |
| Yola yönlendir | Kılavuz yol kullanırken nesne gittiği yöne baksın. | 9. bölümde kullanacağız. |

!why Yavaşlama neden önemli?
Gerçek dünyada hiçbir şey birden son hızına çıkmaz, birden durmaz. Yukarı atılan top yavaşlar, tepede bir an durur, sonra hızlanarak düşer. Yavaşlama olmayan hareket robot gibi görünür; biraz yavaşlama onu canlı gösterir. Animasyonun en eski sırlarından biridir.

## Yalnızca yer değil

Ara doldurma, iki anahtar kare arasında değişen her şeyi doldurur. Bitiş karesinde nesneyi:

| Değiştirirsen | Arada ne olur |
| Taşırsan | Kayar |
| {{Ölçek}} aracıyla büyütürsen | Giderek büyür |
| Döndürürsen | Giderek döner |
| Özellikler'de {{Opaklık}}'ı 0 yaparsan | Giderek kaybolur |

## Bir ara doldurmanın ortasını düzeltmek

Topun yolun ortasında biraz daha yukarıdan geçmesini mi istiyorsun? O kareye git ve topu sürükle. PisiAnimasyon o kareyi kendiliğinden anahtar kareye çevirir ve iki yanını yeniden doldurur. Flash'ta da böyledir.

!stuck Takıldıysan
- **Menüde "Hareket Ara Doldurması Oluştur" soluk.** Bulunduğun kareden sonra başka anahtar kare yok. Ara doldurma iki anahtar kare ister.
- **Top hâlâ atlıyor.** Ara doldurmayı yanlış kareye koymuş olabilirsin. Mavi olmayan aralığın **ilk** anahtar karesine tıkla ve yeniden oluştur.
- **Vazgeçtim.** Aralıktaki bir kareye tıkla, {{Zaman Çizelgesi > Ara Doldurmayı Kaldır}}.

## Sıra sende

- [ ] Yukarı çıkan aralığa 100, inen aralığa −100 yavaşlama ver. Fark görünüyor mu?
- [ ] 24. karede topu {{Ölçek}} aracıyla iki katına büyüt. Top yaklaşıyormuş gibi görünüyor mu?
- [ ] 24. karede topun opaklığını 0 yap: top uzaklaşırken kaybolsun.
- [ ] Zor soru: bir yazının sahneye soldan kayarak girip ortada durmasını, bir saniye bekleyip sağdan çıkmasını sağla. Kaç anahtar kare gerekir?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Hareket ara doldurması | İki anahtar kare arasındaki kareleri programın hesaplaması. |
| Yavaşlama | Hareketin yavaş başlaması ya da yavaş bitmesi. |
| Tur | Nesnenin yol boyunca attığı tam dönüş sayısı. |

!teacher Öğretmene not
Bu, Flash müfredatının merkezindeki konudur ve PisiAnimasyon'da aynı adlarla, aynı mantıkla çalışır. Öğrencilerin kavraması gereken tek şey şudur: ara doldurma bir **aralığa** konur ve o aralığın **ilk** anahtar karesinden oluşturulur. Yavaşlama için sınıfta gerçek bir topu havaya atıp tepe noktasındaki duraksamayı göstermek çok işe yarar. "Zor soru" dört anahtar kare ister (dışarıda, ortada, ortada, dışarıda) ve "beklemek de iki anahtar kare ister" fikrini öğretir.
