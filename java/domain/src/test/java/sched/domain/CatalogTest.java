package sched.domain;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;

class CatalogTest {

    private static Section section(String id, String course) {
        TimeSlot slot = TimeSlot.of(Weekday.MONDAY, 420, 540).orElseThrow();
        return new Section(new SectionId(id), new CourseId(course), List.of(slot), 30);
    }

    private static CatalogEvent seats(long seq, String id, int available) {
        return new CatalogEvent(seq, new CatalogChange.SeatsChanged(new SectionId(id), available));
    }

    private static List<String> openIds(Catalog catalog, String... courses) {
        List<CourseId> ids = Arrays.stream(courses).map(CourseId::new).toList();
        return catalog.openSectionsOf(ids).stream().map(s -> s.id().value()).toList();
    }

    @Test
    void appliesEventsInOrder() throws Exception {
        Catalog catalog = Catalog.load(10, List.of(new SectionSeats(section("MAT-01", "MAT"), 5)));

        assertEquals(ApplyOutcome.APPLIED, catalog.apply(seats(11, "MAT-01", 0)));
        assertEquals(11, catalog.seq());
        assertEquals(0, catalog.available(new SectionId("MAT-01")).orElseThrow());
    }

    @Test
    void repeatedEventsAreIgnored() throws Exception {
        Catalog catalog = Catalog.load(10, List.of(new SectionSeats(section("MAT-01", "MAT"), 5)));

        assertEquals(ApplyOutcome.DUPLICATE, catalog.apply(seats(10, "MAT-01", 0)));
        assertEquals(5, catalog.available(new SectionId("MAT-01")).orElseThrow());
    }

    @Test
    void gapsAreRejectedWithoutChanges() {
        Catalog catalog = Catalog.load(10, List.of(new SectionSeats(section("MAT-01", "MAT"), 5)));

        SequenceGapException gap = assertThrows(SequenceGapException.class,
                () -> catalog.apply(seats(12, "MAT-01", 0)));
        assertEquals(11, gap.expected());
        assertEquals(12, gap.received());
        assertEquals(10, catalog.seq());
    }

    @Test
    void onlySectionsOfRequestedCoursesAreReturned() {
        Catalog catalog = Catalog.load(1, List.of(
                new SectionSeats(section("MAT-02", "MAT"), 5),
                new SectionSeats(section("ART-01", "ART"), 5),
                new SectionSeats(section("MAT-01", "MAT"), 5),
                new SectionSeats(section("FIS-01", "FIS"), 5)));

        assertEquals(List.of("FIS-01", "MAT-01", "MAT-02"), openIds(catalog, "FIS", "MAT"));
    }

    @Test
    void repeatedCoursesDoNotRepeatSections() {
        Catalog catalog = Catalog.load(1, List.of(new SectionSeats(section("MAT-01", "MAT"), 5)));
        assertEquals(List.of("MAT-01"), openIds(catalog, "MAT", "MAT"));
    }

    @Test
    void indexFollowsUpsertsAndRemovals() throws Exception {
        Catalog catalog = Catalog.load(1, List.of(new SectionSeats(section("X-01", "MAT"), 5)));

        // El grupo se reasigna a otro curso: debe salir del índice de MAT.
        catalog.apply(new CatalogEvent(2, new CatalogChange.SectionUpserted(section("X-01", "FIS"), 5)));
        assertTrue(openIds(catalog, "MAT").isEmpty());
        assertEquals(List.of("X-01"), openIds(catalog, "FIS"));

        catalog.apply(new CatalogEvent(3, new CatalogChange.SectionRemoved(new SectionId("X-01"))));
        assertTrue(openIds(catalog, "FIS").isEmpty());
        assertEquals(0, catalog.indexedCourses());
    }

    @Test
    void fullSectionsAreNotOffered() {
        Catalog catalog = Catalog.load(1, List.of(
                new SectionSeats(section("MAT-01", "MAT"), 0),
                new SectionSeats(section("MAT-02", "MAT"), 3)));

        assertEquals(List.of("MAT-02"), openIds(catalog, "MAT"));
    }
}
