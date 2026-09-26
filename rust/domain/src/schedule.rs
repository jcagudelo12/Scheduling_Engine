use std::sync::Arc;

use crate::Section;

/// Una combinación válida: un grupo por curso, sin cruces de horario.
///
/// Los grupos se comparten con `Arc` (no se copian): muchas combinaciones apuntan a los
/// mismos grupos del catálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    pub sections: Vec<Arc<Section>>,
}

impl Schedule {
    pub fn has_clashes(&self) -> bool {
        self.sections
            .iter()
            .enumerate()
            .any(|(i, a)| self.sections[i + 1..].iter().any(|b| a.clashes_with(b)))
    }
}
