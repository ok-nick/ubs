//! # ubs
//! `ubs` provides an interface for fetching real-time Univeristy at Buffalo class schedules.
//! ## Iterating class schedules
//! ```rust
//! use ubs_lib::{Course, Semester};
//!
//! let mut schedule_iter = ubs_lib::schedule_iter(
//!     Course::Cse115,
//!     Semester::Spring2023,
//! ).await?;
//!
//! while let Some(schedule) = schedule_iter.try_next().await? {
//!     for group in schedule?.group_iter() {
//!         for class in group.class_iter() {
//!             // do stuff
//!         }
//!     }
//! }
//!```

mod ids;
pub mod parser;
pub mod session;

pub use ids::{Career, Course, ParseIdError, Semester};
use parser::{ClassSchedule, ParseError};
use session::{Query, Session, SessionError, Token};

use futures::{TryStream, TryStreamExt};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;

/// Iterator over each page of the specified query.
///
/// If there is no course to career mapping for the specified course, this function
/// will return [`ScheduleError::FailedToInferCareer`](ScheduleError::FailedToInferCareer).
/// In that case, consider manually specifying the career via [`schedule_iter_with_career`](schedule_iter_with_career),
/// and sending a PR to add the inference.
pub async fn schedule_iter<'a>(
    course: Course,
    semester: Semester,
) -> Result<
    impl TryStream<Ok = Result<ClassSchedule, ParseError>, Error = SessionError> + 'a,
    ScheduleError,
> {
    let career = course
        .career()
        .ok_or_else(|| ScheduleError::FailedToInferCareer(course.clone()))?;
    schedule_iter_with_career(course, semester, career).await
}

/// iterator over each page of the specified query with an explicit career.
///
/// Note that in some cases the career cannot be inferred from the course, thus it
/// is necessary to manually specify the career. Considering sending a PR with the
/// course to career mapping.
pub async fn schedule_iter_with_career<'a>(
    course: Course,
    semester: Semester,
    career: Career,
) -> Result<
    impl TryStream<Ok = Result<ClassSchedule, ParseError>, Error = SessionError> + 'a,
    ScheduleError,
> {
    let client = Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build(),
    );
    let token = Token::new(&client).await?;
    Ok(Session::new(client, token)
        .schedule_iter(Query::new(course, semester, career))
        // TODO: set page accordingly. Ideally, the schedule should be able to figure it out itself
        .map_ok(|bytes| ClassSchedule::new(bytes.into(), 1)))
}

/// Error when iterating schedules.
#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    /// Failed to connect to host.
    #[error(transparent)]
    ConnectionFailed(#[from] SessionError),
    /// Failed to parse data returned from host.
    #[error(transparent)]
    ParseFailed(#[from] ParseError),
    /// Failed to infer career from course.
    #[error("failed to infer career from course `{0:?}`, consider passing it explicitly via `schedule_iter_with_career`")]
    FailedToInferCareer(Course),
}
