package sched.domain;

import java.util.Optional;

/** Franja semanal {@code [start, end)} expresada en minutos desde la medianoche. */
public record TimeSlot(Weekday day, int start, int end) {

    /** Devuelve vacío si la franja está vacía o se sale del día. */
    public static Optional<TimeSlot> of(Weekday day, int start, int end) {
        return start >= 0 && start < end && end <= 24 * 60
                ? Optional.of(new TimeSlot(day, start, end))
                : Optional.empty();
    }

    public boolean overlaps(TimeSlot other) {
        return day == other.day && start < other.end && other.start < end;
    }
}
