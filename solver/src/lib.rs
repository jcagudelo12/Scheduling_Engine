//! Estrategias para generar combinaciones de horario.
//!
//! El solver es síncrono y puro. Si una búsqueda llega a ser costosa, la capa de
//! aplicación decide cómo ejecutarla (por ejemplo, en `spawn_blocking`).

mod backtracking;

pub use backtracking::BacktrackingSolver;

use sched_domain::{CourseId, Schedule, Section};

/// Entrada del solver: cursos a cubrir y grupos candidatos (ya filtrados por cupo).
#[derive(Debug, Clone)]
pub struct Problem {
    pub courses: Vec<CourseId>,
    pub candidates: Vec<Section>,
    pub max_results: usize,
}

pub trait Solver {
    fn solve(&self, problem: &Problem) -> Vec<Schedule>;
}
