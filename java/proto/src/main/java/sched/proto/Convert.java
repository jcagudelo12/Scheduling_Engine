package sched.proto;

import java.util.ArrayList;
import java.util.List;
import sched.domain.CatalogChange;
import sched.domain.CatalogEvent;
import sched.domain.CourseId;
import sched.domain.Section;
import sched.domain.SectionId;
import sched.domain.SectionSeats;
import sched.domain.TimeSlot;
import sched.domain.Weekday;
import sched.proto.institution.v1.SectionRemoved;
import sched.proto.institution.v1.SectionUpserted;
import sched.proto.institution.v1.SeatsChanged;

/**
 * Conversión entre mensajes protobuf y tipos del dominio. Al convertir hacia el dominio se
 * validan los datos recibidos.
 */
public final class Convert {

    /** Datos recibidos que no cumplen el contrato. */
    public static final class ConversionException extends Exception {
        public ConversionException(String message) {
            super(message);
        }
    }

    private Convert() {
    }

    /** Grupo con sus cupos disponibles. */
    public static SectionSeats section(sched.proto.institution.v1.Section pb) throws ConversionException {
        if (pb.getSectionId().isEmpty() || pb.getCourseId().isEmpty()) {
            throw new ConversionException("grupo sin identificador de grupo o de curso");
        }
        List<TimeSlot> slots = new ArrayList<>(pb.getSlotsCount());
        for (var slot : pb.getSlotsList()) {
            slots.add(timeSlot(slot, pb.getSectionId()));
        }
        Section section = new Section(
                new SectionId(pb.getSectionId()), new CourseId(pb.getCourseId()), slots, pb.getCapacity());
        return new SectionSeats(section, pb.getAvailableSeats());
    }

    public static CatalogEvent catalogEvent(sched.proto.institution.v1.CatalogEvent pb) throws ConversionException {
        CatalogChange change = switch (pb.getChangeCase()) {
            case SEATS_CHANGED -> new CatalogChange.SeatsChanged(
                    new SectionId(pb.getSeatsChanged().getSectionId()), pb.getSeatsChanged().getAvailableSeats());
            case SECTION_UPSERTED -> {
                if (!pb.getSectionUpserted().hasSection()) {
                    throw new ConversionException("SectionUpserted sin grupo");
                }
                SectionSeats s = section(pb.getSectionUpserted().getSection());
                yield new CatalogChange.SectionUpserted(s.section(), s.available());
            }
            case SECTION_REMOVED -> new CatalogChange.SectionRemoved(
                    new SectionId(pb.getSectionRemoved().getSectionId()));
            case CHANGE_NOT_SET -> throw new ConversionException("evento " + pb.getSeq() + " sin cambio");
        };
        return new CatalogEvent(pb.getSeq(), change);
    }

    public static sched.proto.institution.v1.Section sectionToPb(Section section, int available) {
        var builder = sched.proto.institution.v1.Section.newBuilder()
                .setSectionId(section.id().value())
                .setCourseId(section.course().value())
                .setCapacity(section.capacity())
                .setAvailableSeats(available);
        for (TimeSlot slot : section.slots()) {
            builder.addSlots(sched.proto.institution.v1.TimeSlot.newBuilder()
                    .setDay(weekdayToPb(slot.day()))
                    .setStartMinute(slot.start())
                    .setEndMinute(slot.end()));
        }
        return builder.build();
    }

    public static sched.proto.institution.v1.CatalogEvent catalogEventToPb(CatalogEvent event) {
        var builder = sched.proto.institution.v1.CatalogEvent.newBuilder().setSeq(event.seq());
        switch (event.change()) {
            case CatalogChange.SeatsChanged(SectionId id, int available) -> builder.setSeatsChanged(
                    SeatsChanged.newBuilder().setSectionId(id.value()).setAvailableSeats(available));
            case CatalogChange.SectionUpserted(Section section, int available) -> builder.setSectionUpserted(
                    SectionUpserted.newBuilder().setSection(sectionToPb(section, available)));
            case CatalogChange.SectionRemoved(SectionId id) -> builder.setSectionRemoved(
                    SectionRemoved.newBuilder().setSectionId(id.value()));
        }
        return builder.build();
    }

    private static TimeSlot timeSlot(sched.proto.institution.v1.TimeSlot pb, String sectionId)
            throws ConversionException {
        Weekday day = switch (pb.getDay()) {
            case WEEKDAY_MONDAY -> Weekday.MONDAY;
            case WEEKDAY_TUESDAY -> Weekday.TUESDAY;
            case WEEKDAY_WEDNESDAY -> Weekday.WEDNESDAY;
            case WEEKDAY_THURSDAY -> Weekday.THURSDAY;
            case WEEKDAY_FRIDAY -> Weekday.FRIDAY;
            case WEEKDAY_SATURDAY -> Weekday.SATURDAY;
            case WEEKDAY_SUNDAY -> Weekday.SUNDAY;
            case WEEKDAY_UNSPECIFIED, UNRECOGNIZED -> null;
        };
        if (day == null) {
            throw new ConversionException("franja inválida en el grupo " + sectionId);
        }
        return TimeSlot.of(day, pb.getStartMinute(), pb.getEndMinute())
                .orElseThrow(() -> new ConversionException("franja inválida en el grupo " + sectionId));
    }

    private static sched.proto.institution.v1.Weekday weekdayToPb(Weekday day) {
        return switch (day) {
            case MONDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_MONDAY;
            case TUESDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_TUESDAY;
            case WEDNESDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_WEDNESDAY;
            case THURSDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_THURSDAY;
            case FRIDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_FRIDAY;
            case SATURDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_SATURDAY;
            case SUNDAY -> sched.proto.institution.v1.Weekday.WEEKDAY_SUNDAY;
        };
    }
}
