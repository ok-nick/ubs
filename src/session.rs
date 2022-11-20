use cookie::Cookie;
use hyper::{
    body::{self, Bytes},
    client::{HttpConnector, ResponseFuture},
    header::{COOKIE, SET_COOKIE},
    Body, Client, HeaderMap, Request, Uri,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use thiserror::Error;

pub type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

#[derive(Debug, Clone)]
pub struct Session {
    client: HttpsClient,
    token: Token,
}

impl Session {
    pub fn new(token: Token) -> Self {
        Self {
            client: Client::builder().build(
                HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http1()
                    .build(),
            ),
            token,
        }
    }

    pub fn with_client(&self, client: HttpsClient, token: Token) -> Self {
        Self { client, token }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    // TODO: remove excess queries from uri
    // TODO: return an iterator over pages
    // pages must be fetched over the same TCP connection, otherwise it will not work
    pub async fn class_schedule_raw(
        &self,
        course_id: u32,
    ) -> Result<Iterator<Item = Page>, SessionError> {
        // Must be executed sequentially; the hub does some weird server-sided magic linked to
        // the token.
        self.get_with_token("https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_MAIN_FL.GBL?Page=SSR_CLSRCH_MAIN_FL&pslnkid=CS_S201605302223124733554248&ICAJAXTrf=true&ICAJAX=1&ICMDTarget=start&ICPanelControlStyle=%20pst_side1-fixed%20pst_panel-mode%20")?.await?;
        self.get_with_token("https://www.pub.hub.buffalo.edu/psc/csprdpub_1/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CLSRCH_ES_FL.GBL?Page=SSR_CLSRCH_ES_FL&SEARCH_GROUP=SSR_CLASS_SEARCH_LFF&SEARCH_TEXT=gly%20105&ES_INST=UBFLO&ES_STRM=2231&ES_ADV=N&INVOKE_SEARCHAGAIN=PTSF_GBLSRCH_FLUID")?.await?;

        let response = self.get_with_token("https://www.pub.hub.buffalo.edu/psc/csprdpub_3/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_CRSE_INFO_FL.GBL?Page=SSR_CRSE_INFO_FL&Action=U&Page=SSR_CS_WRAP_FL&Action=U&ACAD_CAREER=UGRD&CRSE_ID=004544&CRSE_OFFER_NBR=1&INSTITUTION=UBFLO&STRM=2231&CLASS_NBR=19606&pts_Portal=EMPLOYEE&pts_PortalHostNode=SA&pts_Market=GBL&ICAJAX=1")?.await?;

        Ok(Page::new(
            &self.client,
            body::to_bytes(response.into_body()).await?,
        ))
    }

    // TODO: lighten up with the 'static requirement
    fn get_with_token(&self, uri: &'static str) -> Result<ResponseFuture, SessionError> {
        get_with_token(&self.client, self.token.to_string_cookie(), uri)
    }
}

#[derive(Debug, Clone)]
pub struct Page<'a> {
    client: &'a HttpsClient,
    bytes: Bytes,
}

impl<'a> Page<'a> {
    fn new(client: &'a HttpsClient, bytes: Bytes) -> Self {
        Self { client, bytes }
    }

    pub fn next_page(&self) -> Option<Page> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Token(Cookie<'static>);

impl Token {
    const TOKEN_NAME: &str = "psprd-8083-PORTAL-PSJSESSIONID";

    pub async fn new(client: HttpsClient) -> Result<Self, SessionError> {
        let first = client.get(Uri::from_static("https://www.pub.hub.buffalo.edu/psc/csprdpub/EMPLOYEE/SA/c/NUI_FRAMEWORK.PT_LANDINGPAGE.GBL?tab=DEFAULT")).await?;
        let token_cookie =
            Token::token_cookie(first.headers()).ok_or(SessionError::TokenCookieNotFound)?;

        // TODO: check if redirect and use redirect Location instead of hardcoded link
        let second = get_with_token(
            &client,
            token_cookie.to_string(),
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
            // TODO: is this the correct header?
            .get_all(COOKIE)
            .iter()
            // If it can't be parsed then skip it
            .filter_map(|string| {
                string
                    .to_str()
                    .ok()
                    .and_then(|raw_cookie| Cookie::parse(raw_cookie).ok())
            })
            .find(|cookie| cookie.name() == Token::TOKEN_NAME)
    }
}

fn get_with_token(
    client: &HttpsClient,
    token: String,
    uri: &'static str,
) -> Result<ResponseFuture, SessionError> {
    Ok(client.request(
        Request::builder()
            .uri(Uri::from_static(uri))
            // TODO: correct header?
            .header(COOKIE, token)
            .body(Body::empty())?,
    ))
}

/// Represents errors that can occur retrieving course data.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An argument to build an HTTP request was invalid.
    /// See more [here](https://docs.rs/http/0.2.8/http/request/struct.Builder.html#errors)
    #[error("an argument while building an HTTP request was invalid")]
    MalformedHttpArgs(#[from] hyper::http::Error),
    /// Failed to send HTTP request.
    #[error("failed to send HTTP request")]
    HttpRequestFailed(#[from] hyper::Error),
    /// Attempted to parse a cookie with an invalid format.
    #[error("cannot parse cookie with an invalid format")]
    MalformedCookie(#[from] cookie::ParseError),
    // TODO: provide cookie parsing errors
    /// Could not find or parse the token cookie.
    #[error("cannot find or parse the token cookie")]
    TokenCookieNotFound,
}
