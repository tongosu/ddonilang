# Ddonilang (또니랑) — తెలుగు (te)

> గేమ్ మరియు AI యుగానికి కొరియన్-నేటివ్ ప్రోగ్రామింగ్ భాష మరియు సాధనాలు.
> Determinism (పునరుత్పాదకత) ను వ్యాకరణంలోనే నిర్మించారు.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## పరిచయం

- Ddonilang లో కొరియన్ వాక్యరూపాన్ని నేరుగా కోడ్‌గా వాడవచ్చు.
- ఒకే input కు ఒకే output రావడం దీనిలో ప్రధాన సూత్రం.
- AI సహకారం కోసం pack test, canonicalization, replay/trace ప్రవాహం ఉంది.

## వ్యాకరణంలో Determinism ఎందుకు

- `~` role-marker సరిహద్దును స్పష్టం చేస్తుంది.
- `<-` ఒక్కటే state assignment రూపం.
- definition మరియు execution వేరుగా ఉంచబడతాయి.

## చిన్న ఉదాహరణ

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

## 6 మౌలిక అంశాలు

| పేరు | పాత్ర |
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

## పూర్తి పాఠ్యం

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

