use std::{borrow::Cow, str, sync::Arc};

use chrono::NaiveDateTime;
use quick_xml::{events::Event, Reader};
use thiserror::Error;
use tl::{ParserOptions, VDom};

const CLASSES_PER_PAGE: u32 = 50;
const CLASSES_PER_GROUP: u32 = 3;

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
const DATES_TAG_PARTS: [&str; 1] = ["SSR_CLSRCH_F_WK_SSR_MTG_DT_LONG_1$88$"];
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

#[derive(Debug, Clone)]
pub struct ClassScheduleParser<'a> {
    bytes: &'a [u8],
    page: u32,
}

impl<'a> ClassScheduleParser<'a> {
    // TODO: see if I can extract the page from the data
    // Page starts from 0
    pub fn new(bytes: &'a [u8], page: u32) -> Self {
        Self { bytes, page }
    }

    pub async fn class_iter(&self) -> ClassGroup {
        todo!()
    }

    fn parse_xml(&self) -> Result<&[u8], ParseError> {
        let mut reader = Reader::from_reader(self.bytes);
        loop {
            match reader.read_event()? {
                Event::Start(event) => {
                    if event.name().as_ref() == b"FIELD" {
                        if let Some(attribute) = event.try_get_attribute("id")? {
                            // First page has different XML than the rest
                            if self.page == 0
                                && &*attribute.value == b"win3divSSR_CLSRCH_F_WK_SSR_GROUP_BOX_1"
                                || &*attribute.value == b"divPAGECONTAINER_TGT"
                            {
                                // TODO: does this get what's inside the CData padding?
                                let range = reader.read_to_end(event.to_end().name())?;
                                return Ok(&self.bytes[range]);
                            }
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Err(ParseError::FieldMissing)
    }

    // Search for TERM_VAL_TBL_DESCR to get "Spring 2023"
    // TODO: I can return an iterator over each class group rather than collecting
    fn parse_html(&self) -> Result<Vec<ClassGroup<'a>>, ParseError> {
        let string = str::from_utf8(self.bytes)?;
        // TODO: consider enabling tracking for perf and the Arc isn't too pretty
        let dom = Arc::new(tl::parse(string, ParserOptions::default())?);

        // Every page contains the bytes of the previous pages
        let first_class_index = self.page.saturating_sub(1) * CLASSES_PER_PAGE;
        let last_class_index = (self.page * CLASSES_PER_PAGE) - 1;
        Ok((first_class_index..last_class_index)
            .map(|group_num| ClassGroup {
                classes: (0..CLASSES_PER_GROUP)
                    .map(|class_num| Class {
                        dom: dom.clone(),
                        class_num,
                        group_num,
                    })
                    .collect(),
                dom: dom.clone(),
                group_num,
            })
            .collect())
    }
}

// TODO: Every lecture is paired with every possible combo of recs/labs, I can simplify this
#[derive(Debug, Clone)]
pub struct ClassGroup<'a> {
    dom: Arc<VDom<'a>>,
    classes: Vec<Class<'a>>,
    group_num: u32,
}

impl<'a> ClassGroup<'a> {
    pub fn classes(&self) -> &[Class<'a>] {
        &self.classes
    }

    // TODO: get `win6divUB_SR_FL_WRK_HTMLAREA1$5` and retrieve sub-node
    pub fn is_open(&self) -> Result<bool, ParseError> {
        todo!()
    }

    pub fn session(&self) -> Result<String, ParseError> {
        let session = get_text_from_id(
            &self.dom,
            &*format!("{}{}", SESSION_TAG_PARTS[0], self.group_num),
        )?;
        todo!()
    }

    pub fn start_date(&self) -> Result<NaiveDateTime, ParseError> {
        todo!()
    }

    pub fn end_date(&self) -> Result<NaiveDateTime, ParseError> {
        todo!()
    }

    pub fn dates(&self) -> Result<&str, ParseError> {
        get_text_from_id(
            &self.dom,
            &*format!("{}{}", DATES_TAG_PARTS[0], self.group_num),
        )
    }
}

// TODO: empty text will equal `&nbsp;`
#[derive(Debug, Clone)]
pub struct Class<'a> {
    dom: Arc<VDom<'a>>,
    class_num: u32,
    group_num: u32,
}

impl Class<'_> {
    pub fn r#type(&self) -> Result<ClassType, ParseError> {
        todo!()
    }

