@kisim 1. Başlarken
# İlk oyunun: yürüyen ve zıplayan kutu
> On dakikada, tek satır kod yazmadan, ok tuşlarıyla yürüyen ve zıplayan bir kutu yapacaksın. Yanına bir de durmadan dönen bir kare koyacaksın.
:: ⏱ 10 dakika | 🎯 İlk kez açanlar için | 🧰 Yalnızca PisiAnimasyon

![Bölümün sonunda elinde bu olacak. Mavi kutuyu ok tuşlarıyla sen yürütüyorsun; yeşil kare kendi kendine dönüyor.](01-sonuc.png "Tarayıcıda oynayan film: mavi kutu havada, yeşil kare dönmüş" 486){01-ilk-oyun.html 660x500}

!goal Bu bölümde öğreneceklerin
PisiAnimasyon penceresinin bölümlerini tanıyacak, bir şekil çizip ona renk verecek, *davranış* denen hazır hareketleri bir tıkla ekleyecek ve yaptığın filmi tarayıcıda deneyeceksin.

## Önce pencereyi tanıyalım

PisiAnimasyon'u aç. Karşına aşağıdaki pencere gelir. Kalabalık görünebilir; bu bölümde yalnızca numaralı altı yeri kullanacağız.

![PisiAnimasyon penceresi.](01-pencere.png "PisiAnimasyon penceresi, altı yeri numaralanmış")

| No | Adı | Ne işe yarar |
| ((1)) | Araçlar | Çizim araçları. Bu bölümde yalnızca Dikdörtgen aracını kullanacağız. |
| ((2)) | Sahne | Beyaz alan. Filmde görünen her şey buraya çizilir. |
| ((3)) | Zaman çizelgesi | Filmin karelerini gösterir. Bu bölümde dokunmayacağız. |
| ((4)) | Davranışlar menüsü | Hazır hareketler. Bu bölümün yıldızı. |
| ((5)) | Özellikler | Seçili şeklin boyutu, yeri, rengi. |
| ((6)) | Filmi Test Et | Filmi tarayıcıda açar; oyunu orada oynarsın. |

## Adım adım

### 1. Kutuyu çiz

1. Soldaki araçlardan {{Dikdörtgen}} aracına tıkla ((1)).
2. Sahnenin alt yarısında bir yere farenin tuşunu bas, basılı tutarak sağ alta doğru sürükle ve bırak. Bir kutu oluşur ((2)). Büyüklüğü önemli değil; parmak kadar bir kutu yeter.
3. Sağdaki {{Stiller}} panelinde mavi kareye tıkla ((3)). Kutu maviye boyanır.

![Araç, çizdiğin kutu ve renk. Kutunun çevresindeki küçük kareler onun *seçili* olduğunu gösterir.](01-kutu-ciz.png "Dikdörtgen aracı, sahnede çizilmiş mavi kutu ve Stiller panelindeki mavi kare işaretli")

!stuck Takıldıysan
Kutudan başka bir yere tıkladığın için seçim kalktıysa, soldaki araçların en üstündeki siyah ok aracını al ve kutuya bir kez tıkla. Sonraki adımlar hep seçili şekle uygulanır.

### 2. Kutuya yürümeyi ve zıplamayı öğret

1. Kutu seçiliyken sağ üstteki {{Davranışlar}} menüsünü aç ((1)).
2. {{Ok Tuşlarıyla Yürü}}'ye tıkla ((2)). Menü kapanır.
3. {{Davranışlar}} menüsünü yeniden aç ve bu kez {{Düş ve Zıpla}}'ya tıkla ((3)).

![Davranışlar menüsü. Çizginin üstündekiler kendi kendine olan hareketler, altındakiler oyunla ilgili olanlardır.](01-davranis-menu.png "Davranışlar menüsü açık; Ok Tuşlarıyla Yürü ve Düş ve Zıpla işaretli" 600)

