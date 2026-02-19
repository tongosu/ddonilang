# Ddonilang (또니랑) — Runasimi (qu)

> Pukllaykuna AIwan ima pachapaq, Korea simipi ruraq simi-hatunchay.
> Determinism (kutin llamk'aq kikin) grammarpi churasqa.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## T'ikray (Overview)

- Ddonilangqa Korea simi qillqayta code nisqapi chiqanchan.
- Chay inputqa kikin kaptin, lluqsiypas kikinmi (determinism).
- AIwan yanapakuq ruwanapaq: pack test, canonicalization, replay/trace.

## Imarayku determinism grammarpi

- `~` nisqa role-marker chiqanchay.
- `<-` huklla state tikray.
- Ruray/haykuyta sutinchayman rakiy (`:move = {...}` vs call).

## Uchuy qhaway

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

## 6 elementokuna

| Suti | Ruray |
|---|---|
| 샘 | Input snapshot |
| 누리 | World state |
| 이야기 | Rules/progression |
| 보개 | Visual/sound output |
| 거울 | Record/replay |
| 슬기 | AI helper/guard |

## Ñan (Roadmap)

1. Console simulation.
2. Web 2D creation tool.
3. Semgrim + physics/economics labs + NuriGym.
4. Multilingual expansion.

## Qillqa aswan hatun

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

