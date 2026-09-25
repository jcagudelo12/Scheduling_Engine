//! Historial compartido del catálogo sobre NATS JetStream (ADR 0005).
//!
//! - [`JetStreamCatalogLog`] implementa el puerto `CatalogLog`: la instancia de ingesta
//!   escribe las entradas en el stream.
//! - [`CatalogFollower`] es un adaptador de entrada: cada instancia lee el stream en orden
//!   y aplica las entradas a su réplica local.
//!
//! Sujetos (dentro de `{prefix}.catalog.>`), con carga útil `engine.v1.CatalogLogEntry`:
//! `snapshot`, `change` y `sync`.

mod follower;
mod log;

pub use follower::CatalogFollower;
pub use log::JetStreamCatalogLog;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
