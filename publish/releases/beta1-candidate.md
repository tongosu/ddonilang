# 또니랑 베타1 검토 후보

여러 파일로 작품을 만들고, 고치고, 저장한 뒤 다시 열어 이어 실행하는 흐름을 다듬었습니다. 임자·원자·분수가 든 Tuple의 저장과 복원, 입력 기록 V3와 여러 보기를 포함합니다.

## 소스로 사용하기

Rust와 Cargo가 필요합니다.

```sh
cargo build --locked --release -p ddonirang-tool --no-default-features --bin ddonilang --bin ddonirang --bin 또니랑
cargo run --locked -p ddonirang-tool --no-default-features --bin ddonilang -- workbench
```

소스 저장소에는 컴파일된 WASM을 넣지 않습니다. 셈그림 실행 전에 wasm-pack으로 브라우저 실행기를 만듭니다.

```sh
wasm-pack build tool --target web --release --out-dir ../solutions/seamgrim_ui_mvp/ui/wasm --out-name ddonirang_tool -- --locked --no-default-features --features wasm
```

간단한 원문을 `hello.ddn`에 저장하고 실행해 볼 수도 있습니다.

```ddn
3 + 4 보여주기.
```

```sh
cargo run --locked -p ddonirang-tool --no-default-features --bin ddonilang -- run hello.ddn --source-profile v1-core-v25
```

## 확인한 범위와 남은 일

- CLI 세 이름·WASM·실제 Chrome에서 저장·재열기·이어 실행 및 실패 시 부분 게시 방지를 확인했습니다.
- r66 변경분 독립검토는 한정 PASS이며 새 차단 결함은 없었습니다. 전체 테스트 통과·사용자 직접 확인·표면 동결·정식 출시 승인이 아닙니다.
- 전체 tool 검사에는 기존 실패 4건이 남아 있습니다. [상세](beta1/BETA1_KNOWN_TEST_FAILURES.md)를 숨기지 않고 제공합니다.
- 실시간 입력을 중단한 기록은 미완 기록이며 정상 재생할 수 없습니다. 미리 정한 유한 입력 요청으로 만든 완료 기록의 재생과 구별합니다.
- 정전·모든 강제 종료의 내구 복구와 지속 파생 `<<-`의 실행은 베타2 후속 과제입니다.
- [직접 써 볼 항목](beta1/BETA1_SMOKE.md)은 준비됐지만 사용자 직접 통과를 대신 기록하지 않았습니다. 이 안내의 ZIP 기준 절차와 소스 빌드 절차는 구별해 주세요.
- 이번 반영은 소스·문서 공개입니다. 새 바이너리 릴리스는 아직 게시하지 않습니다. ZIP 의존성 라이선스 본문 네 건과 공개 배포 점검이 남아 있습니다.

기술적 출처: 검토된 내부 고정 소스 `7cea18201655d5ae9f1ccfc5f8ef218118245a7c`의 허용 경로를 공개 이력에 투영했습니다. 내부 Git 이력을 공개 저장소에 합치지 않았습니다. 공개 스펙은 v0.8.1입니다.
