# 개발 소스 빌드 범위

기존 RC1 실행 파일을 찾는 경우 [시작 안내](../../START_HERE.md)와 Release를 사용합니다. 이 소스 후보가 기존 RC1 출하물과 같은 실행물이라는 뜻은 아닙니다. 새 ZIP·태그·Release를 게시하지 않습니다.

## 생산 빌드 접점

Rust/Cargo가 필요합니다. Windows Native는 MSVC C/C++ 빌드 도구와 Windows SDK가 필요합니다. 캐시에 없는 공식 의존성을 처음 받을 때는 네트워크가 필요합니다. lockfile을 임의로 갱신하지 마세요.

```text
cargo check --workspace --lib --bins --locked --no-default-features --features ddonirang-tool/v2-shared-modules
cargo build -p ddonirang-tool --bins --locked --no-default-features --features v2-shared-modules
```

PowerShell에서는 tools/build-public-source.ps1 -Offline로 같은 생산 접점을 확인할 수 있습니다. -BuildNative는 개발용 Native 빌드를 추가합니다. 서명 설치판이나 기존 출하물의 바이트 재현을 만드는 명령이 아닙니다.

WASM/작업대 소스는 포함하지만 생성 binding·WASM·빌드 manifest는 기존 바이너리처럼 섞지 않습니다. wasm32 target과 lock에 맞는 wasm-bindgen으로 같은 소스의 실행물·JS·manifest·Native serving 자산을 함께 생성하는 후속 단계가 필요합니다. 이 공개 후보에서 전체 작업대 포장·Chrome 실행·WASM 재현 빌드를 확인했다고 주장하지 않습니다.

## 선택 검사·권리 보류

프로젝트 코드와 새 직접 작성 예제는 기존 프로젝트 라이선스 범위를 따릅니다. lang의 MIT OR Apache-2.0 및 개별 제삼자 조건은 유지합니다. 이전 공개 제외 목록의 권리 보류 파일은 동의 없이 복구하지 않았습니다.

개인 Ready fixture를 쓰던 ignored 진단 검사는 공개 사본에서 DDN_WO25_READY_FIXTURE와 DDN_WO25_READY_LOCALAPPDATA를 명시 입력받습니다. 실제 자료·보호 저장소·예전 승인 결과를 동봉하거나 가짜 fixture로 바꾸지 않습니다. 해시·크기·operation·관측 flag 검사는 유지하고 기본 실행 대상에서는 계속 제외합니다.

권리 보류 tests 모듈/fixture와 로컬 전용 runner는 없을 수 있습니다. 전체 cargo test --workspace의 입력·컴파일 완결성은 이 후보의 확인 범위가 아닙니다. 생산 빌드와 선택 검사 소스를 구분합니다. 포함 파일별 해시와 범위는 FILE_INVENTORY.json에 있습니다.

## 소스와 실행물 계보

V2~V5의 내부 검토는 각각 고정된 소스·입력·실행물에 한정됩니다. 개발 소스의 출처 결속은 SOURCE_SNAPSHOT.json을, 현재 실제 공개 실행물의 출처는 기존 RC1 기록을 따릅니다. 저장소 루트의 PUBLIC_SOURCE_ORIGIN.json·PUBLICATION.json은 RC1 출하 이력이며 이 개발 스냅샷으로 다시 표시하지 않습니다.
