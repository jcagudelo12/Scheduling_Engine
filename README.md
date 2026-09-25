# Scheduling Engine

Motor de generación de combinaciones de horario para estudiantes, en Rust.

## Arquitectura

Hexagonal, con un crate por capa ([ADR 0001](docs/adr/0001-arquitectura-hexagonal.md)):

```
domain/            sched-domain       modelo puro, sin dependencias
solver/            sched-solver       búsqueda de combinaciones (depende de domain)
application/       sched-application  casos de uso y puertos (traits)
proto/             sched-proto        contrato gRPC con la institución
adapters/
  gateway_http/        API HTTP para el estudiante
  grpc_catalog/        servidor gRPC: carga completa del catálogo desde el SDK
  grpc_events/         servidor gRPC: cambios incrementales del catálogo
  nats_bus/            historial compartido del catálogo en NATS JetStream
  in_memory/           historial en memoria (simulación y pruebas)
simulation/        sched-simulation   harness de evaluación e institución simulada
server/            sched-server       binario de composición
docs/adr/          decisiones de arquitectura
```

## Flujo de datos

La institución empuja toda la información y el motor nunca la consulta
([ADR 0002](docs/adr/0002-roles-de-la-institucion.md)):

- **Catálogo y cupos:** el SDK hace una carga completa y luego envía cambios con `seq`;
  el motor mantiene una réplica en memoria ([ADR 0003](docs/adr/0003-replica-del-catalogo.md)).
- **Varias instancias:** una instancia de ingesta escribe en un historial de NATS
  JetStream y todas lo leen para armar su réplica
  ([ADR 0005](docs/adr/0005-varias-instancias-con-historial-compartido.md)).
- **Contexto del estudiante:** llega en cada solicitud dentro de un token firmado por la
  institución ([ADR 0004](docs/adr/0004-contexto-del-estudiante-en-token-firmado.md),
  pendiente: hoy el endpoint HTTP lo recibe sin firmar).

## Tecnologías

### Lenguaje y herramientas base

| Tecnología | Para qué se usa |
|---|---|
| [Rust](https://www.rust-lang.org/) 1.98 (edición 2024) | Lenguaje de todo el proyecto. Se eligió porque no tiene recolector de basura: responde en tiempos predecibles aunque haya muchos estudiantes consultando a la vez. |
| [Cargo](https://doc.rust-lang.org/cargo/) (workspace) | Compila, prueba y gestiona dependencias. Cada capa es un crate distinto, así el compilador impide que el dominio dependa de la infraestructura. |
| `rustfmt` y `clippy` | Formateo automático del código y revisión de malas prácticas. |

### Librerías (crates)

| Crate | Para qué se usa | Dónde |
|---|---|---|
| [tokio](https://tokio.rs/) | Ejecuta código asíncrono: atiende muchas conexiones a la vez sin un hilo por cada una. | `server`, adaptadores |
| [axum](https://github.com/tokio-rs/axum) | Servidor HTTP: la API que usa el estudiante y las sondas `/health` y `/ready`. | `adapters/gateway_http` |
| [tonic](https://github.com/hyperium/tonic) | Servidor gRPC: por aquí la institución envía el catálogo y los cambios de cupo. | `adapters/grpc_catalog`, `adapters/grpc_events` |
| [prost](https://github.com/tokio-rs/prost) y [protox](https://github.com/andrewhickman/protox) | Convierten los archivos `.proto` en código Rust. `protox` evita instalar `protoc`. | `proto` |
| [async-nats](https://github.com/nats-io/nats.rs) | Cliente de NATS JetStream: escribe y lee el historial compartido del catálogo. | `adapters/nats_bus` |
| [serde](https://serde.rs/) y `serde_json` | Convierten datos entre Rust y JSON (API HTTP y escenarios de simulación). | `gateway_http`, `simulation` |
| [thiserror](https://github.com/dtolnay/thiserror) | Define los tipos de error de forma concisa. | `application` |
| [tracing](https://github.com/tokio-rs/tracing) | Registro de lo que hace el motor (logs estructurados). | Todos los crates con I/O |
| [criterion](https://github.com/bheisler/criterion.rs) | Mide el rendimiento del solver con estadísticas. | `solver/benches` |

### Comunicación

| Tecnología | Para qué se usa |
|---|---|
| **gRPC + Protocol Buffers** | Contrato entre la institución y el motor. Mensajes binarios, compactos y con tipos; la institución genera su SDK en su propio lenguaje a partir de los `.proto`. |
| **HTTP + JSON** | API para el estudiante (provisional hasta el token firmado y el gateway definitivo). |
| **[NATS JetStream](https://docs.nats.io/nats-concepts/jetstream)** | Historial compartido: la instancia de ingesta escribe los cambios del catálogo y todas las instancias los leen en el mismo orden. |

### Despliegue e infraestructura

| Tecnología | Para qué se usa |
|---|---|
| [Docker](https://www.docker.com/) | Empaqueta el motor en una sola imagen; la variable `SCHED_ROLE` decide qué hace cada contenedor. |
| [Docker Compose](https://docs.docker.com/compose/) | Levanta en local NATS, la instancia de ingesta, varias instancias de consulta y el balanceador. |
| [Traefik](https://traefik.io/traefik/) | Balanceador de carga: reparte las solicitudes entre las instancias y solo envía tráfico a las que ya tienen el catálogo al día. |
| [GitHub Actions](https://docs.github.com/actions) | Integración continua: en cada push revisa formato, `clippy`, pruebas y la simulación. |

### Planeadas (aún no implementadas)

| Tecnología | Para qué se usará |
|---|---|
| **WebSocket** | Avisar en tiempo real al estudiante cuando cambia un cupo mientras arma su horario. |
| **JWT firmado con Ed25519** | Recibir los datos del estudiante firmados por la institución, para que no se puedan alterar ([ADR 0004](docs/adr/0004-contexto-del-estudiante-en-token-firmado.md)). |

## Comandos

```sh
cargo test --workspace                                        # pruebas
cargo run -p sched-simulation --bin sched-simulation          # escenario de ejemplo
cargo bench -p sched-solver                                   # benchmarks del solver
cargo bench -p sched-simulation --bench catalog_lookup      # búsqueda en el catálogo
```

## Despliegue local con varias instancias

```sh
docker compose -f deploy/docker-compose.yml up -d --build     # NATS, ingesta, 3 consultas, Traefik

# Institución simulada: carga el catálogo y envía un cambio de cupo
cargo run -p sched-simulation --bin fake-institution -- load
cargo run -p sched-simulation --bin fake-institution -- seats 2 MAT101-01 0

curl -X POST localhost:8080/combinations -H 'content-type: application/json' \
  -d '{"student_id":"est-001","eligible_courses":["MAT101","FIS101"]}'

docker compose -f deploy/docker-compose.yml up -d --scale engine=5   # más réplicas; Traefik las toma solo
```

| Servicio | Rol (`SCHED_ROLE`) | Puertos |
|---|---|---|
| `ingest` | `ingest`: recibe datos de la institución | gRPC `:50051` |
| `engine` | `query`: atiende estudiantes, escalable | solo red interna |
| `lb` | Traefik frente a `engine`; solo enruta a réplicas `healthy` | HTTP `:8080`, panel `:8090` |
| `nats` | JetStream con el historial | `:4222`, monitoreo `:8222` |

Cada instancia expone `/health` y `/ready` (lista cuando su réplica está al día).
La configuración se lee de variables `SCHED_*`; ver `server/src/config.rs`.
Los `.proto` se compilan con `protox`, así que no hace falta instalar `protoc`.
