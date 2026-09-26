//! Adaptador de entrada HTTP.
//!
//! - `POST /combinations`, con el contexto del estudiante en el cuerpo.
//! - `GET /health`: el proceso está vivo.
//! - `GET /ready`: la réplica ya se puso al día con el historial (ADR 0005).
//!
//! Provisional: el contexto llega sin firmar. Debe reemplazarse por el token firmado
//! por la institución (ADR 0004) antes de exponer este endpoint.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sched_application::use_cases::GenerateCombinations;
use sched_application::{AppError, PortError, Readiness};
use sched_domain::{CourseId, Schedule, StudentContext, StudentId, Weekday};
use sched_solver::Solver;
use serde::{Deserialize, Serialize};

pub fn router<S>(generate: Arc<GenerateCombinations<S>>) -> Router
where
    S: Solver + Send + Sync + 'static,
{
    Router::new()
        .route("/combinations", post(combinations::<S>))
        .with_state(generate)
}

/// Sondas para el orquestador y el balanceador.
pub fn health_router(readiness: Readiness) -> Router {
    Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/ready", get(ready))
        .with_state(readiness)
}

async fn ready(State(readiness): State<Readiness>) -> StatusCode {
    if readiness.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn combinations<S: Solver>(
    State(generate): State<Arc<GenerateCombinations<S>>>,
    Json(request): Json<StudentContextDto>,
) -> Result<Json<CombinationsResponse>, HttpError> {
    let result = generate.execute(&request.into())?;
    Ok(Json(CombinationsResponse {
        catalog_seq: result.catalog_seq,
        stale: result.stale,
        schedules: result.schedules.iter().map(ScheduleDto::from).collect(),
    }))
}

#[derive(Debug, Deserialize)]
struct StudentContextDto {
    student_id: String,
    eligible_courses: Vec<String>,
}

impl From<StudentContextDto> for StudentContext {
    fn from(dto: StudentContextDto) -> Self {
        Self {
            id: StudentId::new(dto.student_id),
            eligible_courses: dto
                .eligible_courses
                .into_iter()
                .map(CourseId::new)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct CombinationsResponse {
    catalog_seq: u64,
    stale: bool,
    schedules: Vec<ScheduleDto>,
}

#[derive(Debug, Serialize)]
struct ScheduleDto {
    sections: Vec<SectionDto>,
}

#[derive(Debug, Serialize)]
struct SectionDto {
    section_id: String,
    course_id: String,
    slots: Vec<SlotDto>,
}

#[derive(Debug, Serialize)]
struct SlotDto {
    day: &'static str,
    start_minute: u16,
    end_minute: u16,
}

impl From<&Schedule> for ScheduleDto {
    fn from(schedule: &Schedule) -> Self {
        let sections = schedule
            .sections
            .iter()
            .map(|s| SectionDto {
                section_id: s.id.to_string(),
                course_id: s.course.to_string(),
                slots: s
                    .slots
                    .iter()
                    .map(|slot| SlotDto {
                        day: weekday_name(slot.day),
                        start_minute: slot.start,
                        end_minute: slot.end,
                    })
                    .collect(),
            })
            .collect();
        Self { sections }
    }
}

fn weekday_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "monday",
        Weekday::Tuesday => "tuesday",
        Weekday::Wednesday => "wednesday",
        Weekday::Thursday => "thursday",
        Weekday::Friday => "friday",
        Weekday::Saturday => "saturday",
        Weekday::Sunday => "sunday",
    }
}

#[derive(Debug)]
struct HttpError(AppError);

impl From<AppError> for HttpError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::CatalogUnavailable | AppError::Port(PortError::Unavailable(_)) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            AppError::SequenceGap(_) | AppError::Port(PortError::InvalidData(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        if status.is_server_error() {
            tracing::error!(err = %self.0, "error atendiendo la solicitud");
        }
        (status, self.0.to_string()).into_response()
    }
}
