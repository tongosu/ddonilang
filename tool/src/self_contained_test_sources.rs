//! Synthetic, repository-local sources used by `ddonirang-tool` unit tests.
//!
//! These fixtures intentionally exercise the same product contracts as the
//! historical pack and review inputs without making the product crate depend
//! on private `pack/**`, `work/**`, or `docs/ssot/**` paths at compile time.

pub(crate) const V25_DEFINITION_AND_BINDING: &str = r#"검산:움직씨 := {
    합계:수 := 4.
    합계 <- 7.
    합계 보여주기.
}
"#;

pub(crate) const PROJECTILE_CANON_SURFACE: &str = r#"채비 {
  초기속도: 수 <- (12) 매김 { 범위: 5..20. 간격: 1. }.
}.

(시작)할때 {
  vx <- 초기속도.
}.
"#;

pub(crate) const EXACT_ACYCLIC_MODEL: &str = r#"검증방법 <- (방식: "ddn.exact_acyclic_relation.v1").

검산 = 모형 {
  원본:나눔수@m :: (들임, 내줌).
  답:나눔수@m :: (그자리, 내줌).
  답::구함.
  답 =:= 원본 + 3@m.
}.

검산결과 <- (
  대상=검산,
  들임=(원본: 2@m),
  방법=검증방법,
  내줌=(답, 원본)
) 풀기.

(답값: 검산결과.답, 원본값: 검산결과.원본) 보임.
"#;

pub(crate) const MODEL_RELATION_CYCLE: &str = r#"검증방법 <- (방식: "ddn.exact_acyclic_relation.v1").

순환검산 = 모형 {
  왼:나눔수 :: (그자리, 내줌).
  오른:나눔수 :: (그자리, 내줌).
  왼::구함.
  오른::구함.
  왼 =:= 오른 + 1.
  오른 =:= 왼 + 1.
}.

검산결과 <- (
  대상=순환검산,
  들임=(),
  방법=검증방법,
  내줌=(왼, 오른)
) 풀기.

(왼값: 검산결과.왼, 오른값: 검산결과.오른) 보임.
"#;

pub(crate) const MODEL_UNIT_MISMATCH: &str = r#"검증방법 <- (방식: "ddn.exact_acyclic_relation.v1").

단위검산 = 모형 {
  거리:나눔수@m :: (들임, 내줌).
  기간:나눔수@s :: (그자리, 내줌).
  기간::구함.
  거리 =:= 기간.
}.

검산결과 <- (
  대상=단위검산,
  들임=(거리: 2@m),
  방법=검증방법,
  내줌=(기간)
) 풀기.
"#;

pub(crate) const MODEL_UNAUTHORIZED_INITIAL: &str = r#"검증방법 <- (방식: "ddn.exact_acyclic_relation.v1").

초기검산 = 모형 {
  원본:나눔수 :: (들임, 내줌).
  답:나눔수 :: (그자리, 내줌).
  답::구함.
  원본 =:= 답.
}.

검산결과 <- (
  대상=초기검산,
  들임=(원본: 2),
  처음=(원본: 4),
  방법=검증방법,
  내줌=(답)
) 풀기.
"#;

pub(crate) const MODEL_UNSUPPORTED_EVENT: &str = r#"검증방법 <- (방식: "ddn.exact_acyclic_relation.v1").

사건검산 = 모형 {
  원본:나눔수 :: (들임, 내줌).
  답:나눔수 :: (그자리, 내줌).
  답::구함.
  원본이 될때 { 답 <- 원본. }.
  원본 =:= 답.
}.

검산결과 <- (
  대상=사건검산,
  들임=(원본: 2),
  방법=검증방법,
  내줌=(답)
) 풀기.
"#;

pub(crate) const MODEL_WRONG_BACKEND: &str = r#"다른방법 <- (방식: "ddn.unapproved_backend.v1").

백엔드검산 = 모형 {
  원본:나눔수 :: (들임, 내줌).
  답:나눔수 :: (그자리, 내줌).
  답::구함.
  원본 =:= 답.
}.

검산결과 <- (
  대상=백엔드검산,
  들임=(원본: 2),
  방법=다른방법,
  내줌=(답)
) 풀기.
"#;