    pub fn class_id(&self) -> Result<u32, ParseError> {
        todo!()
    }

    pub fn section(&self) -> Result<String, ParseError> {
        todo!()
    }

    pub fn start_day(&self) -> Result<NaiveDateTime, ParseError> {
        todo!()
    }

    pub fn end_day(&self) -> Result<NaiveDateTime, ParseError> {
        todo!()
    }

    pub fn room(&self) -> Result<String, ParseError> {
        let room = get_text_from_id(
            &self.dom,
            &*format!(
                "{}{}{}{}",
                ROOM_TAG_PARTS[0],
                self.class_num + 1,
                ROOM_TAG_PARTS[1],
                self.group_num
            ),
        )?;
        todo!()
    }

    pub fn instructor(&self) -> Result<String, ParseError> {
        let instructor = get_text_from_id(
            &self.dom,
            &*format!(
                "{}{}{}{}{}{}",
                INSTRUCTOR_TAG_PARTS[0],
                self.class_num + 1,
                INSTRUCTOR_TAG_PARTS[1],
                INSTRUCTOR_TAG_SERIES[self.class_num as usize],
                INSTRUCTOR_TAG_PARTS[2],
                self.group_num
            ),
        )?;
        todo!()
    }

    pub fn open_seats(&self) -> Result<u32, ParseError> {
        todo!()
    }

    pub fn closed_seats(&self) -> Result<u32, ParseError> {
        todo!()
    }

    fn class_num(&self) -> Result<&str, ParseError> {
        get_text_from_id(
            &self.dom,
            &*format!(
                "{}{}{}{}{}{}",
                CLASS_ID_TAG_PARTS[0],
                self.class_num + 1,
                CLASS_ID_TAG_PARTS[1],
                CLASS_ID_TAG_PARTS[self.class_num as usize],
                CLASS_ID_TAG_PARTS[2],
                self.group_num
            ),
        )
    }

    fn datetime(&self) -> Result<Option<&str>, ParseError> {
        get_text_from_id(
            &self.dom,
            &*format!(
                "{}{}{}{}{}{}",
                DATETIME_TAG_PARTS[0],
                self.class_num + 1,
                DATETIME_TAG_PARTS[1],
                DATETIME_TAG_SERIES[self.class_num as usize],
                DATETIME_TAG_SERIES[2],
                self.group_num
            ),
        )
        // If the tag is missing it could mean `Time Conflict` is being displayed. In that
        // case, skip it and label the datetime as non-existent
        .map_or_else(
            |err| match err {
                ParseError::MissingTag => Ok(None),
                _ => Err(err),
            },
            |value| Ok(Some(value)),
        )
    }

    fn seats(&self) -> Result<&str, ParseError> {
        get_text_from_id(
            &self.dom,
            &*format!(
                "{}{}{}{}",
                SEATS_TAG_PARTS[0],
                self.class_num + 1,
                SEATS_TAG_PARTS[1],
                self.group_num
            ),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClassType {
    Recitation,
    Lab,
    Lecture,
    Seminar,
}

fn get_text_from_id<'a>(dom: &'a VDom, id: &str) -> Result<&'a str, ParseError> {
    let text = dom
        .get_element_by_id(id)
        .ok_or(ParseError::MissingTag)?
        .get(dom.parser())
        // We know the element exists in the DOM because that's where we got it from
        .unwrap()
        .inner_text(dom.parser());
    match text {
        Cow::Borrowed(string) => Ok(string),
        // TODO: this is relying on implementation details, make it more explicit
        // If it's owned, that means the element had multiple sub-nodes, which shouldn't be the
        // case
        Cow::Owned(_) => Err(ParseError::UnknownFormat),
    }
}

// TODO: add more specific errors and more context
#[derive(Debug, Error)]
pub enum ParseError {
    /// XML is not in a valid format.
    #[error("could not parse XML due to invalid format")]
    InvalidXmlFormat(#[from] quick_xml::Error),
    /// XML is missing the correct fields.
    #[error("could not find correct field in XML")]
    FieldMissing,
    /// HTML is not valid Utf-8.
    #[error("could not parse HTML due to invalid Utf-8 encoding")]
    HtmlInvalidUtf8(#[from] str::Utf8Error),
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
