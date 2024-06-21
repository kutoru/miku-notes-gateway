use axum_extra::extract::CookieJar;
use axum::{Json, http::StatusCode};
use serde::Serialize;
use crate::error::ResError;

pub type ServerResult<T> = Result<(StatusCode, Json<ResultBody<T>>), ResError>;
pub type CookieResult = Result<(StatusCode, CookieJar, Json<ResultBody<()>>), ResError>;

#[derive(Debug, Serialize, Clone)]
pub struct ResultBody<T> {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<T>,
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
