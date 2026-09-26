package sched.application;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.domain.Catalog;

/**
 * Caso de uso: aplica una entrada del historial compartido a la réplica local.
 *
 * <p>Todas las instancias lo ejecutan con las mismas entradas y en el mismo orden, así que
 * todas terminan con el mismo catálogo.
 */
public final class ApplyLogEntry {

    private static final Logger log = LoggerFactory.getLogger(ApplyLogEntry.class);

    private final CatalogReplica replica;

    public ApplyLogEntry(CatalogReplica replica) {
        this.replica = replica;
    }

    public void execute(CatalogLogEntry entry) throws AppException {
        switch (entry) {
            case CatalogLogEntry.Snapshot(long seq, var sections) -> {
                Catalog catalog = Catalog.load(seq, sections);
                log.info("réplica recargada: seq={} grupos={}", seq, catalog.size());
                replica.replace(catalog);
            }
            // Un hueco aquí indica un historial inconsistente; la réplica queda
            // desactualizada hasta la siguiente carga completa.
            case CatalogLogEntry.Change(var event) -> replica.apply(event);
            case CatalogLogEntry.SyncLost() -> replica.markStale();
        }
    }
}
