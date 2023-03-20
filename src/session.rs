use std::sync::Arc;

use cookie::Cookie;
use futures::{stream, StreamExt, TryFutureExt, TryStream, TryStreamExt};
use hyper::{
    body::{self, Bytes},
    client::{connect::Connect, ResponseFuture},
    header, Body, Client, HeaderMap, Request,
};
use thiserror::Error;

use crate::course::Course;

const USER_AGENT: &str = "ubs";

// TODO: remove excess queries from url
const FAKE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_MAIN_FL.GBL?Page=SSR_CLSRCH_MAIN_FL&pslnkid=CS_S201605302223124733554248&ICAJAXTrf=true&ICAJAX=1&ICMDTarget=start&ICPanelControlStyle=%20pst_side1-fixed%20pst_panel-mode%20";
const FAKE2_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_ES_FL.GBL?Page=SSR_CLSRCH_ES_FL&SEARCH_GROUP=SSR_CLASS_SEARCH_LFF&SEARCH_TEXT=gly%20105&ES_INST=UBFLO&ES_STRM=2231&ES_ADV=N&INVOKE_SEARCHAGAIN=PTSF_GBLSRCH_FLUID";
const PAGE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_3/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CRSE_INFO_FL.GBL?Page=SSR_CRSE_INFO_FL&Page=SSR_CS_WRAP_FL&ACAD_CAREER=UGRD&CRSE_ID=004544&CRSE_OFFER_NBR=1&INSTITUTION=UBFLO&STRM=2231";
// STRM = semester id
// CRSE = course id
// ACAD_CAREER = undergrad/grad/law/etc.

const TOKEN1_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT";
const TOKEN2_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT&";
const TOKEN_COOKIE_NAME: &str = "psprd-8083-PORTAL-PSJSESSIONID";

pub struct Session<T> {
    client: Client<T, Body>,
    token: Arc<str>,
}

impl<T> Session<T> {
    pub fn new(client: Client<T, Body>, token: &Token) -> Self {
        Self {
            client,
            token: Arc::from(token.to_string_cookie()),
        }
    }
}

impl<T> Session<T>
where
    T: Connect + Clone + Send + Sync + 'static,
{
    // TODO: allow choosing semester, note that this may be another unique ID per, just like courses
    // TODO: AsRef<str>
    pub fn schedule_iter<'a>(
        &self,
        course: Course,
    ) -> impl TryStream<Ok = Bytes, Error = SessionError> + 'a {
        let client = self.client.clone();
        let token = self.token.clone();
        stream::iter(1..)
            .then(move |page_num| {
                // Cloning `client` and `token` above is to avoid having the closure live as long
                // as `self`. Cloning again is necessary because new ownership is needed for a new
                // step in the iteration.
                let client = client.clone();
                let token = token.clone();
                // `async move` doesn't implement `Unpin`, thus it is necessary to manually pin it.
                // TODO: simplify this
                Box::pin(async move { Ok(Self::get_page(&client, &token, page_num).await?.await?) })
            })
            .and_then(|response| Box::pin(body::to_bytes(response.into_body()).err_into()))
    }

    // TODO: you MUST go page-by-page, otherwise it won't return the correct result?
    async fn get_page(
        client: &Client<T, Body>,
        token: &str,
        page_num: u32,
    ) -> Result<ResponseFuture, SessionError> {
        loop {
            match page_num {
                1 => {
                    // TODO: fix boilerplate
                    client
                        .request(
                            Request::builder()
                                .uri(FAKE1_URL)
                                .header(header::COOKIE, token)
                                .body(Body::empty())?,
                        )
                        .await?;
                    client
                        .request(
                            Request::builder()
                                .uri(FAKE2_URL)
                                .header(header::COOKIE, token)
                                .body(Body::empty())?,
                        )
                        .await?;
                    let page = client.request(
                        Request::builder()
                            .uri(PAGE1_URL)
                            .header(header::USER_AGENT, USER_AGENT)
                            .header(header::COOKIE, token)
                            .header(header::COOKIE, "HttpOnly")
                            .header(header::COOKIE, "Path=/")
                            .body(Body::empty())?,
                    );
                    // TODO: do I need to send the fake result here (with ICState=2) for the next
                    // pages to load?
                    return Ok(page);
                }
                _ => {
                    // The second page has an `ICState` of 3.
                    let page_num = page_num + 1;
                    // TODO: Multiple things to know about >1 pages:
                    //  1. Each page holds 50 groups max.
                    //  2. They are all POST requests with a slightly differing body (ICState and
                    //     ICAction).
                    //  3. How I currently have it set up is not how it may actually work. Meaning,
                    //     I know there is a second "phony" request, though invoking it does not
                    //     seem to enable the next page to return the correct result. I'm either
                    //     missing some minute detail in the request or I need to send more phony
                    //     requests prior.
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token(Cookie<'static>);

impl Token {
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

        Ok(Self(
            Token::token_cookie(response.headers())
                .ok_or(SessionError::TokenCookieNotFound)?
                .into_owned(),
        ))
    }

    fn to_string_cookie(&self) -> String {
        self.0.to_string()
    }

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

/// Represents errors that can occur retrieving course data.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An argument to build the HTTP request was invalid.
    /// See more [here](https://docs.rs/http/0.2.8/http/request/struct.Builder.html#errors)
    #[error("an argument while building an HTTP request was invalid")]
    MalformedHttpArgs(#[from] hyper::http::Error),
    /// Failed to send HTTP request.
    #[error("failed to send HTTP request")]
    HttpRequestFailed(#[from] hyper::Error),
    /// Attempted to parse a cookie with an invalid format.
    #[error("could not parse cookie with an invalid format")]
    MalformedCookie(#[from] cookie::ParseError),
    // TODO: provide cookie parsing errors
    /// Could not find or parse the token cookie.
    #[error("could not find or parse the token cookie")]
    TokenCookieNotFound,
}
