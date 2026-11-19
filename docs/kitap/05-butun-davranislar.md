@kisim 2. Hareket
# Bütün davranışlar
> Davranışlar menüsündeki on dokuz hazır hareketi tek tek tanıyacak, hangisinin ne zaman işe yaradığını ve birbirleriyle nasıl birleştiğini göreceksin.
:: ⏱ 25 dakika | 🎯 1. kısmı bitirenler için | 🧰 Örnekler > Davranışlar

![Davranışlar örneği: sahnedeki her şey tek bir karede duruyor; hiçbirinin anahtar karesi yok, hepsi birer davranışla hareket ediyor.](05-film.png "Davranışlar örneğinin oynayan hâli: dönen güneş, zıplayan top, giden ok, yanıp sönen yazı" 560){05-davranislar.html 560x300}

!goal Bu bölümde öğreneceklerin
Davranışların üç ailesini, her birinin sayılarını, davranışı kaldırmayı ve birkaç davranışı aynı nesnede birleştirmeyi öğreneceksin. Bu bölüm bir başvuru bölümüdür: ileride "şunu nasıl yapıyorduk?" dediğinde dönüp bakacağın yer burası.

## Örneği aç

1. Sağ üstteki {{Örnekler}} menüsünü aç ve {{Davranışlar}}'a tıkla.
2. Önce [[Enter]]'a bas: kendi kendine olan hareketler programın içinde oynar. Durdurmak için yeniden [[Enter]].
3. Sonra {{Filmi Test Et}} ([[Ctrl]]+[[Enter]]): tarayıcıda ok tuşlarıyla kutuyu yürüt, yıldızı sürükle, fareni gezdir.

![Örnek açıldığında sahne.](05-ornek.png "Davranışlar örneğinin sahnesi" 580)

## Birinci aile: kendi kendine olanlar

Bunlar kimseyi beklemez; film başlar başlamaz çalışır. Programın içinde [[Enter]] ile de görünürler.

| Davranış | Nesne ne yapar | Sayıları | Nerede kullanılır |
| Dön | Olduğu yerde döner | Bir tur kaç saniye; ters yön | Güneş, pervane, çark, yıldız |
| Zıpla | Olduğu yerde zıplar durur | Yükseklik; bir zıplama kaç saniye | Top, sevinen karakter |
| Git Gel | Sağa gider, geri gelir | Uzaklık; gidiş-geliş süresi | Nöbet tutan düşman, hareketli platform |
| İn Çık | Yukarı çıkar, iner | Yükseklik; süre | Balon, asansör, süzülen bulut |
| Sürekli Git | Bir yönde gider; sahneden çıkınca öbür yandan girer | Hız; yön | Bulut, akan yol, kayan yıldız |
| Titre | Yerinde titrer | Kuvvet | Çalan zil, korkan karakter, deprem |
| Büyü Küçül | Büyür, küçülür | Yüzde; süre | Kalp atışı, dikkat çeken düğme |
| Yanıp Sön | Görünür, kaybolur | Süre | Uyarı yazısı, "Başla!" |
| Belir Kaybol | Yavaşça saydamlaşır, geri gelir | Süre | Hayalet, parlayan ışık |
| Bir Süre Sonra Kaybol | Verilen süre dolunca yok olur | Süre | Patlama, "+1" yazısı, selamlama |

## İkinci aile: oyuncunun yönettikleri

Bunlar klavyeyi ya da fareyi bekler; bu yüzden yalnızca **Filmi Test Et** ile, tarayıcıda çalışır.

| Davranış | Nesne ne yapar | Sayıları | Nerede kullanılır |
| Fareyi İzle | Fareye doğru gider | Hız (0: anında) | Nişangâh, takip eden arı |
| Ok Tuşlarıyla Yürü | Ok tuşlarıyla yürür | Hız | Oyunun kahramanı |
| Düş ve Zıpla | Düşer; yukarı ok ya da boşlukla zıplar | Çekim; zıplama hızı | Platform oyunlarının kahramanı |
| Sürüklenebilir | Fareyle sürüklenir | Yok | Yapboz parçası, taşınacak eşya |

## Üçüncü aile: dünyayı kuranlar

Bunlar nesnenin öbür nesnelerle ve sahneyle ilişkisini belirler. Onlar da tarayıcıda çalışır.

