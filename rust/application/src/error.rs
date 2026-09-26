use sched_domain::SequenceGap;

/// Error que devuelve un adaptador al implementar un puerto.
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("datos inválidos: {0}")]
    InvalidData(String),
    #[error("servicio externo no disponible: {0}")]
    Unavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Port(#[from] PortError),
    #[error("el catálogo aún no se ha cargado")]
    CatalogUnavailable,
    #[error("réplica desincronizada ({0}); hace falta una carga completa")]
    SequenceGap(#[from] SequenceGap),
}
