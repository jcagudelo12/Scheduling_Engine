package sched.solver;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import sched.domain.CourseId;
import sched.domain.Schedule;
import sched.domain.Section;
import sched.domain.SectionId;
import sched.domain.TimeSlot;
import sched.domain.Weekday;

class BacktrackingSolverTest {

    private static Section section(String id, String course, Weekday day, int start, int end) {
        return new Section(new SectionId(id), new CourseId(course),
                List.of(TimeSlot.of(day, start, end).orElseThrow()), 30);
    }

    private static Problem problem(List<Section> candidates, int maxResults, String... courses) {
        return new Problem(Arrays.stream(courses).map(CourseId::new).toList(), candidates, maxResults);
    }

    @Test
    void skipsCombinationsWithClashes() {
        Problem p = problem(List.of(
                section("MAT-01", "MAT", Weekday.MONDAY, 420, 540),
                section("MAT-02", "MAT", Weekday.TUESDAY, 420, 540),
                section("FIS-01", "FIS", Weekday.MONDAY, 480, 600)), 100, "MAT", "FIS");

        List<Schedule> schedules = new BacktrackingSolver().solve(p);

        assertEquals(1, schedules.size());
        assertTrue(schedules.stream().noneMatch(Schedule::hasClashes));
        assertEquals(new SectionId("MAT-02"), schedules.getFirst().sections().getFirst().id());
    }

    @Test
    void courseWithoutSectionsYieldsNothing() {
        Problem p = problem(List.of(section("MAT-01", "MAT", Weekday.MONDAY, 420, 540)), 100, "MAT", "FIS");
        assertTrue(new BacktrackingSolver().solve(p).isEmpty());
    }

    @Test
    void respectsMaxResults() {
        Problem p = problem(List.of(
                section("MAT-01", "MAT", Weekday.MONDAY, 420, 540),
                section("MAT-02", "MAT", Weekday.TUESDAY, 420, 540)), 1, "MAT");
        assertEquals(1, new BacktrackingSolver().solve(p).size());
    }
}
