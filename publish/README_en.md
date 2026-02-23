# Ddonilang (또니랑)

> A Korean‑native programming language and tools for the era of games and AI  
> Determinism (reproducibility) is built into the grammar, not an option.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## Overview

Ddonilang is a Korean‑first language/tooling project aiming for “a language where thoughts become games.”

- **Korean‑native**: Write code in Korean word order/expressions; ambiguity is reduced through canonicalization.
- **Determinism**: Same input yields the same output anywhere. Repro is the default for debugging, experiments, and RL.
- **AI‑oriented + AI‑aware**: We design for trustworthy AI‑written code with pack tests, canonicalization, replay/trace.
- **Education/creation‑friendly**: Extends from games to math/physics/economics as experiments.
- **Spellworld (Ddonilang World)**: A demo/tutorial world where spells (words) change the world.

---

## Why I made it

I started this project after a year of preparation because I wanted to leave a memorable work before finishing elementary school.

- Learning Python: powerful, but the early **grammar/English barrier** was real.
- Using Scratch: fun and easy, but it hits **complexity/scale limits**.
- The goal:  
  **A language readable in Korean + a world (Nuri) for games and study + collaboration with AI**.

Now I’m building **Semgrim (moving math drawings)** with my older sister to validate what “learning through creation” should feel like.

---

## Determinism is grammar, not an engine option

Ddonilang reduces ambiguity at the grammar level so that “human‑written” and “AI‑written” intent converge to the same form.

Key design choices:

- **Role‑marker boundary `~` (canonical)**: Make case/role markers explicit in code.  
  Example: `door~obj`, `spell~with`, `I~topic`
- **Single assignment `<-`**: One unified state‑update form.
- **Definition vs execution**: Define with `:move = { ... }`, execute in **imperative form** (e.g., `show`, `open`).
- **Single chaining `then`**: One canonical way to link actions.
- **Contracts/guards**: Violations are diagnosed and recorded instead of ignored.

---

## 30‑second taste: “Open the door with a spell”

A tiny example made to feel like magic.  
(particle‑based binding + execution tail + state assignment)

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

---

## The 6 elements of Ddonilang

Ddonilang models the “world” with six layers.

| Name | Role | One line |
|---|---|---|
| Source (샘) | Input snapshot | Everything starts from input |
| World (누리) | World state | The state of a game/simulation |
| Story (이야기) | Rules/progression | The layer that explains why things change |
| Visible Realm (보개) | Visuals/sound | The visible realm — 2D/3D/sound |
| Mirror (거울) | Record/replay | Rewind, audit, reproduce |
| Wisdom (슬기) | AI helper/guard | AI helps and sometimes guards |

---

## How we collaborate with AI

Ddonilang assumes AI can write code, but keeps the core deterministic with verifiable flow.

- Human: sets intent, explanation, and criteria
- AI: code generation, idea proposals, improvements
- **Pack tests**: examples must keep passing goldens
- **Canonicalization**: unstable surface forms converge to one
- **Replay/trace**: issues are reproduced exactly

---

## aiGYM — deterministic AI training/experiment environment

aiGYM is Ddonilang’s **determinism‑first training/experiment environment**.  
Same input and seed always produce the same result, making learning/comparison/debugging easier.

- Bundle input/log/replay into **reproducible experiments**.
- Compare policy changes with **pack goldens**.
- Connect games, math, physics, and economics under a **standard training interface**.

---

## Spellworld (Ddonilang World) — a world opened by spells

Spellworld (Ddonilang World) is a demo/tutorial universe where words (spells) change the world.

- Write spells in **natural‑language‑like code**
- Show a **deterministic world‑state transition** flow
- Let beginners verify “why it works” via **replay/trace**

---

## Learning labs (planned)

Ddonilang aims to turn study into experiments, not just games.

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

## Additional docs

- Document index: `publish/INDEX.md`
- CI stabilization note (2026-02-23): `publish/CI_AGGREGATE_GATE_FIX_NOTE_20260223.md`

---

## Contributing

Contributions are welcome.

- Easy‑to‑read sentences and example ideas
- Spellworld (Ddonilang World) quest/spell ideas
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
