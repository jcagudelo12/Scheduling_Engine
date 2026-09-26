//! Tipos y stubs gRPC generados a partir de los `.proto`, más las conversiones con el
//! dominio que comparten los adaptadores.
//!
//! - `institution.v1`: contrato público con la institución (base del SDK).
//! - `engine.v1`: contrato interno entre instancias del motor.

pub mod convert;

pub mod institution {
    pub mod v1 {
        tonic::include_proto!("institution.v1");
    }
}

pub mod engine {
    pub mod v1 {
        tonic::include_proto!("engine.v1");
    }
}
