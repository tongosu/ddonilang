import json

def gen_graph(income=0.0):
    a=100.0; b=5.0; c=20.0; d=4.0; k=1.0
    p_min=0.0; p_max=20.0; step=0.5
    p_vals=[]
    p=p_min
    while p <= p_max + 1e-9:
        p_vals.append(round(p,6))
        p += step
    demand=[]
    supply=[]
    for p in p_vals:
        Qd = a - b*p + k*income
        Qs = c + d*p
        demand.append([p, round(Qd,6)])
        supply.append([p, round(Qs,6)])
    p_star = (a + k*income - c)/(b + d)
    Q_star = c + d*p_star
    eq = [[round(p_star,6), round(Q_star,6)]]
    return {
        "schema":"seamgrim.graph.v0",
        "graph_kind":"xy_line",
        "x":{"kind":"price","unit":"unit","label":"가격 p"},
        "y":{"kind":"quantity","unit":"unit","label":"수량 Q"},
        "series":[
            {"id":"demand","role":"baseline","points":demand},
            {"id":"supply","role":"baseline","points":supply},
            {"id":"eq","role":"baseline","points":eq}
        ]
    }

with open("expected.graph.v0.json","w",encoding="utf-8") as f:
    json.dump(gen_graph(0.0), f, ensure_ascii=False, indent=2, sort_keys=True)
    f.write("\n")
print("Wrote expected.graph.v0.json")
