package sched.adapters.http;

import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;
import sched.application.Readiness;

/**
 * Sondas para el orquestador y el balanceador.
 *
 * <ul>
 *   <li>{@code GET /health}: el proceso está vivo.
 *   <li>{@code GET /ready}: la réplica ya se puso al día con el historial (ADR 0005).
 * </ul>
 */
@RestController
public class HealthController {

    private final Readiness readiness;

    public HealthController(Readiness readiness) {
        this.readiness = readiness;
    }

    @GetMapping("/health")
    public ResponseEntity<Void> health() {
        return ResponseEntity.ok().build();
    }

    @GetMapping("/ready")
    public ResponseEntity<Void> ready() {
        return ResponseEntity.status(readiness.isReady() ? HttpStatus.OK : HttpStatus.SERVICE_UNAVAILABLE).build();
    }
}
