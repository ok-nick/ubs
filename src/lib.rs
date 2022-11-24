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
    session: String,
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
