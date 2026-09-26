package sched.application;

import java.util.List;
import sched.domain.Schedule;

/**
 * Resultado junto con el estado del catálogo con que se calculó.
 *
 * @param catalogSeq {@code seq} del catálogo usado, definido por la institución
 * @param stale {@code true} si la réplica podía no estar al día (ADR 0003)
 */
public record Combinations(long catalogSeq, boolean stale, List<Schedule> schedules) {
}
