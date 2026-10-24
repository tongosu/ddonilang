# RC1 공개 소스 빌드와 출하물의 관계

실제 컴파일 기준은 고정 공개 스냅샷 `4c7760b7a9c6f6e78ef5882f7129a1f386e00276`입니다. 코드 변경 없이 경로 remap을 적용해 Native 세 이름·LSP·WASM을 새로 빌드합니다. 라이선스·안내가 추가된 최종 스냅샷과의 코드 동일성은 출처 대응표로 구분합니다. 기존 베타3 main을 RC1의 실제 컴파일 소스로 표시하지 않습니다.

## 소스 검사와 Native 빌드

Rust, Cargo, MSVC C/C++ 빌드 도구와 Windows SDK가 필요합니다. SQLite C 코드는 기존 bundled 의존성으로 빌드합니다. 캐시가 없는 환경에서는 먼저 공식 의존성을 받아야 합니다.

```sh
cargo check --locked --workspace --no-default-features
cargo build --release --locked --offline -p ddonirang-tool --bin ddonilang --bin ddonirang --bin 또니랑 --bin ddonirang-lsp --target x86_64-pc-windows-msvc -j 2
```

출하 프로필은 Windows x64 MSVC, 정적 CRT와 16MB PE 스택을 유지합니다. 빌드 환경의 `CARGO_ENCODED_RUSTFLAGS`에는 `-C link-arg=/STACK:16777216`, `-C target-feature=+crt-static` 및 소스·캐시·개인 작업 경로를 중립적인 경로로 치환하는 `--remap-path-prefix=원경로=중립경로`를 각각 인자로 지정합니다. encoded 인자 구분자는 U+001F입니다. 프로젝트 `.cargo`의 개인 target-dir을 가져오지 않고 별도 `CARGO_TARGET_DIR`를 사용합니다.

## WASM 빌드

```sh
wasm-pack build tool --target web --out-dir ../target/public-wasm --mode no-install -- --locked --offline --no-default-features --features wasm
```

WASM에는 경로 remap만 전달하고 Native의 MSVC CRT/스택 플래그를 전달하지 않습니다. wasm-pack·wasm-bindgen과 WASM target은 미리 설치돼 있어야 합니다. 새 바이너리와 JS·선언 파일·자산 목록은 같은 생산 묶음으로 연결합니다. 임의 파일 덮어쓰기를 검증된 RC1로 취급하지 않습니다.

## 잠금·라이선스·재현성

현재 공개 Cargo.lock과 보존된 고지의 원 lock SHA는 실제로 같습니다: `16515a401df33e29f6b134d6bc29f3ed1aa26d47b32ae5983723cd31f4aa9d0d`. 잠금 파일을 공개용으로 바꿨다고 주장하지 않습니다. 고지의 역사 역할과 재현 빌드 검증은 별개의 항목입니다.

빌드 명령 제공은 출하 바이너리의 바이트 재현 완료나 독립 PASS가 아닙니다. 개인 경로의 실제 포함 검사는 UTF-8·UTF-16LE·UTF-16BE를 구분해 기록합니다. 이전 r142와 공개 r3는 그대로 보존하고 새 실행물은 새 정체로 검증합니다. 경로별 조건은 저장소 최상위 LICENSE_SCOPE.md와 LICENSE_RIGHTS_PENDING.json을 확인해 주세요. 권리 미확인 자료는 공개 가능으로 간주하지 않습니다.
