package sched.benchmarks;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.openjdk.jmh.annotations.Benchmark;
import org.openjdk.jmh.annotations.BenchmarkMode;
import org.openjdk.jmh.annotations.Fork;
import org.openjdk.jmh.annotations.Measurement;
import org.openjdk.jmh.annotations.Mode;
import org.openjdk.jmh.annotations.OutputTimeUnit;
import org.openjdk.jmh.annotations.Param;
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;
import org.openjdk.jmh.annotations.Warmup;
import sched.domain.Catalog;
import sched.domain.CourseId;
import sched.domain.Section;
import sched.domain.SectionId;
import sched.domain.SectionSeats;
import sched.domain.TimeSlot;
import sched.domain.Weekday;

/**
 * Costo de encontrar los grupos de los cursos de un estudiante dentro del catálogo completo.
 * Equivalente a {@code simulation/benches/catalog_lookup.rs}.
 */
@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.NANOSECONDS)
@Warmup(iterations = 3, time = 2)
@Measurement(iterations = 5, time = 2)
@Fork(1)
public class CatalogLookupBenchmark {

    private static final int SECTIONS_PER_COURSE = 5;
    private static final int STUDENT_COURSES = 6;

    /** Cursos del catálogo: 200, 1.000 y 5.000 cursos → 1.000, 5.000 y 25.000 grupos. */
    @Param({"200", "1000", "5000"})
    public int courses;

    private Catalog catalog;
    private List<CourseId> student;

    @Setup
    public void setup() {
        TimeSlot slot = TimeSlot.of(Weekday.MONDAY, 420, 540).orElseThrow();
        List<SectionSeats> sections = new ArrayList<>();
        for (int c = 0; c < courses; c++) {
            for (int s = 0; s < SECTIONS_PER_COURSE; s++) {
                sections.add(new SectionSeats(new Section(
                        new SectionId("C%04d-%02d".formatted(c, s)),
                        new CourseId("C%04d".formatted(c)),
                        List.of(slot), 30), 10));
            }
        }
        catalog = Catalog.load(1, sections);
        student = new ArrayList<>();
        for (int i = 0; i < STUDENT_COURSES; i++) {
            student.add(new CourseId("C%04d".formatted(i * courses / STUDENT_COURSES)));
        }
    }

    @Benchmark
    public int openSectionsOf() {
        return catalog.openSectionsOf(student).size();
    }
}
