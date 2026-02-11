# Ddonirang (또니랑)

> Oyun ve AI çağının **Korece yerel** programlama dili ve aracı  
> **Determinism (yeniden üretilebilirlik)** bir seçenek değil, **dil bilgisinin** parçasıdır.

![Ddonirang logo](assets/ddonirang_wordmark.png)

---

## Proje özeti

Ddonirang, “düşüncenin doğrudan oyuna dönüştüğü bir dil” hedefiyle ilerleyen **Korece merkezli** bir dil/araç projesidir.

- **Korece yerel**: Korece sözdizimi/ifade biçimleri doğrudan koda yazılır, belirsizlikler kanonlama ile azaltılır.
- **Determinism**: Aynı giriş her yerde aynı sonucu üretir.  
  Hata yeniden üretimi, deney karşılaştırması ve RL güveni için “reprodüksiyon” varsayılandır.
- **AI odaklı + AI‑aware**: AI kod yazsa bile güvenilirlik için  
  **pack testleri + kanonlama golden karşılaştırması + replay/trace** birlikte tasarlanır.
- **Eğitim/üretim dostu**: Oyunun yanında matematik/fizik/ekonomi deneylerine doğru genişler.
- **Malhimnuri (Ddonirang dünyası)**: Sözlerin (büyülerin) dünyayı değiştirdiği demo/öğretici evreni geliştiriyoruz.

---

## Neden yaptık

Bu projeye, **ilkokuldan önce akılda kalacak bir eser bırakmak istediğim için** bir yıl hazırlık sonrası başladım.

- Python öğrenirken: güçlü ama **gramer/İngilizce bariyeri** büyüktü.
- Scratch kullanırken: eğlenceli ve kolay, fakat büyüdükçe **karmaşıklık/ölçek sınırı** hissediliyor.
- Bu yüzden hedef:  
  **Korece okunabilir bir dil + oyun ve öğrenmeyi birleştiren bir dünya (Nuri) + AI ile iş birliği**.

Şu an **ablamla birlikte “Semgrim” (hareketli matematik çizimleri)** üretip  
“öğrenmeye dönüşen üretim”in nasıl hissettirmesi gerektiğini doğruluyoruz.

---

## Determinizm bir motor seçeneği değil, dil bilgisidir

Ddonirang belirsiz yorum ihtimalini dil bilgisinde azaltır.  
Böylece “insan yazsa da AI yazsa da” niyet aynı biçime yakınsar.

Örnek tasarım tercihleri:

- **Edat sınırı `~` (kanonik)**: Korece edat sınırını koda açıkça taşır.  
  Örnek: `문~을`, `주문~으로`, `나~는`
- **Tek atama `<-`**: Durum güncellemesini tek biçimde birleştirir.
- **Tanım vs çalıştırma ayrımı**: Tanım `:움직씨 = { ... }`, çalıştırma `~기 / ~하기`.
- **Tek bağlama `해서`**: Eylemleri bağlama biçimi tekleşir.
- **Sözleşme/guard**: İhlaller “sessizce geçme” yerine teşhis/kayıt ile kalır.

---

## 30 saniyelik tat: “Büyüyle kapı açma”

Büyü gibi görünen küçük bir örnek.  
(edat temelli bağlama + çalıştırma kuyruğu + durum ataması)

```ddn
바탕.문_상태 <- "닫힘".

(주문:말 ~로) 문~을_열기:움직씨 = {
  { 바탕.문_상태 == "닫힘" }인것 일때 {
    바탕.문_상태 <- "열림".
    (주문) 보여주기.
  } 아니면 {
    "이미 열려있어요." 보여주기.
  }.
}.

("열려라!")~로 문~을_열기 하기.
```

---

## Ddonirang’ın 6 öğe sözleşmesi

Ddonirang “dünya”yı 6 katmanla ele alır.

| Ad | Rol | Kısa açıklama |
|---|---|---|
| Sam(샘) | Girdi anlık görüntüsü | Tüm davranışlar girdiden başlar |
| Nuri(누리) | Dünya durumu | Oyun/simülasyonun durumu |
| Iyagi(이야기) | Kurallar/ilerleme | Neden değiştiğini açıklayan katman |
| Bogae(보개) | Görsel/ses | Görünen âlem — 2D/3D/ses |
| Geoul(거울) | Kayıt/tekrar | Geri sarma/denetim/yeniden üretim |
| Seulgi(슬기) | AI yardımcı/koruyucu | AI yardım eder, bazen korur |

