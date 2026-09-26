package sched.application;

import java.util.concurrent.locks.ReentrantReadWriteLock;
import sched.domain.ApplyOutcome;
import sched.domain.Catalog;
import sched.domain.CatalogEvent;
import sched.domain.SequenceGapException;

/**
 * Réplica en memoria del catálogo institucional (ADR 0003), compartida entre casos de uso.
 *
 * <p>Queda desactualizada ({@code stale}) cuando se detecta un hueco de secuencia o se pierde
 * la conexión con la institución. Vuelve a estar al día con una carga completa o cuando se
 * aplica el siguiente evento sin huecos (tras un hueco eso solo ocurre después de recargar).
 */
public final class CatalogReplica {

    /** Lectura del catálogo junto con su marca de desactualización. */
    @FunctionalInterface
    public interface Reader<T> {
        T read(Catalog catalog, boolean stale);
    }

    private final ReentrantReadWriteLock lock = new ReentrantReadWriteLock();
    private Catalog catalog;
    private boolean stale;

    public <T> T read(Reader<T> reader) throws AppException.CatalogUnavailable {
        lock.readLock().lock();
        try {
            if (catalog == null) {
                throw new AppException.CatalogUnavailable();
            }
            return reader.read(catalog, stale);
        } finally {
            lock.readLock().unlock();
        }
    }

    public boolean isStale() {
        lock.readLock().lock();
        try {
            return stale;
        } finally {
            lock.readLock().unlock();
        }
    }

    void replace(Catalog replacement) {
        lock.writeLock().lock();
        try {
            catalog = replacement;
            stale = false;
        } finally {
            lock.writeLock().unlock();
        }
    }

    ApplyOutcome apply(CatalogEvent event) throws AppException {
        lock.writeLock().lock();
        try {
            if (catalog == null) {
                throw new AppException.CatalogUnavailable();
            }
            ApplyOutcome outcome = catalog.apply(event);
            if (outcome == ApplyOutcome.APPLIED) {
                stale = false;
            }
            return outcome;
        } catch (SequenceGapException gap) {
            stale = true;
            throw new AppException.SequenceGap(gap);
        } finally {
            lock.writeLock().unlock();
        }
    }

    void markStale() {
        lock.writeLock().lock();
        try {
            stale = true;
        } finally {
            lock.writeLock().unlock();
        }
    }
}
