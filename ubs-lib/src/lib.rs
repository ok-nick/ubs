mod ids;
mod parser;
mod session;

pub use ids::{Career, Course, ParseIdError, Semester};
pub use parser::{Class, ClassGroup, ClassSchedule, ClassType, ParseError};
pub use session::{Query, Session, SessionError, Token};

use futures::{TryStream, TryStreamExt};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;

// TODO: add feature in docs
// #[cfg(feature = "client")]
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

#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    #[error(transparent)]
    ConnectionFailed(#[from] SessionError),
    #[error(transparent)]
    ParseFailed(#[from] ParseError),
    #[error("failed to infer career from course `{0:?}`, consider passing it explicitly via `schedule_iter_with_career`")]
    FailedToInferCareer(Course),
}
