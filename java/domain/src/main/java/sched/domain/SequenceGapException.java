package sched.domain;

/** Se perdió al menos un evento: hace falta una carga completa. */
public final class SequenceGapException extends Exception {

    private final long expected;
    private final long received;

    public SequenceGapException(long expected, long received) {
        super("se esperaba seq " + expected + " y llegó " + received);
        this.expected = expected;
        this.received = received;
    }

    public long expected() {
        return expected;
    }

    public long received() {
        return received;
    }
}
