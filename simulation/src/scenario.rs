//! Formato JSON de los escenarios. Los DTO viven aquí para que el dominio no dependa de serde.

use std::fs;

use sched_domain::{CourseId, Section, SectionId, StudentContext, StudentId, TimeSlot, Weekday};
use serde::Deserialize;

#[derive(Debug)]
pub struct Scenario {
    pub max_results: usize,
    pub catalog_seq: u64,
    /// Cada grupo con sus cupos disponibles.
    pub sections: Vec<(Section, u32)>,
    pub students: Vec<StudentContext>,
}

#[derive(Debug, Deserialize)]
struct ScenarioFile {
    #[serde(default = "default_max_results")]
    max_results: usize,
    #[serde(default)]
    catalog_seq: u64,
    students: Vec<StudentFile>,
    sections: Vec<SectionFile>,
}

#[derive(Debug, Deserialize)]
struct StudentFile {
    id: String,
    courses: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SectionFile {
    id: String,
    course: String,
    capacity: u32,
    /// Si no se indica, el grupo arranca con todo su cupo disponible.
    available: Option<u32>,
    slots: Vec<SlotFile>,
}

#[derive(Debug, Deserialize)]
struct SlotFile {
    day: DayFile,
    start: String,
    end: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DayFile {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

fn default_max_results() -> usize {
    200
}

pub fn load(path: &str) -> Result<Scenario, Box<dyn std::error::Error>> {
    let file: ScenarioFile = serde_json::from_str(&fs::read_to_string(path)?)?;

    let students = file
        .students
        .into_iter()
        .map(|s| StudentContext {
            id: StudentId::new(s.id),
            eligible_courses: s.courses.into_iter().map(CourseId::new).collect(),
        })
        .collect();

    let sections = file
        .sections
        .into_iter()
        .map(|s| {
            let slots = s
                .slots
                .iter()
                .map(|slot| parse_slot(slot).ok_or_else(|| format!("franja inválida en {}", s.id)))
                .collect::<Result<_, _>>()?;
            let section = Section {
                id: SectionId::new(s.id),
                course: CourseId::new(s.course),
                slots,
                capacity: s.capacity,
            };
            Ok::<_, String>((section, s.available.unwrap_or(s.capacity)))
        })
        .collect::<Result<_, _>>()?;

    Ok(Scenario {
        max_results: file.max_results,
        catalog_seq: file.catalog_seq,
        sections,
        students,
    })
}

fn parse_slot(slot: &SlotFile) -> Option<TimeSlot> {
    let day = match slot.day {
        DayFile::Monday => Weekday::Monday,
        DayFile::Tuesday => Weekday::Tuesday,
        DayFile::Wednesday => Weekday::Wednesday,
        DayFile::Thursday => Weekday::Thursday,
        DayFile::Friday => Weekday::Friday,
        DayFile::Saturday => Weekday::Saturday,
        DayFile::Sunday => Weekday::Sunday,
    };
    TimeSlot::new(day, parse_hhmm(&slot.start)?, parse_hhmm(&slot.end)?)
}

/// "07:30" -> 450
fn parse_hhmm(value: &str) -> Option<u16> {
    let (h, m) = value.split_once(':')?;
    let (h, m): (u16, u16) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}
