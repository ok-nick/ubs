// XML PARSING:
// Find <PAGE>
// Find <FIELD> with id `divPAGECONTAINER_TGT`
// Use CDATA event to parse value
//
// Use `serde` for now. If I want to make it more performant, I can use streams
//
// HTML PARSING:
// Search for Id `win80divDAYS_TIMES$0` whilst incrementing 0

use chrono::NaiveDateTime;

mod parser;
mod session;

// TODO: Every lecture is paired with every possible combo of recs/labs, I can simplify this
#[derive(Debug, Clone)]
pub struct ClassGroup {
    classes: Vec<Class>,
    open: bool,
}

#[derive(Debug, Clone)]
pub struct Class {
    r#type: ClassType,
    class_id: u32,
    section: String,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    room: String,
    instructor: String,
    open_seats: u32,
    total_seats: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum ClassType {
    Recitation,
    Lab,
    Lecture,
    Seminar,
}

// TODO: allow choosing semester
// TODO: expand pages
pub fn list_classes(course_id: u32) -> Vec<ClassGroup> {
    // split into web api and parsing
    todo!()
}
