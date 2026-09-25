use async_nats::jetstream::{self, stream};
use prost::Message;
use sched_application::ports::CatalogLog;
use sched_application::{CatalogLogEntry, PortError};
use sched_proto::convert;
use sched_proto::engine::v1 as log_pb;

use crate::BoxError;

/// Grupos por mensaje en una carga completa, para no superar el tamaño máximo de NATS.
const SECTIONS_PER_CHUNK: usize = 500;

#[derive(Debug, Clone)]
pub struct JetStreamCatalogLog {
    js: jetstream::Context,
    stream_name: String,
    prefix: String,
}

impl JetStreamCatalogLog {
    /// Crea el stream si no existe y devuelve el historial junto con el stream, que
    /// necesita el seguidor.
    pub async fn connect(
        client: async_nats::Client,
        stream_name: &str,
        prefix: &str,
    ) -> Result<(Self, stream::Stream), BoxError> {
        let js = jetstream::new(client);
        let stream = js
            .get_or_create_stream(stream::Config {
                name: stream_name.to_owned(),
                subjects: vec![format!("{prefix}.catalog.>")],
                storage: stream::StorageType::File,
                ..Default::default()
            })
            .await?;
        let log = Self {
            js,
            stream_name: stream_name.to_owned(),
            prefix: prefix.to_owned(),
        };
        Ok((log, stream))
    }

    async fn publish(
        &self,
        kind: &str,
        entry: log_pb::catalog_log_entry::Entry,
    ) -> Result<u64, PortError> {
        let payload = log_pb::CatalogLogEntry { entry: Some(entry) }.encode_to_vec();
        let ack = self
            .js
            .publish(format!("{}.catalog.{kind}", self.prefix), payload.into())
            .await
            .map_err(unavailable)?
            .await
            .map_err(unavailable)?;
        Ok(ack.sequence)
    }

    /// Borra lo anterior a la última carga completa: una instancia nueva solo necesita
    /// esa carga y los cambios posteriores.
    async fn purge_before(&self, sequence: u64) {
        let result = async {
            let stream = self.js.get_stream(&self.stream_name).await?;
            stream.purge().sequence(sequence).await?;
            Ok::<_, BoxError>(())
        };
        if let Err(err) = result.await {
            // No es crítico: el historial queda más largo, pero sigue siendo correcto.
            tracing::warn!(%err, "no se pudo compactar el historial del catálogo");
        }
    }
}

impl CatalogLog for JetStreamCatalogLog {
    async fn append(&self, entry: CatalogLogEntry) -> Result<(), PortError> {
        use log_pb::catalog_log_entry::Entry;

        match entry {
            CatalogLogEntry::Snapshot { seq, sections } => {
                let chunks: Vec<_> = if sections.is_empty() {
                    vec![&[][..]]
                } else {
                    sections.chunks(SECTIONS_PER_CHUNK).collect()
                };
                let total = u32::try_from(chunks.len())
                    .map_err(|_| PortError::InvalidData("catálogo demasiado grande".into()))?;

                let mut first_sequence = None;
                for (index, chunk) in (0..).zip(chunks) {
                    let sections = chunk
                        .iter()
                        .map(|(s, available)| convert::section_to_pb(s, *available));
                    let sequence = self
                        .publish(
                            "snapshot",
                            Entry::SnapshotChunk(log_pb::SnapshotChunk {
                                seq,
                                index,
                                total,
                                sections: sections.collect(),
                            }),
                        )
                        .await?;
                    first_sequence.get_or_insert(sequence);
                }
                if let Some(sequence) = first_sequence {
                    self.purge_before(sequence).await;
                }
            }
            CatalogLogEntry::Change(event) => {
                self.publish(
                    "change",
                    Entry::Change(convert::catalog_event_to_pb(&event)),
                )
                .await?;
            }
            CatalogLogEntry::SyncLost => {
                let reason = "se perdió la sincronía con la institución".to_owned();
                self.publish("sync", Entry::SyncLost(log_pb::SyncLost { reason }))
                    .await?;
            }
        }
        Ok(())
    }
}

fn unavailable(err: impl std::fmt::Display) -> PortError {
    PortError::Unavailable(err.to_string())
}
