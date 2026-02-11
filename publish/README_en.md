# Ddonirang (또니랑)

> A Korean‑native programming language and tools for the era of games and AI  
> Determinism (reproducibility) is built into the grammar, not an option.

![Ddonirang logo](assets/ddonirang_wordmark.png)

---

## Overview

Ddonirang is a Korean‑first language/tooling project aiming for “a language where thoughts become games.”

- **Korean‑native**: Write code in Korean word order/expressions; ambiguity is reduced through canonicalization.
- **Determinism**: Same input yields the same output anywhere. Repro is the default for debugging, experiments, and RL.
- **AI‑oriented + AI‑aware**: We design for trustworthy AI‑written code with pack tests, canonicalization, replay/trace.
- **Education/creation‑friendly**: Extends from games to math/physics/economics as experiments.
- **Malhimnuri (Ddonirang World)**: A demo/tutorial world where spells (words) change the world.

---

## Why we made it

I started this project after a year of preparation because I wanted to leave a memorable work before finishing elementary school.

- Learning Python: powerful, but the early **grammar/English barrier** was real.
- Using Scratch: fun and easy, but it hits **complexity/scale limits**.
- The goal:  
  **A language readable in Korean + a world (Nuri) for games and study + collaboration with AI**.

Now I’m building **Semgrim (moving math drawings)** with my older sister to validate what “learning through creation” should feel like.

---

## Determinism is grammar, not an engine option

Ddonirang reduces ambiguity at the grammar level so that “human‑written” and “AI‑written” intent converge to the same form.

Key design choices:

- **Particle boundary `~` (canonical)**: Make Korean particles explicit in code.  
  Example: `문~을`, `주문~으로`, `나~는`
- **Single assignment `<-`**: One unified state‑update form.
- **Definition vs execution**: Define with `:움직씨 = { ... }`, execute with `~기 / ~하기`.
- **Single chaining `해서`**: One canonical way to link actions.
- **Contracts/guards**: Violations are diagnosed and recorded instead of ignored.

---

## 30‑second taste: “Open the door with a spell”

A tiny example made to feel like magic.  
(particle‑based binding + execution tail + state assignment)

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

## The 6 elements of Ddonirang

Ddonirang models the “world” with six layers.

| Name | Role | One line |
|---|---|---|
| Sam(샘) | Input snapshot | Everything starts from input |
| Nuri(누리) | World state | The state of a game/simulation |
| Iyagi(이야기) | Rules/progression | The layer that explains why things change |
| Bogae(보개) | Visuals/sound | The visible realm — 2D/3D/sound |
| Geoul(거울) | Record/replay | Rewind, audit, reproduce |
| Seulgi(슬기) | AI helper/guard | AI helps and sometimes guards |

---

## How we collaborate with AI

Ddonirang assumes AI can write code, but keeps the core deterministic with verifiable flow.

- Human: sets intent, explanation, and criteria
- AI: code generation, idea proposals, improvements
- **Pack tests**: examples must keep passing goldens
- **Canonicalization**: unstable surface forms converge to one
- **Replay/trace**: issues are reproduced exactly

---

## aiGYM — deterministic AI training/experiment environment

aiGYM is Ddonirang’s **determinism‑first training/experiment environment**.  
Same input and seed always produce the same result, making learning/comparison/debugging easier.

- Bundle input/log/replay into **reproducible experiments**.
- Compare policy changes with **pack goldens**.
- Connect games, math, physics, and economics under a **standard training interface**.

---

## Malhimnuri (또니랑누리/또니랑세상) — a world opened by spells

Malhimnuri (Ddonirang Nuri/World) is a demo/tutorial universe where words (spells) change the world.

- Write spells in **natural‑language‑like code**
- Show a **deterministic world‑state transition** flow
- Let beginners verify “why it works” via **replay/trace**

---

## Learning labs (planned)

Ddonirang aims to turn study into experiments, not just games.

### 1) Semgrim (moving math drawings)
- Animate formulas/geometry/physics with a **madi timeline**
- Same code → same frames (repro)
- A tool to **make math visible**

### 2) Physics Studio (Physics 2D)
- Theory (formula) ↔ experiment (Nuri) ↔ graph (comparison)
- “Hands‑on” experiments like free‑fall, parabolas, and collisions

### 3) Economics Lab (Eco‑Lab)
- Policy/market experiments with sliders
- Market/flow/relationship views + rewind (review)

---

## Roadmap

1. **Console‑based declarative simulation**  
   A world that moves in text (replay by default)
2. **Web‑based 2D creation tool**  
   Scratch‑like ease + Korean blueprints
3. **Semgrim/physics/economics labs + NuriGym**  
   Manim‑level expressiveness + deterministic RL + learning studio
4. **Multilingual expansion**  
   Prepare Japanese/Turkish/Mongolian READMEs, examples, and tutorials  
   For case‑inflected languages, we are considering **case‑marker canonicalization** rather than particle mapping

---

## Public channels (planned)

- **GitHub**: code + docs / issues & discussions / release tags
- **YouTube**: 3–5 minute intro + 30‑second demo + roadmap
- **Naver Blog**: weekly dev log + easy tutorials + new demos

---

## Contributing

Contributions are welcome.

- Easy‑to‑read sentences and example ideas
- Malhimnuri (Ddonirang Nuri) quest/spell ideas
- Semgrim (math/physics animation) scene ideas
- Physics Studio experiments (free‑fall, parabolas, collisions)
- Economics Lab experiments (inflation, wealth concentration)
- Pack cases for bug reproduction

---

## Acknowledgements

This project is built with **AI collaboration** in mind.  
Documentation, code refactoring, and test case organization were created **together with AI (including Codex)**.  
Thank you for the collaboration.

---

## License

- Planned to be open‑source (license will be announced on GitHub).

---
