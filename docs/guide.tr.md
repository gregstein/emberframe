# PisiWorks kullanım kılavuzu

Şekilleri ve çizgileri kısıtlamak için çizerken Shift'i basılı tutun. Tuvali kaydırmak için Boşluk tuşunu basılı tutun. Ctrl+kaydırma yakınlaştırır. Kalem yolunu bitirmek için Enter'a basın, kapatmak için ilk noktasına tıklayın.

## Eğriler

Kalem (P) ile köşe noktası koymak için tıklayın. Yumuşak nokta koymak için basıp sürükleyin: çektiğiniz iki tutamaç, yolu noktanın iki yanında büker.

Alt Seçim (A) ile bir yolu seçtiğinizde noktaları görünür. Bir noktayı seçmek için tıklayın; tutamaçları görünür. Shift+tıklama seçime nokta ekler; boş tuvalde kutu çizmek içindeki bütün noktaları seçer. Seçili noktayı, seçtiğiniz diğer noktalarla birlikte taşımak için sürükleyin; eğriyi yeniden biçimlendirmek için tutamacı sürükleyin. Karşı tutamaç da onunla birlikte döner ve nokta yumuşak kalır; tutamaçlardan yalnızca birini oynatıp sivri bir köşe yapmak için Alt'ı basılı tutun. Bir köşeden tutamaç çekmek için Alt+sürükleyin, bir noktayı yeniden köşeye çevirmek için Alt+tıklayın. Delete, seçili noktaları siler.

Kalem de seçili bir yolu düzenler. Nokta eklemek için yolun bir parçasına tıklayın. Eğri noktasını köşeye çevirmek için ona, köşe noktasını silmek için köşeye tıklayın. Bir noktadan tutamaç çekmek için noktadan sürükleyin. Açık bir yolun serbest ucuna tıklayarak çizmeye oradan devam edin; başka bir açık yolun serbest ucunda bitirerek ikisini birleştirin.

Elipsler, noktaları düzenlendiğinde birkaç eğri noktasından oluşan yollara dönüşür. Dikdörtgen önce onay ister, çünkü yola dönüşünce yuvarlaklık ayarı kalmaz. Serbest Biçim, Bıçak ve yol işlemleri sonucu yine eğri yol olarak verir: başka bir daireyle birleştirilen daire yine eğrilerden oluşur.

Arayüz dili masaüstünüzün diline göre seçilir. Değiştirmek için Düzenle > Dil menüsünü kullanın.

## Görüntüleri düzenleme

Bir görüntüyü düzenlemek için Dosya > Aç (Ctrl+O), geçerli belgeye bir görüntü eklemek için Dosya > İçe Aktar (Ctrl+R) komutunu kullanın.

Fırça, Kurşun Kalem ve Silgi, işaretçinin altındaki bitmap'in piksellerini değiştirir. Bitmap dışında Fırça ve Kurşun Kalem vektör konturları çizer. Fırça kontur rengiyle boyar, Boya Kovası dolgu rengiyle doldurur. Fırça veya Silgi etkinken fırça boyutunu Özellikler panelinden ayarlayın.

Kırp aracıyla sürükleyin, ardından uygulamak için Enter'a basın veya çift tıklayın. Esc iptal eder.

Kaydet, açılan görüntünün üzerine hiçbir zaman yazmaz. Nesnelerinizi koruyan bir `.pisi` belgesi oluşturur; Dosya > Dışa Aktar ise düz bir görüntü yazar.

Seçim Çerçevesi (M), Kement (L) ve Sihirli Değnek (W) bir bitmap üzerinde piksel seçer. Seçime eklemek için Shift'i, seçimden çıkarmak için Alt'ı basılı tutun. Boyama, doldurma ve filtreler seçimin içinde kalır. Kes, Kopyala ve Sil seçili piksellere uygulanır; Yapıştır bunları yeni bir bitmap olarak ekler. Seçili pikselleri taşımak için İşaretçi ile seçimin içinden sürükleyin veya ok tuşlarına basın; geride bir kopya bırakmak için Alt'ı basılı tutun. Seçim Çerçevesi veya Kement ile seçimin içinden sürüklemek yalnızca seçim sınırını taşır; Ctrl basılıyken pikselleri taşır. Ctrl+D veya Esc seçimi kaldırır.

Seç menüsü piksel seçimini değiştirir. Benzerini Seç, bitmap'in her yerindeki benzer renkli pikselleri seçime katar; Tersini Seç (Ctrl+Shift+I) geri kalan her şeyi seçer. Seçim Çerçevesini Genişlet ve Daralt, seçimi verilen piksel kadar büyütür veya küçültür; Seçim Çerçevesine Kenar Çiz onu kenarı boyunca bir şeride çevirir; Seçim Çerçevesini Düzgünleştir pürüzlü kenarı düzeltir. Geçiş Yumuşatma, seçimin kenarını verilen yarıçap boyunca yumuşatır; silme, kopyalama, taşıma, boyama ve filtreler de kenarda yumuşayarak uygulanır. Piksel seçili değilken Tersini Seç, seçili olmayan nesneleri seçer.

Lastik Damgası (S): kaynağı belirlemek için Alt+tıklayın, ardından oradan klonlamak için sürükleyin.

Değiştir > Seçimi Düzleştir (Ctrl+Alt+Shift+Z) seçili nesneleri tek bir bitmap yapar. Aşağıda Birleştir (Ctrl+E) onları hemen altlarındaki bitmap ile birleştirir. Katmanları Düzleştir, görünen katmanlardaki her şeyi tek katmanda tek bir bitmap yapar; gizli katmanlar silinir, dilimler ve sıcak noktalar korunur. Değiştir > Tuval > Tuvali Kes, tuvali çizime kadar kırpar.

Düzenle > Ekle > Boş Bitmap, üzerine boyamak için tuval boyutunda saydam bir bitmap ekler. Düzenle > Klonla (Ctrl+Shift+D) seçimi yerinde kopyalar; Nitelikleri Yapıştır (Ctrl+Alt+Shift+V) seçime, kopyalanan nesnenin dolgusunu, konturunu, opaklığını, karışım modunu ve filtrelerini verir. Filtreler > Filtreyi Tekrarla (Ctrl+Alt+Shift+F) son filtreyi yeniden uygular. Dosya > Geri Dön, dosyanın son kaydedilen haline döner. Görünüm > Seçimi Sığdır (Ctrl+Alt+0) seçime yakınlaşır.

## Vektörleri yeniden biçimlendirme

Ölçek (Q) aracı, seçili her şeyi tutamaçlarından yeniden boyutlandırır. Shift'i basılı tutmadığınız sürece köşeler oranları korur.

Eğriltme ve Deformasyon araçları, Ölçek düğmesini paylaşır: listelemek için düğmeye basılı tutun veya üçü arasında sırayla geçmek için Q'ya yeniden basın. Üçünde de döndürmek için seçim kutusunun dışını sürükleyin; 15 derecelik adımlarla döndürmek için Shift'i basılı tutun. Döndürme, kutunun ortasındaki küçük dairenin çevresinde yapılır; bu daireyi önce başka bir yere sürükleyebilirsiniz.

- **Eğriltme**: seçimi yatırmak için bir kenar tutamacını, perspektif için bir köşe tutamacını sürükleyin; perspektifte köşe ile komşusu birbirinden uzaklaşır veya birbirine yaklaşır.
- **Deformasyon**: yalnızca bir köşeyi oynatmak için köşe tutamacını, bütün bir kenarı taşımak için kenar tutamacını sürükleyin.

Sürüklerken çizim dönüşümü yalnızca önizler; başka bir tutamaçla veya üç araçtan bir diğeriyle devam edebilirsiniz. Uygulamak için Enter'a basın veya kutunun içine çift tıklayın, her şeyi eski haline getirmek için Esc'ye basın. Kutunun içini sürüklemek seçimi taşır.

Değiştir > Dönüştür menüsünde aynı araçlar ile 180° Döndür, 90° döndürmeler ve iki çevirme bulunur. Nümerik Dönüştürme (Ctrl+Shift+T) yüzdeyle ölçekler, piksel cinsinden boyut verir veya bir açıyla döndürür. Merkez noktasını sürüklediyseniz o noktayı, sürüklemediyseniz seçimin ortasını sabit tutar.

Metin dönüştürüldüğünde metin olarak kalır; düzenlemeye ve boyutunu değiştirmeye devam edebilirsiniz. Sembol kitaplığa bağlı kalır; dikdörtgen yuvarlaklığını, otomatik şekil ayarlarını korur. Değiştir > Dönüştür > Dönüştürmeleri Kaldır bunları yeniden dik hale getirir. Yatay Çevir ve Dikey Çevir metni, sembolleri ve grupları da aynalar. Bitmap'ler yeniden çizilir. Web alanları dönüştürülmez. Metin > Yollara Dönüştür, metni harflerinin ana hatlarına çevirir.

Serbest Biçim (O), işaretçinin yakınındaki bir yolu veya şekli çeker. Bıçak (Y, Kurşun Kalem seçiliyken yeniden basın) seçili vektörleri sürüklenen çizgi boyunca keser. Hiçbir şey seçili değilken Bıçak, kestiği her vektörü böler. İkisi de dikdörtgenleri ve elipsleri yola dönüştürür.

## Canlı filtreler

Özellikler panelindeki Filtreler, seçili nesnelere efekt ekler: renk ayarları, eğim ve kabartmalar, bulanıklaştırmalar, parazit, gölge ve ışımalar, keskinleştirme. Eklemek ve ayarlamak için listeden birini seçin. Her filtre; onu kapatan bir kutu, ayarlarını yeniden açan adı ve onu kaldıran bir eksi ile listelenir. Filtreler listenin başından sonuna doğru, nesne her çizildiğinde uygulanır; bu yüzden nesnenin kendisi düzenlenebilir kalır. Bir filtrenin yerini değiştirmek için adını yukarı veya aşağı sürükleyin. Bir nesneden oluşturulan stil, nesnenin filtrelerini de saklar.

Düzeyler, her düzeyde kaç piksel olduğunu ve girişte siyah, orta gri ve beyaz, çıkışta siyah ve beyaz sayılacak düzeyleri gösterir. Eğriler, girdi düzeyinden çıktı düzeyine bir eğri gösterir: nokta eklemek için eğriye tıklayın, bükmek için bir noktayı sürükleyin, kaldırmak için noktayı kutunun dışına sürükleyin. İkisinde de Kanal listesi vardır; kırmızı, yeşil ve mavinin, üçünün ortak ayarlarından ayrı kendi ayarları olur. Otomatik düğmesi her rengi en koyu düzeyinden en açık düzeyine yayar. Düzeyler'de sayıları yazmak yerine histogramın ve renk şeridinin altındaki üçgenleri sürükleyebilirsiniz. Üç damlalık siyah, gri veya beyaz noktayı belirler: birini seçin, sonra nesnenin üzerinde bir renge tıklayın.

Filtre pencerelerinde Önizleme kutusu vardır. Kapalıyken nesne, kutu yeniden işaretlenene veya pencere Tamam ile kapatılana kadar pencere açılmadan önceki haliyle görünür.

Photoshop Canlı Efektleri; Alt Gölge, İç Gölge, Dış Işıma, İç Işıma, Eğim ve Kabartma, Saten, Renk Kaplama, Degrade Kaplama, Desen Kaplama ve Kontur efektlerini bir arada tutan tek bir filtredir. İstediklerinizi işaretleyin ve ayarlarını görmek için efektin adına tıklayın: gölge ve ışımalarda karışım modu, opaklık, renk, açı, mesafe, yayılma veya boğma, boyut ve parazit; eğimde stil, teknik, derinlik, yön, yumuşatma, yükseklik, açıkton ve gölge; degradede stil, açı ve ölçek; konturda konum ve dolgu türü. Desenler kısa bir yerleşik listeden seçilir.

