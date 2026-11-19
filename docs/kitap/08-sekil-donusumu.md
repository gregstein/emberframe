@kisim 2. Hareket
# Şekil dönüşümü
> Bir kare yavaş yavaş daireye, daire yıldıza dönüşsün. Hareket ara doldurması nesneyi taşır; şekil ara doldurması nesnenin kendisini değiştirir.
:: ⏱ 15 dakika | 🎯 7. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Dört kare, tek dönüşüm.](08-donusum.png "Mavi karenin kırmızı daireye dönüşümünden dört kare"){08-donusum.html 660x500}

!goal Bu bölümde öğreneceklerin
**Boş anahtar kare** ([[F7]]) ile yeni bir şekil çizmeyi, iki şekil arasına **şekil ara doldurması** koymayı ve hareket ara doldurmasından farkını öğreneceksin.

## Hareket mi, şekil mi?

| | Hareket ara doldurması | Şekil ara doldurması |
| Ne değişir | Nesnenin yeri, boyutu, dönüşü, saydamlığı | Nesnenin biçimi ve rengi |
| İki uçta ne olur | Aynı nesne, farklı yerde | İki ayrı şekil |
| Zaman çizelgesinde rengi | Mavi | Yeşil |
| Sonraki anahtar kare | [[F6]] (kopyalar) | [[F7]] (boş açar) |

## Adım adım

### 1. İlk şekli çiz

1. Yeni bir film aç.
2. {{Dikdörtgen}} aracıyla sahnenin soluna bir kare çiz ve maviye boya.

### 2. Son karede başka bir şekil çiz

1. Zaman çizelgesinde **24.** kareye tıkla.
2. [[F7]] tuşuna bas (ya da {{Zaman Çizelgesi > Boş Anahtar Kare Ekle}}). Sahne boşalır: bu karede henüz hiçbir şey yok.
3. {{Elips}} aracıyla sahnenin sağına bir daire çiz ve kırmızıya boya.

!why [[F6]] ile [[F7]]'nin farkı
[[F6]] önceki karedeki nesneleri yeni anahtar kareye **kopyalar**; aynı nesneyi taşıyacaksan onu kullanırsın. [[F7]] **boş** bir anahtar kare açar; bambaşka bir şey çizeceksen onu kullanırsın.

### 3. Aradaki kareleri doldur

1. **1.** kareye tıkla.
2. {{Zaman Çizelgesi > Şekil Ara Doldurması Oluştur}}'a tıkla.
3. Zaman çizelgesinde aradaki karelere tek tek tıkla, ya da [[Enter]]'a bas.

![((1)) Yeşil kareler şekil ara doldurmasını gösterir. ((2)) Şekil ara doldurmasının ayarları: yavaşlama ve ipucu sayısı.](08-yesil.png "Zaman çizelgesinde yeşil ara doldurma; sahnede mor, yarı kare yarı daire bir şekil")

Kare köşelerini yuvarlatarak daireye, mavi de mordan geçerek kırmızıya dönüşür. Aynı anda yeri de değişir.

## Dönüşüm tuhaf olursa: şekil ipuçları

Karmaşık şekillerde PisiAnimasyon hangi köşenin nereye gideceğini yanlış tahmin edebilir; şekil ortada buruşur. O zaman ona yol gösterirsin.

1. İlk karedeyken {{Zaman Çizelgesi > Şekil İpucu Ekle}}'ye tıkla ([[Ctrl]]+[[Shift]]+[[H]]). Şeklin köşesinde küçük, numaralı sarı bir işaret belirir; zaman çizelgesinin altındaki **İpucu** sayısı bir artar.
2. İşareti ilk şekilde, izlemek istediğin köşeye sürükle.
3. Son kareye git; orada aynı numaralı işaret yeşildir. Onu, o köşenin dönüşmesini istediğin yere sürükle.

İlk karedeki işaret sarı, son karedeki yeşildir. Çoğu zaman bir iki ipucu yeter. Hepsini silmek için {{Zaman Çizelgesi > Tüm İpuçlarını Kaldır}}.

!stuck Takıldıysan
- **"Şekil Ara Doldurması Oluştur" bir şey yapmadı.** İki uçta da düz birer şekil olmalı. Gruplar, yazılar ve semboller şekil değiştiremez; önce {{Değiştir > Grubu Çöz}} ile grubu çöz.
- **Son karede eski şekil de duruyor.** [[F7]] yerine [[F6]]'ya basmışsın. Eski şekli seçip sil.
- **Şekil ortada buruşuyor.** Şekil ipucu ekle; yukarıdaki bölüme bak.

## Sıra sende

- [ ] 48. kareye [[F7]] ile bir yıldız çiz (sağdaki {{Otomatik Şekiller}} panelinden sahneye sürükleyebilirsin) ve daireyi yıldıza dönüştür.
- [ ] İlk aralığa 100 yavaşlama ver.
- [ ] Küçük bir daireyi kocaman bir daireye dönüştür. Bunu hareket ara doldurmasıyla da yapabilir miydin?
- [ ] Zor soru: "A" harfini "B" harfine dönüştürmek istiyorsun ama yazılar şekil değiştiremiyor. Ne yapabilirsin? (İpucu: {{Metin > Yollara Dönüştür}}.)

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Boş anahtar kare | İçinde hiçbir şey olmayan yeni anahtar kare. [[F7]] |
| Şekil ara doldurması | Bir şeklin başka bir şekle dönüşmesini programın hesaplaması. |
| Şekil ipucu | Dönüşümde hangi noktanın nereye gideceğini gösteren işaret. |

!teacher Öğretmene not
Öğrenciler en çok iki ara doldurma türünü karıştırır. Tahtaya baştaki tabloyu yazıp "aynı nesne mi taşınıyor, yoksa şekil mi değişiyor?" sorusunu sordurun. Şekil ara doldurmasının gruplarla ve sembollerle çalışmaması Flash'ta da aynıdır ve sınavlarda sık sorulur. Şekil ipuçları ileri bir konudur; sınıfın düzeyine göre atlanabilir.
