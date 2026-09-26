package sched.adapters.nats;

import io.nats.client.Connection;
import io.nats.client.JetStream;
import io.nats.client.JetStreamApiException;
import io.nats.client.JetStreamManagement;
import io.nats.client.PurgeOptions;
import io.nats.client.api.StorageType;
import io.nats.client.api.StreamConfiguration;
import java.io.IOException;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.CatalogLogEntry;
import sched.application.PortException;
import sched.application.ports.CatalogLog;
import sched.domain.SectionSeats;
import sched.proto.Convert;
import sched.proto.engine.v1.SnapshotChunk;
import sched.proto.engine.v1.SyncLost;

/**
 * Implementa {@link CatalogLog} sobre NATS JetStream. Mismo stream, sujetos y formato
 * ({@code engine.v1.CatalogLogEntry}) que la versión en Rust.
 */
public final class JetStreamCatalogLog implements CatalogLog {

    private static final Logger log = LoggerFactory.getLogger(JetStreamCatalogLog.class);

    /** Grupos por mensaje en una carga completa, para no superar el tamaño máximo de NATS. */
    private static final int SECTIONS_PER_CHUNK = 500;

    private final JetStream js;
    private final JetStreamManagement jsm;
    private final String streamName;
    private final String prefix;

    private JetStreamCatalogLog(Connection connection, String streamName, String prefix) throws IOException {
        this.js = connection.jetStream();
        this.jsm = connection.jetStreamManagement();
        this.streamName = streamName;
        this.prefix = prefix;
    }

    /** Crea el stream si no existe. */
    public static JetStreamCatalogLog connect(Connection connection, String streamName, String prefix)
            throws IOException, JetStreamApiException {
        JetStreamCatalogLog catalogLog = new JetStreamCatalogLog(connection, streamName, prefix);
        try {
            catalogLog.jsm.getStreamInfo(streamName);
        } catch (JetStreamApiException notFound) {
            catalogLog.jsm.addStream(StreamConfiguration.builder()
                    .name(streamName)
                    .subjects(prefix + ".catalog.>")
                    .storageType(StorageType.File)
                    .build());
        }
        return catalogLog;
    }

    @Override
    public void append(CatalogLogEntry entry) throws PortException {
        var builder = sched.proto.engine.v1.CatalogLogEntry.newBuilder();
        switch (entry) {
            case CatalogLogEntry.Snapshot(long seq, List<SectionSeats> sections) -> appendSnapshot(seq, sections);
            case CatalogLogEntry.Change(var event) ->
                    publish("change", builder.setChange(Convert.catalogEventToPb(event)).build());
            case CatalogLogEntry.SyncLost() -> publish("sync", builder.setSyncLost(
                    SyncLost.newBuilder().setReason("se perdió la sincronía con la institución")).build());
        }
    }

    private void appendSnapshot(long seq, List<SectionSeats> sections) throws PortException {
        int total = Math.max(1, (sections.size() + SECTIONS_PER_CHUNK - 1) / SECTIONS_PER_CHUNK);
        long firstSequence = -1;
        for (int index = 0; index < total; index++) {
            var chunk = SnapshotChunk.newBuilder().setSeq(seq).setIndex(index).setTotal(total);
            int from = index * SECTIONS_PER_CHUNK;
            for (SectionSeats s : sections.subList(from, Math.min(sections.size(), from + SECTIONS_PER_CHUNK))) {
                chunk.addSections(Convert.sectionToPb(s.section(), s.available()));
            }
            long sequence = publish("snapshot",
                    sched.proto.engine.v1.CatalogLogEntry.newBuilder().setSnapshotChunk(chunk).build());
            if (firstSequence < 0) {
                firstSequence = sequence;
            }
        }
        purgeBefore(firstSequence);
    }

    private long publish(String kind, sched.proto.engine.v1.CatalogLogEntry entry) throws PortException {
        try {
            return js.publish(prefix + ".catalog." + kind, entry.toByteArray()).getSeqno();
        } catch (IOException | JetStreamApiException e) {
            throw new PortException(PortException.Kind.UNAVAILABLE, e.getMessage(), e);
        }
    }

    /**
     * Borra lo anterior a la última carga completa: una instancia nueva solo necesita esa carga y
     * los cambios posteriores.
     */
    private void purgeBefore(long sequence) {
        try {
            jsm.purgeStream(streamName, PurgeOptions.builder().sequence(sequence).build());
        } catch (IOException | JetStreamApiException e) {
            // No es crítico: el historial queda más largo, pero sigue siendo correcto.
            log.warn("no se pudo compactar el historial del catálogo: {}", e.getMessage());
        }
    }
}
