mod ids;
mod parser;
mod session;

pub use ids::{Career, Course, Semester};
pub use parser::{Class, ClassGroup, ClassSchedule, ClassType, ParseError};
pub use session::{Query, Session, SessionError, Token};

use futures::{TryStream, TryStreamExt};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;
use thiserror::Error;

// TODO: add feature in docs
// #[cfg(feature = "client")]
pub async fn schedule_iter(
    query: Query<'_>,
) -> Result<
    impl TryStream<Ok = Result<ClassSchedule, ParseError>, Error = SessionError> + '_,
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
        .schedule_iter(query)
        // TODO: set page accordingly. Ideally, the schedule should be able to figure it out itself
        .map_ok(|bytes| ClassSchedule::new(bytes.into(), 1)))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to connect to server")]
    ConnectionFailed(#[from] SessionError),
    #[error("failed to parse data")]
    ParsingFailed(#[from] ParseError),
}
