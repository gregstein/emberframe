@kisim 3. Oyun kuralları
# Değince ve kenara gelince
> Oyunların çoğu iki şeyin birbirine değmesiyle ilgilidir: sepet elmayı yakalar, top duvara çarpar, kahraman düşmana dokunur. Bu bölümde değmeyi ve sahnenin kenarını olay olarak kullanacaksın.
:: ⏱ 25 dakika | 🎯 11. bölümü bitirenler için | 🧰 Örnekler > Oyun: elma yakala

![Bu bölümde kurallarını okuyacağın oyun.](15-film.png "Elma yakala oyunu" 640){15-elma.html 640x400}

!goal Bu bölümde öğreneceklerin
**Bir nesne ötekine değince** ve **bir nesne sahnenin kenarına değince** olaylarını, bir nesneyi yeniden yukarı göndermeyi ve hazır bir oyunun kurallarını okumayı öğreneceksin.

## Önce bir oyunu oku

Yeni bir şey yazmadan önce, çalışan bir oyunun kurallarına bakalım.

1. {{Örnekler > Oyun: elma yakala}}'yı aç.
2. {{Filmi Test Et}} ile bir dakika oyna: ok tuşlarıyla sepeti yürüt, elmaları yakala.
3. Programa dön ve {{Kurallar > Kurallar ve Sayılar…}}'ı aç.

![Bütün oyun beş cümleden ibaret. İşaretli satır oyunun kalbidir.](12-kurallar.png "Elma yakala oyununun beş kuralı; ikinci kural işaretli")

Kuralları sırayla oku:

| Kural | Anlamı |
| Film başlayınca: Artı nesnesini gizle | "+1" yazısı başta görünmesin. |
| Elma nesnesi Sepet nesnesine değince: … | Elma yakalandı: puan ver. |
| Elma nesnesi sahnenin alt kenarına değince: … | Elma yere düştü: can al. |
| Can sayısı 1 değerinin altına inince: 2. sahneye git | Can bitti: oyun bitti. |
| Yeniden nesnesine tıklanınca: baştan başla | "Yeniden oyna" düğmesi. |

## Değme olayı

Listede ikinci kurala tıkla. Altında parçaları açılır.

![((1)) Olay: hangi nesne hangisine değince. ((2)) Eylemler: sırayla beş şey yapılır.](12-degince.png "Değme kuralının parçaları: olay satırında Elma ve Sepet; beş eylem")

Olay satırında iki nesne var: **Elma** ve **Sepet**. Eylemler sırayla:

1. **Puan** sayısını 1 artır.
2. **Artı** nesnesinin bir kopyasını sepetin yerinde oluştur (yükselip kaybolan "+1"; kopyaları bir sonraki bölümde öğreneceğiz).
3. Elmayı **üstte rastgele bir yere koy**: aynı elma yeniden düşmeye başlar.
4. Elmayı yüzde 4 **hızlandır**: oyun giderek zorlaşır.
5. **çın** sesini çal.

!why Hangi elma?
Sahnede üç elma var ve üçünün adı da "Elma". Kural "Elma" der; ama eylemler yalnızca **sepete değen** elmaya uygulanır, öbür ikisine değil. PisiAnimasyon olaya karışan nesneyi aklında tutar. Bu yüzden bir kural, aynı adı taşıyan bütün nesneler için yeter.

## Kenar olayı

Üçüncü kural: *Elma nesnesi sahnenin alt kenarına değince: Can sayısını 1 azalt; Elma nesnesini üstte rastgele bir yere koy; eyvah sesini çal.*

Kenar olayında nesneyi ve kenarı seçersin:

| Kenar | Nerede kullanılır |
| alt kenar | Yere düşen elma, kaçan top |
| üst kenar | Ekrandan çıkan mermi |
| sol kenar, sağ kenar | Kaleye giren top, sahneden çıkan düşman |
| herhangi bir kenar | Sınırı aşan her şey |

## Kendin yap: top ve raket

Öğrendiklerinle klasik top-raket oyununu kur.

1. Yeni bir filmde altta yatık bir dikdörtgen çiz, adını **Raket** koy. Ona **Ok Tuşlarıyla Yürü** ve **Katı** ver.
2. Küçük bir kare çiz, adını **Top** koy. Ona **Kenarlardan Sek** ver.
3. **Puan** adında bir sayı ekle (başlangıcı 0) ve `Puan: {Puan}` yazısını koy.
4. Şu iki kuralı kur:

| Olay | Eylem |
| Top nesnesi Raket nesnesine değince | Puan sayısını 1 artır |
| Top nesnesi sahnenin alt kenarına değince | Puan sayısını 0 yap ({{Sayıyı ayarla}}) |

5. {{Filmi Test Et}}. Top raketten sekiyor ve her sekişte puan veriyor; kaçırırsan puan sıfırlanıyor.

!why Değmek için Katı gerekmez
Değme olayı, nesnelerin katı olup olmadığına bakmaz: iki şeklin kutuları üst üste gelince olur. **Katı**, topun raketten *sekmesi* için; **değince** kuralı ise puanı *saymak* için. İkisi ayrı işler.

!stuck Takıldıysan
- **Değme kuralı hiç çalışmıyor.** İki nesnenin de adı olmalı ve adlar kuraldakilerle aynı yazılmalı.
- **Kural bir kez çalışıyor, sonra çalışmıyor.** Değme, nesneler birbirine *değmeye başladığı anda* olur. Nesneler ayrılmadan yeniden olmaz. Elmayı yukarı göndermek ya da gizlemek bu yüzden gerekir.
- **Top alttan çıkıp gitmiyor, geri sekiyor.** Kenarlardan Sek alt kenardan da sektirir; kural yine de çalışır, çünkü top kenara değmiştir.

## Sıra sende

- [ ] Elma yakala oyununda başlangıç canını 5'ten 3'e indir.
- [ ] Aynı oyunda elmanın her yakalanışta yüzde 4 yerine yüzde 10 hızlanmasını sağla. Kaç elmadan sonra yetişemez olursun?
- [ ] Top-raket oyununa bir **Can** sayısı ekle: top alt kenara değince can azalsın.
- [ ] Zor soru: top-raket oyununa üstte duran üç "Tuğla" ekle. Top bir tuğlaya değince o tuğla kaybolsun ve 5 puan versin. Kaç kural gerekir?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Değince | İki nesnenin birbirine dokunmaya başladığı an. |
| Kenara değince | Nesnenin sahnenin bir kenarına ulaştığı an. |
| Üstte rastgele bir yere koy | Nesneyi sahnenin üstünde, şansa bağlı bir yere gönderen eylem. |

!teacher Öğretmene not
Hazır bir oyunun kurallarını okutmak, sıfırdan kural yazdırmaktan önce gelmelidir: öğrenci beş cümlenin bir oyun ettiğini görünce kendi oyununu kurmaya cesaret eder. "Hangi elma?" kutusu önemli bir kavramı anlatır: kural bir *tür* nesne hakkındadır, ama o anda olaya karışan tek nesneye uygulanır. Top-raket alıştırması bir ders saatini doldurur ve tuğlalı sürümü (son soru) bir sonraki dersin ısınması olabilir; yanıt tek kuraldır, çünkü üç tuğlanın adı aynıdır.
