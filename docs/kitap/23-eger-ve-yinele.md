@kisim 5. Kuralları yazmak
# Eğer ve yinele
> Bir kuralın içinde karar vermek ("eğer puan 10'u geçtiyse şunu, değilse bunu yap") ve bir şeyi birkaç kez yapmak ("üç kez yinele"). Programlamanın iki temel taşı.
:: ⏱ 30 dakika | 🎯 22. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Bir kuralın içinde hem karar hem tekrar.](23-eger.png "Kuralları Yaz penceresi: eğer, değilse ve 3 kez yinele blokları olan bir kural")

!goal Bu bölümde öğreneceklerin
`eğer … :` ve `değilse:` ile karar vermeyi, `… kez yinele:` ile tekrar etmeyi, `… saniye bekle` ile araya süre koymayı ve iç içe girintiyi öğreneceksin.

## Blok: içeriden yazılan satırlar

İki noktayla biten her satır bir **blok** açar; altında, bir kademe daha içeriden yazılan satırlar o bloğa aittir. Olay satırı da böyle bir bloktu. Şimdi kuralın *içinde* bloklar açacağız.

```
Kutu nesnesine tıklanınca:
    eğer Puan sayısı 10 değerinden büyükse:
        Kutu nesnesini gizle
    değilse:
        Puan sayısını 1 artır
```

Girintiye dikkat: `eğer` dört boşluk içeride (kuralın eylemi), `Kutu nesnesini gizle` sekiz boşluk içeride (eğer'in içi). [[Enter]]'a bastığında pencere girintiyi kendisi ayarlar; bir kademe geri çıkmak için [[Backspace]].

## Eğer

| Yazdığın | Ne zaman içindekiler yapılır |
| `eğer Puan sayısı 10 değerinden büyükse:` | Puan 11 ve üstüyse |
| `eğer Can sayısı 1 değerinden küçükse:` | Can 0 ya da altındaysa |
| `eğer Anahtar sayısı 1 ise:` | Anahtar tam 1 ise |

`değilse:` isteğe bağlıdır: koşul tutmazsa yapılacakları yazar. `eğer` ile aynı girintide durur.

!why Koşullu kuraldan farkı ne?
11. bölümde kuralın tamamına koşul koymuştun: koşul tutmazsa kural hiç çalışmaz. `eğer` ise kuralın **bir kısmını** koşula bağlar: önceki ve sonraki eylemler her durumda yapılır. Ayrıca `değilse` ile öbür durumu da söyleyebilirsin.

## Yinele

```
Kutu nesnesine tıklanınca:
    3 kez yinele:
        Kutu nesnesini 30 derece döndür
        0.2 saniye bekle
```

Kutu üç kez, her seferinde 30 derece döner; aralarında 0.2 saniye bekler. Toplam 90 derece, ama **adım adım**: gözle görülür bir hareket.

`… saniye bekle` olmadan üç dönüş aynı anda olur ve sen yalnızca sonucu görürsün. Beklemek, tekrarı animasyona çevirir.

Sayı yerine bir sayının adı da olur: `Zar kez yinele:`

## Koşullu yineleme

```
Puan sayısı 0 değerinden büyükse yinele:
    Puan sayısını 1 azalt
    0.1 saniye bekle
```

Koşul tuttuğu sürece içindekiler yeniden yapılır. Burada puan, sıfıra inene kadar birer birer azalır: geri sayan bir sayaç.

!stuck Sonsuz döngü
Koşul hiç değişmezse yineleme hiç bitmez. `Puan sayısı 0 değerinden büyükse yinele:` bloğunun içinde puanı azaltan bir satır olmalı. PisiAnimasyon filmin donmasını önlemek için çok uzun süren yinelemeleri kendiliğinden keser; ama doğrusu, koşulu bir gün yanlış yapacak bir satırın içeride bulunmasıdır.

## İç içe

Blokların içine blok yazabilirsin. Bölümün başındaki resimdeki kural:

```
Kutu nesnesine tıklanınca:
    Puan sayısını Puan * 2 + 1 yap
    eğer Puan sayısı 10 değerinden büyükse:
        Kutu nesnesini gizle
    değilse:
        3 kez yinele:
            Kutu nesnesini 30 derece döndür
            0.2 saniye bekle
```

Okuyalım: kutuya tıklanınca puan ikiye katlanıp bir eklenir. Puan 10'u geçtiyse kutu gizlenir. Geçmediyse kutu üç adımda döner.

Tıklamaları say: puan 0 → 1 → 3 → 7 → 15. Dördüncü tıklamada kutu kaybolur.

## Adım adım

1. Yeni bir filmde **Kutu** adında bir kutu çiz; sahneye `Puan: {Puan}` yaz.
2. {{Kuralları Yaz…}}'ı aç ve yukarıdaki metni, başında `Sayılar:` bölümüyle birlikte yaz.
3. {{Uygula ve Test Et}}. Kutuya dört kez tıkla ve puanı izle.

![Kutuya tıkla: ilk üç tıklamada adım adım döner, dördüncüde kaybolur.](23-film.png "Oynayan film: mavi kutu dönmüş" 660){23-eger.html 660x500}

!stuck Takıldıysan
- **"değilse" kırmızı.** `değilse:` kendi `eğer`'i ile aynı girintide olmalı ve hemen onun bloğundan sonra gelmeli.
- **Yinelenen şeyler bir anda oluyor.** Araya `… saniye bekle` koy.
- **Girinti karıştı.** Her kademe dört boşluktur. Bir satır hangi bloğa aitse onun bir kademe içinde durur.

## Sıra sende

- [ ] Kutu 3 kez değil 12 kez, her seferinde 30 derece dönsün: tam bir tur.
- [ ] Tıklanınca kutu beş kez yanıp sönsün (her seferinde gizle, bekle, göster, bekle).
- [ ] Bir zar at; 6 gelirse "Kazandın" yazısını göster, değilse kutuyu döndür.
- [ ] Zor soru: boşluk tuşuna basılınca 3'ten geriye sayan (3, 2, 1) ve sonunda "Başla!" yazısını gösteren bir sayaç yap.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Blok | İki noktadan sonra, içeriden yazılan satırlar. |
| `eğer …:` / `değilse:` | Koşul tutarsa birini, tutmazsa öbürünü yap. |
| `… kez yinele:` | İçindekileri o kadar kez yap. |
| `… saniye bekle` | Devam etmeden önce bekle. |

!teacher Öğretmene not
Karar ve tekrar, bütün programlama dillerinin ortak temelidir; burada Türkçe ve girintiyle öğretilir. Girinti Python'un bloklarıyla aynı mantıktadır. "Tıklamaları say" alıştırmasını (0 → 1 → 3 → 7 → 15) öğrencilere önce kâğıt üzerinde hesaplatıp sonra denettirmek, kodu "kafada çalıştırma" becerisi kazandırır. `bekle` olmadan yinelemenin gözle görülmediği gerçeği, bilgisayarın hızını sezdirmek için iyi bir fırsattır. Sonsuz döngü kutusunu atlamayın: öğrenciler mutlaka bir tane yazacaktır.
