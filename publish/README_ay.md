# Ddonilang (또니랑) — Aymar aru (ay)

> Korea aru tuqita programaña, pukllaña ukat AI pachataki.
> Determinism ukax grammar taypin utt'ayatawa.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## Qhanañcha (Overview)

- Ddonilangax Korea arut qillqata code ukar tukuyi.
- Pachpa mantaña, pachpa mistuña (kutt'ayañataki).
- AI yanapt'ampi: pack test, canonicalization, replay/trace.

## Determinism ukax grammar satawa

- `~` role-marker qhanañcha.
- `<-` mayakïki ukham state mayjt'ayaña.
- Uñstayaña ukat jawsaña jaljtaña.

## Jisk'a uñacht'äwi

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

## 6 wakichawi

| Suti | Lurawi |
|---|---|
| 샘 | Input snapshot |
| 누리 | World state |
| 이야기 | Rules/progression |
| 보개 | Visual/sound output |
| 거울 | Record/replay |
| 슬기 | AI helper/guard |

## Thakhi (Roadmap)

1. Console simulation.
2. Web 2D creation tool.
3. Semgrim + physics/economics labs + NuriGym.
4. Multilingual expansion.

## Juk'amp qhana qillqata

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

