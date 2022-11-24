use std::str;

use quick_xml::{events::Event, Reader};
use thiserror::Error;
use tl::{Node, ParserOptions};

use crate::ClassGroup;

#[derive(Debug, Clone)]
pub struct ClassScheduleParser<'a> {
    bytes: &'a [u8],
    page: u32,
}

impl<'a> ClassScheduleParser<'a> {
    const CLASSES_PER_PAGE: u32 = 50;

    // First is the class group index ((page * 50) - 1)
    const SESSION_TAG: &str = "SSR_DER_CS_GRP_SESSION_CODE$215$${}";
    // First is class index in group (1-3)
    // Second is (294, 295, 296) depending on class index in group (1-3)
    // Third is the class group index ((page * 50) - 1)
    const CLASS_ID_TAG: &str = "SSR_CLSRCH_F_WK_SSR_CMPNT_DESCR_{}${}$${}";
    const CLASS_ID_TAG_PARTS: [u32; 3] = [294, 295, 296];
    // First is the class group index ((page * 50) - 1)
    const DATES_TAG: &str = "SSR_CLSRCH_F_WK_SSR_MTG_DT_LONG_1$88${}";
    // First is class index in group (1-3)
    // Second is (134, 135, 154) depending on class index in group (1-3)
    // Third is the class group index ((page * 50) - 1)
    const DAY_TIMES_TAG: &str = "SSR_CLSRCH_F_WK_SSR_MTG_SCHED_L_{}${}$${}";
    const DAY_TIMES_TAG_PARTS: [u32; 3] = [134, 135, 154];
    // First is class index in group (1-3)
    // Second is the class group index ((page * 50) - 1)
    const ROOM_TAG: &str = "SSR_CLSRCH_F_WK_SSR_MTG_LOC_LONG_{}${}";
    // First is class index in group (1-3)
    // Second is (86, 161, 162) depending on class index in group (1-3)
    // Third is the class group index ((page * 50) - 1)
    const INSTRUCTOR_TAG: &str = "SSR_CLSRCH_F_WK_SSR_INSTR_LONG_{}${}$${}";
    const INSTRUCTOR_TAG_PARTS: [u32; 3] = [86, 161, 162];
    // First is class index in group (1-3)
    // Second is the class group index ((page * 50) - 1)
    const SEATS_TAG: &str = "SSR_CLSRCH_F_WK_SSR_DESCR50_{}${}";

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

    // Search for win80divDAYS_TIMES$0 and increment 0
    // Search for TERM_VAL_TBL_DESCR to get "Spring 2023"
    fn parse_html(&self, bytes: &[u8]) -> Result<(), ParseError> {
        let string = str::from_utf8(bytes)?;
        // TODO: consider enabling HTML tracking
        let dom = tl::parse(string, ParserOptions::default())?;

        let first_handle = dom.children().get(0).ok_or(ParseError::EmptyHtml)?;
        // handle will always exist since we pull it straight from the DOM
        let first_node = first_handle.get(dom.parser()).unwrap();
        let id = match first_node {
            Node::Tag(tag) => {
                let bytes = tag.attributes().id().ok_or(ParseError::UnknownFormat)?;
                let full_id = str::from_utf8(bytes.as_bytes())?;
                id_from_full_id(full_id)
            }
            _ => Err(ParseError::UnknownFormat),
        };

        // Every page contains the bytes of the previous pages
        let last_class_index = (self.page * Self::CLASSES_PER_PAGE) - 1;
        for i in 0..last_class_index {
            let element = dom.get_element_by_id(&*format!(Self::SESSION_TAG, i));
        }
        todo!()
    }
}

fn id_from_full_id(string: &str) -> Result<u32, ParseError> {
    let start = string.find("win").ok_or(ParseError::UnknownFormat)?;
    let end = string
        .find("divPSPAGECONTAINER")
        .ok_or(ParseError::UnknownFormat)?;
    u32::from_str_radix(&string[start..end], 10).or(Err(ParseError::UnknownFormat))
}

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
}
