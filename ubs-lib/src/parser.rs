use std::{borrow::Cow, fmt::Display, str::FromStr};

use chrono::{NaiveDate, NaiveTime};
use regex::Regex;
use thiserror::Error;
use tl::{Node, ParserOptions, VDom, VDomGuard};

const CLASSES_PER_PAGE: u32 = 50;
const CLASSES_PER_GROUP: u32 = 3;

const DATES_FORMAT: &str = "%m/%d/%Y";

// Rust does macro expansion before resolving consts, therefore I cannot embed `{}` directly
// into the consts and use the `format!` macro.

// First is the class group index ((page * 50) - 1)
const SESSION_TAG_PARTS: [&str; 1] = ["SSR_DER_CS_GRP_SESSION_CODE$215$$"];
// First is class index in group (1-3)
// Second is (294, 295, 296) depending on class index in group (1-3)
// Third is the class group index ((page * 50) - 1)
const CLASS_ID_TAG_PARTS: [&str; 3] = ["SSR_CLSRCH_F_WK_SSR_CMPNT_DESCR_", "$", "$$"];
const CLASS_ID_TAG_SERIES: [u32; 3] = [294, 295, 296];
// First is the class group index ((page * 50) - 1)
const DATES_TAG_PARTS: [&str; 1] = ["SSR_CLSRCH_F_WK_SSR_MTG_DT_LONG_1$88$$"];
// First is class index in group (1-3)
// Second is (134, 135, 154) depending on class index in group (1-3)
// Third is the class group index ((page * 50) - 1)
const DATETIME_TAG_PARTS: [&str; 3] = ["SSR_CLSRCH_F_WK_SSR_MTG_SCHED_L_", "$", "$$"];
const DATETIME_TAG_SERIES: [u32; 3] = [134, 135, 154];
// First is class index in group (1-3)
// Second is the class group index ((page * 50) - 1)
const ROOM_TAG_PARTS: [&str; 2] = ["SSR_CLSRCH_F_WK_SSR_MTG_LOC_LONG_", "$"];
// First is class index in group (1-3)
// Second is (86, 161, 162) depending on class index in group (1-3)
// Third is the class group index ((page * 50) - 1)
const INSTRUCTOR_TAG_PARTS: [&str; 3] = ["SSR_CLSRCH_F_WK_SSR_INSTR_LONG_", "$", "$$"];
const INSTRUCTOR_TAG_SERIES: [u32; 3] = [86, 161, 162];
// First is class index in group (1-3)
// Second is the class group index ((page * 50) - 1)
const SEATS_TAG_PARTS: [&str; 2] = ["SSR_CLSRCH_F_WK_SSR_DESCR50_", "$"];

#[derive(Debug)]
pub struct ClassSchedule {
    dom: VDomGuard,
    page: u32,
}

// TODO: search for TERM_VAL_TBL_DESCR to get "Spring 2023"
impl ClassSchedule {
    pub fn new(bytes: Vec<u8>, page: u32) -> Result<Self, ParseError> {
        // TODO: consider enabling tracking for perf and the Arc isn't too pretty
        let dom = unsafe { tl::parse_owned(String::from_utf8(bytes)?, ParserOptions::default())? };

        Ok(Self { dom, page })
    }

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
#[derive(Debug, Clone)]
pub struct ClassGroup<'a> {
    dom: &'a VDom<'a>,
    group_num: u32,
}

impl<'a> ClassGroup<'a> {
    pub fn class_iter(&self) -> impl Iterator<Item = Class<'a>> + '_ {
        (0..CLASSES_PER_GROUP).map(|class_num| Class {
            dom: self.dom,
            class_num,
            group_num: self.group_num,
        })
    }

    // TODO: get `win6divUB_SR_FL_WRK_HTMLAREA1$5` and retrieve sub-node
    // Or get element of class ps-box-value and use its inner text
    pub fn is_open(&self) -> Result<bool, ParseError> {
        todo!()
    }

