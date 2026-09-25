use crate::Section;

/// Una combinación válida: un grupo por curso, sin cruces de horario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    pub sections: Vec<Section>,
}

impl Schedule {
    pub fn has_clashes(&self) -> bool {
        self.sections
            .iter()
            .enumerate()
            .any(|(i, a)| self.sections[i + 1..].iter().any(|b| a.clashes_with(b)))
    }
}
