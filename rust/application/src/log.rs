use sched_domain::{CatalogEvent, Section};

/// Entrada del historial compartido del catálogo (ADR 0005).
///
/// La instancia de ingesta las escribe; todas las instancias las leen en el mismo orden
/// y las aplican a su réplica local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogLogEntry {
    /// Carga completa: reemplaza el catálogo.
    Snapshot {
        seq: u64,
        sections: Vec<(Section, u32)>,
    },
    /// Cambio incremental ya validado contra el `seq` anterior.
    Change(CatalogEvent),
    /// La ingesta perdió la sincronía con la institución: las réplicas quedan desactualizadas.
    SyncLost,
}
