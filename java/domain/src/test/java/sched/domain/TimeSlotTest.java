package sched.domain;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class TimeSlotTest {

    private static TimeSlot slot(Weekday day, int start, int end) {
        return TimeSlot.of(day, start, end).orElseThrow();
    }

    @Test
    void contiguousSlotsDoNotOverlap() {
        assertFalse(slot(Weekday.MONDAY, 420, 540).overlaps(slot(Weekday.MONDAY, 540, 660)));
    }

    @Test
    void sameHoursOnDifferentDaysDoNotOverlap() {
        assertFalse(slot(Weekday.MONDAY, 420, 540).overlaps(slot(Weekday.TUESDAY, 420, 540)));
    }

    @Test
    void partialOverlapIsDetected() {
        assertTrue(slot(Weekday.FRIDAY, 420, 540).overlaps(slot(Weekday.FRIDAY, 500, 600)));
    }

    @Test
    void invalidSlotsAreRejected() {
        assertTrue(TimeSlot.of(Weekday.MONDAY, 600, 600).isEmpty());
        assertTrue(TimeSlot.of(Weekday.MONDAY, 600, 1500).isEmpty());
    }
}
