package sched.adapters.nats;

import com.google.protobuf.InvalidProtocolBufferException;
import io.nats.client.Connection;
import io.nats.client.IterableConsumer;
import io.nats.client.Message;
import io.nats.client.api.DeliverPolicy;
import io.nats.client.api.OrderedConsumerConfiguration;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.AppException;
import sched.application.ApplyLogEntry;
import sched.application.CatalogLogEntry;
import sched.application.Readiness;
import sched.domain.SectionSeats;
import sched.proto.Convert;
import sched.proto.engine.v1.SnapshotChunk;

/**
 * Lee el historial desde el principio (la última carga completa) y lo aplica a la réplica
 * local. Marca la instancia como lista cuando ya no quedan mensajes pendientes.
 */
public final class CatalogFollower implements AutoCloseable {

    private static final Logger log = LoggerFactory.getLogger(CatalogFollower.class);
    private static final Duration POLL = Duration.ofSeconds(1);
    private static final Duration RETRY_DELAY = Duration.ofSeconds(1);

    private final Connection connection;
    private final String streamName;
    private final ApplyLogEntry apply;
    private final Readiness readiness;
    private volatile boolean running = true;
    private Thread thread;

    public CatalogFollower(Connection connection, String streamName, ApplyLogEntry apply, Readiness readiness) {
        this.connection = connection;
        this.streamName = streamName;
        this.apply = apply;
        this.readiness = readiness;
    }

    /**
     * Corre en segundo plano. Si se pierde la conexión, marca la réplica como desactualizada y
     * vuelve a leer el historial desde el principio.
     */
    public void start() {
        thread = Thread.ofVirtual().name("catalog-follower").start(() -> {
            while (running) {
                try {
                    follow();
                } catch (InterruptedException e) {
                    return;
                } catch (Exception e) {
                    log.warn("se interrumpió la lectura del historial del catálogo: {}", e.getMessage());
                }
                if (!running) {
                    return;
                }
                applyEntry(new CatalogLogEntry.SyncLost());
                try {
                    Thread.sleep(RETRY_DELAY);
                } catch (InterruptedException e) {
                    return;
                }
            }
        });
    }

    @Override
    public void close() {
        running = false;
        if (thread != null) {
            thread.interrupt();
        }
    }

    private void follow() throws Exception {
        var jsm = connection.jetStreamManagement();
        if (jsm.getStreamInfo(streamName).getStreamState().getMsgCount() == 0) {
            // Historial vacío: no hay nada que esperar, aunque tampoco hay catálogo aún.
            readiness.markReady();
        }

        var consumer = connection.getStreamContext(streamName)
                .createOrderedConsumer(new OrderedConsumerConfiguration().deliverPolicy(DeliverPolicy.All));
        SnapshotBuffer snapshot = new SnapshotBuffer();

        try (IterableConsumer messages = consumer.iterate()) {
            while (running) {
                Message message = messages.nextMessage(POLL);
                if (message == null) {
                    continue;
                }
                boolean caughtUp = message.metaData().pendingCount() == 0;

                try {
                    var entry = sched.proto.engine.v1.CatalogLogEntry.parseFrom(message.getData());
                    CatalogLogEntry domain = toDomain(entry, snapshot);
                    if (domain != null) {
                        applyEntry(domain);
                    }
                } catch (InvalidProtocolBufferException e) {
                    log.error("entrada del historial ilegible: {}", e.getMessage());
                }

                if (caughtUp && !readiness.isReady()) {
                    log.info("réplica al día con el historial");
                    readiness.markReady();
                }
            }
        }
    }

    private static CatalogLogEntry toDomain(sched.proto.engine.v1.CatalogLogEntry entry, SnapshotBuffer snapshot) {
        return switch (entry.getEntryCase()) {
            case SNAPSHOT_CHUNK -> snapshot.push(entry.getSnapshotChunk());
            case CHANGE -> {
                try {
                    yield new CatalogLogEntry.Change(Convert.catalogEvent(entry.getChange()));
                } catch (Convert.ConversionException e) {
                    log.error("cambio inválido en el historial: {}", e.getMessage());
                    yield null;
                }
            }
            case SYNC_LOST -> new CatalogLogEntry.SyncLost();
            case ENTRY_NOT_SET -> null;
        };
    }

    private void applyEntry(CatalogLogEntry entry) {
        try {
            apply.execute(entry);
        } catch (AppException e) {
            log.warn("la réplica no pudo aplicar la entrada del historial: {}", e.getMessage());
        }
    }

    /** Reúne los bloques de una carga completa antes de aplicarla. */
    private static final class SnapshotBuffer {
        private long seq;
        private int total;
        private int nextIndex;
        private List<SectionSeats> sections = new ArrayList<>();

        CatalogLogEntry push(SnapshotChunk chunk) {
            if (chunk.getIndex() == 0) {
                seq = chunk.getSeq();
                total = chunk.getTotal();
                nextIndex = 0;
                sections = new ArrayList<>();
            } else if (chunk.getSeq() != seq || chunk.getIndex() != nextIndex) {
                log.error("bloque de carga fuera de orden: seq={} index={}", chunk.getSeq(), chunk.getIndex());
                total = 0;
                nextIndex = 0;
                sections = new ArrayList<>();
                return null;
            }

            for (var section : chunk.getSectionsList()) {
                try {
                    sections.add(Convert.section(section));
                } catch (Convert.ConversionException e) {
                    log.error("grupo inválido en la carga completa: {}", e.getMessage());
                }
            }
            nextIndex++;

            if (nextIndex == total) {
                CatalogLogEntry snapshot = new CatalogLogEntry.Snapshot(seq, sections);
                sections = new ArrayList<>();
                return snapshot;
            }
            return null;
        }
    }
}
