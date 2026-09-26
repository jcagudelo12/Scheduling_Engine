package sched.application;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import sched.application.ports.CatalogLog;
import sched.domain.ApplyOutcome;
import sched.domain.CatalogEvent;

/** Caso de uso: la institución notifica un cambio incremental del catálogo. */
public final class IngestCatalogEvent {

    private static final Logger log = LoggerFactory.getLogger(IngestCatalogEvent.class);

    private final IngestState ingest;
    private final CatalogLog catalogLog;

    public IngestCatalogEvent(IngestState ingest, CatalogLog catalogLog) {
        this.ingest = ingest;
        this.catalogLog = catalogLog;
    }

    /**
     * Valida el orden y escribe el cambio en el historial.
     *
     * @throws AppException.SequenceGap se perdieron eventos; la institución debe enviar una carga
     *     completa y todas las réplicas quedan desactualizadas mientras tanto
     * @throws AppException.CatalogUnavailable aún no hay carga completa
     */
    public ApplyOutcome execute(CatalogEvent event) throws AppException {
        IngestState.Admission admission;
        try {
            admission = ingest.admit(event.seq());
        } catch (AppException.SequenceGap gap) {
            log.warn("hueco en los cambios de la institución: {}", gap.getMessage());
            notifySyncLost();
            throw gap;
        }
        if (admission == IngestState.Admission.DUPLICATE) {
            return ApplyOutcome.DUPLICATE;
        }

        try {
            catalogLog.append(new CatalogLogEntry.Change(event));
        } catch (PortException e) {
            ingest.requireResync();
            throw new AppException.PortFailure(e);
        }
        return ApplyOutcome.APPLIED;
    }

    /** Se cerró el canal de cambios: ya no hay garantía de estar al día. */
    public void connectionLost() {
        log.warn("se perdió la conexión de cambios con la institución");
        notifySyncLost();
    }

    private void notifySyncLost() {
        try {
            catalogLog.append(new CatalogLogEntry.SyncLost());
        } catch (PortException e) {
            log.error("no se pudo avisar la pérdida de sincronía: {}", e.getMessage());
        }
    }
}