!why Ne oldu?
Kutuya iki *davranış* verdin. Davranış, bir şekle "bundan sonra hep şunu yap" demektir. **Ok Tuşlarıyla Yürü** kutuyu sağ ve sol okla yürütür. **Düş ve Zıpla** ona ağırlık verir: kutu sahnenin altına düşer, yukarı ok ya da boşluk tuşuyla zıplar. Bir şeklin aynı anda birkaç davranışı olabilir.

### 3. Dönen kareyi ekle

1. {{Dikdörtgen}} aracını yeniden al ve sahnenin üst yarısına ikinci bir kutu çiz. Bu kez kare gibi olsun ((1)).
2. {{Stiller}} panelinde yeşil kareye tıkla.
3. {{Davranışlar}} menüsünü aç ve en üstteki {{Dön}}'e tıkla ((2)).

![Menüyü yeniden açarsan **Dön**'ün yanında ✓ görürsün: seçili şeklin o davranışı var demektir. Aynı satıra bir daha tıklarsan davranış kalkar.](01-yesil-kare.png "Sahnede yeşil kare; Davranışlar menüsünde Dön'ün yanında onay işareti")

### 4. Oyununu dene

1. Sahnenin üstündeki {{Filmi Test Et}} düğmesine tıkla (ya da [[Ctrl]]+[[Enter]]).
2. Film tarayıcıda açılır. Önce filmin üstüne bir kez tıkla, sonra dene: [[→]] ve [[←]] yürütür, [[↑]] ya da [[Boşluk]] zıplatır.

![Filmi Test Et düğmesi.](01-test-et.png "Zaman çizelgesinin altındaki Filmi Test Et düğmesi işaretli")

!stuck Takıldıysan
- **Kutu tuşlara yanıt vermiyor.** Tarayıcıda filmin üstüne bir kez tıkla; tuşlar o zaman filme gider.
- **Kutu zıplamıyor.** Kutuyu seçip Davranışlar menüsünü aç; **Düş ve Zıpla**'nın yanında ✓ olmalı.
- **Programın içinde Oynat'a bastım, kutu yürümedi.** Doğru: oyuncunun yönettiği davranışlar yalnızca **Filmi Test Et** ile, tarayıcıda çalışır. **Dön** gibi kendi kendine olanlar programın içinde de görünür.

## Sıra sende

Aşağıdakileri kendi başına dene. Her biri bir iki tıktır; yanlış bir şey yapamazsın, [[Ctrl]]+[[Z]] her şeyi geri alır.

- [ ] Yeşil kareye **Büyü Küçül** davranışını da ver. Hem dönüp hem büyüyüp küçülüyor mu?
- [ ] Üçüncü bir şekil çiz ve ona **Fareyi İzle** ver. Tarayıcıda fareni gezdir.
- [ ] Yeşil kareyi seç, {{Davranışlar > Davranış Ayarları…}}'nı aç ve "Bir tur kaç saniye" kutusuna **0.5** yaz. Ne değişti?
- [ ] Zor soru: kutunun üstüne çıkabileceği bir basamak yapabilir misin? İpucu: bir dikdörtgen çiz ve ona **Katı** ver.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Sahne | Filmde görünen beyaz alan. |
| Seçili | Çevresinde küçük kareler olan şekil; menüler ona uygulanır. |
| Davranış | Bir şekle bir tıkla verilen hazır hareket. |
| Filmi Test Et | Filmi tarayıcıda açıp oynamak. |

!teacher Öğretmene not
Bu bölüm bir ders saatinin ilk yarısına sığar. Öğrencilerden hiçbir ön bilgi beklemez; fareyi sürükleyebilmek yeterlidir. Sık görülen tek takılma, şeklin seçimini kaldırıp menüyü boşuna açmaktır: "çevresinde kareler var mı?" sorusu çoğu zaman sorunu çözer. "Sıra sende"nin son maddesi hızlı bitirenler içindir ve bir sonraki bölümün kapısını açar. Değerlendirme için öğrenciden filmi tarayıcıda oynatıp göstermesini istemek yeterlidir.
