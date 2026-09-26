#!/usr/bin/env bash
# Microbenchmarks (solo el cálculo, sin HTTP): criterion en Rust y JMH en Java.
# Guarda los tiempos en bench/results/micro.json para bench/plot.py.
set -euo pipefail
cd "$(dirname "$0")/.."
OUT=bench/results
mkdir -p "$OUT"

echo "==> Rust (criterion)"
cargo bench -q --manifest-path rust/Cargo.toml -p sched-solver >/dev/null 2>&1
cargo bench -q --manifest-path rust/Cargo.toml -p sched-simulation --bench catalog_lookup >/dev/null 2>&1

echo "==> Java (JMH)"
(cd java && ./mvnw -q -pl benchmarks -am package -DskipTests)
java -jar java/benchmarks/target/benchmarks.jar -rf json -rff "$OUT/jmh.json" >/dev/null

python3 - "$OUT" <<'PY'
import json, sys
from pathlib import Path

out = Path(sys.argv[1])

def criterion_us(name):
    estimates = json.loads(Path(f"rust/target/criterion/{name}/new/estimates.json").read_text())
    return estimates["median"]["point_estimate"] / 1000  # ns -> µs

jmh = {}
for r in json.loads((out / "jmh.json").read_text()):
    score, unit = r["primaryMetric"]["score"], r["primaryMetric"]["scoreUnit"]
    us = score / 1000 if unit.startswith("ns") else score
    jmh[(r["benchmark"].rsplit(".", 1)[-1], r.get("params", {}).get("courses"))] = us

micro = {
    "backtracking_6x5": {"rust": criterion_us("backtracking_6x5"), "java": jmh[("backtracking6x5", None)]},
    # 1.000 cursos = 5.000 grupos en el catálogo.
    "catalog_lookup_5000": {
        "rust": criterion_us("catalog_open_sections_of/5000"),
        "java": jmh[("openSectionsOf", "1000")],
    },
}
(out / "micro.json").write_text(json.dumps(micro, indent=2))
print(json.dumps(micro, indent=2))
PY
