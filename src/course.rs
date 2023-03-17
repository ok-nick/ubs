pub enum Course {
    CSE115,
}

impl Course {
    pub fn id(self) -> &'static str {
        match self {
            Course::CSE115 => "004544",
        }
    }
}
