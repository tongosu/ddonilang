# Tests

## Manual run
- `teul-cli latency simulate --L 3 --mode fixed --count 3 --seed 0`
- `teul-cli latency simulate --L 3 --mode jitter --count 4 --seed 7`
- `teul-cli latency simulate --L 18446744073709551615 --mode fixed --count 2 --seed 0`
- `teul-cli latency simulate --L 0 --mode fixed --count 3 --seed 0 --current-madi 2`
- `teul-cli run pack/gogae5_w49_latency_madi/input_replay.ddn --madi 4 --seed 0x0 --sam pack/gogae5_w49_latency_madi/sam/right_hold_2ticks.input.bin --latency-madi 0 --geoul-out build/w49_geoul_l0 --trace-tier T-OFF`
- `teul-cli run pack/gogae5_w49_latency_madi/input_replay.ddn --madi 4 --seed 0x0 --sam pack/gogae5_w49_latency_madi/sam/right_hold_2ticks.input.bin --latency-madi 10 --geoul-out build/w49_geoul_l10 --trace-tier T-OFF`
- `teul-cli replay verify --geoul build/w49_geoul_l0`
- `teul-cli replay verify --geoul build/w49_geoul_l10`
