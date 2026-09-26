package sched.domain;

import java.util.Objects;

public record StudentId(String value) implements Comparable<StudentId> {
    public StudentId {
        Objects.requireNonNull(value);
    }

    @Override
    public int compareTo(StudentId other) {
        return value.compareTo(other.value);
    }

    @Override
    public String toString() {
        return value;
    }
}
