//! Compila los .proto con `protox` (en Rust puro), así no hace falta `protoc`.
//!
//! Los .proto están en `proto/` en la raíz del repo, compartidos con la versión en Java.

/// Carpeta de los .proto, relativa a este crate (`rust/proto/`).
const PROTO_ROOT: &str = "../../proto";

const PROTOS: &[&str] = &[
    "institution/v1/common.proto",
    "institution/v1/catalog.proto",
    "institution/v1/catalog_events.proto",
    "engine/v1/catalog_log.proto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptors = protox::compile(PROTOS, [PROTO_ROOT])?;
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_fds(descriptors)?;

    for proto in PROTOS {
        println!("cargo:rerun-if-changed={PROTO_ROOT}/{proto}");
    }
    Ok(())
}
