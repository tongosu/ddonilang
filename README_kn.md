# Ddonilang (또니랑) — ಕನ್ನಡ (kn)

> ಆಟ ಮತ್ತು AI ಯುಗಕ್ಕಾಗಿ ಕೊರಿಯನ್-ನೆಟಿವ್ ಪ್ರೋಗ್ರಾಮಿಂಗ್ ಭಾಷೆ ಮತ್ತು ಸಾಧನಗಳು.
> Determinism (ಮರುಉತ್ಪಾದಕತೆ) ಅನ್ನು ವ್ಯಾಕರಣದಲ್ಲೇ ನಿರ್ಮಿಸಲಾಗಿದೆ.

![Ddonilang logo](publish/assets/ddonirang_wordmark.png)

---

## ಪರಿಚಯ

- Ddonilang ನಲ್ಲಿ ಕೊರಿಯನ್ ವಾಕ್ಯರಚನೆಯನ್ನು ನೇರವಾಗಿ ಕೋಡ್ ಆಗಿ ಬಳಸಬಹುದು.
- ಒಂದೇ input ಗೆ ಒಂದೇ output ಬರುತ್ತದೆ ಎಂಬ ನಿಯಮ ಮುಖ್ಯ.
- AI ಸಹಕಾರಕ್ಕೆ pack test, canonicalization, replay/trace ಮಾರ್ಗಗಳಿವೆ.

## ವ್ಯಾಕರಣ-ಆಧಾರಿತ Determinism

- `~` role-marker ಗಡಿ ಸ್ಪಷ್ಟಪಡಿಸುತ್ತದೆ.
- `<-` ಒಂದೇ state assignment ರೂಪ.
- definition ಮತ್ತು execution ಸ್ಪಷ್ಟವಾಗಿ ಬೇರ್ಪಡಿಸಲಾಗಿದೆ.

## ಸಣ್ಣ ಉದಾಹರಣೆ

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

## 6 ಮೂಲ ಘಟಕಗಳು

| ಹೆಸರು | ಪಾತ್ರ |
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

## ಸಂಪೂರ್ಣ ಆವೃತ್ತಿ

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

