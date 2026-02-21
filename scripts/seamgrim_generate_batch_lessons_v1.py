#!/usr/bin/env python3
"""Generate rewritten Seamgrim lessons from generation plan."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


TEMPLATES = {
    "physics_motion_v1": """#이름: {name}
#설명: {desc}
#control: x0:수=0 [-20..20] step=0.5; v0:수=1 [-20..20] step=0.5; a:수=0.2 [-10..10] step=0.1; dt:수=0.1 [0.02..1] step=0.01; t_max:수=8 [1..30] step=0.5

x0 <- 0.
v0 <- 1.
a <- 0.2.
dt <- 0.1.
t_max <- 8.

(시작)할때 {{
  t <- 0.
  x <- x0.
  v <- v0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    v <- v + a * dt.
    x <- x + v * dt.
    t <- t + dt.

    t 보여주기.
    x 보여주기.
    v 보여주기.
  }}.
}}.
""",
    "physics_orbit_v1": """#이름: {name}
#설명: {desc}
#control: x0:수=1 [0.2..5] step=0.1; y0:수=0 [-5..5] step=0.1; vx0:수=0 [-5..5] step=0.1; vy0:수=1 [0.1..5] step=0.1; mu:수=1 [0.2..10] step=0.1; dt:수=0.02 [0.005..0.1] step=0.005; t_max:수=8 [1..30] step=0.5

x0 <- 1.
y0 <- 0.
vx0 <- 0.
vy0 <- 1.
mu <- 1.
dt <- 0.02.
t_max <- 8.

(시작)할때 {{
  t <- 0.
  x <- x0.
  y <- y0.
  vx <- vx0.
  vy <- vy0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    r2 <- x * x + y * y + 0.0001.
    ax <- 0 - mu * x / r2.
    ay <- 0 - mu * y / r2.
    vx <- vx + ax * dt.
    vy <- vy + ay * dt.
    x <- x + vx * dt.
    y <- y + vy * dt.
    t <- t + dt.

    x 보여주기.
    y 보여주기.
  }}.
}}.
""",
    "physics_thermal_v1": """#이름: {name}
#설명: {desc}
#control: T_env:수=20 [-10..50] step=0.5; T0:수=90 [0..150] step=0.5; k:수=0.15 [0.01..1] step=0.01; dt:수=0.1 [0.02..1] step=0.01; t_max:수=12 [1..50] step=0.5

T_env <- 20.
T0 <- 90.
k <- 0.15.
dt <- 0.1.
t_max <- 12.

(시작)할때 {{
  t <- 0.
  T <- T0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    dT <- 0 - k * (T - T_env).
    T <- T + dT * dt.
    t <- t + dt.

    t 보여주기.
    T 보여주기.
  }}.
}}.
""",
    "physics_oscillator_v1": """#이름: {name}
#설명: {desc}
#control: x0:수=1 [-5..5] step=0.1; v0:수=0 [-5..5] step=0.1; k:수=1 [0.1..8] step=0.1; c:수=0.1 [0..2] step=0.05; dt:수=0.02 [0.005..0.1] step=0.005; t_max:수=8 [1..30] step=0.5

x0 <- 1.
v0 <- 0.
k <- 1.
c <- 0.1.
dt <- 0.02.
t_max <- 8.

(시작)할때 {{
  t <- 0.
  x <- x0.
  v <- v0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    a <- 0 - k * x - c * v.
    v <- v + a * dt.
    x <- x + v * dt.
    t <- t + dt.

    t 보여주기.
    x 보여주기.
    v 보여주기.
  }}.
}}.
""",
    "math_quadratic_integral_v1": """#이름: {name}
#설명: {desc}
#control: a:수=1 [-5..5] step=0.1; b:수=0 [-8..8] step=0.1; c:수=0 [-20..20] step=0.1; x_start:수=-3 [-20..20] step=0.5; x_end:수=3 [-20..20] step=0.5; dx:수=0.1 [0.02..1] step=0.02

a <- 1.
b <- 0.
c <- 0.
x_start <- -3.
x_end <- 3.
dx <- 0.1.

(시작)할때 {{
  x <- x_start.
  area <- 0.
}}.

