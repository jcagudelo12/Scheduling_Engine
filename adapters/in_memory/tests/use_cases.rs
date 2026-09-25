//! Pruebas de los casos de uso con los adaptadores en memoria.

use std::sync::Arc;

use sched_adapter_in_memory::InMemoryCatalogLog;
use sched_application::use_cases::{
    ApplyLogEntry, GenerateCombinations, IngestCatalog, IngestCatalogEvent,
};
use sched_application::{AppError, CatalogLogEntry, CatalogReplica, IngestState};
use sched_domain::{
    ApplyOutcome, CatalogChange, CatalogEvent, CourseId, Section, SectionId, StudentContext,
    StudentId, TimeSlot, Weekday,
};
use sched_solver::BacktrackingSolver;

fn section(id: &str, course: &str, day: Weekday) -> Section {
    Section {
        id: SectionId::new(id),
        course: CourseId::new(course),
        slots: vec![TimeSlot::new(day, 420, 540).unwrap()],
        capacity: 30,
    }
}

fn catalog() -> Vec<(Section, u32)> {
    vec![
        (section("MAT-01", "MAT", Weekday::Monday), 5),
        (section("MAT-02", "MAT", Weekday::Wednesday), 5),
        (section("FIS-01", "FIS", Weekday::Tuesday), 5),
    ]
}

fn student() -> StudentContext {
    StudentContext {
        id: StudentId::new("s1"),
        eligible_courses: vec![CourseId::new("MAT"), CourseId::new("FIS")],
    }
}

fn seats(seq: u64, id: &str, available: u32) -> CatalogEvent {
    CatalogEvent {
        seq,
        change: CatalogChange::SeatsChanged {
            section: SectionId::new(id),
            available,
        },
    }
}

struct Engine {
    log: Arc<InMemoryCatalogLog>,
    load: IngestCatalog<Arc<InMemoryCatalogLog>>,
    apply: IngestCatalogEvent<Arc<InMemoryCatalogLog>>,
    generate: GenerateCombinations<BacktrackingSolver>,
}

fn engine() -> Engine {
    let replica = CatalogReplica::default();
    let log = Arc::new(InMemoryCatalogLog::new(Arc::new(ApplyLogEntry::new(
        replica.clone(),
    ))));
    let ingest = IngestState::new(replica.clone());
    Engine {
        load: IngestCatalog::new(ingest.clone(), Arc::clone(&log)),
        apply: IngestCatalogEvent::new(ingest, Arc::clone(&log)),
        generate: GenerateCombinations::new(BacktrackingSolver, replica, 50),
        log,
    }
}

#[tokio::test]
async fn full_section_is_excluded_after_seat_change() {
    let engine = engine();
    engine.load.execute(100, catalog()).await.unwrap();

    let before = engine.generate.execute(&student()).unwrap();
    assert_eq!(before.schedules.len(), 2);
    assert_eq!(before.catalog_seq, 100);

    engine.apply.execute(seats(101, "MAT-01", 0)).await.unwrap();

    let after = engine.generate.execute(&student()).unwrap();
    assert_eq!(after.schedules.len(), 1);
    assert_eq!(after.catalog_seq, 101);
    assert!(!after.stale);
}

#[tokio::test]
async fn duplicates_are_not_written_twice() {
    let engine = engine();
    engine.load.execute(100, catalog()).await.unwrap();

    engine.apply.execute(seats(101, "MAT-01", 0)).await.unwrap();
    let again = engine.apply.execute(seats(101, "MAT-01", 0)).await.unwrap();

    assert_eq!(again, ApplyOutcome::Duplicate);
    assert_eq!(engine.log.entries().len(), 2);
}

#[tokio::test]
async fn gap_marks_replicas_stale_until_reload() {
    let engine = engine();
    engine.load.execute(100, catalog()).await.unwrap();

    let gap = engine.apply.execute(seats(105, "MAT-01", 0)).await;
    assert!(matches!(gap, Err(AppError::SequenceGap(_))));
    assert_eq!(
        engine.log.entries().last(),
        Some(&CatalogLogEntry::SyncLost)
    );
    assert!(engine.generate.execute(&student()).unwrap().stale);

    // Tras un hueco se rechaza todo, incluso el seq "correcto", hasta la recarga.
    let still = engine.apply.execute(seats(101, "MAT-01", 0)).await;
    assert!(matches!(still, Err(AppError::SequenceGap(_))));

    engine.load.execute(105, catalog()).await.unwrap();
    assert!(!engine.generate.execute(&student()).unwrap().stale);
    engine.apply.execute(seats(106, "MAT-01", 0)).await.unwrap();
}

#[tokio::test]
async fn reconnecting_without_gaps_clears_stale() {
    let engine = engine();
    engine.load.execute(100, catalog()).await.unwrap();

    engine.apply.connection_lost().await;
    assert!(engine.generate.execute(&student()).unwrap().stale);

    engine.apply.execute(seats(101, "MAT-01", 3)).await.unwrap();
    assert!(!engine.generate.execute(&student()).unwrap().stale);
}

#[tokio::test]
async fn changes_before_first_load_are_refused() {
    let engine = engine();
    assert!(matches!(
        engine.apply.execute(seats(1, "MAT-01", 0)).await,
        Err(AppError::CatalogUnavailable)
    ));
    assert!(matches!(
        engine.generate.execute(&student()),
        Err(AppError::CatalogUnavailable)
    ));
}
