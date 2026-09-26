package sched.solver;

import java.util.ArrayList;
import java.util.List;
import sched.domain.CourseId;
import sched.domain.Schedule;
import sched.domain.Section;

/**
 * Enumeración exhaustiva con poda por cruce de horario.
 *
 * <p>Sirve como línea base contra la cual comparar estrategias más elaboradas.
 */
public final class BacktrackingSolver implements Solver {

    @Override
    public List<Schedule> solve(Problem problem) {
        List<List<Section>> options = new ArrayList<>(problem.courses().size());
        for (CourseId course : problem.courses()) {
            List<Section> sections = new ArrayList<>();
            for (Section section : problem.candidates()) {
                if (section.course().equals(course)) {
                    sections.add(section);
                }
            }
            // Un curso sin grupos disponibles hace imposible cualquier combinación completa.
            if (sections.isEmpty()) {
                return List.of();
            }
            options.add(sections);
        }

        List<Schedule> results = new ArrayList<>();
        search(options, 0, new ArrayList<>(options.size()), results, problem.maxResults());
        return results;
    }

    private static void search(
            List<List<Section>> options,
            int depth,
            List<Section> current,
            List<Schedule> results,
            int maxResults) {
        if (results.size() >= maxResults) {
            return;
        }
        if (depth == options.size()) {
            results.add(new Schedule(current));
            return;
        }
        for (Section section : options.get(depth)) {
            if (clashesWithAny(current, section)) {
                continue;
            }
            current.add(section);
            search(options, depth + 1, current, results, maxResults);
            current.removeLast();
        }
    }

    private static boolean clashesWithAny(List<Section> chosen, Section section) {
        for (Section other : chosen) {
            if (other.clashesWith(section)) {
                return true;
            }
        }
        return false;
    }
}
