self.onmessage = (event) => {
  const data = event?.data && typeof event.data === "object" ? event.data : {};
  if (data.type !== "numeric-factor-step") return;
  const job = data.job && typeof data.job === "object" ? data.job : null;
  self.postMessage({
    type: "numeric-factor-step-result",
    ok: false,
    status: "blocked",
    message: job
      ? "wasm numeric factor step API 연결 대기"
      : "numeric factor job snapshot missing",
  });
};