pub(crate) const RD206_INNER_EQUALS: &str = r#"검산=모형{원본:나눔수::(들임). 답:나눔수::(그자리,내줌). 답::구함. 답 =:= 원본 + 1.}.
검산결과 <- (대상=검산, 들임=(원본: 2), 실행계약=(결과=#정본값동일, 재현범위: #이식가능, 성능: #안전최적화, 외부유입: #봉인입력, 증거: #최소증거, 재생: #검증재생, 자원: #표준한도_v1), 내줌=(답)) 풀기.
"#;

pub(crate) const MULTIFILE_ROOT: &str = r#"프로젝트 {
  이름: "자가완결 다중 파일".
  첫글: "./app/main.ddn".
  목표 { 앱: "./app/main.ddn". }
  모듈길 { 로컬: "./lib". }
}.
"#;

pub(crate) const MULTIFILE_ENTRY: &str = r#"쓰임 {
  도움: "./lib/도움".
}

채비 {
  시간 <- 0.
  각도 <- 도움.시작값.
}.

(매마디)마다 {
  시간 <- 시간 + 1.
  각도 <- 각도 + 도움.증가값.
  (경과: 시간 * 1@s, 각도: 각도 * 1@rad, 증가량: 도움.증가값) 보임.
}.
"#;

pub(crate) const MULTIFILE_HELPER: &str = r#"드러냄 {
  시작값.
  증가값.
}

시작값 <- 20.
증가값 <- 3.
"#;

pub(crate) const W23_NET_EVENTS_SORTED: &str = r#"{
  "schema": "ddn.input_snapshot.v1",
  "net_events": [
    {"sender":"peer-a","seq":1,"order_key":"peer-a#1","payload":{"kind":"net_key","key":"W"}},
    {"sender":"peer-a","seq":2,"order_key":"peer-a#2","payload":{"kind":"net_key","key":"D"}},
    {"sender":"peer-b","seq":1,"order_key":"peer-b#1","payload":{"kind":"net_key","key":"A"}}
  ]
}
"#;

pub(crate) const W23_NET_EVENTS_UNSORTED: &str = r#"{
  "schema": "ddn.input_snapshot.v1",
  "net_events": [
    {"sender":"peer-b","seq":1,"order_key":"peer-b#1","payload":{"kind":"net_key","key":"A"}},
    {"sender":"peer-a","seq":2,"order_key":"peer-a#2","payload":{"kind":"net_key","key":"D"}},
    {"sender":"peer-a","seq":1,"order_key":"peer-a#1","payload":{"kind":"net_key","key":"W"}}
  ]
}
"#;

pub(crate) const W23_STATE_HASH: &str =
    "blake3:778a0af30f57f0448509fb418caf1437baa542817481666d6e306f51ab0c2655";

pub(crate) const W24_PARAMS: &str = r#"개체수 <- 10000.
컴포넌트수 <- 3.
아키타입_이동 <- 250.
성능_캡 <- 1.
"#;
pub(crate) const W24_STATE_HASH: &str =
    "blake3:29833ae4ef6be278b6e83ffa36dd11e549421d21c1174d6d66b3404a33c52bb7";

pub(crate) const W25_PARAMS: &str = r#"쿼리_대상수 <- 1000.
쿼리_배치 <- 10.
스냅샷_고정 <- 1.
"#;
pub(crate) const W25_STATE_HASH: &str =
    "blake3:2658ffb9b1da51bef5c96a55109a4d503a6f3661afcf536c70472a146c1d7cb1";

pub(crate) const W26_PARAMS: &str = r#"임자수 <- 4.
상품수 <- 3.
거래수 <- 100.
초기_잔고 <- 1000.
초기_재고 <- 10.
기본_가격 <- 5.
"#;
pub(crate) const W26_STATE_HASH: &str =
    "blake3:8816160087022b76d56de27b5e2be584d4bc67098a641d1ca96f8cd417b9a7fa";

pub(crate) const W27_PARAMS: &str = r#"임자수 <- 3.
거래수 <- 12.
초기_잔고 <- 10.
잔고_최소 <- 3.
거래_금액 <- 4.
"#;
pub(crate) const W27_STATE_HASH: &str =
    "blake3:4c2f4b689b3ca23b41a5d21a56c6684e8f0b45f1f6d77d5d05a0c7dd41ceadb6";

