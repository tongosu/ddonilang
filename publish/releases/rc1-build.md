# RC1 공개 소스 빌드와 배포물의 관계

## 어떤 소스인가요?

이 공개 후보는 기존 베타3 공개 이력 위에 RC1의 코드·출하 예제·안내를 투영한 스냅샷입니다. 내부 Git 이력, 작업 기록과 정본 본문은 포함하지 않습니다.

- 로컬 검토 제품: `25bc4477d66e6ef1a855c1a9f0ee148c4f3d4aa4`(r142).
- 실제 Native 컴파일 기준: `07dd1cf390d943430a83a8e431ece84cfe3f43cf`.
- 공개 ZIP SHA-256: `538a1e881dd81d48fa5bb8fed287dc29daa2131b566e5daf044aa83f714ecccc`.
- 공개 태그 예정: `v1.0.0-rc.1`. 검토 전에는 태그가 존재하거나 게시됐다고 주장하지 않습니다.

기존 베타3 main은 이 스냅샷의 부모일 뿐 RC1 컴파일 소스가 아닙니다. 여섯 Rust crate는 실제 컴파일 기준에서, 셈그림의 비바이너리 소스·예제는 공개 ZIP에서 가져왔습니다. 공개 workspace에 맞춘 Cargo.lock 조정은 패키지를 추가하거나 버전을 올리지 않았습니다. 기존 무시된 관측 테스트 한 곳의 개인 PC 경로는 명시적 환경변수로 바꾸었으며 제품 실행 코드는 바꾸지 않았습니다.

## 필요한 도구

Rust와 Cargo, Windows x64의 MSVC C/C++ 빌드 도구 및 Windows SDK가 필요합니다. SQLite C 코드는 기존 `libsqlite3-sys`의 bundled 기능으로 빌드됩니다. 의존성을 처음 받는 환경에서는 Cargo의 공식 저장소 접근이 필요합니다. 캐시가 갖춰진 뒤에는 아래 명령에 `--offline`을 추가할 수 있습니다.

원 출하 빌드의 모든 도구·SDK·환경과 동일한 재현 환경을 봉인한 것은 아닙니다. 아래는 소스 빌드 절차이며, 출하 실행파일과 바이트까지 같은 재현 빌드를 완료했다는 뜻이 아닙니다.

## 공개 소스 검사

저장소 최상위에서 실행합니다.

```sh
cargo check --locked --workspace --no-default-features
```

공개 후보에서는 같은 명령에 `--offline`을 붙인 검사가 통과했습니다. 기존 경고는 남아 있으며 전체 테스트 통과나 P5/P6 완료를 뜻하지 않습니다.

## Windows Native 실행파일 빌드

```sh
cargo build --locked --release --target x86_64-pc-windows-msvc -p ddonirang-tool --no-default-features --bin ddonilang --bin ddonirang --bin 또니랑 --bin ddonirang-lsp
```

기본 출력은 `target/x86_64-pc-windows-msvc/release/`입니다. 내부 운영 도구는 `--no-default-features`로 제외합니다. 원 출하물의 정적 CRT 프로필과 이 일반 빌드의 연결 조건이 같다고 자동 가정하지 마세요. 위 Native 빌드의 공개 후보 재실행은 아직 하지 않았고 공통 소스 검사만 수행했습니다.

Native 작업대에는 출하 ZIP의 별도 정적 자산과 자산 목록도 필요합니다. 실행파일만 새로 만든 뒤 기존 ZIP에 덮어씌운 결과를 검증된 RC1로 취급하지 마세요. 새 조합은 별도의 자산·패키지 검증이 필요합니다.

## WASM 소스

공통 Rust 제품 경로는 `tool`의 `wasm` 기능입니다. wasm-pack이 설치된 환경에서는 다음 형태로 빌드할 수 있습니다.

```sh
wasm-pack build tool --target web --out-dir ../target/public-wasm -- --features wasm --no-default-features
```

이 명령으로 만든 산출물을 배포 ZIP의 기존 WASM과 같다고 주장하지 않습니다. 출하 ZIP은 기존 고정 WASM을 재사용했고, 이번 공개 소스 준비에서는 WASM을 재빌드하지 않았습니다. 새 산출물은 원문 대조·자산 목록 갱신·브라우저 검증 뒤에만 새 제품으로 다뤄야 합니다.

## 라이선스와 제한

[라이선스·출처 확인 상태](rc1-license-status.md), [제삼자 고지](rc1/THIRD_PARTY_NOTICES.md), [알려진 제한](rc1/RC1_KNOWN_LIMITATIONS.md)을 함께 읽어 주세요. 스냅샷 생성 전의 `PUBLICATION.json`은 역사 출처 자료이며 최종 태그·소스 봉인을 대신하지 않습니다. 독립검토 후 태그·Release와 정확한 대응 근거를 함께 게시합니다.
