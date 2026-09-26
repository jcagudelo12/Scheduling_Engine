package sched.solver;

import java.util.List;
import sched.domain.Schedule;

/** Estrategia para generar combinaciones de horario. Síncrona y pura. */
public interface Solver {

    List<Schedule> solve(Problem problem);
}
