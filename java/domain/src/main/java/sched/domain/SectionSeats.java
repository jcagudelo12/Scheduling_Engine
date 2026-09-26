package sched.domain;

/** Grupo junto con sus cupos disponibles, como llega en una carga completa. */
public record SectionSeats(Section section, int available) {
}
