@kisim 3. Oyun kuralları
# Sahneler: başla, oyna, bitti
> Gerçek bir oyunun birden çok ekranı vardır: bir giriş ekranı, oyunun kendisi, bir "oyun bitti" ekranı. PisiAnimasyon'da her ekran bir sahnedir.
:: ⏱ 25 dakika | 🎯 12. bölümü bitirenler için | 🧰 Örnekler > Oyun: bilgi yarışması

![Bilgi yarışmasının ilk sahnesi. Her soru ayrı bir sahne; son sahne sonucu gösterir.](14-soru.png "Bilgi yarışması: Örümceğin kaç bacağı vardır sorusu ve üç yanıt düğmesi" 670){19-yarisma.html 640x400}

!goal Bu bölümde öğreneceklerin
Sahne eklemeyi, bir sahnede beklemeyi (**Durdur**), kuralla sahne değiştirmeyi ve oyunu baştan başlatmayı öğreneceksin. Bu bölümden sonra başı ve sonu olan tam bir oyun yapabilirsin.

## Sahne nedir?

Bir **sahne**, kendi zaman çizelgesi ve kendi çizimleri olan ayrı bir ekrandır. Film sahneleri sırayla oynatır. Sağdaki **Sahneler** paneli hepsini listeler; birine tıklayınca o sahneye geçersin.

Dikkat edilecek iki şey:

- **Sayılar bütün sahnelerde aynıdır.** Birinci sahnede kazandığın puan son sahnede de durur.
- **Kurallar bütün film içindir.** "Doğru nesnesine tıklanınca" kuralı, adı "Doğru" olan nesne hangi sahnedeyse orada çalışır.

## Örneği incele

1. {{Örnekler > Oyun: bilgi yarışması}}'nı aç.
2. Sahneler panelinde sırayla her sahneye tıkla ((1)): üç soru ve bir sonuç sahnesi.

![((1)) Sahneler paneli: dört sahne. Seçili olan son sahne, "Aferin" ekranı. ((2)) Kare eylemi **Durdur**: film bu sahnede bekler.](14-sahneler.png "Bilgi yarışmasının son sahnesi; Sahneler paneli ve Kare eylemi kutusu işaretli")

### Film neden kendi kendine sonraki sahneye geçmiyor?

Normalde film bir sahne bitince sonrakine geçer. Soru ekranının beklemesi gerekir. Bunun için sahnenin karesine **Durdur** eylemi verilmiştir ((2)): zaman çizelgesinin altındaki {{Kare eylemi}} kutusunda **Durdur** seçilidir. Film o kareye gelince durur ve bir kural onu başka yere gönderene kadar bekler.

### Kurallar

3. {{Kurallar > Kurallar ve Sayılar…}}'ı aç.

![Bütün yarışma üç kuralla çalışır.](14-kurallar.png "Bilgi yarışmasının üç kuralı; birinci kural işaretli")

| Kural | Anlamı |
| Doğru nesnesine tıklanınca: Puan sayısını 1 artır; çın sesini çal; sonraki sahneye git | Doğru yanıt: puan ver, ilerle. |
| Yanlış nesnesine tıklanınca: eyvah sesini çal; sonraki sahneye git | Yanlış yanıt: puansız ilerle. |
| Yeniden nesnesine tıklanınca: baştan başla | Sonuç ekranındaki düğme. |

!why Üç soru, üç kural değil
Her sahnede doğru yanıtın kutusunun adı **Doğru**, öbür ikisinin adı **Yanlış**. Bu yüzden üç soru için ayrı ayrı kural gerekmez: iki kural bütün sorulara yeter. Yüz soruluk bir yarışma da aynı iki kuralla çalışır.

## Sahne değiştiren eylemler

