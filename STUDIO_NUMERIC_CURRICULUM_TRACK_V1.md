# STUDIO_NUMERIC_CURRICULUM_TRACK_V1

## Summary
`STUDIO_NUMERIC_CURRICULUM_TRACK_V1` packages the closed numeric solver and simulation evidence into a Studio curriculum planning track. It is a documentation/checker/evidence step only.

This stage does not add Studio UI, lesson schema, stdlib surface, parser/frontdoor grammar, runtime semantics, or automatic solver behavior.

## Current Numeric Anchors
- `ODE_TICK_LOOP_LESSON_BASELINE_V1`
- `ODE_METHOD_COMPARISON_V1`
- `NUMERIC_ROOT_FINDING_V1`
- `POLYNOMIAL_SOLVE_MINIMUM_V1`
- `CONSTRAINT_SOLVE_REBASE_V1`
- `LINEAR_INEQUALITY_SOLVE_MINIMUM_V1`

## Studio Lesson Anchors
The current active representative lesson allowlist already contains the minimum cross-subject anchors needed for a numeric track:

- `rep_math_function_line_v1`
- `rep_phys_projectile_xy_v1`
- `rep_econ_supply_demand_tax_v1`

These are anchors only. V1 does not mutate the allowlist and does not create a new lesson type.

## Track Modules
1. `simulation_time_step`
   - Uses `적분.오일러`, `적분.반암시적오일러`, and the ODE tick-loop/method-comparison packs.
2. `root_finding`
   - Uses `수치해.이분법` and `numeric_root_finding_bisection_v1`.
3. `exact_polynomial`
   - Uses `다항식.풀기` and `polynomial_solve_minimum_v1`.
4. `linear_inequality_interval`
   - Uses `선형부등식.풀기` and `linear_inequality_solve_minimum_v1`.
5. `post_solve_range_reporting`
   - Uses the connect endpoint solve/range/report/check closure through `connect_flow_v1v_closure_v1`.

## Boundary
- No new lesson schema.
- No active allowlist mutation.
- No browser UI change.
- No automatic solve in Studio.
- No nonlinear/general LP/CSP claim beyond the closed evidence.
- No public release action.
- No `docs/ssot/**` modification.

## Evidence
- `pack/studio_numeric_curriculum_track_v1`
- `tests/run_studio_numeric_curriculum_track_check.py`
- `docs/studio/NUMERIC_CURRICULUM_TRACK_V1.md`

## Next
The recommended next implementation item is `SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1`: expose this sealed track as a local browse/filter affordance in the Studio lesson library without changing lesson schema.