(매마디)마다 {{
  {{ x <= x_end }}인것 일때 {{
    y <- a * x * x + b * x + c.
    nx <- x + dx.
    ny <- a * nx * nx + b * nx + c.
    area <- area + (y + ny) * dx / 2.

    x 보여주기.
    y 보여주기.
    area 보여주기.

    x <- nx.
  }}.
}}.
""",
    "math_linear_v1": """#이름: {name}
#설명: {desc}
#control: m:수=1 [-10..10] step=0.1; b:수=0 [-20..20] step=0.1; x_start:수=-5 [-20..20] step=0.5; x_end:수=5 [-20..20] step=0.5; dx:수=0.25 [0.05..1] step=0.05

m <- 1.
b <- 0.
x_start <- -5.
x_end <- 5.
dx <- 0.25.

(시작)할때 {{
  x <- x_start.
}}.

(매마디)마다 {{
  {{ x <= x_end }}인것 일때 {{
    y <- m * x + b.
    x 보여주기.
    y 보여주기.
    x <- x + dx.
  }}.
}}.
""",
    "math_stats_v1": """#이름: {name}
#설명: {desc}
#control: x0:수=10 [0..100] step=1; trend:수=0.4 [-5..5] step=0.1; noise:수=0.2 [0..2] step=0.1; dt:수=1 [1..1] step=1; t_max:수=20 [5..60] step=1

x0 <- 10.
trend <- 0.4.
noise <- 0.2.
dt <- 1.
t_max <- 20.

(시작)할때 {{
  t <- 0.
  x <- x0.
  mean <- x0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    pulse <- ((t % 4) - 1.5) * noise.
    x <- x + trend + pulse.
    mean <- (mean * t + x) / (t + 1).
    t <- t + dt.

    t 보여주기.
    x 보여주기.
    mean 보여주기.
  }}.
}}.
""",
    "math_sequence_v1": """#이름: {name}
#설명: {desc}
#control: a0:수=1 [-20..20] step=0.5; d:수=2 [-10..10] step=0.5; r:수=1.1 [0.2..3] step=0.05; blend:수=0.5 [0..1] step=0.05; n_max:수=20 [5..80] step=1

a0 <- 1.
d <- 2.
r <- 1.1.
blend <- 0.5.
n_max <- 20.

(시작)할때 {{
  n <- 0.
  a_n <- a0.
  sum <- 0.
}}.

(매마디)마다 {{
  {{ n <= n_max }}인것 일때 {{
    arith <- a0 + d * n.
    geom <- a0 * (1 + (r - 1) * n).
    a_n <- arith * (1 - blend) + geom * blend.
    sum <- sum + a_n.
    n <- n + 1.

    n 보여주기.
    a_n 보여주기.
    sum 보여주기.
  }}.
}}.
""",
    "math_matrix_v1": """#이름: {name}
#설명: {desc}
#control: a:수=1 [-10..10] step=0.5; b:수=2 [-10..10] step=0.5; c:수=3 [-10..10] step=0.5; d:수=4 [-10..10] step=0.5; delta:수=0.1 [-2..2] step=0.1; n_max:수=20 [5..80] step=1

a <- 1.
b <- 2.
c <- 3.
d <- 4.
delta <- 0.1.
n_max <- 20.

(시작)할때 {{
  n <- 0.
}}.

(매마디)마다 {{
  {{ n <= n_max }}인것 일때 {{
    trace <- a + d.
    det <- a * d - b * c.
    n <- n + 1.
    a <- a + delta.
    d <- d - delta.

    n 보여주기.
    trace 보여주기.
    det 보여주기.
  }}.
}}.
""",
    "economy_supply_demand_v1": """#이름: {name}
#설명: {desc}
#control: d0:수=120 [30..300] step=1; d1:수=2 [0.5..6] step=0.1; s0:수=20 [0..150] step=1; s1:수=1.2 [0.2..4] step=0.1; p0:수=10 [1..80] step=0.5; k:수=0.08 [0.01..0.3] step=0.01; dt:수=0.1 [0.02..0.5] step=0.01; t_max:수=10 [2..30] step=0.5

