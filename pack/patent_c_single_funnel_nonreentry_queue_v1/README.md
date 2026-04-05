# patent_c_single_funnel_nonreentry_queue_v1

특허 C 증빙용 단일 깔때기 비재진입 큐 D-PACK.

검증 항목:

- `받으면` 훅 내부 `~~>` 연쇄 발송이 즉시 재귀하지 않고 Next Pass로 지연
- 훅 디스패치가 비재진입 FIFO 순서로 진행
- 발신자 생략 시 외부 전송은 `누리`, 임자 내부 전송은 `제(현재 임자)`로 자동 주입

실행:

- `python tests/run_pack_golden.py patent_c_single_funnel_nonreentry_queue_v1`
