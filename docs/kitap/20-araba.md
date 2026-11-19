@kisim 4. Oyun dersleri
# Oyun dersi: araba ve kayan yol
> Hızlanan, tuşu bırakınca yavaşlayarak duran bir araba. Tekerleri gittiği yol kadar döner; yol altından akar, tepeler uzakta ağır ağır kayar.
:: ⏱ 45 dakika | 🎯 3. kısmı bitirenler için | 🧰 Örnekler > Araba sürmek

![Bitmiş film. Araba ekranda yerinde durur; hareket eden dünyadır.](20-film.png "Araba sürmek: kırmızı araba, yol çizgileri, çalılar, tepeler, bulutlar ve güneş" 720){20-araba.html 720x360}

!goal Bu derste kullanacakların
Dört yeni şey: **Teker Gibi Yuvarlan**, bir nesneyi ötekine **bağlamak**, hızlanıp yavaşlayan hareket ve **kamera**. Hepsi bu derste, tek bir arabada birleşiyor.

## Oyunun parçaları

| Nesne | Adı | Davranışı | Bağlı olduğu |
| Araba gövdesi | Araba | Kamera Beni İzlesin | |
| İki teker | Tekerlek | Teker Gibi Yuvarlan | Araba |
| Beş yol çizgisi | Yol çizgisi | Arka Plan Gibi Kay: yüzde **100**, geri gelir | |
| Üç çalı | Çalı | Arka Plan Gibi Kay: yüzde **100**, geri gelir | |
| Üç tepe | Tepe | Arka Plan Gibi Kay: yüzde **20**, **30**, **45**, geri gelir | |
| İki bulut | Bulut | Sürekli Git + Arka Plan Gibi Kay: yüzde **8** ve **5** | |
| Güneşin ışınları | | Dön + Arka Plan Gibi Kay: yüzde **0** | |
| Gökyüzü, asfalt | | Yok | |

## Kurallar

```
Sağ Ok tuşuna basılınca:
    Araba nesnesi sağa 320 hızla gitsin

Sağ Ok tuşu bırakılınca:
    Araba nesnesi yavaşlayarak dursun

Sol Ok tuşuna basılınca:
    Araba nesnesi sola 320 hızla gitsin

Sol Ok tuşu bırakılınca:
    Araba nesnesi yavaşlayarak dursun
```

## Birinci adım: araba ve tekerler

1. Gövdeyi çiz (bir iki yuvarlak köşeli dikdörtgen ve pencereler), **grupla**, adını **Araba** koy.
2. **Ayrı** bir teker çiz: bir daire, içine bir artı işareti (dönmesi görünsün diye). Grupla, adını **Tekerlek** koy.
3. Tekerleği kopyalayıp ikincisini yerine koy.
4. İki tekerleği seç, {{Davranışlar > Teker Gibi Yuvarlan}}.
5. İki teker seçiliyken {{Davranışlar > Başka Nesneye Bağla…}}'yı aç ve listeden **Araba**'ya tıkla.

!why Tekerleri neden gövdeyle gruplamıyoruz?
Davranışlar grubun kendisine uygulanır, içindeki parçalara değil. Tekerleri gövdenin grubuna katarsan dönemezler. Bu yüzden tekerler ayrı nesne olarak kalır ve gövdeye **bağlanır**: bağlı nesne, bağlandığı nesne nereye giderse oraya gider, ama kendi davranışlarını korur.

## İkinci adım: hızlanıp yavaşlayan hareket

Dört kuralı kur. İki yeni eylem var:

| Eylem | Ne yapar |
| Nesneyi yola çıkar | Nesne seçtiğin yöne, verdiğin hıza doğru **hızlanarak** gider ve gitmeye devam eder. |
| Nesne yavaşlayarak dursun | Hızı azar azar düşer; bir buçuk saniyede durur. |

Ve yeni bir olay: {{Bir tuş bırakılınca}}. Tuşa basınca araba hızlanır, bırakınca yavaşlar.

Test et. Araba hızlanıyor, yavaşlıyor, tekerler onunla birlikte dönüyor ve yavaşlıyor. Ama araba sahnenin dışına çıkıp gidiyor.

!why Teker neden kendiliğinden yavaşlıyor?
**Teker Gibi Yuvarlan**'ın hız ayarı yoktur: nesne ne kadar yol giderse o kadar döner. Araba hızlı giderse teker hızlı döner, yavaşlarsa yavaş, durursa durur, geri giderse ters döner. Hesabı sen yapmazsın.

