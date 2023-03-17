mod course;
mod parser;
mod session;

pub use course::Course;
pub use parser::{Class, ClassGroup, ClassSchedule, ClassType, ParseError};
pub use session::{Session, SessionError, Token};

use futures::{TryStream, TryStreamExt};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;
use thiserror::Error;

// TODO: add feature in docs
// #[cfg(feature = "client")]
pub async fn schedule_iter(
    course: Course,
) -> Result<
    impl TryStream<Ok = Result<ClassSchedule, ParseError>, Error = SessionError>,
    SessionError,
> {
    let client = Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build(),
    );
    let token = Token::new(&client).await?;
    Ok(Session::new(client, &token)
        .schedule_iter(course)
        .map_ok(|bytes| ClassSchedule::new(bytes, 0)))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to connect to server")]
    SessionError(#[from] SessionError),
    #[error("failed to parse data")]
    ParseError(#[from] ParseError),
}
