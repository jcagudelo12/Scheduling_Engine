//! Casos de uso. Los adaptadores de entrada (HTTP, gRPC, seguidor del historial) los invocan.
//!
//! - Ingesta (solo la instancia de ingesta): `IngestCatalog`, `IngestCatalogEvent`.
//! - Réplica (todas las instancias): `ApplyLogEntry`.
//! - Consulta: `GenerateCombinations`.

mod apply_log_entry;
mod generate_combinations;
mod ingest_catalog;
mod ingest_catalog_event;

pub use apply_log_entry::ApplyLogEntry;
pub use generate_combinations::{Combinations, GenerateCombinations};
pub use ingest_catalog::IngestCatalog;
pub use ingest_catalog_event::IngestCatalogEvent;
