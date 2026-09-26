//! Conversión entre mensajes protobuf y tipos del dominio. Al convertir hacia el dominio se
//! validan los datos recibidos.

use std::fmt;

use sched_domain::{CatalogChange, CatalogEvent, CourseId, Section, SectionId, TimeSlot, Weekday};

use crate::institution::v1 as pb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionError(String);

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConversionError {}

/// Grupo con sus cupos disponibles.
pub fn section(section: pb::Section) -> Result<(Section, u32), ConversionError> {
    if section.section_id.is_empty() || section.course_id.is_empty() {
        return Err(ConversionError(
            "grupo sin identificador de grupo o de curso".into(),
        ));
    }
    let slots = section
        .slots
        .iter()
        .map(|slot| time_slot(slot, &section.section_id))
        .collect::<Result<_, _>>()?;
    let available = section.available_seats;
    let section = Section {
        id: SectionId::new(section.section_id),
        course: CourseId::new(section.course_id),
        slots,
        capacity: section.capacity,
    };
    Ok((section, available))
}

pub fn catalog_event(event: pb::CatalogEvent) -> Result<CatalogEvent, ConversionError> {
    use pb::catalog_event::Change;

    let change = match event.change {
        Some(Change::SeatsChanged(c)) => CatalogChange::SeatsChanged {
            section: SectionId::new(c.section_id),
            available: c.available_seats,
        },
        Some(Change::SectionUpserted(c)) => {
            let raw = c
                .section
                .ok_or_else(|| ConversionError("SectionUpserted sin grupo".into()))?;
            let (section, available) = section(raw)?;
            CatalogChange::SectionUpserted { section, available }
        }
        Some(Change::SectionRemoved(c)) => CatalogChange::SectionRemoved {
            section: SectionId::new(c.section_id),
        },
        None => return Err(ConversionError(format!("evento {} sin cambio", event.seq))),
    };
    Ok(CatalogEvent {
        seq: event.seq,
        change,
    })
}

fn time_slot(slot: &pb::TimeSlot, section_id: &str) -> Result<TimeSlot, ConversionError> {
    let invalid = || ConversionError(format!("franja inválida en el grupo {section_id}"));

    let day = match pb::Weekday::try_from(slot.day).map_err(|_| invalid())? {
        pb::Weekday::Monday => Weekday::Monday,
        pb::Weekday::Tuesday => Weekday::Tuesday,
        pb::Weekday::Wednesday => Weekday::Wednesday,
        pb::Weekday::Thursday => Weekday::Thursday,
        pb::Weekday::Friday => Weekday::Friday,
        pb::Weekday::Saturday => Weekday::Saturday,
        pb::Weekday::Sunday => Weekday::Sunday,
        pb::Weekday::Unspecified => return Err(invalid()),
    };
    let start = u16::try_from(slot.start_minute).map_err(|_| invalid())?;
    let end = u16::try_from(slot.end_minute).map_err(|_| invalid())?;
    TimeSlot::new(day, start, end).ok_or_else(invalid)
}

pub fn section_to_pb(section: &Section, available: u32) -> pb::Section {
    pb::Section {
        section_id: section.id.to_string(),
        course_id: section.course.to_string(),
        slots: section.slots.iter().map(time_slot_to_pb).collect(),
        capacity: section.capacity,
        available_seats: available,
    }
}

pub fn catalog_event_to_pb(event: &CatalogEvent) -> pb::CatalogEvent {
    use pb::catalog_event::Change;

    let change = match &event.change {
        CatalogChange::SeatsChanged { section, available } => {
            Change::SeatsChanged(pb::SeatsChanged {
                section_id: section.to_string(),
                available_seats: *available,
            })
        }
        CatalogChange::SectionUpserted { section, available } => {
            Change::SectionUpserted(pb::SectionUpserted {
                section: Some(section_to_pb(section, *available)),
            })
        }
        CatalogChange::SectionRemoved { section } => Change::SectionRemoved(pb::SectionRemoved {
            section_id: section.to_string(),
        }),
    };
    pb::CatalogEvent {
        seq: event.seq,
        change: Some(change),
    }
}

fn time_slot_to_pb(slot: &TimeSlot) -> pb::TimeSlot {
    let day = match slot.day {
        Weekday::Monday => pb::Weekday::Monday,
        Weekday::Tuesday => pb::Weekday::Tuesday,
        Weekday::Wednesday => pb::Weekday::Wednesday,
        Weekday::Thursday => pb::Weekday::Thursday,
        Weekday::Friday => pb::Weekday::Friday,
        Weekday::Saturday => pb::Weekday::Saturday,
        Weekday::Sunday => pb::Weekday::Sunday,
    };
    pb::TimeSlot {
        day: day.into(),
        start_minute: slot.start.into(),
        end_minute: slot.end.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_round_trip() {
        let original = Section {
            id: SectionId::new("MAT-01"),
            course: CourseId::new("MAT"),
            slots: vec![TimeSlot::new(Weekday::Friday, 420, 540).unwrap()],
            capacity: 30,
        };
        let (back, available) = section(section_to_pb(&original, 7)).unwrap();
        assert_eq!(back, original);
        assert_eq!(available, 7);
    }

    #[test]
    fn event_round_trip() {
        let original = CatalogEvent {
            seq: 42,
            change: CatalogChange::SeatsChanged {
                section: SectionId::new("MAT-01"),
                available: 3,
            },
        };
        assert_eq!(
            catalog_event(catalog_event_to_pb(&original)).unwrap(),
            original
        );
    }
}
