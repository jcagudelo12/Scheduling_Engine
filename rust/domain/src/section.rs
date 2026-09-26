use crate::{CourseId, SectionId, TimeSlot};

/// Un grupo concreto de un curso, con sus franjas y su cupo total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub id: SectionId,
    pub course: CourseId,
    pub slots: Vec<TimeSlot>,
    pub capacity: u32,
}

impl Section {
    pub fn clashes_with(&self, other: &Section) -> bool {
        self.slots
            .iter()
            .any(|a| other.slots.iter().any(|b| a.overlaps(b)))
    }
}
