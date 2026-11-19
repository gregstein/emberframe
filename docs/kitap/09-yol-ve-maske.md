@kisim 2. Hareket
# Yol boyunca hareket ve maske
> Ara doldurma nesneyi dümdüz götürür. Bir kavis çizerek gitmesini istersen ona bir yol çizersin. Bir de maskeyi öğreneceksin: yalnızca bir deliğin içinden görünen resim.
:: ⏱ 25 dakika | 🎯 7. bölümü bitirenler için | 🧰 Örnekler > Yol boyunca, Spot ışığı

!goal Bu bölümde öğreneceklerin
Katmanları, bir nesneyi **kılavuz katmanındaki** yol boyunca yürütmeyi ve **maske katmanıyla** bir resmin yalnızca bir bölümünü göstermeyi öğreneceksin.

## Önce: katman nedir?

Şimdiye kadar her şeyi tek katmana çizdin. **Katman**, üst üste konmuş saydam kâğıtlar gibidir: her birine ayrı şeyler çizersin, hepsi birlikte görünür. Zaman çizelgesinin solundaki satırlar katmanlardır; üstteki katman önde görünür.

Katmanların animasyondaki asıl önemi şudur: **her katmanın kendi anahtar kareleri vardır.** Arka plan bir katmanda hiç kıpırdamadan dururken, karakter başka bir katmanda hareket eder.

| Ne yapmak istiyorsun | Nasıl |
| Yeni katman eklemek | Zaman çizelgesinin altındaki {{Yeni Katman}} düğmesi |
| Katmana çizmek | Önce zaman çizelgesinde o katmanın satırına tıkla |
| Katmanı gizlemek | Adının solundaki göze tıkla |
| Katmanı silmek | {{Katmanı Sil}} düğmesi |

## Birinci kısım: yol boyunca hareket

### Örneğe bak

1. {{Örnekler > Yol boyunca}}'yı aç ve [[Enter]]'a bas.

Ok, dümdüz gitmek yerine kavisli çizgiyi izliyor ve gittiği yöne dönüyor.

![((1)) Üstte kılavuz katmanı, altında ona bağlı "Ok" katmanı (adı içeri girintili). ((2)) Ok, çizginin üstünde ve çizginin yönüne dönük.](09-kilavuz.png "Yol boyunca örneği: kılavuz katmanı, kavisli çizgi ve çizgi üstündeki ok")

![Yayımlanan film: ok kavisi izler ve gittiği yöne döner. Kılavuz çizgisi filmde görünmez.](09-yol-film.png "Oynayan film: mavi ok kavisli bir yol izliyor" 520){09-yol.html 520x320}

### Kendin yap