Degrade, altında duraklarıyla gösterilir. Durak eklemek için degradenin altına tıklayın, taşımak için durağı sürükleyin, kaldırmak için degradeden uzağa çekin. Seçili durağın rengi, yüzde olarak konumu ve opaklığı görünür. Hazır Ayarlar, degradeyi hazır bir degradeyle değiştirir. Kaplamanın, degrade konturun ve degrade ışımanın ayrı degradeleri vardır; degrade ışıma nesnenin kenarından başlar ve Değişim renklerini dağıtır.

Alt gölge ve dış ışıma, nesnenin altında kalanlarla kendi karışım modlarıyla karıştırılır. Listede Photoshop Canlı Efektleri'nden sonra gelen bir filtre, nesneye olduğu gibi onlara da uygulanır.

## Maskeler

Maske, bir nesnenin bir bölümünü gizler. Komutlar Değiştir > Maske menüsündedir:

- **Maske olarak Grupla**: iki veya daha fazla nesne seçin; en üstteki, diğerlerinin maskesi olur. Maske ne kadar açık renkliyse o kadar çok şey görünür; temiz bir parça kesmek için beyaz bir şekil kullanın.
- **Maske Olarak Yapıştır**: bir nesneyi kopyalayın, başka birini seçip bu komutu verin; kopyalanan nesne onu anahattıyla maskeler. **İçine Yapıştır** (Ctrl+Shift+V) tersini yapar: kopyaladığınız şey seçili nesnenin içinde görünür, seçili nesne çevresinde görünmeye devam eder.
- **Tümünü Göster** ve **Tümünü Gizle**, üzerine boyamak için her şeyi gösteren veya hiçbir şeyi göstermeyen bir bitmap maskesi ekler. Bir bitmap üzerinde piksel seçiliyken **Seçimi Göster** ve **Seçimi Gizle** maskeyi seçimden oluşturur.
- **Maskeyi Kapat** maskeyi kaldırmadan kapatır; **Maskeyi Sil**, önce nesneye uygulanıp uygulanmayacağını sorduktan sonra kaldırır.

Maskeli nesnenin Özellikler panelinde Maske alanı görünür: vektör maskesi için Yol anahattı veya Gri tonlama görünümü ile Dolguyu ve konturu göster; bitmap maskesi için Gri tonlama görünümü veya Alfa kanalı. Oradaki Maskeyi Düzenle düğmesi ya da Katmanlar panelinde maskenin küçük resmine tıklamak maskenin kendisini seçer; maskeyi herhangi bir araçla taşıyabilir, yeniden biçimlendirebilir veya boyayabilirsiniz. Beyaz boya gösterir, siyah gizler. Bitirmek için başka bir şey seçin.

Katmanlar panelinde nesne ile maskesi arasındaki zincir, ikisinin bağlı olduğunu ve birlikte taşındığını gösterir. Maskeyi serbest bırakıp nesneyi altında taşımak için zincire tıklayın, yeniden bağlamak için bir daha tıklayın. Maske Olarak Yapıştır, vektörün bitmap maskesine mi dönüşeceğini yoksa vektör mü kalacağını ve var olan maskenin değiştirilip değiştirilmeyeceğini ya da ona eklenip eklenmeyeceğini sorar.

## Araç grupları

Köşesinde küçük üçgen olan düğme birkaç aracı temsil eder. Listelemek için düğmeyi basılı tutun veya aralarında gezinmek için tuşuna yeniden basın; düğme en son kullanılanı gösterir.

- İşaretçi (V): Arkasındakini Seç, her tıklamada farenin altındaki bir alttaki nesneyi seçer.
- Ölçek (Q): Eğriltme ve Deformasyon'dan sonra 9 dilimli ölçekleme gelir. Dört mavi çizgi seçimi dokuz parçaya böler; bir çizgiyi taşımak için sürükleyin, sonra seçimin bir tutamacını sürükleyin: köşeler boyutunu korur, yalnızca orta kısım uzar; böylece yuvarlak köşeler yuvarlak kalır.
- Kırpma (C): Dışa Aktarma Alanı, sayfanın yalnızca üzerinden sürüklediğiniz kısmını dışa aktarır. Enter'a basın veya çift tıklayın; o kısım için Görüntü Önizleme açılır.
- Seçim Çerçevesi (M): Oval Seçim Çerçevesi oval bir alan seçer. Kement (L): Çokgen Kement düz kenarlarla seçer; her köşeye tıklayın, kapatmak için çift tıklayın veya ilk köşeye tıklayın.
- Bulanıklaştırma (R): Keskinleştir, Soldurma (açar), Yakma (koyulaştırır) ve Leke (boyayı fırça boyunca sürükler) Bulanıklaştırma gibi fırçanın altında çalışır. Soldurma ve Yakma'nın Özellikler'de Pozlama ve ton Aralığı; Leke'nin Basınç ayarı vardır ve Leke, kontur rengiyle yüklü başlayabilir.
- Lastik Damgası (S): Renk Değiştir, Kimden rengini Tolerans içinde Kime rengine çevirir; Kimden rengi yoksa fırçanın başladığı yerdeki rengi değiştirir. Kırmızı Göz Kaldırma, üzerinden sürüklediğiniz alandaki gözlerin kırmızısını Tolerans ve Güç ayarına göre alır.
- Kalem (P): Vektör Yol serbest elle yol çizer. Yolu Yeniden Çiz bir yolun bir kısmını değiştirir: çizgiye yolun üzerinde başlayın ve yine yolun üzerinde bitirin.
- Serbest Biçim (O): Alanı Yeniden Şekillendir, dairesinin içindeki bütün noktaları fareyle birlikte iter. İki Yol Süpürgesi, üzerinden geçtiğiniz yerde konturu Özellikler'deki Hız kadar kalınlaştırır veya inceltir; bu işlemden geçen yol çok sayıda kısa düz parçaya dönüşür.
- Sıcak Nokta (J): Daire Sıcak Nokta ortasından dışa doğru, Çokgen Sıcak Nokta ise Çokgen Kement gibi köşe köşe çizilir. Dilim (K): Çokgen Dilim de aynı biçimde çizilir. Dışa aktarılan görüntü eşleminde daire ve çokgen olarak yazılırlar.
- Dikdörtgen (U): Elips ve ortasından dışa doğru çizilen Çokgen; kenar sayısı ve yıldız seçeneği Özellikler'dedir.
- Boya Kovası (G): Degrade, seçili nesneyi sürüklemeye başladığınız yerden bıraktığınız yere uzanan bir degradeyle doldurur.

## Degrade ve desen dolguları

Bir vektör nesnesi seçiliyken Özellikler'de dolgunun yanındaki liste, nesnenin neyle doldurulacağını belirler: tek renk için Düz, on iki degradeden biri (Doğrusal, Radyal, Elips, Dikdörtgen, Koni, Kontur, Saten, Yıldız Patlaması, Kıvrımlar, Çubuklar, Dalgacıklar, Dalgalar) veya Desen. Yeni degrade, nesnenin dolgu renginden beyaza uzanır.

İşaretçi ile degradenin tuvalde iki tutamacı görünür: başladığı yerde yuvarlak, bittiği yerde kare. Degradeyi taşımak, döndürmek, uzatmak veya kısaltmak için bunları sürükleyin; nesnenin dışında da durabilirler. Kontur, nesnenin dış hattını izler ve tutamacı yoktur.

Elips ve Dikdörtgen degradesinin, genişliğini belirleyen üçüncü, daha küçük bir tutamacı vardır. Web Renk Taklidi iki rengi piksel piksel sırayla doldurur. Metin de her şekil gibi degrade veya desenle doldurulabilir.

Dolgu listesinin altındaki Kenar, dolgunun kenarının nasıl çizileceğini belirler: Sert, Kenar Yumuşatma veya kaç piksel boyunca söneceği yazılan Geçiş Yumuşatma. Doku, desenlerden birini yüzdesi kadar güçlü biçimde dolgunun içine işler.

Düzenle, degradenin renklerini açar: renk durağı eklemek için degradenin altına tıklayın, taşımak için durağı sürükleyin, kaldırmak için uzağa çekin; seçili durağın rengini ve konumunu ayarlayın. Opaklık durakları degradenin üstünde aynı biçimde çalışır: her biri degradenin bulunduğu yerde ne kadar opak olacağını belirler. Hazır Ayarlar, degradeyi hazır bir degradeyle değiştirir. Desende Düzenle, deseni ve ölçeğini seçtirir. Stiller, Nitelikleri Yapıştır ve dosyalar dolguyu korur.

## Görüntü Önizleme

Dosya > Görüntü Önizleme (Ctrl+Shift+X), dışa aktarma ayarlarını verdikleri görüntünün yanında gösterir; biçimi, dosya boyutu ve ölçüleri altında yazar, ayarları değiştirdikçe görüntü de değişir. Dışa Aktar görüntüyü kaydeder; Tamam dışa aktarmadan ayarları korur, İptal eski haline döndürür.

## Yazdırma

Dosya > Sayfa Yapısı kâğıdı, yönünü, kenar boşluklarını ve sayfanın kâğıda sığacak şekilde ölçeklenip ölçeklenmeyeceğini belirler. Dosya > Yazdır (Ctrl+P) sayfayı bilgisayarda kurulu bir yazıcıya gönderir veya PDF dosyası olarak kaydeder. Sayfa, dışa aktarıldığı gibi, dilimler ve sıcak noktalar olmadan, inç başına 72 pikselle yazdırılır; kâğıttan büyük sayfa, Sayfaya sığdır açık değilse birkaç kâğıda yazdırılır. Yazdırma aralığı görünen sayfayı, belgenin bütün sayfalarını veya bir sayfa aralığını seçer. Yazdırma için sistemin yazdırma hizmeti (CUPS) gerekir.

## Gruplar

Değiştir > Grupla (Ctrl+G) seçili nesnelerin tek nesne gibi taşınıp ölçeklenmesini sağlar. Grubu çözmeden bir üyeyi değiştirmek için Alt Seçim aracıyla üzerine tıklayın veya seçili grubun bütün üyelerini seçmek için Seç > Alt Seç'i kullanın. Üye artık tek başına taşınabilir, renklendirilebilir, yeniden biçimlendirilebilir veya silinebilir. Üyelerinden hiçbiri seçili kalmadığında grup yeniden bütün olur; Seç > Üst Seç ise bir üyeden bütün gruba geçer.

## Metin

Bir metin nesnesi seçiliyken Özellikler'de fontu ve boyutu, kalın, italik ve altı çizili düğmeleri ile satırları sola, ortaya veya sağa hizalayan düğmeler görünür. Aynıları Metin menüsünde de vardır: Font bilgisayarda kurulu fontları listeler; Boyut'ta Daha Küçük (Ctrl+Shift+,), Daha Büyük (Ctrl+Shift+.) ve bir boyut listesi; Stil'de Düz, Kalın (Ctrl+B), İtalik (Ctrl+I) ve Altı Çizili (Ctrl+U); Hizala'da Sol, Yatay Ortalanmış, Sağ, Yaslanmış (son satır dışındaki her satır boşluklarıyla kenardan kenara yayılır) ve Uzatılmış (her satır kutuyu dolduracak kadar genişletilir) bulunur. Hiçbir şey seçili değilken ayarlanırsa bir sonraki yazacağınız metne uygulanır. Kendi kalın veya italik yüzü olmayan fontta kalın, harfler kalınlaştırılarak; italik, harfler yatırılarak yapılır.

