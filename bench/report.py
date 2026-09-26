#!/usr/bin/env python3
"""Arma la tabla comparativa a partir de los resultados de bench/run.sh.

Uso: python3 bench/report.py <carpeta> <repeticiones> <tasa> [<tasa> ...]
"""

import sys

from results_lib import ENGINES, Results


def ms(v):
    if v is None:
        return "–"
    return f"{v:.2f}" if v < 10 else f"{v:.0f}"


def num(v):
    return "–" if v is None else f"{v:.0f}"


results = Results(sys.argv[1], sys.argv[2])
rates = sys.argv[3:]

env = results.environment()
if env:
    print(env + "\n")

print("## Arranque y reposo (mediana)\n")
print("| | Rust | Java |")
print("|---|---|---|")
print("| Arranque hasta `/ready` (ms) | " + " | ".join(num(results.startup_ms(e)) for e in ENGINES) + " |")
print("| Memoria en reposo con catálogo (MiB) | " + " | ".join(num(results.idle_mem_mib(e)) for e in ENGINES) + " |")
print("| CPUs detectadas por el motor | " + " | ".join(", ".join(results.cpus(e)) for e in ENGINES) + " |")

phases = [("cold", "Primeras solicitudes en frío (200/s, 20 s)")] + [(f"r{r}", f"{r} solicitudes/s") for r in rates]
for name, title in phases:
    print(f"\n## {title}\n")
    print("| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for e in ENGINES:
        p = results.phase(e, name)
        if p is None:
            print(f"| {e} | sin datos ||||||||||")
            continue
        print(
            f"| {e} | {num(p['rps'])} | {ms(p['p50'])} | {ms(p['p95'])} | {ms(p['p99'])} "
            f"| {ms(p['p99_min'])}–{ms(p['p99_max'])} | {ms(p['max'])} | {p['errors']:.1f} % "
            f"| {num(p['dropped'])} | {num(p['cpu'])} % | {num(p['mem'])} |"
        )

print(f"\nMediana de {len(results.reps)} repeticiones. CPU sobre 100 % por núcleo asignado.")
