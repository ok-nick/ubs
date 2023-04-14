use ubs_lib::{Career, Course, Semester};

pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

pub fn find_course(s: &str) -> Course {
    match s {
        "CSE115" => Course::Cse115,
        // TODO: valid course id is 6 characters and an integer
        _ => Course::Raw(s),
    }
}

pub fn find_semester(s: &str) -> Semester {
    match s {
        "SPRING2023" => Semester::Spring2023,
        "SUMMER2023" => Semester::Summer2023,
        "FALL2023" => Semester::Fall2023,
        "WINTER2023" => Semester::Winter2023,
        _ => Semester::Raw(s),
    }
}

pub fn find_career(s: &str) -> Career {
    match s {
        "UNDERGRADUATE" => Career::Undergraduate,
        "GRADUATE" => Career::Graduate,
        "LAW" => Career::Law,
        "DENTALMEDICINE" => Career::DentalMedicine,
        "MEDICINE" => Career::Medicine,
        "PHARMACY" => Career::Pharmacy,
        _ => Career::Raw(s),
    }
}
