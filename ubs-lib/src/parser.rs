//! Low-level access to the schedule parser.

use std::{borrow::Cow, fmt::Display, str::FromStr};

use chrono::{NaiveDate, NaiveTime};
use regex::Regex;
use thiserror::Error;
use tl::{Node, ParserOptions, VDom, VDomGuard};

use crate::{ParseIdError, Semester};

const CLASSES_PER_PAGE: u32 = 50;
const CLASSES_PER_GROUP: u32 = 3;

// Rust does macro expansion before resolving consts, thus I cannot embed `{}` directly
// in consts and use the `format!` macro. Defining declarative macros via `macro_rules!` is an
// alternative to get around this limitation.

// First is the class group index ((page * 50) - 1)
const SESSION_FORMAT: &str = r"^University (\d\d?) Week Session$";
macro_rules! SESSION_TAG {
    () => {
        "SSR_DER_CS_GRP_SESSION_CODE$215$${}"
    };
}
// First is class index in group (1-3)
// Second is (294, 295, 296) depending on class index in group (1-3)
// Third is the class group index ((page * 50) - 1)
const CLASS_ID_FORMAT: &str = r"^Class Nbr (\d+) - Section ([A-Z](?:\d?)+) ([A-Z]+)$";
const CLASS_ID_TAG_SEQ: [u32; 3] = [294, 295, 296];
macro_rules! CLASS_ID_TAG {
    () => {
        "SSR_CLSRCH_F_WK_SSR_CMPNT_DESCR_{}${}$${}"
    };
}
// First is the class group index ((page * 50) - 1)
const DATES_TIME_FORMAT: &str = "%m/%d/%Y";
macro_rules! DATES_TAG {
    () => {
        "SSR_CLSRCH_F_WK_SSR_MTG_DT_LONG_1$88$${}"
    };
}
// First is class index in group (1-3)
// Second is (134, 135, 154) depending on class index in group (1-3)
// Third is the class group index ((page * 50) - 1)
const DATETIME_TIME_FORMAT: &str = "%-I:%M%p";
const DATETIME_FORMAT: &str =
    r"^((?:[A-Z][a-z]+\s)+)(\d?\d:\d\d(?:AM|PM)) to (\d?\d:\d\d(?:AM|PM))$";
const DATETIME_TAG_SEQ: [u32; 3] = [134, 135, 154];
macro_rules! DATETIME_TAG {
    () => {
        "SSR_CLSRCH_F_WK_SSR_MTG_SCHED_L_{}${}$${}"
    };
}
// First is class index in group (1-3)
// Second is the class group index ((page * 50) - 1)
macro_rules! ROOM_TAG {
    () => {
        "SSR_CLSRCH_F_WK_SSR_MTG_LOC_LONG_{}${}"
    };
}
// First is class index in group (1-3)
// Second is (86, 161, 162) depending on class index in group (1-3)
// Third is the class group index ((page * 50) - 1)
const INSTRUCTOR_TAG_SEQ: [u32; 3] = [86, 161, 162];
macro_rules! INSTRUCTOR_TAG {
    () => {
        "SSR_CLSRCH_F_WK_SSR_INSTR_LONG_{}${}$${}"
    };
}
// First is class index in group (1-3)
// Second is the class group index ((page * 50) - 1)
const SEATS_FORMAT: &str = r"^Open Seats (\d+) of (\d+)$";
macro_rules! SEATS_TAG {
    () => {
        "SSR_CLSRCH_F_WK_SSR_DESCR50_{}${}"
    };
}

// TODO: I can supply more information, like class description, units, etc.
/// Parser for raw class schedule data.
#[derive(Debug)]
pub struct ClassSchedule {
    dom: VDomGuard,
    page: u32,
}

