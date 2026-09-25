use sched_domain::{Schedule, Section};

use crate::{Problem, Solver};

/// Enumeración exhaustiva con poda por cruce de horario.
///
/// Sirve como línea base contra la cual comparar estrategias más elaboradas.
#[derive(Debug, Default, Clone, Copy)]
pub struct BacktrackingSolver;

impl Solver for BacktrackingSolver {
    fn solve(&self, problem: &Problem) -> Vec<Schedule> {
        let options: Vec<Vec<&Section>> = problem
            .courses
            .iter()
            .map(|course| {
                problem
                    .candidates
                    .iter()
                    .filter(|s| &s.course == course)
                    .collect()
            })
            .collect();

        // Un curso sin grupos disponibles hace imposible cualquier combinación completa.
        if options.iter().any(Vec::is_empty) {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut current = Vec::with_capacity(options.len());
        search(&options, &mut current, &mut results, problem.max_results);
        results
    }
}

fn search<'a>(
    options: &[Vec<&'a Section>],
    current: &mut Vec<&'a Section>,
    results: &mut Vec<Schedule>,
    max_results: usize,
) {
    if results.len() >= max_results {
        return;
    }
    let Some((course_options, rest)) = options.split_first() else {
        results.push(Schedule {
            sections: current.iter().map(|s| (*s).clone()).collect(),
        });
        return;
    };
    for &section in course_options {
        if current.iter().any(|chosen| chosen.clashes_with(section)) {
            continue;
        }
        current.push(section);
        search(rest, current, results, max_results);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use sched_domain::{CourseId, SectionId, TimeSlot, Weekday};

    use super::*;

    fn section(id: &str, course: &str, day: Weekday, start: u16, end: u16) -> Section {
        Section {
            id: SectionId::new(id),
            course: CourseId::new(course),
            slots: vec![TimeSlot::new(day, start, end).unwrap()],
            capacity: 30,
        }
    }

    fn problem(courses: &[&str], candidates: Vec<Section>) -> Problem {
        Problem {
            courses: courses.iter().map(|c| CourseId::new(*c)).collect(),
            candidates,
            max_results: 100,
        }
    }

    #[test]
    fn skips_combinations_with_clashes() {
        let p = problem(
            &["MAT", "FIS"],
            vec![
                section("MAT-01", "MAT", Weekday::Monday, 420, 540),
                section("MAT-02", "MAT", Weekday::Tuesday, 420, 540),
                section("FIS-01", "FIS", Weekday::Monday, 480, 600),
            ],
        );

        let schedules = BacktrackingSolver.solve(&p);

        assert_eq!(schedules.len(), 1);
        assert!(schedules.iter().all(|s| !s.has_clashes()));
        assert_eq!(schedules[0].sections[0].id, SectionId::new("MAT-02"));
    }

    #[test]
    fn course_without_sections_yields_nothing() {
        let p = problem(
            &["MAT", "FIS"],
            vec![section("MAT-01", "MAT", Weekday::Monday, 420, 540)],
        );
        assert!(BacktrackingSolver.solve(&p).is_empty());
    }

    #[test]
    fn respects_max_results() {
        let mut p = problem(
            &["MAT"],
            vec![
                section("MAT-01", "MAT", Weekday::Monday, 420, 540),
                section("MAT-02", "MAT", Weekday::Tuesday, 420, 540),
            ],
        );
        p.max_results = 1;
        assert_eq!(BacktrackingSolver.solve(&p).len(), 1);
    }
}
