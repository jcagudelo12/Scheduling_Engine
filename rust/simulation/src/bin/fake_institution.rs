//! Institución simulada: habla con la instancia de ingesta por el contrato gRPC real,
//! como lo haría el SDK. Sirve para probar el despliegue de punta a punta.
//!
//! ```sh
//! cargo run -p sched-simulation --bin fake-institution -- load [escenario.json] [seq]
//! cargo run -p sched-simulation --bin fake-institution -- seats <seq> <grupo> <cupos>
//! ```
//!
//! `SCHED_INGEST_URL` indica el motor (por defecto `http://localhost:50051`).

use pb::catalog_events_service_client::CatalogEventsServiceClient;
use pb::catalog_service_client::CatalogServiceClient;
use sched_proto::convert;
use sched_proto::institution::v1 as pb;
use sched_simulation::scenario;

const DEFAULT_SCENARIO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sample.json");
const SECTIONS_PER_CHUNK: usize = 100;

type BoxError = Box<dyn std::error::Error>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), BoxError> {
    let url = std::env::var("SCHED_INGEST_URL").unwrap_or_else(|_| "http://localhost:50051".into());
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["load", rest @ ..] => {
            let path = rest.first().copied().unwrap_or(DEFAULT_SCENARIO);
            let seq = rest.get(1).map(|s| s.parse()).transpose()?;
            load(url, path, seq).await
        }
        ["seats", seq, section, available] => {
            seats(url, seq.parse()?, section, available.parse()?).await
        }
        _ => {
            Err("uso: fake-institution load [escenario] [seq] | seats <seq> <grupo> <cupos>".into())
        }
    }
}

async fn load(url: String, path: &str, seq: Option<u64>) -> Result<(), BoxError> {
    let scenario = scenario::load(path)?;
    let seq = seq.unwrap_or(scenario.catalog_seq);
    let sections: Vec<_> = scenario
        .sections
        .iter()
        .map(|(section, available)| convert::section_to_pb(section, *available))
        .collect();
    let chunks: Vec<_> = sections
        .chunks(SECTIONS_PER_CHUNK)
        .map(|chunk| pb::CatalogChunk {
            seq,
            sections: chunk.to_vec(),
        })
        .collect();

    let mut client = CatalogServiceClient::connect(url).await?;
    let response = client
        .load_catalog(tokio_stream::iter(chunks))
        .await?
        .into_inner();
    println!(
        "catálogo cargado: seq={} grupos={}",
        response.seq, response.sections_loaded
    );
    Ok(())
}

async fn seats(url: String, seq: u64, section: &str, available: u32) -> Result<(), BoxError> {
    let event = pb::CatalogEvent {
        seq,
        change: Some(pb::catalog_event::Change::SeatsChanged(pb::SeatsChanged {
            section_id: section.to_owned(),
            available_seats: available,
        })),
    };

    let mut client = CatalogEventsServiceClient::connect(url).await?;
    let mut statuses = client
        .sync_changes(tokio_stream::iter([event]))
        .await?
        .into_inner();
    while let Some(status) = statuses.message().await? {
        println!("respuesta del motor: {:?}", status.status);
    }
    Ok(())
}
