use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::consumer::{DeliverPolicy, pull};
use async_nats::jetstream::stream;
use prost::Message;
use sched_application::use_cases::ApplyLogEntry;
use sched_application::{CatalogLogEntry, Readiness};
use sched_domain::Section;
use sched_proto::convert;
use sched_proto::engine::v1 as log_pb;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

use crate::BoxError;

const RETRY_DELAY: Duration = Duration::from_secs(1);

/// Lee el historial desde el principio (la última carga completa) y lo aplica a la
/// réplica local. Marca la instancia como lista cuando ya no quedan mensajes pendientes.
#[derive(Debug)]
pub struct CatalogFollower {
    stream: stream::Stream,
    apply: Arc<ApplyLogEntry>,
    readiness: Readiness,
}

impl CatalogFollower {
    pub fn new(stream: stream::Stream, apply: Arc<ApplyLogEntry>, readiness: Readiness) -> Self {
        Self {
            stream,
            apply,
            readiness,
        }
    }

    /// Corre en segundo plano. Si se pierde la conexión, marca la réplica como
    /// desactualizada y vuelve a leer el historial desde el principio.
    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(err) = self.follow().await {
                    tracing::warn!(%err, "se interrumpió la lectura del historial del catálogo");
                }
                self.apply_entry(CatalogLogEntry::SyncLost);
                tokio::time::sleep(RETRY_DELAY).await;
            }
        })
    }

    async fn follow(&self) -> Result<(), BoxError> {
        if self.stream.get_info().await?.state.messages == 0 {
            // Historial vacío: no hay nada que esperar, aunque tampoco hay catálogo aún.
            self.readiness.mark_ready();
        }

        let consumer = self
            .stream
            .create_consumer(pull::OrderedConfig {
                deliver_policy: DeliverPolicy::All,
                ..Default::default()
            })
            .await?;
        let mut messages = consumer.messages().await?;
        let mut snapshot = SnapshotBuffer::default();

        while let Some(message) = messages.next().await {
            let message = message?;
            let caught_up = message.info().is_ok_and(|info| info.pending == 0);

            match log_pb::CatalogLogEntry::decode(message.payload.clone()) {
                Ok(entry) => {
                    if let Some(entry) = self.to_domain(entry, &mut snapshot) {
                        self.apply_entry(entry);
                    }
                }
                Err(err) => tracing::error!(%err, "entrada del historial ilegible"),
            }

            if caught_up && !self.readiness.is_ready() {
                tracing::info!("réplica al día con el historial");
                self.readiness.mark_ready();
            }
        }
        Err("el stream del historial terminó".into())
    }

    fn to_domain(
        &self,
        entry: log_pb::CatalogLogEntry,
        snapshot: &mut SnapshotBuffer,
    ) -> Option<CatalogLogEntry> {
        use log_pb::catalog_log_entry::Entry;

        match entry.entry? {
            Entry::SnapshotChunk(chunk) => snapshot.push(chunk),
            Entry::Change(event) => match convert::catalog_event(event) {
                Ok(event) => Some(CatalogLogEntry::Change(event)),
                Err(err) => {
                    tracing::error!(%err, "cambio inválido en el historial");
                    None
                }
            },
            Entry::SyncLost(_) => Some(CatalogLogEntry::SyncLost),
        }
    }

    fn apply_entry(&self, entry: CatalogLogEntry) {
        if let Err(err) = self.apply.execute(entry) {
            tracing::warn!(%err, "la réplica no pudo aplicar la entrada del historial");
        }
    }
}

/// Reúne los bloques de una carga completa antes de aplicarla.
#[derive(Debug, Default)]
struct SnapshotBuffer {
    seq: u64,
    total: u32,
    next_index: u32,
    sections: Vec<(Section, u32)>,
}

impl SnapshotBuffer {
    fn push(&mut self, chunk: log_pb::SnapshotChunk) -> Option<CatalogLogEntry> {
        if chunk.index == 0 {
            *self = Self {
                seq: chunk.seq,
                total: chunk.total,
                ..Self::default()
            };
        } else if chunk.seq != self.seq || chunk.index != self.next_index {
            tracing::error!(
                seq = chunk.seq,
                index = chunk.index,
                "bloque de carga fuera de orden"
            );
            *self = Self::default();
            return None;
        }

        for section in chunk.sections {
            match convert::section(section) {
                Ok(section) => self.sections.push(section),
                Err(err) => tracing::error!(%err, "grupo inválido en la carga completa"),
            }
        }
        self.next_index += 1;

        (self.next_index == self.total).then(|| CatalogLogEntry::Snapshot {
            seq: self.seq,
            sections: std::mem::take(&mut self.sections),
        })
    }
}
