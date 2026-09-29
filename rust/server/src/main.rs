//! Punto de entrada. Es el único lugar que conoce todas las implementaciones concretas.

mod config;

use std::future::Future;
use std::io::IsTerminal;
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
use tokio::net::TcpSocket;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type Task = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send>>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        // Sin códigos de color cuando la salida no es una terminal (p. ej. en contenedores).
        .with_ansi(std::io::stdout().is_terminal())
        .init();

    let config = Config::from_env()?;
    tracing::info!(?config, "iniciando motor de horarios");
    // tokio crea un hilo de trabajo por CPU detectada; en un contenedor debe coincidir con
    // su límite de CPU. La comparación de rendimiento (bench/) verifica este valor.
    let cpus = std::thread::available_parallelism().map_or(0, |n| n.get());
    tracing::info!(cpus, "CPUs detectadas");
    raise_open_files_limit();

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
    let listener = bind_http(config.http_addr)?;
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

/// Cada conexión abierta ocupa un descriptor de archivo. El límite blando por defecto (1.024
/// en muchos sistemas y contenedores) hace fallar `accept` con muchos estudiantes conectados
/// a la vez. Se sube hasta el límite duro, igual que hace la JVM al arrancar.
fn raise_open_files_limit() {
    match rlimit::increase_nofile_limit(u64::MAX) {
        Ok(limit) => tracing::info!(limit, "límite de archivos abiertos"),
        Err(err) => tracing::warn!(%err, "no se pudo subir el límite de archivos abiertos"),
    }
}

/// Cola de conexiones pendientes del socket HTTP. El valor por defecto de tokio (1.024) se
/// desborda cuando llegan muchas conexiones nuevas a la vez (p. ej. al abrir la matrícula) y
/// las conexiones descartadas quedan esperando reintentos de TCP. El kernel lo limita a
/// `net.core.somaxconn`.
const HTTP_BACKLOG: u32 = 4096;

fn bind_http(addr: std::net::SocketAddr) -> std::io::Result<tokio::net::TcpListener> {
    let socket = if addr.is_ipv4() {
        TcpSocket::new_v4()?
    } else {
        TcpSocket::new_v6()?
    };
    socket.set_reuseaddr(true)?;
    socket.bind(addr)?;
    socket.listen(HTTP_BACKLOG)
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