Metin > Yola Ekle (Ctrl+Shift+Y), seçili metni seçili yol veya şekil boyunca dizer; yol artık metne aittir ve onunla birlikte taşınır. Kapalı bir şeklin çevresinde metin soldan başlar ve üstten dolaşır. Özellikler'de harflerin nasıl duracağı (Metin > Metin Yönlendirmesi: Yolun Etrafında Döndür, Dikey, Dikey Eğrilt, Yatay Eğrilt) ve metni yol boyunca ileriden başlatan Metin kaydırma görünür. Yönü Ters Çevir metni öbür yöne çevirir; Metni Yoldan Ayır metni ve yolu geri verir.

Metin > Yazım Denetimi (Shift+F7), seçili metinleri veya sayfadaki bütün metinleri gözden geçirir ve sözlükte olmayan her sözcükte durur. Değiştir, alandaki sözcüğü onun yerine koyar; Tümünü Değiştir bunu her yerde yapar. Yoksay yalnızca bu yeri, Tümünü Yoksay sözcüğün tamamını geçer; Kişisel Sözlüğe Ekle sözcüğü bundan sonra doğru sayar. Denetim arayüzün dilinde, sistemde kurulu yazım sözlükleriyle yapılır (`enchant-2` veya `hunspell` programı ve `hunspell-tr` gibi bir sözlük).

## Kılavuzlar

Sayfaya kılavuz koymak için üstteki veya yandaki cetvelden sürükleyin. İşaretçi ile bir kılavuzu sürükleyerek taşıyın; kaldırmak için cetvelin üzerine geri sürükleyin. Taşınan, çizilen veya yeniden boyutlandırılan nesneler kılavuzlara bitişir. Konumunu yazmak için kılavuza çift tıklayın. Kalem ve Alt Seçim aracıyla sürüklenen noktalar da kılavuzlara bitişir. Görünüm > Kılavuzlar onları gösterir veya gizler (Ctrl+;), kilitler, bitiştirmeyi kapatır ve hepsini temizler; oradaki Kılavuzları Düzenle renklerini de belirler.

Akıllı kılavuzlar bir nesneyi taşırken belirir: nesnenin bir kenarı veya ortası başka bir nesneyle ya da sayfayla hizalandığında bir çizgi görünür ve nesne oraya bitişir. Görünüm > Akıllı Kılavuzlar çizgileri veya bitiştirmeyi kapatır. Görünüm > Dilim Kılavuzları, dilimlerin kenarları boyunca uzanan kırmızı çizgileri gösterir veya gizler.

## Web alanları

Sıcak Nokta (J) ve Dilim (K), çizimin üzerine alanlar çizer. Düzenle > Ekle > Sıcak Nokta (Ctrl+Shift+U) ve Dikdörtgen Dilim, seçili nesnenin üzerine bir tane koyar; birden fazla nesne seçiliyse hepsi için bir tane mi yoksa her biri için ayrı mı oluşturulacağı sorulur. Bunları İşaretçi ile taşıyıp yeniden boyutlandırın; sıcak noktanın bağlantısını Özellikler panelinden ayarlayın. Gizlemek veya göstermek için 2'ye basın.

Bu alanlar düzleştirilmiş görüntüde hiçbir zaman görünmez. `ad.png` dışa aktarılırken dilimler için `ad_slice1.png`, `ad_slice2.png` ve devamı, sıcak noktalar için de görüntü haritası içeren `ad_map.html` yazılır.

## Paneller

Pencere menüsü her paneli açıp kapatır; F4 hepsini gizler veya geri getirir. Bir sekmeyi öne getirmek için tıklayın, grubunu daraltmak için öndeki sekmeye çift tıklayın.

- **Katmanlar**: üstteki liste seçili nesnelerin karışım modunu, yani renklerinin alttaki çizimle nasıl karışacağını belirler; Opaklık ise alttakinin ne kadar görüneceğini. Aynı liste Özellikler panelinde de vardır. Karışım modu kullanılan sayfa, dışa aktarılacağı gibi piksel piksel gösterilir.
- **Durumlar**: her durumun kendi nesneleri vardır. Animasyonu oynatın, soğan zarını açın ve Katmanlar panelinden bir katmanı durumlar arasında paylaşın. Yeniden adlandırmak için bir sayfaya, duruma, katmana veya nesneye çift tıklayın.
- **En İyileştir**: GIF, Animasyonlu GIF, JPEG, PNG 8, PNG 24 veya PNG 32 seçin. Panel dosya boyutunu tahmin eder; dışa aktarma bu ayarları kullanır. JPEG için Düzgünleştirme (0-8), dosyayı küçültmek üzere görüntüyü biraz bulanıklaştırır. Seçmeli kalite, sayfanın bir bölümünü kendi kalitesinde tutar: pikselleri seçin, Değiştir > Seçmeli JPEG > Seçimden JPEG Maskesi Oluştur komutunu verin ve Seçmeli kalite kutusunu işaretleyin. Maske pembe bir renklendirmeyle görünür. Ayarlar penceresinden tüm metnin ve tüm düğmelerin seçmeli kalitede tutulması istenebilir, renklendirmenin rengi de değiştirilebilir. Bir sembol, Sembol Özellikleri'ndeki Tür alanı Düğme ise düğmedir; Ortak Kitaplık'taki düğmeler böyle gelir.
- **Stiller**: uygulamak veya sonraki şeklin renklerini ayarlamak için bir stile tıklayın. Stiller dolgu, kontur ve opaklık bilgisini tutar.
- **Yol**: Topla, Kesiştir, Zımbala, Kırp, Birleştir, Böl, Basitleştir, Konturu Genişlet, Yolu Daralt ve Yolu Genişlet.
- **Otomatik Şekiller**: bir kutucuğu tuvale sürükleyin veya şekli sayfanın ortasına yerleştirmek için tıklayın; ardından Otomatik Şekil Özellikleri panelinde ayarlayın.
- **Belge Kitaplığı**: Yeni, seçimden bir sembol oluşturur. Ekle sembolü yerleştirir, Güncelle onu seçimden yeniden tanımlar, Sil kaldırır.
- **Ortak Kitaplık**: bir öğeyi belgeye sembol olarak eklemek için tuvale sürükleyin veya tıklayın. Belge Kitaplığı'ndaki semboller de tuvale sürüklenebilir.
- **Bul**: bu sayfada veya tüm sayfalarda metin ya da renk değiştirin.
- **Renk Paleti**: bir renkle doldurmak için tıklayın, kontur için Shift+tıklayın. Geçerli dolgu için tamamlayıcı, benzeşik ve üçlü renkler de önerir.
- **Sembol Özellikleri**: seçili sembolü yeniden adlandırın, kaç kopyası olduğunu görün ve sembolü ayırın.

Her sekme satırının sağındaki panel menüsü o grubu daraltır veya kapatır. Üst çubuktaki Genişletilmiş Mod, açık paneller ile yalnızca sekmeler arasında geçiş yapar.

## Dışa aktarmaları karşılaştırma

Önizleme, sayfayı dışa aktarılacağı haliyle gösterir. 2 Yukarı görünümü orijinal sayfayı yanına koyar; 4 Yukarı görünümü orijinal sayfayı üç dışa aktarma ayarı ve dosya boyutlarıyla karşılaştırır. Bu görünümlerde kaydırmak için sürükleyin.

## Tercihler ve Klavye Kısayolları

**Düzenle > Tercihler** (Ctrl+U) geri alma adımı sayısını, işaretçinin bir nesneyi ne kadar yakından seçtiğini, nesnelerin ne kadar yakından bitiştiğini, ızgaranın boyutunu ve rengini ve kılavuzların rengini ayarlar. Metin seçiliyken Ctrl+U altını çizer. **Görünüm > Izgara > Izgaraya Bitiştir** (Ctrl+Alt+Shift+G) ızgara görünürken taşınan nesneleri ızgara çizgilerine bitiştirir.

**Düzenle > Klavye Kısayolları** bütün menü komutlarını tuşlarıyla listeler. Bir komutu seçip **Değiştir**'e, sonra yeni tuşlara basın. Tuşlar başka bir komuttaysa o komuttan alınır. **Kaldır** komutu tuşsuz bırakır, **Sıfırla** ilk tuşlarına döndürür. Değişiklikler ayar dosyasında saklanır.

## Semboller, düğmeler ve animasyon

Bir sembol kopyasına **çift tıklayınca** sembol olduğu yerde açılır: parçaları sayfada serbest kalır, sayfanın geri kalanı kilitlenir ve bu sırada çizilen her şey sembole katılır. Boş bir yere çift tıklamak ya da belgenin üstündeki **◀ Sayfa** düğmesi düzenlemeyi bitirir; sembolün bütün kopyaları değişir. Aynısı **Değiştir > Sembol > Sembolü Düzenle** ile de yapılır.

**Düzenle > Ekle > Yeni Sembol** (Ctrl+F8) adını ve türünü (Grafik, Animasyon, Düğme) sorar, sonra boş sembolü çizmeniz için açar. **Yeni Düğme** (Ctrl+Shift+F8) doğrudan bir düğme sembolü açar. Bir düğmenin dört durumu vardır: Yukarıda, Üzerinde, Aşağıda, Aşağıdayken Üzerinde. Düzenlerken üstteki listeden durumu seçin; boş bir durumda **… Grafiğini Kopyala** bir önceki durumu getirir. Sayfadaki düğme seçiliyken Özellikler'deki **Bağlantı** alanı tıklanınca gidilecek adresi alır.

**Önizleme** görünümünde fareyi düğmenin üzerine getirince Üzerinde, basınca Aşağıda durumu görünür. Dışa aktarırken her düğme için durum görüntüleri ve bunları fareyle değiştiren bir HTML sayfası (`ad.html`) yazılır.

**Değiştir > Animasyon > Seçimi Canlandır** (Alt+Shift+F8) seçimi bir animasyon sembolüne çevirir: durum sayısı, taşıma mesafesi ve yönü (0° sağ, 90° yukarı), ölçek, opaklık ve dönüş sorulur; gerekirse yeni durumlar eklenir. Seçiliyken yolu görünür (yeşil başlangıç, kırmızı bitiş); kırmızı ucu sürüklemek mesafeyi ve yönü, yeşil ucu sürüklemek başlangıç yerini değiştirir; kopyalardan birini taşımak bütün animasyonu taşır. **Ayarlar** aynı pencereyi yeniden açar, **Animasyonu Kaldır** yalnızca ilk durumdaki kopyayı bırakır.

## Şablonlar, toplu işleme ve komutlar

**Dosya > Şablondan Yeni** hazır sayfa boyutlarını, kılavuzlarla işaretlenmiş ızgara sistemlerini, telefon ve tablet ekranlarını ve kendi şablonlarınızı listeler; **Gözat** herhangi bir belgeyi şablon olarak açar. Şablondan açılan belgenin dosyası yoktur, kaydederken ad sorulur. **Dosya > Şablon Olarak Kaydet** belgeyi şablonlarınıza ekler.

**Dosya > Toplu İşleme** birçok dosyayı tek seferde ölçekler, dönüştürür ve yeniden adlandırır: dosyaları ekleyin, ölçeği seçin (boyuta, alana sığacak şekilde, yüzdeye), isterseniz bu belgenin En İyileştir ayarlarını kullanın, önek ya da sonek verin ve çıktı klasörünü seçin. Bir sonuç var olan bir dosyanın yerini alacaksa eskisi önce `Original Files` klasörüne kopyalanır.

