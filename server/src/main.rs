//! Punto de entrada. Es el único lugar que conoce todas las implementaciones concretas.

mod config;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use sched_adapter_gateway_http::{health_router, router};
use sched_adapter_grpc_catalog::CatalogGrpc;
use sched_adapter_grpc_events::CatalogEventsGrpc;
use sched_adapter_nats_bus::{CatalogFollower, JetStreamCatalogLog};
use sched_application::use_cases::{
    ApplyLogEntry, GenerateCombinations, IngestCatalog, IngestCatalogEvent,
};
use sched_application::{CatalogReplica, IngestState, Readiness};
use sched_solver::BacktrackingSolver;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type Task = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send>>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    tracing::info!(?config, "iniciando motor de horarios");

    // Historial compartido y réplica local (todas las instancias).
    let nats = async_nats::connect(&config.nats_url).await?;
    let (log, stream) =
        JetStreamCatalogLog::connect(nats, &config.nats_stream, &config.nats_subject_prefix)
            .await?;
    let log = Arc::new(log);
    let replica = CatalogReplica::default();
    let readiness = Readiness::default();
    CatalogFollower::new(
        stream,
        Arc::new(ApplyLogEntry::new(replica.clone())),
        readiness.clone(),
    )
    .spawn();

    // HTTP: sondas siempre; consultas de estudiantes según el rol.
    let mut app = health_router(readiness.clone());
    if config.role.serves_queries() {
        let generate =
            GenerateCombinations::new(BacktrackingSolver, replica.clone(), config.max_combinations);
        app = app.merge(router(Arc::new(generate)));
    }
    let listener = tokio::net::TcpListener::bind(config.http_addr).await?;
    let mut tasks: Vec<Task> = vec![Box::pin(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    })];

    // gRPC de ingesta: solo cuando la réplica local ya está al día, porque de ella sale
    // el último seq aceptado.
    if config.role.ingests() {
        let ingest = IngestState::new(replica);
        let load = Arc::new(IngestCatalog::new(ingest.clone(), Arc::clone(&log)));
        let apply = Arc::new(IngestCatalogEvent::new(ingest, log));
        let grpc_addr = config.grpc_addr;
        tasks.push(Box::pin(async move {
            wait_until_ready(&readiness).await;
            tracing::info!(grpc = %grpc_addr, "ingesta escuchando");
            Server::builder()
                .add_service(CatalogGrpc::new(load).into_service())
                .add_service(CatalogEventsGrpc::new(apply).into_service())
                .serve_with_shutdown(grpc_addr, shutdown_signal())
                .await?;
            Ok(())
        }));
    }

    tracing::info!(http = %config.http_addr, role = ?config.role, "escuchando");
    try_join_all(tasks).await
}

async fn wait_until_ready(readiness: &Readiness) {
    while !readiness.is_ready() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Ejecuta las tareas en paralelo y termina con el primer error.
async fn try_join_all(tasks: Vec<Task>) -> Result<(), BoxError> {
    let mut set = tokio::task::JoinSet::new();
    for task in tasks {
        set.spawn(task);
    }
    while let Some(result) = set.join_next().await {
        result??;
    }
    Ok(())
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(%err, "no se pudo escuchar la señal de apagado");
    }
}
