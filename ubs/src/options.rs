use clap::{ValueEnum, Parser};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Options {
    /// Course name and number to query (e.g. CSE115)
    pub course: String,
    /// Semester to query
    #[clap(value_enum)]
    pub semester: Semester,
    /// Year to query (e.g. 2023)
    pub year: u32,
    /// Format to output data
    #[clap(long, value_enum, default_value_t = DataFormat::Json)]
    pub format: DataFormat,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Semester {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DataFormat {
    Json
}
