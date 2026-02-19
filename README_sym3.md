# Ddonilang (또니랑) — sym3

> game + AI era :: Korean-native PL + tools
> determinism == grammar, not option

![Ddonilang logo](publish/assets/ddonirang_wordmark.png)

---

## summary

- Korean-first syntax, canonical output.
- same input => same output.
- AI workflow: pack + canon + replay/trace.

## core

- `~` = role boundary
- `<-` = single state write
- define != execute

## sample

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

## 6 elements

| key | role |
|---|---|
| 샘 | input snapshot |
| 누리 | world state |
| 이야기 | rules/progression |
| 보개 | visual/sound output |
| 거울 | record/replay |
| 슬기 | AI helper/guard |

## roadmap

1. console sim
2. web 2d tool
3. semgrim + labs + nurigym
4. multilingual expansion

## full

- full reference: `README_en.md`

## license

- planned open-source license (to be announced)

