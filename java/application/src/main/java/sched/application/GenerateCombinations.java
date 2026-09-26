package sched.application;

import java.util.List;
import sched.domain.Section;
import sched.domain.StudentContext;
import sched.solver.Problem;
import sched.solver.Solver;

/**
 * Caso de uso: el estudiante pide combinaciones de horario para sus cursos habilitados.
 *
 * <p>Solo lee la réplica en memoria y ejecuta el solver.
 */
public final class GenerateCombinations {

    private record Snapshot(long seq, boolean stale, List<Section> candidates) {
    }

    private final Solver solver;
    private final CatalogReplica replica;
    private final int maxResults;

    public GenerateCombinations(Solver solver, CatalogReplica replica, int maxResults) {
        this.solver = solver;
        this.replica = replica;
        this.maxResults = maxResults;
    }

    public Combinations execute(StudentContext student) throws AppException {
        // Llegan curso por curso y ordenados por id: resultados reproducibles.
        Snapshot snapshot = replica.read((catalog, stale) ->
                new Snapshot(catalog.seq(), stale, catalog.openSectionsOf(student.eligibleCourses())));

        Problem problem = new Problem(student.eligibleCourses(), snapshot.candidates(), maxResults);
        return new Combinations(snapshot.seq(), snapshot.stale(), solver.solve(problem));
    }
}
