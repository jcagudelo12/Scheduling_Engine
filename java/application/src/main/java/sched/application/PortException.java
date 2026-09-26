package sched.application;

/** Error que devuelve un adaptador al implementar un puerto. */
public final class PortException extends Exception {

    public enum Kind { INVALID_DATA, UNAVAILABLE }

    private final Kind kind;

    public PortException(Kind kind, String message, Throwable cause) {
        super(message, cause);
        this.kind = kind;
    }

    public PortException(Kind kind, String message) {
        this(kind, message, null);
    }

    public Kind kind() {
        return kind;
    }
}
