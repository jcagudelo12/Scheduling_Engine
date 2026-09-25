//! Corre el motor sobre un escenario de prueba y reporta métricas por estudiante.
//!
//! Uso: `cargo run -p sched-simulation -- [ruta/escenario.json]`

use std::sync::Arc;
use std::time::Instant;

use sched_adapter_in_memory::InMemoryCatalogLog;
use sched_application::use_cases::{ApplyLogEntry, GenerateCombinations, IngestCatalog};
use sched_application::{CatalogReplica, IngestState};
use sched_simulation::scenario;
use sched_solver::BacktrackingSolver;

const DEFAULT_SCENARIO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sample.json");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_SCENARIO.to_owned());
    let scenario = scenario::load(&path)?;

    // El catálogo entra por el mismo caso de uso que usa el SDK de la institución,
    // con el historial en memoria en lugar de JetStream.
    let replica = CatalogReplica::default();
    let log = InMemoryCatalogLog::new(Arc::new(ApplyLogEntry::new(replica.clone())));
    IngestCatalog::new(IngestState::new(replica.clone()), log)
        .execute(scenario.catalog_seq, scenario.sections)
        .await?;
    let generate = GenerateCombinations::new(BacktrackingSolver, replica, scenario.max_results);

    println!("escenario: {path}");
    println!(
        "{:<12} {:>13} {:>12}",
        "estudiante", "combinaciones", "tiempo (µs)"
    );
    for student in &scenario.students {
        let started = Instant::now();
        let result = generate.execute(student)?;
        let elapsed = started.elapsed().as_micros();
        println!(
            "{:<12} {:>13} {:>12}",
            student.id.as_str(),
            result.schedules.len(),
            elapsed
        );
    }
    Ok(())
}
