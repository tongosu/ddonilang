# seulgi_v1

Seulgi v1 contract skeleton pack.

The pack focuses on boundary semantics, not model implementation:

- Seulgi is an async advisor role.
- World mutation cannot happen directly from Seulgi.
- Inputs enter through `sam_input_snapshot`.
- Replay never re-calls Seulgi source/LLM.
- Gatekeeper rejection must keep state uncommitted.

The check entry is:

- `python tests/run_seulgi_v1_pack_check.py`