---

## AI ile iş birliği

Ddonirang, AI’ın kod yazabileceğini varsayar.  
Ama çekirdeği deterministik tutup **doğrulanabilir bir akış** kurar.

- İnsan: niyet/açıklama/ölçütleri belirler
- AI: kod üretimi + fikir + iyileştirme
- **Pack testleri**: örneklerin golden’ı sürekli geçmesi
- **Kanonlama**: dalgalı yazımın tek biçime inmesi
- **Replay/trace**: sorunların birebir yeniden üretimi

---

## aiGYM — deterministik AI eğitim/deney ortamı

aiGYM, Ddonirang’ın **determinism temelli eğitim/deney ortamıdır**.  
Aynı giriş ve tohumla **her zaman aynı sonuç** elde edilir; öğrenme/karşılaştırma/hata ayıklama kolaylaşır.

- Girdi/log/replay’i birleştirip **yeniden üretilebilir deneyler** kurar.
- Politika değişimini **pack golden** ile karşılaştırır.
- Oyun/matematik/fizik/ekonomiyi **standart eğitim arayüzünde** birleştirir.

---

## Malhimnuri (또니랑누리/또니랑세상) — sözle açılan dünya

Malhimnuri (Ddonirang Nuri), “sözlerle dünyayı değiştirme” konseptli demo/öğretici evrendir.

- Büyüler (sözler) **doğal dile yakın kod** ile yazılır
- Dünya (Nuri) durumu **deterministik olarak değişir**
- Başlangıç seviyesinde “neden çalıştığı” **replay/trace** ile doğrulanır

---

## Öğrenmeye yardımcı laboratuvarlar (plan)

Ddonirang, oyunun yanında **öğrenmeyi deneye dönüştüren** araçlar da hedefler.  
Bu kısım belki “birileriyle” birlikte yapılacaktır.

### 1) Semgrim (hareketli matematik çizimleri)
- Formülleri/geom/füzik kurallarını **“madi zaman çizelgesi”** ile hareketlendirme
- Aynı kod → aynı kareler (reprodüksiyon)
- Matematiği **görselleştiren** araç

### 2) Fizik Stüdyosu (Physics 2D)
- Teori (formül) ↔ deney (Nuri) ↔ grafik (karşılaştırma)
- Serbest düşüş/parambola/çarpışma gibi “dokunulur” deneyler

### 3) Ekonomi Laboratuvarı (Eco‑Lab)
- Politika/pazar deneyleri için kaydırıcılar
- Pazar/akış/ilişki görünümü + geri sarma (inceleme)

---

## Yol haritası

1. **Konsol tabanlı deklaratif simülasyon**  
   Metinle hareket eden dünya (replay varsayılan)
2. **Web tabanlı 2D yaratım aracı**  
   Scratch kadar kolay + Korece blueprint
3. **Semgrim/fizik/ekonomi laboratuvarları + NuriGym**  
   Manim düzeyi ifade + deterministik RL + öğrenme stüdyosu
4. **Çok dillilik**  
   Japonca/Türkçe/Moğolca README, örnek ve eğitimlerin hazırlanması  
   Çekimli diller için “edat eşleştirme” değil **hâl işaretleme kanonlaması** da düşünülüyor

---

## Açık kanallar (plan)

- **GitHub**: kod + doküman / issue‑tartışma / release tag
- **YouTube**: 3‑5 dakikalık tanıtım + 30 saniye demo + yol haritası
- **Naver Blog**: haftalık geliştirme günlüğü + basit eğitim + yeni demo

---

## Katkı (Contributing)

Katkıları memnuniyetle karşılarız.

- Başlangıç seviyesine uygun cümle/örnek fikirleri
- Malhimnuri (Ddonirang Nuri) görev/büyü fikirleri
- Semgrim (matematik/fizik animasyonu) sahne fikirleri
- Fizik stüdyosu deney fikirleri
- Ekonomi laboratuvarı deney fikirleri
- Hata yeniden üretimi için pack testleri

---

## Teşekkür

Bu proje **AI ile birlikte üretim** yaklaşımıyla yürütülmektedir.  
Dokümantasyon, refaktör ve test düzenlemeleri **AI (Codex dahil) ile birlikte** hazırlandı.  
Bu iş birliği için teşekkürler.

---

## Lisans

- Açık kaynak olarak yayımlanacaktır (lisans GitHub’da duyurulur).

---
