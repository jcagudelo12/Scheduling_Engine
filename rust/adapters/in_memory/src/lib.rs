//! Implementaciones en memoria de los puertos, para una sola instancia, simulación y pruebas.
//!
//! Entran por los mismos puertos que los adaptadores reales, así que el código
//! que se evalúa en la simulación es el mismo que corre en producción.

use std::sync::{Arc, Mutex, PoisonError};

use sched_application::ports::CatalogLog;
use sched_application::use_cases::ApplyLogEntry;
use sched_application::{CatalogLogEntry, PortError};

/// Historial en el mismo proceso: cada entrada se aplica de inmediato a la réplica local.
#[derive(Debug)]
pub struct InMemoryCatalogLog {
    follower: Arc<ApplyLogEntry>,
    entries: Mutex<Vec<CatalogLogEntry>>,
}

impl InMemoryCatalogLog {
    pub fn new(follower: Arc<ApplyLogEntry>) -> Self {
        Self {
            follower,
            entries: Mutex::default(),
        }
    }

    /// Entradas escritas hasta ahora, para inspeccionarlas en pruebas.
    pub fn entries(&self) -> Vec<CatalogLogEntry> {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl CatalogLog for InMemoryCatalogLog {
    async fn append(&self, entry: CatalogLogEntry) -> Result<(), PortError> {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(entry.clone());
        if let Err(err) = self.follower.execute(entry) {
            tracing::warn!(%err, "la réplica local no pudo aplicar la entrada");
        }
        Ok(())
    }
}
