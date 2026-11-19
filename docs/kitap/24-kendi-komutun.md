@kisim 5. Kuralları yazmak
# Kendi komutun
> Aynı beş satırı on kuralda yeniden yazmak yerine onlara bir ad verirsin ve adıyla çağırırsın. PisiAnimasyon'a yeni bir sözcük öğretmiş olursun.
:: ⏱ 30 dakika | 🎯 23. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![İki komut tanımı ve onları kullanan bir kural.](24-komut.png "Kuralları Yaz penceresi: Kutla komutu, Döndür komutu ve Kutu nesnesine tıklanınca kuralı")

!goal Bu bölümde öğreneceklerin
Bir **komut** tanımlamayı, onu kurallardan çağırmayı ve komuta **değer** vermeyi öğreneceksin. Bununla birlikte PisiAnimasyon'un yazı dilinin tamamını görmüş olacaksın.

## Sorun: aynı satırlar her yerde

Bir oyunda puan kazandıran beş ayrı şey olsun. Her birinde aynı üç şey yapılıyor: puan artır, ses çal, parıltı çıkar. Beş kuralda üçer satır: on beş satır. Sesi değiştirmek istersen beş yeri düzeltirsin.

## Çözüm: komut

```
Kutla komutu:
    Kutu nesnesini bir an yanıp söndür
    Puan sayısını 1 artır
```

Bu, PisiAnimasyon'a **Kutla** adında yeni bir sözcük öğretir. Artık herhangi bir kuralda şunu yazabilirsin:

```
Kutu nesnesine tıklanınca:
    Kutla komutunu yap
```

`Kutla komutunu yap` satırı, komutun içindeki iki satırın yerine geçer. Komutu değiştirirsen onu kullanan bütün kurallar değişir.

| Yazım | Anlamı |
| `Kutla komutu:` | Tanım. Kenardan başlar; altına, içeriden, yapılacaklar yazılır. |
| `Kutla komutunu yap` | Çağrı. Bir eylemdir; kuralların içinde kullanılır. |

## Komuta değer vermek

Bazen komutun her seferinde biraz farklı çalışmasını istersin: bir kez kutuyu döndürsün, bir kez yıldızı. Bunun için komut **değer** alır.

```
Döndür komutu, Kim, Kaç ile:
    Kaç kez yinele:
        Kim nesnesini 45 derece döndür
        0.1 saniye bekle
```

Bu komut iki değer alır: **Kim** (hangi nesne) ve **Kaç** (kaç kez). Tanımın içinde bu adları normal bir nesne adı ve sayı gibi kullanırsın.

Çağırırken değerleri verirsin:

```
Kutu nesnesine tıklanınca:
    Döndür komutunu Kutu, 8 ile yap
```

Burada **Kim** yerine *Kutu*, **Kaç** yerine *8* geçer: kutu sekiz adımda tam bir tur atar. Aynı komutu `Döndür komutunu Yıldız, 4 ile yap` diye de çağırabilirsin.

| Değer ne olabilir | Örnek |
| Bir sayı | `8` |
| Bir işlem | `Zar * 2` |
| Bir nesnenin adı | `Kutu` |

!why Değerlerin adları ne olmalı?
Sen seçersin; ne işe yaradığını söyleyen adlar seç. `Kim`, `Kaç`, `Hız`, `Nereye` iyi adlardır. `A`, `B`, `X` de çalışır; ama bir hafta sonra neyin ne olduğunu hatırlamazsın.

## Adım adım

1. Yeni bir filmde **Kutu** adında bir kutu çiz; sahneye `Puan: {Puan}` yaz.
2. {{Kuralları Yaz…}}'ı aç ve bölümün başındaki resimdeki metni yaz.
3. {{Uygula ve Test Et}}. Kutuya tıkla: yanıp söner, puan artar, sonra sekiz adımda döner.
4. Sahneye bir yıldız ekle, adını **Yıldız** koy ve şu kuralı yaz:

```
Yıldız nesnesine tıklanınca:
    Döndür komutunu Yıldız, 4 ile yap
```

Yeni bir şey tanımlamadın; var olan komutu başka değerlerle kullandın.

![Kutuya tıkla: önce yanıp söner, sonra sekiz adımda tam bir tur atar.](24-film.png "Oynayan film: mavi kutu" 660){24-komut.html 660x500}

## Metnin düzeni

Uzun bir metinde şu sıra okumayı kolaylaştırır:

```
Sayılar:
    …

… komutu:
    …

… olunca:
    …
```

Önce sayılar, sonra komutlar, en son kurallar. Zorunlu değildir; ama herkes aynı sırayı kullanırsa birbirinin metnini kolay okur.

!stuck Takıldıysan
- **"… adında bir komut yok" diyor.** Çağırdığın ad ile tanımdaki ad aynı mı? Büyük-küçük harf dahil.
- **"Komut … değer alır, … verilmiş" diyor.** Tanımda kaç değer varsa çağrıda da o kadar olmalı.
- **Komut kendini çağırıyor.** Bir komut kendi içinde kendini çağırabilir; ama bir yerde durması gerekir. PisiAnimasyon çok derine inen çağrıları keser.

## Dilin tamamı

Bu bölümle birlikte yazı dilinin her parçasını gördün:

| Parça | Bölüm |
| Olaylar ve eylemler | 10, 21 |
| Sayılar ve `{Ad}` yazıları | 11 |
| Koşullu kural | 11, 17 |
| İşlemler, `rastgele`, `Ad.x` | 22 |
| `eğer`, `değilse`, `yinele`, `bekle` | 23 |
| Komutlar ve değerler | bu bölüm |

Vitrin oyunu **Pisi: Demir Pençe** ({{Örnekler}} menüsünün ilk satırı) bunların hepsini kullanır: altı sahne, yirmi sekiz kural. Aç, oyna, sonra Kuralları Yaz'da metnini oku. Artık her satırını anlayabilirsin.

## Sıra sende

- [ ] **Kutla** komutuna bir ses ekle. Kaç kuralı değiştirmen gerekti?
- [ ] `Işınla komutu, Kim ile:` diye bir komut yaz: verilen nesneyi rastgele bir yere koysun ve yanıp söndürsün.
- [ ] `Say komutu, Kaçtan ile:` diye bir komut yaz: verilen sayıdan geriye doğru saysın (23. bölümdeki sayaç).
- [ ] Zor soru: 17. bölümdeki platform oyununda kahraman hem düşmana değince hem aşağı düşünce can kaybediyor olsun. İki kuralın ortak kısmını bir komuta taşı.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Komut | Adı olan bir eylemler dizisi; kurallardan adıyla çağrılır. |
| Değer | Komuta çağrılırken verilen sayı ya da nesne adı. |
| Tanım / çağrı | `… komutu:` tanımlar; `… komutunu yap` çağırır. |

!teacher Öğretmene not
Komut, programlamadaki "fonksiyon" ya da "yordam" kavramıdır; değerler de "parametre". Bu sözcükleri kullanmak ya da kullanmamak size kalmış. Kavramın gerekçesi "tekrarı azaltmak ve tek yerden değiştirmek"tir; **Kutla** komutuna ses ekleme alıştırması bunu somut olarak gösterir. Bu bölüm kitabın en soyut bölümüdür ve her sınıf için uygun olmayabilir; 3. ve 4. kısımlar bu bölüm olmadan da eksiksiz oyunlar yaptırır. İlerleyen öğrencileri Pisi: Demir Pençe'nin metnini okumaya yönlendirin: gerçek büyüklükte bir programı okuyabilmek başlı başına bir kazanımdır.
