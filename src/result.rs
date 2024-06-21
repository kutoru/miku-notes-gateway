use axum_extra::extract::CookieJar;
use serde::Serialize;
pub use axum::http::StatusCode;
pub use axum::Json;
pub use crate::error::ResError;

pub type ServerResult<T> = anyhow::Result<(StatusCode, Json<ResultBody<T>>), ResError>;
pub type CookieResult = anyhow::Result<(StatusCode, CookieJar, Json<ResultBody<()>>), ResError>;

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
