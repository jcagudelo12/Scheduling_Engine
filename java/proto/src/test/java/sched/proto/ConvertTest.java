package sched.proto;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import org.junit.jupiter.api.Test;
import sched.domain.CatalogChange;
import sched.domain.CatalogEvent;
import sched.domain.CourseId;
import sched.domain.Section;
import sched.domain.SectionId;
import sched.domain.SectionSeats;
import sched.domain.TimeSlot;
import sched.domain.Weekday;

class ConvertTest {

    @Test
    void sectionRoundTrip() throws Exception {
        Section original = new Section(new SectionId("MAT-01"), new CourseId("MAT"),
                List.of(TimeSlot.of(Weekday.FRIDAY, 420, 540).orElseThrow()), 30);
        assertEquals(new SectionSeats(original, 7), Convert.section(Convert.sectionToPb(original, 7)));
    }

    @Test
    void eventRoundTrip() throws Exception {
        CatalogEvent original = new CatalogEvent(42, new CatalogChange.SeatsChanged(new SectionId("MAT-01"), 3));
        assertEquals(original, Convert.catalogEvent(Convert.catalogEventToPb(original)));
    }
}
