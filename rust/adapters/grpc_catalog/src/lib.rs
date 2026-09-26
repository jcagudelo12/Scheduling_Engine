//! Carga completa del catálogo (ADR 0003): la institución lo envía en bloques por streaming
//! y la instancia de ingesta lo escribe en el historial compartido (ADR 0005).

use std::sync::Arc;

use pb::catalog_service_server::{CatalogService, CatalogServiceServer};
use sched_application::ports::CatalogLog;
use sched_application::use_cases::IngestCatalog;
use sched_proto::convert;
use sched_proto::institution::v1 as pb;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug)]
pub struct CatalogGrpc<E> {
    load_catalog: Arc<IngestCatalog<E>>,
}

impl<E: CatalogLog + 'static> CatalogGrpc<E> {
    pub fn new(load_catalog: Arc<IngestCatalog<E>>) -> Self {
        Self { load_catalog }
    }

    pub fn into_service(self) -> CatalogServiceServer<Self> {
        CatalogServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl<E: CatalogLog + 'static> CatalogService for CatalogGrpc<E> {
    async fn load_catalog(
        &self,
        request: Request<Streaming<pb::CatalogChunk>>,
    ) -> Result<Response<pb::LoadCatalogResponse>, Status> {
        let mut stream = request.into_inner();
        let mut seq = None;
        let mut sections = Vec::new();

        // Se acumula la carga completa antes de reemplazar la réplica: si el stream se
        // corta a la mitad, la réplica anterior queda intacta.
        while let Some(chunk) = stream.message().await? {
            match seq {
                None => seq = Some(chunk.seq),
                Some(s) if s != chunk.seq => {
                    return Err(Status::invalid_argument(format!(
                        "bloques con seq distinto en una misma carga: {s} y {}",
                        chunk.seq
                    )));
                }
                Some(_) => {}
            }
            for section in chunk.sections {
                sections.push(
                    convert::section(section)
                        .map_err(|err| Status::invalid_argument(err.to_string()))?,
                );
            }
        }

        let seq = seq.ok_or_else(|| Status::invalid_argument("carga de catálogo vacía"))?;
        let loaded = self
            .load_catalog
            .execute(seq, sections)
            .await
            .map_err(|err| {
                tracing::error!(%err, "no se pudo cargar el catálogo");
                Status::internal(err.to_string())
            })?;

        Ok(Response::new(pb::LoadCatalogResponse {
            seq,
            sections_loaded: u32::try_from(loaded).unwrap_or(u32::MAX),
        }))
    }
}
