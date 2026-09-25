# Imagen única del motor; SCHED_ROLE decide si la instancia hace ingesta, consultas o ambas.
FROM rust:1.98-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked -p sched-server

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home sched
COPY --from=builder /app/target/release/sched-server /usr/local/bin/sched-server
USER sched
EXPOSE 8080 50051
HEALTHCHECK --interval=5s --timeout=2s --retries=3 CMD curl -fsS http://localhost:8080/ready || exit 1
ENTRYPOINT ["sched-server"]
