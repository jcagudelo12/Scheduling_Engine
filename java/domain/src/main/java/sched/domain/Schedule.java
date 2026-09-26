package sched.domain;

import java.util.List;

/** Una combinación válida: un grupo por curso, sin cruces de horario. */
public record Schedule(List<Section> sections) {

    public Schedule {
        sections = List.copyOf(sections);
    }

    public boolean hasClashes() {
        for (int i = 0; i < sections.size(); i++) {
            for (int j = i + 1; j < sections.size(); j++) {
                if (sections.get(i).clashesWith(sections.get(j))) {
                    return true;
                }
            }
        }
        return false;
    }
}
