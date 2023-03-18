use std::sync::Arc;

use cookie::Cookie;
use futures::{stream, StreamExt, TryFutureExt, TryStream, TryStreamExt};
use hyper::{
    body::{self, Bytes},
    client::{connect::Connect, ResponseFuture},
    header, Body, Client, HeaderMap, Request, Uri,
};
use thiserror::Error;

use crate::course::Course;

// TODO: remove excess queries from url
const FAKE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_MAIN_FL.GBL?Page=SSR_CLSRCH_MAIN_FL&pslnkid=CS_S201605302223124733554248&ICAJAXTrf=true&ICAJAX=1&ICMDTarget=start&ICPanelControlStyle=%20pst_side1-fixed%20pst_panel-mode%20";
const FAKE2_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_ES_FL.GBL?Page=SSR_CLSRCH_ES_FL&SEARCH_GROUP=SSR_CLASS_SEARCH_LFF&SEARCH_TEXT=gly%20105&ES_INST=UBFLO&ES_STRM=2231&ES_ADV=N&INVOKE_SEARCHAGAIN=PTSF_GBLSRCH_FLUID";
const PAGE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_3/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CRSE_INFO_FL.GBL?Page=SSR_CRSE_INFO_FL&Action=U&Page=SSR_CS_WRAP_FL&Action=U&ACAD_CAREER=UGRD&CRSE_ID=004544&CRSE_OFFER_NBR=1&INSTITUTION=UBFLO&STRM=2231&CLASS_NBR=19606&pts_Portal=EMPLOYEE&pts_PortalHostNode=SA&pts_Market=GBL&ICAJAX=1";

const TOKEN_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT&";
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
        mut page_num: u32,
    ) -> Result<ResponseFuture, SessionError> {
        // If the page is 2 then it sends a fake request and internally increments the page to 3.
        // This statement prevents page 3 from being returned twice.
        if page_num >= 3 {
            page_num += 1;
        }

        Self::get_page_internal(client, token, page_num).await
    }

    async fn get_page_internal(
        client: &Client<T, Body>,
        token: &str,
        mut page_num: u32,
    ) -> Result<ResponseFuture, SessionError> {
        loop {
            match page_num {
                1 => {
                    get_with_token(client, token, FAKE1_URL)?.await?;
                    get_with_token(client, token, FAKE2_URL)?.await?;
                    return get_with_token(client, token, PAGE1_URL);
                }
                2 => {
                    // TODO: Multiple things to know about >1 pages:
                    //  1. Each page holds 50 groups max.
                    //  2. They are all POST requests with a slightly differing body (ICState and
                    //     ICAction).
                    //  3. How I currently have it set up is not how it may actually work. Meaning,
                    //     I know there is a second "phony" request, though invoking it does not
                    //     seem to enable the next page to return the correct result. I'm either
                    //     missing some minute detail in the request or I need to send more phony
                    //     requests prior.

                    // async recursion is a little funky and would require me to pull in the
                    // `async-recursion` crate. It's better off with a little mutation.
                    page_num += 1;
                }
                _ => {
                    todo!();
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
        let response = client.get(Uri::from_static(TOKEN_URL)).await?;
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
            .get_all(header::COOKIE)
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

#[inline]
fn get_with_token<T>(
    client: &Client<T, Body>,
    token: &str,
    // TODO: Into<Uri>
    uri: &'static str,
) -> Result<ResponseFuture, SessionError>
where
    T: Connect + Clone + Send + Sync + 'static,
{
    Ok(client.request(
        Request::builder()
            .uri(Uri::from_static(uri))
            .header(header::COOKIE, token)
            .body(Body::empty())?,
    ))
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
