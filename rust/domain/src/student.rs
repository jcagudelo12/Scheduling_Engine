use crate::{CourseId, StudentId};

/// Datos del estudiante que entrega la institución en cada solicitud (ADR 0004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudentContext {
    pub id: StudentId,
    /// Cursos que el estudiante puede matricular en el periodo.
    pub eligible_courses: Vec<CourseId>,
}