| Davranış | Nesne ne yapar | Sayıları | Nerede kullanılır |
| Katı | İçinden geçilemez | Yok | Zemin, platform, duvar |
| Kenarlardan Sek | Dümdüz uçar, çarpınca geri seker | Hız; yön | Top, dolaşan düşman |
| Teker Gibi Yuvarlan | Gittiği yol kadar döner | Yok | Araba tekeri, yuvarlanan taş |
| Kamera Beni İzlesin | Ekranda yerinde kalır, dünya yanından kayar | Yok | Yana kayan oyunların kahramanı |
| Arka Plan Gibi Kay | Kamera kaydıkça yüzdesi kadar kayar | Yüzde; öbür yandan geri gelsin mi | Tepeler, bulutlar, yol çizgileri |

Son iki davranışı 20. bölümde, araba dersinde kullanacağız.

## Sayıları değiştirmek

1. Nesneyi seç ((1)).
2. {{Davranışlar > Davranış Ayarları…}}'nı aç.
3. Kutudaki sayıyı değiştir ve [[Enter]]'a bas. Davranışı tümüyle kaldırmak için yanındaki {{Kaldır}}'a tıkla ((2)).

![Davranış Ayarları penceresi: seçili topun **Zıpla** davranışı, iki sayısı ve Kaldır düğmesi.](05-ayarlar.png "Davranış Ayarları penceresi; arkada seçili top" 600)

!note Bir davranışı kaldırmanın iki yolu
Davranış Ayarları'ndaki **Kaldır** düğmesi, ya da Davranışlar menüsünde aynı davranışa bir kez daha tıklamak. Hepsini birden silmek için {{Davranışlar > Tüm Davranışları Kaldır}}.

## Davranışları birleştirmek

Bir nesnenin aynı anda birkaç davranışı olabilir; hepsi birlikte çalışır. İşte işe yarayan birkaç tarif:

| İstediğin | Ver |
| Platform oyunu kahramanı | Ok Tuşlarıyla Yürü + Düş ve Zıpla |
| Patlama | Büyü Küçül + Dön + Bir Süre Sonra Kaybol (0.4 saniye) |
| Yükselip kaybolan "+1" | İn Çık + Belir Kaybol + Bir Süre Sonra Kaybol |
| Hareketli platform | Git Gel + Katı |
| Düşen elma | Düş ve Zıpla (zıplama hızı 0) |
| Dolaşan arı | Fareyi İzle + Titre |

!why Davranış mı, anahtar kare mi?
Davranış, "hep şunu yap" demektir ve hiç bitmez. Anahtar kareyle yapılan animasyon (bir sonraki bölüm) ise "şu anda şurada ol, iki saniye sonra şurada" demektir ve bir hikâye anlatır. Dönen bir pervane davranıştır; sahneye yürüyerek girip selam veren bir karakter anahtar karedir. İkisi aynı nesnede birlikte de kullanılabilir.

## Sıra sende

- [ ] Örnekteki güneşi seç ve bir turu 8 saniye yerine 2 saniyede attır.
- [ ] Oku seç, **Git Gel**'i kaldır ve yerine **Sürekli Git** ver.
- [ ] Boş bir filmde yukarıdaki "patlama" tarifini dene: bir yıldız çiz, üç davranışı ver, [[Enter]]'a bas.
- [ ] Zor soru: **Git Gel** ile **İn Çık**'ı aynı nesneye, aynı süreyle verirsen nesne nasıl bir yol çizer? Önce tahmin et, sonra dene.

## Bölümün özeti

| Aile | Ne zaman çalışır | Nerede görünür |
| Kendi kendine olanlar | Hep | Programda ([[Enter]]) ve tarayıcıda |
| Oyuncunun yönettikleri | Tuşa basınca, fare oynayınca | Yalnızca tarayıcıda (Filmi Test Et) |
| Dünyayı kuranlar | Bir şeye değince, bir yere gidince | Yalnızca tarayıcıda (Filmi Test Et) |

!teacher Öğretmene not
Bu bölüm ezberletilmek için değil, keşfettirilmek için yazıldı. Öğrencilere Davranışlar örneğini açtırıp on dakika serbest bırakın; her nesneye tıklayıp Davranış Ayarları'na bakmalarını, sayıları abartılı biçimde değiştirmelerini isteyin. Ardından "birleştirme tarifleri" tablosundan birer tarif dağıtıp boş bir filmde yaptırın. "Davranış mı, anahtar kare mi?" kutusu bir sonraki bölümün girişidir; ders sonunda birkaç örnek sorup sınıfa hangisi olduğunu tahmin ettirmek iyi bir kapanış olur.
