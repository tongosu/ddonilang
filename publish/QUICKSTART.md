# 빠른 시작

## 바이너리 실행
1) GitHub Releases에서 OS에 맞는 바이너리를 내려받는다.
2) 압축을 해제한다.
3) 터미널에서 실행한다.

예시
- Windows: `.\ddonirang-tool.exe --help`
- macOS/Linux: `./ddonirang-tool --help`

## 소스 빌드
필요: Rust + Cargo

```sh
cargo build --release
```

실행 예시
- `target/release/ddonirang-tool -- run-once`
