use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Indica si la réplica local ya se puso al día con el historial compartido.
///
/// El balanceador no debe enviar tráfico a una instancia que aún no está lista.
#[derive(Debug, Clone, Default)]
pub struct Readiness(Arc<AtomicBool>);

impl Readiness {
    pub fn is_ready(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn mark_ready(&self) {
        self.0.store(true, Ordering::Release);
    }
}
