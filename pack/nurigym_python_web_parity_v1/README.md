# nurigym_python_web_parity_v1

This pack records a bounded Python/Web parity gate for ROADMAP_V2 `아-3`.

The Python checker reruns existing NuriGym golden packs and then invokes a Web-side JavaScript evaluator for deterministic GridMaze and Bandit episode artifacts. The Web evaluator recomputes step projections from the same input JSON and compares them with the CLI-generated `nurigym.dataset.jsonl` rows.

This does not claim CartPole/Pendulum physics parity, full browser runtime integration, public registry publication, training workflow, or a new NuriGym runtime.

## Progress

- Current stage: A3 NuriGym Python/Web parity 5/5 = 100%
- ROADMAP_V2 matrix behavior-closed: 77/90 = 86%
- ROADMAP_V2 docs-closed: 5/90 = 6%
- ROADMAP_V2 pack evidence reference: 79/90 = 88%
- Studio-local super-long plan: 9/18 = 50%
