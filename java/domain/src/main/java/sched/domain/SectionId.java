package sched.domain;

import java.util.Objects;

public record SectionId(String value) implements Comparable<SectionId> {
    public SectionId {
        Objects.requireNonNull(value);
    }

    @Override
    public int compareTo(SectionId other) {
        return value.compareTo(other.value);
    }

    @Override
    public String toString() {
        return value;
    }
}
