//! Mappings of course/semester/career to internal ids.

use std::str::FromStr;

use thiserror::Error;

/// An enum of available courses in the catalog.
///
/// If a course is missing, manually specify its id with [`Course::Raw`](Course::Raw) and
/// consider sending a PR adding that mapping.
#[derive(Debug, Clone)]
pub enum Course {
    Cse115,
    Raw(String),
}

/// An enum of available semesters in the catalog.
///
/// If a semester is missing, manually specify its id with [`Semester::Raw`](Semester::Raw) and
/// consider sending a PR adding that mapping.
#[derive(Debug, Clone)]
pub enum Semester {
    Spring2023,
    Summer2023,
    Fall2023,
    Winter2023,
    Raw(String),
}

/// An enum of available careers in the catalog.
///
/// If a career is missing, manually specify its id with [`Career::Raw`](Career::Raw) and
/// consider sending a PR adding that mapping.
///
/// Specifying the career is an internal implementation detail exposed by the backend
/// network API. It doesn't make much sense to have, but nevertheless, it is required.
#[derive(Debug, Clone)]
pub enum Career {
    Undergraduate,
    Graduate,
    Law,
    DentalMedicine,
    Medicine,
    Pharmacy,
    Raw(String),
}

impl Course {
    /// Infer the career from the course.
    ///
    /// Note that this isn't always possible because a mapping does not yet exist. In
    /// that case, consider sending a PR adding the mapping.
    pub fn career(&self) -> Option<Career> {
        match self {
            Course::Cse115 => Some(Career::Undergraduate),
            // in this case it's highly dependent on the course to determine the career
            Course::Raw(_) => None,
        }
    }

    /// Internal id of the course.
    pub(crate) fn id(&self) -> &str {
        match self {
            Course::Cse115 => "004544",
            Course::Raw(id) => id,
        }
    }
}

impl FromStr for Course {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*normalize(s) {
            "CSE115" => Ok(Course::Cse115),
            // TODO: valid course id is 6 characters and an integer
            _ => Err(ParseIdError::InvalidId {
                id: "Course".to_owned(),
                given: s.to_owned(),
            }),
        }
    }
}

impl Semester {
    /// Internal id of the semester.
    pub(crate) fn id(&self) -> &str {
        match self {
            Semester::Spring2023 => "2231",
            Semester::Summer2023 => "",
            Semester::Fall2023 => "",
            Semester::Winter2023 => "",
            Semester::Raw(id) => id,
        }
    }
}

impl FromStr for Semester {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*normalize(s) {
            "SPRING2023" => Ok(Semester::Spring2023),
            "SUMMER2023" => Ok(Semester::Summer2023),
            "FALL2023" => Ok(Semester::Fall2023),
            "WINTER2023" => Ok(Semester::Winter2023),
            _ => Err(ParseIdError::InvalidId {
                id: "Semester".to_owned(),
                given: s.to_owned(),
            }),
        }
    }
}

impl Career {
    /// Internal id of the career.
    pub(crate) fn id(&self) -> &str {
        match self {
            Career::Undergraduate => "UGRD",
            Career::Graduate => "GRAD",
            Career::Law => "LAW",
            Career::DentalMedicine => "SDM",
            Career::Medicine => "MED",
            Career::Pharmacy => "PHRM",
            Career::Raw(career) => career,
        }
    }
}

impl FromStr for Career {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*normalize(s) {
            "UNDERGRADUATE" => Ok(Career::Undergraduate),
            "GRADUATE" => Ok(Career::Graduate),
            "LAW" => Ok(Career::Law),
            "DENTALMEDICINE" => Ok(Career::DentalMedicine),
            "MEDICINE" => Ok(Career::Medicine),
            "PHARMACY" => Ok(Career::Pharmacy),
            _ => Err(ParseIdError::InvalidId {
                id: "Career".to_owned(),
                given: s.to_owned(),
            }),
        }
    }
}

/// Normalize the input string for use in [`FromStr`](std::str:FromStr) implementations.
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

/// Error when parsing id.
#[derive(Debug, Error)]
pub enum ParseIdError {
    /// Specified id could not be converted to enum.
    ///
    /// Considering using the `Raw` variant for specifying raw ids.
    #[error("`{given}` is an invalid `{id}``")]
    InvalidId { id: String, given: String },
}
