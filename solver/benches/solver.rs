use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use sched_domain::{CourseId, Section, SectionId, TimeSlot, Weekday};
use sched_solver::{BacktrackingSolver, Problem, Solver};

const DAYS: [Weekday; 6] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
];

/// Problema sintético: `courses` cursos con `sections_per_course` grupos repartidos en la semana.
fn synthetic_problem(courses: usize, sections_per_course: usize) -> Problem {
    let mut candidates = Vec::new();
    for c in 0..courses {
        for s in 0..sections_per_course {
            let day = DAYS[(c + s) % DAYS.len()];
            let start = 360 + ((c * 7 + s * 3) % 7) as u16 * 120;
            candidates.push(Section {
                id: SectionId::new(format!("C{c}-{s}")),
                course: CourseId::new(format!("C{c}")),
                slots: vec![TimeSlot::new(day, start, start + 120).unwrap()],
                capacity: 30,
            });
        }
    }
    Problem {
        courses: (0..courses)
            .map(|c| CourseId::new(format!("C{c}")))
            .collect(),
        candidates,
        max_results: 500,
    }
}

fn bench_backtracking(c: &mut Criterion) {
    let problem = synthetic_problem(6, 5);
    c.bench_function("backtracking/6x5", |b| {
        b.iter(|| BacktrackingSolver.solve(black_box(&problem)))
    });
}

criterion_group!(benches, bench_backtracking);
criterion_main!(benches);
