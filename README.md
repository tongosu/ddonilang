# 또니랑 (ddonilang)

또니랑은 한국어 중심의 결정론 프로그래밍 언어·도구 프로젝트입니다.

이 저장소는 `e5474dcb49dcf3afe8e8d8a4ea3e731fff2a75b0`에서 만든 개발 중인 공개 스냅샷입니다.

## 공개 스냅샷 안내

- 개발 중인 공개 스냅샷입니다.
- 완성판이나 정식 릴리스가 아닙니다.
- V1 readiness, V1 closure, release, activation, Human PASS를 주장하지 않습니다.
- 이 공개 tree는 내부 SSOT, 작업 기록, 검토 자료, private history와 분리되어 있습니다.

## 이름

- 공식 이름: `ddonilang`, `또니랑`
- `ddonirang`: 호환성 별칭(compatibility alias)

## 포함 범위

이 스냅샷은 언어·코어·수치·증명·기호·도구의 Rust workspace와 공개 Language SPEC v0.6.0을 포함합니다.

공개 언어 스펙의 현재 포인터는 [`publish/spec/CURRENT.md`](publish/spec/CURRENT.md)입니다.

## 시작하기

Rust가 설치된 환경에서 다음처럼 workspace를 검사할 수 있습니다.

```text
cargo check --offline --locked
```

## 라이선스

프로젝트 공개 스냅샷의 라이선스는 [`LICENSE`](LICENSE)를 참조하십시오. third-party dependency와 fixture의 권리·고지는 각각의 원본 조건을 따릅니다.
