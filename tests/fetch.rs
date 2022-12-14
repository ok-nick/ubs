use futures::TryStreamExt;
use ubs::{ClassSchedule, Session, Token};

const CSE_115_ID: &str = "004544";

// TODO: return result
// #[cfg(feature = "client")]
#[tokio::test]
async fn fetch_schedules() {
    // let client = Client::builder().build(
    //     HttpsConnectorBuilder::new()
    //         .with_native_roots()
    //         .https_only()
    //         .enable_http1()
    //         .build(),
    // );
    let session = Session::new(client, Token::new(client).await.unwrap());
    while let Some(bytes) = session.schedule_iter(CSE_115_ID).await.try_next() {
        let page = 0;
        let class_schedule = ClassSchedule::new(bytes.unwrap(), page).unwrap();
        for group in class_schedule.group_iter() {}
    }
}
