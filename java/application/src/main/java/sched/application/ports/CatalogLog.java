package sched.application.ports;

import sched.application.CatalogLogEntry;
import sched.application.PortException;

/**
 * Historial compartido del catálogo. Implementaciones: {@code nats-bus} (JetStream) e
 * {@code in-memory} (una sola instancia y pruebas).
 *
 * <p>El motor no consulta a la institución (ADR 0002): los datos entran por los casos de
 * uso de ingesta y se escriben aquí (ADR 0005).
 */
public interface CatalogLog {

    /** Escribe una entrada al final del historial. Solo la instancia de ingesta escribe. */
    void append(CatalogLogEntry entry) throws PortException;
}
