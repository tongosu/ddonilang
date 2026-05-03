# Ddonilang (또니랑) — Euskara (eu)

> Jokoen eta AI aroarentzat, koreera-oinarritutako programazio hizkuntza eta tresnak.
> Determinismoa (erreproduzigarritasuna) gramatikan bertan txertatuta dago.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## Ikuspegi orokorra

- Ddonilang-ek koreerazko adierazpena zuzenean kode bihurtzen du.
- Sarrera bera -> emaitza bera printzipioa lehenetsita dago.
- AI lankidetzarako pack test, canonicalization, replay/trace erabiltzen dira.

## Uneko egoera

- Rust `ddonirang-lang`, `ddonirang-tool` eta `teul-cli` tresnek DDN syntax, hooks, formula, observe rows eta current-line execution egiaztatzen dituzte.
- Seamgrim workspace-k WASM lehenesten du; CLI/WASM parity checker-ek irteera bera eskatzen dute.
- Web workspace-n DDN examples ireki eta run, pause, reset, step-by-madi erabil daitezke.
- `solutions/seamgrim_ui_mvp/samples/` barruan console-grid, space2d pendulum, Tetris, formula/proof/lambda, maze eta bounce-probe examples daude.

## Zergatik determinismoa gramatikan

- `~` role-marker muga argitzen du.
- `<-` state assignment forma bakarra da.
- Definizioa eta exekuzioa bereizita daude.

## Adibide laburra

```ddn
x <- 15.
y <- 8.
합 <- (x + y).

"콘솔 보개 예제" 보여주기.
합 보여주기.
x 보여주기.
y 보여주기.
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
- Quick start: `QUICKSTART.md`

## License

- Planned open-source license (to be announced).
