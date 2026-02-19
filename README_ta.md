# Ddonilang (또니랑) — தமிழ் (ta)

> விளையாட்டு மற்றும் AI காலத்திற்கான கொரிய-நேட்டிவ் நிரலாக்க மொழி மற்றும் கருவிகள்.
> Determinism (மீளுருவாக்கத் திறன்) இலக்கணத்திலேயே உறுதிசெய்யப்பட்டுள்ளது.

![Ddonilang logo](publish/assets/ddonirang_wordmark.png)

---

## அறிமுகம்

- Ddonilang கொரிய சொற்றொடரை நேரடியாக code ஆக எழுத உதவுகிறது.
- ஒரே inputக்கு ஒரே output கிடைக்குமென்ற நிர்ணயத்தைக் காப்பது இதன் மையம்.
- AI கூட்டாண்மைக்கு pack test, canonicalization, replay/trace வழிமுறைகள் உள்ளது.

## ஏன் இலக்கண-அடிப்படையிலான Determinism

- `~` role-marker எல்லையை தெளிவாக்குகிறது.
- `<-` ஒரே state assignment வடிவம்.
- definition மற்றும் execution தெளிவாக பிரிக்கப்படுகின்றன.

## சிறு எடுத்துக்காட்டு

```ddn
base.door_state <- "closed".

(spell:word ~with) door_obj_opening:move = {
  { base.door_state == "closed" } as_thing if {
    base.door_state <- "open".
    show spell.
  } else {
    show "It's already open.".
  }.
}.

"Open up!" with door_obj_opening.
```

## 6 அடிப்படை கூறுகள்

| பெயர் | பங்கு |
|---|---|
| 샘 | Input snapshot |
| 누리 | World state |
| 이야기 | Rules/progression |
| 보개 | Visual/sound output |
| 거울 | Record/replay |
| 슬기 | AI helper/guard |

## Roadmap

1. Console simulation.
2. Web 2D creation tool.
3. Semgrim + physics/economics labs + NuriGym.
4. Multilingual expansion.

## முழு ஆவணம்

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

