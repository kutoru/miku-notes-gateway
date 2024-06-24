use axum_extra::extract::CookieJar;
use axum::{Json, http::StatusCode};
use axum_typed_multipart::{TryFromMultipart, FieldData};
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
    pub token_exp: i64,

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

#[macro_export]
macro_rules! res {
    ($code:expr, $success:expr, $msg:expr, $data:expr) => {
        Ok(($code, Json(ResultBody { success: $success, error: $msg, data: $data })))
    }
}

#[macro_export]
macro_rules! jar_res {
    ($code:expr, $jar:expr, $success:expr, $msg:expr) => {
        Ok(($code, $jar, Json(ResultBody { success: $success, error: $msg, data: None })))
    }
}
