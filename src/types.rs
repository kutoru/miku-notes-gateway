use axum_extra::extract::cookie::{CookieJar, Cookie, SameSite};
use axum::{Json, http::StatusCode};
use axum_typed_multipart::{TryFromMultipart, FieldData};
use futures_util::Future;
use serde::Serialize;
use tempfile::NamedTempFile;
use tonic::transport::Channel;

use crate::proto::{notes::notes_client::NotesClient, tags::tags_client::TagsClient, files::files_client::FilesClient, auth::auth_client::AuthClient};
use crate::error::ResError;

pub type ServerResult<T> = Result<(StatusCode, Json<ResultBody<T>>), ResError>;
pub type CookieResult = Result<(StatusCode, CookieJar, Json<ResultBody<()>>), ResError>;

#[derive(Clone)]
pub struct AppState {
    pub service_addr: String,
    pub frontend_url: String,
    pub req_body_limit: usize,
    pub file_chunk_size: usize,

    pub access_token_exp: i64,
    pub refresh_token_exp: i64,
    pub access_token_key: String,
    pub refresh_token_key: String,

    pub auth_token: String,
    pub data_token: String,

    pub auth_client: AuthClient<Channel>,
    pub notes_client: NotesClient<Channel>,
    // pub tags_client: TagsClient<Channel>,
    pub files_client: FilesClient<Channel>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResultBody<T> {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, TryFromMultipart)]
pub struct MultipartRequest {
    // unlimited is supposed to follow the request body limit
    #[form_data(limit = "unlimited")]
    pub file: FieldData<NamedTempFile>,
    pub note_id: i32,
}

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

pub fn new_err_res(code: StatusCode, msg: String) -> (StatusCode, Json<ResultBody<()>>) {
    (code, Json(ResultBody { success: false, error: Some(msg), data: None }))
}
