use axum::extract::FromRequest;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::{CookieJar, Cookie, SameSite};
use axum::http::StatusCode;
use futures_util::Future;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::{debug, error};
use utoipa::openapi::{ResponseBuilder, ResponsesBuilder};
use utoipa::{IntoResponses, ToSchema};

use crate::proto::shelves::shelves_client::ShelvesClient;
use crate::proto::{notes::notes_client::NotesClient, tags::tags_client::TagsClient, files::files_client::FilesClient, auth::auth_client::AuthClient};
use crate::error::ResError;

pub type ServerResult<T> = Result<(StatusCode, Json<ResultBody<T>>), ResError>;
pub type CookieResult = Result<(StatusCode, CookieJar, Json<ResultBody<()>>), ResError>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub log_level: tracing::Level,
    pub service_port: u16,
    pub frontend_url: String,
    pub req_body_limit: usize,
    pub file_chunk_size: usize,

    pub auth_token: String,
    pub data_token: String,

    pub access_token_ttl: i64,
    pub refresh_token_ttl: i64,
    pub access_token_key: String,
    pub refresh_token_key: String,

    pub auth_client: AuthClient<Channel>,
    pub notes_client: NotesClient<Channel>,
    pub tags_client: TagsClient<Channel>,
    pub files_client: FilesClient<Channel>,
    pub shelves_client: ShelvesClient<Channel>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ResultBody<T> {
    pub success: bool,
    pub error: Option<String>,
    #[schema(value_type = Option<Object>)]
    pub data: Option<T>,
}

// Example responses for the OpenAPI docs
pub enum ExRes200 {}
pub enum ExRes201 {}
pub enum ExRes400 {}
pub enum ExRes401 {}
pub enum ExRes404 {}
pub enum ExRes415 {}
pub enum ExRes422 {}
pub enum ExRes5XX {}

type ResponseMap = std::collections::BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>>;

fn build_response(code: &str, desc: &str) -> ResponseMap {
    ResponsesBuilder::new()
        .response(code, ResponseBuilder::new().description(desc))
        .build()
        .into()
}

impl IntoResponses for ExRes200 {
    fn responses() -> ResponseMap { build_response("200", "Success") }
}
impl IntoResponses for ExRes201 {
    fn responses() -> ResponseMap { build_response("201", "Item created successfully") }
}
impl IntoResponses for ExRes400 {
    fn responses() -> ResponseMap { build_response("400", "The request was invalid. Most likely the body, path, or query format was incorrect") }
}
impl IntoResponses for ExRes401 {
    fn responses() -> ResponseMap { build_response("401", "The required auth token is either missing or invalid") }
}
impl IntoResponses for ExRes404 {
    fn responses() -> ResponseMap { build_response("404", "Some item related to the request was not found") }
}
impl IntoResponses for ExRes415 {
    fn responses() -> ResponseMap { build_response("415", "The request's content type was incorrect") }
}
impl IntoResponses for ExRes422 {
    fn responses() -> ResponseMap { build_response("422", "There was something wrong with the request's body fields") }
}
impl IntoResponses for ExRes5XX {
    fn responses() -> ResponseMap { build_response("5XX", "Some internal server error happened that wasn't the client's fault") }
}

/// custom `Json` type used to handle json errors manually (more specifically, convert them to `ResError`)
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(ResError))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

/// Generically calls a grpc service
pub async fn call_grpc_service<ReqBody, ReqFn, ResBody, ResFuture>(
    body: ReqBody,
    req_fn: ReqFn,
    service_token: &str,
) -> Result<ResBody, tonic::Status>
where
    ReqFn: FnOnce(tonic::Request<ReqBody>) -> ResFuture,
    ResFuture: Future<Output = Result<tonic::Response<ResBody>, tonic::Status>>,
{
    let mut request = tonic::Request::new(body);
    let header_value = format!("Bearer {}", service_token).parse().unwrap();
    request.metadata_mut().append("authorization", header_value);
    let response = req_fn(request).await?;
    Ok(response.into_inner())
}

pub trait CreateAndAddCookie {
    fn add_new_cookie(self, _: String,  _: String, _: i64) -> Self;
}
impl CreateAndAddCookie for CookieJar {
    /// Creates a new cookie based on the arguments and adds it to the jar
    fn add_new_cookie(self, cookie_key: String, token: String, token_exp: i64) -> Self {
        let exp_time = time::Duration::seconds(token_exp);

        let cookie = Cookie::build((cookie_key, token))
            .max_age(exp_time)
            .path("/")
            .same_site(SameSite::None)
            .http_only(true)
            .secure(false);

        self.add(cookie)
    }
}

pub fn new_ok_res<T>(code: StatusCode, data: T) -> ServerResult<T> {
    Ok((
        code,
        Json(ResultBody { success: true, error: None, data: Some(data) }),
    ))
}

pub fn new_cookie_ok_res(jar: CookieJar) -> CookieResult {
    Ok((
        StatusCode::OK,
        jar,
        Json(ResultBody { success: true, error: None, data: None }),
    ))
}

/// Converts the arguments into a tuple that implements IntoResponse. It also logs the error messages that it receives
pub fn new_err_res(status_code: StatusCode, response_msg: &str, internal_msg: String) -> (StatusCode, Json<ResultBody<()>>) {
    let code = status_code.as_u16();
    let status = status_code.canonical_reason();

    if code >= 500 {
        error!(code, status, response_msg, internal_msg);
    } else {
        debug!(code, status, response_msg, internal_msg);
    }

    (status_code, Json(ResultBody { success: false, error: Some(response_msg.into()), data: None }))
}
