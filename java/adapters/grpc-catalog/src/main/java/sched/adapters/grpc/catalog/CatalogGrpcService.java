package sched.adapters.grpc.catalog;

import io.grpc.Status;
import io.grpc.stub.StreamObserver;
import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.AppException;
import sched.application.IngestCatalog;
import sched.domain.SectionSeats;
import sched.proto.Convert;
import sched.proto.institution.v1.CatalogChunk;
import sched.proto.institution.v1.CatalogServiceGrpc;
import sched.proto.institution.v1.LoadCatalogResponse;

/**
 * Carga completa del catálogo (ADR 0003): la institución lo envía en bloques por streaming y
 * la instancia de ingesta lo escribe en el historial compartido (ADR 0005).
 */
public final class CatalogGrpcService extends CatalogServiceGrpc.CatalogServiceImplBase {

    private static final Logger log = LoggerFactory.getLogger(CatalogGrpcService.class);

    private final IngestCatalog loadCatalog;

    public CatalogGrpcService(IngestCatalog loadCatalog) {
        this.loadCatalog = loadCatalog;
    }

    @Override
    public StreamObserver<CatalogChunk> loadCatalog(StreamObserver<LoadCatalogResponse> response) {
        // Se acumula la carga completa antes de reemplazar la réplica: si el stream se
        // corta a la mitad, la réplica anterior queda intacta.
        return new StreamObserver<>() {
            private Long seq;
            private final List<SectionSeats> sections = new ArrayList<>();
            private boolean failed;

            @Override
            public void onNext(CatalogChunk chunk) {
                if (failed) {
                    return;
                }
                if (seq == null) {
                    seq = chunk.getSeq();
                } else if (seq != chunk.getSeq()) {
                    fail(Status.INVALID_ARGUMENT.withDescription(
                            "bloques con seq distinto en una misma carga: " + seq + " y " + chunk.getSeq()));
                    return;
                }
                try {
                    for (var section : chunk.getSectionsList()) {
                        sections.add(Convert.section(section));
                    }
                } catch (Convert.ConversionException e) {
                    fail(Status.INVALID_ARGUMENT.withDescription(e.getMessage()));
                }
            }

            @Override
            public void onError(Throwable error) {
                log.warn("se interrumpió la carga del catálogo: {}", error.getMessage());
            }

            @Override
            public void onCompleted() {
                if (failed) {
                    return;
                }
                if (seq == null) {
                    fail(Status.INVALID_ARGUMENT.withDescription("carga de catálogo vacía"));
                    return;
                }
                try {
                    int loaded = loadCatalog.execute(seq, sections);
                    response.onNext(LoadCatalogResponse.newBuilder()
                            .setSeq(seq)
                            .setSectionsLoaded(loaded)
                            .build());
                    response.onCompleted();
                } catch (AppException e) {
                    log.error("no se pudo cargar el catálogo: {}", e.getMessage());
                    fail(Status.INTERNAL.withDescription(e.getMessage()));
                }
            }

            private void fail(Status status) {
                failed = true;
                response.onError(status.asRuntimeException());
            }
        };
    }
}
