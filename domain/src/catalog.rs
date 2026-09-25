//! Réplica del catálogo que envía la institución (ADR 0003).

use std::collections::HashMap;
use std::fmt;

use crate::{CourseId, Section, SectionId};

/// Cambio incremental del catálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogChange {
    SeatsChanged { section: SectionId, available: u32 },
    SectionUpserted { section: Section, available: u32 },
    SectionRemoved { section: SectionId },
}

impl CatalogChange {
    pub fn section_id(&self) -> &SectionId {
        match self {
            Self::SeatsChanged { section, .. } | Self::SectionRemoved { section } => section,
            Self::SectionUpserted { section, .. } => &section.id,
        }
    }
}

/// Cambio con su número de secuencia, asignado por la institución.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEvent {
    pub seq: u64,
    pub change: CatalogChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    Applied,
    /// El evento ya estaba aplicado (`seq` repetido o antiguo); no cambia nada.
    Duplicate,
}

/// Se perdió al menos un evento: hace falta una carga completa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceGap {
    pub expected: u64,
    pub received: u64,
}

impl fmt::Display for SequenceGap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "se esperaba seq {} y llegó {}",
            self.expected, self.received
        )
    }
}

impl std::error::Error for SequenceGap {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    section: Section,
    available: u32,
}

/// Catálogo completo del periodo con los cupos disponibles de cada grupo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    seq: u64,
    entries: HashMap<SectionId, Entry>,
}

impl Catalog {
    /// Construye el catálogo a partir de una carga completa.
    pub fn load(seq: u64, sections: impl IntoIterator<Item = (Section, u32)>) -> Self {
        let entries = sections
            .into_iter()
            .map(|(section, available)| (section.id.clone(), Entry { section, available }))
            .collect();
        Self { seq, entries }
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn available(&self, section: &SectionId) -> Option<u32> {
        self.entries.get(section).map(|e| e.available)
    }

    /// Grupos de los cursos indicados que aún tienen cupo.
    pub fn open_sections_of<'a>(
        &'a self,
        courses: &'a [CourseId],
    ) -> impl Iterator<Item = &'a Section> {
        self.entries
            .values()
            .filter(|e| e.available > 0 && courses.contains(&e.section.course))
            .map(|e| &e.section)
    }

    /// Aplica un cambio respetando el orden estricto de `seq`.
    pub fn apply(&mut self, event: CatalogEvent) -> Result<ApplyOutcome, SequenceGap> {
        let expected = self.seq + 1;
        if event.seq < expected {
            return Ok(ApplyOutcome::Duplicate);
        }
        if event.seq > expected {
            return Err(SequenceGap {
                expected,
                received: event.seq,
            });
        }

        match event.change {
            CatalogChange::SeatsChanged { section, available } => {
                // Un cambio de cupo sobre un grupo desconocido no tiene datos para crearlo.
                if let Some(entry) = self.entries.get_mut(&section) {
                    entry.available = available;
                }
            }
            CatalogChange::SectionUpserted { section, available } => {
                self.entries
                    .insert(section.id.clone(), Entry { section, available });
            }
            CatalogChange::SectionRemoved { section } => {
                self.entries.remove(&section);
            }
        }
        self.seq = event.seq;
        Ok(ApplyOutcome::Applied)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TimeSlot, Weekday};

    use super::*;

    fn section(id: &str) -> Section {
        Section {
            id: SectionId::new(id),
            course: CourseId::new("MAT"),
            slots: vec![TimeSlot::new(Weekday::Monday, 420, 540).unwrap()],
            capacity: 30,
        }
    }

    fn seats(seq: u64, id: &str, available: u32) -> CatalogEvent {
        CatalogEvent {
            seq,
            change: CatalogChange::SeatsChanged {
                section: SectionId::new(id),
                available,
            },
        }
    }

    #[test]
    fn applies_events_in_order() {
        let mut catalog = Catalog::load(10, [(section("MAT-01"), 5)]);

        assert_eq!(
            catalog.apply(seats(11, "MAT-01", 0)),
            Ok(ApplyOutcome::Applied)
        );
        assert_eq!(catalog.seq(), 11);
        assert_eq!(catalog.available(&SectionId::new("MAT-01")), Some(0));
    }

    #[test]
    fn repeated_events_are_ignored() {
        let mut catalog = Catalog::load(10, [(section("MAT-01"), 5)]);

        assert_eq!(
            catalog.apply(seats(10, "MAT-01", 0)),
            Ok(ApplyOutcome::Duplicate)
        );
        assert_eq!(catalog.available(&SectionId::new("MAT-01")), Some(5));
    }

    #[test]
    fn gaps_are_rejected_without_changes() {
        let mut catalog = Catalog::load(10, [(section("MAT-01"), 5)]);

        assert_eq!(
            catalog.apply(seats(12, "MAT-01", 0)),
            Err(SequenceGap {
                expected: 11,
                received: 12
            })
        );
        assert_eq!(catalog.seq(), 10);
    }

    #[test]
    fn full_sections_are_not_offered() {
        let catalog = Catalog::load(1, [(section("MAT-01"), 0), (section("MAT-02"), 3)]);
        let courses = [CourseId::new("MAT")];
        let open: Vec<_> = catalog.open_sections_of(&courses).collect();

        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, SectionId::new("MAT-02"));
    }
}
