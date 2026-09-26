use sched_domain::Catalog;

use crate::{AppError, CatalogLogEntry, CatalogReplica};

/// Aplica una entrada del historial compartido a la réplica local.
///
/// Todas las instancias lo ejecutan con las mismas entradas y en el mismo orden,
/// así que todas terminan con el mismo catálogo.
#[derive(Debug)]
pub struct ApplyLogEntry {
    replica: CatalogReplica,
}

impl ApplyLogEntry {
    pub fn new(replica: CatalogReplica) -> Self {
        Self { replica }
    }

    pub fn execute(&self, entry: CatalogLogEntry) -> Result<(), AppError> {
        match entry {
            CatalogLogEntry::Snapshot { seq, sections } => {
                let catalog = Catalog::load(seq, sections);
                tracing::info!(seq, sections = catalog.len(), "réplica recargada");
                self.replica.replace(catalog);
            }
            CatalogLogEntry::Change(event) => {
                // Un hueco aquí indica un historial inconsistente; la réplica queda
                // desactualizada hasta la siguiente carga completa.
                self.replica.apply(event)?;
            }
            CatalogLogEntry::SyncLost => self.replica.mark_stale(),
        }
        Ok(())
    }
}