impl ClassSchedule {
    /// Construct a new [`ClassSchedule`](ClassSchedule) with the specified bytes at the specified page.
    pub fn new(bytes: Vec<u8>, page: u32) -> Result<Self, ParseError> {
        // TODO: consider enabling tracking for perf
        let dom = unsafe { tl::parse_owned(String::from_utf8(bytes)?, ParserOptions::default())? };

        Ok(Self { dom, page })
    }

    /// Get the semester for the schedule.
    pub fn semester(&self) -> Result<Semester, ParseError> {
        get_text_from_id_without_sub_nodes(self.dom.get_ref(), "TERM_VAL_TBL_DESCR")?
            .parse::<Semester>()
            .map_err(|err| err.into())
    }

    /// Get a group from its index.
    pub fn group_from_index(&self, index: u32) -> ClassGroup {
        return ClassGroup {
            dom: self.dom.get_ref(),
            group_num: index,
        };
    }

    /// Iterator over groups of classes.
    ///
    /// In the catalog, classes are grouped in sets of 3 (usually)
    /// which can only be selected together.
    pub fn group_iter(&self) -> impl Iterator<Item = ClassGroup<'_>> + '_ {
        // Every page contains the bytes of the previous pages
        let first_class_index = self.page.saturating_sub(1) * CLASSES_PER_PAGE;
        let last_class_index = (self.page * CLASSES_PER_PAGE).saturating_sub(1);

        (first_class_index..last_class_index).map(|group_num| ClassGroup {
            dom: self.dom.get_ref(),
            group_num,
        })
    }
}

// TODO: Every lecture is paired with every possible combo of recs/labs, I can simplify this
/// Parser for raw class group data.
#[derive(Debug, Clone)]
pub struct ClassGroup<'a> {
    dom: &'a VDom<'a>,
    group_num: u32,
}

// TODO: return if group is open/closed (not as straightforward as getting id)
impl<'a> ClassGroup<'a> {
    /// Get a class from its index.
    pub fn class_from_index(&self, index: u32) -> Class {
        return Class {
            dom: self.dom,
            class_num: index,
            group_num: self.group_num,
        };
    }

    /// Iterator over classes in group.
    pub fn class_iter(&self) -> impl Iterator<Item = Class<'a>> + '_ {
        (0..CLASSES_PER_GROUP).map(|class_num| Class {
            dom: self.dom,
            class_num,
            group_num: self.group_num,
        })
    }

    /// Get the current session of the class group.
    ///
    /// For instance, if the session is `University 15 Week Session`,
    /// this function will return `15`.
    pub fn session(&self) -> Result<u32, ParseError> {
        let session =
            get_text_from_id_without_sub_nodes(self.dom, &format!(SESSION_TAG!(), self.group_num))?;
        let re = Regex::new(SESSION_FORMAT)
            .unwrap()
            .captures(session)
            .ok_or(ParseError::UnknownElementFormat)?;
        re.get(1)
            .ok_or(ParseError::UnknownElementFormat)?
            .as_str()
            .parse()
            .map_err(|_| ParseError::UnknownElementFormat)
    }

    /// Get the start date of the class group.
    pub fn start_date(&self) -> Result<NaiveDate, ParseError> {
        Ok(self.dates()?.0)
    }

    /// Get the end date of the class group.
    pub fn end_date(&self) -> Result<NaiveDate, ParseError> {
        Ok(self.dates()?.1)
    }

    /// Get the start and end date of the class group.
    fn dates(&self) -> Result<(NaiveDate, NaiveDate), ParseError> {
        let dates =
            get_text_from_id_without_sub_nodes(self.dom, &format!(DATES_TAG!(), self.group_num))?;

        let mut split_dates = dates.split(" - ");
        // TODO: remove boilerplate, regex?
        Ok((
            NaiveDate::parse_from_str(
                split_dates.next().ok_or(ParseError::UnknownElementFormat)?,
                DATES_TIME_FORMAT,
            )
            .or(Err(ParseError::UnknownElementFormat))?,
            NaiveDate::parse_from_str(
                split_dates.next().ok_or(ParseError::UnknownElementFormat)?,
                DATES_TIME_FORMAT,
            )
            .or(Err(ParseError::UnknownElementFormat))?,
        ))
    }
}

