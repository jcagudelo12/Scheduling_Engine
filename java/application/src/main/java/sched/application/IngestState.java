package sched.application;

import sched.domain.SequenceGapException;

/**
 * Estado de la instancia de ingesta: último {@code seq} aceptado de la institución.
 *
 * <p>Es la única fuente de validación de orden (un solo escritor, ADR 0005). Al arrancar
 * se inicializa desde la réplica local, que ya se puso al día con el historial.
 */
public final class IngestState {

    /** Resultado de validar el {@code seq} de un cambio antes de escribirlo en el historial. */
    enum Admission {
        /** Es el siguiente esperado; queda reservado y debe escribirse. */
        NEXT,
        /** Ya se había aceptado; no se escribe de nuevo. */
        DUPLICATE
    }

    private final CatalogReplica replica;
    private Long lastSeq;
    private boolean needsResync;

    public IngestState(CatalogReplica replica) {
        this.replica = replica;
    }

    synchronized Admission admit(long seq) throws AppException {
        if (lastSeq == null) {
            lastSeq = replica.read((catalog, stale) -> catalog.seq());
        }
        long expected = lastSeq + 1;

        if (seq < expected) {
            return Admission.DUPLICATE;
        }
        if (needsResync || seq > expected) {
            needsResync = true;
            throw new AppException.SequenceGap(new SequenceGapException(expected, seq));
        }
        lastSeq = seq;
        return Admission.NEXT;
    }

    /** Tras una carga completa escrita en el historial. */
    synchronized void reset(long seq) {
        lastSeq = seq;
        needsResync = false;
    }

    /**
     * No se pudo escribir un cambio ya admitido: solo una carga completa garantiza que el
     * historial y la institución vuelvan a coincidir.
     */
    synchronized void requireResync() {
        needsResync = true;
    }
}
