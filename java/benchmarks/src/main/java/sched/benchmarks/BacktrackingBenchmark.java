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
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;
import org.openjdk.jmh.annotations.Warmup;
import sched.domain.CourseId;
import sched.domain.Schedule;
import sched.domain.Section;
import sched.domain.SectionId;
import sched.domain.TimeSlot;
import sched.domain.Weekday;
import sched.solver.BacktrackingSolver;
import sched.solver.Problem;

/**
 * Backtracking sobre el mismo problema sintético que {@code solver/benches/solver.rs}:
 * 6 cursos con 5 grupos cada uno, hasta 500 resultados.
 */
@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Warmup(iterations = 3, time = 2)
@Measurement(iterations = 5, time = 2)
@Fork(1)
public class BacktrackingBenchmark {

    private static final Weekday[] DAYS = {
        Weekday.MONDAY, Weekday.TUESDAY, Weekday.WEDNESDAY, Weekday.THURSDAY, Weekday.FRIDAY, Weekday.SATURDAY,
    };

    private final BacktrackingSolver solver = new BacktrackingSolver();
    private Problem problem;

    @Setup
    public void setup() {
        int courses = 6;
        int sectionsPerCourse = 5;
        List<Section> candidates = new ArrayList<>();
        List<CourseId> courseIds = new ArrayList<>();
        for (int c = 0; c < courses; c++) {
            courseIds.add(new CourseId("C" + c));
            for (int s = 0; s < sectionsPerCourse; s++) {
                Weekday day = DAYS[(c + s) % DAYS.length];
                int start = 360 + ((c * 7 + s * 3) % 7) * 120;
                candidates.add(new Section(
                        new SectionId("C" + c + "-" + s),
                        new CourseId("C" + c),
                        List.of(TimeSlot.of(day, start, start + 120).orElseThrow()),
                        30));
            }
        }
        problem = new Problem(courseIds, candidates, 500);
    }

    @Benchmark
    public List<Schedule> backtracking6x5() {
        return solver.solve(problem);
    }
}
