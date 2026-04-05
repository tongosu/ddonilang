# Intent

Define a single contract surface for external intent ingress.

All external intents must:

1. avoid direct world mutation
2. pass through `sam` as normalized input events
3. be replayed from recorded events only
4. respect gatekeeper/policy checks before commit

Canonical axes:

- `입력원천`: `사람` / `슬기` / `밖일` / `일정` / `이어전달` / `펼침실행`
- `주입방식`: `실주입` / `재연주입`
- `행동갈래`: `보기만` / `세계영향`