| Eylem | Ne yapar | Nerede kullanılır |
| Sonraki sahneye git | Sıradaki sahneye geçer | Sonraki soru, sonraki bölüm |
| Sahneye git | Numarasını verdiğin sahneye geçer | Can bitince "oyun bitti" sahnesi |
| Baştan başla | İlk sahneye döner; bütün sayılar başlangıcına gelir | "Yeniden oyna" düğmesi |
| Her şeyi durdur | Filmi olduğu yerde dondurur | Süre dolunca |

## Kendin yap: başı ve sonu olan oyun

12. bölümdeki top-raket oyununa bir giriş ve bir bitiş ekle.

1. Top-raket filmini aç. **Can** adında, başlangıcı 3 olan bir sayın olsun.
2. Sahneler panelinin altındaki **yeni sahne** simgesine tıkla. İkinci bir sahne açılır.
3. Bu sahneye büyük bir yazı yaz: **Oyun bitti**. Altına `Puan: {Puan}` yaz.
4. Bir dikdörtgen çiz, adını **Yeniden** koy; üstüne "Yeniden oyna" yaz.
5. Zaman çizelgesinin altındaki {{Kare eylemi}} kutusundan **Durdur**'u seç. Bunu birinci sahne için de yap.
6. Şu iki kuralı ekle:

| Olay | Eylem |
| Can sayısı 1 değerinin altına inince | Sahneye git: 2 |
| Yeniden nesnesine tıklanınca | Baştan başla |

7. {{Filmi Test Et}}: üç top kaçırınca bitiş sahnesi gelmeli, düğme oyunu yeniden başlatmalı.

!stuck Takıldıysan
- **Film sahneler arasında durmadan dönüyor.** Sahnelerde **Durdur** kare eylemi yok. Her sahnede Kare eylemi kutusundan Durdur'u seç.
- **Bitiş sahnesinde puan 0 görünüyor.** `{Puan}` adını yanlış yazmış olabilirsin; ya da "Baştan başla" fazladan çalışmıştır.
- **"Yeniden" düğmesi çalışmıyor.** Adı dikdörtgene verdiğinden ve kuraldaki adla aynı yazdığından emin ol. Üstündeki yazıya ayrıca bir davranış verdiysen tıklamayı o alır: ikisini grupla ve adı gruba ver.

## Sıra sende

- [ ] Bilgi yarışmasına dördüncü bir soru ekle: bir soru sahnesini seç, sahneyi çoğalt ya da yeniden çiz, soruyu ve yanıtları değiştir. Kutu adlarını (Doğru, Yanlış) doğru vermeyi unutma.
- [ ] Top-raket oyununa bir giriş sahnesi ekle: oyunun adı ve "Başla" düğmesi. Düğmeye tıklanınca sonraki sahneye geçsin.
- [ ] Bitiş sahnesindeki yazı puana göre değişsin: puan 10'u geçmişse "Harika!" göster. (İpucu: iki yazı, biri başta gizli, bir koşullu kural.)
- [ ] Zor soru: iki bölümlü bir oyun yap. Puan 5 olunca ikinci bölüme geçilsin ve ikinci bölümde top daha hızlı olsun.

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Sahne | Kendi çizimleri ve zaman çizelgesi olan ayrı ekran. |
| Durdur (kare eylemi) | Filmin o karede beklemesini sağlar. |
| Sonraki sahneye git | Sıradaki sahneye geçiren kural eylemi. |
| Baştan başla | İlk sahneye dönüp sayıları sıfırlayan eylem. |

!teacher Öğretmene not
Bu bölümle birlikte öğrenciler tam bir oyun döngüsü kurabilir: giriş, oyun, bitiş, yeniden başla. Dönem ödevi için iyi bir eşiktir. Bilgi yarışması örneği başka derslerle birleştirmeye çok uygundur: öğrenciler kendi derslerinden sorular hazırlayıp birbirlerine oynatabilir. "Üç soru, üç kural değil" kutusundaki fikir (aynı ad, tek kural) iyi tasarımın özüdür; üzerinde durmaya değer. En sık yaşanan sorun, Durdur kare eyleminin unutulması ve filmin sahneler arasında dönüp durmasıdır.