**Komutlar** menüsünde hazır komutlar vardır: Yaratıcı (Ok Başları Ekle, Resim Çerçevesi Ekle, Gölge Ekle, Otomatik Vektör Maskesi, Gri Tonlamaya Dönüştür, Sepya Tonuna Dönüştür, Bük ve Soldur) ve Belge (Katmanlara Dağıt, Diğer Katmanları Gizle, Diğer Katmanları Kilitle, Tüm Durumları Ters Çevir, Durum Aralığını Ters Çevir). **Geçmiş** panelindeki **Adımlardan Komut Oluştur**, belgeyi değiştiren menü komutlarını bir ad altında saklar; kaydedilen komutlar menünün altında görünür ve **Kaydedilmiş Komutları Yönet** ile yeniden adlandırılır ya da silinir. Fareyle yapılan adımlar (çizim, sürükleme) kaydedilmez. **Komut Dosyasını Çalıştır** böyle bir dosyayı (her satırda `Menü > Komut`) çalıştırır; Fireworks'ün JSF dosyalarını çalıştırmaz.

## Metin yönü

Metin seçiliyken Özellikler'deki yön listesi dört seçenek sunar: **Soldan sağa yatay**, **Sağdan sola yatay** (satırın harfleri ters sırada dizilir), **Soldan sağa dikey** ve **Sağdan sola dikey**. Dikey metinde her satır bir sütundur ve harfler dik durarak alt alta dizilir. Hizalama düğmeleri dikey metinde sütun boyunca çalışır; **Metin > Hizala** menüsündeki Üst, Dikey Ortalanmış, Alt, Dikey Yaslanmış ve Dikey Uzatılmış aynı işi görür.

## Fireworks PNG dosyaları

Fireworks'ün kendi PNG dosyaları **Dosya > Aç** ile katmanlarıyla açılır: katman adları ve görünürlükleri, durumlar, bitmap nesneleri ve Alt Gölge efekti okunur. Dosyada yol, metin ya da grup varsa bunlar okunamaz; o zaman dosya düz görüntü olarak açılır ve durum çubuğu bunu söyler. Açılan belge yeni bir belgedir: kaydetmek Fireworks dosyasının üzerine yazmaz.

## Örnekler

Menü çubuğunun sağ ucundaki **Örnekler** menüsü, Fireworks'te bulunmayan ve PisiWorks'e özgü olan özellikler içindir; soldaki menüler Fireworks kullanıcısının bildiği gibi kalır. Menüden bir örnek seçince yeni, adsız bir belge olarak açılır; istediğiniz gibi değiştirebilir, kaydedebilir ya da dışa aktarabilirsiniz. Örneklerdeki yazılar arayüzün dilinde gelir.

| Örnek | Gösterdiği |
|---|---|
| Web afişi | Degrade dolgu, metin, canlı filtre ve Otomatik Şekil |
| Gezinme düğmeleri | Üç durumlu düğme sembolleri ve bağlantılar; Önizleme'de fareyle deneyin, dışa aktarınca HTML sayfası da yazılır |
| Uçan yıldız | Sekiz duruma yayılan animasyon sembolü; Durumlar panelinden oynatın, animasyonlu GIF olarak dışa aktarın |
| Okul afişi | Yola eklenmiş metin, maske ve canlı filtreler |
| Dolgular ve degradeler | On iki degrade türü yan yana |
| Canlı filtreler | Aynı şekil sekiz filtreyle |
| Web sayfası taslağı | 12 sütunlu kılavuz ızgarası, katmanlar, dilimler ve sıcak noktalar |

## Animasyonlu GIF

- **Açma.** Animasyonlu bir GIF **Dosya > Aç** ile açılınca her karesi bir durum olur; kare süreleri Durumlar panelinde görünür. Açılan belge yenidir, GIF dosyasının üzerine yazılmaz.
- **Dışa aktarma ve döngü.** En İyileştir panelinde **Animasyonlu GIF** seçin. Aynı satırdaki **Döngü** listesi animasyonun kaç kez oynayacağını belirler: Sonsuz, Döngü Yok (bir kez) ya da belirli bir sayı.
- **Ara kareler.** Aynı sembolün iki ya da daha çok kopyasını seçip **Değiştir > Sembol > Örneklerin Arasını Doldur** (Ctrl+Alt+Shift+T) deyin. Adım sayısı kadar ara kopya üretilir; konum, boyut, dönüş ve opaklık adım adım değişir. **Durumlara dağıt** işaretliyse her kopya kendi durumuna yerleşir ve animasyon hazır olur.
- **GIF Oluştur.** Menü çubuğunun sağındaki **GIF > GIF Oluştur** resim dosyalarından animasyon yapar: dosyaları ekleyin (ada göre sıralanır), kare süresini ve döngüyü seçin. **GIF Kaydet** doğrudan dosyaya yazar; **Belge Olarak Aç** kareleri durumlara dağıtır ve üzerinde çalışmaya devam edersiniz. Farklı boyuttaki resimler en büyüğünün boyutundaki tuvalin ortasına yerleşir.

## PisiAnimasyon ile animasyon

PisiAnimasyon, PisiWorks'ün bütün çizim araçlarını kullanır; farkı, sahnenin üstündeki zaman çizelgesidir. Aşağıdaki terimler bizim çevirimizdir; ders notlarınızdaki sözcükler farklı olabilir.

**Zaman çizelgesi.** Her katmanın bir kare satırı vardır. Bir kareye tıklayınca o kare gösterilir; numaraların üzerinde sürükleyerek kareler arasında gezinirsiniz. Katman adının yanındaki göz katmanı gizler, kilit kilitler.

| Tuş | İş |
|---|---|
| F5 | Kare ekle (anahtar karenin süresini uzatır) |
| Shift+F5 | Kareleri kaldır |
| F6 | Anahtar kare ekle (o karedeki görüntüyle başlar) |
| F7 | Boş anahtar kare ekle |
| Shift+F6 | Anahtar kareyi temizle |
| Enter | Sona kadar bir kez oynat ya da durdur |
| . ve , | Bir kare ileri, bir kare geri |
| Ctrl+Enter | Filmi Test Et |

**Oynatma.** Enter, filmi o anki kareden başlayarak bir kez oynatır ve son karede durur; Flash'ta da böyledir. Son karedeyken basılırsa baştan başlar. Filmin dönüp durması için zaman çizelgesinin altındaki **Döngü** kutusunu işaretleyin ya da **Kontrol > Döngülü Oynat**'ı seçin. Bu yalnızca program içindeki oynatma içindir: yayımlanan film, son anahtar karesinde **Dur** eylemi yoksa, son sahneden sonra hep baştan başlar.

**Uzun filmler.** Cetveldeki mavi işaretler tam saniyelerdir (saniyede 24 karede 20. saniye 480. karedir). Zaman çizelgesinin altındaki **Kare** kutusu o anki kareyi, yanındaki kutu saniye olarak zamanını gösterir; birine sayı yazıp Enter'a basınca doğrudan oraya gidilir. Zaman çizelgesi fare tekerleğiyle yana kayar, film oynarken de oynatma kafasını izler.

**Her katmanın kendi zamanı vardır.** Bir katmanın anahtar kareleri yalnız ona aittir; bu yüzden şekiller ayrı zamanlarda hareket edebilir:

- *Hareket edip durmak.* Hareket ara doldurmasını 1. kare ile, diyelim, 480. kare arasında yapın. Sonra aynı katmanın daha ilerideki bir karesine (960. kare) tıklayıp F5'e basın: son anahtar kare oraya kadar sürer, şekil vardığı yerde kalır.
- *Geç başlamak.* Yeni bir katman ekleyin, o katmanda 480. kareye tıklayıp F7'ye (boş anahtar kare) basın, şekli orada çizin. Katman o kareye kadar hiçbir şey göstermez. Daha ileriye kendi anahtar karesini (F6) ve ara doldurmasını ekleyin.
- *Erken çıkmak.* Şeklin kaybolacağı karede F7'ye basın: o boş anahtar kareden sonra katman boştur.

**Sırayla** örneği bunların hepsini sekiz saniyede gösterir: araba dört saniye gidip durur, sonra güneş doğar, sonra yazı belirir.

**Davranışlar: anahtar karesiz hareket.** Bu PisiAnimasyon'un kendi özelliğidir; Flash'ta karşılığı yoktur. Bir nesneyi seçin ve menü çubuğunun sağındaki **Davranışlar** menüsünden (ya da sağ tık menüsünden) bir hareket seçin: nesne bundan sonra her karede o hareketi yapar; anahtar kare de ara doldurma da gerekmez. Aynı davranışı yeniden seçmek onu kaldırır. Bir nesnenin aynı anda birkaç davranışı olabilir.

| Davranış | Nesne ne yapar | Sayıları |
|---|---|---|
| Dön | Dönüştürme noktasının çevresinde döner durur | Bir tur kaç saniye (küçük sayı hızlı döner: 0.5 saniyede iki tur demektir); saat yönünün tersine (1: evet, 0: hayır) |
| Zıpla | Yukarı sıçrar, iner; yeniden | Yükseklik, bir zıplama kaç saniye |
| Git Gel | Sağa gider, geri gelir (eksi uzaklık sola gider) | Uzaklık, gidiş dönüş kaç saniye |
| İn Çık | Yükselir, alçalır | Uzaklık, gidiş dönüş kaç saniye |
| Sürekli Git | Durmadan gider; sahnenin bir kenarından çıkınca karşı kenarından girer | Hız, yön (0 sağ, 90 yukarı) |
| Titre | Titrer | Güç |
| Büyü Küçül | Büyür, eski boyuna döner | Büyüme yüzdesi, bir atım kaç saniye |
| Yanıp Sön | Görünür, kaybolur | Bir yanıp sönme kaç saniye |
| Belir Kaybol | Yavaşça silinir, geri gelir | Gidiş dönüş kaç saniye |
| Bir Süre Sonra Kaybol | Süresi dolunca yok olur: kıvılcım ve patlamalar için | Saniye |
| Fareyi İzle | Farenin peşinden gider | Hız (0: farenin üstünde durur) |
| Ok Tuşlarıyla Yürü | İzleyen ok tuşlarıyla hareket ettirir; sahnenin dışına çıkmaz | Hız |
| Düş ve Zıpla | Sahnenin alt kenarına düşer; Yukarı ok ya da Boşluk zıplatır | Çekim, zıplama hızı (0: zıplamaz) |
| Sürüklenebilir | İzleyen fareyle sürükler | |
| Kenarlardan Sek | Dümdüz uçar; sahnenin her kenarından ve katı olan her şeyden top gibi geri seker | Hız, yön (derece; 0 sağ, 90 yukarı) |
| Teker Gibi Yuvarlan | Gittiği yol kadar döner: hızlı giderse hızlı, yavaşlarsa yavaş, durursa durur, geri giderse ters | |
| Kamera Beni İzlesin | Görüntü onunla birlikte gider: nesne ekranda çizildiği yerde kalır, geri kalan her şey yanından kayar | |
| Arka Plan Gibi Kay | Görüntü kaydıkça yalnızca bir kısmı kadar kayar, uzaktaki şeyler gibi; ekrandan çıkınca öbür yandan geri gelebilir | Kayma yüzdesi (100 yol gibi, 0 ekranda sabit), öbür yandan geri gelir (1: evet, 0: hayır) |
| Katı | Yürüyen ya da düşen hiçbir şey içinden geçemez: yanında durur, üstünde ayakta kalır, altına başını çarpar | |

İlk onu program içinde oynatırken (Enter) görünür: film o zaman son karesinde bekleyerek, siz durdurana kadar oynar. Son dokuzu izleyenle, sahnenin kenarlarıyla, nesnenin gittiği yolla ya da kamerayla ilgilidir, bu yüzden yayımlanan filmde çalışır: **Filmi Test Et** (Ctrl+Enter) ile görün. **Davranışlar > Davranış Ayarları**, seçili nesnenin davranışlarını sayılarıyla ve her biri için bir Kaldır düğmesiyle listeler. Davranışlar ara doldurmayla birlikte kullanılabilir: ara doldurması olan nesne ikisini de yapar. Davranışlar sahnedeki nesnelerde çalışır, bir sembolün ya da grubun içindekilerde çalışmaz; davranışı sembolün kopyasına ya da grubun kendisine verin.

