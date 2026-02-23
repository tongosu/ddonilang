FROM rust:1.84-bookworm AS teul-builder

WORKDIR /build

COPY core ./core
COPY lang ./lang
COPY tools/teul-cli ./tools/teul-cli

RUN cargo build --release --manifest-path tools/teul-cli/Cargo.toml


FROM python:3.11-slim AS runtime

WORKDIR /app

ENV PYTHONUNBUFFERED=1
ENV TEUL_CLI_BIN=/usr/local/bin/teul-cli
ENV DDN_EXEC_SERVER_HOST=0.0.0.0
ENV DDN_EXEC_SERVER_PORT=8787

COPY --from=teul-builder /build/tools/teul-cli/target/release/teul-cli /usr/local/bin/teul-cli
COPY solutions/seamgrim_ui_mvp ./solutions/seamgrim_ui_mvp

EXPOSE 8787

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8787/api/health', timeout=3)"

CMD ["python", "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py", "--host", "0.0.0.0", "--port", "8787"]
