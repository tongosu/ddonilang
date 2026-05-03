# Ddonilang (또니랑) — ಕನ್ನಡ (kn)

> ಆಟ ಮತ್ತು AI ಯುಗಕ್ಕಾಗಿ ಕೊರಿಯನ್-ನೆಟಿವ್ ಪ್ರೋಗ್ರಾಮಿಂಗ್ ಭಾಷೆ ಮತ್ತು ಸಾಧನಗಳು.
> Determinism (ಮರುಉತ್ಪಾದಕತೆ) ಅನ್ನು ವ್ಯಾಕರಣದಲ್ಲೇ ನಿರ್ಮಿಸಲಾಗಿದೆ.

![Ddonilang logo](assets/ddonirang_wordmark.png)

---

## ಪರಿಚಯ

- Ddonilang ನಲ್ಲಿ ಕೊರಿಯನ್ ವಾಕ್ಯರಚನೆಯನ್ನು ನೇರವಾಗಿ ಕೋಡ್ ಆಗಿ ಬಳಸಬಹುದು.
- ಒಂದೇ input ಗೆ ಒಂದೇ output ಬರುತ್ತದೆ ಎಂಬ ನಿಯಮ ಮುಖ್ಯ.
- AI ಸಹಕಾರಕ್ಕೆ pack test, canonicalization, replay/trace ಮಾರ್ಗಗಳಿವೆ.

## ಪ್ರಸ್ತುತ ಸ್ಥಿತಿ

- Rust `ddonirang-lang`, `ddonirang-tool`, `teul-cli` DDN syntax, hooks, formulas, observe rows, current-line execution ಅನ್ನು ಪರಿಶೀಲಿಸುತ್ತವೆ.
- Seamgrim workspace WASM-first ಮಾರ್ಗವನ್ನು ಬಳಸುತ್ತದೆ; CLI/WASM parity checks ಒಂದೇ ಫಲಿತಾಂಶವನ್ನು ನಿರೀಕ್ಷಿಸುತ್ತವೆ.
- Web workspace ನಲ್ಲಿ DDN examples ತೆರೆಯಬಹುದು: run, pause, reset, step-by-madi.
- `solutions/seamgrim_ui_mvp/samples/` ನಲ್ಲಿ console-grid, space2d pendulum, Tetris, formula/proof/lambda, maze, bounce-probe examples ಇವೆ.

## ವ್ಯಾಕರಣ-ಆಧಾರಿತ Determinism

- `~` role-marker ಗಡಿ ಸ್ಪಷ್ಟಪಡಿಸುತ್ತದೆ.
- `<-` ಒಂದೇ state assignment ರೂಪ.
- definition ಮತ್ತು execution ಸ್ಪಷ್ಟವಾಗಿ ಬೇರ್ಪಡಿಸಲಾಗಿದೆ.

## ಸಣ್ಣ ಉದಾಹರಣೆ

```ddn
x <- 15.
y <- 8.
합 <- (x + y).

"콘솔 보개 예제" 보여주기.
합 보여주기.
x 보여주기.
y 보여주기.
```

## 6 ಮೂಲ ಘಟಕಗಳು

| ಹೆಸರು | ಪಾತ್ರ |
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

## ಸಂಪೂರ್ಣ ಆವೃತ್ತಿ

- Full reference: `README_en.md`
- Quick start: `QUICKSTART.md`

## License

- Planned open-source license (to be announced).
