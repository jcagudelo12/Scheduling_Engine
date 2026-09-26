package sched.domain;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.OptionalInt;
import java.util.Set;
import java.util.TreeSet;

/**
 * Réplica del catálogo que envía la institución (ADR 0003): todos los grupos del periodo
 * con sus cupos disponibles.
 *
 * <p>No es segura para hilos; {@code CatalogReplica} controla el acceso concurrente.
 */
public final class Catalog {

    private record Entry(Section section, int available) {
    }

    private long seq;
    private final Map<SectionId, Entry> entries = new HashMap<>();
    /**
     * Índice curso → grupos, para no recorrer todo el catálogo en cada consulta.
     * Los grupos van ordenados por id para que los resultados sean reproducibles.
     */
    private final Map<CourseId, TreeSet<SectionId>> byCourse = new HashMap<>();

    private Catalog(long seq) {
        this.seq = seq;
    }

    /** Construye el catálogo a partir de una carga completa. */
    public static Catalog load(long seq, Iterable<SectionSeats> sections) {
        Catalog catalog = new Catalog(seq);
        for (SectionSeats s : sections) {
            catalog.insert(s.section(), s.available());
        }
        return catalog;
    }

    public long seq() {
        return seq;
    }

    public int size() {
        return entries.size();
    }

    public OptionalInt available(SectionId section) {
        Entry entry = entries.get(section);
        return entry == null ? OptionalInt.empty() : OptionalInt.of(entry.available());
    }

    /**
     * Grupos de los cursos indicados que aún tienen cupo, curso por curso.
     *
     * <p>El costo depende de cuántos grupos tienen esos cursos, no del tamaño del catálogo.
     */
    public List<Section> openSectionsOf(List<CourseId> courses) {
        List<Section> open = new ArrayList<>();
        Set<CourseId> seen = new HashSet<>();
        for (CourseId course : courses) {
            // Un curso repetido en la lista no debe repetir sus grupos.
            if (!seen.add(course)) {
                continue;
            }
            Set<SectionId> ids = byCourse.get(course);
            if (ids == null) {
                continue;
            }
            for (SectionId id : ids) {
                Entry entry = entries.get(id);
                if (entry != null && entry.available() > 0) {
                    open.add(entry.section());
                }
            }
        }
        return open;
    }

    /** Aplica un cambio respetando el orden estricto de {@code seq}. */
    public ApplyOutcome apply(CatalogEvent event) throws SequenceGapException {
        long expected = seq + 1;
        if (event.seq() < expected) {
            return ApplyOutcome.DUPLICATE;
        }
        if (event.seq() > expected) {
            throw new SequenceGapException(expected, event.seq());
        }

        switch (event.change()) {
            case CatalogChange.SeatsChanged(SectionId id, int available) -> {
                // Un cambio de cupo sobre un grupo desconocido no tiene datos para crearlo.
                Entry entry = entries.get(id);
                if (entry != null) {
                    entries.put(id, new Entry(entry.section(), available));
                }
            }
            case CatalogChange.SectionUpserted(Section section, int available) -> insert(section, available);
            case CatalogChange.SectionRemoved(SectionId id) -> remove(id);
        }
        seq = event.seq();
        return ApplyOutcome.APPLIED;
    }

    /** Inserta o reemplaza un grupo manteniendo el índice por curso al día. */
    private void insert(Section section, int available) {
        Entry previous = entries.put(section.id(), new Entry(section, available));
        // Si el grupo cambió de curso, sale del índice del curso anterior.
        if (previous != null && !previous.section().course().equals(section.course())) {
            unindex(previous.section().course(), section.id());
        }
        byCourse.computeIfAbsent(section.course(), c -> new TreeSet<>()).add(section.id());
    }

    private void remove(SectionId id) {
        Entry entry = entries.remove(id);
        if (entry != null) {
            unindex(entry.section().course(), id);
        }
    }

    private void unindex(CourseId course, SectionId id) {
        TreeSet<SectionId> ids = byCourse.get(course);
        if (ids != null) {
            ids.remove(id);
            if (ids.isEmpty()) {
                byCourse.remove(course);
            }
        }
    }

    /** Solo para pruebas: cursos presentes en el índice. */
    int indexedCourses() {
        return byCourse.size();
    }
}
