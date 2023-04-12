use std::str::FromStr;

pub enum Course {
    Cse115,
}

impl Course {
    // NOTE: I don't think there is any way to automatically gather course -> course ids?
    pub fn id(self) -> &'static str {
        match self {
            Course::Cse115 => "004544",
        }
    }
}

impl FromStr for Course {
    // TODO:
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CSE115" => Ok(Course::Cse115),
            _ => Err(()),
        }
    }
}
