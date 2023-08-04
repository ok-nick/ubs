use std::str::FromStr;

use clap::Parser;
use futures::TryStreamExt;
use options::Options;
use ubs_lib::{Career, Course, Semester};

use crate::{
    model::ClassSchedule,
    options::{DataFormat, Raw},
};

mod model;
mod options;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ubs_lib::Error> {
    let args = Options::parse();

    // TODO: handle errors
    let course = if args.raw.contains(&Raw::Course) {
        Ok(Course::Raw(args.course))
    } else {
        Course::from_str(&args.course)
    }
    .unwrap();
    let semester = if args.raw.contains(&Raw::Semester) {
        Ok(Semester::Raw(args.semester))
    } else {
        Semester::from_str(&args.semester)
    }
    .unwrap();
    let career = if args.raw.contains(&Raw::Career) {
        // TODO: error if doesn't exist
        Ok(Career::Raw(args.career.unwrap()))
    } else {
        match course.career() {
            Some(career) => Ok(career),
            // TODO: same here
            None => Career::from_str(&args.career.unwrap()),
        }
    }
    .unwrap();

    let mut schedule_iter = ubs_lib::schedule_iter(course, semester, career).await?;
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
