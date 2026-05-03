# 바이너리 다운로드

> 현재 GitHub 저장소에는 사용자 공개 바이너리를 포함하지 않는다. 공개 바이너리는 GitHub Releases에 둘 때 이 규칙을 따른다.

## 배포 위치
- GitHub Releases
- 저장소에는 바이너리를 포함하지 않는다.

## 지원 대상
- Windows x64
- macOS (x64/arm64)
- Linux (x64/arm64)

## 파일명 규칙(권장)
- `ddonirang-tool-<version>-<os>-<arch>.<ext>`
- `<os>`: `windows` | `macos` | `linux`
- `<arch>`: `x64` | `arm64`
- `<ext>`: Windows/macOS는 `zip`, Linux는 `tar.gz`

예:
- `ddonirang-tool-0.1.0-windows-x64.zip`
- `ddonirang-tool-0.1.0-macos-arm64.zip`
- `ddonirang-tool-0.1.0-linux-x64.tar.gz`

## 패키지 구조(권장)
```
ddonirang-tool-<version>-<os>-<arch>/
  ddonirang-tool(.exe)
  LICENSE
  NOTICE.txt        (선택)
  README.txt        (간단 사용법)
```

## 체크섬(권장)
- 릴리스에 `SHA256SUMS.txt`를 함께 제공한다.
- 가능한 경우 `SHA256SUMS.txt.sig` 서명을 추가한다.

## 소스 실행

현재 개발 상태를 바로 확인하려면 소스 빌드와 셈그림 작업실 실행을 권장한다.

- 빠른 시작: `QUICKSTART.md`
- 셈그림 예제: `solutions/seamgrim_ui_mvp/samples/index.json`