pub(crate) const W28_PARAMS: &str = r#"임자수 <- 4.
상품수 <- 3.
거래수 <- 20.
기본_가격 <- 5.
거래_금액 <- 4.
"#;
pub(crate) const W28_STATE_HASH: &str =
    "blake3:772f022a41767c31e96e8a58e6a99616cc6a6006d30c703a2bde5cd7d4a69eb1";

pub(crate) const W29_PARAMS: &str = r#"반응_패스_최대 <- 16.
알림_연쇄 <- 20.
반응_증분 <- 2.
초기_값 <- 5.
"#;
pub(crate) const W29_STATE_HASH: &str =
    "blake3:2552450c479669c166b227d1f8b1ff2267c25bd21a136782e78aa27bf27b583f";

pub(crate) const W30_PARAMS: &str = r#"제안_수 <- 5.
승인_토큰 <- 3.
적용_요청 <- 4.
승인_필수 <- 1.
"#;
pub(crate) const W30_STATE_HASH: &str =
    "blake3:1b5942c9a6efc5c841c8fcb9a3818eb9c471b34ca2df83c2d4885a2083f262b8";

pub(crate) const W31_PARAMS: &str = r#"참가자수 <- 3.
호스트_입력 <- 5.
손님_입력 <- 8.
동기_라운드 <- 4.
시작_값 <- 10.
"#;
pub(crate) const W31_STATE_HASH: &str =
    "blake3:d4a39990d5478e5ae4920f883213026ab18f2048d3168f5ed05643cf7eee2137";

pub(crate) const W32_PARAMS: &str = r#"차분_개수 <- 6.
코드_길이_전 <- 120.
코드_길이_후 <- 150.
상태_필드_수 <- 4.
요약_캡 <- 3.
"#;
pub(crate) const W32_STATE_HASH: &str =
    "blake3:bca7aa58b2be2e4bd16f654fd36c76a99a8d1e31bc7bebab50ed826c98d3ac77";

pub(crate) const W33_PARAMS: &str = r#"임자수 <- 6.
상품수 <- 4.
거래수 <- 18.
쿼리_배치 <- 3.
반응_패스_최대 <- 8.
"#;
pub(crate) const W33_STATE_HASH: &str =
    "blake3:6c1b2ef5c9fbef09d55be4ad833e79702c02766ff8a5954e8851500606499ca5";

pub(crate) const CONTAINER_RESOURCE: &str = r#"매틱:움직씨 = {
  차림 <- (1, 2, 3) 차림.
  모음값 <- (2, 1, 2) 모음.
  짝 <- ("b", 2, "a", 1) 짝맞춤.
  결과 <- (차림) 길이.
}
"#;
pub(crate) const CONTAINER_RESOURCE_STATE_HASH: &str =
    "blake3:cd91eb3ebb891c8226dd16bbbf16549f0ccf49175fb3997ffd5a15ef2fd83c1a";

pub(crate) const RUNTIME_TYPECHECK_CASES: &[(&str, &str)] = &[
    (
        "input_infer_ok.ddn",
        "(값:_~을) 통과:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (값=\"글\") 통과하기.\n}\n",
    ),
    (
        "input_int_mismatch.ddn",
        "(값:정수~을) 받:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (값=1.5) 받기.\n}\n",
    ),
    (
        "input_list_mismatch.ddn",
        "(목록:(수)차림~을) 합:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (목록=[1, \"a\"]) 합하기.\n}\n",
    ),
    (
        "input_map_mismatch.ddn",
        "(표:(글, 수)짝맞춤~을) 통과:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (표=(\"a\", 1, 2, 3) 짝맞춤) 통과하기.\n}\n",
    ),
    (
        "input_num_mismatch.ddn",
        "(값:수~을) 두배:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (값=\"글\") 두배하기.\n}\n",
    ),
    (
        "input_num_unit_mismatch.ddn",
        "(값:수~을) 통과:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (값=1@m) 통과하기.\n}\n",
    ),
    (
        "input_optional_mismatch.ddn",
        "(값:글~을?) 통과:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (값=1) 통과하기.\n}\n",
    ),
    (
        "input_optional_ok.ddn",
        "(값:글~을?) 통과:셈씨 = {\n}\n\n매틱:움직씨 = {\n    () 통과하기.\n}\n",
    ),
    (
        "input_set_mismatch.ddn",
        "(집합:(글)모음~을) 통과:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (집합=(\"a\", 1) 모음) 통과하기.\n}\n",
    ),
    (
        "input_unit_mismatch.ddn",
        "(거리:(m)수~을) 이동:셈씨 = {\n}\n\n매틱:움직씨 = {\n    (거리=1@s) 이동하기.\n}\n",
    ),
];

