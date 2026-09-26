#!/usr/bin/env python3
"""Genera los datos de la prueba de carga con semillas fijas, para que los dos motores
reciban exactamente el mismo catálogo y los mismos estudiantes en el mismo orden.

- bench/scenario.json: catálogo del tamaño de una universidad (formato de `fake-institution`).
- bench/students.json: estudiantes con 6 cursos cada uno, que k6 recorre en orden.

Uso: python3 bench/generate_scenario.py [cursos] [estudiantes]
"""

import json
import random
import sys
from pathlib import Path

COURSES = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
STUDENTS = int(sys.argv[2]) if len(sys.argv) > 2 else 50_000
SECTIONS_PER_COURSE = 5
COURSES_PER_STUDENT = 6
DAYS = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday"]
STARTS = [6 * 60 + 120 * i for i in range(8)]  # bloques de 2 h entre 06:00 y 22:00
OUT = Path(__file__).parent


def hhmm(minutes):
    return f"{minutes // 60:02d}:{minutes % 60:02d}"


catalog_rng = random.Random(42)
sections = []
for c in range(COURSES):
    for s in range(SECTIONS_PER_COURSE):
        days = catalog_rng.sample(DAYS, 2)  # dos sesiones semanales
        start = catalog_rng.choice(STARTS)
        capacity = catalog_rng.choice([25, 30, 35, 40])
        sections.append({
            "id": f"C{c:04d}-{s:02d}",
            "course": f"C{c:04d}",
            "capacity": capacity,
            # Alrededor del 10 % de los grupos ya están llenos.
            "available": 0 if catalog_rng.random() < 0.1 else catalog_rng.randint(1, capacity),
            "slots": [{"day": d, "start": hhmm(start), "end": hhmm(start + 120)} for d in days],
        })

students_rng = random.Random(7)
students = [
    {
        "student_id": f"est-{i:05d}",
        "eligible_courses": [f"C{c:04d}" for c in students_rng.sample(range(COURSES), COURSES_PER_STUDENT)],
    }
    for i in range(STUDENTS)
]

(OUT / "scenario.json").write_text(json.dumps({"catalog_seq": 1, "students": [], "sections": sections}))
(OUT / "students.json").write_text(json.dumps(students))
print(f"{len(sections)} grupos de {COURSES} cursos; {len(students)} estudiantes")
