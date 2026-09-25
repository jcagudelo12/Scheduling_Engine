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

## Comandos

```sh
cargo test --workspace                                        # pruebas
cargo run -p sched-simulation --bin sched-simulation          # escenario de ejemplo
cargo bench -p sched-solver                                   # benchmarks del solver
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
