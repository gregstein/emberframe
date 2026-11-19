@kisim 4. Oyun dersleri
# Oyun dersi: bilgi yarışması
> Her sahnede bir soru ve üç yanıt. Doğru yanıt bir puan; sonda toplam puan. Soruları sen yazarsın: matematik, tarih, İngilizce, ne istersen.
:: ⏱ 35 dakika | 🎯 14. bölümü bitirenler için | 🧰 Örnekler > Oyun: bilgi yarışması

![Bitmiş oyun.](19-film.png "Bilgi yarışması: Soru 1, Örümceğin kaç bacağı vardır, üç yanıt kutusu" 640){19-yarisma.html 640x400}

!goal Bu derste kullanacakların
Sahneler, Durdur kare eylemi, tıklama olayı ve "aynı ad, tek kural" fikri. Bu oyunun hiç klavye kullanmaması ve içeriğinin başka derslerden gelebilmesi onu sınıf için çok uygun yapar.

## Oyunun yapısı

| Sahne | İçinde ne var |
| Soru 1, Soru 2, Soru 3 | Soru yazısı, üç yanıt kutusu, puan yazısı |
| Sonuç | "Aferin", `Puan: {Puan} / 3`, Yeniden düğmesi |

Her soru sahnesinde:

| Nesne | Adı | Not |
| Doğru yanıtın kutusu | Doğru | Üstündeki sayı ayrı bir yazıdır. |
| Öbür iki kutu | Yanlış | İkisinin adı da aynı. |
| Soru yazısı | | Ad gerekmez. |
| Puan yazısı | | `Puan: {Puan}` |

Kutulara hafif bir **İn Çık** verilmiştir (6 piksel): yerlerinde hafifçe sallanırlar ve tıklanabilir oldukları anlaşılır.

Sayı: **Puan** (0).

## Kurallar

```
Doğru nesnesine tıklanınca:
    Puan sayısını 1 artır
    çın sesini çal
    sonraki sahneye git

Yanlış nesnesine tıklanınca:
    eyvah sesini çal
    sonraki sahneye git

Yeniden nesnesine tıklanınca:
    baştan başla
```

Hepsi bu. Üç kural, kaç soru olursa olsun.

## Yapım sırası

1. **İlk soru sahnesi.** Arka planı, soru yazısını ve üç kutuyu çiz. Kutuların üstüne yanıtları yaz. Doğru kutunun adını **Doğru**, öbürlerinin adını **Yanlış** koy.
2. **Durdur.** Zaman çizelgesinin altındaki {{Kare eylemi}} kutusundan **Durdur**'u seç.
3. **Sonuç sahnesi.** Yeni sahne ekle; "Aferin", puan yazısı ve Yeniden düğmesi. Buna da Durdur ver.
4. **Kurallar.** Puan sayısını ekle, üç kuralı kur. Test et: tek soruluk yarışma çalışıyor mu?
5. **Öbür sorular.** Soru sahnesini çoğalt, soruyu ve yanıtları değiştir, **kutu adlarını yeniden düzenle**: doğru yanıt her soruda başka bir kutuda olsun.

!why Yanıtın üstündeki yazı tıklamayı engellemez mi?
Hayır. Kutunun üstünde duran "8" yazısı ayrı bir nesnedir; ama davranışı yoktur ve hiçbir kuralda adı geçmez. Böyle nesneler filmde arka planın parçası olur ve tıklamayı almaz: tıklama altındaki kutuya gider. Yazıya bir davranış verirsen ya da adını bir kuralda kullanırsan tıklamayı o alır; o zaman kutu ile yazıyı gruplayıp adı gruba ver.

!stuck Takıldıysan
- **Sorular kendi kendine geçiyor.** Sahnelerde Durdur kare eylemi yok.
- **Doğru yanıta tıklıyorum, puan artmıyor.** O sahnedeki kutunun adı "Doğru" mu? Büyük harfle, Türkçe karakterlerle, tam aynı.
- **Son sorudan sonra yarışma başa dönüyor.** Sonuç sahnesi en sonda olmalı ve onda da Durdur olmalı.
- **Hep aynı kutu doğru.** Sahneyi çoğaltınca adlar da çoğalır. Her sahnede adları yeniden ver.

## Kendi yarışman

| Ders | Soru örneği |
| Matematik | 7 × 8 kaç eder? |
| Türkçe | Hangisi bir sıfattır? |
| İngilizce | "Elma" hangisidir? |
| Fen | Hangisi bir gezegendir? |
| Coğrafya | Türkiye'nin başkenti neresidir? |

## Sıra sende

- [ ] Kendi dersinden beş soruluk bir yarışma hazırla ve arkadaşına oynat.
- [ ] Sonuç sahnesindeki yazıyı toplam soru sayına göre düzelt (`Puan: {Puan} / 5`).
- [ ] Yanlış yanıtta 1 puan düşsün.
- [ ] Zor soru: her soruya 10 saniye süre koy. Süre dolunca soru yanlış sayılsın ve sonrakine geçilsin. (İpucu: her sahne başlayınca süreyi yeniden 10 yapan bir kural: "Bir sahne başlayınca".)

!teacher Öğretmene not
Kitabın disiplinler arası kullanıma en uygun dersidir. Öğrencilere kendi seçtikleri bir dersten yarışma hazırlatıp sınıfta birbirlerine oynatmak hem o dersin tekrarı hem de PisiAnimasyon pratiği olur; başka branş öğretmenleriyle ortak etkinlik için elverişlidir. Teknik olarak tek zorluk, sahne çoğaltıldığında kutu adlarının yeniden düzenlenmesidir. Sorular resimli de olabilir (bir bayrak, bir hayvan): Dosya > İçe Aktar ile resim eklenir. Son alıştırma "Bir sahne başlayınca" olayını tanıtır.