pub(crate) const ECO_MACRO: &str = r#"(시작)할때 {
  세율 <- 0.
}.

(매마디)마다 {
  균형가격 <- (100 + 세율 * 50).
}.
"#;

pub(crate) const ECO_C02_MICRO: &str = r#"(시작)할때 {
  세율 <- 0.
}.

(매마디)마다 {
  평균가격 <- (100 + 세율 * 200).
}.
"#;

pub(crate) const ECO_C02_RUNNER: &str = r#"{
  "schema":"ddn.macro_micro_runner.v0","seed":42,"ticks":4,
  "shock":{"type":"세율_인상","target":"세율","delta":0.1,"at_tick":2,"scope":"양쪽"},
  "models":{"거시":"macro.ddn","미시":"micro.ddn"},
  "diagnostics":[{"name":"거시↔제한합리","lhs":"거시.균형가격","rhs":"미시.평균가격","threshold":5.0}]
}
"#;

pub(crate) const ECO_C02_EXPECTED: &str = r#"{"results":[{"convergence_after_shock":false,"convergence_before_shock":true,"divergence_tick":2,"error_code":"E_ECO_DIVERGENCE_DETECTED","max_delta":"14.999999979","name":"거시↔제한합리"}],"schema":"ddn.runner_report.v0","seed":42,"shock_delta":"0.0999999998","shock_scope":"양쪽","shock_target":"세율","shock_tick":2,"shock_type":"세율_인상","ticks":4}"#;

pub(crate) const ECO_C03_MICRO: &str = r#"(시작)할때 {
  세율 <- 0.
}.

(매마디)마다 {
  가격_m1 <- (100 + 세율 * 50).
  가격_m3 <- (100 + 세율 * 52).
  가격_m5 <- (100 + 세율 * 54).
  가격_m10 <- (100 + 세율 * 58).
}.
"#;

pub(crate) const ECO_C03_RUNNER: &str = r#"{
  "schema":"ddn.macro_micro_runner.v0","seed":42,"ticks":4,
  "shock":{"type":"세율_인상","target":"세율","delta":0.2,"at_tick":3,"scope":"양쪽"},
  "models":{"거시":"macro.ddn","미시":"micro.ddn"},
  "diagnostics":[
    {"name":"기억=1","lhs":"거시.균형가격","rhs":"미시.가격_m1","threshold":0.5},
    {"name":"기억=3","lhs":"거시.균형가격","rhs":"미시.가격_m3","threshold":0.5},
    {"name":"기억=5","lhs":"거시.균형가격","rhs":"미시.가격_m5","threshold":0.5},
    {"name":"기억=10","lhs":"거시.균형가격","rhs":"미시.가격_m10","threshold":0.5}
  ]
}
"#;

pub(crate) const ECO_C03_EXPECTED: &str = r#"{"results":[{"convergence_after_shock":true,"convergence_before_shock":true,"divergence_tick":null,"name":"기억=1"},{"convergence_after_shock":true,"convergence_before_shock":true,"divergence_tick":null,"name":"기억=3"},{"convergence_after_shock":false,"convergence_before_shock":true,"divergence_tick":3,"error_code":"E_ECO_DIVERGENCE_DETECTED","max_delta":"0.7999999998","name":"기억=5"},{"convergence_after_shock":false,"convergence_before_shock":true,"divergence_tick":3,"error_code":"E_ECO_DIVERGENCE_DETECTED","max_delta":"1.5999999996","name":"기억=10"}],"schema":"ddn.runner_report.v0","seed":42,"shock_delta":"0.1999999999","shock_scope":"양쪽","shock_target":"세율","shock_tick":3,"shock_type":"세율_인상","ticks":4}"#;