// TODO: empty text will equal `&nbsp;`
/// Parser for raw class data.
#[derive(Debug, Clone)]
pub struct Class<'a> {
    dom: &'a VDom<'a>,
    class_num: u32,
    group_num: u32,
}

impl Class<'_> {
    /// Get if the class is open or closed.
    pub fn is_open(&self) -> Result<bool, ParseError> {
        let seats = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(SEATS_TAG!(), self.class_num + 1, self.group_num),
        )?;

        if seats == "Closed" {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get the type of class.
    ///
    /// For instance, this function will return `Lecture`, `Seminar`,
    /// `Lab`, `Recitation`.
    pub fn class_type(&self) -> Result<ClassType, ParseError> {
        self.class_info()
            .map(|info| info.2.parse().map_err(|_| ParseError::UnknownElementFormat))?
    }

    /// Get id of this class.
    ///
    /// For instance, if the class says `Class Nbr 23229`, this function
    /// will return `23229`.
    pub fn class_id(&self) -> Result<u32, ParseError> {
        self.class_info()
            .map(|info| info.0.parse().map_err(|_| ParseError::UnknownElementFormat))?
    }

    /// Get the section of this class.
    ///
    /// For instance, if the class says `Section A5`, this function will
    /// return `A5`.
    pub fn section(&self) -> Result<&str, ParseError> {
        self.class_info().map(|info| info.1)
    }

    // If the class is asynchronous a datetime doesn't exist.
    /// Get the days of week this class is in action.
    pub fn days_of_week(&self) -> Result<Option<Vec<Result<DayOfWeek, ParseError>>>, ParseError> {
        self.datetime().map(|result| {
            result.map(|datetime| {
                datetime
                    .0
                    .iter()
                    .map(|days| days.parse().map_err(|_| ParseError::UnknownElementFormat))
                    .collect()
            })
        })
    }

    /// Get the start time of this class.
    pub fn start_time(&self) -> Result<Option<NaiveTime>, ParseError> {
        self.datetime()
            .map(|result| {
                result.map(|datetime| {
                    NaiveTime::parse_from_str(&datetime.1, DATETIME_TIME_FORMAT)
                        .map_err(|_| ParseError::UnknownElementFormat)
                })
            })?
            .transpose()
    }

    /// Get the end time of this class.
    pub fn end_time(&self) -> Result<Option<NaiveTime>, ParseError> {
        // TODO: fix boilerplate with above
        self.datetime()
            .map(|result| {
                result.map(|datetime| {
                    NaiveTime::parse_from_str(&datetime.2, DATETIME_TIME_FORMAT)
                        .map_err(|_| ParseError::UnknownElementFormat)
                })
            })?
            .transpose()
    }

    // Sometimes it returns `Arr Arr`
    /// Get the room and room number of this class.
    ///
    /// For instance, if the class says `Nsc 215`, this function will
    /// return `Nsc 215`.
    pub fn room(&self) -> Result<&str, ParseError> {
        // TODO: use regex to validate result
        get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(ROOM_TAG!(), self.class_num + 1, self.group_num),
        )
    }

    // TODO: specific error if the class says "To be Announced"
    /// Get the name of the instructor.
    ///
    /// Note that sometimes the instructor doesn't exist and is labeled as
    /// `To be Announced`. In that case, the function will error.
    pub fn instructor(&self) -> Result<&str, ParseError> {
        // Not much I can do in terms of validation. Some people have very unique patterns in their
        // names.
        get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(
                INSTRUCTOR_TAG!(),
                self.class_num + 1,
                INSTRUCTOR_TAG_SEQ[self.class_num as usize],
                self.group_num
            ),
        )
    }

    // TODO: specific error for closed class
    /// Get the open seats for this class.
    ///
    /// Note that if the class is closed this function will error.
    pub fn open_seats(&self) -> Result<Option<u32>, ParseError> {
        self.seats().map(|seats| seats.map(|seats| seats.0))
    }

    // TODO: ^
    /// Get the total seats for this class.
    ///
    /// Note that if the class is closed this function will error.
    pub fn total_seats(&self) -> Result<Option<u32>, ParseError> {
        self.seats().map(|seats| seats.map(|seats| seats.1))
    }

    /// Get various bits of information for this class in the form,
    /// `(class_type, class_id, section)`.
    fn class_info(&self) -> Result<(&str, &str, &str), ParseError> {
        let class_info = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(
                CLASS_ID_TAG!(),
                self.class_num + 1,
                CLASS_ID_TAG_SEQ[self.class_num as usize],
                self.group_num
            ),
        )?;

        let re = Regex::new(CLASS_ID_FORMAT)
            .unwrap()
            .captures(class_info)
            .ok_or(ParseError::UnknownElementFormat)?;
        Ok((
            re.get(1).ok_or(ParseError::UnknownElementFormat)?.as_str(),
            re.get(2).ok_or(ParseError::UnknownElementFormat)?.as_str(),
            re.get(3).ok_or(ParseError::UnknownElementFormat)?.as_str(),
        ))
    }

    /// Get various bits of information for this class dates in the form,
    /// `(days_of_weeek, start_time, end_time)`.
    fn datetime(&self) -> Result<Option<(Vec<String>, String, String)>, ParseError> {
        get_node_from_id(
            self.dom,
            &format!(
                DATETIME_TAG!(),
                self.class_num + 1,
                DATETIME_TAG_SEQ[self.class_num as usize],
                self.group_num
            ),
        )
        // If the tag is missing it could mean `Time Conflict` is being displayed. In that
        // case, skip it and label the datetime as non-existent.
        // TODO: but it could also mean the format is unknown. Return error with source attached.
        .map_or_else(
            |err| match err {
                ParseError::MissingTag => Ok(None),
                _ => Err(err),
            },
            |node| {
                match node.inner_text(self.dom.parser()) {
                    Cow::Borrowed(_) => Err(ParseError::UnknownHtmlFormat),
                    Cow::Owned(value) => {
                        let re = Regex::new(DATETIME_FORMAT)
                            .unwrap()
                            .captures(&value)
                            .ok_or(ParseError::UnknownElementFormat)?;

                        Ok(Some((
                            re.get(1)
                                .ok_or(ParseError::UnknownElementFormat)?
                                .as_str()
                                .split_whitespace()
                                .map(|string| string.to_owned())
                                .collect(), // Days of week (e.g. Wednesday)
                            re.get(2)
                                .ok_or(ParseError::UnknownElementFormat)?
                                .as_str()
                                .to_owned(), // Start time (e.g. 3:00PM)
                            re.get(3)
                                .ok_or(ParseError::UnknownElementFormat)?
                                .as_str()
                                .to_owned(), // End time (e.g. 4:00PM)
                        )))
                    }
                }
            },
        )
    }

    /// Get various bits of information for this class seats in the form,
    /// `(days_of_weeek, start_time, end_time)`.
    fn seats(&self) -> Result<Option<(u32, u32)>, ParseError> {
        let seats = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(SEATS_TAG!(), self.class_num + 1, self.group_num),
        )?;

        match seats {
            "Closed" => Ok(None),
            _ => {
                let re = Regex::new(SEATS_FORMAT)
                    .unwrap()
                    .captures(seats)
                    .ok_or(ParseError::UnknownElementFormat)?;

                Ok(Some((
                    re.get(1)
                        .ok_or(ParseError::UnknownElementFormat)?
                        .as_str()
                        .parse()
                        .map_err(|_| ParseError::UnknownElementFormat)?, // Open seats
                    re.get(2)
                        .ok_or(ParseError::UnknownHtmlFormat)?
                        .as_str()
                        .parse()
                        .map_err(|_| ParseError::UnknownElementFormat)?, // Total seats
                )))
            }
        }
    }
}

