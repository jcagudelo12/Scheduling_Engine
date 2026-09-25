use sched_domain::Section;

use crate::ports::CatalogLog;
use crate::{AppError, CatalogLogEntry, IngestState};

/// La institución envía el catálogo completo (al conectarse o para resincronizar).
#[derive(Debug)]
pub struct IngestCatalog<L> {
    ingest: IngestState,
    log: L,
}

impl<L: CatalogLog> IngestCatalog<L> {
    pub fn new(ingest: IngestState, log: L) -> Self {
        Self { ingest, log }
    }

    /// `sections` trae cada grupo con sus cupos disponibles. Devuelve cuántos se cargaron.
    pub async fn execute(
        &self,
        seq: u64,
        sections: Vec<(Section, u32)>,
    ) -> Result<usize, AppError> {
        let loaded = sections.len();
        self.log
            .append(CatalogLogEntry::Snapshot { seq, sections })
            .await?;
        self.ingest.reset(seq);
        tracing::info!(
            seq,
            sections = loaded,
            "carga completa escrita en el historial"
        );
        Ok(loaded)
    }
}
