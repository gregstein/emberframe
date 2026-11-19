@kisim 6. Çizim ve karakter
# Ses
> Bir oyunu canlandırmanın en kolay yolu: her şeye bir ses. Altın toplanınca "çın", vurulunca "bum".
:: ⏱ 20 dakika | 🎯 10. bölümü bitirenler için | 🧰 Bir iki kısa ses dosyası (ya da hazır örnekler)

!goal Bu bölümde öğreneceklerin
Bir ses dosyasını filme eklemeyi, kuralla ses çalmayı, bir kareye ses bağlamayı ve hazır oyunlardaki sesleri kendi filmine taşımayı öğreneceksin.

## Sesin iki kullanımı

| | Kuralla çalınan ses | Kareye bağlı ses |
| Ne zaman çalar | Bir olay olunca | Film o kareye gelince |
| Nerede kullanılır | Oyunlar: vuruş, puan, hata | Animasyonlar: bir sahneye eşlik eden müzik, anlatım |
| Nasıl kurulur | `… sesini çal` eylemi | Zaman çizelgesindeki {{Ses}} kutusu |

## Adım adım

### 1. Sesi filme ekle

1. Zaman çizelgesinin altındaki {{Ses İçe Aktar…}} düğmesine tıkla.
2. Bir ses dosyası seç. **MP3**, **OGG** ve **WAV** dosyaları kabul edilir.

Ses filmin içine girer ve dosyanın tam adıyla, uzantısı da içinde, anılır: `cin.wav` dosyası **cin.wav** adında bir ses olur. Kurallarda da bu adı kullanırsın: `cin.wav sesini çal`.

!note Ses bulamadıysan
Hazır oyun örnekleri beş ses taşır: **çın**, **vız**, **bum**, **eyvah** ve **zafer**. Bir oyun örneğini açıp üzerinde çalışırsan bu sesleri hemen kullanabilirsin.

### 2. Kuralla çal

1. {{Kurallar > Kurallar ve Sayılar…}}'ı aç.
2. Bir kuralın {{Eylem Ekle}} listesinden {{Ses çal}}'ı seç ve yanındaki listeden sesi seç.

Yazarak: `çın sesini çal` (hazır örneklerdeki ses), ya da kendi dosyan için `cin.wav sesini çal`

```
Kahraman nesnesi Altın nesnesine değince:
    Puan sayısını 1 artır
    çın sesini çal
```

### 3. Kareye bağla

1. Zaman çizelgesinde sesin başlamasını istediğin anahtar kareye tıkla.
2. {{Ses}} kutusundan sesi seç.

Zaman çizelgesinde sesli anahtar kare mavi çerçeveyle gösterilir. Film o kareye her geldiğinde ses baştan çalar.

!why İçe aktardığımda ses neden bir kareye yerleşti?
{{Ses İçe Aktar}}, sesi filme ekler ve aynı anda bulunduğun anahtar kareye bağlar. Sesi yalnızca kurallarla çalacaksan o karedeki {{Ses}} kutusunu yeniden **Yok** yap: ses filmde kalır, ama kendiliğinden çalmaz.

## İyi oyun sesi nasıl olur?

| Öğüt | Neden |
| Kısa olsun: bir saniyenin altında. | Uzun ses, bir sonrakinin üstüne biner. |
| Her olayın sesi farklı olsun. | Oyuncu ekrana bakmadan ne olduğunu anlar. |
| İyi şey tiz, kötü şey pes olsun. | Kulağın alıştığı düzen budur. |
| Az olsun. | Her şeyin sesi varsa hiçbiri duyulmaz. |

## Programda duymak

Yayımlanan filmde ses her tarayıcıda çalar. Programın içinde [[Enter]] ile oynatırken duyabilmek için bilgisayarda bir ses oynatıcı kurulu olmalıdır; Pardus'ta genellikle hazırdır. Duymuyorsan **Filmi Test Et** ile tarayıcıda dinle.

!stuck Takıldıysan
- **Tarayıcıda ses çıkmıyor.** Tarayıcılar, sen sayfaya bir kez tıklamadan ses çalmaz. Filme bir kez tıkla.
- **Ses listesi boş.** Henüz ses içe aktarmadın.
- **"… adında bir ses yok" uyarısı.** Yazdığın ad, sesin adıyla aynı değil. İçe aktarılan sesin adı, dosya adının uzantısıyla birlikte tamamıdır (`cin.wav`). Kurallar penceresindeki ses listesinden seçersen yanlış yazma olasılığı kalmaz.
- **Ses her sahnede yeniden başlıyor.** Kareye bağlı ses, o kareye her gelişte çalar. Arka plan müziğini yalnızca ilk sahnenin ilk karesine bağla.

## Sıra sende

- [ ] Top-raket oyununda top rakete değince bir ses çal.
- [ ] Bilgi yarışmasına doğru ve yanlış için farklı sesler koy (örnekte hazır).
- [ ] Bir animasyonun ilk karesine kısa bir müzik bağla.
- [ ] Zor soru: bilgisayarın mikrofonuyla kendi sesini kaydet ("çın!", "eyvah!") ve oyununda kullan. Hangi programla kaydedebilirsin?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Ses İçe Aktar | Bir ses dosyasını filme ekleyen düğme. |
| `… sesini çal` | Kural içinde sesi çalan eylem. |
| Kareye bağlı ses | Film bir anahtar kareye gelince kendiliğinden çalan ses. |

!teacher Öğretmene not
Ses kısa bir konudur ama oyunların niteliğini belirgin biçimde artırır. Sınıfta kulaklık yoksa sesi kapalı tutup bu bölümü ödev olarak verebilirsiniz. Öğrencilerin kendi seslerini kaydetmesi (son alıştırma) çok sevilir; Pardus'ta ses kaydı için hazır araçlar bulunur. Telif konusuna değinmek için iyi bir fırsattır: internetten indirilen müziklerin kullanım izni olmayabilir; kendi kaydettikleri ya da serbest lisanslı sesleri kullanmaları gerektiğini söyleyin.
