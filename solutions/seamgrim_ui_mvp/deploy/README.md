# Seamgrim DDN Exec Server Docker Deploy

## 시작
- `docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up --build -d`

## 확인
- 앱: `http://localhost:8787/`
- 헬스체크: `http://localhost:8787/api/health`

## 운영
- 로그: `docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml logs -f ddn-exec-server`
- 중지: `docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml down`

## 옵션
- 외부 포트 변경: `DDN_EXEC_SERVER_PORT=18087 docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up -d`
- worker 모드: `TEUL_CLI_WORKER=1 docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up -d`

## Nginx Reverse Proxy (선택)
- 템플릿: `solutions/seamgrim_ui_mvp/deploy/nginx/seamgrim.conf`
- 핵심 규칙:
  - `application/wasm` MIME을 명시해야 `WebAssembly.instantiateStreaming` 경고를 피할 수 있습니다.
  - `/wasm/` 경로를 정적 서빙할 때 `default_type application/wasm`을 유지합니다.
  - 나머지 경로는 `proxy_pass http://ddn-exec-server:8787`로 전달합니다.
