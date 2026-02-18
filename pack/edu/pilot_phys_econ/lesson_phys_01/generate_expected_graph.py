import json, math

g = 9.8
fps = 60.0
total_s = 5.0
endtick = int(total_s * fps)

theta0 = 0.6
L = 1.0
omegaN = math.sqrt(g / L)

points = []
for tick in range(endtick + 1):
    t = tick / fps
    theta = theta0 * math.cos(omegaN * t)
    points.append([round(t, 6), round(theta, 6)])

graph_v0 = {
  "schema": "seamgrim.graph.v0",
  "graph_kind": "timeseries",
  "x": {"kind": "time", "unit": "s", "label": "t"},
  "y": {"kind": "angle", "unit": "rad", "label": "θ"},
  "series": [{"id": "theta", "role": "baseline", "points": points}]
}

with open("expected.graph.v0.json", "w", encoding="utf-8") as f:
    json.dump(graph_v0, f, ensure_ascii=False, indent=2, sort_keys=True)
    f.write("\n")

print("Wrote expected.graph.v0.json")
