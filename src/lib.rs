use chrono::NaiveDateTime;

mod parser;
mod session;

// TODO: Every lecture is paired with every possible combo of recs/labs, I can simplify this
#[derive(Debug, Clone)]
pub struct ClassGroup {
    classes: Vec<Class>,
    // TODO: get `win6divUB_SR_FL_WRK_HTMLAREA1$5` and retrieve sub-node
    // open: bool,
    session: String,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Class {
    r#type: ClassType,
    class_id: u32,
    section: String,
    start_day: NaiveDateTime,
    end_day: NaiveDateTime,
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
