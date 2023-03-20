use futures::TryStreamExt;
use ubs::Course;

#[tokio::test]
async fn schedule_iter() -> Result<(), ubs::Error> {
    let mut schedule_iter = ubs::schedule_iter(Course::Cse115).await?;
    while let Some(schedule) = schedule_iter.try_next().await? {
        for group in schedule?.group_iter() {
            for class in group.class_iter() {}
        }
    }

    Ok(())
}
