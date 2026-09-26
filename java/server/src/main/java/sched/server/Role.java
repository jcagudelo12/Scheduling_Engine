package sched.server;

/** Qué hace esta instancia (ADR 0005). Todas mantienen su réplica leyendo el historial. */
public enum Role {
    /** Recibe los datos de la institución por gRPC y los escribe en el historial. Una sola. */
    INGEST,
    /** Atiende a los estudiantes. Se puede escalar horizontalmente. */
    QUERY,
    /** Ambos: para desarrollo o despliegues de una sola instancia. */
    ALL;

    public static Role parse(String value) {
        return switch (value) {
            case "ingest" -> INGEST;
            case "query" -> QUERY;
            case "all" -> ALL;
            default -> throw new IllegalArgumentException(
                    "SCHED_ROLE inválido: " + value + " (use ingest, query o all)");
        };
    }

    public boolean ingests() {
        return this == INGEST || this == ALL;
    }

    public boolean servesQueries() {
        return this == QUERY || this == ALL;
    }
}
