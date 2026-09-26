package sched.domain;

import java.util.List;

/** Datos del estudiante que entrega la institución en cada solicitud (ADR 0004). */
public record StudentContext(StudentId id, List<CourseId> eligibleCourses) {

    public StudentContext {
        eligibleCourses = List.copyOf(eligibleCourses);
    }
}
