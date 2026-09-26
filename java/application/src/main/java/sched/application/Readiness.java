package sched.application;

import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Indica si la réplica local ya se puso al día con el historial compartido.
 *
 * <p>El balanceador no debe enviar tráfico a una instancia que aún no está lista.
 */
public final class Readiness {

    private final AtomicBoolean ready = new AtomicBoolean();

    public boolean isReady() {
        return ready.get();
    }

    public void markReady() {
        ready.set(true);
    }
}
