//! Réplica del catálogo que envía la institución (ADR 0003).

use std::collections::{BTreeSet, HashMap};
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
    /// Índice curso → grupos, para no recorrer todo el catálogo en cada consulta.
    /// Los grupos van ordenados por id para que los resultados sean reproducibles.
    by_course: HashMap<CourseId, BTreeSet<SectionId>>,
}

impl Catalog {
    /// Construye el catálogo a partir de una carga completa.
    pub fn load(seq: u64, sections: impl IntoIterator<Item = (Section, u32)>) -> Self {
        let mut catalog = Self {
            seq,
            entries: HashMap::new(),
            by_course: HashMap::new(),
        };
        for (section, available) in sections {
            catalog.insert(section, available);
        }
        catalog
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

    /// Grupos de los cursos indicados que aún tienen cupo, curso por curso.
    ///
    /// El costo depende de cuántos grupos tienen esos cursos, no del tamaño del catálogo.
    pub fn open_sections_of<'a>(
        &'a self,
        courses: &'a [CourseId],
    ) -> impl Iterator<Item = &'a Section> {
        courses
            .iter()
            .enumerate()
            // Un curso repetido en la lista no debe repetir sus grupos.
            .filter(|(i, course)| !courses[..*i].contains(course))
            .filter_map(|(_, course)| self.by_course.get(course))
            .flatten()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.available > 0)
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
                self.insert(section, available);
            }
            CatalogChange::SectionRemoved { section } => {
                self.remove(&section);
            }
        }
        self.seq = event.seq;
        Ok(ApplyOutcome::Applied)
    }

    /// Inserta o reemplaza un grupo manteniendo el índice por curso al día.
    fn insert(&mut self, section: Section, available: u32) {
        let id = section.id.clone();
        let course = section.course.clone();
        if let Some(previous) = self
            .entries
            .insert(id.clone(), Entry { section, available })
        {
            // Si el grupo cambió de curso, sale del índice del curso anterior.
            if previous.section.course != course {
                self.unindex(&previous.section.course, &id);
            }
        }
        self.by_course.entry(course).or_default().insert(id);
    }

    fn remove(&mut self, id: &SectionId) {
        if let Some(entry) = self.entries.remove(id) {
            self.unindex(&entry.section.course, id);
        }
    }

    fn unindex(&mut self, course: &CourseId, id: &SectionId) {
        if let Some(ids) = self.by_course.get_mut(course) {
            ids.remove(id);
            if ids.is_empty() {
                self.by_course.remove(course);
            }
        }
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

    fn section_of(id: &str, course: &str) -> Section {
        Section {
            course: CourseId::new(course),
            ..section(id)
        }
    }

    fn open_ids(catalog: &Catalog, courses: &[&str]) -> Vec<String> {
        let courses: Vec<_> = courses.iter().map(|c| CourseId::new(*c)).collect();
        catalog
            .open_sections_of(&courses)
            .map(|s| s.id.to_string())
            .collect()
    }

    #[test]
    fn only_sections_of_requested_courses_are_returned() {
        let catalog = Catalog::load(
            1,
            [
                (section_of("MAT-02", "MAT"), 5),
                (section_of("ART-01", "ART"), 5),
                (section_of("MAT-01", "MAT"), 5),
                (section_of("FIS-01", "FIS"), 5),
            ],
        );
        assert_eq!(
            open_ids(&catalog, &["FIS", "MAT"]),
            ["FIS-01", "MAT-01", "MAT-02"]
        );
    }

    #[test]
    fn repeated_courses_do_not_repeat_sections() {
        let catalog = Catalog::load(1, [(section_of("MAT-01", "MAT"), 5)]);
        assert_eq!(open_ids(&catalog, &["MAT", "MAT"]), ["MAT-01"]);
    }

    #[test]
    fn index_follows_upserts_and_removals() {
        let mut catalog = Catalog::load(1, [(section_of("X-01", "MAT"), 5)]);

        // El grupo se reasigna a otro curso: debe salir del índice de MAT.
        let moved = CatalogChange::SectionUpserted {
            section: section_of("X-01", "FIS"),
            available: 5,
        };
        catalog
            .apply(CatalogEvent {
                seq: 2,
                change: moved,
            })
            .unwrap();
        assert!(open_ids(&catalog, &["MAT"]).is_empty());
        assert_eq!(open_ids(&catalog, &["FIS"]), ["X-01"]);

        let removed = CatalogChange::SectionRemoved {
            section: SectionId::new("X-01"),
        };
        catalog
            .apply(CatalogEvent {
                seq: 3,
                change: removed,
            })
            .unwrap();
        assert!(open_ids(&catalog, &["FIS"]).is_empty());
        assert!(catalog.by_course.is_empty());
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
