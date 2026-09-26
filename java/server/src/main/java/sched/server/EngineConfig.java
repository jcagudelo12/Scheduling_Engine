package sched.server;

import io.nats.client.Connection;
import io.nats.client.Nats;
import java.util.List;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.autoconfigure.condition.ConditionalOnExpression;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import sched.adapters.grpc.catalog.CatalogGrpcService;
import sched.adapters.grpc.events.CatalogEventsGrpcService;
import sched.adapters.http.CombinationsController;
import sched.adapters.http.HealthController;
import sched.adapters.nats.CatalogFollower;
import sched.adapters.nats.JetStreamCatalogLog;
import sched.application.ApplyLogEntry;
import sched.application.CatalogReplica;
import sched.application.GenerateCombinations;
import sched.application.IngestCatalog;
import sched.application.IngestCatalogEvent;
import sched.application.IngestState;
import sched.application.Readiness;
import sched.solver.BacktrackingSolver;

/** Composición: crea los adaptadores concretos y los inyecta en los casos de uso. */
@Configuration
class EngineConfig {

    private static final String SERVES_QUERIES = "'${sched.role}' == 'query' or '${sched.role}' == 'all'";
    private static final String INGESTS = "'${sched.role}' == 'ingest' or '${sched.role}' == 'all'";

    @Bean
    Role role(@Value("${sched.role}") String role) {
        return Role.parse(role);
    }

    // Historial compartido y réplica local (todas las instancias).

    @Bean(destroyMethod = "close")
    Connection nats(@Value("${sched.nats.url}") String url) throws Exception {
        return Nats.connect(url);
    }

    @Bean
    JetStreamCatalogLog catalogLog(
            Connection nats,
            @Value("${sched.nats.stream}") String stream,
            @Value("${sched.nats.subject-prefix}") String prefix) throws Exception {
        return JetStreamCatalogLog.connect(nats, stream, prefix);
    }

    @Bean
    CatalogReplica replica() {
        return new CatalogReplica();
    }

    @Bean
    Readiness readiness() {
        return new Readiness();
    }

    @Bean(initMethod = "start", destroyMethod = "close")
    CatalogFollower follower(
            Connection nats,
            @Value("${sched.nats.stream}") String stream,
            CatalogReplica replica,
            Readiness readiness,
            JetStreamCatalogLog catalogLog) {
        // catalogLog se inyecta para garantizar que el stream exista antes de leerlo.
        return new CatalogFollower(nats, stream, new ApplyLogEntry(replica), readiness);
    }

    // HTTP: sondas siempre; consultas de estudiantes según el rol.

    @Bean
    HealthController healthController(Readiness readiness) {
        return new HealthController(readiness);
    }

    @Bean
    @ConditionalOnExpression(SERVES_QUERIES)
    CombinationsController combinationsController(
            CatalogReplica replica, @Value("${sched.max-combinations}") int maxCombinations) {
        return new CombinationsController(new GenerateCombinations(new BacktrackingSolver(), replica, maxCombinations));
    }

    // gRPC de ingesta, según el rol.

    @Bean(initMethod = "start", destroyMethod = "stop")
    @ConditionalOnExpression(INGESTS)
    IngestGrpcServer ingestGrpcServer(
            @Value("${sched.grpc.port}") int port,
            CatalogReplica replica,
            Readiness readiness,
            JetStreamCatalogLog catalogLog) {
        IngestState ingest = new IngestState(replica);
        return new IngestGrpcServer(port, readiness, List.of(
                new CatalogGrpcService(new IngestCatalog(ingest, catalogLog)),
                new CatalogEventsGrpcService(new IngestCatalogEvent(ingest, catalogLog))));
    }
}
