use futures::TryStreamExt;
use ubs_lib::{Career, Course, Query, Semester};

#[tokio::test]
async fn schedule_iter() -> Result<(), ubs_lib::Error> {
    let query = Query::new(Course::Cse115, Semester::Spring2023, Career::Undergraduate);
    let mut schedule_iter = ubs_lib::schedule_iter(&query).await?;

    #[allow(clippy::never_loop)] // TODO: temp
    while let Some(schedule) = schedule_iter.try_next().await? {
        for group in schedule?.group_iter() {
            println!(
                // TODO: group.is_open()
                "{} | {} | {}",
                group.session()?,
                group.start_date()?,
                group.end_date()?
            );
            for class in group.class_iter() {
                println!(
                    "{} | {} | {} | {:?} | {:?} | {:?} | {} | {} | {:?} | {:?}",
                    class.class_type()?,
                    class.class_id()?,
                    class.section()?,
                    class.days_of_week()?,
                    class.start_time()?,
                    class.end_time()?,
                    class.room()?,
                    class.instructor()?,
                    class.open_seats()?,
                    class.total_seats()?,
                );
            }

            println!("");
        }

        break; // TODO: for now since subsequent pages aren't implemented
    }

    Ok(())
}
