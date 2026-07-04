# SSOT_LANG_INVENTORY_V1

| 구성물 | 갈래(타입/키워드/블록/stdlib함수/리터럴/연산자) | 정본명 | 입력 별칭 | 상태(landed/docs-first/이월/폐기) | SSOT 근거(파일:행) | 코드 근거(파일:행 또는 "미발견") |
|---|---|---|---|---|---|---|
| 수 | 타입 | 수 | 셈수/Fixed64 이전 표면 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2239 | tools/teul-cli/src/canon.rs:50 |
| 셈수 | 타입 | 셈수 | 수, fixed64, sim_num | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7889; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:939 | lang/src/canonicalizer.rs:468 |
| 셈수2/셈수4/셈수8 | 타입 | 셈수2/4/8 | sim_num16, sim_num32, fixed64 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7894; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:940 | lang/src/canonicalizer.rs:468 |
| 바른수 | 타입 | 바른수 | 정수, int, int64 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7890; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:941 | lang/src/canonicalizer.rs:469 |
| 바른수1/2/4/8 | 타입 | 바른수1/2/4/8 | int8, int16, int32, int64 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7894; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:942 | lang/src/canonicalizer.rs:469 |
| 큰바른수 | 타입 | 큰바른수 | bigint, big_int | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7891; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:943 | lang/src/canonicalizer.rs:472; tools/teul-cli/src/runtime/eval.rs:2493 |
| 나눔수 | 타입 | 나눔수 | 유리수, rational, ratio, frac | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7892; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:944 | lang/src/canonicalizer.rs:473; tools/teul-cli/src/runtime/eval.rs:2506 |
| 곱수 | 타입 | 곱수 | factor, factorized, primepow | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7893; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:945 | lang/src/canonicalizer.rs:474; tools/teul-cli/src/runtime/eval.rs:2532 |
| 글 | 타입 | 글 | string | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2239 | tools/teul-cli/src/canon.rs:50 |
| 참거짓/논 | 타입 | 참거짓 | 논, bool, boolean | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2239 | tools/teul-cli/src/canon.rs:50 |
| 차림 | 타입 | 차림 | list | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2251 | tools/teul-cli/src/runtime/eval.rs:3501 |
| 짝맞춤 | 타입 | 짝맞춤 | map | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2133 | tools/teul-cli/src/runtime/eval.rs:5418 |
| 흐름(N) | 타입 | 흐름 | 흐름씨/흐름뷰 문맥 유지 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:167; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:62 | lang/src/ast.rs:345 |
| 값꾸러미 | 타입 | 값꾸러미 | value bundle | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2569 | 미발견 |
| 묶음씨 | 타입 | 묶음씨 | 레코드, 구조체 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2581 | 미발견 |
| 튜플 | 타입 | 튜플 | tuple | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2641 | 미발견 |
| 고름씨 | 타입 | 고름씨 | tag union | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2649 | 미발견 |
| 씨앗 | 키워드 | 씨앗 | function/seed name | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6022 | tools/teul-cli/src/canon.rs:3884 |
| 씨앗 리터럴 | 리터럴 | {x \| 식} | inline callable | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6499; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6510 | tools/teul-cli/src/runtime/eval.rs:3516 |
| 이름씨 | 타입 | 이름씨 | struct | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2453; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:425 | 미발견 |
| 임자 | 타입 | 임자 | entity, owner, actor | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2001; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:630 | tools/teul-cli/src/canon.rs:1345; lang/src/frontdoor.rs:46 |
| 임자가리킴 | 타입 | 임자가리킴 | entity ref/handle | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7050; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1056 | 미발견 |
| 임자무리 | 타입 | 임자무리 | entity set/collection | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7050; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1057 | 미발견 |
| 알림씨 | 타입 | 알림씨 | event type | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2029 | tools/teul-cli/src/canon.rs:7460 |
| 움직씨 | 키워드 | 움직씨 | 동작 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1371; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2768 | tools/teul-cli/src/canon.rs:7460 |
| 이음씨 | 키워드 | 이음씨 | 중위연산 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2780 | 미발견 |
| 흐름씨 | 타입 | 흐름씨 | reactive stream seed | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2954; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2958 | lang/src/ast.rs:345 |
| 일때씨 | 타입 | 일때씨 | reactive when seed | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3015 | 미발견 |
| 셈씨 | 타입 | 셈씨 | numeric function | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7055 | 미발견 |
| 주사위씨 | 키워드 | 주사위씨 | deterministic RNG seed | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5903; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1236 | 미발견 |
| 모양 | 타입 | 모양 | shape | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7749; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:569 | tools/teul-cli/src/canon.rs:472 |
| 모양씨 | 타입 | 모양씨 | shape generator | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7772; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1279 | 미발견 |
| 겹보기씨 | 타입 | 겹보기씨 | overlay layer | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7781; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1280 | 미발견 |
| 세움씨 | 타입 | 세움씨 | legacy assertion seed | 이월 | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5830 | 미발견 |
| 겹차림/텐서 | 타입 | 겹차림 | 텐서 | ⚠️모순 | docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:444; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:494 | tools/teul-cli/src/cli/tensor.rs:67 |
| 채비 | 블록 | 채비 | 그릇채비/붙박이마련은 historical | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1757; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1314 | tools/teul-cli/src/canon.rs:1806 |
| 설정 | 블록 | 설정 | file-leading meta | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7092; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7700 | lang/src/frontdoor.rs:327 |
| 보임 | 블록 | 보임 | view block | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3934; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3950 | tools/teul-cli/src/canon.rs:3708 |
| 보개마당 | 블록 | 보개마당 | 보개장면은 폐기/비정본 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7296; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7309 | tools/teul-cli/src/canon.rs:489 |
| 매김 | 블록 | 매김 | 조건 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1801; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:547 | tools/teul-cli/src/canon.rs:1909; tools/teul-cli/src/canon.rs:1923 |
| 매김.가늠/갈래 | 블록 | 가늠/갈래 | action/control section | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8006; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8009 | tools/teul-cli/src/canon.rs:1968 |
| 판 | 블록 | 판 | lifecycle unit | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:327; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8036 | tools/teul-cli/src/cli/run.rs:4926 |
| 마당 | 블록 | 마당 | lifecycle unit | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:327; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8037 | tools/teul-cli/src/cli/run.rs:4930 |
| 기억 | 블록 | 기억 | story-memory surface | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:335; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:213 | 미발견 |
| 갈림 | 블록 | 갈림 | choice protocol surface | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:344; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:214 | 미발견 |
| 고르기 | 블록 | 고르기 | 아니면/그밖의 경우 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1448 | tools/teul-cli/src/canon.rs:512; tools/teul-cli/src/canon.rs:3316 |
| ~에 따라 | 블록 | 에 따라 | 갈래 분기 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4668 | lang/src/currentline.rs:290 |
| 토막 | 블록 | { } | 안은문장 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1363 | lang/src/ast.rs:48 |
| 토막 꼬리 | 키워드 | }한것/}인것/}아닌것/}하고/}해서 | 평가/파이프 표지 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1363 | tools/teul-cli/src/canon.rs:10232 |
| 제 | 키워드 | 제 | self | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2001; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:546 | tools/teul-cli/src/canon.rs:4828; tools/teul-cli/src/cli/run.rs:4844 |
| 성질 | 블록 | 성질 | 임자 속성 블록 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2001; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:630 | tools/teul-cli/src/cli/frontdoor_parse.rs:233 |
| 받으면 | 키워드 | 받으면 | on_receive | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2045; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:545 | tools/teul-cli/src/canon.rs:1719 |
| 될때 | 키워드 | 될때 | condition edge hook | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:419; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7927 | lang/src/frontdoor.rs:1170 |
| 인 동안 | 키워드 | 인 동안 | state/level hook | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:431; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7928 | lang/src/frontdoor.rs:1170 |
| 할때/마다 | 키워드 | 할때/마다 | hook family | landed | docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:413; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1494 | tools/teul-cli/src/canon.rs:2172 |
| 건너뛰기 | 키워드 | 건너뛰기 | continue | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:442 | tools/teul-cli/src/canon.rs:517 |
| 전제하에/보장하고 | 키워드 | 전제하에/보장하고 | pre/post contract | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1510 | tools/teul-cli/src/canon.rs:509; tools/teul-cli/src/canon.rs:511 |
| 수선/물림/알림 계약모드 | 키워드 | 수선/물림/알림 | contract mode | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1510 | tools/teul-cli/src/canon.rs:3451 |
| = | 연산자 | = | readonly definition | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1865; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1099 | tools/teul-cli/src/canon.rs:2327 |
| <- | 연산자 | <- | mutable assignment | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1888 | tools/teul-cli/src/canon.rs:10223 |
| -> | 연산자 | <- | 우향 대입 설탕 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1908 | 미발견 |
| ~~> | 연산자 | ~~> | 알림 송신/말결 통신 화살표 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1941 | tools/teul-cli/src/canon.rs:7470 |
| 산술 연산자 | 연산자 | + - * / % ^ | 산술 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6638 | tools/teul-cli/src/runtime/eval.rs:6361 |
| @ 단위 | 연산자 | @ | 값@단위 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6242; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6897 | tools/teul-cli/src/canon.rs:2726; lang/src/parser.rs:3780 |
| @ 쓸감 | 리터럴 | @"..." | resource/asset literal | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4551; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6921 | tools/teul-cli/src/canon.rs:2822; tools/teul-cli/src/canon.rs:10583 |
| 숫자/글/참거짓 리터럴 | 리터럴 | 숫자/글/참/거짓 | literal | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1711; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2422 | lang/src/ast.rs:446 |
| 수식 리터럴 | 리터럴 | 수식{...} | #ascii/#ascii1 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5284 | tools/teul-cli/src/canon.rs:487; lang/src/ast.rs:351 |
| 글무늬 리터럴 | 리터럴 | 글무늬{...} | template seed | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5524; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5530 | tools/teul-cli/src/canon.rs:486; tools/teul-cli/src/runtime/template.rs:405 |
| 정규식 리터럴 | 리터럴 | 정규식{...} | regex literal | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5676; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5688 | tools/teul-cli/src/cli/run.rs:2480; tools/teul-cli/src/runtime/eval.rs:3748 |
| 세움 블록 | 블록 | 세움{} | 세움값 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5738 | tools/teul-cli/src/runtime/eval.rs:6253 |
| 상태머신 블록 | 블록 | 상태머신{} | 상태머신값 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5778 | 미발견 |
| 밝히기/증명/근거로/귀납으로 | 블록 | 밝히기/증명{} | proof surface | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5802; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5823 | tools/teul-cli/src/cli/symbolic.rs:55 |
| 지키기/반례찾기/해찾기 | stdlib함수 | 지키기/반례찾기/해찾기 | proof runtime minimum | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5849 | tools/teul-cli/src/canon.rs:9636 |
| 논리 양화 | 키워드 | 낱낱에 대해/중 하나가/중 딱 하나가 | forall/exists/exactly-one | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5868 | tools/teul-cli/src/canon.rs:9636 |
| 덩이/미루기 | 블록 | 덩이{} | 미루기/예약 | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5925; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7984 | 미발견 |
| 파이프 | 연산자 | 해서 | }해서 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5947; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5982 | tools/teul-cli/src/canon.rs:10232 |
| 변환 | stdlib함수 | 변환 | map | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6491; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:1338 | lang/src/frontdoor.rs:520; tools/teul-cli/src/runtime/eval.rs:3512 |
| 거르기 | stdlib함수 | 거르기 | filter/필터/체 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6490; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:893 | lang/src/frontdoor.rs:520; tools/teul-cli/src/runtime/eval.rs:3501 |
| 합치기 | stdlib함수 | 합치기 | reduce/문자열 연결 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6492; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6520 | lang/src/frontdoor.rs:531; tools/teul-cli/src/runtime/eval.rs:3618 |
| 찾기? | stdlib함수 | 찾기? | optional lookup | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2162 | tools/teul-cli/src/runtime/eval.rs:20765 |
| 이음관계.endpoint_equality | stdlib함수 | 이음관계.* | 같게 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:100; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:50 | lang/src/frontdoor.rs:994 |
| 이음관계.endpoint_flow | stdlib함수 | 이음관계.* | 흐르게/거슬러 흐르게 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:100; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:179 | lang/src/frontdoor.rs:1008 |
| 이음관계.carried_property | stdlib함수 | 이음관계.* | 실리게 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:100 | lang/src/frontdoor.rs:1028 |
| 이음관계.풀이원복 | stdlib함수 | 이음관계.풀이원복 | solver remap | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:278 | tools/teul-cli/src/runtime/eval.rs:5233 |
| 미분하기 | stdlib함수 | 미분하기 | symbolic diff transform | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5417; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:572 | tools/teul-cli/src/cli/symbolic.rs:24 |
| 적분하기 | stdlib함수 | 적분하기 | symbolic integrate transform | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5417; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:572 | tools/teul-cli/src/cli/symbolic.rs:29 |
| 미분.중앙차분 | stdlib함수 | 미분.중앙차분 | 수치미분 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5452; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5465 | lang/src/stdlib.rs:1111; tools/teul-cli/src/runtime/eval.rs:3125 |
| 적분.사다리꼴 | stdlib함수 | 적분.사다리꼴 | 수치적분 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5452; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5486 | lang/src/stdlib.rs:1116; tools/teul-cli/src/runtime/eval.rs:3174 |
| 수치해.이분법 | stdlib함수 | 수치해.이분법 | numeric root solve | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5452 | lang/src/stdlib.rs:1121; tools/teul-cli/src/runtime/eval.rs:3225 |
| 선형보간/역선형보간/부드럽게 | stdlib함수 | 선형보간/역선형보간/부드럽게 | lerp/invlerp/smoothstep | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6207; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6211 | tools/teul-cli/src/runtime/eval.rs:13358 |
| 정규맞추기/정규찾기/정규바꾸기/정규나누기 | stdlib함수 | 정규맞추기/정규찾기/정규바꾸기/정규나누기 | regex API | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5725; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5728 | lang/src/stdlib.rs:216; tools/teul-cli/src/runtime/eval.rs:3748 |
| 입력키/입력키?/입력키! | stdlib함수 | 입력키/입력키?/입력키! | compat/권장/엄격 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6821; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6842 | lang/src/stdlib.rs:546; tools/teul-cli/src/runtime/eval.rs:4257 |
| DetMath | stdlib함수 | DetMath | 결정적 수학 함수 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6136; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6781 | tools/teul-cli/src/runtime/eval.rs:2760 |
| 단위 시스템 | stdlib함수 | @단위 | SI/유도/접두사/@K/@C/@F | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6242; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6303 | tools/teul-cli/src/runtime/eval.rs:11214; tools/teul-cli/src/runtime/eval.rs:15488 |
| 말결 토큰 | 키워드 | 말결 | 매우/꽤/조금/약간/거의 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6333; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6735 | tools/teul-cli/src/runtime/eval.rs:15403 |
| AI-GYM 함수 | stdlib함수 | 끝내/지금관찰/보상 | 눈떠 | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6668; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6712 | tools/teul-cli/src/cli/run.rs:2481; tools/teul-cli/src/cli/reward.rs:91 |
| 쓰임/드러냄 | 블록 | 쓰임/드러냄 | import/export | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7549; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7575 | 미발견 |
| 프로젝트/실행기본 | 블록 | 프로젝트{}/실행기본{} | project root/default run profile | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7633; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7689 | 미발견 |
| 설정.보개/설정.슬기 | 블록 | 설정.보개/설정.슬기 | view/AI config meta | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7724; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7805 | tools/teul-cli/src/canon.rs:4033 |
| lifecycle 동사 | 키워드 | 시작하기/넘어가기/불러오기/누리다시 | lifecycle/reset | landed | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7961; docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7969 | tools/teul-cli/src/cli/run.rs:4926 |