1. Yeni bir filmde bir ok çiz (sağdaki {{Otomatik Şekiller}} panelinden **Ok**'u sahneye sürükle).
2. 48. karede [[F6]]'ya bas ve oku sahnenin öbür ucuna taşı. 1. kareye dönüp {{Hareket Ara Doldurması Oluştur}} de. Ok şimdilik dümdüz gidiyor.
3. {{Yeni Katman}}'a tıkla. Yeni katman okun katmanının üstüne gelir.
4. Bu yeni katmanda, {{Kalem}} ya da {{Vektör Yol}} aracıyla okun izlemesini istediğin çizgiyi çiz. Çizgi okun başladığı yerden başlayıp bittiği yerde bitsin.
5. Yeni katmanın adına **sağ tıkla** ve {{Kılavuz Katmanı}}'nı seç.
6. Okun katmanına, ara doldurmalı bir kareye tıkla ve altta beliren {{Yola yönlendir}} kutusunu işaretle.

![Katmanın adına sağ tıklayınca açılan menü.](09-katman-menu.png "Katman menüsü; Kılavuz Katmanı işaretli" 420)

!why Ne oldu?
Kılavuz katmanındaki çizgi bir raydır. Altındaki katmanın ara doldurulan nesnesi, dümdüz gitmek yerine o rayı izler. **Yola yönlendir** açıksa nesne rayın yönüne de döner; kapalıysa hep aynı yöne bakar. Çizginin kendisi yayımlanan filmde görünmez: yalnızca sana yol gösterir.

## İkinci kısım: maske

### Örneğe bak

1. {{Örnekler > Spot ışığı}}'nı aç ve {{Filmi Test Et}}'e tıkla.

![Yayımlanan film: parlak sarı yazı yalnızca dairenin olduğu yerde görünür. Daire soldan sağa kaydıkça yazı "aydınlanır".](09-maske-film.png "Koyu zeminde soluk PisiAnimasyon yazısı; sol baştaki harf parlak sarı" 520){09-maske.html 520x200}

Sahnede ise şunu görürsün:

![Sahnede maske bir beyaz daire olarak görünür. Üstteki katman maske, altındaki "Parlak" katmanı ona bağlı.](09-maske-sahne.png "Spot ışığı örneği: maske katmanı ve sahnede beyaz daire")

!why Ne oldu?
Üç katman var. En altta soluk yazı. Ortada aynı yazının parlak sarısı. En üstte bir daire, ve o katman **maske katmanı**. Maske, altındaki katmanı yalnızca kendi şeklinin olduğu yerde gösterir: daire bir delik gibidir. Daireye hareket ara doldurması verilince delik yazının üstünde gezer.

### Kendin yap

1. Bir katmana göstermek istediğin şeyi çiz (bir yazı, bir resim).
2. {{Yeni Katman}}'a tıkla ve yeni katmana bir şekil çiz: bu senin deliğin.
3. Yeni katmanın adına sağ tıkla ve {{Maske Katmanı}}'nı seç.
4. {{Filmi Test Et}} ile bak: alttaki katman yalnızca şeklin içinde görünür.
5. Şekle hareket ara doldurması ver: delik gezer.

!note Maskeyi sahnede görmek
Çalışırken maskeyi şekil olarak görürsün ki düzenleyebilesin. Sonucu görmek için sahnenin üstündeki {{Önizleme}} sekmesine tıkla ya da filmi test et.

## Bir katmanı kılavuza ya da maskeye bağlamak

Kılavuz ya da maske, hemen altındaki katmanı etkiler. Başka bir katmanın da aynı yolu izlemesini ya da aynı maskenin altında kalmasını istersen o katmanın adına sağ tıkla ve {{Kılavuza ya da Maskeye Bağla}}'yı seç. Bağlı katmanların adı içeri girintili görünür.

!stuck Takıldıysan
- **Ok yolu izlemiyor.** Okun ilk ve son anahtar karesinde çizginin uçlarına yakın durduğundan emin ol. Ayrıca okta hareket ara doldurması olmalı; kılavuz yalnızca ara doldurulan nesneleri yürütür.
- **Maske hiçbir şeyi gizlemiyor.** Sahnede öyle görünür; {{Önizleme}} sekmesine ya da filme bak.
- **Yanlış katmana çizdim.** Çizdiğini seç, [[Ctrl]]+[[X]] ile kes, doğru katmanın satırına tıkla, [[Ctrl]]+[[V]] ile yapıştır.

## Sıra sende

- [ ] Yol boyunca örneğinde çizgiyi seçip uçlarını oynat. Ok yeni yolu izliyor mu?
- [ ] Aynı örnekte **Yola yönlendir**'i kapat. Fark ne?
- [ ] Spot ışığı örneğinde daireyi büyüt. Ne değişti?
- [ ] Zor soru: bir dürbünden bakıyormuş gibi görünen bir sahne yap: manzara resmi, üstünde yan yana iki daireden oluşan bir maske.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Katman | Üst üste duran, her biri kendi anahtar karelerine sahip çizim tabakası. |
| Kılavuz katmanı | İçindeki çizgiyi, altındaki katmanın nesnesine yol yapan katman. |
| Maske katmanı | Altındaki katmanı yalnızca kendi şeklinin olduğu yerde gösteren katman. |
| Yola yönlendir | Nesnenin gittiği yöne dönmesi. |

!teacher Öğretmene not
Kılavuz ve maske katmanları Flash müfredatının son iki klasik konusudur ve PisiAnimasyon'da aynı adlarla bulunur. Bölümü iki derse bölmek rahat olur. Her iki konuda da önce hazır örneği açtırıp çözdürmek, sonra sıfırdan yaptırmak iyi sonuç verir: öğrenci neye ulaşacağını görmüş olur. Maskede en sık yaşanan şaşkınlık, sahnede maskenin şekil olarak görünmesidir; Önizleme sekmesini erkenden gösterin. "Dürbün" alıştırması sınıfça yapılabilecek güzel bir kapanıştır.
