@kisim 5. Kuralları yazmak
# Kuralları cümle cümle yazmak
> Şimdiye kadar kuralları listelerden seçtin. Aynı kuralları klavyeden, Türkçe cümleler olarak da yazabilirsin. Daha hızlıdır, ve listelerde olmayan şeyleri de yapar.
:: ⏱ 25 dakika | 🎯 3. kısmı bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Elma yakala oyununun bütün kuralları, yazı olarak. Kurallar penceresinde gördüğün cümlelerin aynısı.](21-elma-metni.png "Kuralları Yaz penceresi; elma yakala oyununun metni")

!goal Bu bölümde öğreneceklerin
**Kuralları Yaz** penceresini, bir kuralın yazılışını (olay satırı, içeriden eylemler), Tab ile tamamlamayı ve pencerenin hataları nasıl gösterdiğini öğreneceksin.

## Aynı kurallar, başka bir görünüm

Listelerle kurduğun kurallar ile yazdığın kurallar **aynı kurallardır**. Birinde yaptığın değişiklik öbüründe görünür. İstediğin zaman birinden öbürüne geçebilirsin; hangisi kolayına geliyorsa onu kullan.

1. {{Örnekler > Oyun: elma yakala}}'yı aç.
2. {{Kurallar > Kuralları Yaz…}}'a tıkla.

Yukarıdaki pencere açılır. Oku: daha önce Kurallar penceresinde gördüğün beş cümle, alt alta.

## Bir kural nasıl yazılır?

```
Elma nesnesi Sepet nesnesine değince:
    Puan sayısını 1 artır
    çın sesini çal
```

Üç kural var, hepsi bu:

| Kural | Açıklama |
| Olay satırı **kenardan başlar** ve **iki noktayla biter**. | `Elma nesnesi Sepet nesnesine değince:` |
| Eylemler **içeriden** (dört boşluk) başlar. | `    Puan sayısını 1 artır` |
| Her eylem **kendi satırında** durur. | Alt alta yazılır, sırayla yapılır. |

Kurallar arasına boş satır koymak okumayı kolaylaştırır; zorunlu değildir.

## Adım adım: ilk yazdığın kural

### 1. Hazırlık

1. Yeni bir filmde bir kutu çiz ve adını **Kutu** koy.
2. {{Kurallar > Kuralları Yaz…}}'ı aç.

### 2. Yaz, ama hepsini değil

1. **Kutu** yaz. Hemen altında öneriler belirir: PisiAnimasyon, bir nesne adıyla başlayabilecek cümleleri gösterir.

![Yazdıkça öneriler gelir. Mavi satır seçili olandır.](21-oneriler.png "Kutu yazılmış; altında üç cümle önerisi")

2. [[Tab]]'a bas. Seçili öneri satıra yazılır: `Kutu nesnesine tıklanınca:`
3. [[Enter]]'a bas. İmleç kendiliğinden içeriden başlar: iki noktadan sonra eylem beklenir.
4. **Kutu nesnesini 100 yükseğe zıplat** yaz. (Ya da "Kutu" yazıp okla önerilerden seç.)

### 3. Pencere seni uyarır

![Sarı satır bir öğüttür: cümle doğru, ama umduğunu yapmayacak. Yanındaki düğme sorunu tek tıkla giderir.](21-uyari.png "Sarı uyarı: Kutu henüz zıplayamaz, Düş ve Zıpla davranışı gerekir; yanında Düş ve Zıpla Ver düğmesi")

Cümlen doğru; ama kutunun **Düş ve Zıpla** davranışı yok, yani zıplayamaz. Pencere bunu sarı bir satırla söyler ve yanına çözümü koyar.

5. {{Düş ve Zıpla Ver}} düğmesine tıkla. Davranış kutuya eklenir, uyarı kaybolur.

### 4. Yanlış yazarsan

1. Yeni bir satıra **Kutu nesnesini uçur** yaz.

![Kırmızı satır bir hatadır: PisiAnimasyon bu cümleyi tanımıyor. Altta hangi satır olduğu ve neden olduğu yazar.](21-hata.png "Üçüncü satır kırmızı; altta Satır 3: bu, bildiğim bir eylem değil")

"Uçur" diye bir eylem yok. Satır kırmızıya döner ve altta nedeni yazar. Satırı sil ([[Ctrl]]+[[Z]] de olur).

### 5. Uygula

1. {{Uygula ve Test Et}}'e tıkla (ya da [[Ctrl]]+[[Enter]]).

!why Sarı ile kırmızının farkı
**Kırmızı**: cümle okunamıyor. Düzeltmeden uygulayamazsın. **Sarı**: cümle okunuyor ama bir şey eksik (davranış yok, o adda nesne yok). Uygulayabilirsin; ama büyük olasılıkla istediğin olmaz.

