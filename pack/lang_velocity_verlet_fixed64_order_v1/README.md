# lang_velocity_verlet_fixed64_order_v1

This pack records the Fixed64 operation order and one deterministic smoke trace for the `속도 베를레` D-STRICT candidate.

It is evidence-only:

- no runtime integrator landing
- no stdlib surface landing
- no parser/frontdoor change
- no `docs/ssot/**` edit

The seed trace is a one-tick harmonic oscillator:

- `x0 = 1`
- `v0 = 0`
- `dt = 0.25`
- `v_half = -0.125`
- `x1 = 0.96875`
- `v1 = -0.24609375`
- `energy1 = 0.9990386962890625`