*Akıcılık.* Yayımlanan filmde davranışların ve kuralların hareket ettirdiği her şey, filmin kare hızı ne olursa olsun, ekranın tazeleme hızında (çoğu ekranda saniyede 60 kez) çizilir; bu yüzden yürüyen, düşen, kayan nesneler akıcıdır. Anahtar karelerle ve ara doldurmayla yapılan çizimler ise filmin kendi kare hızında değişir: onları akıcılaştırmak için kare hızını (zaman çizelgesinin altındaki **kare/sn** kutusu) 30'a çıkarın. Program içinde Enter ile oynatırken her şey filmin kare hızındadır.

*İki dakikada ilk oyun:* bir kutu çizin, ona **Ok Tuşlarıyla Yürü** ve **Düş ve Zıpla** verin; bir yıldız çizip ona **Dön** verin; Ctrl+Enter'a basın. **Davranışlar** örneğinde bütün türler tek karededir.

**Kurallar: filmden oyuna.** Bu da PisiAnimasyon'un kendi özelliğidir. Bir kural *şu olunca şunu yap* der; iki yarısı da listeden seçilir, sayılardan başka hiçbir şey yazılmaz. **Kurallar > Kurallar ve Sayılar** her kuralı bir cümle olarak gösterir; birine tıklayınca parçalarına ayrılır.

- *Önce adlar.* Kural bir nesneyi adıyla tanır. Ad, nesne seçiliyken **Özellikler** panelinin solundaki **Ad** kutusuna yazılır (Enter ile alınır); çizilen her şekil için aynıdır: otomatik şekil, elle çizim, grup. Bir sembolün sahnedeki kopyası seçiliyken açılan **Sembol Özellikleri** panelinde de aynı **Ad** kutusu vardır; orada yazan ad kopyanın adıdır, **Sembol** kutusundaki ise kitaplıktaki sembolün adıdır ve kurallar ona bakmaz. Yeni çizilen şekil türünün adını taşır ("Elips", "Çark"); kural yazmadan önce ona kendi adını verin. Elmaya "Elma", sepete "Sepet" adını verin. Aynı adı taşıyan nesneler kurallarını paylaşır: "Elma" hakkındaki bir kural bütün elmalar için geçerlidir.
- *Yeni Kural* olayları listeler:

| Olay | Ne zaman olur |
|---|---|
| Film başlayınca | Bir kez, başta ve "baştan başla"dan sonra |
| Bir sahne başlayınca | Başta ve film başka bir sahneye her geçtiğinde |
| Bir tuşa basılınca | Boşluk, oklar, Enter, bir harf ya da rakam |
| Bir nesneye tıklanınca | İzleyen o addaki bir nesneye tıklayınca |
| Bir nesne ötekine değince | İkisinin kutuları üst üste gelince |
| Bir nesne sahnenin kenarına değince | Herhangi bir kenar ya da alt, üst, sol, sağ kenar |
| Her şu kadar saniyede bir | Tekrar tekrar |
| Şu kadar saniye sonra | Bir kez |
| Bir sayı bir değere gelince | O değer olunca, onu geçince ya da altına inince |

- *Eylem Ekle* yapılacakları listeler: sayıyı değiştir, ayarla ya da rastgele yap; nesneyi gizle, göster, taşı, bir konuma, rastgele bir yere ya da üstte rastgele bir yere koy; bir nesnenin kopyasını başka bir nesnenin yerinde oluştur; nesneyi sil; nesneyi bir an yanıp söndür; nesneyi hızlandır ya da yavaşlat; nesneyi zıplat; nesneyi derece kadar döndür ya da sürekli döndür, boyutunu değiştir, saydamlaştır; nesneye bir davranış ver ya da davranışını al; bir nesneyi ötekinin peşine tak; nesneyi yola çıkar, yavaşlayarak durdur ya da hemen durdur; bir nesneyi ötekine bağla ya da serbest bırak; kamera bir nesneyi izlesin ya da yerinde dursun; ses çal; sonraki sahneye, bir sahneye ya da bir kareye git; filmi durdur ya da oynat; her şeyi durdur; baştan başla. Bir kuralın birkaç eylemi olabilir; sırayla yapılır.
- *Koşul.* Olayın altındaki **Koşul**, bir sayıyı bir değerle karşılaştırır: "Kahraman nesnesi Kapı nesnesine değince, Puan sayısı 2 değerinden büyükse". Kural ancak bu da doğruysa izlenir.
- *Kopyalar.* **Nesnenin kopyasını oluştur**, aynı adda ve aynı davranışlarda bir nesneyi ikinci bir nesnenin olduğu yerde oluşturur: geminin olduğu yerde bir mermi. Tek bir mermi çizin ve film başlayınca gizleyin; her atış onun bir kopyasıdır. **Nesneyi sil** bir kopyayı tümüyle kaldırır; başka bir nesneyi film baştan başlayana kadar gizler.
- *Her birine bir kopya.* İkinci ad birkaç nesnenin adıysa her biri bir kopya alır: "Her 2 saniyede bir: Ateş nesnesinin kopyasını Fare nesnesinin yerinde oluştur" kuralı bütün farelere ateş ettirir. Geçit, kopyaların belireceği yer olarak sahnenin hemen dışına çizilen bir nesnedir: "Her 3 saniyede bir: Fare nesnesinin kopyasını Geçit nesnesinin yerinde oluştur".
- *Yanıp sönme.* **Nesneyi bir an yanıp söndür**, nesnenin vurulduğunu gösterir. Yanıp sönerken ona hiçbir şey değemez; bu, oyuncuya kaçacak bir an verir.
- *Hızlandırma.* **Nesneyi hızlandır**, nesnenin kendiliğinden yaptığı her şeyi çabuklaştırır: düşmeyi, gitmeyi, dönmeyi. Oyunun gittikçe zorlaşması böyle sağlanır.
- *Değme.* İki nesne, kutuları buluşunca değmiş olur; döndürülmemiş bir elips kutusuyla değil, kendi çizgisiyle hesaba katılır. Katı bir nesnenin üstünde duran şey ona değiyordur.
- *Hangi nesne?* "Elma nesnesi Sepet nesnesine değince: Elma nesnesini gizle" kuralında gizlenen, sepete değen elmadır; bütün elmalar değil.
- *Sayılar.* **Yeni Sayı** adı ve başlangıç değeri olan bir sayı oluşturur: puan, can, süre. İçinde süslü ayraçla adı geçen bir yazı onu filmde gösterir: Metin aracıyla `Puan: {Puan}` yazın.
- *Bir şey bir kez olur.* Kural bir şey olduğu anda izlenir, sürdüğü sürece değil: yerde duran elma yere bir kez değmiştir.
- Kurallar yayımlanan filmde çalışır: **Filmi Test Et** (Ctrl+Enter). Oyuncuyu bekleyecek sahnenin kare eylemi **Durdur** olmalıdır; yoksa film sonraki sahneye geçer.

**Ders: elma yakala** (**Oyun: elma yakala** örneği).

1. Sahnenin altına bir sepet çizin, adını "Sepet" yapın. Ona **Ok Tuşlarıyla Yürü** ve zıplama hızı 0 olan **Düş ve Zıpla** verin: düşmek onu yerde tutar.
2. Üste bir elma çizin, adını "Elma" yapın. Ona çekimi küçük (70), zıplama hızı 0 olan **Düş ve Zıpla** verin. İkinci elma için kopyalayıp daha yukarı koyun.
3. **Kurallar ve Sayılar**'da iki sayı oluşturun: başlangıcı 0 olan Puan ve başlangıcı 5 olan Can. Sahneye `Puan: {Puan}` ve `Can: {Can}` yazın.
4. Kural: *Bir nesne ötekine değince*, Elma, Sepet. Eylemler: *Sayıyı değiştir*, Puan, 1; *Nesneyi üstte rastgele bir yere koy*, Elma.
5. Kural: *Bir nesne sahnenin kenarına değince*, Elma, alt kenarına. Eylemler: *Sayıyı değiştir*, Can, −1 (cümle "Can sayısını 1 azalt" diye okunur); *Nesneyi üstte rastgele bir yere koy*, Elma.
6. "Oyun bitti" yazan ikinci bir sahne ekleyin; içine "Yeniden" adlı bir kutu koyun. İki sahnenin de kare eylemini **Durdur** yapın.
7. Kural: *Bir sayı bir değere gelince*, Can, altına inince, 1. Eylem: *Sahneye git*, 2.
8. Kural: *Bir nesneye tıklanınca*, Yeniden. Eylem: *Baştan başla*.
9. Ctrl+Enter.

**Ders: hedef vurma.** "Hedef" adlı üç daire; her birinin kendi davranışı var (Git Gel, İn Çık, Sürekli Git). Sayılar: Puan 0 ve Süre 20. Kurallar: *Hedef nesnesine tıklanınca: Puan sayısını 1 artır; Hedef nesnesini rastgele bir yere koy*. *Her 1 saniyede bir: Süre sayısını 1 azalt*. *Süre sayısı 1 değerinin altına inince: 2. sahneye git*.

**Ders: bilgi yarışması.** Her soru için bir sahne; her birinin kare eylemi Durdur. Yanıtlar üzerinde yazı olan kutulardır; doğru olanın adını "Doğru", ötekilerin adını "Yanlış" yapın. Kaç soru olursa olsun üç kural yeter: *Doğru nesnesine tıklanınca: Puan sayısını 1 artır; sonraki sahneye git*. *Yanlış nesnesine tıklanınca: sonraki sahneye git*. *Yeniden nesnesine tıklanınca: baştan başla*. Son sahne `Puan: {Puan}` gösterir.

**Ders: platform oyunu** (**Oyun: platform** örneği).

