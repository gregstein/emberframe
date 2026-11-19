@kisim 4. Oyun dersleri
# Oyun dersi: uzay atışı
> Bir uzay gemisi, yukarıdan inen uzaylılar ve boşluk tuşuyla atılan mermiler. Her isabet uzaylıları biraz daha hızlandırır.
:: ⏱ 45 dakika | 🎯 13. bölümü bitirenler için | 🧰 Örnekler > Oyun: uzay atışı

![Bitmiş oyun.](18-film.png "Uzay atışı: gemi, mermi, uzaylı, gezegen ve yıldızlar" 640){18-uzay.html 640x400}

!goal Bu derste kullanacakların
Kopyalar (mermi ve patlama), tuş olayı, iki nesnenin değmesi (mermi ile uzaylı), kenar olayları ve kayan yıldızlardan bir arka plan.

## Oyunun parçaları

| Nesne | Adı | Davranışı | Not |
| Gemi | Gemi | Ok Tuşlarıyla Yürü: 320; Düş ve Zıpla: çekim 900, zıplama 0 | Düş ve Zıpla gemiyi sahnenin altında tutar. |
| Beş uzaylı | Uzaylı | Düş ve Zıpla: çekim **18**, zıplama 0 | Çok küçük çekim: yavaşça süzülür. Dördü sahnenin üstünde, sırayla girecek biçimde dizili. |
| Mermi | Mermi | Sürekli Git: hız 460, yön 90 | Asıl nesne; başta gizlenir. |
| Patlama | Patlama | Büyü Küçül, Dön, Bir Süre Sonra Kaybol (0.35) | Asıl nesne; başta gizlenir. |
| Yıldızlar | | Sürekli Git: hız 14, 36 ya da 58; yön 270 | Küçük beyaz noktalar; aşağı kayar. |

Sayılar: **Puan** (0), **Can** (3).

## Kurallar

```
Film başlayınca:
    Mermi nesnesini gizle
    Patlama nesnesini gizle

Boşluk tuşuna basılınca:
    Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur
    vız sesini çal

Mermi nesnesi sahnenin üst kenarına değince:
    Mermi nesnesini sil

Mermi nesnesi Uzaylı nesnesine değince:
    Puan sayısını 1 artır
    Patlama nesnesinin kopyasını Uzaylı nesnesinin yerinde oluştur
    Mermi nesnesini sil
    Uzaylı nesnesini üstte rastgele bir yere koy
    Uzaylı nesnesini yüzde 6 hızlandır
    bum sesini çal

Uzaylı nesnesi sahnenin alt kenarına değince:
    Can sayısını 1 azalt
    Uzaylı nesnesini üstte rastgele bir yere koy
    eyvah sesini çal

Can sayısı 1 değerinin altına inince:
    2. sahneye git

Yeniden nesnesine tıklanınca:
    baştan başla
```

## Üç yeni fikir

### İki tarafı da olan değme

*Mermi nesnesi Uzaylı nesnesine değince* kuralında iki nesne de "o an değen" nesnedir: silinen, tam o uzaylıya çarpan mermidir; yukarı gönderilen, tam o merminin vurduğu uzaylıdır. Ekranda on mermi ve beş uzaylı olsa bile kural doğru çifti bulur.

### Derinlik veren yıldızlar

Arka plandaki yıldızlar üç farklı hızla kayıyor: yavaş olanlar küçük, hızlı olanlar biraz daha büyük. Göz bunu "yavaş olanlar uzakta" diye okur ve düz bir resim derinlik kazanır. Buna **paralaks** denir; 20. bölümde kamerayla birlikte yeniden göreceğiz.

### Düşman kuyruğu

Beş uzaylının dördü başta sahnenin üstünde, görünmeyen yerde durur: her biri bir öncekinden daha yukarıda. Hepsi aynı hızla düştüğü için sırayla sahneye girerler. Düşmanları zamanlamak için kural gerekmez; onları doğru yere koymak yeter.

## Yapım sırası

1. **Gemi.** Çiz, adlandır, davranışlarını ver. Test et.
2. **Mermi ve ateş.** 13. bölümdeki üç kural: gizle, kopyala, kenarda sil. Test et.
3. **Tek uzaylı.** Çiz, Düş ve Zıpla ver, çekimi 18 yap. Test et: yavaşça iniyor mu?
4. **Vurma kuralı.** Önce yalnızca: puan artır, mermiyi sil, uzaylıyı yukarı koy. Test et.
5. **Kaçırma kuralı ve bitiş.** Can, alt kenar kuralı, ikinci sahne.
6. **Öbür uzaylılar.** Kopyala ve sahnenin üstüne, basamak basamak diz.
7. **Süsler.** Patlama, hızlanma, sesler, yıldızlar, gezegen.

!stuck Takıldıysan
- **Mermi uzaylının içinden geçiyor.** Değme kuralında adları kontrol et. Mermi çok hızlı ve uzaylı çok inceyse mermi bir karede öbür yanına atlayabilir: mermiyi yavaşlat ya da uzaylıyı kalınlaştır.
- **Tuşu basılı tutunca tek mermi çıkıyor.** Doğru: "tuşa basılınca" olayı her basışta bir kez olur. Seri ateş için her basışta yeniden basmak gerekir.
- **Uzaylılar aynı anda geliyor.** Hepsini aynı yüksekliğe koymuşsun; basamak basamak diz.

## Sıra sende

- [ ] Gemiye yukarı aşağı hareket de ver. (İpucu: Düş ve Zıpla'yı kaldır.)
- [ ] Uzaylıların düz inmek yerine sağa sola da sallanmasını sağla: onlara **Git Gel** de ver.
- [ ] Mermi sayısını sınırla: **Mermi** adında bir sayı (başlangıcı 20) her atışta azalsın; 0 olunca ateş edilemesin. (İpucu: ateş kuralına koşul.)
- [ ] Zor soru: 10 puanda bir "Patron" çıksın: büyük, yavaş ve üç isabet isteyen bir uzaylı. Onun canını nasıl sayarsın?

!teacher Öğretmene not
Bu ders 13. bölümün (kopyalar) uygulamasıdır ve öğrencilerin en sevdiği oyun türüdür. "İki tarafı da olan değme" kutusu ince ama önemli bir noktayı açıklar; öğrenciler genellikle "hangi mermi silinecek?" diye sormaz, kendiliğinden doğru çalıştığını görür; ama nedenini anlamaları ileride hata ayıklarken işe yarar. Mermi sınırı alıştırması koşullu kuralların ikinci pratiğidir. "Patron" sorusu açık uçludur: bir çözüm, "Patron Canı" adında ayrı bir sayı tutmaktır.
