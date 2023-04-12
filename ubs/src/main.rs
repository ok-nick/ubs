use clap::Parser;
use futures::TryStreamExt;
use options::Options;
use ubs_lib::Course;

use crate::model::ClassSchedule;

mod model;
mod options;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ubs_lib::Error> {
    let args = Options::parse();

    // Normalize the string
    let course: Course = args
        .course
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
        .parse()
        .unwrap(); // TODO: handle unwrap

    let mut schedule_iter = ubs_lib::schedule_iter(course).await?;
    let mut schedules = Vec::new();

    #[allow(clippy::never_loop)] // TODO: temp
    while let Some(schedule) = schedule_iter.try_next().await? {
        schedules.push(ClassSchedule::try_from(schedule?)?);
        break; // TODO: for now since subsequent pages aren't implemented
    }

    // TODO: since there's only 1 page for now
    let json = serde_json::to_string(&schedules[0].groups).unwrap();
    println!("{json}");

    Ok(())
}
