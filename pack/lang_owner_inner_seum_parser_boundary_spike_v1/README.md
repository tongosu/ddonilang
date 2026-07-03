# lang_owner_inner_seum_parser_boundary_spike_v1

This pack records the parser/frontdoor boundary for owner-local `세움{}` inside an `임자` seed.

It lands a narrow product parser boundary:

- `성질 {}` inside `임자` body
- `세움 { 위치' =:= 속도. }` inside `임자` body
- `owner_inner_seum_canon_rows` inspection helper

It does not land:

- equation solving
- derivative semantics
- runtime integrator changes
- stdlib surface changes
- `docs/ssot/**` edits
