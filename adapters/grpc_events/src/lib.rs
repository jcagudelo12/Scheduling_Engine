//! Cambios incrementales del catálogo: la institución envía eventos en orden de `seq`
//! y el motor responde a cada uno con `Ack`, `ResyncRequired` o `Rejected`. Los cambios
//! aceptados se escriben en el historial compartido (ADR 0005).

use std::pin::Pin;
use std::sync::Arc;

use pb::catalog_events_service_server::{CatalogEventsService, CatalogEventsServiceServer};
use pb::sync_status::Status as SyncState;
use sched_application::AppError;
use sched_application::ports::CatalogLog;
use sched_application::use_cases::IngestCatalogEvent;
use sched_proto::convert;
use sched_proto::institution::v1 as pb;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

type StatusStream = Pin<Box<dyn Stream<Item = Result<pb::SyncStatus, Status>> + Send>>;

#[derive(Debug)]
pub struct CatalogEventsGrpc<E> {
    apply_event: Arc<IngestCatalogEvent<E>>,
}

impl<E: CatalogLog + 'static> CatalogEventsGrpc<E> {
    pub fn new(apply_event: Arc<IngestCatalogEvent<E>>) -> Self {
        Self { apply_event }
    }

    pub fn into_service(self) -> CatalogEventsServiceServer<Self> {
        CatalogEventsServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl<E: CatalogLog + 'static> CatalogEventsService for CatalogEventsGrpc<E> {
    type SyncChangesStream = StatusStream;

    async fn sync_changes(
        &self,
        request: Request<Streaming<pb::CatalogEvent>>,
    ) -> Result<Response<Self::SyncChangesStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(64);
        let apply_event = Arc::clone(&self.apply_event);

        tokio::spawn(async move {
            loop {
                let status = match inbound.message().await {
                    Ok(Some(event)) => handle(&apply_event, event).await,
                    Ok(None) => break,
                    Err(err) => {
                        tracing::warn!(%err, "error en el stream de cambios del catálogo");
                        break;
                    }
                };
                if tx
                    .send(Ok(pb::SyncStatus {
                        status: Some(status),
                    }))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            apply_event.connection_lost().await;
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

async fn handle<E: CatalogLog>(
    apply_event: &IngestCatalogEvent<E>,
    event: pb::CatalogEvent,
) -> SyncState {
    let seq = event.seq;
    let event = match convert::catalog_event(event) {
        Ok(event) => event,
        Err(err) => {
            return SyncState::Rejected(pb::Rejected {
                seq,
                reason: err.to_string(),
            });
        }
    };

    match apply_event.execute(event).await {
        Ok(_) => SyncState::Ack(pb::Ack { seq }),
        // Sin carga previa, la institución debe empezar con una carga completa.
        Err(AppError::CatalogUnavailable) => SyncState::ResyncRequired(pb::ResyncRequired {
            expected_seq: 0,
            received_seq: seq,
        }),
        Err(AppError::SequenceGap(gap)) => SyncState::ResyncRequired(pb::ResyncRequired {
            expected_seq: gap.expected,
            received_seq: gap.received,
        }),
        Err(err) => SyncState::Rejected(pb::Rejected {
            seq,
            reason: err.to_string(),
        }),
    }
}
