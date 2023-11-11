//! Low-level access to the host connection.

use std::sync::Arc;

use cookie::Cookie;
use futures::{stream, TryFutureExt, TryStream, TryStreamExt};
use hyper::{
    body::{self, Bytes},
    client::connect::Connect,
    header, Body, Client, HeaderMap, Request, Response,
};
use thiserror::Error;

use crate::{
    ids::{Course, Semester},
    Career,
};

const USER_AGENT: &str = "ubs";

// TODO: remove excess queries from url
const FAKE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_MAIN_FL.GBL?Page=SSR_CLSRCH_MAIN_FL&pslnkid=CS_S201605302223124733554248&ICAJAXTrf=true&ICAJAX=1&ICMDTarget=start&ICPanelControlStyle=%20pst_side1-fixed%20pst_panel-mode%20";
macro_rules! FAKE2_URL {
    () => { "https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_ES_FL.GBL?SEARCH_GROUP=SSR_CLASS_SEARCH_LFF&SEARCH_TEXT=placeholder&ES_INST=UBFLO&ES_STRM={}" };
}
macro_rules! PAGE1_URL {
    () => { "https://www.pub.hub.buffalo.edu/psc/csprdpub_3/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CRSE_INFO_FL.GBL?CRSE_OFFER_NBR=1&INSTITUTION=UBFLO&CRSE_ID={}&STRM={}&ACAD_CAREER={}" };
}

const TOKEN1_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT";
const TOKEN2_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT&";
const TOKEN_COOKIE_NAME: &str = "psprd-8083-PORTAL-PSJSESSIONID";

/// Information about the course query.
#[derive(Debug, Clone)]
pub struct Query {
    course: Course,
    semester: Semester,
    career: Career,
}

impl Query {
    /// Construct a new [`Query`](Query).
    pub fn new(course: Course, semester: Semester, career: Career) -> Self {
        Self {
            course,
            semester,
            career,
        }
    }
}

#[derive(Debug)]
struct ScheduleIterState<T> {
    page_num: u32,
    query: Query,
    client: Client<T, Body>,
    token: Token,
}

/// Manages the session to the host server.
#[derive(Debug)]
pub struct Session<T> {
    client: Client<T, Body>,
    token: Token,
}

impl<T> Session<T> {
    /// Construct a new [`Session`](Session).
    pub fn new(client: Client<T, Body>, token: Token) -> Self {
        Self { client, token }
    }
}

impl<T> Session<T>
where
    T: Connect + Clone + Send + Sync + 'static,
{
    /// Initializes the session.
    ///
    /// This only needs to be called once before the schedule is iterated.
    // TODO: I believe this only needs to be called once. When the token expires, a new token is needed.
    // I also believe the semester can be any semester, as long as it is valid. So in the future it can be
    // replaced if the latest semesters are auto found
    pub async fn initialize(&self, semester: &Semester) -> Result<(), SessionError> {
        self.client
            .request(
                Request::builder()
                    .uri(FAKE1_URL)
                    .header(header::COOKIE, self.token.as_str())
                    .body(Body::empty())?,
            )
            .await?;
        self.client
            .request(
                Request::builder()
                    .uri(format!(FAKE2_URL!(), semester.id()))
                    .header(header::COOKIE, self.token.as_str())
                    .body(Body::empty())?,
            )
            .await?;

        Ok(())
    }

    /// Iterate over pages of schedules with the specified [`Query`](Query).
    pub fn schedule_iter(&self, query: Query) -> impl TryStream<Ok = Bytes, Error = SessionError> {
        stream::unfold(
            ScheduleIterState {
                page_num: 1,
                query,
                // both Arc, so it's cheap
                client: self.client.clone(),
                token: self.token.clone(),
            },
            |mut state| {
                Box::pin(async move {
                    Self::get_page(&state.client, &state.token, &state.query, state.page_num)
                        .await
                        .transpose()
                        .map(|response| {
                            state.page_num += 1;
                            (response, state)
                        })
                })
            },
        )
        .and_then(|response| Box::pin(body::to_bytes(response.into_body()).err_into()))
    }

    // TODO: you MUST go page-by-page, otherwise it won't return the correct result?
    /// Get specific page for query.
    ///
    /// Note that this must be called incrementally, page-by-page.
    async fn get_page(
        client: &Client<T, Body>,
        token: &Token,
        query: &Query,
        page_num: u32,
    ) -> Result<Option<Response<Body>>, SessionError> {
        #[allow(clippy::never_loop)] // TODO: temp
        loop {
            match page_num {
                1 => {
                    let page = client
                        .request(
                            Request::builder()
                                .uri(format!(
                                    PAGE1_URL!(),
                                    query.course.id(),
                                    query.semester.id(),
                                    query.career.id()
                                ))
                                .header(header::USER_AGENT, USER_AGENT)
                                .header(header::COOKIE, token.as_str())
                                .header(header::COOKIE, "HttpOnly")
                                .header(header::COOKIE, "Path=/")
                                .body(Body::empty())?,
                        )
                        .await?;
                    // TODO: do I need to send the fake result here (with ICState=2) for the next
                    // pages to load?
                    return Ok(Some(page));
                }
                _ => {
                    // The second page has an `ICState` of 3.
                    let _page_num = page_num + 1;
                    // TODO: Multiple things to know about >1 pages:
                    //  1. Each page holds 50 groups max.
                    //  2. They are all POST requests with a slightly differing body (ICState and
                    //     ICAction).
                    //  3. How I currently have it set up is not how it may actually work. Meaning,
                    //     I know there is a second "phony" request, though invoking it does not
                    //     seem to enable the next page to return the correct result. I'm either
                    //     missing some minute detail in the request or I need to send more phony
                    //     requests prior.
                    return Ok(None);
                }
            }
        }
    }
}

