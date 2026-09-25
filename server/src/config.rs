use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

/// Qué hace esta instancia (ADR 0005). Todas mantienen su réplica leyendo el historial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Recibe los datos de la institución por gRPC y los escribe en el historial.
    /// Debe haber una sola instancia con este rol.
    Ingest,
    /// Atiende a los estudiantes. Se puede escalar horizontalmente.
    Query,
    /// Ambos: para desarrollo o despliegues de una sola instancia.
    All,
}

impl Role {
    pub fn ingests(self) -> bool {
        matches!(self, Self::Ingest | Self::All)
    }

    pub fn serves_queries(self) -> bool {
        matches!(self, Self::Query | Self::All)
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ingest" => Ok(Self::Ingest),
            "query" => Ok(Self::Query),
            "all" => Ok(Self::All),
            other => Err(format!(
                "SCHED_ROLE inválido: {other} (use ingest, query o all)"
            )),
        }
    }
}

/// Configuración leída de variables de entorno, con valores por defecto para desarrollo local.
#[derive(Debug, Clone)]
pub struct Config {
    pub role: Role,
    pub http_addr: SocketAddr,
    pub grpc_addr: SocketAddr,
    pub nats_url: String,
    pub nats_subject_prefix: String,
    pub nats_stream: String,
    pub max_combinations: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            role: var("SCHED_ROLE", "all").parse()?,
            http_addr: var("SCHED_HTTP_ADDR", "0.0.0.0:8080").parse()?,
            grpc_addr: var("SCHED_GRPC_ADDR", "0.0.0.0:50051").parse()?,
            nats_url: var("SCHED_NATS_URL", "nats://localhost:4222"),
            nats_subject_prefix: var("SCHED_NATS_SUBJECT_PREFIX", "sched"),
            nats_stream: var("SCHED_NATS_STREAM", "SCHED_CATALOG"),
            max_combinations: var("SCHED_MAX_COMBINATIONS", "200").parse()?,
        })
    }
}

fn var(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}
