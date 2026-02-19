# Ddonilang (또니랑) — नेपाली (ne)

> खेल र AI को युगका लागि कोरियन-नेेटिभ प्रोग्रामिङ भाषा र उपकरण।
> Determinism (पुनरुत्पादन) व्याकरणमै समावेश छ।

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## परिचय

- Ddonilang ले कोरियन वाक्यशैलीलाई कोडमा प्रत्यक्ष प्रयोग गर्छ।
- एउटै इनपुटमा एउटै परिणाम आउने निर्धारणलाई आधार बनाउँछ।
- AI सहकार्यका लागि pack test, canonicalization, replay/trace प्रयोग गर्छ।

## किन व्याकरण-आधारित Determinism

- `~` ले role-marker सीमा स्पष्ट बनाउँछ।
- `<-` एकीकृत state assignment हो।
- परिभाषा र चलाउने चरण छुट्टै राखिन्छ।

## छोटो उदाहरण

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

## ६ आधार तत्त्व

| नाम | भूमिका |
|---|---|
| 샘 | Input snapshot |
| 누리 | World state |
| 이야기 | Rules/progression |
| 보개 | Visual/sound output |
| 거울 | Record/replay |
| 슬기 | AI helper/guard |

## रोडम्याप

1. Console simulation.
2. Web 2D creation tool.
3. Semgrim + physics/economics labs + NuriGym.
4. Multilingual expansion.

## थप विवरण

- Full reference: `README_en.md`

## License

- Planned open-source license (to be announced).

