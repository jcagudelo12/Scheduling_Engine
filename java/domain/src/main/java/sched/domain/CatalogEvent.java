package sched.domain;

/** Cambio con su número de secuencia, asignado por la institución. */
public record CatalogEvent(long seq, CatalogChange change) {
}
