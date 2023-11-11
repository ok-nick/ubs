use futures::TryStreamExt;
use ubs_lib::{Course, Semester};

#[tokio::test]
async fn schedule_iter() -> Result<(), ubs_lib::ScheduleError> {
    // TODO: fix when classes in group < 3
    // let mut schedule_iter = ubs_lib::schedule_iter(Course::Apy106Lec, Semester::Spring2024).await?;
    let mut schedule_iter = ubs_lib::schedule_iter(Course::Cse115Llr, Semester::Spring2024).await?;

    while let Some(schedule) = schedule_iter.try_next().await? {
        for group in schedule?.group_iter()? {
            println!("Session: {}", group.session()?);
            println!("Start Date: {}", group.start_date()?);
            println!("End Date: {}", group.end_date()?);
            println!();
            for class in group.class_iter() {
                println!("Id: {}", class.class_type()?);
                println!("Open: {}", class.is_open()?);
                println!("Section: {}", class.section()?);
                println!("Days of Week: {:?}", class.days_of_week()?);
                println!("Start Time: {:?}", class.start_time()?);
                println!("End Time: {:?}", class.end_time()?);
                println!("Room: {}", class.room()?);
                println!("Insturctor: {}", class.instructor()?);
                println!("Open Seats: {:?}", class.open_seats()?);
                println!("Total Seats: {:?}", class.total_seats()?);
                println!();
            }
            println!();
        }
    }

    Ok(())
}
