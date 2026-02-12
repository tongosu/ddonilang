# 빠른 시작

## 바이너리 실행
1) GitHub Releases에서 OS에 맞는 바이너리를 내려받는다.
2) (권장) 체크섬을 검증한다.
3) 압축을 해제한다.
4) macOS/Linux는 실행 권한을 부여한다.
5) 터미널에서 실행한다.

예시
- Windows: `.\ddonirang-tool.exe --help`
- macOS/Linux: `chmod +x ./ddonirang-tool` 후 `./ddonirang-tool --help`

체크섬 예시
```sh
# Linux/macOS
sha256sum -c SHA256SUMS.txt
```

## 소스 빌드
필요: Rust + Cargo

```sh
cargo build --release
```

실행 예시
- `target/release/ddonirang-tool -- run-once`
