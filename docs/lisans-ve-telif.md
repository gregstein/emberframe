# PisiWorks ve PisiAnimasyon: lisans ve telif bildirimi

Bu belge, PisiWorks'ü ve onunla aynı pakette gelen animasyon programı PisiAnimasyon'u Pardus depolarına ya da okullara dağıtmadan önce inceleyecek kişiler için yazıldı. Programın hangi lisansla yayımlandığını, kodunun nereden geldiğini, içine hangi kitaplıkların derlendiğini ve Adobe ile ilişkisini tek yerde toplar. Sonunda, bizim karar veremediğimiz ve inceleyenin görüşüne bıraktığımız noktalar listelenir.

Bu bir hukuki görüş değildir; bir hukukçu tarafından gözden geçirilmedi. Buradaki her bilgi depodaki dosyalardan doğrulanabilir; nasıl doğrulanacağı ilgili bölümde yazıyor.

Sürüm: PisiWorks ve PisiAnimasyon 0.2.0, Kasım 2026. İki program aynı kaynak koddan derlenir, aynı lisansı taşır ve tek bir `.deb` paketiyle kurulur; aşağıda "PisiWorks" için söylenen her şey, ayrıca belirtilmedikçe PisiAnimasyon için de geçerlidir. Depo: [github.com/MuratSAPMAZ/PisiWorks](https://github.com/MuratSAPMAZ/PisiWorks)

## Amaç

PisiWorks ve PisiAnimasyon **ticari bir amaç taşımaz.** Tek amaçları, müfredatında hâlâ Fireworks ve Flash anlatılan okullarda, Pardus kullanan öğretmen ve öğrencilerin dersi kendi bilgisayarlarında işleyebilmesidir.

- Programlar ücretsizdir ve ücretsiz kalacaktır. Satılmaz; ücretli sürümü, aboneliği, lisans anahtarı, uygulama içi satın alması yoktur.
- Reklam göstermez, bağış istemez, kullanıcıdan ya da bilgisayarından veri toplamaz, kendiliğinden internete bağlanmaz.
- Gelir amacı güdülmez. Gönüllü bir çalışmadır.
- Adobe'nin bir ürününün yerine pazarda yer almayı amaçlamaz. Fireworks 2013'ten beri geliştirilmiyor; Flash Player 2020'de sonlandırıldı. İkisi de Pardus'ta hiç çalışmadı. Bu programlar, o araçların çalışmadığı bir yerde dersin sürebilmesi içindir.
- Benzerlik bu amaca hizmet ettiği kadardır: öğretmenin ders notundaki adım, öğrencinin ekranındaki adımla aynı olsun diye. Bunun ötesinde Adobe'ye ait hiçbir şey alınmadı.

Kaynak kodunun GNU Genel Kamu Lisansı (GPL) ile açık olması da aynı amaçtandır: okullar, Pardus ekibi ve isteyen herkes programı inceleyebilsin, değiştirebilsin ve dağıtabilsin; ve değiştirip dağıtan da kendi sürümünü aynı biçimde açık tutsun. GPL ticari kullanımı yasaklamaz; ama programı alıp kapalı bir ürüne çevirmeye izin vermez.

## 1. Özet

| Soru | Cevap |
|---|---|
| Amaç | Eğitim. Ticari amaç yoktur; programlar ücretsizdir, reklam ve veri toplama içermez (bkz. Amaç) |
| İki programın lisansı | GNU GPL, sürüm 3 ya da sonrası. Tam metin: [`COPYING`](../COPYING); özet ve istisna: [`NOTICE`](../NOTICE) |
| Emberframe'den gelen kodun lisansı | MIT (Debian'daki adıyla Expat), yazarının telif bildirimiyle: [`LICENSES/MIT.txt`](../LICENSES/MIT.txt) |
| Programlarla yapılan film ve sayfalar | Yapanındır; içlerine yazılan oynatıcı betiğiyle birlikte istenen koşulla dağıtılabilir (GPL 7. madde uyarınca ek izin) |
| Telif sahipleri | Med Amine Belbachir (Emberframe, 2026) ve Murat SAPMAZ (PisiWorks için yazılanlar, 2026) |
| Kaynağı | MIT lisanslı [Emberframe](https://github.com/gregstein/emberframe) projesinin çatalı |
| İçine derlenen kitaplıklar | 517 kitaplık, hepsi özgür yazılım; listesi [`THIRD-PARTY.md`](THIRD-PARTY.md) |
| Copyleft (GPL türü) yükümlülük | Var: programın kendisi GPL'dir; değiştirilmiş bir sürümünü dağıtan, kaynak kodunu da aynı lisansla vermelidir. Kitaplıklardan gelen tek zayıf copyleft, MPL-2.0 lisanslı `option-ext`; değiştirilmeden kullanılıyor (bkz. 4. bölüm) |
| Adobe'ye ait kod, görsel, font ya da dosya | Depoda ve pakette yok |
| Adobe dosya biçimleri | Fireworks PNG'nin katmanları okunur. Flash'ın `.fla` ve `.swf` biçimleri ne okunur ne yazılır |
| Ticari kullanım, değiştirme, yeniden dağıtım | Serbest; koşulu, dağıtılan sürümün de GPL ile, kaynak koduyla birlikte ve telif bildirimleri korunarak verilmesi |
| Pardus, TÜBİTAK ya da Adobe ile kurumsal bağ | Yok; bağımsız bir projedir |

## 2. PisiWorks'ün lisansı

PisiWorks ve PisiAnimasyon, 16 Kasım 2026'dan itibaren **GNU Genel Kamu Lisansı, sürüm 3 ya da sonrası** (GPL-3.0-or-later) ile yayımlanır. GPL programın kullanılmasına, kopyalanmasına, değiştirilmesine ve dağıtılmasına izin verir; karşılığında, programı ya da değiştirilmiş bir sürümünü dağıtanın kaynak kodunu da aynı lisansla vermesini ister. Program "olduğu gibi", hiçbir garanti verilmeden sunulur.

GPL, Özgür Yazılım Vakfı'nın özgür yazılım, Açık Kaynak Girişimi'nin açık kaynak, Debian'ın da Debian Özgür Yazılım Yönergeleri'ne uygun saydığı lisanslardandır; Pardus'un gönüllü geliştiricilere önerdiği lisanstır.

**Neden MIT'ten GPL'ye geçildi.** Proje başlangıçta, üzerine kurulduğu Emberframe gibi MIT lisansıyla yayımlandı. MIT, kodun kapalı bir ürünün içine alınmasına da izin verir. Okullar için yazılmış bir programın başkasınca alınıp kapatılması projenin amacına aykırıdır; GPL bunu önler ve Pardus'un yönlendirmesine uyar.

**Geçiş nasıl yapıldı.**

- Lisans metni [`COPYING`](../COPYING) dosyasındadır; kısa bildirim, Emberframe'e ilişkin açıklama ve ek izin [`NOTICE`](../NOTICE) dosyasındadır.
- Her kaynak dosyasının başında telif ve lisans bildirimi vardır; Emberframe'den gelen kod barındıran on beş dosya bunu ayrıca söyler.
- Emberframe'den gelen kod MIT lisansı altında kalır. MIT lisanslı kod GPL'li bir işin içinde kullanılabilir; koşulu, yazarının telif bildiriminin ve lisans metninin korunmasıdır. Bunlar [`LICENSES/MIT.txt`](../LICENSES/MIT.txt) dosyasındadır.
- 16 Kasım 2026'dan önce yayımlanan sürümler MIT ile yayımlanmıştı ve öyle kalır; verilmiş bir lisans geri alınamaz.
- İçine derlenen kitaplıkların lisansları GPL sürüm 3 ile bağdaşır (bkz. 4. bölüm). Bazıları (Apache-2.0) GPL sürüm 2 ile bağdaşmadığı için lisans "sürüm 3 ya da sonrası"dır.

**Yapılan filmler ve sayfalar: ek izin.** PisiAnimasyon bir filmi yayımlarken, PisiWorks bir sayfayı dışa aktarırken HTML dosyasının içine kendi yazdığı bir betiği koyar. Bu betik programın parçasıdır; ek bir izin olmasaydı bir öğrencinin yaptığı oyunun da GPL'ye bağlanıp bağlanmadığı tartışılabilirdi. Bunu açıkça ortadan kaldırmak için GPL'nin 7. maddesinin tanıdığı yolla bir ek izin verilmiştir: böyle bir dosya, içindeki betik programın yazdığı hâliyle kaldıkça, istenen koşulla dağıtılabilir. Film ya da sayfa yapanındır. Ek izin programın kendisini ve değiştirilmiş betiği kapsamaz. Tam metni `NOTICE` dosyasındadır; yayımlanan her filmin başında da kısa bir not vardır.

**Ad, köken ve emeğin görünürlüğü: ek koşullar.** GPL, programı alanın kendi sürümünü kapatmasını önler; ama tek başına, birinin programı başka bir adla sunup yazarını gözden kaçırmasını ya da değiştirdiği bir sürümü özgün adıyla dağıtmasını engellemez. GPL'nin 7. maddesi telif sahibine bu konularda ek koşul koyma hakkı tanır. Üç koşul konmuştur:

- *(7b) Atıf.* Hakkında penceresindeki geliştirici satırı, sitesinin ve kaynağın adresi ile Emberframe yazarına teşekkür, programı içeren her işte okunur biçimde korunur. Kaynak dosyalarının başındaki bildirimler ve `NOTICE` dosyası da korunur.
- *(7c) Köken.* Kimse özgün programları kendisinin yazdığını ileri süremez. Değiştirilmiş bir sürüm, değiştirilmiş olduğunu, kimin ve ne zaman değiştirdiğini açıkça söyler ve özgünüyle karıştırılmayacak, kendi adını taşır.
- *(7e) Adlar ve işaretler.* "PisiWorks" ve "PisiAnimasyon" adları ile programların simgeleri ve işaretleri, değiştirilmiş bir sürüm ya da başka bir ürün için kullanılamaz. Bir işin nereden geldiğini doğru biçimde söylemek ("PisiAnimasyon'a dayanır") serbesttir.

Değiştirilmemiş kopyalar programların kendi adlarıyla dağıtılabilir. Kaynağı bir sistem için derlemek, bir dağıtım için paketlemek ve yalnızca bunun gerektirdiği değişiklikler "değiştirilmiş sürüm" sayılmaz; yani Pardus'un paketlemesi bu koşullara takılmaz.

Bu koşullar GPL'nin özgürlüklerinden bir şey eksiltmez: programı herkes kullanabilir, inceleyebilir, değiştirebilir ve dağıtabilir. İstenen, değiştirilenin başka bir adla anılması ve özgününü kimin yazdığının söylenmesidir. Aynı düzeni Firefox gibi tanınmış özgür yazılımlar da kullanır: kod özgürdür, ad korunur.

Telif satırları:

```
Copyright (C) 2026 Murat SAPMAZ
Copyright (c) 2026 Med Amine Belbachir (https://medamineweb.com)   (Emberframe; MIT)
```

## 3. Kodun kökeni

PisiWorks, Med Amine Belbachir'in yazdığı ve MIT lisansıyla yayımladığı Emberframe 0.1'in çatalıdır. Çatallama GitHub üzerinde açıkça yapıldı; depo sayfasında "forked from gregstein/emberframe" yazar ve Emberframe'in bütün işleme geçmişi korunur.

- Emberframe'den gelen: çatal anında 12.286 satır Rust kodu. Bunun yaklaşık %88'i hâlâ yerindedir.
- PisiWorks ve PisiAnimasyon için yazılan: yaklaşık 26.400 satır. Bugünkü 37.312 satırın yüzde yetmişinden fazlası.
- PisiAnimasyon'un tamamı bu proje için yazıldı; Emberframe'de bir animasyon programı ya da zaman çizelgesi yoktur. PisiAnimasyon, PisiWorks'ün çizim kodunu bir kitaplık olarak kullanır.
- MIT lisansının gereği olan telif bildirimi `LICENSES/MIT.txt` ve `NOTICE` dosyalarında, Emberframe'den kod barındıran kaynak dosyalarının başında, README'lerde, Yardım > Hakkında penceresinde ve `.deb` paketinin `copyright` dosyasında korunur.

Doğrulamak için: `git log` çatal noktasını (`64dcd71`) ve sonrasındaki işlemeleri gösterir; `git diff 64dcd71 HEAD --stat -- src` eklenen ve silinen satırları verir.

Emberframe'in yazarı, çatalın adının "Emberframe" olarak kalmasını rica etti. MIT lisansı adın değiştirilmesine engel değildir ve PisiWorks kendi adıyla yayımlanır; bu rica 8. bölümde açık nokta olarak yer alır.

Depoda Emberframe'den kalan ve onunla aynı lisansı taşıyan dosyalar: `docs/logo.svg`, `docs/ef-logo.png`, `docs/screenshots/` altındaki eski ekran görüntüleri ve `examples/solstice-landing.pisi` örnek belgesi.

### Depoya kopyalanmış tek üçüncü taraf kaynak

`vendor/xim-ctext/` klasörü, Riey'nin MIT lisanslı [xim-rs](https://github.com/Riey/xim-rs) projesindeki `xim-ctext` 0.5.0 kitaplığının kopyasıdır. Arayüz kitaplığı GPUI'nin bağlı olduğu eski sürüm, X giriş yöntemiyle "Ç" ya da "ş" yazıldığında programı çökertiyordu; yeni sürüm bu klasörden kullanılır. Klasördeki `Cargo.toml` kaynağı, yazarı ve nedeni belirtir.

## 4. İçine derlenen kitaplıklar

PisiWorks Rust ile yazılmıştır ve kitaplıkları Cargo ile alır. Linux programına 517 kitaplık derlenir; bunların çoğu arayüz kitaplığı GPUI'nin (Zed Industries, Apache-2.0) dolaylı bağımlılıklarıdır. Tam liste, sürümleri, lisansları ve kaynak adresleriyle [`THIRD-PARTY.md`](THIRD-PARTY.md) dosyasındadır ve `packaging/third-party.py` betiğiyle Cargo'nun kendi verisinden üretilir.

Doğrudan kullanılan başlıca kitaplıklar:

| Kitaplık | Ne için | Lisans |
|---|---|---|
| gpui | Arayüz | Apache-2.0 |
| tiny-skia | Çizim ve dışa aktarma | BSD-3-Clause |
| ab_glyph, fontdb | Yazı tipleri | Apache-2.0, MIT |
| image, png, gif | Görüntü dosyaları | MIT ya da Apache-2.0 |
| flate2 | Sıkıştırma | MIT ya da Apache-2.0 |
| i_overlay | Yol işlemleri | MIT ya da Apache-2.0 |
| serde, serde_json | Belge dosyaları | MIT ya da Apache-2.0 |

Lisanslara göre dağılım:

| Lisans türü | Kitaplık sayısı | Yükümlülük |
|---|---|---|
| MIT, Apache-2.0 ya da ikisinden biri | 440 civarı | Telif ve lisans bildirimini korumak |
| Unicode-3.0 | 18 | Bildirimi korumak |
| BSD-2-Clause, BSD-3-Clause, ISC, Zlib | 20 civarı | Bildirimi korumak |
| Unlicense, CC0-1.0, 0BSD | 10 civarı | Yok (kamu malına bırakılmış) |
| MPL-2.0 | 1 (`option-ext`) | Aşağıda |

Dikkat isteyen üç durum:

- **`option-ext` (MPL-2.0).** Ayar klasörünü bulan `dirs` kitaplığının bağımlılığıdır. MPL-2.0 dosya düzeyinde zayıf copyleft'tir: yalnızca o kitaplığın kendi dosyaları değiştirilirse değişikliklerin kaynağı yayımlanmalıdır. PisiWorks onu değiştirmeden kullanır; kaynağı crates.io'da herkese açıktır. Programın geri kalanına bir yükümlülük getirmez.
- **Seçmeli lisanslar.** `self_cell` (Apache-2.0 ya da GPL-2.0), `r-efi` (MIT, Apache-2.0 ya da LGPL-2.1) gibi birkaç kitaplık lisans seçimi sunar. PisiWorks bunları her zaman izin verici olan seçenekle (MIT ya da Apache-2.0) alır; GPL ya da LGPL koşulları devreye girmez.
- **Apache-2.0 kitaplıkları.** Apache-2.0, lisans metninin ve varsa `NOTICE` dosyasının dağıtımla birlikte verilmesini ister. `.deb` paketi her kitaplığın kendi lisans dosyalarını `/usr/share/doc/pisiworks/third-party-licenses.txt.gz` içinde taşır (bkz. 6. bölüm).

517 kitaplığın 34'ü kaynak paketinde ayrı bir lisans dosyası taşımaz; lisansları yalnızca paket bilgisinde (`Cargo.toml`) yazılıdır. Bunlar listede lisans adlarıyla yer alır.

### Sistemden kullanılanlar

Bunlar PisiWorks'ün içine derlenmez; Pardus'un kendi paketlerinden gelir.

- Çalışma zamanında bağlanan sistem kitaplıkları: `libc6`, `libgcc-s1`, `libxcb1`, `libxkbcommon0`, `libxkbcommon-x11-0`, `libvulkan1`.
- Ayrı program olarak çağrılanlar: yazdırma için CUPS'un `lp` ve `lpstat` komutları, yazım denetimi için `enchant-2` ya da `hunspell` (Türkçe sözlük `hunspell-tr` paketinden), dosya seçme pencereleri için masaüstünün kendi penceresi ya da `zenity`.
- Yazı tipleri: programa font gömülmez; sistemde kurulu fontlar kullanılır.

## 5. Adobe ile ilişki

PisiWorks, Adobe Fireworks'ün; PisiAnimasyon de Adobe Flash'ın çalışma biçimini bilerek örnek alan bağımsız programlardır. İnceleyenin en çok soracağı konu bu olduğu için ne yapıldığını ve ne yapılmadığını ayrı ayrı yazıyoruz.

**Yapılmayanlar:**

- Adobe'nin kaynak kodu, derlenmiş kodu ya da herhangi bir dosyası depoda ve pakette yoktur. Fireworks tersine derlenmedi ve kodu incelenmedi.
- Adobe'nin simgeleri, logoları, desen, doku, degrade, stil ve şablon kitaplıkları, yardım metinleri ve fontları kullanılmadı. PisiWorks'teki desenler, degrade hazır ayarları, şablonlar ve örnek belge ya Emberframe'den gelir ya da PisiWorks için yapılmıştır.
- Programlar "Fireworks" ya da "Flash" adını kendi adları olarak kullanmaz. Ad yalnızca neyin benzeri olduğunu ve hangi dosyaları açabildiğini anlatmak için geçer; her geçtiği yerde Adobe'nin markası olduğu belirtilir.

**Yapılanlar:**

- **Benzer çalışma alanı.** Pencere düzeni, araç grupları, paneller ve menü yapısı Fireworks kullanıcısının tanıyacağı biçimdedir. Bu düzen Emberframe'den gelir.
- **Türkçe terimler.** Kısa menü ve komut adları ("Seçimi Canlandır", "Sembole Dönüştür", "Geçiş Yumuşatma" gibi), öğretmenlerin ders notlarıyla uyuşsun diye Türkçe Fireworks'ün arayüzündeki terimlerle aynı tutuldu. Bunun için geliştirme sırasında Türkçe Fireworks kurulumundaki arayüz metinlerine bakıldı. Alınan şey kısa, işlevsel komut adlarıdır; yardım metni, açıklama ya da belge kopyalanmadı. Çeviri tablosu `src/i18n/tr.rs` dosyasında açıkça görülebilir.

  Örtüşmeyi tahmine bırakmamak için ölçtük (Kasım 2026, `packaging/terim-olcumu.py`). Programın 1488 Türkçe metninden 581'i (yüzde 39) Türkçe Fireworks'ün arayüz metinlerinden biriyle aynıdır:

  | Aynı olan metinler | Sayı |
  |---|---|
  | Tek sözcük ("Dosya", "Katman", "Kırp") | 294 |
  | İki ya da üç sözcük ("Sembole Dönüştür", "Seçim Çerçevesini Genişlet") | 287 |
  | Dört ve daha çok sözcük | 0 |

  Geri kalan 907 metin (yüzde 61) bu proje için yazıldı: iletiler, açıklamalar, ipuçları ve PisiAnimasyon'un, Davranışlar'ın, Kurallar'ın ve yazılı dilin bütün metinleri. İlk ölçümde dört ile altı sözcüklü 16 metin de aynı çıkmıştı ("Seçimi JPEG Maskesi Olarak Kaydet" gibi komut adları ve bir soru cümlesi); bunlar kendi sözlerimizle yeniden yazıldı, böylece aynı kalanların hepsi üç sözcüğü geçmeyen komut adlarıdır. Ölçüm harf büyüklüğüne, kısayol işaretine ve sondaki üç noktaya bakmaz; betik Adobe'ye ait hiçbir metin içermez, ölçen kişinin kendi elindeki dosyayla çalışır.
- **Fireworks PNG dosyalarını okuma.** Fireworks'ün kendi PNG dosyalarındaki katmanlar okunabilir. Biçim Adobe tarafından yayımlanmadığı için okuyucu, Fireworks 8 ile yazılmış sekiz dosyanın yapısı incelenerek, yalnızca birlikte çalışabilirlik amacıyla yazıldı. O dosyalar depoya ya da pakete alınmadı; testlerde kullanılan örnek dosya program tarafından sıfırdan üretilir. Okuyucunun tamamı `src/fwpng.rs` dosyasındadır ve biçim hakkında öğrenilenler dosyanın başında yazılıdır. PisiWorks Fireworks PNG yazmaz.
- **PisiAnimasyon ve Flash.** PisiAnimasyon'un zaman çizelgesi, anahtar kareleri, hareket ve şekil ara doldurması, kılavuz ve maske katmanları, film klibi ve düğme sembolleri Flash'taki karşılıklarının davranışını örnek alır; kısayolları (F5, F6, F7, Enter, Ctrl+Enter) aynıdır. Bunların tamamı bu proje için yazıldı. Geliştirme sırasında elimizde bir Flash kurulumu yoktu: Flash'ın hiçbir dosyasına, arayüz metnine ya da örnek dosyasına bakılmadı. Türkçe terimler ("Zaman Çizelgesi", "Anahtar Kare", "Hareket Ara Doldurması") bizim çevirimizdir. PisiAnimasyon `.fla` ve `.swf` dosyalarını açmaz ve yazmaz; ActionScript çalıştırmaz. Animasyonları kendi `.sahne` biçiminde saklar ve yalnızca HTML5 sayfası olarak yayımlar.
- **Yayımlanan HTML5 sayfaları.** PisiAnimasyon'un ürettiği sayfanın içindeki oynatıcı betik bu proje için yazıldı ve sayfayla birlikte gider. Kullanıcının yayımladığı animasyon kullanıcının kendi eseridir; betik için ayrı bir bildirim ya da lisans koşulu istenmez.
- **Photoshop Canlı Efektleri.** Fireworks'teki bu özelliğin karşılığı vardır; efektlerin adları ve ayarları aynıdır, hesaplamaları PisiWorks için yazılmıştır.

Adobe, Fireworks, Flash, Photoshop ve ActionScript, Adobe Inc.'in ticari markalarıdır.

## 6. Dağıtımda verilen bildirimler

`packaging/build-deb.sh` ile üretilen `.deb` paketi şunları `/usr/share/doc/pisiworks/` altına koyar:

| Dosya | İçeriği |
|---|---|
| `copyright` | Debian biçiminde telif ve lisans: iki telif sahibi, GPL-3+ bildirimi, Emberframe'in MIT (Expat) metni ve atfı |
| `NOTICE` | Kısa lisans bildirimi, Emberframe açıklaması ve yapılan filmlere ilişkin ek izin |
| `THIRD-PARTY.md` | 517 kitaplığın adı, sürümü, lisansı ve kaynak adresi |
| `third-party-licenses.txt.gz` | Her kitaplığın kendi `LICENSE`, `COPYING` ve `NOTICE` dosyaları, yazarlarının yayımladığı haliyle |
| `lisans-ve-telif.md` | Bu belge |

Paket iki programı da kurar: `/usr/bin/pisiworks` ve `/usr/bin/pisianimasyon`, her birinin kendi menü girdisi, simgesi ve dosya türü (`.pisi`, `.sahne`) ile.

Program içinde Yardım > Hakkında penceresi geliştireni, depo adresini, Emberframe'e teşekkürü ve lisansı gösterir.

## 7. Nasıl doğrulanır

```bash
# Kitaplık listesini Cargo'nun verisinden yeniden üretir
python3 packaging/third-party.py

# Depoda Adobe'ye ait dosya olmadığını gösterir: izlenen bütün görsel ve font dosyaları
git ls-files | grep -iE '\.(png|jpg|svg|gif|ttf|otf|dll)$'

# Çataldan bu yana yazılanları gösterir
git diff 64dcd71 HEAD --stat -- src

# Türkçe metinlerin başka bir programın arayüz metinleriyle örtüşmesini ölçer.
# Karşılaştırılacak metinler ölçen kişinin kendi dosyasından gelir; depoda yoktur.
python3 packaging/terim-olcumu.py KARSILASTIRILACAK_METINLER.txt --liste
```

Kaynak koddan derleme ve testler README'de anlatılır; 349 otomatik test her derlemede çalışır; dördü yalnızca elle istenince çalışır (biri elde gerçek bir Fireworks dosyası varsa).

## 8. İnceleyenin görüşüne bırakılan noktalar

Bunlar bizim tek başımıza karar veremediğimiz ya da bir hukukçunun bakmasını doğru bulduğumuz konulardır. Gizlemek yerine buraya yazıyoruz.

1. **Görünüm benzerliği.** PisiWorks'ün çalışma alanı Fireworks'e, PisiAnimasyon'unki Flash'ın klasik çalışma alanına (zaman çizelgesi üstte, araçlar solda, Özellikler altta) bilerek benzer. PisiAnimasyon, PisiWorks'ün renklerini ve simgelerini kullanır; Flash'ın görsellerinden hiçbiri alınmadı. Kod ve görseller Adobe'ye ait değildir, ancak bir programın genel görünümünün ne ölçüde korunduğu ülkeden ülkeye değişen bir hukuk konusudur. Fireworks 2013'ten beri geliştirilmiyor ve satılmıyor; bu durumu değiştirmez ama riski değerlendirirken bilinmelidir.
2. **Sol üstteki "Pw" ve "Pa" işaretleri.** Uygulama çubuğunun başındaki iki harfli işaretler Emberframe'in "Ef" işaretinden uyarlandı ve Adobe ürünlerinin iki harfli işaretlerini andırır. PisiAnimasyon'un işareti "Pa"dır; Adobe Animate'in kullandığı "An" ile karışmasın diye o seçilmedi. Uygulama simgesi bu yüzden daha önce değiştirildi (kedi başı); pencere içindeki işaret duruyor. Değiştirilmesi istenirse küçük bir iştir.
3. **Türkçe terimlerin kaynağı.** Kısa komut adlarının 581'i Türkçe Fireworks'tekilerle aynıdır; hepsi bir ile üç sözcük arasındadır (ölçüm 5. bölümde). Tek tek komut adlarının telifle korunmadığını düşünüyoruz. Değerlendirilmesini istediğimiz soru şudur: bu adların tek tek değil, toplu olarak aynı olması bir derleme hakkı doğurur mu? Doğurduğu sonucuna varılırsa adlar değiştirilebilir; bunun bedeli, öğretmenlerin ders notlarıyla programın uyuşmamasıdır.
4. **Fireworks PNG biçiminin incelenmesi.** Okuyucu, yasal olarak edinilmiş bir Fireworks kurulumunun örnek dosyaları incelenerek, birlikte çalışabilirlik için yazıldı. 5846 sayılı Kanun'un bilgisayar programlarına ilişkin hükümlerinin bu tür incelemeye ne ölçüde izin verdiği teyit edilmelidir. Gerekirse okuyucu tek dosya olduğu için kolayca çıkarılabilir.
5. **"Pisi" ve "PisiAnimasyon" adları.** PisiAnimasyon adı bu projede konuldu; başka bir ürünle çakışıp çakışmadığı araştırılmadı.  Ad, Pardus geleneğine bir göndermedir. "PiSi" Pardus'un eski paket yöneticisinin adıdır ve "Pisi Linux" adında ayrı bir dağıtım vardır. Karışıklık ya da bir marka hakkı doğurup doğurmadığına Pardus ekibi karar vermelidir.
6. **Emberframe yazarının ad ricası.** Yazar, çatalın "Emberframe" adını korumasını rica etti. Lisans buna zorlamaz; PisiWorks her yerde kaynağını ve yazarını belirtir. Kendisine Kasım 2026'da yazılarak adın neden değiştirildiği anlatıldı.
7. **Telif satırındaki kişi.** PisiWorks için yazılanların telifi şimdilik Murat SAPMAZ adınadır. Proje bir kurum çatısı altına alınacaksa telif sahipliği ve katkı kuralları buna göre düzenlenmelidir.
8. **Kitaplık sayısı.** 517 kitaplığın lisansları Cargo'nun bildirdiği bilgilere dayanır; her birinin kaynak kodu tek tek gözle denetlenmedi. Pardus depoları için gerekiyorsa bağımlılıkların ayrıca denetlenmesi ya da Debian'da paketlenmiş sürümlerine geçilmesi ayrı bir çalışmadır.
9. **Garanti ve sorumluluk.** GPL programı garantisiz sunar. Okullarda kullanım için ayrı bir destek ya da sorumluluk taahhüdü yoktur.
10. **Bakılan Fireworks kurulumunun lisans koşulları.** Terimler karşılaştırılırken ve PNG biçimi incelenirken bir Fireworks kurulumunun dosyalarına bakıldı. O kurulumun son kullanıcı lisans sözleşmesinin arayüz metinlerine ve örnek dosyalara bu amaçla bakmayı kısıtlayıp kısıtlamadığı incelenmedi. Bu telif değil sözleşme konusudur ve kurulumun sahibini ilgilendirir; yine de inceleyenin bilmesi için yazıyoruz. O dosyalar depoya ve pakete hiç girmedi.
11. **Ticari amacın bulunmaması.** Proje gelir amacı taşımaz (bkz. Amaç). Bunun hukuki değerlendirmede ne ölçüde ağırlık taşıdığı, özellikle eğitim amaçlı kullanım ve marka hukuku bakımından, inceleyenin takdirindedir; biz bunu bir savunma olarak değil, durumun doğru anlaşılması için yazıyoruz.
12. **Emberframe yazarına bildirim.** Lisans değişikliği Emberframe'in yazarının iznini gerektirmez: onun kodu MIT altında kalır ve MIT buna izin verir. Yine de kendisine haber verilmesi doğru olur; ayrıca PisiWorks için yazılanları kendi MIT projesine geri almak isterse bunun için ayrı bir izin gerekir. Kendisine Kasım 2026'da yazılarak lisans değişikliği ve kendi kodunun MIT altında kaldığı bildirildi.
13. **İşveren ya da okul feragatnamesi.** Pardus'un gönüllüler için yayımladığı yönerge, yazarın işvereninden ya da okulundan telif feragatnamesi almasını önerir. Yazar, bu çalışma için kimseden feragatname alması gerekmediğini bildirmiştir (Kasım 2026); çalışma kendi adına, kendi imkânlarıyla yapılmıştır. Bu belgeyi hazırlarken bu ayrıca incelenmedi.
14. **Ek iznin sözleri.** Yapılan filmlere ilişkin ek izin bir hukukçu tarafından yazılmadı; GCC'nin çalışma zamanı kitaplığı istisnası gibi bilinen örneklerin amacını izler. Sözlerinin bir hukukçuya gösterilmesi yerinde olur.
15. **Kitabın lisansı.** `docs/kitap/` altındaki kitap için ayrı bir lisans belirlenmedi; deponun parçası olarak aynı lisansın altındadır. Ders kitabı gibi çoğaltılıp dağıtılacaksa Creative Commons türü bir belge lisansı daha uygun olabilir.
16. **Adların tescili.** "PisiWorks" ve "PisiAnimasyon" adları için marka başvurusu yapılmadı. Lisanstaki ek koşul (7e) yalnızca "bu lisans ad üzerinde hak vermez" der; adı üçüncü kişilere karşı gerçekten koruyan şey marka tescilidir. Ayrıca "Pisi" sözcüğünün Pardus geleneğindeki yeri (bkz. 5. madde) başvuruda göz önüne alınmalıdır.
17. **Ek koşulların sözleri.** Atıf, köken ve adlara ilişkin ek koşullar bir hukukçu tarafından yazılmadı; GPL'nin 7. maddesinin (b), (c) ve (e) bentlerinin izin verdiği sınırlar içinde, bilinen örnekler izlenerek yazıldı. "Değiştirilmiş sürüm kendi adını taşır" koşulunun (7c) kapsamında sayılıp sayılmayacağı bir hukukçuya sorulmalıdır.
18. **Bugüne kadarki sürümler.** 16 Kasım 2026'dan önceki sürümler MIT ile, ek koşulsuz yayımlandı. O sürümleri almış olan biri için bu koşullar geçerli değildir.

## 9. İletişim

Sorular, düzeltmeler ve lisansla ilgili talepler için: [github.com/MuratSAPMAZ/PisiWorks/issues](https://github.com/MuratSAPMAZ/PisiWorks/issues)
