mod parser;
mod session;

pub use parser::{Class, ClassGroup, ClassSchedule, ClassType, ParseError};
pub use session::{Session, SessionError, Token};

//  Client::builder().build(
//     HttpsConnectorBuilder::new()
//         .with_native_roots()
//         .https_only()
//         .enable_http1()
//         .build(),
// );
