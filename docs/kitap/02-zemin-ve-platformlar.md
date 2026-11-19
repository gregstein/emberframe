@kisim 1. Başlarken
# Zemin, platformlar ve seken top
> Kutunun üstüne çıkabileceği platformlar ve sahnede durmadan seken bir top ekleyeceksin. Bunun için tek bir yeni davranış yeter: Katı.
:: ⏱ 15 dakika | 🎯 1. bölümü bitirenler için | 🧰 Yalnızca PisiAnimasyon

![Bölümün sonunda: kutu platformun üstünde duruyor, kırmızı top duvarlardan ve platformlardan sekiyor.](02-sonuc.png "Tarayıcıda oynayan film: mavi kutu turuncu platformun üstünde, kırmızı kare havada" 634){02-platformlar.html 660x500}

!goal Bu bölümde öğreneceklerin
Bir şekli **Katı** yaparak zemin ve platform kurmayı, **Kenarlardan Sek** ile kendi kendine dolaşan bir top yapmayı ve bir davranışın sayılarını değiştirmeyi öğreneceksin.

## Hazırlık

Yeni bir film aç: {{Dosya > Yeni}} ([[Ctrl]]+[[N]]). 1. bölümdeki gibi mavi bir kutu çiz ve ona **Ok Tuşlarıyla Yürü** ile **Düş ve Zıpla** ver. Bunu artık biliyorsun; takılırsan 1. bölüme bak.

## Adım adım

### 1. İlk platformu çiz

1. {{Dikdörtgen}} aracını al ve kutunun sağına, biraz yukarıya, yatık ince bir dikdörtgen çiz ((1)).
2. {{Stiller}} panelinde turuncu kareye tıkla ((2)).

![Platform çizildi ve turuncuya boyandı. Henüz yalnızca bir resim: kutu içinden geçer.](02-platform.png "Sahnede mavi kutu ve turuncu ince dikdörtgen; Stiller panelinde turuncu işaretli")

### 2. Platformu katı yap

1. Platform seçiliyken {{Davranışlar}} menüsünü aç.
2. Listenin altındaki {{Katı}}'ya tıkla.

![Katı, davranışlar listesinin sonundadır.](02-kati.png "Davranışlar menüsünde Katı işaretli" 600)

!why Ne oldu?
**Katı** olan bir şeklin içinden yürüyen ya da düşen hiçbir şey geçemez. Kutu ona yandan çarparsa durur, üstüne düşerse üstünde kalır, altından zıplarsa başını çarpar. Zemin, platform, duvar, kapı: hepsi Katı verilmiş birer dikdörtgendir.

### 3. İkinci platformu ve topu ekle

1. Aynı yolla, daha yukarıya ikinci bir platform çiz, turuncuya boya ve ona da **Katı** ver.
2. Sahnenin üst tarafına küçük bir kare çiz ve kırmızıya boya. Bu bizim topumuz olacak.
3. Kırmızı kare seçiliyken {{Davranışlar > Kenarlardan Sek}}'e tıkla.

![Sahnenin son hâli: bir kutu, iki platform, bir top.](02-sahne.png "Sahnede mavi kutu, iki turuncu platform ve kırmızı küçük kare")

!why Ne oldu?
**Kenarlardan Sek** verilen şekil dümdüz uçar; sahnenin kenarına ya da katı bir şeye çarpınca top gibi geri seker. Hiç durmaz.

### 4. Dene

1. {{Filmi Test Et}}'e tıkla ([[Ctrl]]+[[Enter]]).
2. Kutuyu platformun yanına yürüt, zıpla ve havadayken platformun üstüne doğru yürü.
3. Topu izle: platformlara çarpınca yön değiştiriyor mu?

!stuck Takıldıysan
- **Kutu platforma yetişemiyor.** Platform çok yüksekte. Ya platformu aşağı sürükle ya da kutuyu daha yükseğe zıplat: kutuyu seç, {{Davranışlar > Davranış Ayarları…}}'nı aç, **Düş ve Zıpla**'nın "zıplama hızı" kutusundaki sayıyı büyüt (örneğin 420 yerine 600) ve [[Enter]]'a bas.
- **Kutu platformun altından zıplayınca başını çarpıyor.** Bu doğru; katı şeyler her yönden katıdır. Platforma yandan zıplayarak çık.
- **Kutu platformun içinden geçiyor.** Platformu seçip Davranışlar menüsüne bak: **Katı**'nın yanında ✓ yoksa davranış verilmemiştir.

## Bir davranışın sayıları

Çoğu davranışın bir iki sayısı vardır: hız, yükseklik, süre gibi. Bunları {{Davranışlar > Davranış Ayarları…}} penceresinde değiştirirsin. Pencere seçili şeklin bütün davranışlarını, her birinin sayılarını ve yanında bir **Kaldır** düğmesini gösterir.

| Davranış | Sayıları | Dene |
| Ok Tuşlarıyla Yürü | Hız | 160 yerine 300 yaz: kutu koşar. |
| Düş ve Zıpla | Çekim, zıplama hızı | Çekimi 300 yap: kutu ayda gibi süzülür. |
| Kenarlardan Sek | Hız, yön | Hızı 400 yap: top çıldırır. |

## Sıra sende

- [ ] Sahnenin en altına, bir uçtan öbür uca uzanan bir zemin çiz ve **Katı** yap. Fark eder mi? (İpucu: kutu zaten sahnenin altına düşünce duruyor.)
- [ ] Üç basamaklı bir merdiven kur. Kutu en üste çıkabiliyor mu?
- [ ] İkinci bir top ekle ve Davranış Ayarları'nda yönünü 120 yap. İki top farklı yönlere gidiyor mu?
- [ ] Zor soru: topun içinde dolaşacağı kapalı bir oda yapabilir misin? Dört duvar gerekir.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Katı | İçinden geçilemeyen şekil: zemin, platform, duvar. |
| Kenarlardan Sek | Kenarlara ve katı şeylere çarpınca geri seken, hiç durmayan hareket. |
| Davranış Ayarları | Seçili şeklin davranışlarının sayılarını değiştirdiğin pencere. |

!teacher Öğretmene not
Bu bölüm bir ders saatinin ikinci yarısına sığar. Öğrencilerin en sık yaptığı hata platformu çizip Katı vermeyi unutmaktır; "içinden geçiyor" diyen öğrenciye menüdeki ✓ işaretini gösterin. Platformun çok yükseğe çizilmesi ikinci yaygın sorundur ve iyi bir fırsattır: öğrenci ya platformu taşır ya da zıplama hızını artırır, ikisi de doğru çözümdür. "Sıra sende"nin son maddesi iki öğrencinin birlikte çalışmasına uygundur.
