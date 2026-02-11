# D-PACK: gogae4_w38_emotion_wave

## 목적
- 감정씨(valence/arousal) 2축 모델과 포화(clamp) 산술을 검증한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- 감정더하기 누적 결과가 clamp 규칙을 따른다.
- state_hash가 결정적으로 고정된다.