    pub fn session(&self) -> Result<u32, ParseError> {
        let session = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!("{}{}", SESSION_TAG_PARTS[0], self.group_num),
        )?;
        // TODO: cleanup
        let re = Regex::new(r"University (\d\d?) Week Session")
            .unwrap()
            .captures(session)
            .unwrap();
        Ok(re.get(1).unwrap().as_str().parse().unwrap())
    }

    pub fn start_date(&self) -> Result<NaiveDate, ParseError> {
        Ok(self.dates()?.0)
    }

    pub fn end_date(&self) -> Result<NaiveDate, ParseError> {
        Ok(self.dates()?.1)
    }

    fn dates(&self) -> Result<(NaiveDate, NaiveDate), ParseError> {
        let dates = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!("{}{}", DATES_TAG_PARTS[0], self.group_num),
        )?;

        let mut split_dates = dates.split(" - ");
        // TODO: remove boilerplate, regex?
        Ok((
            NaiveDate::parse_from_str(
                // TODO: more specific error type, here and below
                split_dates.next().ok_or(ParseError::UnknownFormat)?,
                DATES_FORMAT,
            )
            .or(Err(ParseError::UnknownFormat))?,
            NaiveDate::parse_from_str(
                split_dates.next().ok_or(ParseError::UnknownFormat)?,
                DATES_FORMAT,
            )
            .or(Err(ParseError::UnknownFormat))?,
        ))
    }
}

// TODO: empty text will equal `&nbsp;`
#[derive(Debug, Clone)]
pub struct Class<'a> {
    dom: &'a VDom<'a>,
    class_num: u32,
    group_num: u32,
}

