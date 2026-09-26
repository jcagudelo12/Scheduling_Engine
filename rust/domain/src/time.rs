#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Franja semanal `[start, end)` expresada en minutos desde la medianoche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeSlot {
    pub day: Weekday,
    pub start: u16,
    pub end: u16,
}

impl TimeSlot {
    /// Devuelve `None` si la franja está vacía o se sale del día.
    pub fn new(day: Weekday, start: u16, end: u16) -> Option<Self> {
        (start < end && end <= 24 * 60).then_some(Self { day, start, end })
    }

    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.day == other.day && self.start < other.end && other.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(day: Weekday, start: u16, end: u16) -> TimeSlot {
        TimeSlot::new(day, start, end).unwrap()
    }

    #[test]
    fn contiguous_slots_do_not_overlap() {
        let a = slot(Weekday::Monday, 420, 540);
        let b = slot(Weekday::Monday, 540, 660);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn same_hours_on_different_days_do_not_overlap() {
        let a = slot(Weekday::Monday, 420, 540);
        let b = slot(Weekday::Tuesday, 420, 540);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn partial_overlap_is_detected() {
        let a = slot(Weekday::Friday, 420, 540);
        let b = slot(Weekday::Friday, 500, 600);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn invalid_slots_are_rejected() {
        assert!(TimeSlot::new(Weekday::Monday, 600, 600).is_none());
        assert!(TimeSlot::new(Weekday::Monday, 600, 1500).is_none());
    }
}