d0 <- 120.
d1 <- 2.
s0 <- 20.
s1 <- 1.2.
p0 <- 10.
k <- 0.08.
dt <- 0.1.
t_max <- 10.

(시작)할때 {{
  t <- 0.
  p <- p0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    qd <- d0 - d1 * p.
    qs <- s0 + s1 * p.
    gap <- qd - qs.
    p <- p + k * gap * dt.
    t <- t + dt.

    t 보여주기.
    p 보여주기.
    qd 보여주기.
    qs 보여주기.
    gap 보여주기.
  }}.
}}.
""",
    "economy_growth_v1": """#이름: {name}
#설명: {desc}
#control: y0:수=100 [10..500] step=1; g:수=0.03 [-0.05..0.2] step=0.005; dt:수=1 [1..1] step=1; t_max:수=20 [5..80] step=1

y0 <- 100.
g <- 0.03.
dt <- 1.
t_max <- 20.

(시작)할때 {{
  t <- 0.
  y <- y0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    y <- y * (1 + g * dt).
    t <- t + dt.

    t 보여주기.
    y 보여주기.
  }}.
}}.
""",
    "economy_budget_v1": """#이름: {name}
#설명: {desc}
#control: income:수=100 [20..500] step=1; consume_rate:수=0.7 [0..1] step=0.01; dt:수=1 [1..1] step=1; t_max:수=20 [5..80] step=1

income <- 100.
consume_rate <- 0.7.
dt <- 1.
t_max <- 20.

(시작)할때 {{
  t <- 0.
  saving <- 0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    spend <- income * consume_rate.
    save_delta <- income - spend.
    saving <- saving + save_delta.
    t <- t + dt.

    t 보여주기.
    spend 보여주기.
    saving 보여주기.
  }}.
}}.
""",
    "economy_stock_flow_v1": """#이름: {name}
#설명: {desc}
#control: stock0:수=100 [0..500] step=1; inflow:수=20 [0..200] step=1; out_rate:수=0.15 [0..1] step=0.01; dt:수=1 [1..1] step=1; t_max:수=20 [5..80] step=1

stock0 <- 100.
inflow <- 20.
out_rate <- 0.15.
dt <- 1.
t_max <- 20.

(시작)할때 {{
  t <- 0.
  stock <- stock0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    outflow <- stock * out_rate.
    stock <- stock + (inflow - outflow) * dt.
    t <- t + dt.

    t 보여주기.
    inflow 보여주기.
    outflow 보여주기.
    stock 보여주기.
  }}.
}}.
""",
    "generic_series_v1": """#이름: {name}
#설명: {desc}
#control: x0:수=0 [-20..20] step=0.5; rate:수=1 [-10..10] step=0.1; dt:수=0.1 [0.02..1] step=0.01; t_max:수=8 [1..30] step=0.5

x0 <- 0.
rate <- 1.
dt <- 0.1.
t_max <- 8.

(시작)할때 {{
  t <- 0.
  x <- x0.
}}.

