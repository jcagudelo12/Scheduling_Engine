use std::sync::{Arc, Mutex, PoisonError};

use sched_domain::SequenceGap;

use crate::{AppError, CatalogReplica};

/// Resultado de validar el `seq` de un cambio antes de escribirlo en el historial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// Es el siguiente esperado; queda reservado y debe escribirse.
    Next,
    /// Ya se había aceptado; no se escribe de nuevo.
    Duplicate,
}

#[derive(Debug, Default)]
struct Inner {
    last_seq: Option<u64>,
    needs_resync: bool,
}

/// Estado de la instancia de ingesta: último `seq` aceptado de la institución.
///
/// Es la única fuente de validación de orden (un solo escritor, ADR 0005). Al arrancar
/// se inicializa desde la réplica local, que ya se puso al día con el historial.
#[derive(Debug, Clone)]
pub struct IngestState {
    replica: CatalogReplica,
    inner: Arc<Mutex<Inner>>,
}

impl IngestState {
    pub fn new(replica: CatalogReplica) -> Self {
        Self {
            replica,
            inner: Arc::default(),
        }
    }

    pub(crate) fn admit(&self, seq: u64) -> Result<Admission, AppError> {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if inner.last_seq.is_none() {
            inner.last_seq = self.replica.read(|catalog, _| catalog.seq()).ok();
        }
        let last = inner.last_seq.ok_or(AppError::CatalogUnavailable)?;
        let expected = last + 1;

        if seq < expected {
            return Ok(Admission::Duplicate);
        }
        if inner.needs_resync || seq > expected {
            inner.needs_resync = true;
            return Err(SequenceGap {
                expected,
                received: seq,
            }
            .into());
        }
        inner.last_seq = Some(seq);
        Ok(Admission::Next)
    }

    /// Tras una carga completa escrita en el historial.
    pub(crate) fn reset(&self, seq: u64) {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        inner.last_seq = Some(seq);
        inner.needs_resync = false;
    }

    /// No se pudo escribir un cambio ya admitido: solo una carga completa garantiza
    /// que el historial y la institución vuelvan a coincidir.
    pub(crate) fn require_resync(&self) {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .needs_resync = true;
    }
}
