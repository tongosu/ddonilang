# Ddonilang (또니랑) — sym3

> game + AI era :: Korean-native PL + tools
> determinism == grammar, not option

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## summary

- Korean-first syntax, canonical output.
- same input => same output.
- AI workflow: pack + canon + replay/trace.

## current

- Rust crates validate DDN syntax, hooks, formulas, observe rows, current-line execution.
- Seamgrim workspace is WASM-first; CLI/WASM parity checks guard product behavior.
- Web workspace supports run, pause, reset, step-by-madi.
- samples include console-grid, space2d, Tetris, formula/proof/lambda, maze, bounce-probe.

## core

- `~` = role boundary
- `<-` = single state write
- define != execute

## sample

```ddn
x <- 15.
y <- 8.
합 <- (x + y).

"콘솔 보개 예제" 보여주기.
합 보여주기.
x 보여주기.
y 보여주기.
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
- quick start: `QUICKSTART.md`

## license

- planned open-source license (to be announced)