## SSOT_LANG 절 반영/소거

| 구성물 | 갈래(타입/키워드/블록/stdlib함수/리터럴/연산자) | 정본명 | 입력 별칭 | 상태(landed/docs-first/이월/폐기) | SSOT 근거(파일:행) | 코드 근거(파일:행 또는 "미발견") |
|---|---|---|---|---|---|---|
| 절 반영/소거: v24.12.9-candidate integrated revision | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7 | 미발견 |
| 절 반영/소거: v24.12.9 language proposal overlay | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:17 | 미발견 |
| 절 반영/소거: v24.12.9-candidate local temporary / local purity proposal addendum | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:29 | 미발견 |
| 절 반영/소거: Local temporary binding 후보 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:33 | 미발견 |
| 절 반영/소거: Local purity 후보 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:63 | 미발견 |
| 절 반영/소거: 유지/기각된 UX sugar | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:76 | 미발견 |
| 절 반영/소거: v24.12.9-candidate integrated language revision | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:87 | 미발견 |
| 절 반영/소거: Operator triad | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:89 | 미발견 |
| 절 반영/소거: Connect candidate | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:100 | 미발견 |
| 절 반영/소거: Seum and derivative open issue | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:123 | 미발견 |
| 절 반영/소거: Story-tech grammar boundary | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:127 | 미발견 |
| 절 반영/소거: v24.12.9-candidate solver/tier/flow-term addendum | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:130 | 미발견 |
| 절 반영/소거: Deterministic solver methods are cataloged separately from connect surface | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:132 | 미발견 |
| 절 반영/소거: D-ULTRA solver strategy relaxation | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:155 | 미발견 |
| 절 반영/소거: `흐름` 명사와 `흐르게` 속문장 분리 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:167 | 미발견 |
| 절 반영/소거: `흐르게` signed flow convention open issue | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:179 | 미발견 |
| 절 반영/소거: v24.10.0 story-tech support surface R2.1 (docs-first / follow-on) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:198 | 미발견 |
| 절 반영/소거: 4채널 meta/support family | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:200 | 미발견 |
| 절 반영/소거: guardrail | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:206 | 미발견 |
| 절 반영/소거: v24.7.0 story-tech education/AI support surface (docs-first / follow-on) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:223 | 미발견 |
| 절 반영/소거: current-line guardrail | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:230 | 미발견 |
| 절 반영/소거: v24.5.0 core acceleration / DR decision line | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:254 | 미발견 |
| 절 반영/소거: numeric runtime priority | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:256 | 미발견 |
| 절 반영/소거: DR-α — `#태그` 문맥 유일성 (decided) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:264 | 미발견 |
| 절 반영/소거: DR-β — 인라인 람다 function pin 확장 (decided direction) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:269 | 미발견 |
| 절 반영/소거: stored/returned lambda follow-on direction | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:274 | 미발견 |
| 절 반영/소거: relation solve — math-core follow-on | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:278 | 미발견 |
| 절 반영/소거: v24.6.0 representative landed / minimum landed status-sync | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:283 | 미발견 |
| 절 반영/소거: numeric representative exact/factor runtime | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:285 | 미발견 |
| 절 반영/소거: DR-α / DR-β status-sync | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:295 | 미발견 |
| 절 반영/소거: stored/returned lambda snapshot subset | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:300 | 미발견 |
| 절 반영/소거: symbolic minimum / proof minimum | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:310 | 미발견 |
| 절 반영/소거: v24.6.0 story-tech docs-first exact contracts | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:325 | 미발견 |
| 절 반영/소거: `판 {}` / `마당 {}` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:327 | 미발견 |
| 절 반영/소거: `기억 {}` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:335 | 미발견 |
| 절 반영/소거: `갈림 {}` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:344 | 미발견 |
| 절 반영/소거: v24.3.1 current-line hardcut / docs-first additions | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:352 | 미발견 |
| 절 반영/소거: current implementation declaration lock (v24.6.0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:354 | 미발견 |
| 절 반영/소거: removed root / declaration surfaces | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:365 | 미발견 |
| 절 반영/소거: canonical file-leading meta | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:380 | 미발견 |
| 절 반영/소거: docs-first pointers | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:385 | 미발견 |
| 절 반영/소거: v24.0.4 runtime evidence re-balance / docs-only carry-forward | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:389 | 미발견 |
| 절 반영/소거: v24.0.4 runtime golden closure / migration note rectification | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:396 | 미발견 |
| 절 반영/소거: v24.0.2 ordering consistency / metadata carry-forward | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:403 | 미발견 |
| 절 반영/소거: v24.0.1 rectification / metadata hard-cut | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:411 | 미발견 |
| 절 반영/소거: v24.0.0 runtime promotion / capability gate 문장 정렬 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:417 | 미발견 |
| 절 반영/소거: 1) `될때` v1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:419 | 미발견 |
| 절 반영/소거: 2) `인 동안` v1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:431 | 미발견 |
| 절 반영/소거: 3) `건너뛰기.` v1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:442 | 미발견 |
| 절 반영/소거: 4) generic iterable v1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:450 | 미발견 |
| 절 반영/소거: 5) capability/open 경계 메모 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:461 | 미발견 |
| 절 반영/소거: 6) docs-only 슬기 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:466 | 미발견 |
| 절 반영/소거: v23.2.4 execution / product / docs closure bundle | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:489 | 미발견 |
| 절 반영/소거: §V23.2.3A 공식 훅 우회 패턴 (current line) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:491 | 미발견 |
| 절 반영/소거: §V23.2.3B ResultPolicyV1 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:525 | 미발견 |
| 절 반영/소거: §V23.2.3C lesson canonical pattern — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:561 | 미발견 |
| 절 반영/소거: §V23.2.3D 마디 파이프라인 canonical anchor — REFERENCE | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:571 | 미발견 |
| 절 반영/소거: v24.0.6 current-line language closure interpretation memo | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:690 | 미발견 |
| 절 반영/소거: v24.0.5 next-bundle language boundary memo | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:697 | 미발견 |
| 절 반영/소거: v20.20.0 변경 요지 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:711 | 미발견 |
| 절 반영/소거: §0 v19.2.4 핵심 결정(요약) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:721 | 미발견 |
| 절 반영/소거: §0A AI/LLM 코드 생성 규칙 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:743 | 미발견 |
| 절 반영/소거: §1 바닥 규칙(상식 없는 구현체/AI를 위한 최소 문법) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:756 | 미발견 |
| 절 반영/소거: §V18 v19.2.4 통합 개정 (Gate0 기준) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:767 | 미발견 |
| 절 반영/소거: [V18-00] Gate0 어조·활용 제한 (문법 단순화) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:795 | 미발견 |
| 절 반영/소거: [V18-00C] 실행 꼬리(호출 어미) 동치 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:805 | 미발견 |
| 절 반영/소거: [V18-00D] `X` vs `X하` 이름 충돌 금지 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:875 | 미발견 |
| 절 반영/소거: [V18-00A] 정본 어휘·별칭 정책 (순우리말 우선) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:892 | 미발견 |
| 절 반영/소거: §TERM 순우리말 정본/별칭(말모이) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:911 | 미발견 |
| 절 반영/소거: [V18-00A-1] TERM-MAP(정본 용어 맵) — MUST (단일 소스 / Gate0 린트의 근거) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:935 | 미발견 |
| 절 반영/소거: [V18-00B] 버전 표기 정책 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1355 | 미발견 |
| 절 반영/소거: [V18-01] 토막(별칭: 안은문장) `{ }`과 평가/파이프 표지 `}한것 / }인것 / }아닌것 / }하고 / }해서` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1363 | 미발견 |
| 절 반영/소거: [V18-01A] 움직씨(동작) 호출과 `}하고`의 경계 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1371 | 미발견 |
| 절 반영/소거: [V18-02] 상태 대조 `~해보고:` + 분기 `고르기:` (아니면 MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1435 | 미발견 |
| 절 반영/소거: [V18-03] 선택 분기 `고르기:` — Gate0 MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1448 | 미발견 |
| 절 반영/소거: [V18-03A] 분기 표면 선택 안내 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1473 | 미발견 |
| 절 반영/소거: [V18-08A] 시간/훅 표면 선택 안내 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1494 | 미발견 |
| 절 반영/소거: [V18-04] 계약(전제/보장) — Gate0 기본형(명시적 비상구 + 수선) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1510 | 미발견 |
| 절 반영/소거: [V18-05] 운(Random) 소비 규칙 — “리플레이가 절대 깨지지 않게” (Gate0 MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1551 | 미발견 |
| 절 반영/소거: [V18-05A] 머릿말(HeadSpec) — 블록을 여는 말의 통합 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1570 | 미발견 |
| 절 반영/소거: [V18-05B] 순회/반복 머릿말씨 — `~에 대해:` / `~동안:` / `반복:` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1635 | 미발견 |
| 절 반영/소거: 3. 기호 규칙: "생김새가 기능을 정의한다" — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1687 | 미발견 |
| 절 반영/소거: 4. 어휘 및 구문 규칙 (Lexical & Syntax) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1704 | 미발견 |
| 절 반영/소거: 4.1 파일 및 인코딩 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1706 | 미발견 |
| 절 반영/소거: 4.2 식별자와 리터럴 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1711 | 미발견 |
| 절 반영/소거: [REMOVED-ROOT-SURFACE-01] `바탕` / root-hide pragma — removed current line | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1735 | 미발견 |
| 절 반영/소거: [HISTORICAL-GUREUT-DECL-01] `그릇채비 {}` / `붙박이마련 {}` — historical/internal note | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1753 | 미발견 |
| 절 반영/소거: [GUREUT-DECL-03] `채비` — 변수/붙박이 혼합 선언 입력 설탕 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1757 | 미발견 |
| 절 반영/소거: [CHAEVI-MAEGIM-01] `(<값식>) 매김 { ... }`는 `채비` 항목 제어 메타 — MUST (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1801 | 미발견 |
| 절 반영/소거: [EQUAL-DEF-01] `=`는 “정의(Definition)” 전용 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1865 | 미발견 |
| 절 반영/소거: [ASSIGN-ARROW-01] `<-`는 “대입(Assignment)” 전용 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1888 | 미발견 |
| 절 반영/소거: [ASSIGN-ARROW-RIGHT-01] `->` 우향 대입은 “입력 설탕”, 정본은 `<-` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1908 | 미발견 |
| 절 반영/소거: [SIGNAL-ARROW-RESERVE-01] `~~>` 말결/통신 화살표 + Gate0 알림 송신 문장 — MUST (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1941 | 미발견 |
| 절 반영/소거: [ROOT-DECL-01] 바탕칸 선언(등록) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:1976 | 미발견 |
| 절 반영/소거: [IMJA-DECL-01] `임자` 정의와 `제` 자기참조 — MUST (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2001 | 미발견 |
| 절 반영/소거: [ALRIM-DECL-01] `알림씨` 정의 — MUST (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2029 | 미발견 |
| 절 반영/소거: [ALRIM-RECV-HOOK-01] `...를 받으면` 수신 훅 v1 — MUST (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2045 | 미발견 |
| 절 반영/소거: 4.3 문장 경계 (Statement Boundary) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2102 | 미발견 |
| 절 반영/소거: [MAP-DOT-READ-01] `짝맞춤` 점(`.`) 접근 Step A(읽기) — MUST (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2133 | 미발견 |
| 절 반영/소거: [MAP-FIND-OPTIONAL-01] `짝맞춤` 안전 조회(옵션) — `찾기?` (Step C; AGE1+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2162 | 미발견 |
| 절 반영/소거: 4.4 예약어 (Keywords) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2182 | 미발견 |
| 절 반영/소거: 문법 거버넌스 매니페스트 v1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2201 | 미발견 |
| 절 반영/소거: 필드(v0.1) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2206 | 미발견 |
| 절 반영/소거: `state_hash_effect` v1 규칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2223 | 미발견 |
| 절 반영/소거: proof / sim 경계(초기 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2231 | 미발견 |
| 절 반영/소거: 5. 타입 시스템 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2237 | 미발견 |
| 절 반영/소거: 5.1 기본 알 타입 (Primitives) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2239 | 미발견 |
| 절 반영/소거: 5.1.1 복합 알 타입 (Composites) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2251 | 미발견 |
| 절 반영/소거: 5.1.2 흐름(Stream) 값 타입 — SHOULD (AGE1+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2268 | 미발견 |
| 절 반영/소거: 5.2 수(숫자) 구체 타입 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2285 | 미발견 |
| 절 반영/소거: 5.3 수치 정책(det_tier) + 추적 정책(trace_tier) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2381 | 미발견 |
| 절 반영/소거: 5.4 FIXED64_Q32_32 정의 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2413 | 미발견 |
| 절 반영/소거: 5.5 리터럴과 승격 규칙 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2422 | 미발견 |
| 절 반영/소거: 5.6 이름씨 (Struct) 정의 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2453 | 미발견 |
| 절 반영/소거: 5.6.1 구조/주체/관계 경계 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2459 | 미발견 |
| 절 반영/소거: 5.6.2 이름씨 본문 초기화 규칙 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2466 | 미발견 |
| 절 반영/소거: 규칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2547 | 미발견 |
| 절 반영/소거: 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2556 | 미발견 |
| 절 반영/소거: 5.7 값꾸러미 (Value Bundle) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2569 | 미발견 |
| 절 반영/소거: 5.8 묶음씨(레코드/구조체) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2581 | 미발견 |
| 절 반영/소거: 5.8.1 정의(선언) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2586 | 미발견 |
| 절 반영/소거: 5.8.2 값 만들기(구성/형변환) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2602 | 미발견 |
| 절 반영/소거: 5.8.3 필드 접근 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2621 | 미발견 |
| 절 반영/소거: 5.8.4 패턴 분해(선택) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2630 | 미발견 |
| 절 반영/소거: 5.9 튜플 정규화 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2641 | 미발견 |
| 절 반영/소거: 5.10 고름씨 (Tag Union) 정의 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2649 | 미발견 |
| 절 반영/소거: 6. 함수, 연산, 표현식 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2746 | 미발견 |
| 절 반영/소거: 5.11 타입 자리표시자 (Type Parameter) — docs-first / AGE2+ 구현 타겟 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2748 | 미발견 |
| 절 반영/소거: 6.1 함수 정의 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2768 | 미발견 |
| 절 반영/소거: 6.2 이음씨 (중위연산) 정의 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2780 | 미발견 |
| 절 반영/소거: 6.3 표현식 우선순위 (높은 순위부터) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2901 | 미발견 |
| 절 반영/소거: 6.3.1 `!` 토큰 모호성 대장 (disambiguation ledger) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2927 | 미발견 |
| 절 반영/소거: 7. 흐름씨와 일때씨 (Reactive Model) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2954 | 미발견 |
| 절 반영/소거: 7.1 흐름씨 (Stream) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2958 | 미발견 |
| 절 반영/소거: 7.2 피드백과 시간 지연: `이전값보기` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2970 | 미발견 |
| 절 반영/소거: 7.3 흐름씨-훅 위상 분리 (Flow-Hook Phase Separation) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2986 | 미발견 |
| 절 반영/소거: 7.4 일때씨 (Trigger) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3015 | 미발견 |
| 절 반영/소거: 8. 진입점 및 이벤트 기반 실행 모델 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3029 | 미발견 |
| 절 반영/소거: 8.1 표준 진입점(시스템 훅) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3031 | 미발견 |
| 절 반영/소거: 8.2 알림(이벤트) — 닫힌 집합 + 패턴 기반 훅 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3042 | 미발견 |
| 절 반영/소거: 8.3 알림 디스패치(처리) 순서 — MUST (Deterministic) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3093 | 미발견 |
| 절 반영/소거: 8.4 시스템/훅 실행 순서 (Deterministic Sort) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3105 | 미발견 |
| 절 반영/소거: 9. 단위 시스템 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3114 | 미발견 |
| 절 반영/소거: 9.1 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3118 | 미발견 |
| 절 반영/소거: 9.2 표기 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3123 | 미발견 |
| 절 반영/소거: 9.3 단위 정의 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3131 | 미발견 |
| 절 반영/소거: 9.4 차원 대수 (Dimensional Analysis) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3138 | 미발견 |
| 절 반영/소거: 9.5 자동 환산 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3169 | 미발견 |
| 절 반영/소거: 10. 조사/별칭 시스템 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3182 | 미발견 |
| 절 반영/소거: 10.1 매개변수 별칭 선언 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3184 | 미발견 |
| 절 반영/소거: 10.2 세 가지 호출 바인딩 모드 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3193 | 미발견 |
| 절 반영/소거: 10.3 바인딩 스타일 통일 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3256 | 미발견 |
| 절 반영/소거: 10.4 조사/별칭 토큰화 알고리즘 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3267 | 미발견 |
| 절 반영/소거: 10.5 명시적 접미 접착자 (`@`, `:`, `~`) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3317 | 미발견 |
| 절 반영/소거: 10.5 표기 스타일(style): canon / pretty | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3459 | 미발견 |
| 절 반영/소거: 10.6 말씨(dialect): ko + sym3 + (선택한 1개 말씨) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3476 | 미발견 |
| 절 반영/소거: 10.7 SYM3 토큰셋(정본) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3673 | 미발견 |
| 절 반영/소거: 10.8 조사(role) 말씨별 표면형: preferred / accepted | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3698 | 미발견 |
| 절 반영/소거: 10.9 `}` 토막 꼬리 예약(선점) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3757 | 미발견 |
| 절 반영/소거: 11. 실행 어미 (Execution Endings) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3765 | 미발견 |
| 절 반영/소거: 11.1 기본 어미 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3767 | 미발견 |
| 절 반영/소거: 11.2 확장 어미 — MAY (점진 도입) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3775 | 미발견 |
| 절 반영/소거: 11.3 용언 활용 (어간 별칭) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3786 | 미발견 |
| 절 반영/소거: 12. 알림/자원/목표 ID화 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3902 | 미발견 |
| 절 반영/소거: 12.1 정규화 규칙 (컴파일 타임) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3904 | 미발견 |
| 절 반영/소거: 12.2 ID 생성 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3915 | 미발견 |
| 절 반영/소거: 12.3 표준 ID 타입 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3921 | 미발견 |
| 절 반영/소거: 13. 출력 규칙: 상태 기반 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3934 | 미발견 |
| 절 반영/소거: 13.1 핵심 원칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3936 | 미발견 |
| 절 반영/소거: 13.2 표준 해석 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3941 | 미발견 |
| 절 반영/소거: 13.2A `보임 {}` structured view sugar — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3950 | 미발견 |
| 절 반영/소거: 13.3 컴포넌트 수명 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3970 | 미발견 |
| 절 반영/소거: 14. 다중 디스덧댐 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3979 | 미발견 |
| 절 반영/소거: 14.1 규칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3981 | 미발견 |
| 절 반영/소거: 14.2 구체성 판정 알고리즘 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3987 | 미발견 |
| 절 반영/소거: 14.3 애매모호 처리 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:3998 | 미발견 |
| 절 반영/소거: 15. 비목표 (Non-Goals) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4020 | 미발견 |
| 절 반영/소거: 16. 검사 훅: `늘지켜보고` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4031 | 미발견 |
| 절 반영/소거: 16.1 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4033 | 미발견 |
| 절 반영/소거: 16.2 문법 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4039 | 미발견 |
| 절 반영/소거: 16.3 실행 시점 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4047 | 미발견 |
| 절 반영/소거: 16.4 제약 (읽기 전용) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4061 | 미발견 |
| 절 반영/소거: 16.5 거부/실패 정책 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4073 | 미발견 |
| 절 반영/소거: 16.6 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4083 | 미발견 |
| 절 반영/소거: 17. 말결/퍼지 토큰 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4099 | 미발견 |
| 절 반영/소거: 17.1 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4101 | 미발견 |
| 절 반영/소거: 17.2 문법 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4105 | 미발견 |
| 절 반영/소거: 17.3 표준 가중치 매핑 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4116 | 미발견 |
| 절 반영/소거: 17.4 사용 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4129 | 미발견 |
| 절 반영/소거: 17.5 결합 규칙 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4137 | 미발견 |
| 절 반영/소거: 17.6 확장성 — MAY | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4148 | 미발견 |
| 절 반영/소거: 18. GOAP 목표 어미: `-도록/-게` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4158 | 미발견 |
| 절 반영/소거: 18.1 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4160 | 미발견 |
| 절 반영/소거: 18.2 문법 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4164 | 미발견 |
| 절 반영/소거: 18.3 의미론 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4175 | 미발견 |
| 절 반영/소거: 18.4 외부 플래너 연동 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4186 | 미발견 |
| 절 반영/소거: 18.5 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4201 | 미발견 |
| 절 반영/소거: 19. 한국어성: 어순과 생략 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4216 | 미발견 |
| 절 반영/소거: 19.1 어순의 자유 — 인자 순서 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4218 | 미발견 |
| 절 반영/소거: 19.2 생략 규칙 — 옵션 A 채택 (안전) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4239 | 미발견 |
| 절 반영/소거: 19.3 옵션 B/C는 후속 버전으로 보류 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4285 | 미발견 |
| 절 반영/소거: 20. 표준 패턴 갤러리 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4292 | 미발견 |
| 절 반영/소거: 21. 구현 체크리스트 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4297 | 미발견 |
| 절 반영/소거: 22. 참고 자료 (References) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4302 | 미발견 |
| 절 반영/소거: 22.1 관련 언어/시스템 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4304 | 미발견 |
| 절 반영/소거: 22.2 ECS 참고 구현 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4313 | 미발견 |
| 절 반영/소거: 22.3 결정론 참고 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4319 | 미발견 |
| 절 반영/소거: 23. 누리 쿼리 (Nuri Query) 및 군집 제어 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4329 | 미발견 |
| 절 반영/소거: 16.1 목적과 원칙 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4331 | 미발견 |
| 절 반영/소거: 16.2 쿼리 표현식과 필터 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4339 | 미발견 |
| 절 반영/소거: 16.3 군집 실행 블록 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4356 | 미발견 |
| 절 반영/소거: 16.4 스냅샷 의미론 (Snapshot Semantics) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4380 | 미발견 |
| 절 반영/소거: 16.5 구현 요구 사항 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4385 | 미발견 |
| 절 반영/소거: 24. Fixed64 산술 안전 및 결정론적 오류 전이 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4393 | 미발견 |
| 절 반영/소거: 17.1 원칙 (D-STRICT 산술) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4395 | 미발견 |
| 절 반영/소거: 17.2 고장(Fault) 표준 구조 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4405 | 미발견 |
| 절 반영/소거: 17.3 산술 규칙 (핵심 4연산) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4416 | 미발견 |
| 절 반영/소거: 17.4 DetMath 함수 (삼각함수 등)의 결정론 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4447 | 미발견 |
| 절 반영/소거: 25.1 문제 정의 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4491 | 미발견 |
| 절 반영/소거: 25.2 실행 모델: Update + Reactive(패스 루프) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4502 | 미발견 |
| 절 반영/소거: 25.3 한 번의 Reactive 패스 규칙 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4514 | 미발견 |
| 절 반영/소거: 25.4 상한: `ReactiveMaxPass` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4526 | 미발견 |
| 절 반영/소거: 25.5 전파 제한(Propagation) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4537 | 미발견 |
| 절 반영/소거: 26. 쓸감 리터럴 및 @ 기호 용도 명확화 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4551 | 미발견 |
| 절 반영/소거: 19.1 문제 상황 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4555 | 미발견 |
| 절 반영/소거: 19.2 확정 규칙 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4563 | 미발견 |
| 절 반영/소거: 19.3 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4580 | 미발견 |
| 절 반영/소거: 27. PinSpec + 곳간(Registry) 상세 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4603 | 미발견 |
| 절 반영/소거: 27.1 PinSpec 표준 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4605 | 미발견 |
| 절 반영/소거: 27.1.1 갈래 분기: `~에 따라` — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4668 | 미발견 |
| 절 반영/소거: [JOSA-SPLIT-01] 조사 접미사 자동 분리 (Gate0 MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4813 | 미발견 |
| 절 반영/소거: 27.1.2 FieldSpec (postfix 괄호) — RESERVED | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4857 | 미발견 |
| 절 반영/소거: 27.2 곳간(Registry) (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:4871 | 미발견 |
| 절 반영/소거: 20.3 파서 Reverse-Lookup (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5185 | 미발견 |
| 절 반영/소거: 20.4 IDE 자동완성 (SHOULD) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5209 | 미발견 |
| 절 반영/소거: 28. 설탕 구문 계층 (Story-driven Syntax) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5226 | 미발견 |
| 절 반영/소거: §M1 수식 체계 — 삼층(입력/알맹이/표시) 분리 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5275 | 미발견 |
| 절 반영/소거: §M1.1 삼층 분리 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5279 | 미발견 |
| 절 반영/소거: §M1.2 표지 단일화: `수식{...}` (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5284 | 미발견 |
| 절 반영/소거: §M1.3 `(#ascii)` vs `(#ascii1)` (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5300 | 미발견 |
| 절 반영/소거: §M1.4 결정성 규칙 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5335 | 미발견 |
| 절 반영/소거: §M1.5 MathIR v1 정본(요약) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5339 | 미발견 |
| 절 반영/소거: §M2 미분/적분/풀기 — “변환 동사” — Gate0/AGE1 범위 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5350 | 미발견 |
| 절 반영/소거: §M2.1 실행(동작)과 결과(값) 분리 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5356 | 미발견 |
| 절 반영/소거: §M2.2 수식 평가: 풀기 (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5374 | 미발견 |
| 절 반영/소거: §M2.3 미분하기/적분하기 (심볼릭 변환) — Gate0/AGE1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5417 | 미발견 |
| 절 반영/소거: §M2.4 수치 미분/수치 적분(근사) v1 — AGE3+ | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5452 | 미발견 |
| 절 반영/소거: §W1 글무늬(템플릿) 체계 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5524 | 미발견 |
| 절 반영/소거: §W1.1 글무늬 표지(Seed) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5530 | 미발견 |
| 절 반영/소거: §W1.2 자리표시자(placeholder) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5543 | 미발견 |
| 절 반영/소거: §W1.3 @포맷(자리표시자 내부 전용) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5567 | 미발견 |
| 절 반영/소거: §W1.4 채우기(렌더) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5594 | 미발견 |
| 절 반영/소거: §W1.5 맞추기(패턴 매칭) — MAY (AGE1+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5641 | 미발견 |
| 절 반영/소거: §R1 정규식(Regex) — `정규식{}` 리터럴 — AGE3+ | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5676 | 미발견 |
| 절 반영/소거: §R1.1 리터럴 문법 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5686 | 미발견 |
| 절 반영/소거: §R1.2 타입/값 의미 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5698 | 미발견 |
| 절 반영/소거: §R1.3 깃발(Flags) 최소 집합 (AGE3) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5708 | 미발견 |
| 절 반영/소거: §R1.4 표준 API(정본) — 이름만 선반영 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5719 | 미발견 |
| 절 반영/소거: §R1.5 오류 코드(신규) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5730 | 미발견 |
| 절 반영/소거: §R1A 세움(Assertion Block) — `세움{}` / `세움값` — AGE1+ (설계 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5738 | 미발견 |
| 절 반영/소거: §R1B 상태머신(State Machine) — `상태머신{}` / `상태머신값` — AGE1+ (설계 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5778 | 미발견 |
| 절 반영/소거: §R1C 밝히기 / 증명 / 근거로 / 귀납으로 — AGE1+/AGE2+/AGE4 (설계 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5802 | 미발견 |
| 절 반영/소거: 즉시 실행형 `밝히기` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5806 | 미발견 |
| 절 반영/소거: 경우 나누기 / 완전성 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5812 | 미발견 |
| 절 반영/소거: `근거로` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5819 | 미발견 |
| 절 반영/소거: 저장형 `증명{}` / `증명값` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5823 | 미발견 |
| 절 반영/소거: §R1D `세움씨` / `세움씨{}` legacy surface — AGE4 compat/internal (설계 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5830 | 미발견 |
| 절 반영/소거: §R1E `지키기` / `반례찾기` / `해찾기` — AGE2 (설계 고정 / runtime minimum 우선 구현선) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5849 | 미발견 |
| 절 반영/소거: v20.17.0 운영 메모 — proof runtime minimum 우선선 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5861 | 미발견 |
| 절 반영/소거: §R1F 논리 양화 — `낱낱에 대해` / `중 하나가` / `중 딱 하나가` — AGE4 (설계 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5868 | 미발견 |
| 절 반영/소거: §R1G 결정적 게임/시뮬레이션 확장 — `주사위씨` / `사용자입력` / `덩이 {}` / `예약` — AGE2/AGE3 (설계 고정) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5899 | 미발견 |
| 절 반영/소거: `주사위씨` — AGE2 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5903 | 미발견 |
| 절 반영/소거: `사용자입력` / `open.input` — AGE2 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5915 | 미발견 |
| 절 반영/소거: `덩이 {}` / `미루기` — AGE3 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5925 | 미발견 |
| 절 반영/소거: 표준 가지(REFERENCE) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5942 | 미발견 |
| 절 반영/소거: §P1 파이프(해서) — 흐름값/주입/모호성 금지 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5947 | 미발견 |
| 절 반영/소거: §P1.1 파이프의 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5951 | 미발견 |
| 절 반영/소거: §P1.2 호출식만 허용 (PIPE-CALL-ONLY-01, MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5955 | 미발견 |
| 절 반영/소거: §P1.3 암묵 주입 금지 (PIPE-NO-IMPLICIT-INJECT-01, MUST NOT) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5960 | 미발견 |
| 절 반영/소거: §P1.4 단일 전달(권장 정본) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5970 | 미발견 |
| 절 반영/소거: §P1.5 다단계 연쇄(권장 정본) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5976 | 미발견 |
| 절 반영/소거: §P1.6 `{ ... }해서` 파이프 꼬리 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5982 | 미발견 |
| 절 반영/소거: §P1.7 흐름값 주입 알고리즘 (PIPE-INJECT-01, MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5991 | 미발견 |
| 절 반영/소거: §P1.8 예시 (요약) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:5999 | 미발견 |
| 절 반영/소거: §C2 STDLIB (표준 라이브러리) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6006 | 미발견 |
| 절 반영/소거: ⭐ AI-친화 정리: "호출 꼬리 규칙"을 표/예제에 더 강하게 드러내기 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6020 | 미발견 |
| 절 반영/소거: 1. 핀(Pin) 스펙 표기법 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6041 | 미발견 |
| 절 반영/소거: 2. 핵심 함수 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6069 | 미발견 |
| 절 반영/소거: 2.1 출력/로그 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6071 | 미발견 |
| 절 반영/소거: 2.2 알림 (alrim) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6083 | 미발견 |
| 절 반영/소거: 2.3 쓸감 (Asset) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6098 | 미발견 |
| 절 반영/소거: 2.4 목표 (Goal) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6125 | 미발견 |
| 절 반영/소거: 3. DetMath (결정적 수학) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6136 | 미발견 |
| 절 반영/소거: 3.1 원칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6140 | 미발견 |
| 절 반영/소거: 3.2 기본 연산 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6145 | 미발견 |
| 절 반영/소거: 3.3 반올림/분해 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6158 | 미발견 |
| 절 반영/소거: 3.4 삼각함수 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6168 | 미발견 |
| 절 반영/소거: 3.5 지수/로그 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6180 | 미발견 |
| 절 반영/소거: 3.6 기타 수학 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6190 | 미발견 |
| 절 반영/소거: 3.7 범위 제한 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6199 | 미발견 |
| 절 반영/소거: 3.8 보간 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6207 | 미발견 |
| 절 반영/소거: 3.9 벡터 연산 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6215 | 미발견 |
| 절 반영/소거: 3.10 상수 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6225 | 미발견 |
| 절 반영/소거: 3.11 각도 변환 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6233 | 미발견 |
| 절 반영/소거: 4. 단위 시스템 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6242 | 미발견 |
| 절 반영/소거: 4.1 기본 SI 단위 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6244 | 미발견 |
| 절 반영/소거: 4.2 유도 단위 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6254 | 미발견 |
| 절 반영/소거: 4.3 접두사 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6263 | 미발견 |
| 절 반영/소거: 4.3.1 Gate0 지원 단위 목록 (정본) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6271 | 미발견 |
| 절 반영/소거: 4.3.2 환산 실패 규칙 (Gate0, MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6295 | 미발견 |
| 절 반영/소거: 4.3.3 온도 단위 `@K/@C/@F`와 Kelvin 정규화 (Gate0, MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6303 | 미발견 |
| 절 반영/소거: 4.4 차원 검증 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6321 | 미발견 |
| 절 반영/소거: 5. 말결(Nuance) 매핑 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6333 | 미발견 |
| 절 반영/소거: 5.1 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6335 | 미발견 |
| 절 반영/소거: 5.2 표준 매핑 테이블 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6339 | 미발견 |
| 절 반영/소거: 5.3 사용 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6351 | 미발견 |
| 절 반영/소거: 5.4 규칙 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6359 | 미발견 |
| 절 반영/소거: 6. 표준 컴포넌트 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6367 | 미발견 |
| 절 반영/소거: 6.1 물리/공간 (Persistent) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6369 | 미발견 |
| 절 반영/소거: 6.2 시각 (Persistent) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6379 | 미발견 |
| 절 반영/소거: 6.3 고차 상태 (Persistent) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6387 | 미발견 |
| 절 반영/소거: 6.4 임시 (Transient) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6395 | 미발견 |
| 절 반영/소거: 7. 입력 함수 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6407 | 미발견 |
| 절 반영/소거: 7.1 콘솔/텍스트 입력 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6409 | 미발견 |
| 절 반영/소거: 7.2 키보드/마우스 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6417 | 미발견 |
| 절 반영/소거: 8. 결정적 난수 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6428 | 미발견 |
| 절 반영/소거: 8.1 함수 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6430 | 미발견 |
| 절 반영/소거: 8.2 결정론 규칙 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6438 | 미발견 |
| 절 반영/소거: 9. 시간 함수 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6446 | 미발견 |
| 절 반영/소거: 10. 엔티티 관리 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6458 | 미발견 |
| 절 반영/소거: 11. 컴포넌트 접근 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6469 | 미발견 |
| 절 반영/소거: 12. 차림 연산 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6480 | 미발견 |
| 절 반영/소거: 13. 문자열 연산 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6515 | 미발견 |
| 절 반영/소거: 14. 디버그/개발 — MAY | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6531 | 미발견 |
| 절 반영/소거: 15. 모듬 구조 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6541 | 미발견 |
| 절 반영/소거: 16. 호스트 등록 확장 — MAY | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6572 | 미발견 |
| 절 반영/소거: 16.1 `.ddn-lib` 매니페스트 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6574 | 미발견 |
| 절 반영/소거: 16.2 결정성 정책 연동 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6595 | 미발견 |
| 절 반영/소거: 15.1 이름공간(모듬/가지) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6610 | 미발견 |
| 절 반영/소거: 15.1.1 기본 규칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6615 | 미발견 |
| 절 반영/소거: 15.1.2 모듬의 선택/별칭(툴체인 책임) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6627 | 미발견 |
| 절 반영/소거: 15.1.3 정본 원칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6633 | 미발견 |
| 절 반영/소거: 17. 산술 연산 의미론 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6638 | 미발견 |
| 절 반영/소거: 17.1 나눗셈 `/` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6640 | 미발견 |
| 절 반영/소거: 17.2 나머지 `%` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6651 | 미발견 |
| 절 반영/소거: 17.3 거듭제곱 `^` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6656 | 미발견 |
| 절 반영/소거: 17.4 정수 오버플로 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6663 | 미발견 |
| 절 반영/소거: 추가 §AI. AI-GYM 전용 함수 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6668 | 미발견 |
| 절 반영/소거: AI.1 끝내 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6670 | 미발견 |
| 절 반영/소거: AI.2 지금관찰 (MUST, 별칭: 눈떠) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6691 | 미발견 |
| 절 반영/소거: AI.3 보상 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6712 | 미발견 |
| 절 반영/소거: 추가 §뉘. 말결 토큰 매핑 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6735 | 미발견 |
| 절 반영/소거: 뉘.1 말결 토큰 정의 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6737 | 미발견 |
| 절 반영/소거: 뉘.2 사용 예시 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6755 | 미발견 |
| 절 반영/소거: 뉘.3 부정과의 조합 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6769 | 미발견 |
| 절 반영/소거: 추가 §DetMath. 결정론적 수학 함수 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6781 | 미발견 |
| 절 반영/소거: DetMath.1 룩업 테이블 (LUT) 기반 구현 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6783 | 미발견 |
| 절 반영/소거: DetMath.2 입력/출력 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6797 | 미발견 |
| 절 반영/소거: DetMath.3 도메인 오류 처리 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6804 | 미발견 |
| 절 반영/소거: 추가 §입력키. 입력키 표준 함수(compat/strict) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6821 | 미발견 |
| 절 반영/소거: 입력키.1 `입력키` (compat) — MAY (AGE1) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6823 | 미발견 |
| 절 반영/소거: 입력키.2 `입력키?` (권장) — SHOULD (AGE1+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6834 | 미발견 |
| 절 반영/소거: 입력키.3 `입력키!` (엄격) — SHOULD (AGE2) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6842 | 미발견 |
| 절 반영/소거: 추가 §글. 글바꾸기 표준 함수 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6852 | 미발견 |
| 절 반영/소거: 글.1 `글바꾸기` (안전) — MAY | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6854 | 미발견 |
| 절 반영/소거: 글.2 `글바꾸기!` (엄격) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6865 | 미발견 |
| 절 반영/소거: 추가 §매마디 / 매틱 호환 — MAY (Gate0/Compat) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6873 | 미발견 |
| 절 반영/소거: PinSpec(핀 스펙) — 상세 구현 노트 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6887 | 미발견 |
| 절 반영/소거: 추가 §단위. 단위 시스템 확장 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6892 | 미발견 |
| 절 반영/소거: 단위.1 @ 기호 표준화 (재확인) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6894 | 미발견 |
| 절 반영/소거: 단위.2 자동 환산 규칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6906 | 미발견 |
| 절 반영/소거: 추가 §쓸감. 자원 핸들 시스템 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6919 | 미발견 |
| 절 반영/소거: 쓸감.1 쓸감 리터럴 (MUST) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6921 | 미발견 |
| 절 반영/소거: 쓸감.2 @ 문법 설탕 (MAY) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6934 | 미발견 |
| 절 반영/소거: GOAP (목표 지향 계획) — 예약 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6948 | 미발견 |
| 절 반영/소거: 추가 §G. 확장 구문 프레임 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6954 | 미발견 |
| 절 반영/소거: §G.1 원칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6958 | 미발견 |
| 절 반영/소거: §G.2 확장 블록(예시) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6965 | 미발견 |
| 절 반영/소거: [NAMING-LINT-01] 조사 단독 금지 + 조사 접미사 자동 분리 — MUST/SHOULD (Gate0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:6971 | 미발견 |
| 절 반영/소거: §NEW (v20.1.2) 가지(gaji) 메타데이터와 기능 게이트 선언 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7010 | 미발견 |
| 절 반영/소거: 1) 가지 메타데이터(gaji.toml) — 정본 (B안) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7014 | 미발견 |
| 절 반영/소거: 2) 강제 규칙 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7032 | 미발견 |
| 절 반영/소거: AGE2(Open) 지시문 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7038 | 미발견 |
| 절 반영/소거: AGE3(Bogae) 출력 결정성 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7041 | 미발견 |
| 절 반영/소거: 좌표변환 표준(격자→픽셀) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7044 | 미발견 |
| 절 반영/소거: 용어(순우리말) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7047 | 미발견 |
| 절 반영/소거: 임자무리(임자가리킴 차림) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7050 | 미발견 |
| 절 반영/소거: 셈씨 규칙 보강(v20.4.1) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7055 | 미발견 |
| 절 반영/소거: [SEMSI-RESULT-01] 셈씨 결과칸(결과 이름 대입) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7057 | 미발견 |
| 절 반영/소거: [FILE-META-01] 파일 선두 메타는 `설정 {}`만 허용 — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7092 | 미발견 |
| 절 반영/소거: [BOGAE-CONTRACT-01] 보개 계약(뷰 인터페이스) — SHOULD (교육/팩/셈그림) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7121 | 미발견 |
| 절 반영/소거: [BOGAE-UPDATE-01] 보개 갱신 규칙: update=append\|replace (+ 선택적 tick) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7148 | 미발견 |
| 절 반영/소거: [INPUT-REGISTRY-01] 입력원 레지스트리 — SHOULD (셈그림) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7166 | 미발견 |
| 절 반영/소거: [BOGAE-VIEWSET-01] 필수보기(교과/lesson) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7186 | 미발견 |
| 절 반영/소거: [BOGAE-IF-01] 보개별 필수 채널(최소) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7204 | 미발견 |
| 절 반영/소거: [CROSS-VIEW-01] 보개 동기화(맞물림) — SHOULD (교육 효과) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7241 | 미발견 |
| 절 반영/소거: [STRUCTURE-IDENTITY-01] 보개/구조의 정체성(생각의 지도) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7261 | 미발견 |
| 절 반영/소거: [OVERLAY-PEDAGOGY-01] 비교(겹)의 교육학 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7280 | 미발견 |
| 절 반영/소거: [SCENE-BLOCK-01] `보개마당 { ... }` — SHOULD (한국어 Manim, AGE1+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7296 | 미발견 |
| 절 반영/소거: [NARRATIVE-TRACK-01] 해설/자막 트랙 — SHOULD (교육/AI 연동) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7355 | 미발견 |
| 절 반영/소거: [TIMEBOOKMARK-01] 시간의 조각(책갈피) — SHOULD (타임줄/디버그/되감기) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7373 | 미발견 |
| 절 반영/소거: [INVERSE-CONTROL-01] 만져지는 수식(역조작) — SHOULD (control 입력원) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7387 | 미발견 |
| 절 반영/소거: [SPACE2D-DRAWLIST-01] space2d drawlist primitive — MUST (C+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7401 | 미발견 |
| 절 반영/소거: [CONTROL-META-01] control 정의를 DDN 메타(`#키: 값`)로 제공 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7420 | 미발견 |
| 절 반영/소거: [OVERLAY-PARAM-01] 오버레이(겹) 정본: 파라미터 비교 baseline+variant — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7441 | 미발견 |
| 절 반영/소거: [SCENE-DRAW-MODE-01] `그려지기` 구현 모드: A(progress) → B(append) 확장 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7454 | 미발견 |
| 절 반영/소거: [PHYS-BACKEND-01] 물리 백엔드: A(계산) + B(매임/엔진) 공존 — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7468 | 미발견 |
| 절 반영/소거: [GRAPH-KIND-01] graph 보개 종류(`graph_kind`) — MUST (v0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7481 | 미발견 |
| 절 반영/소거: [GRAPH-AXIS-META-01] graph 축 메타(`x_kind/unit/label`, `y_kind/unit/label`) — MUST | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7496 | 미발견 |
| 절 반영/소거: [GRAPH-DATA-01] graph_kind별 최소 데이터 채널 — SHOULD (v0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7509 | 미발견 |
| 절 반영/소거: [SPACE2D-DRAWLIST-02] space2d optional primitive 확장(C++) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7519 | 미발견 |
| 절 반영/소거: [SPACE2D-3D-01] 3D 표현 확장(보개 증식 없이) — SHOULD | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7525 | 미발견 |
| 절 반영/소거: §MOD1 모듬(모듈) 시스템 v1 (AGE4+) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7536 | 미발견 |
| 절 반영/소거: [MOD1-00] 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7542 | 미발견 |
| 절 반영/소거: [MOD1-01] `쓰임 {}` — 모듬 들여오기(Import) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7549 | 미발견 |
| 절 반영/소거: [MOD1-02] `드러냄 {}` — 모듬 내보내기(Export) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7575 | 미발견 |
| 절 반영/소거: [MOD1-03] 모듬 사용 표면 — `별명.심볼` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7598 | 미발견 |
| 절 반영/소거: [MOD1-04] 레거시/호환(compat-only) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7604 | 미발견 |
| 절 반영/소거: [MOD1-05] 진단 코드(정본) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7612 | 미발견 |
| 절 반영/소거: §PROJECT1 프로젝트 루트 선언 블록 (AGE2+ designed) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7633 | 미발견 |
| 절 반영/소거: [PROJECT1-00] 목적 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7639 | 미발견 |
| 절 반영/소거: [PROJECT1-01] canonical 위치와 범위 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7644 | 미발견 |
| 절 반영/소거: [PROJECT1-02] 역할 경계 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7651 | 미발견 |
| 절 반영/소거: [PROJECT1-03] 최소 표면 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7657 | 미발견 |
| 절 반영/소거: [PROJECT1-04] `실행기본 {}` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7689 | 미발견 |
| 절 반영/소거: [PROJECT1-05] manifest/상태표 메모 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7694 | 미발견 |
| 절 반영/소거: v20.26.0 부록 A — `설정 {}` 단일 헤더 메타 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7700 | 미발견 |
| 절 반영/소거: v20.26.0 부록 B — `설정.보개` 와 `매김 {}` 경계 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7724 | 미발견 |
| 절 반영/소거: v20.26.0 부록 C — `모양 {}` / `모양씨` / `겹보기씨` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7749 | 미발견 |
| 절 반영/소거: C-1. `모양 {}` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7751 | 미발견 |
| 절 반영/소거: C-2. `모양씨` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7772 | 미발견 |
| 절 반영/소거: C-3. `겹보기씨` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7781 | 미발견 |
| 절 반영/소거: v23.2.5 부록 C-4 — `설정.슬기` docs-first representative note | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7805 | 미발견 |
| 절 반영/소거: v20.26.0 부록 D — 문법 상태표 v1 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7864 | 미발견 |
| 절 반영/소거: v20.26.0 부록 E — 숫자 2층 구조 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7896 | 미발견 |
| 절 반영/소거: v22.0.0 부록 E — 시간 / 훅 / lifecycle / reset | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7914 | 미발견 |
| 절 반영/소거: E-1. 시간축 계약 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7916 | 미발견 |
| 절 반영/소거: E-2. 훅 family 4분할 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7922 | 미발견 |
| 절 반영/소거: E-3. `판` / `마당` lifecycle unit | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7932 | 미발견 |
| 절 반영/소거: E-4. lifecycle 동사 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7961 | 미발견 |
| 절 반영/소거: E-5. reset kind 계약 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7969 | 미발견 |
| 절 반영/소거: E-6. `덩이 {}` canonical atomic commit block | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7984 | 미발견 |
| 절 반영/소거: E-7. 상태 replay vs 보기 playback | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8000 | 미발견 |
| 절 반영/소거: E-8. 문법 상태표 v1 — lifecycle addendum | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8005 | 미발견 |
| 절 반영/소거: E-7A. action space 정본 문법 — `매김 { 가늠 {..} 갈래 {..} }` | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8006 | 미발견 |
| 절 반영/소거: E-7B. 실행축 차림새 / preset canonical (v23.0.1) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8047 | 미발견 |
| 절 반영/소거: E-7C. `action_status -> 보람` 경계 (v23.2.0) | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8080 | 미발견 |
| 절 반영/소거: v23.2.6 부록 C-5 — 배움틀 docs-first representative note | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8102 | 미발견 |
| 절 반영/소거: 말씨 tier 승격 기준 | 해당 구성물 없음 | - | - | docs-first | docs/ssot/ssot/SSOT_LANG_v24.12.9.md:8138 | 미발견 |