/// Type of class.
#[derive(Debug, Clone, Copy)]
pub enum ClassType {
    Recitation,
    Lab,
    Lecture,
    Seminar,
}

impl FromStr for ClassType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "REC" => ClassType::Recitation,
            "LAB" => ClassType::Lab,
            "LEC" => ClassType::Lecture,
            "SEM" => ClassType::Seminar,
            _ => return Err(ParseError::UnknownElementFormat),
        })
    }
}

impl Display for ClassType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ClassType::Recitation => "Recitation",
                ClassType::Lab => "Lab",
                ClassType::Lecture => "Lecture",
                ClassType::Seminar => "Seminar",
            }
        )
    }
}

/// Day of week.
#[derive(Debug, Clone, Copy)]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl FromStr for DayOfWeek {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Sunday" => DayOfWeek::Sunday,
            "Monday" => DayOfWeek::Monday,
            "Tuesday" => DayOfWeek::Tuesday,
            "Wednesday" => DayOfWeek::Wednesday,
            "Thursday" => DayOfWeek::Thursday,
            "Friday" => DayOfWeek::Friday,
            "Saturday" => DayOfWeek::Saturday,
            _ => return Err(ParseError::UnknownElementFormat),
        })
    }
}

impl Display for DayOfWeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DayOfWeek::Sunday => "Sunday",
                DayOfWeek::Monday => "Monday",
                DayOfWeek::Tuesday => "Tuesday",
                DayOfWeek::Wednesday => "Wednesday",
                DayOfWeek::Thursday => "Thursday",
                DayOfWeek::Friday => "Friday",
                DayOfWeek::Saturday => "Saturday",
            }
        )
    }
}

