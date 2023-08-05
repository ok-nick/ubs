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
async fn main() -> Result<(), Error> {
    let args = Options::parse();

    let course = if args.raw.contains(&Raw::Course) {
        Ok(Course::Raw(args.course.to_owned()))
    } else {
        Course::from_str(&args.course)
    }?;
    let semester = if args.raw.contains(&Raw::Semester) {
        Ok(Semester::Raw(args.semester.to_owned()))
    } else {
        Semester::from_str(&args.semester)
    }?;
    let career = if args.raw.contains(&Raw::Career) {
        Ok(Career::Raw(
            args.career.ok_or(Error::RawCareerNotSpecified)?,
        ))
    } else {
        match course.career() {
            Some(career) => Ok(career),
            None => Career::from_str(&args.career.ok_or(Error::CareerNotSpecified)?),
        }
    }?;

    let mut schedule_iter = ubs_lib::schedule_iter_with_career(course, semester, career).await?;
    let mut schedules = Vec::new();

    #[allow(clippy::never_loop)] // TODO: temp
    while let Some(schedule) = schedule_iter.try_next().await? {
        schedules.push(ClassSchedule::try_from(schedule?)?);
        break; // TODO: for now since subsequent pages aren't implemented
    }

    let result = match args.format {
        DataFormat::Json => match args.pretty {
            true => serde_json::to_string_pretty(&schedules)?,
            false => serde_json::to_string(&schedules)?,
        },
    };
    println!("{result}");

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ScheduleError(#[from] ubs_lib::ScheduleError),
    #[error(transparent)]
    ParseError(#[from] ubs_lib::ParseError),
    #[error(transparent)]
    SessionError(#[from] ubs_lib::SessionError),
    #[error(transparent)]
    FailedToInferId(#[from] ubs_lib::ParseIdError),
    #[error(transparent)]
    JsonSerializeFailed(#[from] serde_json::Error),
    #[error("career not specified with `--raw` argument passed")]
    RawCareerNotSpecified,
    #[error("career could not be inferred and was not specified, consider specifying the career")]
    CareerNotSpecified,
}
