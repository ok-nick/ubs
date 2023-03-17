use futures::TryStreamExt;
use ubs::{ClassSchedule, Course, Session, Token};

// TODO: return result
// #[cfg(feature = "client")]
// #[tokio::test]
// async fn fetch_schedules() -> Result<(), ubs::SessionError> {
//     // let client = Client::builder().build(
//     //     HttpsConnectorBuilder::new()
//     //         .with_native_roots()
//     //         .https_only()
//     //         .enable_http1()
//     //         .build(),
//     // );
//     let session = Session::new(client, &Token::new(&client).await?);
//     while let Some(bytes) = session.schedule_iter(Course::Cse115).await.try_next() {
//         let page = 0;
//         let class_schedule = ClassSchedule::new(bytes.unwrap(), page).unwrap();
//         for group in class_schedule.group_iter() {
//             for class in group.class_iter() {}
//         }
//     }
//
//     Ok(())
// }

#[tokio::test]
async fn schedule_iter() -> Result<(), ubs::Error> {
    let schedule_iter = ubs::schedule_iter(Course::CSE115).await?;
    while let Some(schedule) = schedule_iter.try_next().await? {
        for group in schedule?.group_iter() {
            for class in group.class_iter() {}
        }
    }

    Ok(())
}
