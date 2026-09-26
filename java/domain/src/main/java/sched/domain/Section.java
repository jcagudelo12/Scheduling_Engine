package sched.domain;

import java.util.List;

/** Un grupo concreto de un curso, con sus franjas y su cupo total. */
public record Section(SectionId id, CourseId course, List<TimeSlot> slots, int capacity) {

    public Section {
        slots = List.copyOf(slots);
    }

    public boolean clashesWith(Section other) {
        for (TimeSlot a : slots) {
            for (TimeSlot b : other.slots) {
                if (a.overlaps(b)) {
                    return true;
                }
            }
        }
        return false;
    }
}
