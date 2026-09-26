package sched.domain;

/** Cambio incremental del catálogo. */
public sealed interface CatalogChange {

    SectionId sectionId();

    record SeatsChanged(SectionId section, int available) implements CatalogChange {
        @Override
        public SectionId sectionId() {
            return section;
        }
    }

    record SectionUpserted(Section section, int available) implements CatalogChange {
        @Override
        public SectionId sectionId() {
            return section.id();
        }
    }

    record SectionRemoved(SectionId section) implements CatalogChange {
        @Override
        public SectionId sectionId() {
            return section;
        }
    }
}
