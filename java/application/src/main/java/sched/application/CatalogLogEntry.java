package sched.application;

import java.util.List;
import sched.domain.CatalogEvent;
import sched.domain.SectionSeats;

/**
 * Entrada del historial compartido del catálogo (ADR 0005).
 *
 * <p>La instancia de ingesta las escribe; todas las instancias las leen en el mismo orden
 * y las aplican a su réplica local.
 */
public sealed interface CatalogLogEntry {

    /** Carga completa: reemplaza el catálogo. */
    record Snapshot(long seq, List<SectionSeats> sections) implements CatalogLogEntry {
        public Snapshot {
            sections = List.copyOf(sections);
        }
    }

    /** Cambio incremental ya validado contra el {@code seq} anterior. */
    record Change(CatalogEvent event) implements CatalogLogEntry {
    }

    /** La ingesta perdió la sincronía con la institución: las réplicas quedan desactualizadas. */
    record SyncLost() implements CatalogLogEntry {
    }
}
