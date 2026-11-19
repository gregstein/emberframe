@kisim 5. Kuralları yazmak
# Sayılarla hesap
> Listelerde bir sayıyı 1 artırabiliyordun. Yazarak, sayının yerine bir işlem koyabilirsin: puanı ikiye katla, zar at, nesneyi öbürünün yanına koy.
:: ⏱ 25 dakika | 🎯 21. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Üç eylem, üçünde de sayı yerine bir işlem var.](22-hesap.png "Kuralları Yaz penceresi: rastgele(1, 6), Puan + Zar ve Kutu.x + 20 içeren üç eylem")

!goal Bu bölümde öğreneceklerin
Bir cümlede sayı yazılan her yere **işlem** yazabileceğini, dört işlemi, rastgele sayıyı ve bir nesnenin yerini (`Ad.x`, `Ad.y`) kullanmayı öğreneceksin.

## Sayı yerine işlem

Bir cümlede sayının geldiği her yere bir işlem yazabilirsin:

| Yazdığın | Anlamı |
| `Puan sayısını 5 artır` | Puanı 5 artır. |
| `Puan sayısını Zar artır` | Puanı, **Zar** sayısı kaçsa o kadar artır. |
| `Puan sayısını Puan * 2 yap` | Puanı ikiye katla. |
| `Puan sayısını (Puan + 1) * 2 yap` | Önce parantezin içi. |

İşaretler: `+` toplama, `-` çıkarma, `*` çarpma, `/` bölme. Çarpma ve bölme önce yapılır; sırayı değiştirmek için parantez kullan.

## Sayıları tanımlamak

Yazarken sayıları da metnin başında tanımlarsın:

```
Sayılar:
    Puan = 0
    Zar = 1
```

Bu, Kurallar penceresindeki **Yeni Sayı** ile aynı şeydir: ad ve başlangıç.

## Zar atmak: rastgele

`rastgele(1, 6)`, her seferinde 1 ile 6 arasında (ikisi de dahil) bir tam sayı verir.

```
Kutu nesnesine tıklanınca:
    Zar sayısını rastgele(1, 6) yap
    Puan sayısını Puan + Zar yap
```

Her tıklamada bir zar atılır ve puana eklenir. Zarı görmek için sahneye `Zar: {Zar}` yazan bir yazı koy.

## Nesnenin yeri: Ad.x ve Ad.y

Her nesnenin sahnede bir yeri vardır: **x** soldan uzaklığı, **y** yukarıdan uzaklığı (piksel). Özellikler panelindeki X ve Y kutuları bunlardır. Bir işlemde `Kutu.x` yazarsan kutunun o anki x'ini alırsın.

```
Kutu nesnesine tıklanınca:
    Kutu nesnesini Kutu.x + 20, Kutu.y konumuna koy
```

Kutu her tıklamada 20 piksel sağa kayar: yeni x'i, eski x'inin 20 fazlası.

| Yazdığın | Ne olur |
| `Kedi nesnesini Fare.x, Fare.y konumuna koy` | Kedi, farenin olduğu yere ışınlanır. |
| `Top nesnesini 330, Top.y konumuna koy` | Top yatayda ortaya gelir, yüksekliği değişmez. |
| `Kalkan nesnesini Gemi.x, Gemi.y - 40 konumuna koy` | Kalkan geminin 40 piksel üstünde durur. |

!why y neden aşağı doğru büyüyor?
Bilgisayar ekranında sayma sol üst köşeden başlar: x sağa, y **aşağı** doğru artar. Matematik dersindeki grafiklerin tersidir. Bir nesneyi yukarı çıkarmak için y'den **çıkarırsın**.

## Koşullarda işlem

Koşulda karşılaştırılan **değer** de bir işlem olabilir:

```
Top nesnesi Kale nesnesine değince, Puan sayısı Rekor değerinden büyükse:
    Rekor sayısını Puan yap
```

Koşulun başındaki ad bir sayının adı olmalıdır; işlem, "değerinden" sözcüğünden önceki yere yazılır: `Puan sayısı Rekor + 5 değerinden büyükse`.

## Adım adım: şans oyunu

1. Yeni bir filmde bir kutu çiz, adını **Kutu** koy.
2. Sahneye iki yazı ekle: `Zar: {Zar}` ve `Puan: {Puan}`.
3. {{Kurallar > Kuralları Yaz…}}'ı aç ve bölümün başındaki resimdeki metni yaz.
4. {{Uygula ve Test Et}}. Kutuya tıkla: zar atılıyor, puan artıyor, kutu sağa kayıyor.

![Kutuya her tıklayışta zar atılır, puan artar ve kutu 20 piksel sağa kayar.](22-film.png "Oynayan film: mavi kutu" 660){22-hesap.html 660x500}

!stuck Takıldıysan
- **"… adında bir sayı yok" diyor.** İşlemde kullandığın her adı önce `Sayılar:` altında tanımlamalısın.
- **Ondalık sayı.** Nokta ile de virgül ile de yazabilirsin: `0.5` ve `0,5` aynıdır.
- **"Ad. dan sonra x ya da y gelir" diyor.** Yalnızca `.x` ve `.y` vardır; genişlik ya da yükseklik alınamaz.
- **Nesnemin adında boşluk var.** Tırnak içine al: `"Kırmızı Top".x`.

## Sıra sende

- [ ] Kutu her tıklamada sağa değil, rastgele bir miktar (5 ile 50 arası) kaysın.
- [ ] İki zar at: **Zar1** ve **Zar2**; puana ikisinin toplamı eklensin.
- [ ] Bir "Gölge" nesnesi her zaman kutunun 10 piksel sağında ve 10 piksel altında dursun. (İpucu: `Her 0.05 saniyede bir:`)
- [ ] Zor soru: puan her 10'un katına geldiğinde (10, 20, 30…) kutu yanıp sönsün. Bölme işaretiyle bunu yapabilir misin?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| İşlem | Sayı yerine yazılan hesap: `Puan * 2 + 1` |
| `rastgele(a, b)` | a ile b arasında şansa bağlı bir tam sayı. |
| `Ad.x`, `Ad.y` | Nesnenin o anki yatay ve dikey yeri. |

!teacher Öğretmene not
Bu bölüm matematik dersiyle doğrudan bağlantılıdır: işlem önceliği, parantez, koordinat sistemi. Ekran koordinatlarında y'nin aşağı doğru artması öğrencileri şaşırtır; tahtaya ekranı çizip birkaç noktayı birlikte yerleştirmek iyi olur. `rastgele` ile basit olasılık deneyleri yapılabilir: zarı 100 kez atıp kaç kez 6 geldiğini saydırmak hem bu bölümün hem bir sonraki bölümün (yinele) pratiğidir. "Gölge" alıştırması sürekli güncellenen konum fikrini verir.
