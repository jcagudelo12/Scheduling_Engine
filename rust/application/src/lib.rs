//! Capa de aplicación: orquesta dominio y solver a través de puertos.
//!
//! Aquí se definen los traits que implementan los adaptadores. Esta capa conoce
//! las abstracciones, nunca las implementaciones concretas (tonic, axum, NATS...).

pub mod error;
pub mod ingest;
pub mod log;
pub mod ports;
pub mod readiness;
pub mod replica;
pub mod use_cases;

pub use error::{AppError, PortError};
pub use ingest::IngestState;
pub use log::CatalogLogEntry;
pub use readiness::Readiness;
pub use replica::CatalogReplica;
