#[derive(Debug, Clone)]
pub enum Course<'a> {
    Cse115,
    Raw(&'a str),
}

#[derive(Debug, Clone)]
pub enum Semester<'a> {
    Spring2023,
    Summer2023,
    Fall2023,
    Winter2023,
    Raw(&'a str),
}

#[derive(Debug, Clone)]
pub enum Career<'a> {
    Undergraduate,
    Graduate,
    Law,
    DentalMedicine,
    Medicine,
    Pharmacy,
    Raw(&'a str),
}

impl<'a> Career<'a> {
    pub fn id(self) -> &'a str {
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

impl<'a> Semester<'a> {
    pub fn id(self) -> &'a str {
        match self {
            Semester::Spring2023 => "2231",
            Semester::Summer2023 => "",
            Semester::Fall2023 => "",
            Semester::Winter2023 => "",
            Semester::Raw(id) => id,
        }
    }
}

impl<'a> Course<'a> {
    // NOTE: I don't think there is any way to automatically gather course -> course ids?
    pub fn id(self) -> &'a str {
        match self {
            Course::Cse115 => "004544",
            Course::Raw(id) => id,
        }
    }
}
