use convert_case::{Case, Casing};
use csv::Reader;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::{collections::HashMap, env, fs, io, path::Path};

const COURSES_PATH: &str = "data/courses.csv";

// in case more information is added to each course (ex: is it a pathway?)
#[derive(Debug, Clone)]
struct Course {
    // id: String,
    career: String,
    name: String,
}

// TODO: split this function into many smaller functions,
// good ref: https://github.com/baptiste0928/rosetta/blob/main/rosetta-build/src/gen.rs
fn generate_tokens(courses: HashMap<String, Course>) -> io::Result<TokenStream> {
    let ids = courses.keys();
    let num_courses = courses.len();

    let names1 = courses.values().map(|course| {
        Ident::new(
            course.name.to_case(Case::Pascal).as_str(),
            Span::call_site(),
        )
    });
    let names2 = names1.clone();
    let names3 = names1.clone();
    let names4 = names1.clone();
    let names5 = courses
        .values()
        .map(|course| course.name.to_ascii_uppercase());
    let names6 = names1.clone();

    let careers = courses
        .values()
        .map(|course| match course.career.as_str() {
            "UGRD" => "Undergraduate",
            _ => "",
        })
        // TODO: use format_ident!
        .map(|career| Ident::new(career, Span::call_site()));

    // TODO: add this as module-level doc
    // Mappings of course/semester/career to internal ids.
    Ok(quote!(
        use std::str::FromStr;

        use thiserror::Error;

        /// An enum of available courses in the catalog.
        ///
        /// If a course is missing, manually specify its id with [`Course::Raw`](Course::Raw) and
        /// consider sending a PR adding that mapping.
        #[derive(Debug, Clone)]
        #[non_exhaustive]
        pub enum Course {
            Raw(String),
            #(#names1),*
        }

        impl Course {
            pub const ALL: [Course; #num_courses] = [#(Course::#names4),*];

            /// Infer the career from the course.
            ///
            /// Note that this isn't always possible because a mapping does not yet exist. In
            /// that case, consider sending a PR adding the mapping.
            pub fn career(&self) -> Option<Career> {
                match self {
                    #(Course::#names2 => Some(Career::#careers),)*
                    // in this case it's highly dependent on the course to determine the career
                    Course::Raw(_) => None,
                }
            }

            /// Internal id of the course.
            pub fn id(&self) -> &str {
                match self {
                    #(Course::#names3 => #ids,)*
                    Course::Raw(id) => id,
                }
            }
        }

        // TODO: rust-phf could be more optimal
        // https://github.com/rust-phf/rust-phf
        impl FromStr for Course {
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match &*normalize(s) {
                    #(#names5 => Ok(Course::#names6),)*
                    _ => Err(ParseIdError::InvalidId {
                        id: "Course".to_owned(),
                        given: s.to_owned(),
                    }),
                }
            }
        }

        /// An enum of available semesters in the catalog.
        ///
        /// If a semester is missing, manually specify its id with [`Semester::Raw`](Semester::Raw) and
        /// consider sending a PR adding that mapping.
        // TODO: auto-gen semesters?
        #[derive(Debug, Clone)]
        #[non_exhaustive]
        pub enum Semester {
            Spring2023,
            Summer2023,
            Fall2023,
            Winter2024,
            Spring2024,
            Raw(String),
        }

        impl Semester {
            /// Internal id of the semester.
            pub fn id(&self) -> &str {
                match self {
                    Semester::Spring2023 => "2231",
                    Semester::Summer2023 => "2236",
                    Semester::Fall2023 => "2239",
                    Semester::Winter2024 => "2240",
                    Semester::Spring2024 => "2241",
                    Semester::Raw(id) => id,
                }
            }
        }

        impl FromStr for Semester {
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match &*normalize(s) {
                    "SPRING2023" => Ok(Semester::Spring2023),
                    "SUMMER2023" => Ok(Semester::Summer2023),
                    "FALL2023" => Ok(Semester::Fall2023),
                    "WINTER2024" => Ok(Semester::Winter2024),
                    "SPRING2024" => Ok(Semester::Spring2024),
                    _ => Err(ParseIdError::InvalidId {
                        id: "Semester".to_owned(),
                        given: s.to_owned(),
                    }),
                }
            }
        }

        /// An enum of available careers in the catalog.
        ///
        /// If a career is missing, manually specify its id with [`Career::Raw`](Career::Raw) and
        /// consider sending a PR adding that mapping.
        ///
        /// Specifying the career is an internal implementation detail exposed by the backend
        /// network API. It doesn't make much sense to have, but nevertheless, it is required.
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

        impl Career {
            /// Internal id of the career.
            pub fn id(&self) -> &str {
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
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match &*normalize(s) {
                    "UNDERGRADUATE" => Ok(Career::Undergraduate),
                    "GRADUATE" => Ok(Career::Graduate),
                    "LAW" => Ok(Career::Law),
                    "DENTALMEDICINE" => Ok(Career::DentalMedicine),
                    "MEDICINE" => Ok(Career::Medicine),
                    "PHARMACY" => Ok(Career::Pharmacy),
                    _ => Err(ParseIdError::InvalidId {
                        id: "Career".to_owned(),
                        given: s.to_owned(),
                    }),
                }
            }
        }

        /// Normalize the input string for use in [`FromStr`](std::str:FromStr) implementations.
        fn normalize(s: &str) -> String {
            s.chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_ascii_uppercase())
                .collect()
        }

        /// Error when parsing id.
        #[derive(Debug, Error)]
        pub enum ParseIdError {
            /// Specified id could not be converted to enum.
            ///
            /// Considering using the `Raw` variant for specifying raw ids.
            #[error("`{given}` is an invalid `{id}``")]
            InvalidId { id: String, given: String },
        }
    ))
}

fn main() {
    println!("cargo:rerun-if-changed={COURSES_PATH}");

    let mut courses = HashMap::new();

    let mut reader = Reader::from_path(COURSES_PATH).unwrap();
    for result in reader.records() {
        let record = result.unwrap();
        courses.insert(
            record[0].to_owned(),
            Course {
                // id: record[0].to_owned(),
                career: record[1].to_owned(),
                name: record[2].to_owned(),
            },
        );
    }

    let tokens = generate_tokens(courses).unwrap();
    let syntax_tree = syn::parse2(tokens).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("ids.rs");
    fs::write(out_path, formatted).unwrap();
}
