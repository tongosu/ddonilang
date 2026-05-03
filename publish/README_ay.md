# Ddonilang (또니랑) — Aymar aru (ay)

> Korea aru tuqita programaña, pukllaña ukat AI pachataki.
> Determinism ukax grammar taypin utt'ayatawa.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## Qhanañcha (Overview)

- Ddonilangax Korea arut qillqata code ukar tukuyi.
- Pachpa mantaña, pachpa mistuña (kutt'ayañataki).
- AI yanapt'ampi: pack test, canonicalization, replay/trace.

## Jichha lurata (Current status)

- Rust `ddonirang-lang`, `ddonirang-tool`, `teul-cli` ukanakax DDN syntax, hooks, formula, observe rows, current-line ejecución uñakipi.
- Seamgrim workspace ukax WASM nayraqata apnaqasi; CLI/WASM parity check ukax kikipa mistuwi munaraki.
- Web workspace ukan DDN examples jist'arasma: run, pause, reset, step-by-madi.
- `solutions/seamgrim_ui_mvp/samples/` ukan console-grid, space2d pendulum, Tetris, formula/proof/lambda, maze, bounce-probe examples utji.

## Determinism ukax grammar satawa

- `~` role-marker qhanañcha.
- `<-` mayakïki ukham state mayjt'ayaña.
- Uñstayaña ukat jawsaña jaljtaña.

## Jisk'a uñacht'äwi

```ddn
x <- 15.
y <- 8.
합 <- (x + y).

"콘솔 보개 예제" 보여주기.
합 보여주기.
x 보여주기.
y 보여주기.
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
- Quick start: `QUICKSTART.md`

## License

- Planned open-source license (to be announced).
