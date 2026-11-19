@kisim 4. Oyun dersleri
# Oyun dersi: elma yakala
> İlk tam oyunun. Ağaçtan elmalar düşer, sen sepetle yakalarsın. Her elma bir puan; yere düşen her elma bir can götürür.
:: ⏱ 40 dakika | 🎯 3. kısmı bitirenler için | 🧰 Örnekler > Oyun: elma yakala

![Bitmiş oyun.](15-film.png "Elma yakala: sepet, düşen elma, puan ve can" 640){15-elma.html 640x400}

!goal Bu derste kullanacakların
Düş ve Zıpla (düşen elma), Ok Tuşlarıyla Yürü (sepet), değme ve kenar olayları, iki sayı, bir kopya ve iki sahne. Hepsini önceki bölümlerde gördün; burada birleştiriyoruz.

## Önce oyna

{{Örnekler > Oyun: elma yakala}}'yı aç, {{Filmi Test Et}} ile oyna. Sonra sıfırdan kendi sürümünü yap; takıldığında örneğe dönüp bakarsın.

## Oyunun parçaları

| Nesne | Adı | Davranışları | Not |
| Üç elma | Elma | Düş ve Zıpla: çekim **60**, zıplama **0** | Çekim küçük: elma ağır ağır düşer. Üçünü farklı yüksekliklere koy; ikisi sahnenin üstünde, görünmeyen yerde başlasın. |
| Sepet | Sepet | Ok Tuşlarıyla Yürü: hız **300**; Düş ve Zıpla: çekim 900, zıplama **0** | Düş ve Zıpla sepeti yerde tutar: yukarı ok onu havaya kaldıramaz. |
| "+1" yazısı | Artı | İn Çık (60; 1.2), Belir Kaybol (1.2), Bir Süre Sonra Kaybol (0.6) | Yakalayınca çıkan, yükselip kaybolan yazı. |
| Puan yazısı | | | `Puan: {Puan}` |
| Can yazısı | | | `Can: {Can}` |

Sayılar: **Puan** (başlangıcı 0), **Can** (başlangıcı 5).

## Kurallar

```
Film başlayınca:
    Artı nesnesini gizle

Elma nesnesi Sepet nesnesine değince:
    Puan sayısını 1 artır
    Artı nesnesinin kopyasını Sepet nesnesinin yerinde oluştur
    Elma nesnesini üstte rastgele bir yere koy
    Elma nesnesini yüzde 4 hızlandır
    çın sesini çal

Elma nesnesi sahnenin alt kenarına değince:
    Can sayısını 1 azalt
    Elma nesnesini üstte rastgele bir yere koy
    eyvah sesini çal

Can sayısı 1 değerinin altına inince:
    2. sahneye git

Yeniden nesnesine tıklanınca:
    baştan başla
```

Bunları {{Kurallar ve Sayılar…}} penceresinde listelerden seçerek kur. (Aynı metni klavyeden yazmayı 5. kısımda öğreneceksin.)

## Yapım sırası

Her adımdan sonra filmi test et. Bir şey bozulursa neyin bozduğunu hemen anlarsın.

1. **Sepet.** Bir sepet çiz, adını koy, iki davranışını ver. Test et: sağa sola yürüyor mu?
2. **Bir elma.** Tek bir elma çiz, adını koy, Düş ve Zıpla ver ve çekimi 60 yap. Test et: ağır ağır düşüyor mu?
3. **Yakalama kuralı, yalnızca puanla.** Puan sayısını ve yazısını ekle. Değme kuralını yalnızca "Puan sayısını 1 artır" ve "üstte rastgele bir yere koy" eylemleriyle kur. Test et.
4. **Kaçırma kuralı.** Can sayısını ve yazısını ekle; alt kenar kuralını kur. Test et.
5. **Öbür elmalar.** Elmayı kopyala ([[Ctrl]]+[[C]], [[Ctrl]]+[[V]]) ve iki kopyayı daha yukarı taşı. Yeni kural gerekmez: adları aynı.
6. **Bitiş.** İkinci sahneyi, "Oyun bitti" yazısını ve Yeniden düğmesini ekle; iki sahneye de Durdur kare eylemi ver; son iki kuralı kur.
7. **Süsler.** "+1" yazısı, hızlanma, sesler, arka plan.

!why Neden bu sırayla?
Önce oyunun çekirdeği: yürüyen sepet ve düşen elma. Sonra tek bir kural. Sonra ikincisi. Süsler en sona kalır. Her şeyi birden kurup en sonda test edersen, çalışmayan şeyin nerede olduğunu bulamazsın. Oyun yapanlar hep böyle çalışır.

## Oyunu sen yönet

| Değiştir | Ne olur |
| Elmanın çekimi 60 → 120 | Elmalar hızlı düşer; oyun zorlaşır. |
| Sepetin hızı 300 → 150 | Yetişmek zorlaşır. |
| Hızlanma yüzde 4 → 0 | Oyun hiç zorlaşmaz; sıkıcı olur. |
| Can 5 → 1 | Tek hata, oyun biter. |

Bir oyunu eğlenceli yapan bu sayılardır. Arkadaşına oynat, zorlanıp zorlanmadığına bak, sayıları ona göre ayarla.

## Sıra sende

- [ ] Dördüncü bir elma ekle.
- [ ] Çürük bir elma ekle (adı "Çürük", rengi kahverengi): yakalanınca can götürsün, yere düşünce bir şey olmasın.
- [ ] Puan 20 olunca "Kazandın" yazan üçüncü bir sahneye gidilsin.
- [ ] Zor soru: altın bir elma ekle: 5 puan versin ama öbürlerinden iki kat hızlı düşsün.

!teacher Öğretmene not
Bu ders iki ders saatine yayılabilir: ilkinde çekirdek (1–4. adımlar), ikincisinde bitiş ve süsler. "Yapım sırası"ndaki "her adımdan sonra test et" alışkanlığı dersin asıl kazanımıdır; öğrencilerin her şeyi bir seferde kurma eğilimine karşı ısrarla uygulatın. "Oyunu sen yönet" tablosu oyun tasarımına giriş niteliğindedir: aynı kurallar, farklı sayılarla bambaşka bir oyun olur. Öğrencilere oyunlarını birbirlerine oynatıp sayıları buna göre ayarlatmak, test ve geri bildirim kavramlarını doğal biçimde öğretir.
