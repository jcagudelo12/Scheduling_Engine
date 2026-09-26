use sched_domain::{ApplyOutcome, CatalogEvent};

use crate::ingest::Admission;
use crate::ports::CatalogLog;
use crate::{AppError, CatalogLogEntry, IngestState};

/// La institución notifica un cambio incremental del catálogo.
#[derive(Debug)]
pub struct IngestCatalogEvent<L> {
    ingest: IngestState,
    log: L,
}

impl<L: CatalogLog> IngestCatalogEvent<L> {
    pub fn new(ingest: IngestState, log: L) -> Self {
        Self { ingest, log }
    }

    /// Valida el orden y escribe el cambio en el historial.
    ///
    /// - `AppError::SequenceGap`: se perdieron eventos; la institución debe enviar una
    ///   carga completa y todas las réplicas quedan desactualizadas mientras tanto.
    /// - `AppError::CatalogUnavailable`: aún no hay carga completa.
    pub async fn execute(&self, event: CatalogEvent) -> Result<ApplyOutcome, AppError> {
        let admission = match self.ingest.admit(event.seq) {
            Ok(admission) => admission,
            Err(err @ AppError::SequenceGap(_)) => {
                tracing::warn!(%err, "hueco en los cambios de la institución");
                self.notify_sync_lost().await;
                return Err(err);
            }
            Err(err) => return Err(err),
        };
        if admission == Admission::Duplicate {
            return Ok(ApplyOutcome::Duplicate);
        }

        if let Err(err) = self.log.append(CatalogLogEntry::Change(event)).await {
            self.ingest.require_resync();
            return Err(err.into());
        }
        Ok(ApplyOutcome::Applied)
    }

    /// Se cerró el canal de cambios: ya no hay garantía de estar al día.
    pub async fn connection_lost(&self) {
        tracing::warn!("se perdió la conexión de cambios con la institución");
        self.notify_sync_lost().await;
    }

    async fn notify_sync_lost(&self) {
        if let Err(err) = self.log.append(CatalogLogEntry::SyncLost).await {
            tracing::error!(%err, "no se pudo avisar la pérdida de sincronía");
        }
    }
}