// TODO: document
fn get_text_from_id_without_sub_nodes<'a>(dom: &'a VDom, id: &str) -> Result<&'a str, ParseError> {
    match get_node_from_id(dom, id)?.inner_text(dom.parser()) {
        Cow::Borrowed(string) => Ok(string),
        // TODO: this is relying on implementation details, make it more explicit
        // If it's owned, that means the element had multiple sub-nodes, which shouldn't be the
        // case
        Cow::Owned(_) => Err(ParseError::UnknownHtmlFormat),
    }
}

// TODO: ^
fn get_node_from_id<'a>(dom: &'a VDom, id: &str) -> Result<&'a Node<'a>, ParseError> {
    Ok(dom
        .get_element_by_id(id)
        .ok_or(ParseError::MissingTag)?
        .get(dom.parser())
        // We know the element exists in the DOM because that's where we got it from
        .unwrap())
}

/// Error when parsing schedule data.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Id is in an unknown format.
    #[error(transparent)]
    UnknownIdFormat(#[from] ParseIdError),
    /// HTML is not valid Utf-8.
    #[error("could not parse HTML due to invalid Utf-8 encoding")]
    // HtmlInvalidUtf8(#[from] str::Utf8Error),
    HtmlInvalidUtf8(#[from] std::string::FromUtf8Error),
    /// HTML is not in a valid format.
    #[error("could not parse HTML due to invalid format")]
    InvalidHtmlFormat(#[from] tl::errors::ParseError),
    /// HTML is empty.
    #[error("could not find tags in HTML")]
    EmptyHtml,
    /// HTML is in an unknown format.
    #[error("format of HTML could not be parsed because it is unknown")]
    UnknownHtmlFormat,
    // TODO: I can provide much more context here
    /// Content of HTML element is in an unknown format
    #[error("format of element could not be parsed because it is unknown")]
    UnknownElementFormat,
    /// HTML tag for class does not exist
    #[error("could not find tag for class in HTML")]
    MissingTag,
}
