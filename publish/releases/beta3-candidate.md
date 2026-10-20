# 또니랑 베타3 — 다시 해도 똑같다

베타3에서는 저장한 작품과 입력 기록을 다시 열어 같은 결과인지 확인하는 흐름을 다듬었습니다. 공식 모음의 테트리스를 내 작업으로 저장하고, 키보드로 실행한 뒤 기록을 마치고, 창을 닫았다 다시 열어 원래 결과를 확인할 수 있습니다.

## 내려받아 실행하기

[Windows x64 베타3 ZIP](https://github.com/tongosu/ddonilang/releases/tag/v1.0.0-beta.3)을 새 폴더에 풀고 `START_SEAMGRIM.cmd`를 실행하세요. 별도의 Python 설치는 필요하지 않습니다. 작업대의 공식 모음에서 테트리스·이중 진자·함수·LRL 실험실 등을 열 수 있습니다.

기록·재현 확인은 ZIP 안 `BETA3_SMOKE.md` 순서대로 해 보세요. 공개 사본은 [직접 해 볼 순서](beta3/BETA3_SMOKE.md)에서도 읽을 수 있습니다. 원래 입력 기록과 같은 작품을 사용해야 하며, 다른 작품이나 손상·미완 기록은 이유를 붙여 거부합니다.

## 소스로 빌드하기

Rust·Cargo와 Windows 빌드 도구가 필요합니다.

```sh
cargo check --locked --workspace --no-default-features
cargo build --locked -p ddonirang-tool --no-default-features --bin ddonilang --bin ddonirang --bin 또니랑
cargo run --locked -p ddonirang-tool --no-default-features --bin ddonilang -- workbench
```

공개 저장소는 소스 모음입니다. 미리 빌드한 WASM·실행 파일은 GitHub Releases의 ZIP에 들어 있습니다. 작업대 전체를 소스에서 빌드하려면 WASM 빌드도 별도로 필요하므로, 처음에는 ZIP 사용을 권합니다.

## 이번 판에서 확인한 것

- CLI 세 이름과 WASM·작업대의 결과 대조
- 공식 테트리스의 명시적 Native 저장과 입력 자동 연결
- 기록 마치기·완료 기록 확인·종료 후 다시 열기
- 잘못된 기록의 거부와 기존 저장본 보호
- 현재 판과 마디 표시 유지

배포 파일은 검토된 r88입니다. 실행 엔진은 앞선 검증판의 동일 바이트를 재사용하고 화면·안내 후속을 반영했습니다. 공개 소스의 출처와 ZIP 해시는 `PUBLIC_SOURCE_ORIGIN.json`에 기록합니다. 공개 언어 스펙은 v0.8.1입니다.

## 아직 알아둘 점

개발 중인 사전 릴리스이며, 모든 기능·환경·장애 상황이 검증된 정식 제품은 아닙니다. 사람의 직접 사용 확인과 정식 출시 판정은 별도입니다.

기록 마치기 도중 창이 닫혔을 때 실제 진행을 복구하는 기능은 아직 없습니다. 저장본을 지우거나 덮어쓰지 말고 화면 안내를 확인하세요. 새 ID 복제는 원문으로 새 작품을 만드는 방법이지 이전 진행·완료 기록을 복구하는 방법은 아닙니다.

기존 검사 실패·건너뛴 검사와 일부 저장소의 긴 저장 시간은 남아 있습니다. 모든 과거 엔진·전체 작업공간·모든 정전과 중단 시점·모든 플랫폼을 통과했다고 주장하지 않습니다. 내부 필수 능력 26/28 확인과 공개 배포 품질·직접 사용 확인은 서로 다릅니다.

소스는 [LICENSE](../../LICENSE), 외부 소프트웨어는 [제삼자 고지](beta3/THIRD_PARTY_NOTICES.md)와 각 원문의 조건을 확인해 주세요.
