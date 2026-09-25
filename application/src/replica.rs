use std::sync::{Arc, PoisonError, RwLock};

use sched_domain::{ApplyOutcome, Catalog, CatalogEvent};

use crate::AppError;

#[derive(Debug, Default)]
struct State {
    catalog: Option<Catalog>,
    stale: bool,
}

/// Réplica en memoria del catálogo institucional (ADR 0003), compartida entre casos de uso.
///
/// Queda desactualizada (`stale`) cuando se detecta un hueco de secuencia o se pierde la
/// conexión con la institución. Vuelve a estar al día con una carga completa o cuando se
/// aplica el siguiente evento sin huecos (tras un hueco eso solo ocurre después de recargar).
#[derive(Debug, Clone, Default)]
pub struct CatalogReplica(Arc<RwLock<State>>);

impl CatalogReplica {
    /// Lee el catálogo junto con su marca de desactualización.
    pub fn read<T>(&self, f: impl FnOnce(&Catalog, bool) -> T) -> Result<T, AppError> {
        let state = self.0.read().unwrap_or_else(PoisonError::into_inner);
        let catalog = state.catalog.as_ref().ok_or(AppError::CatalogUnavailable)?;
        Ok(f(catalog, state.stale))
    }

    pub fn is_stale(&self) -> bool {
        self.0.read().unwrap_or_else(PoisonError::into_inner).stale
    }

    pub(crate) fn replace(&self, catalog: Catalog) {
        let mut state = self.0.write().unwrap_or_else(PoisonError::into_inner);
        state.catalog = Some(catalog);
        state.stale = false;
    }

    pub(crate) fn apply(&self, event: CatalogEvent) -> Result<ApplyOutcome, AppError> {
        let mut state = self.0.write().unwrap_or_else(PoisonError::into_inner);
        let catalog = state.catalog.as_mut().ok_or(AppError::CatalogUnavailable)?;
        let result = catalog.apply(event);
        match result {
            Ok(ApplyOutcome::Applied) => state.stale = false,
            Ok(ApplyOutcome::Duplicate) => {}
            Err(_) => state.stale = true,
        }
        Ok(result?)
    }

    pub(crate) fn mark_stale(&self) {
        self.0.write().unwrap_or_else(PoisonError::into_inner).stale = true;
    }
}
