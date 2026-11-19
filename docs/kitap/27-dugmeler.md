@kisim 6. Çizim ve karakter
# Düğmeler
> Fare üstüne gelince renk değiştiren, basınca çöken gerçek bir düğme. Menüler, "Başla" ekranları ve sahneler arası geçişler için.
:: ⏱ 25 dakika | 🎯 14. ve 26. bölümleri bitirenler için | 🧰 Örnekler > Düğmeli iki sahne

![İki sahneli bir film: "İleri" düğmesi ikinci sahneye götürür.](27-dugme.png "Birinci sahne yazısı ve altında mavi İleri düğmesi" 670){27-dugmeler.html 480x300}

!goal Bu bölümde öğreneceklerin
Düğme sembolünün durumlarını, hazır düğmeleri kullanmayı, bir düğmeye **eylem** vermeyi ve kural yazmadan sahneler arasında gezinmeyi öğreneceksin.

## Düğme, durumları olan bir semboldür

Sıradan bir çizime tıklamayı kuralla yakalayabilirsin (10. bölüm). Düğme sembolü fazladan bir şey yapar: farenin ne yaptığına göre **görünüşünü değiştirir**.

| Durum | Ne zaman görünür |
| Yukarıda | Fare uzaktayken: düğmenin normal hâli |
| Üzerinde | Fare düğmenin üstündeyken |
| Aşağıda | Düğmeye basılıyken |
| Aşağıdayken Üzerinde | Seçili kalmış bir düğmenin üstüne gelinince; çoğu düğmede boş bırakılır |

Her durum ayrı bir çizimdir. Genellikle aynı düğmenin biraz daha açık, biraz daha koyu ya da biraz küçültülmüş hâli.

## Örneği incele

1. {{Örnekler > Düğmeli iki sahne}}'yi aç ve {{Filmi Test Et}}.
2. Fareni düğmenin üstüne getir, bas, bırak. Görünüşü değişiyor; tıklayınca öbür sahneye geçiyor.
3. Programa dön ve düğmeye **çift tıkla**.

![Düğme sembolünün içindesin. ((1)) Neyi düzenlediğin. ((2)) Durum listesi: şu an **Yukarıda** durumunu görüyorsun.](27-durumlar.png "İleri sembolü düzenleniyor; durum listesinde Yukarıda seçili")

4. Durum listesinden ((2)) **Üzerinde**'yi seç: düğmenin fare üstündeyken görünen hâli. Sonra **Aşağıda**'yı.
5. **◀** ile geri dön.

## Adım adım: hazır düğmeyle menü

### 1. Düğmeyi koy

1. Yeni bir filmde, sağdaki **Ortak Kitaplık** panelindeki {{Düğmeler}} bölümünden bir düğmeyi sahneye sürükle.

### 2. Düğmeye eylem ver

1. Düğme seçiliyken Özellikler paneline bak: bir {{Eylem}} kutusu var.
2. Listeden ne yapacağını seç.

| Eylem | Ne yapar |
| Durdur | Filmi durdurur |
| Oynat | Filmi oynatmaya devam eder |
| Kareye git ve oynat / dur | Verdiğin kareye gider |
| Sahneye Git | Verdiğin sahneye geçer |
| Adrese Git | Verdiğin internet adresini açar |

Bunun için kural yazmak gerekmez: eylem düğmenin kendisindedir.

### 3. İki sahne kur

1. Birinci sahneye bir başlık yaz ve {{Kare eylemi}} kutusundan **Durdur**'u seç.
2. İkinci bir sahne ekle, ona da bir başlık ve **Durdur** ver.
3. Birinci sahnedeki düğmenin eylemini **Sahneye git: 2** yap.
4. İkinci sahneye de bir düğme koy; eylemi **Sahneye git: 1** olsun.
5. {{Filmi Test Et}}.

!why Düğme eylemi mi, kural mı?
İkisi de çalışır. **Düğme eylemi** basit gezinme içindir: sahneye git, oynat, durdur. Hızlıdır, kural gerektirmez. **Kural** ise her şeyi yapabilir: puan ver, ses çal, koşula bak. Bir düğmeye hem ad verip hem kural yazabilirsin: "Başla nesnesine tıklanınca: Puan sayısını 0 yap; sonraki sahneye git".

## Kendi düğmeni yapmak

1. Düğmenin normal hâlini çiz: yuvarlak köşeli bir dikdörtgen ve üstünde bir yazı.
2. İkisini seç, {{Değiştir > Sembol > Sembole Dönüştür}} ([[F8]]). Sonra alttaki {{Sembol Özellikleri}} sekmesinde {{Tür}} listesinden {{Düğme}}'yi seç.
3. Düğmeye çift tıkla. Şu an **Yukarıda** durumundasın.
4. Durum listesinden **Üzerinde**'yi seç. Sahne boş gelir ve listenin yanında {{Yukarıda Grafiğini Kopyala}} düğmesi belirir: ona tıkla, normal hâl gelir; sonra rengini biraz aç.
5. **Aşağıda** için aynısını yap; bu kez rengi koyulaştır ve düğmeyi bir iki piksel aşağı kaydır: basılmış gibi görünür.
6. **◀** ile dön ve düğmeye bir eylem ver.

!stuck Takıldıysan
- **Düğme programın içinde tepki vermiyor.** Sahnede düzenleme yaparken düğme sıradan bir nesnedir. Tepkisini görmek için {{Önizleme}} sekmesine geç ya da filmi test et.
- **Üzerinde durumu boş görünüyor.** Durumu çizmemişsin; **Yukarıda Grafiğini Kopyala** ile başla.
- **Düğme çalışıyor ama film hemen geri dönüyor.** Gittiğin sahnede **Durdur** kare eylemi yok.

## Sıra sende

- [ ] Düğmeli iki sahne örneğine üçüncü bir sahne ve ona giden bir düğme ekle.
- [ ] Kendi düğmeni çiz: üç durumu da farklı olsun.
- [ ] Top-raket oyununa (12. bölüm) düğmeli bir giriş ekranı yap: "Başla" ve "Nasıl oynanır".
- [ ] Zor soru: bir düğme hem ses çalsın hem sahne değiştirsin. Düğme eylemi yeter mi, yoksa kural mı gerekir?

## Bölümün özeti

| Yeni sözcük | Anlamı |
| Düğme sembolü | Farenin durumuna göre görünüşü değişen sembol. |
| Durum | Düğmenin üç görünüşünden biri: Yukarıda, Üzerinde, Aşağıda. |
| Düğme eylemi | Düğmeye Özellikler'den verilen, tıklanınca yapılan iş. |

!teacher Öğretmene not
Düğmeler Flash derslerinin klasik konusudur ve PisiAnimasyon'da aynı durum adlarıyla bulunur. Sınıfta genellikle "sahneler arası gezinen bir tanıtım" projesi olarak işlenir: okulun tanıtımı, bir konunun sunumu, etkileşimli bir hikâye. Oyun yapmak istemeyen öğrenciler için iyi bir alternatif projedir. "Düğme eylemi mi, kural mı?" kutusu iki yolu karşılaştırır; öğrenciler ikisini karıştırıp aynı düğmeye hem eylem hem kural verirse ikisi de çalışır ve film beklenmedik yere gidebilir.
