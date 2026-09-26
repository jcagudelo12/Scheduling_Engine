"""Lectura de los resultados de bench/run.sh, compartida por report.py y plot.py.

Cada métrica de una fase es la mediana de las repeticiones; el p99 guarda además el rango
(mín–máx) entre repeticiones.
"""

import json
import re
import statistics
from pathlib import Path

ENGINES = ["rust", "java"]
UNITS = {"KiB": 1 / 1024, "MiB": 1, "GiB": 1024, "kB": 1 / 1024, "MB": 1, "GB": 1024, "B": 1 / 1024**2}


def mem_mib(text):
    match = re.match(r"\s*([\d.]+)\s*([KMG]?i?B)", text or "")
    if match is None or match.group(2) not in UNITS:
        return None  # muestra vacía (p. ej. contenedor detenido)
    value, unit = match.groups()
    return float(value) * UNITS[unit]


def median(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


class Results:
    def __init__(self, out, reps):
        self.out = Path(out)
        self.reps = range(1, int(reps) + 1)

    def _read(self, name):
        path = self.out / name
        return path.read_text().strip() if path.exists() else None

    def environment(self):
        return self._read("environment.md")

    def startup_ms(self, engine):
        return median(float(v) for r in self.reps if (v := self._read(f"{engine}_rep{r}_startup_ms.txt")))

    def idle_mem_mib(self, engine):
        return median(mem_mib(self._read(f"{engine}_rep{r}_idle_mem.txt")) for r in self.reps)

    def cpus(self, engine):
        return sorted({self._read(f"{engine}_rep{r}_cpus.txt") or "?" for r in self.reps})

    def _stats(self, prefix, name):
        """CPU promedio y memoria máxima de una corrida."""
        path = self.out / f"{prefix}_{name}_stats.txt"
        cpu, mem = [], []
        for line in (path.read_text().splitlines() if path.exists() else []):
            parts = line.split(" ", 1)
            if len(parts) == 2 and parts[0].endswith("%"):
                used = mem_mib(parts[1].split("/")[0])
                if used is not None:
                    cpu.append(float(parts[0][:-1]))
                    mem.append(used)
        return (sum(cpu) / len(cpu) if cpu else None), (max(mem) if mem else None)

    def _run(self, prefix, name):
        path = self.out / f"{prefix}_{name}.json"
        if not path.exists():
            return None
        m = json.loads(path.read_text())["metrics"]
        d = m["http_req_duration"]["values"]
        checks = m.get("checks", {}).get("values", {})
        total = checks.get("passes", 0) + checks.get("fails", 0)
        cpu, mem = self._stats(prefix, name)
        return {
            "rps": m["http_reqs"]["values"]["rate"],
            "p50": d["med"], "p95": d["p(95)"], "p99": d["p(99)"], "max": d["max"],
            "errors": (checks.get("fails", 0) / total * 100) if total else 0,
            "dropped": m.get("dropped_iterations", {}).get("values", {}).get("count", 0),
            "cpu": cpu, "mem": mem,
        }

    def phase(self, engine, name):
        """Mediana de las repeticiones de una fase ('cold' o 'r<tasa>'), o None."""
        runs = [r for r in (self._run(f"{engine}_rep{rep}", name) for rep in self.reps) if r]
        if not runs:
            return None
        result = {key: median(r[key] for r in runs) for key in runs[0] if key != "mem"}
        result["mem"] = max((r["mem"] or 0) for r in runs)
        result["p99_min"] = min(r["p99"] for r in runs)
        result["p99_max"] = max(r["p99"] for r in runs)
        return result

    def micro(self):
        path = self.out / "micro.json"
        return json.loads(path.read_text()) if path.exists() else None
