# Ddonilang (또니랑) — Euskara (eu)

> Jokoen eta AI aroarentzat, koreera-oinarritutako programazio hizkuntza eta tresnak.
> Determinismoa (erreproduzigarritasuna) gramatikan bertan txertatuta dago.

![Ddonilang logo](publish/assets/ddonirang_wordmark.png)

---

## Ikuspegi orokorra

- Ddonilang-ek koreerazko adierazpena zuzenean kode bihurtzen du.
- Sarrera bera -> emaitza bera printzipioa lehenetsita dago.
- AI lankidetzarako pack test, canonicalization, replay/trace erabiltzen dira.

## Zergatik determinismoa gramatikan

- `~` role-marker muga argitzen du.
- `<-` state assignment forma bakarra da.
- Definizioa eta exekuzioa bereizita daude.

## Adibide laburra

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

## 6 elementu

| Izena | Rola |
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

## Dokumentazio osoa

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

