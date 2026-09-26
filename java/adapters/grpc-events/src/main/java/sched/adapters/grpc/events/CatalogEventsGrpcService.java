package sched.adapters.grpc.events;

import io.grpc.stub.StreamObserver;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.AppException;
import sched.application.IngestCatalogEvent;
import sched.proto.Convert;
import sched.proto.institution.v1.Ack;
import sched.proto.institution.v1.CatalogEvent;
import sched.proto.institution.v1.CatalogEventsServiceGrpc;
import sched.proto.institution.v1.Rejected;
import sched.proto.institution.v1.ResyncRequired;
import sched.proto.institution.v1.SyncStatus;

/**
 * Cambios incrementales del catálogo: la institución envía eventos en orden de {@code seq} y el
 * motor responde a cada uno con {@code Ack}, {@code ResyncRequired} o {@code Rejected}. Los
 * cambios aceptados se escriben en el historial compartido (ADR 0005).
 */
public final class CatalogEventsGrpcService extends CatalogEventsServiceGrpc.CatalogEventsServiceImplBase {

    private static final Logger log = LoggerFactory.getLogger(CatalogEventsGrpcService.class);

    private final IngestCatalogEvent applyEvent;

    public CatalogEventsGrpcService(IngestCatalogEvent applyEvent) {
        this.applyEvent = applyEvent;
    }

    @Override
    public StreamObserver<CatalogEvent> syncChanges(StreamObserver<SyncStatus> responses) {
        return new StreamObserver<>() {
            @Override
            public void onNext(CatalogEvent event) {
                responses.onNext(handle(event));
            }

            @Override
            public void onError(Throwable error) {
                log.warn("error en el stream de cambios del catálogo: {}", error.getMessage());
                applyEvent.connectionLost();
            }

            @Override
            public void onCompleted() {
                applyEvent.connectionLost();
                responses.onCompleted();
            }
        };
    }

    private SyncStatus handle(CatalogEvent pb) {
        long seq = pb.getSeq();
        try {
            applyEvent.execute(Convert.catalogEvent(pb));
            return SyncStatus.newBuilder().setAck(Ack.newBuilder().setSeq(seq)).build();
        } catch (Convert.ConversionException e) {
            return rejected(seq, e.getMessage());
        } catch (AppException.CatalogUnavailable e) {
            // Sin carga previa, la institución debe empezar con una carga completa.
            return resync(0, seq);
        } catch (AppException.SequenceGap gap) {
            return resync(gap.expected(), gap.received());
        } catch (AppException e) {
            return rejected(seq, e.getMessage());
        }
    }

    private static SyncStatus resync(long expected, long received) {
        return SyncStatus.newBuilder()
                .setResyncRequired(ResyncRequired.newBuilder().setExpectedSeq(expected).setReceivedSeq(received))
                .build();
    }

    private static SyncStatus rejected(long seq, String reason) {
        return SyncStatus.newBuilder().setRejected(Rejected.newBuilder().setSeq(seq).setReason(reason)).build();
    }
}
