mod parser;
mod session;

pub use parser::{Class, ClassGroup, ClassSchedule, ClassType, ParseError};
pub use session::{Session, SessionError, Token};

use futures::{TryStream, TryStreamExt};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;

// TODO: add feature in docs
// #[cfg(feature = "client")]
pub async fn schedule_iter(
    course_id: &str,
) -> impl TryStream<Ok = ClassSchedule, Error = SessionError> + '_ {
    let client = Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build(),
    );
    let token = Token::new(&client).await.unwrap();
    Session::new(client, &token)
        .schedule_iter(course_id)
        .map_ok(|bytes| ClassSchedule::new(bytes, 0).unwrap())
}
