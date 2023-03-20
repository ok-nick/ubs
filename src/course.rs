pub enum Course {
    Cse115,
}

impl Course {
    pub fn id(self) -> &'static str {
        match self {
            Course::Cse115 => "004544",
        }
    }
}
