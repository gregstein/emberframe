@kisim 1. Başlarken
# Kaydetmek, açmak, yayımlamak
> Yaptığın filmi kaybetmemek için kaydedecek, yarın yeniden açacak ve arkadaşına gönderebileceğin tek bir dosyaya dönüştüreceksin.
:: ⏱ 10 dakika | 🎯 1. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

!goal Bu bölümde öğreneceklerin
Çalışma dosyası ile yayımlanmış film arasındaki farkı, filmi kaydetmeyi, açmayı ve HTML5 olarak yayımlamayı öğreneceksin.

## İki ayrı dosya

PisiAnimasyon ile iki tür dosya üretirsin. Birbirine karıştırmamak önemli:

| | Çalışma dosyası | Yayımlanmış film |
| Uzantısı | `.sahne` | `.html` |
| Ne işe yarar | Üzerinde çalışmaya devam edersin | İzlenir ve oynanır |
| Neyle açılır | PisiAnimasyon | Herhangi bir tarayıcı |
| Kime verilir | Kendine, öğretmenine | Herkese |
| Değiştirilebilir mi | Evet | Hayır |

Kural basit: **her zaman önce kaydet, sonra yayımla.** Yayımlanmış filmi PisiAnimasyon'da geri açıp düzeltemezsin.

## Adım adım

### 1. Kaydet

1. {{Dosya > Kaydet}}'e tıkla ((1)) ya da [[Ctrl]]+[[S]] tuşlarına bas.
2. İlk kaydedişte bir pencere açılır: dosyaya bir ad yaz (örneğin `ilk-oyunum`), nereye kaydedileceğini seç ve **Kaydet**'e tıkla.
3. Sonraki [[Ctrl]]+[[S]]'ler soru sormadan aynı dosyanın üstüne yazar.

![Dosya menüsü. ((1)) Kaydet, ((2)) Aç.](04-dosya.png "Dosya menüsü açık; Kaydet ve Aç işaretli" 300)

!note Kaydedilmemiş değişiklik var mı?
Pencerenin başlığına bak. Dosya adının yanında yıldız (`*`) varsa son kaydettiğinden beri bir şey değiştirmişsin demektir.

### 2. Yeniden aç

1. {{Dosya > Aç…}}'a tıkla ((2)) ya da [[Ctrl]]+[[O]] tuşlarına bas.
2. `.sahne` dosyanı bul ve aç. Her şey bıraktığın gibi gelir: şekiller, davranışlar, adlar.

### 3. Yayımla

1. {{Kontrol > HTML5 Olarak Yayımla…}}'ya tıkla.
2. Filme bir ad ver ve kaydet. Sonunda `.html` uzantılı **tek bir dosya** oluşur.
3. O dosyaya çift tıkla: tarayıcıda açılır ve oynar.

![Kontrol menüsü. Filmi Test Et denemek içindir; HTML5 Olarak Yayımla kalıcı dosyayı üretir.](04-kontrol.png "Kontrol menüsü; HTML5 Olarak Yayımla işaretli" 380)

!why Ne oldu?
Yayımlanan dosyanın içinde filmin her şeyi vardır: resimler, sesler, davranışlar. Yanında başka dosya gerekmez, internet gerekmez, PisiAnimasyon gerekmez. E-postayla gönderebilir, bir belleğe kopyalayabilir, okulun sitesine koyabilirsin. Açan kişinin bilgisayarında yalnızca bir tarayıcı olması yeter. Yayımladığın film **senindir**: dilediğin gibi paylaşabilirsin.

## Filmi Test Et ile Yayımla arasındaki fark

| | Filmi Test Et | HTML5 Olarak Yayımla |
| Ne zaman | Çalışırken, sürekli | İş bitince |
| Dosya nereye gider | Geçici bir yere; sen uğraşmazsın | Senin seçtiğin yere |
| Kısayol | [[Ctrl]]+[[Enter]] | Yok |

## Sıra sende

- [ ] 1. bölümde yaptığın oyunu `ilk-oyunum.sahne` adıyla kaydet.
- [ ] PisiAnimasyon'u kapat, yeniden aç ve dosyanı {{Dosya > Aç…}} ile geri getir.
- [ ] Oyunu yayımla ve `.html` dosyasını yanındaki arkadaşının bilgisayarında açtır.
- [ ] Zor soru: yayımladıktan sonra kutunun rengini değiştirdin. Arkadaşındaki dosya değişir mi? Neden?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Çalışma dosyası | `.sahne` uzantılı, PisiAnimasyon'da açılıp düzenlenen dosya. |
| Yayımlamak | Filmi tarayıcıda oynayan tek bir `.html` dosyasına dönüştürmek. |
| `*` işareti | Başlıkta görünürse kaydedilmemiş değişiklik var demektir. |

!teacher Öğretmene not
Kısa bir bölümdür ama dönem boyunca en çok zaman kazandıran alışkanlığı kurar. Öğrencilerin ödevi `.html` olarak mı `.sahne` olarak mı teslim edeceğini baştan söyleyin: değerlendirmek için `.html` yeter, düzeltme isteyecekseniz `.sahne` gerekir. Ortak bilgisayarlarda öğrenciler dosyalarını kendi klasörlerine kaydetmeli; "Farklı Kaydet" ile başkasının dosyasının üstüne yazma kazaları sık görülür. Son "zor soru", çalışma dosyası ile yayımlanmış film arasındaki farkı kavrayıp kavramadıklarını ölçer.
