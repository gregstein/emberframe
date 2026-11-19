@kisim 3. Oyun kuralları
# Puan ve can: sayılar
> Bir oyunun aklında tutması gereken şeyler vardır: kaç puan aldın, kaç canın kaldı, kaç saniye geçti. Bunların hepsi birer sayıdır.
:: ⏱ 20 dakika | 🎯 10. bölümü bitirenler için | 🧰 10. bölümdeki film

![Elma yakala oyununda iki sayı var: Puan ve Can. Bu bölümde ikisini de kurmayı öğreneceksin.](11-film.png "Elma yakala oyunu: solda Puan: 0, sağda Can: 5" 640){15-elma.html 640x400}

!goal Bu bölümde öğreneceklerin
Bir **sayı** tanımlamayı, bir kuralla onu artırıp azaltmayı, ekranda göstermeyi ve sayı bir değere gelince bir şey yaptırmayı öğreneceksin.

## Adım adım

### 1. Bir sayı ekle

1. 10. bölümdeki filmi aç (ya da "Kutu" adında bir kutu çiz).
2. {{Kurallar > Kurallar ve Sayılar…}}'ı aç.
3. Altta, **Sayılar** başlığının altındaki {{Yeni Sayı}} düğmesine tıkla.

![Yeni bir sayı: adı **Puan**, başlangıcı **0**.](11-sayi.png "Kurallar penceresinde Sayılar bölümü; Puan adlı sayının satırı işaretli")

Sayının iki özelliği var: **adı** ve **başlangıcı**, yani film başladığında kaç olduğu. Adını değiştirmek için kutuya tıklayıp yaz. Puan için başlangıç 0'dır; can için 3 ya da 5 olur.

### 2. Tıklanınca puan ver

1. Kuralın {{Eylem Ekle}} kutusuna tıkla ve en üstteki {{Sayıyı değiştir}}'i seç.
2. Yanındaki kutuda **Puan** seçili olmalı; {{Kaç}} kutusunda **1** yazmalı ((2)).

![((1)) Kuralın cümlesi. ((2)) Eylem: hangi sayı, kaç artsın.](11-artir.png "Kural: Kutu nesnesine tıklanınca Puan sayısını 1 artır")

Cümleyi oku: *Kutu nesnesine tıklanınca: Puan sayısını 1 artır.*

!note Azaltmak için
{{Kaç}} kutusuna eksi bir sayı yaz: **-1** yazarsan sayı bir azalır. Can azaltan kurallar böyle yapılır.

### 3. Puanı ekranda göster

Sayı şu an var ama görünmüyor. Onu bir yazıyla gösterirsin.

1. Pencereyi kapat.
2. {{Metin}} aracını (**T**) al, sahnenin sol üstüne tıkla ve şunu yaz: `Puan: {Puan}`
3. [[Esc]]'e bas.

Süslü parantezler önemlidir: PisiAnimasyon film oynarken `{Puan}` yazan yeri sayının o anki değeriyle değiştirir.

![Sen `{Puan}` yazarsın; film sayıyı gösterir.](11-yazi.png "Üstte programdaki Puan: {Puan} yazısı, altta filmdeki Puan: 0 yazısı" 660)

4. {{Filmi Test Et}} ve kutuya tıkla. Sayı her tıklamada bir artar.

!why Ne oldu?
Üç ayrı şey kurdun ve hepsi adla birbirine bağlı: **Puan** adında bir sayı, onu değiştiren bir kural ve onu gösteren bir yazı. Ad tam aynı yazılmalı; "puan" ile "Puan" aynı sayılmaz.

### 4. Sayı bir değere gelince

Puan 5 olunca bir şey olsun: kutu kaybolsun.

1. {{Kurallar ve Sayılar…}}'ı aç ve {{Yeni Kural}}'dan {{Bir sayı bir değere gelince}}'yi seç.
2. Sayı olarak **Puan**, karşılaştırma olarak **olunca**, değer olarak **5** gir.
3. {{Eylem Ekle}} → {{Nesneyi gizle}} → **Kutu**.

