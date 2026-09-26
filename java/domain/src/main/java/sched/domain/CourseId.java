package sched.domain;

import java.util.Objects;

public record CourseId(String value) implements Comparable<CourseId> {
    public CourseId {
        Objects.requireNonNull(value);
    }

    @Override
    public int compareTo(CourseId other) {
        return value.compareTo(other.value);
    }

    @Override
    public String toString() {
        return value;
    }
}
