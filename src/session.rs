use std::sync::Arc;

use cookie::Cookie;
use futures::{stream, StreamExt, TryFutureExt, TryStream, TryStreamExt};
use hyper::{
    body::{self, Bytes},
    client::{connect::Connect, ResponseFuture},
    header, Body, Client, HeaderMap, Request, Uri,
};
use thiserror::Error;

// TODO: remove excess queries from url
const FAKE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_MAIN_FL.GBL?Page=SSR_CLSRCH_MAIN_FL&pslnkid=CS_S201605302223124733554248&ICAJAXTrf=true&ICAJAX=1&ICMDTarget=start&ICPanelControlStyle=%20pst_side1-fixed%20pst_panel-mode%20";
const FAKE2_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_ES_FL.GBL?Page=SSR_CLSRCH_ES_FL&SEARCH_GROUP=SSR_CLASS_SEARCH_LFF&SEARCH_TEXT=gly%20105&ES_INST=UBFLO&ES_STRM=2231&ES_ADV=N&INVOKE_SEARCHAGAIN=PTSF_GBLSRCH_FLUID";
const PAGE1_URL: &str = "https://www.pub.hub.buffalo.edu/psc/csprdpub_3/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CRSE_INFO_FL.GBL?Page=SSR_CRSE_INFO_FL&Action=U&Page=SSR_CS_WRAP_FL&Action=U&ACAD_CAREER=UGRD&CRSE_ID=004544&CRSE_OFFER_NBR=1&INSTITUTION=UBFLO&STRM=2231&CLASS_NBR=19606&pts_Portal=EMPLOYEE&pts_PortalHostNode=SA&pts_Market=GBL&ICAJAX=1";

const TOKEN_URL: &str ="https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT";
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
    // TODO: allow choosing semester
    // TODO: AsRef<str>
    pub fn schedule_iter<'a>(
        &self,
        course_id: &'a str,
    ) -> impl TryStream<Ok = Bytes, Error = SessionError> + 'a {
        let client = self.client.clone();
        let token = self.token.clone();
        stream::iter(1..)
            .then(move |page_num| {
                // The inner move captures `client` and `token` from the function's scope,
                // then the nested async move captures it from the closures' scope.
                // TODO: when async closures are stabilized this will no longer be necessary?
                let client = client.clone();
                let token = token.clone();
                async move {
                    match page_num {
                        1 => {
                            get_with_token(&client, &token, FAKE1_URL)?.await?;
                            get_with_token(&client, &token, FAKE2_URL)?.await?;
                            // TODO: use `into_err` when stabilized
                            Ok(get_with_token(&client, &token, PAGE1_URL)?.await?)
                        }
                        2 => {
                            // TODO: this case is just a phony request, recurse
                            todo!()
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
            })
            .and_then(|response| body::to_bytes(response.into_body()).err_into())
    }
}

#[derive(Debug, Clone)]
pub struct Token(Cookie<'static>);

impl Token {
    pub async fn new<T>(client: &Client<T, Body>) -> Result<Self, SessionError>
    where
        T: Connect + Clone + Send + Sync + 'static,
    {
        let first = client.get(Uri::from_static(TOKEN_URL)).await?;
        let token_cookie =
            Token::token_cookie(first.headers()).ok_or(SessionError::TokenCookieNotFound)?;

        // TODO: use redirect Location from Self::URL rather than hardcoding
        // What if any of the requests redirect? I need to handle them all, use follow_redirects
        // lib
        // let redirect_uri = first.headers().get(header::LOCATION);
        let second = get_with_token(
            &client,
            &token_cookie.to_string(),
            "https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT&"
        )?.await?;

        Ok(Self(
            Token::token_cookie(second.headers())
                .ok_or(SessionError::TokenCookieNotFound)?
                .into_owned(),
        ))
    }

    fn to_string_cookie(&self) -> String {
        self.0.to_string()
    }

    fn token_cookie<'a>(headers: &'a HeaderMap) -> Option<Cookie<'a>> {
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
