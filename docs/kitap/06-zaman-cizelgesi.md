@kisim 2. Hareket
# Zaman çizelgesi ve anahtar kare
> Davranışlar hep aynı şeyi yapar. Bir hikâye anlatmak için "şimdi burada ol, bir saniye sonra şurada" diyebilmen gerekir. Bunu zaman çizelgesi yapar.
:: ⏱ 20 dakika | 🎯 1. kısmı bitirenler için | 🧰 Yalnızca PisiAnimasyon

!goal Bu bölümde öğreneceklerin
Karenin ve anahtar karenin ne olduğunu, zaman çizelgesinde gezinmeyi, [[F6]] ile anahtar kare eklemeyi ve soğan zarıyla önceki kareyi görmeyi öğreneceksin. Sonunda top üç ayrı yerde görünecek; hareketi yumuşatmak bir sonraki bölümün işi.

## Film nedir?

Film, art arda gösterilen resimlerdir. Her resme **kare** denir. PisiAnimasyon saniyede 24 kare gösterir; yani 24 kare bir saniye, 48 kare iki saniye eder.

Pencerenin üstündeki **zaman çizelgesi** bu kareleri yan yana dizer. Üstteki sayılar kare numaralarıdır; mavi "1sn", "2sn" işaretleri saniyeleri gösterir. Kırmızı dikdörtgen **oynatma kafasıdır**: sahnede o an hangi kareyi gördüğünü söyler.

Bir **anahtar kare**, "bu karede nesneler tam olarak şöyle dursun" dediğin karedir. Zaman çizelgesinde içi dolu bir nokta ile gösterilir. Her film 1. karede bir anahtar kareyle başlar.

## Adım adım

### 1. Topu çiz

1. Yeni bir film aç ([[Ctrl]]+[[N]]).
2. {{Elips}} aracıyla sahnenin sol altına bir top çiz ve kırmızıya boya.

Şu an tek bir karen var: 1. kare, topun sol altta durduğu kare.

### 2. On ikinci karede topu yukarı taşı

1. Zaman çizelgesinde, üstünde **12** yazan sütunun altına, katmanın satırına tıkla ((1)). Kırmızı oynatma kafası oraya gider.
2. [[F6]] tuşuna bas (ya da {{Zaman Çizelgesi > Anahtar Kare Ekle}}). 12. karede bir nokta belirir: yeni anahtar karen.
3. İşaretçi aracıyla topu sahnenin üst ortasına sürükle ((2)).

![12. karedeki anahtar kare. ((1)) tıkladığın yer, ((2)) taşıdığın top, ((3)) hangi karede olduğunu gösteren kutu.](06-anahtar-kare.png "Zaman çizelgesinde 12. kare seçili, top sahnenin üst ortasında")

!why Ne oldu?
[[F6]], önceki anahtar karedeki her şeyi yeni kareye kopyaladı; sen de kopyayı taşıdın. Artık 1. karede top solda aşağıda, 12. karede ortada yukarıda. İkisinin arasındaki karelerde (2'den 11'e) top hâlâ ilk yerinde durur: bir anahtar kare, sonraki anahtar kareye kadar geçerlidir.

### 3. Yirmi dördüncü karede topu sağa indir

1. Zaman çizelgesinde **24.** kareye tıkla (mavi "1sn" işaretinin hemen solu).
2. [[F6]]'ya bas.
3. Topu sahnenin sağ altına sürükle.

![24. kare: top sağ altta. Zaman çizelgesinde üç nokta var: 1, 12 ve 24.](06-son-kare.png "Zaman çizelgesinde üç anahtar kare; top sahnenin sağ altında")

### 4. Oynat

1. [[Enter]]'a bas (ya da {{Oynat}} düğmesine tıkla).

Top yarım saniye solda bekler, birden yukarı sıçrar, yarım saniye sonra sağa atlar. Kaymaz, **zıplar**: çünkü aradaki kareleri biz çizmedik. Çizgi filmlerin eski yöntemi budur: her kareyi tek tek çizmek.