## Penceredeki kolaylıklar

| Tuş | Ne yapar |
| [[Tab]] | Seçili öneriyi alır; öneri yoksa satırdaki sonraki `?` yerine gider. |
| [[↑]] [[↓]] | Öneriler arasında gezer. |
| [[Ctrl]]+[[Z]], [[Ctrl]]+[[Y]] | Geri al, yinele. |
| [[Ctrl]]+[[S]] | Uygula. |
| [[Ctrl]]+[[Enter]] | Uygula ve test et. |
| [[Esc]] | Pencereyi kapat. |

Üstteki {{Bir olay}} ve {{Bir eylem}} kutuları, cümleleri yazmadan eklemek içindir: listeden seçtiğin cümle satıra yazılır, ad ve sayı gereken yerlerde `?` durur. `?` yerine yazarsın; [[Tab]] seni bir sonrakine götürür.

!note Ad yazarken
Bir nesnenin, sayının ya da sesin adının ilk harflerini yaz ve [[Tab]]'a bas: ad tamamlanır. Bir cümlenin ortasında da çalışır. "Çark nesnesine " yazıp boşluk bırakınca davranışların adları da önerilir.

## En çok kullanılan cümleler

| Olaylar | Eylemler |
| `Film başlayınca:` | `… nesnesini gizle` · `… nesnesini göster` |
| `… tuşuna basılınca:` | `… nesnesini … sağa, … aşağı taşı` |
| `… nesnesine tıklanınca:` | `… nesnesini … yükseğe zıplat` |
| `… nesnesi … nesnesine değince:` | `… nesnesini … derece döndür` |
| `… nesnesi sahnenin … değince:` | `… sayısını … artır` · `… sayısını … azalt` |
| `Her … saniyede bir:` | `… nesnesinin kopyasını … nesnesinin yerinde oluştur` |
| `… sayısı … olunca:` | `… sesini çal` · `sonraki sahneye git` · `baştan başla` |

Bütün cümlelerin listesi kitabın sonundaki eklerde.

!stuck Takıldıysan
- **Bütün satırlar kırmızı.** Olay satırının sonundaki iki noktayı unutmuş olabilirsin; o zaman altındaki eylemler de anlamsız kalır.
- **"Kenardan başlayan satır bir olaydır" diyor.** Bir eylemi kenardan başlatmışsın. Başına dört boşluk koy.
- **Nesnemin adında boşluk var.** Tırnak içine yaz: `"Kırmızı Top" nesnesini gizle`.
- **Türkçe harfler.** Cümleler Türkçe harflerle yazılır: "tıklanınca", "değince", "yükseğe". "tiklaninca" tanınmaz.

## Sıra sende

- [ ] Elma yakala oyununun metninde `Puan sayısını 1 artır` satırını `Puan sayısını 5 artır` yap ve uygula.
- [ ] Aynı oyuna yazarak bir kural ekle: Boşluk tuşuna basılınca sepet zıplasın.
- [ ] Kendi filminde bir kuralı önce listelerden kur, sonra Kuralları Yaz'ı açıp nasıl yazıldığına bak.
- [ ] Zor soru: 10. bölümdeki "Düğme ve Lamba" kurallarını (düğmeye tıklanınca lamba görünsün, yeniden tıklanınca gizlensin) yalnızca yazarak kur. İki kural yeter mi?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Kuralları Yaz | Kuralları klavyeden cümle olarak yazdığın pencere. |
| Öneri | Yazdıkça beliren, [[Tab]] ile alınan cümle ya da ad. |
| Kırmızı satır | Okunamayan cümle: hata. |
| Sarı satır | Okunan ama eksiği olan cümle: öğüt. |

!teacher Öğretmene not
Bu bölüm kod yazmaya geçiştir; ama dil Türkçe ve cümleler öğrencinin zaten listelerden tanıdığı cümlelerdir, bu yüzden eşik düşüktür. Önce hazır bir oyunun metnini okutmak, sonra küçük bir değişiklik yaptırmak, en son sıfırdan yazdırmak iyi bir sıradır. Klavye becerisi zayıf öğrenciler için Tab ile tamamlama ve "Bir olay / Bir eylem" kutuları yazma yükünü büyük ölçüde azaltır; bunları özellikle gösterin. Girinti (dört boşluk) kavramı Python gibi dillerle aynıdır ve ileride o dillere geçişi kolaylaştırır. Listeler ile yazı arasında gidip gelmenin serbest olduğunu vurgulayın: yazmak zorunlu değildir, bir seçenektir.
