@kisim 4. Oyun dersleri
# Oyun dersi: platform
> Zıplayan bir kahraman, toplanacak üç altın, nöbet tutan bir düşman ve yalnızca bütün altınlar toplanınca açılan bir kapı.
:: ⏱ 50 dakika | 🎯 3. kısmı bitirenler için | 🧰 Örnekler > Oyun: platform

![Bitmiş oyun.](17-film.png "Platform oyunu: kahraman, üç platform, altınlar, düşman ve kapı" 640){17-platform.html 640x400}

!goal Bu derste kullanacakların
Katı zemin ve platformlar, toplanınca silinen nesneler, kahramanı başa döndürme, **koşullu** bir kural (kapı) ve iki ayrı bitiş sahnesi.

## Oyunun parçaları

| Nesne | Adı | Davranışı | Not |
| Kahraman | Kahraman | Ok Tuşlarıyla Yürü: 190; Düş ve Zıpla: çekim 1100, zıplama 480 | Bir karakter çiz ve grupla. |
| Zemin | Zemin | Katı | Sahnenin altı boyunca. |
| Üç platform | Platform | Katı | Merdiven gibi, her biri bir öncekinden yüksek. |
| Üç altın | Altın | İn Çık: 6; 1.2 | Her platformun üstünde bir tane. |
| Düşman | Düşman | Git Gel: 200; 3.2 | Zeminde gidip gelir. |
| Kapı | Kapı | Yok | Sağ altta. |
| Parıltı | Parıltı | Büyü Küçül, Dön, Bir Süre Sonra Kaybol (0.35) | Altın toplanınca çıkar. |

Sayılar: **Puan** (0), **Can** (3). Üç sahne: oyun, "Aferin", "Oyun bitti".

## Kurallar

```
Film başlayınca:
    Parıltı nesnesini gizle

Kahraman nesnesi Altın nesnesine değince:
    Puan sayısını 1 artır
    Parıltı nesnesinin kopyasını Altın nesnesinin yerinde oluştur
    Altın nesnesini sil
    çın sesini çal

Kahraman nesnesi Düşman nesnesine değince:
    Can sayısını 1 azalt
    Kahraman nesnesini 30, 322 konumuna koy
    eyvah sesini çal

Kahraman nesnesi Kapı nesnesine değince, Puan sayısı 2 değerinden büyükse:
    zafer sesini çal
    2. sahneye git

Can sayısı 1 değerinin altına inince:
    3. sahneye git

Yeniden nesnesine tıklanınca:
    baştan başla
```

## Üç yeni fikir

### Toplanan şey silinir

Elma oyununda yakalanan elma yukarı gidip yeniden düşüyordu. Burada altın bir kez toplanır: `Altın nesnesini sil`. Silinen yalnızca kahramanın değdiği altındır; öbür ikisi durur.

### Kahramanı başa döndürmek

Düşmana değen kahraman başladığı yere döner: `Kahraman nesnesini 30, 322 konumuna koy`. Bu iki sayı kahramanın başlangıçtaki yeridir. Kendi oyununda kahramanı seç ve Özellikler panelindeki **X** ve **Y** kutularına bak: oradaki sayıları yaz.

### Koşullu kural: kapı

Kapı kuralının sonunda bir koşul var: *…, Puan sayısı 2 değerinden büyükse*. Kahraman kapıya değse bile puanı 3 olmadan hiçbir şey olmaz. Kurallar penceresinde bunu kuralın {{Koşul}} satırından ayarlarsın: sayı **Puan**, karşılaştırma **büyükse**, değer **2**.

!why Neden "3 ise" değil de "2'den büyükse"?
İkisi de çalışır. Ama ileride dördüncü bir altın eklersen "3 ise" koşulu kapıyı kilitler: puan 4 olur, 3 değildir. "2'den büyükse" her durumda işler. Koşul yazarken hep "ya sayı beklediğimden büyük olursa?" diye düşün.

## Yapım sırası

1. **Zemin ve kahraman.** Zemini çiz, Katı yap. Kahramanı çiz, iki davranışını ver. Test et: yürüyor ve zıplıyor mu?
2. **Platformlar.** Üç platform, hepsi Katı. Test et: en üste çıkabiliyor musun? Çıkamıyorsan platformları alçalt ya da zıplamayı artır.
3. **Altınlar.** Bir altın çiz, kuralını kur (önce yalnızca puan ve sil). Test et. Sonra kopyala.
4. **Kapı.** Kapıyı çiz, koşullu kuralı ve "Aferin" sahnesini kur. Test et: altınsız kapı açılıyor mu? Açılmamalı.
5. **Düşman.** Düşmanı, Can sayısını, kuralını ve "Oyun bitti" sahnesini ekle.
6. **Süsler.** Parıltı, sesler, manzara.

!stuck Takıldıysan
- **Kahraman platformun içinden geçiyor.** Platforma Katı vermemişsin.
- **Kahraman en üst platforma yetişemiyor.** 2. bölümdeki gibi: platformu alçalt ya da zıplama hızını büyüt.
- **Düşmana değince can üç birden gidiyor.** Kahramanı başa döndüren eylem eksik: kahraman düşmanın içinde kalıyor ve ayrılıp yeniden değiyor.
- **Kapı hiç açılmıyor.** Koşuldaki sayıyı kontrol et; üç altın varsa "2'den büyükse" olmalı.

## Sıra sende

- [ ] Dördüncü bir platform ve dördüncü bir altın ekle. Kapının koşulunu güncellemen gerekti mi?
- [ ] İkinci bir düşmanı platformlardan birinin üstüne koy.
- [ ] Hareketli bir platform yap: Katı + Git Gel.
- [ ] Zor soru: ikinci bir bölüm ekle. Kapıdan geçince yeni bir sahne açılsın, orada da altınlar ve bir kapı olsun.

!teacher Öğretmene not
Kitabın en kapsamlı dersidir ve iki ya da üç ders saatine yayılmalıdır. Öğrenciler platform oyunlarını iyi tanıdığı için motivasyon yüksektir. Koşullu kural yeni kavramdır ve "Neden 2'den büyükse?" kutusu sağlam programlama alışkanlığının iyi bir örneğidir. Can sayısının bir değmede birden fazla azalması ("Takıldıysan"daki üçüncü madde) öğrencilerin kendi başına bulup çözmesi çok öğretici olan bir hatadır; hemen yanıtı vermeyin. Son alıştırma (ikinci bölüm) dönem projesi için iyi bir çekirdektir.
