package sched.adapters.inmemory;

import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.AppException;
import sched.application.ApplyLogEntry;
import sched.application.CatalogLogEntry;
import sched.application.ports.CatalogLog;

/**
 * Historial en el mismo proceso: cada entrada se aplica de inmediato a la réplica local.
 *
 * <p>Entra por el mismo puerto que JetStream, así que el flujo evaluado es el mismo que en
 * producción.
 */
public final class InMemoryCatalogLog implements CatalogLog {

    private static final Logger log = LoggerFactory.getLogger(InMemoryCatalogLog.class);

    private final ApplyLogEntry follower;
    private final List<CatalogLogEntry> entries = new ArrayList<>();

    public InMemoryCatalogLog(ApplyLogEntry follower) {
        this.follower = follower;
    }

    /** Entradas escritas hasta ahora, para inspeccionarlas en pruebas. */
    public synchronized List<CatalogLogEntry> entries() {
        return List.copyOf(entries);
    }

    @Override
    public synchronized void append(CatalogLogEntry entry) {
        entries.add(entry);
        try {
            follower.execute(entry);
        } catch (AppException e) {
            log.warn("la réplica local no pudo aplicar la entrada: {}", e.getMessage());
        }
    }
}