/// Contains a unique identifier for the current session.
#[derive(Debug, Clone)]
pub struct Token(Arc<str>);

impl Token {
    /// Construct a new [`Token`](Token) with the specified [`Client`](Client).
    pub async fn new<T>(client: &Client<T, Body>) -> Result<Self, SessionError>
    where
        T: Connect + Clone + Send + Sync + 'static,
    {
        // TODO: need to follow redirect returned by this URL, two ways to do this:
        //  1. Make a loop and do some magic, hopefully it works.
        //  2. Go to 1st redirect.
        //  3. Just use reqwest.
        let response = client
            .request(
                Request::builder()
                    .uri(TOKEN1_URL)
                    .header(header::USER_AGENT, USER_AGENT)
                    // TODO: may or may not need the httponly and path cookies
                    .body(Body::empty())?,
            )
            .await?;
        let response = client
            .request(
                Request::builder()
                    .uri(TOKEN2_URL)
                    .header(header::USER_AGENT, USER_AGENT)
                    .header(
                        header::COOKIE,
                        Token::token_cookie(response.headers())
                            .ok_or(SessionError::TokenCookieNotFound)?
                            .to_string(),
                    )
                    // TODO: may or may not need the httponly and path cookies
                    .body(Body::empty())?,
            )
            .await?;

        Ok(Self(Arc::from(
            Token::token_cookie(response.headers())
                .ok_or(SessionError::TokenCookieNotFound)?
                .to_string(),
        )))
    }

    /// Convert the token to its string form.
    fn as_str(&self) -> &str {
        &self.0
    }

    /// Fetch the [`Cookie`](Cookie) object from the specified headers.
    fn token_cookie(headers: &HeaderMap) -> Option<Cookie<'_>> {
        headers
            .get_all(header::SET_COOKIE)
            .iter()
            // TODO: collect errors and return them if no cookie was found
            // If it can't be parsed then skip it
            .filter_map(|string| {
                string
                    .to_str()
                    .ok()
                    .and_then(|raw_cookie| Cookie::parse(raw_cookie).ok())
            })
            .find(|cookie| cookie.name() == TOKEN_COOKIE_NAME)
    }
}

/// Error while fetching course data.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An argument to build the HTTP request was invalid.
    /// See more [here](https://docs.rs/http/0.2.8/http/request/struct.Builder.html#errors)
    #[error("an argument while building an HTTP request was invalid")]
    MalformedHttpArgs(#[from] hyper::http::Error),
    /// Failed to send HTTP request.
    #[error(transparent)]
    HttpRequestFailed(#[from] hyper::Error),
    /// Attempt to parse a cookie with an invalid format.
    #[error("could not parse cookie with an invalid format")]
    MalformedCookie(#[from] cookie::ParseError),
    // TODO: provide cookie parsing errors
    /// Could not find or parse the token cookie.
    #[error("could not find or parse the token cookie")]
    TokenCookieNotFound,
}