Cümle: *Puan sayısı 5 olunca: Kutu nesnesini gizle.*

| Karşılaştırma | Ne zaman olur | Örnek |
| olunca | Sayı tam o değere gelince | Puan 10 olunca: kazandın |
| geçince | Sayı o değerin üstüne çıkınca | Puan 100'ü geçince: ikinci bölüm |
| altına inince | Sayı o değerin altına düşünce | Can 1'in altına inince: oyun bitti |

## Koşul: "ama yalnızca şu durumda"

Her kuralın bir de **Koşul** satırı vardır. Normalde "her zaman" yazar. Listeden bir sayı seçersen kural yalnızca o sayı belli bir değerdeyken işler.

![Koşul listesi: "her zaman" ya da tanımladığın sayılardan biri.](11-kosul.png "Koşul listesi açık: her zaman ve Puan")

Örnek: *Kapı nesnesine tıklanınca, Anahtar sayısı 1 ise: sonraki sahneye git.* Kapı, anahtarı almadan açılmaz.

## Zamanla çalışan kurallar

Sayılar zamanı saymak için de kullanılır.

| Kural | Ne yapar |
| Her 1 saniyede bir: Süre sayısını 1 azalt | Geri sayan bir saat |
| Süre sayısı 0 olunca: sonraki sahneye git | Süre bitince oyun biter |
| 3 saniye sonra: Başla nesnesini gizle | "Hazır ol!" yazısı kaybolur |

Geri sayım için **Süre** adında, başlangıcı 20 olan bir sayı ve `Süre: {Süre}` yazan bir yazı gerekir.

!stuck Takıldıysan
- **Ekranda "Puan: {Puan}" yazıyor, sayı gelmiyor.** Programın içinde böyle görünür; sayı yalnızca filmde gösterilir. Filmde de böyleyse, yazıdaki ad ile sayının adı aynı değildir.
- **Eylemde sayı listesi "(seçin)" diyor.** Henüz sayı tanımlamadın. Önce {{Yeni Sayı}}.
- **Puan artıyor ama 5'te kutu kaybolmuyor.** Karşılaştırmayı kontrol et: bir tıklama puanı 2 artırıyorsa 5'e hiç "tam" gelmez; "geçince" kullan.

## Sıra sende

- [ ] **Can** adında, başlangıcı 3 olan ikinci bir sayı ekle ve `Can: {Can}` yazısıyla göster.
- [ ] Boşluk tuşuna basılınca Can 1 azalsın. Can 1'in altına inince kutu kaybolsun.
- [ ] Geri sayan bir saat yap: 10'dan başlasın, her saniye 1 azalsın, 0 olunca kutu gizlensin.
- [ ] Zor soru: kutuya tıklamak 1 puan veriyor. Puan 5'i geçtikten sonra her tıklamanın 2 puan vermesini nasıl sağlarsın? (İpucu: iki kural, iki farklı koşul.)

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Sayı | Oyunun aklında tuttuğu, adı olan bir değer: Puan, Can, Süre. |
| Başlangıç | Sayının film başlarkenki değeri. |
| `{Ad}` | Bir yazının içinde sayının o anki değerini gösteren yer tutucu. |
| Koşul | Kuralın yalnızca belli bir durumda işlemesini sağlayan ek şart. |

!teacher Öğretmene not
Bu bölüm "değişken" kavramını o sözcüğü kullanmadan öğretir: adı olan, değeri değişen, ekranda gösterilebilen bir sayı. İsterseniz dersin sonunda "programcılar buna değişken der" diyebilirsiniz. Ad uyuşmazlığı (büyük-küçük harf, boşluk) en sık görülen hatadır; öğrencilerden sayının adını yazıya kopyala-yapıştır ile geçirmelerini isteyin. "olunca" ile "geçince" arasındaki fark ince ama önemlidir ve "Takıldıysan" kutusundaki son madde bunun gerçek bir örneğidir. Geri sayım alıştırması bir sonraki bölümlerdeki oyunların hepsinde yeniden kullanılacak.
