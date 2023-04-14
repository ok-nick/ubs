use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Options {
    /// Course name and number to query (e.g. CSE115, GLY105) or course id (e.g. 004544)
    pub course: String,
    /// Semester to query (e.g. Spring2023, Summer2023, Fall2023, Winter2023) or semester id (e.g. 2231)
    pub semester: String,
    /// Career to query (e.g Undergraduate, Graduate, Law, DentalMedicine, Medicine, Pharmacy) or
    /// career id (e.g. SDM)
    pub career: String,
    /// Format to output data
    #[clap(long, value_enum, default_value_t = DataFormat::Json)]
    pub format: DataFormat,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DataFormat {
    Json,
}
