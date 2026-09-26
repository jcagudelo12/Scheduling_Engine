package sched.server;

import io.grpc.BindableService;
import io.grpc.Grpc;
import io.grpc.InsecureServerCredentials;
import io.grpc.Server;
import java.io.IOException;
import java.util.List;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.Readiness;

/**
 * Servidor gRPC de ingesta. Abre solo cuando la réplica local ya está al día, porque de ella
 * sale el último {@code seq} aceptado.
 */
final class IngestGrpcServer {

    private static final Logger log = LoggerFactory.getLogger(IngestGrpcServer.class);

    private final int port;
    private final Readiness readiness;
    private final List<BindableService> services;
    private volatile Server server;
    private Thread starter;

    IngestGrpcServer(int port, Readiness readiness, List<BindableService> services) {
        this.port = port;
        this.readiness = readiness;
        this.services = services;
    }

    void start() {
        starter = Thread.ofVirtual().name("ingest-starter").start(() -> {
            try {
                while (!readiness.isReady()) {
                    Thread.sleep(100);
                }
                var builder = Grpc.newServerBuilderForPort(port, InsecureServerCredentials.create())
                        .executor(Executors.newVirtualThreadPerTaskExecutor());
                services.forEach(builder::addService);
                server = builder.build().start();
                log.info("ingesta escuchando en gRPC :{}", port);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (IOException e) {
                log.error("no se pudo abrir el puerto gRPC {}: {}", port, e.getMessage());
            }
        });
    }

    void stop() throws InterruptedException {
        starter.interrupt();
        if (server != null) {
            server.shutdown().awaitTermination(5, TimeUnit.SECONDS);
        }
    }
}
