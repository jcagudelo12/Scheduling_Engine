//! Puertos de salida. Cada trait lo implementa un adaptador.
//!
//! El motor no consulta a la institución (ADR 0002): los datos entran por los casos de
//! uso de ingesta y se escriben en el historial compartido del catálogo (ADR 0005).

use std::future::Future;

use crate::{CatalogLogEntry, PortError};

/// Historial compartido del catálogo. Implementaciones: `nats_bus` (JetStream) e
/// `in_memory` (una sola instancia, simulación y pruebas).
pub trait CatalogLog: Send + Sync {
    /// Escribe una entrada al final del historial. Solo la instancia de ingesta escribe.
    fn append(&self, entry: CatalogLogEntry) -> impl Future<Output = Result<(), PortError>> + Send;
}

// Permite compartir un mismo adaptador entre varios casos de uso.
impl<T: CatalogLog> CatalogLog for std::sync::Arc<T> {
    fn append(&self, entry: CatalogLogEntry) -> impl Future<Output = Result<(), PortError>> + Send {
        (**self).append(entry)
    }
}