## Üçüncü adım: kamera

1. Arabayı seç, {{Davranışlar > Kamera Beni İzlesin}}.
2. Test et.

Artık araba ekranda yerinde duruyor. Ama hiçbir şey kıpırdamıyor gibi: çünkü gökyüzü, tepeler, yol hepsi sabit resim.

## Dördüncü adım: dünyayı kaydır

Neyin nasıl kayacağını üç kural belirler:

| Nesne | Ne olur |
| Davranışı olmayan çizimler | **Ekranda sabit kalır.** Gökyüzü ve asfalt böyle. |
| Davranışı olan ya da kuralda adı geçen nesneler | **Dünyanın parçasıdır:** kamera kaydıkça tam o kadar kayar ve ekrandan çıkar. |
| **Arka Plan Gibi Kay** verilenler | Yüzdesi kadar kayar ve istenirse öbür yandan geri gelir. |

1. Yola beş kısa beyaz çizgi çiz; eşit aralıklı ve aynı genişlikte. Hepsine **Arka Plan Gibi Kay** ver; ayarı yüzde **100**.
2. Yol kenarına birkaç çalı: yine yüzde 100.
3. Tepeler: yüzde 20, 30, 45. Uzaktaki tepe en küçük sayıyı alır.
4. Bulutlar: yüzde 5–10.
5. Test et.

!why Paralaks
Trende pencereden bakarken yakındaki direkler hızla geçer, uzaktaki dağlar neredeyse kıpırdamaz. Göz, yavaş kayanı uzakta sanar. Yüzdeleri buna göre veriyorsun: yol 100, tepeler 20–45, bulutlar 5–10. Düz bir çizim böylece derinlik kazanır.

!stuck Takıldıysan
- **Tekerler arabadan ayrılıyor.** Bağlamamışsın: tekerleri seç, {{Davranışlar > Başka Nesneye Bağla…}}.
- **Bağlama listesinde "Araba" yok.** Gövdeye ad vermemişsin.
- **Yol çizgileri bir kez geçip bitiyor.** Davranış Ayarları'nda "Öbür yandan geri gelir" 1 olmalı.
- **Çizgilerin arası açılıp kapanıyor.** Çizgiler aynı genişlikte ve eşit aralıklı olmalı; hepsini sahnenin içine çiz.
- **Güneşin ışınları kayıp gidiyor.** Dönen ışınların bir davranışı var, yani dünyanın parçası sayılıyor. Onlara yüzde **0** ile Arka Plan Gibi Kay ver: ekrana sabitlenir.
- **Programın içinde Oynat'a bastım, yol kaymıyor.** Kamera yalnızca Filmi Test Et'te çalışır.

## Sıra sende

- [ ] Arabanın en yüksek hızını 320'den 600'e çıkar.
- [ ] Yol kenarına aynı genişlikte direkler diz (yüzde 100).
- [ ] Yola "Benzin" adında bir nesne koy (davranışsız, ama adı bir kuralda geçsin ki dünyanın parçası olsun): araba değince puan versin ve yok olsun.
- [ ] Zor soru: bir **Hız** sayısı tut; yukarı ok hızı 50 artırsın, aşağı ok 50 azaltsın, sağ ok arabayı o hızla yola çıkarsın. (İpucu: 5. kısımda öğreneceğin yazılı kurallarda hızın yerine sayının adını yazabilirsin: `Araba nesnesi sağa Hız hızla gitsin`.)

!teacher Öğretmene not
Bu ders dört yeni kavram tanıtır ve iki ders saati ister: ilkinde araba ve hareket (1–2. adımlar), ikincisinde kamera ve paralaks (3–4. adımlar). Her adım bilerek bir eksikle biter (araba sahneden çıkar; sonra hiçbir şey kıpırdamıyor gibi görünür) ve bir sonraki adım o eksiği giderir; öğrenciye neden gerekli olduğunu göstermenin en iyi yolu budur. Paralaks için sınıfta pencereden dışarı bakarak yürümek ya da bir tren videosu izletmek kavramı hemen oturtur. Kameranın yalnızca sağa-sola kaydığını ve program içinde oynatırken çalışmadığını baştan söyleyin.
