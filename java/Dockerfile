# Imagen de la versión en Java. Se construye desde la raíz del repo porque compila los
# mismos .proto de proto/:  docker build -f java/Dockerfile .
FROM maven:3.9.16-eclipse-temurin-25 AS builder
WORKDIR /app
COPY proto ./proto
COPY java ./java
RUN --mount=type=cache,target=/root/.m2 \
    cd java && mvn -q -B package -DskipTests -pl server -am

FROM eclipse-temurin:25-jre
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home sched
COPY --from=builder /app/java/server/target/sched-server.jar /app/sched-server.jar
USER sched
EXPOSE 8080 50051
HEALTHCHECK --interval=5s --timeout=2s --retries=3 CMD curl -fsS http://localhost:8080/ready || exit 1
# La JVM usa hasta el 75 % de la memoria asignada al contenedor.
ENTRYPOINT ["java", "-XX:MaxRAMPercentage=75", "-jar", "/app/sched-server.jar"]