(매마디)마다 {{
  {{ t <= t_max }}인것 일때 {{
    x <- x + rate * dt.
    t <- t + dt.

    t 보여주기.
    x 보여주기.
  }}.
}}.
""",
}


def normalize_subject(value: str) -> str:
    lower = value.strip().lower()
    if lower == "econ":
        return "economy"
    return lower


def choose_template(subject: str, lesson_id: str) -> tuple[str, str]:
    lower_id = lesson_id.lower()
    if subject == "physics":
        if any(token in lower_id for token in ("orbit", "projectile", "centripetal")):
            return "physics_orbit_v1", TEMPLATES["physics_orbit_v1"]
        if any(token in lower_id for token in ("thermal", "cooling", "heat", "gas", "boiling")):
            return "physics_thermal_v1", TEMPLATES["physics_thermal_v1"]
        if any(token in lower_id for token in ("harmonic", "damped", "resonance", "pendulum", "wave", "spring")):
            return "physics_oscillator_v1", TEMPLATES["physics_oscillator_v1"]
        return "physics_motion_v1", TEMPLATES["physics_motion_v1"]
    if subject == "math":
        if any(token in lower_id for token in ("line", "linear", "slope")):
            return "math_linear_v1", TEMPLATES["math_linear_v1"]
        if any(token in lower_id for token in ("matrix", "vector", "det")):
            return "math_matrix_v1", TEMPLATES["math_matrix_v1"]
        if any(token in lower_id for token in ("series", "sequence", "progression")):
            return "math_sequence_v1", TEMPLATES["math_sequence_v1"]
        if any(token in lower_id for token in ("stats", "probability", "survey", "mean")):
            return "math_stats_v1", TEMPLATES["math_stats_v1"]
        return "math_quadratic_integral_v1", TEMPLATES["math_quadratic_integral_v1"]
    if subject == "economy":
        if any(token in lower_id for token in ("growth", "productivity", "population", "timeline")):
            return "economy_growth_v1", TEMPLATES["economy_growth_v1"]
        if any(token in lower_id for token in ("budget", "saving", "allowance", "unit_price")):
            return "economy_budget_v1", TEMPLATES["economy_budget_v1"]
        if any(token in lower_id for token in ("stock", "flow", "inventory", "store")):
            return "economy_stock_flow_v1", TEMPLATES["economy_stock_flow_v1"]
        return "economy_supply_demand_v1", TEMPLATES["economy_supply_demand_v1"]
    return "generic_series_v1", TEMPLATES["generic_series_v1"]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--plan",
        default="build/reports/seamgrim_generation_plan_v1.json",
        help="Generation plan JSON path",
    )
    parser.add_argument(
        "--out-root",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1",
        help="Output root for generated lessons",
    )
    parser.add_argument(
        "--manifest-out",
        default="solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson",
        help="Generated manifest path",
    )
    parser.add_argument(
        "--subjects",
        default="physics,math,economy",
        help="Comma-separated subjects to include",
    )
    parser.add_argument(
        "--max-per-subject",
        type=int,
        default=2,
        help="Maximum generated lessons per subject",
    )
    args = parser.parse_args()

    plan_path = Path(args.plan)
    plan = json.loads(plan_path.read_text(encoding="utf-8"))
    rows = list(plan.get("items", []))
    out_root = Path(args.out_root)
    out_root.mkdir(parents=True, exist_ok=True)

    requested_subjects = [normalize_subject(x) for x in args.subjects.split(",") if x.strip()]
    buckets: dict[str, list[dict]] = {subject: [] for subject in requested_subjects}
    for row in rows:
        subject = normalize_subject(str(row.get("subject", "")))
        if subject not in buckets:
            continue
        buckets[subject].append(row)

    for subject in buckets:
        buckets[subject].sort(key=lambda row: str(row.get("lesson_id", "")))

    generated: list[dict] = []
    for subject in requested_subjects:
        selected = buckets.get(subject, [])[: max(0, args.max_per_subject)]
        for row in selected:
            lesson_id = str(row.get("lesson_id", "")).strip()
            if not lesson_id:
                continue
            template_id, template = choose_template(subject, lesson_id)
            lesson_dir = out_root / lesson_id
            lesson_dir.mkdir(parents=True, exist_ok=True)
            summary = str(row.get("goal_summary", "")).strip()
            desc = summary or f"{subject} 교과 재작성"
            ddn_text = template.format(name=f"{lesson_id}_rewrite_v1", desc=desc)
            (lesson_dir / "lesson.ddn").write_text(ddn_text, encoding="utf-8")
            (lesson_dir / "text.md").write_text(f"# {lesson_id}\n\n{desc}\n", encoding="utf-8")

            generated.append(
                {
                    "lesson_id": lesson_id,
                    "subject": subject,
                    "source_lesson_ddn": row.get("source_lesson_ddn", ""),
                    "generated_lesson_ddn": str((lesson_dir / "lesson.ddn").as_posix()),
                    "generated_text_md": str((lesson_dir / "text.md").as_posix()),
                    "template_id": template_id,
                }
            )

    manifest = {
        "schema": "seamgrim.curriculum.rewrite_manifest.v1",
        "plan": str(plan_path.as_posix()),
        "count": len(generated),
        "generated": generated,
    }
    manifest_path = Path(args.manifest_out)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ok] wrote {manifest_path} ({len(generated)} lessons)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
