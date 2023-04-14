use clap::Parser;
use futures::TryStreamExt;
use options::Options;

use crate::{find::normalize, model::ClassSchedule, options::DataFormat};

mod find;
mod model;
mod options;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ubs_lib::Error> {
    let args = Options::parse();

    // TODO: How much less ergonomic is it to store borrowed strings rather than owned? Is it worth
    // storing owned?
    let course = normalize(&args.course);
    let semester = normalize(&args.semester);
    let career = normalize(&args.career);

    let mut schedule_iter = ubs_lib::schedule_iter(
        find::find_course(&course),
        find::find_semester(&semester),
        find::find_career(&career),
    ).await?;
    let mut schedules = Vec::new();

    #[allow(clippy::never_loop)] // TODO: temp
    while let Some(schedule) = schedule_iter.try_next().await? {
        schedules.push(ClassSchedule::try_from(schedule?)?);
        break; // TODO: for now since subsequent pages aren't implemented
    }

    let result = match args.format {
        // TODO: impl error type
        DataFormat::Json => serde_json::to_string(&schedules).unwrap(),
    };
    println!("{result}");

    Ok(())
}

// TODO: reevaluate error typs and how they are structured
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`ubs` encountered an error")]
    UbsError(#[from] ubs_lib::Error),
    #[error("failed to serialize JSON")]
    JsonSerializeFailed(#[from] serde_json::Error),
}
