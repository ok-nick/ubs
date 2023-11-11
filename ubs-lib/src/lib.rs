//! # ubs
//! `ubs` provides an interface for fetching real-time Univeristy at Buffalo class schedules.
//! ## Usage
//! ```rust
//! # use futures::stream::TryStreamExt;
//! use ubs_lib::{Course, Semester};
//!
//! # async fn run() -> Result<(), ubs_lib::ScheduleError> {
//! let mut schedule_iter = ubs_lib::schedule_iter(
//!     Course::Cse115Llr,
//!     Semester::Spring2024,
//! ).await?;
//!
//! while let Some(schedule) = schedule_iter.try_next().await? {
//!     for group in schedule?.group_iter()? {
//!         for class in group.class_iter() {
//!             // do stuff
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//!```

mod ids;
pub mod model;
pub mod parser;
pub mod session;

pub use ids::{Career, Course, ParseIdError, Semester};
use parser::{ClassSchedule, ParseError};
use session::{Query, Session, SessionError, Token};

use futures::{TryStream, TryStreamExt};
use hyper::Client;

/// Iterator over each page of the specified query.
///
/// If there is no course to career mapping for the specified course, this function
/// will return [`ScheduleError::FailedToInferCareer`](ScheduleError::FailedToInferCareer).
/// In that case, consider manually specifying the career via [`schedule_iter_with_career`](schedule_iter_with_career),
/// and sending a PR to add the inference.
#[cfg(feature = "rustls")]
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

/// Iterator over each page of the specified query with an explicit career.
///
/// Note that in some cases the career cannot be inferred from the course, thus it
/// is necessary to manually specify the career. Considering sending a PR with the
/// course to career mapping.
#[cfg(feature = "rustls")]
pub async fn schedule_iter_with_career<'a>(
    course: Course,
    semester: Semester,
    career: Career,
) -> Result<
    impl TryStream<Ok = Result<ClassSchedule, ParseError>, Error = SessionError> + 'a,
    ScheduleError,
> {
    let client = Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build(),
    );
    let token = Token::new(&client).await?;

    let session = Session::new(client, token);
    session.initialize(&semester).await?;

    Ok(session
        .schedule_iter(Query::new(course, semester, career))
        .map_ok(|bytes| ClassSchedule::new(bytes.into())))
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
