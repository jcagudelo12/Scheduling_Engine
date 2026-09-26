//! Costo de encontrar los grupos de los cursos de un estudiante dentro del catálogo
//! completo de la institución, según el tamaño del catálogo.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use sched_domain::{Catalog, CourseId, Section, SectionId, TimeSlot, Weekday};

const SECTIONS_PER_COURSE: usize = 5;
const STUDENT_COURSES: usize = 6;

fn catalog(courses: usize) -> Catalog {
    let sections = (0..courses).flat_map(|c| {
        (0..SECTIONS_PER_COURSE).map(move |s| {
            let section = Section {
                id: SectionId::new(format!("C{c:04}-{s:02}")),
                course: CourseId::new(format!("C{c:04}")),
                slots: vec![TimeSlot::new(Weekday::Monday, 420, 540).unwrap()],
                capacity: 30,
            };
            (section, 10)
        })
    });
    Catalog::load(1, sections)
}

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("catalog/open_sections_of");
    for courses in [200, 1_000, 5_000] {
        let catalog = catalog(courses);
        // Cursos repartidos por todo el catálogo, como un estudiante cualquiera.
        let student: Vec<_> = (0..STUDENT_COURSES)
            .map(|i| CourseId::new(format!("C{:04}", i * courses / STUDENT_COURSES)))
            .collect();
        let sections = courses * SECTIONS_PER_COURSE;

        group.bench_with_input(
            BenchmarkId::from_parameter(sections),
            &student,
            |b, student| b.iter(|| catalog.open_sections_of(black_box(student)).count()),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
