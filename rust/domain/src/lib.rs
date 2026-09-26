//! Núcleo del dominio: cursos, grupos, franjas horarias y el catálogo replicado.
//!
//! Este crate es puro: sin `async`, sin I/O y sin dependencias externas.

pub mod catalog;
pub mod ids;
pub mod schedule;
pub mod section;
pub mod student;
pub mod time;

pub use catalog::{ApplyOutcome, Catalog, CatalogChange, CatalogEvent, SequenceGap};
pub use ids::{CourseId, SectionId, StudentId};
pub use schedule::Schedule;
pub use section::Section;
pub use student::StudentContext;
pub use time::{TimeSlot, Weekday};