impl Class<'_> {
    // TODO: make error
    pub fn class_type(&self) -> Result<ClassType, ParseError> {
        // TODO: handle unwrap
        self.class_info().map(|info| info.2.parse().unwrap())
    }

    // TODO: make error
    pub fn class_id(&self) -> Result<u32, ParseError> {
        // TODO: handle unwrap
        self.class_info().map(|info| info.0.parse().unwrap())
    }

    pub fn section(&self) -> Result<&str, ParseError> {
        self.class_info().map(|info| info.1)
    }

    // If the class is asynchronous a datetime doesn't exist.
    pub fn days_of_week(&self) -> Result<Option<Vec<DayOfWeek>>, ParseError> {
        self.datetime().map(|result| {
            result.map(|datetime| {
                datetime
                    .0
                    .iter()
                    // TODO: handle unwrap
                    .map(|days| days.parse().unwrap())
                    .collect()
            })
        })
    }

    pub fn start_time(&self) -> Result<Option<NaiveTime>, ParseError> {
        // TODO: fix up
        self.datetime().map(|result| {
            result.map(|datetime| NaiveTime::parse_from_str(&datetime.1, "%-I:%M%p").unwrap())
        })
    }

    pub fn end_time(&self) -> Result<Option<NaiveTime>, ParseError> {
        // TODO: fix boilerplate and unwrap
        self.datetime().map(|result| {
            result.map(|datetime| NaiveTime::parse_from_str(&datetime.2, "%-I:%M%p").unwrap())
        })
    }

    // Sometimes it returns `Arr Arr`
    pub fn room(&self) -> Result<&str, ParseError> {
        // TODO: use regex to validate result
        get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(
                "{}{}{}{}",
                ROOM_TAG_PARTS[0],
                self.class_num + 1,
                ROOM_TAG_PARTS[1],
                self.group_num
            ),
        )
    }

    pub fn instructor(&self) -> Result<&str, ParseError> {
        // Not much I can do in terms of validation. Some people have very unique patterns in their
        // names.
        get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(
                "{}{}{}{}{}{}",
                INSTRUCTOR_TAG_PARTS[0],
                self.class_num + 1,
                INSTRUCTOR_TAG_PARTS[1],
                INSTRUCTOR_TAG_SERIES[self.class_num as usize],
                INSTRUCTOR_TAG_PARTS[2],
                self.group_num
            ),
        )
    }

    pub fn open_seats(&self) -> Result<Option<u32>, ParseError> {
        self.seats().map(|seats| seats.map(|seats| seats.0))
    }

    pub fn total_seats(&self) -> Result<Option<u32>, ParseError> {
        self.seats().map(|seats| seats.map(|seats| seats.1))
    }

    fn class_info(&self) -> Result<(&str, &str, &str), ParseError> {
        let class_info = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(
                "{}{}{}{}{}{}",
                CLASS_ID_TAG_PARTS[0],
                self.class_num + 1,
                CLASS_ID_TAG_PARTS[1],
                CLASS_ID_TAG_SERIES[self.class_num as usize],
                CLASS_ID_TAG_PARTS[2],
                self.group_num
            ),
        )?;

        // TODO: fix up
        let re = Regex::new(r"Class Nbr (\d+) - Section ([A-Z](?:\d?)+) ([A-Z]+)")
            .unwrap()
            .captures(class_info)
            .unwrap();
        Ok((
            re.get(1).unwrap().as_str(),
            re.get(2).unwrap().as_str(),
            re.get(3).unwrap().as_str(),
        ))
    }

    fn datetime(&self) -> Result<Option<(Vec<String>, String, String)>, ParseError> {
        get_node_from_id(
            self.dom,
            &format!(
                "{}{}{}{}{}{}",
                DATETIME_TAG_PARTS[0],
                self.class_num + 1,
                DATETIME_TAG_PARTS[1],
                DATETIME_TAG_SERIES[self.class_num as usize],
                DATETIME_TAG_PARTS[2],
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
                    Cow::Borrowed(_) => Err(ParseError::UnknownFormat),
                    Cow::Owned(value) => {
                        // TODO: cleanup
                        let re = Regex::new(
                            r"^((?:[A-Z][a-z]+\s)+)(\d?\d:\d\d(?:AM|PM)) to (\d?\d:\d\d(?:AM|PM))$",
                        )
                        .unwrap()
                        .captures(&value)
                        .unwrap();

                        // It's very inconvenient having to work with an owned `String` here, it
                        // seems like a fundamental design constraint of `tl`'s internals. The only
                        // other choice is to get the tag raw and try to regex the content, though
                        // it wouldn't be as reliable.
                        Ok(Some((
                            re.get(1)
                                .unwrap()
                                .as_str()
                                .split_whitespace()
                                .map(|string| string.to_owned())
                                .collect(), // Days of week (e.g. Wednesday)
                            re.get(2).unwrap().as_str().to_owned(), // Start time (e.g. 3:00PM)
                            re.get(3).unwrap().as_str().to_owned(), // End time (e.g. 4:00PM)
                        )))
                    }
                }
            },
        )
    }

    fn seats(&self) -> Result<Option<(u32, u32)>, ParseError> {
        let seats = get_text_from_id_without_sub_nodes(
            self.dom,
            &format!(
                "{}{}{}{}",
                SEATS_TAG_PARTS[0],
                self.class_num + 1,
                SEATS_TAG_PARTS[1],
                self.group_num
            ),
        )?;

        match seats {
            "Closed" => Ok(None),
            _ => {
                // TODO: fix up (constants and error types)
                let re = Regex::new(r"Open Seats (\d+) of (\d+)")
                    .unwrap()
                    .captures(seats)
                    .unwrap();

                Ok(Some((
                    re.get(1).unwrap().as_str().parse().unwrap(), // Open seats
                    re.get(2).unwrap().as_str().parse().unwrap(), // Total seats
                )))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClassType {
    Recitation,
    Lab,
    Lecture,
    Seminar,
}

impl FromStr for ClassType {
    // TODO: need to make error
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "REC" => ClassType::Recitation,
            "LAB" => ClassType::Lab,
            "LEC" => ClassType::Lecture,
            "SEM" => ClassType::Seminar,
            _ => return Err(()),
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
    // TODO: need to make error
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Sunday" => DayOfWeek::Sunday,
            "Monday" => DayOfWeek::Monday,
            "Tuesday" => DayOfWeek::Tuesday,
            "Wednesday" => DayOfWeek::Wednesday,
            "Thursday" => DayOfWeek::Thursday,
            "Friday" => DayOfWeek::Friday,
            "Saturday" => DayOfWeek::Saturday,
            _ => return Err(()),
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

fn get_text_from_id_without_sub_nodes<'a>(dom: &'a VDom, id: &str) -> Result<&'a str, ParseError> {
    match get_node_from_id(dom, id)?.inner_text(dom.parser()) {
        Cow::Borrowed(string) => Ok(string),
        // TODO: this is relying on implementation details, make it more explicit
        // If it's owned, that means the element had multiple sub-nodes, which shouldn't be the
        // case
        Cow::Owned(_) => Err(ParseError::UnknownFormat),
    }
}

fn get_node_from_id<'a>(dom: &'a VDom, id: &str) -> Result<&'a Node<'a>, ParseError> {
    Ok(dom
        .get_element_by_id(id)
        .ok_or(ParseError::MissingTag)?
        .get(dom.parser())
        // We know the element exists in the DOM because that's where we got it from
        .unwrap())
}

// TODO: add more specific errors and more context
#[derive(Debug, Error)]
pub enum ParseError {
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
    /// The most likely cause of this issue is the website being updated. Please leave an issue on
    /// GitHub if this error occurs.
    #[error("could not parse HTML due to unknown format")]
    UnknownFormat,
    /// HTML tag for class does not exist
    #[error("could not find tag for class in HTML")]
    MissingTag,
}