![Üç anahtar kare, aralarında hiçbir şey yok: top bir yerden öbürüne atlar.](06-film.png "Oynayan film: kırmızı top sahnede" 660){06-anahtar-kare.html 660x500}

### 5. Soğan zarı: bir önceki kareyi gör

1. Zaman çizelgesinin altındaki {{Soğan Zarı}} kutusunu işaretle.
2. 12. kareye tıkla.

![Soğan zarı açık: 12. karedeki topun yanında 1. karedeki top soluk görünür.](06-sogan-zari.png "Soğan Zarı işaretli; sahnede koyu top ve sol altta soluk top")

!why Soğan zarı ne işe yarar?
Bir kareyi çizerken önceki karede nesnenin nerede olduğunu görmek istersin; yoksa hareket düzensiz olur. Soğan zarı komşu kareleri soluk gösterir. Adını, çizerlerin üst üste koyduğu yarı saydam kâğıtlardan alır.

## Zaman çizelgesinde gezinmek

| Ne yapmak istiyorsun | Nasıl |
| Bir kareye gitmek | Zaman çizelgesinde o kareye tıkla, ya da {{Kare}} kutusuna numarasını yaz. |
| Bir kare ileri, geri | [[.]] ve [[,]] tuşları |
| Baştan oynatmak | {{Başa Sar}}, sonra [[Enter]] |
| Durmadan dönmesini istemek | {{Döngü}} kutusunu işaretle |
| Anahtar kare eklemek | [[F6]] |
| Anahtar kareyi silmek | [[Shift]]+[[F6]] |
| Filmi uzatmak | [[F5]]: kare ekler, ama anahtar kare değil |

!stuck Takıldıysan
- **Topu taşıdım, bütün karelerde yeri değişti.** Taşımadan önce [[F6]]'ya basmayı unuttun. Anahtar kare yoksa değişiklik önceki anahtar kareye yapılır. [[Ctrl]]+[[Z]] ile geri al, karede dururken [[F6]]'ya bas, sonra taşı.
- **Oynatınca hiçbir şey olmuyor.** Zaman çizelgesinde tek kare varsa oynayacak bir şey yoktur. İleri bir kareye tıklayıp [[F6]]'ya bas.
- **[[Enter]] bir kez oynatıp duruyor.** Böyle olması doğru; Flash da böyle yapar. Dönüp durmasını istersen {{Döngü}}'yü işaretle.

## Sıra sende

- [ ] 36. kareye dördüncü bir anahtar kare ekle ve topu sahnenin ortasına getir. Film artık kaç saniye?
- [ ] Topun yanına bir yazı ekle. Yazı hangi karelerde görünüyor? Neden?
- [ ] 12. kareyi seçip [[Shift]]+[[F6]]'ya bas. Ne oldu?
- [ ] Zor soru: topun her karede biraz ilerlemesini istersen (24 kare, 24 farklı yer) kaç kez [[F6]]'ya basman gerekir? Bunun daha kolay bir yolu olmalı, değil mi?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Kare | Filmin tek bir resmi. Saniyede 24 tane gösterilir. |
| Anahtar kare | Nesnelerin yerini senin belirlediğin kare. [[F6]] |
| Oynatma kafası | Zaman çizelgesindeki kırmızı işaret; hangi kareyi gördüğünü gösterir. |
| Soğan zarı | Komşu kareleri soluk gösteren yardımcı. |

!teacher Öğretmene not
Bu bölüm bilerek "kötü" bir animasyonla biter: top zıplayarak yer değiştirir. Amaç, anahtar kare kavramını ara doldurmadan bağımsız oturtmak ve bir sonraki bölümde ara doldurmanın neyi çözdüğünü hissettirmektir. En yaygın hata, [[F6]]'ya basmadan nesneyi taşımaktır; "Takıldıysan" kutusundaki ilk madde bunun içindir. Vaktiniz varsa öğrencilere defter kenarına çizilen sayfa çevirmeli animasyonu hatırlatın; kare kavramı onunla hemen oturur. Son "zor soru" bir sonraki dersin giriş cümlesidir.
