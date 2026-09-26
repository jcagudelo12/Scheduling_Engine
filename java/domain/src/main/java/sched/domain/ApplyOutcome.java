package sched.domain;

public enum ApplyOutcome {
    APPLIED,
    /** El evento ya estaba aplicado ({@code seq} repetido o antiguo); no cambia nada. */
    DUPLICATE
}