1. Zemini ve üç platformu birer dikdörtgen olarak çizin; her birine **Katı** verin.
2. Kahramanı çizin, adını "Kahraman" yapın, ona **Ok Tuşlarıyla Yürü** ve **Düş ve Zıpla** verin. Deneyin: yürür, zıplar, platformlara iner, altlarına başını çarpar.
3. Her platformun üstüne bir altın koyun; hepsinin adı "Altın" olsun. Kural: *Kahraman nesnesi Altın nesnesine değince: Puan sayısını 1 artır; Altın nesnesini sil*.
4. Zemine bir düşman çizin, adını "Düşman" yapın, ona **Git Gel** verin. Kural: *Kahraman nesnesi Düşman nesnesine değince: Can sayısını 1 azalt; Kahraman nesnesini 30, 322 konumuna koy* (kahramanın başladığı yer; X ve Y değerleri Özellikler'de yazar).
5. Bir kapı çizin, adını "Kapı" yapın. Kural: *Kahraman nesnesi Kapı nesnesine değince*, **Koşul** Puan, büyükse, 2: *2. sahneye git*.
6. Kural: *Can sayısı 1 değerinin altına inince: 3. sahneye git*. 2. sahnede "Aferin", 3. sahnede "Oyun bitti" yazar; ikisinde de "Yeniden" adlı bir kutu ve Durdur kare eylemi vardır.

**Ders: uzay atışı** (**Oyun: uzay atışı** örneği).

1. Gemiyi alta çizin, adını "Gemi" yapın, ona **Ok Tuşlarıyla Yürü** ve zıplama hızı 0 olan **Düş ve Zıpla** verin.
2. Tek bir mermi çizin, adını "Mermi" yapın, ona yönü 90 (yukarı) olan **Sürekli Git** verin. Kural: *Film başlayınca: Mermi nesnesini gizle*.
3. Kural: *Boşluk tuşuna basılınca: Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur*. Kural: *Mermi nesnesi sahnenin üst kenarına değince: Mermi nesnesini sil*.
4. Dört uzaylı çizin; hepsinin adı "Uzaylı" olsun, her biri bir öncekinden daha yukarıda dursun. Her birine çekimi küçük (22), zıplama hızı 0 olan **Düş ve Zıpla** verin.
5. Kural: *Mermi nesnesi Uzaylı nesnesine değince: Puan sayısını 1 artır; Mermi nesnesini sil; Uzaylı nesnesini üstte rastgele bir yere koy; Uzaylı nesnesini yüzde 6 hızlandır*.
6. Kural: *Uzaylı nesnesi sahnenin alt kenarına değince: Can sayısını 1 azalt; Uzaylı nesnesini üstte rastgele bir yere koy*. Kural: *Can sayısı 1 değerinin altına inince: 2. sahneye git*.

**Oyunu canlandırmak.** Örnek oyunlar derslerdeki birkaç kuralı ve dört şeyi daha kullanır. Bir oyunu oynamaya değer yapan bunlardır; her biri bir tık ya da bir cümledir:

- *Karakterler.* Karakteri birkaç şekilden çizin (gövde, gözler, şapka), hepsini seçip gruplayın (Ctrl+G) ve gruba ad verin. Grup, kurallar ve davranışlar için tek bir nesnedir. Bir örneği açıp kahramanın grubunu çözerek nasıl yapıldığına bakın.
- *Bir şey olduğunda parlama.* Bir yıldız çizin, adını "Parıltı" yapın, ona **Büyü Küçül**, **Dön** ve **Bir Süre Sonra Kaybol** (0,35 saniye) verin. Kural: *Film başlayınca: Parıltı nesnesini gizle*. Sonra altının kuralına ekleyin: *Parıltı nesnesinin kopyasını Altın nesnesinin yerinde oluştur*. Alınan her altın parlar.
- *Her şeye bir ses.* Kısa bir ses içe aktarın (**Ses İçe Aktar**) ve kurala *Ses çal* ekleyin. Örnekler kendileri için üretilmiş beş ses taşır: çın, vız, bum, eyvah ve zafer.
- *Hareket eden manzara.* **Sürekli Git** verilmiş bulutlar, **Dön** verilmiş güneş, üç ayrı hızda akan yıldızlar, **Yanıp Sön** verilmiş lambalar. Hiçbiri bir kuralda yer almaz.
- *Gittikçe zorlaşma.* Puan veren kurala *… nesnesini yüzde 5 hızlandır* ekleyin.

**Karakterler nerede, nasıl değiştirilir.** Bir örneğin karakterini değiştirmek için İşaretçi aracıyla (siyah ok) ona **çift tıklayın**. Ne olacağı karakterin ne olduğuna bağlıdır:

- *Grup.* Küçük oyunlardaki karakterlerin çoğu sahnede duran gruplardır. Çift tıklayınca grubun içine girersiniz: parçalar tek tek seçilir ve değiştirilir (kahramanın şapkasının rengi, gözlerinin boyu). Grubun yanına tıklamak onu kapatır. **Değiştir > Grubu Çöz** (Ctrl+Shift+G) grubu tümüyle parçalarına ayırır.
- *Sembol.* Pisi: Demir Pençe'nin bütün karakterleri semboldür: fare, dron, ateş, bomba, kalp, mermi, patlama, taret, Dev Fare. Çift tıklayınca sembol olduğu yerde açılır ve sahnenin üstünde hangi sembolün düzenlendiği yazar. Bitirmek için üç yoldan biri: boş bir yere çift tıklayın, Esc'ye basın ya da sahnenin üstündeki **◀** düğmesine tıklayın. Yarıda kalmış bir eğme ya da ölçekleme varsa ilk Esc onu iptal eder, ikincisi sembolden çıkarır; çift tıklama ve düğme onu uygulayıp çıkarır. Sembolde yapılan değişiklik **bütün sahnelerde** ve o karakterin bütün kopyalarında görünür; fareyi bir kez değiştirirsiniz, her bölümdeki her fare değişir. Semboller **Belge Kitaplığı** panelinde listelenir.
- *Film klibi.* Bacaklarını oynatan karakter bir film klibidir: Pisi öyledir. Çift tıklayınca klip açılır; zaman çizelgesinde koşunun dört adımı vardır, her biri ayrı bir çizimdir. Sahnenin üstündeki **◀** düğmesi geri döner.

*Sahnenin çevresi.* Sahnenin etrafındaki gri alan çalışma alanıdır ve PisiAnimasyon, Flash gibi, orada duranları gösterir. Kopyaların çıkarıldığı asıl nesneler orada tutulur: Pisi: Demir Pençe'de sol kenarın dışında alt alta dururlar; geçitler (küçük pembe yıldızlar) sağ kenarın ve üst kenarın dışındadır. Görünmüyorlarsa uzaklaştırın ya da kaydırın.

*Dikkat edilecekler.* Bir karakteri Ölçekle, Eğ ya da Deforme Et aracıyla biçimlendirirseniz değişiklik Enter'a basınca ya da çift tıklayınca uygulanır; yalnızca o sahnedeki o kopyayı etkiler. Karakterin kendisini her yerde değiştirmek için semboldeki çizimi değiştirin. Bir karakter her sahnede oyuna girmez: dron ancak Fabrika'da gelir, bu yüzden ona yapılan değişiklik ilk bölümde görünmez. Adları değiştirmeyin; kurallar adlara bakar.

**Vitrin oyunu: Pisi: Demir Pençe.** Örnekler menüsünün ilk satırı uzunca bir oyundur: silahlı robot pars Pisi bir şehirden, bir fabrikadan ve bir çatıdan geçer, sonunda dev bir fareyle karşılaşır. Robot fareler gelip ateş eder, dronlar bomba bırakır, çatıyı bir taret korur, arada bir kalp düşer. Derslerde anlatılmayan hiçbir şey kullanmaz, yalnızca daha çoğunu kullanır; parçalarına ayırıp bakmaya değer:

- *Pisi koşar.* Pisi, her biri üç kare süren dört adımlık bir film klibidir. Klibin sahnedeki kopyasının **Ok Tuşlarıyla Yürü** ve **Düş ve Zıpla** davranışları vardır: bir klibin de her nesne gibi davranışı ve adı olabilir.
- *Dünya Pisi'ye gelir.* PisiAnimasyon'da kamera yoktur. Manzaranın her katmanı sahne genişliğinde, sola doğru **Sürekli Git** verilmiş bir şerittir ve ardında ikinci bir şerit vardır; uzaktakiler yavaş, yakındakiler hızlı gider, şehir akıp geçiyormuş gibi görünür.
- *Düşmanlar bir geçitten gelir.* Sol kenarın dışına tek bir fare çizilir ve sahne başlayınca gizlenir. *Her 2.6 saniyede bir: Fare nesnesinin kopyasını Geçit nesnesinin yerinde oluştur*. *Her 2.3 saniyede bir: Ateş nesnesinin kopyasını Fare nesnesinin yerinde oluştur*. *Mermi nesnesi Fare nesnesine değince: Puan sayısını 1 artır; Patlama nesnesinin kopyasını Fare nesnesinin yerinde oluştur; Mermi nesnesini sil; Fare nesnesini sil*.
- *Bölümler.* *Puan sayısı 10 olunca: 2. sahneye git*. Her sahnede aynı adlarda nesneler vardır, bu yüzden aynı kurallar hepsinde geçerlidir; Gök geçidi olmayan sahnede dron da olmaz.
- *Dev Fare.* Zırh adlı bir sayı 30'dan başlar. *Mermi nesnesi Dev Fare nesnesine değince: Zırh sayısını 1 azalt; … Dev Fare nesnesini bir an yanıp söndür*. *Zırh sayısı 1 değerinin altına inince: 5. sahneye git*.

Yirmi sekiz kuralın hepsi **Kurallar > Kurallar ve Sayılar**'da okunabilir.

Kuralların henüz yapmadıkları: hareket eden platform üstündekini taşımaz; değme, elipsler dışında kutulara göre hesaplanır; sayılar tam ya da ondalıklıdır, sözcük saklanmaz; listelerde bir kuralın tek koşulu vardır; birden çok koşul ve "değilse" yazılı dilde vardır. Sahneden büyük bölümler, kamera ve birkaç animasyonu olan karakter kuralların değil, bir oyun stüdyosunun işidir.

**Kuralları yazmak: dil.** Kurallar ve Sayılar penceresindeki her cümle yazılabilir de. **Kurallar > Kuralları Yaz** bütün kuralları ve sayıları bir metin olarak açar; listelerle kurduğunuz oyun burada yazılı hâliyle durur, burada yazdığınız da listelerde görünür. İkisi aynı şeyin iki yüzüdür.

```
Sayılar:
    Puan = 0
    Can = 5

Elma nesnesi Sepet nesnesine değince:
    Puan sayısını 1 artır
    Elma nesnesini üstte rastgele bir yere koy

Can sayısı 1 değerinin altına inince:
    2. sahneye git
```

- *Biçim.* Bir kural, kenardan başlayıp iki noktayla biten bir olay satırı ve altında, içeriden yazılan eylemlerdir. Olay satırından sonra Enter'a basınca imleç kendiliğinden içeri girer. `Sayılar:` başlığının altına her sayı `Ad = değer` diye yazılır. `#` ile başlayan satır not sayılır, okunmaz.
- *Yazarken okunur.* Metin her tuşta yeniden okunur. Okunamayan satır kırmızı olur ve altında nedeni satır numarasıyla yazar: "Satır 7: bu, bildiğim bir eylem değil", "Satır 9: Puan adında bir sayı yok". Nedene tıklayınca imleç o satıra gider. Belgede olmayan bir nesne ya da ses adı uyarı olarak gösterilir; metin yine de alınır.
- *Cümle ekle.* Cümleleri ezberlemek gerekmez: üstteki **Bir olay** ve **Bir eylem** listeleri cümleyi yazar, siz her `?` yerine bir ad ya da sayı koyarsınız.
- *Uygulamak.* **Uygula** (Ctrl+S) yazılanı belgeye alır; **Uygula ve Test Et** (Ctrl+Enter) alıp oyunu tarayıcıda açar. Okunamayan satır varken metin uygulanmaz. Esc pencereyi kapatır.
- *Yapıştırmak.* Ctrl+V başka bir yerden kopyalanmış kuralları yapıştırır: bir oyunun kuralları bir arkadaşa metin olarak gönderilebilir.
- *Geri almak.* Ctrl+Z son değişikliği geri alır, Ctrl+Y (ya da Ctrl+Shift+Z) yeniden yapar. Art arda yazılan harfler tek değişiklik sayılır: bir sözcük bütün olarak geri gelir.
- *Okunan ama işe yaramayan.* Metin okunsa bile umulanı yapmayacaksa pencere bunu sarı bir satırla söyler ve düzeltilebiliyorsa yanına düzelten bir düğme koyar. "Top henüz zıplayamaz: Düş ve Zıpla davranışı gerekir" satırının yanındaki **Düş ve Zıpla Ver**, pencereden çıkmadan o davranışı Top adındaki nesnelere verir. "Belgede kTop adında bir şey yok. Top mu demek istediniz?" satırının yanındaki **Top Yaz**, metindeki adı düzeltir. Çok küçük bir zıplama ("3 piksellik zıplama gözle görülmez") de böyle söylenir.

- *Yazarken öneri.* Bir satıra yazmaya başlayınca imlecin altında öneriler çıkar: o satırın olabileceği cümleler ve belgedeki adlar. Ne yazdığınıza bakar: bir nesnenin adını yazarsanız nesne cümlelerini (`Elma nesnesini gizle`), bir sayının adını yazarsanız sayı cümlelerini (`Puan sayısını ? artır`), bir rakam yazarsanız rakamla başlayanları (`3 kez yinele:`) önerir; adı küçük harfle yazsanız da doğru yazımıyla koyar. Yukarı ve aşağı oklar seçer, **Tab** alır. Cümle alındığında imleç ilk boşluğa gider; boşlukları doldurdukça Tab bir sonrakine geçer. Esc önerileri kapatır.

Yazının listelere göre fazlası şunlardır:

- *İşlem.* Sayı beklenen her yere bir işlem yazılabilir: `Puan sayısını Puan * 2 yap`, `Can sayısını Puan / 10 artır`. Dört işlem ve ayraç kullanılır. Bir nesnenin yeri `Ad.x` ve `Ad.y` ile okunur: `Mermi nesnesini Gemi.x + 20, Gemi.y konumuna koy`. Rastgele bir tam sayı `rastgele(1, 6)` ile alınır. Adında boşluk olan nesne tırnak içine yazılır: `"Dev Fare".x`. (Listelerdeki sayı kutuları da işlem kabul eder.)
- *Birden çok koşul.* Koşullar `ve` ile sıralanır: `Kahraman nesnesi Kapı nesnesine değince, Puan sayısı 2 değerinden büyükse ve Can sayısı 0 değerinden büyükse:`. Hepsi doğruysa kural izlenir.
- *Değilse.* Eylemlerden sonra kenara `değilse:` yazılır; koşullar tutmadığında onun altındakiler yapılır.
- *Bekle.* `1 saniye bekle` ardından gelen eylemleri erteler: önce ses, bir saniye sonra sahne değişimi.
- *Yinele.* `3 kez yinele:` altına içeriden yazılan eylemleri üç kez yapar; sayı yerine bir işlem de olabilir (`Atış kez yinele:`). İçine `bekle` konursa turlar arasında beklenir.
- *Eğer.* Bir kuralın içinde `eğer Puan sayısı 10 değerinden büyükse:` altındakiler yalnız koşul doğruysa yapılır; aynı hizaya yazılan `değilse:` öbür durumu söyler. Bloklar iç içe yazılabilir.
- *Koşullu yinele.* `Puan sayısı 0 değerinden büyükse yinele:` altındakileri koşul doğru kaldıkça yapar. Hiç bitmeyecek bir döngü oyunu kilitlemez; program onu keser.
- *Komut.* Sık yapılan bir işe ad verilir: kenara `Patlat komutu:` yazılır, altına eylemleri; sonra herhangi bir kuralda `Patlat komutunu yap` denir. Komut, kendisinden aşağıda yazılmış olsa da kullanılabilir; başka komutları, kendisini de çağırabilir.
- *Komuta değer vermek.* Komut değer alabilir: `Ödül komutu, Kaç, Kim ile:` iki değer alan bir komut tanımlar; `Ödül komutunu 10, Altın ile yap` onu bu değerlerle yapar. Komutun içinde değerin adı bir sayı gibi kullanılır (`Puan sayısını Kaç * 2 artır`) ya da verilen nesnenin yerine geçer (`Kim nesnesini gizle`). Değer bir sayı, bir işlem ya da bir nesnenin adı olabilir; adında boşluk olan nesne tırnak içine yazılır. Komuta, aldığı kadar değer verilir; eksik ya da fazla verilirse satır kırmızı olur ve nedeni yazar. Komutun içinde bir değeri değiştirmek (`Kaç sayısını 1 azalt`) yalnızca o çalışmayı etkiler.

```
Kahraman nesnesi Kapı nesnesine değince, Puan sayısı 2 değerinden büyükse ve Can sayısı 0 değerinden büyükse:
    zafer sesini çal
    1 saniye bekle
    2. sahneye git
değilse:
    eyvah sesini çal
    Kahraman nesnesini Kahraman.x - 40, 322 konumuna koy
```

```
Sayılar:
    Puan = 0
    Atış = 3

Patlat komutu:
    Patlama nesnesinin kopyasını Düşman nesnesinin yerinde oluştur
    bum sesini çal

Boşluk tuşuna basılınca:
    Atış kez yinele:
        Mermi nesnesinin kopyasını Gemi nesnesinin yerinde oluştur
        0.2 saniye bekle
    eğer Puan sayısı 10 değerinden büyükse:
        Patlat komutunu yap
    değilse:
        vız sesini çal
```

```
Ödül komutu, Kaç, Kim ile:
    Puan sayısını Kaç * 2 artır
    Kim nesnesini bir an yanıp söndür

Altın nesnesine tıklanınca:
    Ödül komutunu 10, Altın ile yap
```

Blok açan satırlar: `… kez yinele:`, `… yinele:` (koşullu), `eğer …:`, `değilse:`. Komut tanımı: `… komutu:` ya da `… komutu, …, … ile:`. Komutu yapmak: `… komutunu yap` ya da `… komutunu …, … ile yap`.

Olay cümleleri: `Film başlayınca`, `Bir sahne başlayınca`, `… tuşuna basılınca`, `… tuşu bırakılınca`, `… nesnesine tıklanınca`, `… nesnesi sahnenin … değince`, `… nesnesi … nesnesine değince`, `Her … saniyede bir`, `… saniye sonra`, `… sayısı … değerini geçince`, `… sayısı … değerinin altına inince`, `… sayısı … olunca`.

*Döndürmek, büyütmek, davranış vermek.* `döndür` nesneyi verilen derece kadar, saat yönünde, dönüştürme noktasının çevresinde çevirir; her yapılışında üstüne ekler (eksi sayı ters yöne çevirir); bir anda olur, bu yüzden 360 derece döndürmek hiçbir şey göstermez. Sürekli dönmesi için `saniyede … tur dönsün` yazın: "Düğme nesnesine tıklanınca: Pervane nesnesi saniyede 3 tur dönsün" vantilatörü çalıştırır; eksi sayı saat yönünün tersine döndürür, 0 durdurur. Bir sayıyla da verilebilir: `Pervane nesnesi saniyede Hız tur dönsün`. `boyuta getir` ve `opak yap` çizildiği hâlin yüzdesiyle çalışır: yüzde 100 çizildiği gibidir, yüzde 200 iki katı, yüzde 0 opak hiç görünmez. `davranışını ver`, Davranışlar menüsündeki bir davranışı adıyla ve başlangıç sayılarıyla o anda verir: "Düğme nesnesine tıklanınca: Çark nesnesine Dön davranışını ver" çarkı döndürmeye başlatır, `davranışını al` durdurur (dönen nesne o anki açısında kalır). `izlesin` bir nesneyi ötekinin peşine takar: saniyede o kadar pikselle ona doğru gider; hız 0 verilirse peşini bırakır.

*Hızlanıp yavaşlayan hareket.* `Araba nesnesi sağa 300 hızla gitsin` arabayı yola çıkarır: yarım saniyeden kısa sürede o hıza çıkar ve gitmeye devam eder. Yön `sağa`, `sola`, `yukarı` ya da `aşağı` yazılır; hız saniyede pikseldir. `Araba nesnesi yavaşlayarak dursun` hızını azar azar düşürür, bir buçuk saniyede durur; `Araba nesnesi dursun` hemen durdurur. Tuşa basılıyken gidip bırakınca yavaşlaması için iki kural yeter: biri `Sağ Ok tuşuna basılınca`, öteki `Sağ Ok tuşu bırakılınca`.

*Bir nesneyi ötekine bağlamak.* En kolayı menüdendir: bağlanacak nesneyi seçin, **Davranışlar > Başka Nesneye Bağla…** deyin ve açılan listede birlikte gideceği nesnenin adına tıklayın (o nesnenin Özellikler'de bir adı olmalıdır). Aynı listedeki **Hiçbir şeye bağlı değil** bağı kaldırır. Oyun sırasında bağlamak için kural da yazılabilir: `Tekerlek nesnesini Araba nesnesine bağla` dedikten sonra tekerlek arabanın gittiği her yere gider; yalnızca arabayı hareket ettirmek yeter. Bağlı nesnenin kendi davranışları sürer: **Teker Gibi Yuvarlan** verilmiş tekerlek, araba ne kadar yol giderse o kadar döner. `serbest bırak` bağı çözer; nesne olduğu yerde kalır. Bağlı nesne ötekinin dönmesini ya da büyümesini izlemez, yalnızca yer değiştirmesini.

*Kamera ve kayan arka plan.* Bir nesneye **Davranışlar > Kamera Beni İzlesin** verin: yayımlanan filmde o nesne ekranda çizildiği yerde kalır, sağa gittikçe dünya sola kayar. Kamera yalnızca sağa-sola kayar. Neyin nasıl kayacağını üç kural belirler:

- Davranışı olmayan ve hiçbir kuralda adı geçmeyen çizimler (gökyüzü, zemin) tek bir resim olur ve **ekranda sabit kalır**.
- Davranışı olan ya da bir kuralda adı geçen her nesne **dünyanın parçasıdır**: kamera kaydıkça tam o kadar kayar ve ekrandan çıkıp gider (düşman, engel, kapı).
- **Arka Plan Gibi Kay** verilen nesne, yüzdesi kadar kayar: yol çizgileri ve yol kenarındaki çalılar için 100, tepeler için 20–40, bulutlar için 5–10. Uzaktaki az, yakındaki çok kayar; paralel arka plan budur. "Öbür yandan geri gelir" açıkken nesne ekranın bir ucundan çıkınca öbür ucundan girer, böylece yol hiç bitmez. Yüzde 0 nesneyi ekrana sabitler: dönen bir güneş ya da bir can göstergesi için.

Bir sayıyı gösteren yazılar (`Puan: {Puan}`) her zaman ekranda sabittir. Kamera açıkken "sahnenin kenarı" ekranın kenarıdır: ekrandan çıkan mermi "sağ kenara değince" kuralıyla silinebilir. Tekrar eden nesneleri sahnenin içine çizin ve eşit aralıklı olmaları için aynı genişlikte yapın: her biri sahne genişliği artı kendi genişliği kadar yolda bir geri gelir. Kuralla da yönetilir: `kamera Araba nesnesini izlesin` kamerayı o anda, sıçratmadan o nesneye verir; `kamera yerinde dursun` bırakır. Program içinde Enter ile oynatırken kamera kaymaz; **Filmi Test Et** ile görün.

**Ders: araba.** (**Araba sürmek** örneği.) Gövdeyi çizip gruplayın ve "Araba" adını verin. Tekerleri gövdenin grubuna katmayın: iki ayrı tekerlek çizin, ikisine de "Tekerlek" adını ve **Teker Gibi Yuvarlan** davranışını verin (dönmesi görünsün diye tekerleğe bir jant çizgisi çizin). İki tekerleği seçip **Davranışlar > Başka Nesneye Bağla…** ile "Araba"ya bağlayın. Kurallar:

```
Sağ Ok tuşuna basılınca:
    Araba nesnesi sağa 320 hızla gitsin

Sağ Ok tuşu bırakılınca:
    Araba nesnesi yavaşlayarak dursun
```

Sol ok için aynı iki kuralı `sola` ile yazın. Araba hızlanır, tuşu bırakınca yavaşlayarak durur; tekerler de onunla birlikte yavaşlar.

Yolun kayması için arabaya **Kamera Beni İzlesin** verin. Yol çizgilerine ve çalılara yüzde 100 ile, tepelere 20–45 ile, bulutlara 5–10 ile **Arka Plan Gibi Kay** verin. Gökyüzüne ve asfalta hiçbir şey vermeyin: yerlerinde kalırlar. Artık araba ekranda durur, yol altından akar; araba yavaşlayınca yol da yavaşlar.

Eylem cümleleri: `… nesnesini … sağa, … aşağı taşı`, `… nesnesini rastgele bir yere koy`, `… nesnesini üstte rastgele bir yere koy`, `… nesnesini …, … konumuna koy`, `… nesnesini göster`, `… nesnesini gizle`, `… nesnesini … yükseğe zıplat`, `… nesnesini … derece döndür`, `… nesnesi saniyede … tur dönsün`, `… nesnesini yüzde … boyuta getir`, `… nesnesini yüzde … opak yap`, `… nesnesine … davranışını ver`, `… nesnesinden … davranışını al`, `… nesnesi … nesnesini … hızla izlesin`, `… nesnesi … … hızla gitsin`, `… nesnesi yavaşlayarak dursun`, `… nesnesi dursun`, `… nesnesini … nesnesine bağla`, `… nesnesini serbest bırak`, `kamera … nesnesini izlesin`, `kamera yerinde dursun`, `… sayısını … azalt`, `… sayısını … artır`, `… nesnesinin kopyasını … nesnesinin yerinde oluştur`, `… nesnesini sil`, `… nesnesini bir an yanıp söndür`, `… nesnesini yüzde … yavaşlat`, `… nesnesini yüzde … hızlandır`, `… sayısını … ile … arasında rastgele yap`, `… sayısını … yap`, `… saniye bekle`, `filmi durdur`, `filmi oynat`, `…. kareye git ve oynat`, `…. kareye git ve dur`, `sonraki sahneye git`, `…. sahneye git`, `… sesini çal`, `her şeyi durdur`, `baştan başla`.

Koşullar: `… sayısı … ise`, `… sayısı … değerinden büyükse`, `… sayısı … değerinden küçükse`. Tuş adları: Boşluk, Yukarı Ok, Aşağı Ok, Sol Ok, Sağ Ok, Enter, harfler ve rakamlar. Kenarlar: bir kenarına, alt kenarına, üst kenarına, sol kenarına, sağ kenarına.

Program İngilizce kullanılırken aynı kurallar İngilizce cümlelerle yazılır ve okunur (`When Apple touches Basket:`); belgedeki kurallar dilden bağımsız saklandığı için bir dilde yazılan oyun ötekinde o dilin cümleleriyle görünür. Nesnelerin ve sayıların adları çevrilmez.

Listelerde (Kurallar ve Sayılar) bloklar ve komutlar görünür ama orada kurulmaz; onlar yazılır. Dilin henüz yapmadıkları: komut bir değer geri vermez (bir işlemin içinde kullanılamaz); sayılar dışında değer (sözcük, liste) saklanmaz; bir nesnenin yalnız yeri okunur, boyu ya da görünürlüğü okunmaz. Yazı alanı sade bir düzenleyicidir: seçme ve bul-değiştir yoktur; uzun bir metni kendi düzenleyicinizde yazıp Ctrl+V ile yapıştırabilirsiniz.

**Eklem yerinden döndürmek.** Hareket ara doldurması, nesneyi Flash'taki gibi *dönüştürme noktasının* çevresinde döndürür. Bu nokta başlangıçta nesnenin ortasındadır. Taşımak için nesneyi seçin, **Ölçekle** aracını (Q) alın ve kutunun ortasındaki küçük kareyi nesnenin döneceği yere sürükleyin: bir kolun omuz ucuna, bir tekerleğin göbeğine, bir kapının menteşesine. Enter'a basın. Nokta bundan sonra nesneyle birlikte kalır; taşımada, döndürmede ve yeni anahtar karelerde de. Ara doldurma noktayı anahtar kareden anahtar kareye düz götürür, nesne onun çevresinde döner. Noktayı ilk anahtar karede, ötekileri eklemeden önce yerine koyun. Bir grup ya da resim için önce sembole dönüştürün (F8); Flash da her hareket ara doldurması için bunu ister.

**Ders: çöp adam dövüşü.** **Örnekler > Çöp adam dövüşü**'nü açıp oynatın (Enter), sonra parçalarına bakın. Böyle bir film şöyle yapılır:

1. *Figür.* Kendi katmanında, Çizgi aracıyla ve kalın konturla figürü çizin: gövde için bir çizgi, her kol için iki (omuzdan dirseğe, dirsekten ele), her bacak için iki çizgi ve baş için bir elips. Her çizgiyi asıldığı eklemden başlayarak çizin.
2. *Eklemler.* Ölçekle aracıyla her çizginin dönüştürme noktasını o ekleme taşıyın: üst kolunkini omza, ön kolunkini dirseğe, uyluğunkini kalçaya, baldırınkini dize.
3. *Pozlar.* Filmi birkaç güçlü poz olarak düşünün: gard, kolu geri çekme, yumruk, yeniden gard. Her pozun olacağı kareye anahtar kare (F6) koyun ve orada çizgileri Ölçekle aracıyla poza getirin (döndürmek için kutunun dışından sürükleyin). Önce üst kolu döndürün, sonra ön kolu yeniden dirseğe taşıyıp onu döndürün.
4. *Zamanlama.* Vuruş hızlıdır: geri çekmeden yumruğa iki kare, sonra yumruk dört kare bekler. Bekleyiş yavaştır: bir gard pozundan biraz daha alçağına ve geri on kare. Canlı göstereni bu eşit olmayan zamanlamadır.
5. *Aradaki kareler.* Anahtar karelerin karelerini seçip **Hareket Ara Doldurması Oluştur**'u seçin. Birbirinden uzak iki pozun ortasına bir anahtar kare ekleyip düzeltin: ön kol anahtar kareden anahtar kareye düz gider, dirsek ise yay çizer; geniş bir savuruşta ayrılırlar. Örnekte her iki karede bir, vuruş sırasında her karede bir anahtar kare vardır.
6. *Düşmanlar.* Her biri kendi katmanında ve kendi zamanında: henüz gelmediği yerde bir boş anahtar kare (F7), gittiği yerde bir tane daha.
7. *Efektler.* Vuruşun indiği yerde, kendi katmanında, dört karede büyüyüp sönen bir yıldız ve yanında bir sözcük.
8. *Filmi Test Et* (Ctrl+Enter) ile tarayıcıda izleyin.

**Sağ tık.** Bir kareye sağ tıklamak o kareye gider ve ara doldurma, kare ve anahtar kare komutlarının menüsünü açar; o kareye uymayan komutlar gri görünür. Katman adına sağ tıklamak katman komutlarını (yeni katman, sil, kılavuz, maske, bağla) açar. Sahnede bir nesneye sağ tıklamak onu seçer ve Kes, Kopyala, Yerleştir, Dönüştür, Grupla, Sembole Dönüştür gibi komutları açar; PisiWorks'ün tuvalinde de aynı menü vardır.

**Hareket ara doldurma.** Nesneyi ilk karede çizin, ileri bir karede F6 ile anahtar kare ekleyip nesneyi taşıyın, büyütün ya da döndürün; ilk anahtar kareye dönüp **Zaman Çizelgesi > Hareket Ara Doldurması Oluştur** deyin. Aradaki kareler mavi olur. Altta **Yavaşlama** (−100 yavaş başlar, 100 yavaş biter) ve **Tur** (yol boyunca tam dönüş sayısı) alanları çıkar. Dönen bir tekerlek için iki yol vardır: ikinci anahtar karede şekli döndürmek (Ölçek aracıyla ya da Dönüştür > 90° Döndür ile), ya da iki anahtar kareyi aynı bırakıp **Tur** alanına 1 yazmak; ikincisi tam bir tur attırır.

**Şekil ara doldurma.** İlk karede bir şekil, ileride F7 ile açtığınız boş anahtar karede başka bir şekil çizin; ilk karede **Şekil Ara Doldurması Oluştur** deyin. Aradaki kareler yeşil olur. Dönüşüm istediğiniz gibi olmuyorsa **Şekil İpucu Ekle** (Ctrl+Shift+H) ile işaretler koyun: ilk karedeki sarı işareti ve son karedeki yeşil işareti, birbirine dönüşmesini istediğiniz noktalara sürükleyin.

**Kılavuz katmanı.** Hareket eden nesnenin katmanının üstüne yeni bir katman ekleyip içine bir yol çizin, sonra **Kılavuz Katmanı** deyin. Nesne yolu izler; **Yola yönlendir** işaretliyse yolun yönüne de döner. Kılavuz yayımlanan filmde görünmez.

**Maske katmanı.** Maskelenecek katmanın üstüne bir katman ekleyip içine şekil çizin ve **Maske Katmanı** deyin: alttaki katman yalnızca o şeklin olduğu yerde görünür. Maske Önizleme'de, yayımda ve iki katman da kilitlenince sahnede uygulanır. Başka katmanları da aynı kılavuza ya da maskeye bağlamak için **Kılavuza ya da Maskeye Bağla** kullanılır.

**Semboller.** **Düzenle > Ekle > Yeni Sembol** üç tür sunar. *Grafik* yerinde düzenlenir. *Film Klibi*nin kendi zaman çizelgesi vardır: kopyasına çift tıklayınca klibin sayfası açılır, sahnenin üstündeki **◀** düğmesi geri döndürür. *Düğme*nin Yukarıda, Üzerinde ve Aşağıda durumları vardır.

**Eylemler.** Zaman çizelgesinin altındaki **Kare eylemi** listesi, o anahtar kareye gelindiğinde ne olacağını belirler; eylemi olan anahtar kare kırmızı görünür. Bir düğme seçiliyken Özellikler'deki **Eylem** listesi tıklanınca ne olacağını belirler. Eylemler: Dur, Oynat, Kareye Git ve Oynat, Kareye Git ve Dur, Sahneye Git, Adrese Git. Kare ve sahne numaraları 1'den başlar.

**Sahneler.** Sahneler panelinde her sayfa bir sahnedir. Film sahneleri sırayla oynatır; sonuncudan sonra başa döner.

**Ses.** **Ses İçe Aktar** bir MP3, OGG ya da WAV dosyasını belgeye ekler ve bulunduğunuz anahtar karede başlatır; mavi çerçeveli anahtar kare sesli demektir. Program içinde oynatırken ses, sistemde `ffplay`, `mpv`, `paplay` ya da `aplay` kuruluysa duyulur.

**Yayımlama.** **Kontrol > HTML5 Olarak Yayımla** filmi tek bir HTML dosyasına yazar; **Filmi Test Et** aynı dosyayı geçici olarak üretip tarayıcıda açar. Film klipleri, düğmeler, eylemler ve ses yalnızca yayımlanan filmde çalışır; program içinde oynatırken klipler ilk kareleriyle görünür.

**Örnekler ve PisiWorks ile alışveriş.** Menü çubuğunun sağındaki **Örnekler** menüsünde on altı hazır film vardır (ilki vitrin oyunu Pisi: Demir Pençe; beş küçük oyun karakterler, manzara, parlamalar ve seslerle çizilmiştir); her biri bir konuyu gösterir (yavaşlamalı hareket, şekil dönüşümü, kılavuz, maske, film klibi, kendi zamanı olan katmanlar, pozdan poza yapılmış bir çöp adam dövüşü, davranışlar, kurallarla yapılmış beş oyun, hızlanıp yavaşlayan bir araba, sahneler ve düğmeler). **Dosya > Kareyi PisiWorks'te Aç**, o anki kareyi PisiWorks'te bir kopya olarak açar; PisiWorks'teki **Dosya > PisiAnimasyon'da Aç** tersini yapar. Kopyalar önbellek klasörüne yazılır; özgün belgeniz değişmez. Bir programda hazırladığınız çizimi öbürüne almak için **Dosya > İçe Aktar** ile `.pisi` ya da `.sahne` dosyasını seçin: nesneler etkin katmana, semboller kitaplığa gelir.
