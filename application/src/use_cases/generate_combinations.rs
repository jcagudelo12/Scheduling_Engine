use sched_domain::{Schedule, StudentContext};
use sched_solver::{Problem, Solver};

use crate::{AppError, CatalogReplica};

/// Resultado junto con el estado del catálogo con que se calculó.
#[derive(Debug, Clone)]
pub struct Combinations {
    /// `seq` del catálogo usado, definido por la institución.
    pub catalog_seq: u64,
    /// `true` si la réplica podía no estar al día (ADR 0003).
    pub stale: bool,
    pub schedules: Vec<Schedule>,
}

/// El estudiante pide combinaciones de horario para sus cursos habilitados.
///
/// Es síncrono: solo lee la réplica en memoria y ejecuta el solver.
#[derive(Debug)]
pub struct GenerateCombinations<S> {
    solver: S,
    replica: CatalogReplica,
    max_results: usize,
}

impl<S: Solver> GenerateCombinations<S> {
    pub fn new(solver: S, replica: CatalogReplica, max_results: usize) -> Self {
        Self {
            solver,
            replica,
            max_results,
        }
    }

    pub fn execute(&self, student: &StudentContext) -> Result<Combinations, AppError> {
        let (catalog_seq, stale, candidates) = self.replica.read(|catalog, stale| {
            let mut candidates: Vec<_> = catalog
                .open_sections_of(&student.eligible_courses)
                .cloned()
                .collect();
            // El catálogo no tiene orden; se fija para que los resultados sean reproducibles.
            candidates.sort_by(|a, b| a.id.cmp(&b.id));
            (catalog.seq(), stale, candidates)
        })?;

        let problem = Problem {
            courses: student.eligible_courses.clone(),
            candidates,
            max_results: self.max_results,
        };
        let schedules = self.solver.solve(&problem);
        tracing::debug!(student = %student.id, found = schedules.len(), catalog_seq, stale, "combinaciones generadas");

        Ok(Combinations {
            catalog_seq,
            stale,
            schedules,
        })
    }
}
