package sched.adapters.inmemory;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import sched.application.AppException;
import sched.application.ApplyLogEntry;
import sched.application.CatalogLogEntry;
import sched.application.CatalogReplica;
import sched.application.Combinations;
import sched.application.GenerateCombinations;
import sched.application.IngestCatalog;
import sched.application.IngestCatalogEvent;
import sched.application.IngestState;
import sched.domain.ApplyOutcome;
import sched.domain.CatalogChange;
import sched.domain.CatalogEvent;
import sched.domain.CourseId;
import sched.domain.Section;
import sched.domain.SectionId;
import sched.domain.SectionSeats;
import sched.domain.StudentContext;
import sched.domain.StudentId;
import sched.domain.TimeSlot;
import sched.domain.Weekday;
import sched.solver.BacktrackingSolver;

/** Pruebas de los casos de uso con el historial en memoria. */
class UseCasesTest {

    private InMemoryCatalogLog log;
    private IngestCatalog load;
    private IngestCatalogEvent apply;
    private GenerateCombinations generate;

    private static Section section(String id, String course, Weekday day) {
        return new Section(new SectionId(id), new CourseId(course),
                List.of(TimeSlot.of(day, 420, 540).orElseThrow()), 30);
    }

    private static List<SectionSeats> catalog() {
        return List.of(
                new SectionSeats(section("MAT-01", "MAT", Weekday.MONDAY), 5),
                new SectionSeats(section("MAT-02", "MAT", Weekday.WEDNESDAY), 5),
                new SectionSeats(section("FIS-01", "FIS", Weekday.TUESDAY), 5));
    }

    private static final StudentContext STUDENT =
            new StudentContext(new StudentId("s1"), List.of(new CourseId("MAT"), new CourseId("FIS")));

    private static CatalogEvent seats(long seq, String id, int available) {
        return new CatalogEvent(seq, new CatalogChange.SeatsChanged(new SectionId(id), available));
    }

    @BeforeEach
    void engine() {
        CatalogReplica replica = new CatalogReplica();
        log = new InMemoryCatalogLog(new ApplyLogEntry(replica));
        IngestState ingest = new IngestState(replica);
        load = new IngestCatalog(ingest, log);
        apply = new IngestCatalogEvent(ingest, log);
        generate = new GenerateCombinations(new BacktrackingSolver(), replica, 50);
    }

    @Test
    void fullSectionIsExcludedAfterSeatChange() throws Exception {
        load.execute(100, catalog());

        Combinations before = generate.execute(STUDENT);
        assertEquals(2, before.schedules().size());
        assertEquals(100, before.catalogSeq());

        apply.execute(seats(101, "MAT-01", 0));

        Combinations after = generate.execute(STUDENT);
        assertEquals(1, after.schedules().size());
        assertEquals(101, after.catalogSeq());
        assertFalse(after.stale());
    }

    @Test
    void duplicatesAreNotWrittenTwice() throws Exception {
        load.execute(100, catalog());

        apply.execute(seats(101, "MAT-01", 0));
        assertEquals(ApplyOutcome.DUPLICATE, apply.execute(seats(101, "MAT-01", 0)));
        assertEquals(2, log.entries().size());
    }

    @Test
    void gapMarksReplicasStaleUntilReload() throws Exception {
        load.execute(100, catalog());

        assertThrows(AppException.SequenceGap.class, () -> apply.execute(seats(105, "MAT-01", 0)));
        assertInstanceOf(CatalogLogEntry.SyncLost.class, log.entries().getLast());
        assertTrue(generate.execute(STUDENT).stale());

        // Tras un hueco se rechaza todo, incluso el seq "correcto", hasta la recarga.
        assertThrows(AppException.SequenceGap.class, () -> apply.execute(seats(101, "MAT-01", 0)));

        load.execute(105, catalog());
        assertFalse(generate.execute(STUDENT).stale());
        apply.execute(seats(106, "MAT-01", 0));
    }

    @Test
    void reconnectingWithoutGapsClearsStale() throws Exception {
        load.execute(100, catalog());

        apply.connectionLost();
        assertTrue(generate.execute(STUDENT).stale());

        apply.execute(seats(101, "MAT-01", 3));
        assertFalse(generate.execute(STUDENT).stale());
    }

    @Test
    void changesBeforeFirstLoadAreRefused() {
        assertThrows(AppException.CatalogUnavailable.class, () -> apply.execute(seats(1, "MAT-01", 0)));
        assertThrows(AppException.CatalogUnavailable.class, () -> generate.execute(STUDENT));
    }
}
