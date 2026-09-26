package sched.solver;

import java.util.List;
import sched.domain.CourseId;
import sched.domain.Section;

/** Entrada del solver: cursos a cubrir y grupos candidatos (ya filtrados por cupo). */
public record Problem(List<CourseId> courses, List<Section> candidates, int maxResults) {

    public Problem {
        courses = List.copyOf(courses);
        candidates = List.copyOf(candidates);
    }
}
