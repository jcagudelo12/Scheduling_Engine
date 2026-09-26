# Scheduling Engine — versión en Java

Réplica en Java del motor en Rust, construida para **comparar rendimiento** con las mismas
funcionalidades. Ver la comparación en [`bench/`](../bench/README.md).

## Paridad con la versión en Rust

Hace exactamente lo mismo y habla los mismos protocolos, así que las dos versiones son
intercambiables:

- **Mismo contrato:** compila los mismos `.proto` de [`proto/`](../proto/). `fake-institution`
  alimenta a cualquiera de las dos.
- **Mismo historial:** escribe y lee NATS JetStream con el mismo formato (`engine.v1`).
- **Misma API HTTP:** `POST /combinations`, `GET /health`, `GET /ready`, con el mismo JSON.
- **Mismos roles y variables:** `SCHED_ROLE=ingest|query|all` y demás `SCHED_*`. Diferencia:
  los puertos se configuran con `SCHED_HTTP_PORT` y `SCHED_GRPC_PORT`.
- **Mismas pruebas:** los 21 casos de prueba de Rust, traducidos a JUnit.

## Arquitectura

Hexagonal, con un módulo Maven por capa (las dependencias apuntan hacia adentro y Maven
impide lo contrario):

```
domain/                  sched-domain         modelo puro, sin dependencias
solver/                  sched-solver         backtracking (depende de domain)
application/             sched-application    casos de uso y puertos (interfaces)
proto/                   sched-proto          código generado desde ../proto + conversiones
adapters/
  gateway-http/          Spring MVC: API del estudiante y sondas
  grpc-catalog/          grpc-java: carga completa del catálogo
  grpc-events/           grpc-java: cambios incrementales
  nats-bus/              jnats: historial compartido en JetStream
  in-memory/             historial en memoria (pruebas)
server/                  sched-server         Spring Boot: composición e inyección
benchmarks/              sched-benchmarks     JMH, equivalentes a los de criterion
```

| Rust | Java |
|---|---|
| tokio | Hilos virtuales de Java 21+ |
| axum | Spring Boot 4 (Spring MVC) |
| tonic + prost | grpc-java + protobuf-java |
| async-nats | jnats |
| criterion | JMH |
| `Arc<RwLock<…>>` | `ReentrantReadWriteLock` |
| `Result<T, E>` | Excepciones verificadas (`AppException`) |

## Comandos

Requiere Java 25. Maven se descarga solo con el wrapper.

```sh
cd java
./mvnw verify                                   # compila y corre las pruebas
java -jar server/target/sched-server.jar        # motor (HTTP :8080, gRPC :50051)
java -jar benchmarks/target/benchmarks.jar      # microbenchmarks con JMH

# Despliegue con varias instancias (desde la raíz del repo)
docker compose -f deploy/docker-compose.java.yml up -d --build
```
