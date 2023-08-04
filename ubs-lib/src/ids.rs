use std::str::FromStr;

// TODO: do more validation in FromStr

#[derive(Debug, Clone)]
pub enum Course {
    Cse115,
    Raw(String),
}

#[derive(Debug, Clone)]
pub enum Semester {
    Spring2023,
    Summer2023,
    Fall2023,
    Winter2023,
    Raw(String),
}

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
    pub fn career(&self) -> Option<Career> {
        match self {
            Course::Cse115 => Some(Career::Undergraduate),
            // in this case it's highly dependent on the course to determine the career
            Course::Raw(_) => None,
        }
    }

    // NOTE: I don't think there is any way to automatically gather course -> course ids?
    pub(crate) fn id(&self) -> &str {
        match self {
            Course::Cse115 => "004544",
            Course::Raw(id) => id,
        }
    }
}

impl FromStr for Course {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*normalize(s) {
            "CSE115" => Ok(Course::Cse115),
            // TODO: valid course id is 6 characters and an integer
            _ => Err(()),
        }
    }
}

impl Semester {
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*normalize(s) {
            "SPRING2023" => Ok(Semester::Spring2023),
            "SUMMER2023" => Ok(Semester::Summer2023),
            "FALL2023" => Ok(Semester::Fall2023),
            "WINTER2023" => Ok(Semester::Winter2023),
            _ => Err(()),
        }
    }
}

impl Career {
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*normalize(s) {
            "UNDERGRADUATE" => Ok(Career::Undergraduate),
            "GRADUATE" => Ok(Career::Graduate),
            "LAW" => Ok(Career::Law),
            "DENTALMEDICINE" => Ok(Career::DentalMedicine),
            "MEDICINE" => Ok(Career::Medicine),
            "PHARMACY" => Ok(Career::Pharmacy),
            _ => Err(()),
        }
    }
}

// TODO: more filtering
pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}
