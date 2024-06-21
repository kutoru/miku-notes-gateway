use axum::{response::{IntoResponse, Response}, http::{StatusCode, header::InvalidHeaderValue}, Json};
use crate::{types::{ResultBody, ServerResult}, res};

#[derive(Debug)]
pub enum ResError {
    InvalidFields(String),  // when the fields are missing or invalid
    NotFound(String),  // when getting, patching or deleting something that doesn't exist
    Unauthorized(String),  // when the request is lacking credentials
    Forbidden(String),  // when the request is authorized but not allowed to access a resource
    BadRequest(String),  // when the issue with the request is too hard to explain

    GRPCError(String),
    FSError(String),
    ServerError(String),  // anything else
}

impl core::fmt::Display for ResError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for ResError {}

impl IntoResponse for ResError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidFields(msg) => err_res(StatusCode::UNPROCESSABLE_ENTITY, msg),
            Self::NotFound(msg) => err_res(StatusCode::NOT_FOUND, msg),
            Self::Unauthorized(msg) => err_res(StatusCode::UNAUTHORIZED, msg),
            Self::Forbidden(msg) => err_res(StatusCode::FORBIDDEN, msg),
            Self::BadRequest(msg) => err_res(StatusCode::BAD_REQUEST, msg),

            Self::GRPCError(msg) => err_res(StatusCode::BAD_REQUEST, msg),
            Self::FSError(msg) => err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::ServerError(msg) => err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
        }.into_response()
    }
}

fn err_res(code: StatusCode, msg: String) -> (StatusCode, Json<ResultBody<()>>) {
    let res: ServerResult<()> = res!(code, false, Some(msg), None);
    res.unwrap()
}

fn get_msg<T: std::fmt::Debug>(value: T) -> String {
    let mut msg = format!("{:?}", value);

    // trimming the quotes that i get from the grpc responses
    if msg.starts_with('"') && msg.ends_with('"') {
        msg = msg[1..msg.len()-1].into();
    }

    println!("Error:\n{msg}");
    msg
}

impl From<axum::extract::multipart::MultipartError> for ResError {
    fn from(value: axum::extract::multipart::MultipartError) -> Self {
        Self::InvalidFields(get_msg(value))
    }
}

impl From<std::str::Utf8Error> for ResError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::InvalidFields(get_msg(value))
    }
}

impl From<tokio::io::Error> for ResError {
    fn from(value: tokio::io::Error) -> Self {
        Self::FSError(get_msg(value))
    }
}

impl From<InvalidHeaderValue> for ResError {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::ServerError(get_msg(value))
    }
}

impl From<tonic::transport::Error> for ResError {
    fn from(value: tonic::transport::Error) -> Self {
        Self::GRPCError(get_msg(value))
    }
}

impl From<tonic::Status> for ResError {
    fn from(value: tonic::Status) -> Self {
        let msg = get_msg(value.message());
        match value.code() {
            tonic::Code::NotFound => Self::NotFound(msg),
            tonic::Code::InvalidArgument => Self::InvalidFields(msg),
            tonic::Code::Unauthenticated => Self::Unauthorized(msg),
            tonic::Code::PermissionDenied => Self::Forbidden(msg),
            _ => Self::ServerError(msg),
        }
    }
}
