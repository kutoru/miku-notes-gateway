use axum::{extract::rejection::JsonRejection, http::{header::InvalidHeaderValue, StatusCode}, response::{IntoResponse, Response}};
use crate::types::new_err_res;

// about 4xx status codes
// https://stackoverflow.com/a/52098667

#[derive(Debug)]
pub enum ResError {
    InvalidFields(String),  // when one or more fields are missing
    InvalidValues(String),  // when the fields are present but the value types are invalid
    InvalidContentType(String),
    NotFound(String),  // when getting, patching or deleting something that doesn't exist
    Unauthorized(String),  // when the request is lacking credentials
    Forbidden(String),  // when the request is authenticated but not allowed to access a resource
    BadRequest(String),  // when the issue with the request is too hard to explain

    GRPCError(String),
    FSError(String),
    NotImplemented(String),
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
            Self::InvalidFields(msg) => new_err_res(StatusCode::BAD_REQUEST, msg),
            Self::InvalidValues(msg) => new_err_res(StatusCode::UNPROCESSABLE_ENTITY, msg),
            Self::InvalidContentType(msg) => new_err_res(StatusCode::UNSUPPORTED_MEDIA_TYPE, msg),
            Self::NotFound(msg) => new_err_res(StatusCode::NOT_FOUND, msg),
            Self::Unauthorized(msg) => new_err_res(StatusCode::UNAUTHORIZED, msg),
            Self::Forbidden(msg) => new_err_res(StatusCode::FORBIDDEN, msg),
            Self::BadRequest(msg) => new_err_res(StatusCode::BAD_REQUEST, msg),

            Self::GRPCError(msg) => new_err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::FSError(msg) => new_err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::NotImplemented(msg) => new_err_res(StatusCode::NOT_IMPLEMENTED, msg),
            Self::ServerError(msg) => new_err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
        }.into_response()
    }
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
            tonic::Code::PermissionDenied => Self::Forbidden(msg),
            tonic::Code::InvalidArgument => Self::InvalidFields(msg),
            tonic::Code::AlreadyExists => Self::BadRequest(msg),
            tonic::Code::Unimplemented => Self::NotImplemented(msg),
            tonic::Code::Unauthenticated => match msg.as_str() {
                "invalid authorization token" => Self::ServerError("Server error".into()),
                _ => Self::Unauthorized(msg),
            },

            _ => Self::ServerError(msg),
        }
    }
}

impl From<JsonRejection> for ResError {
    fn from(value: JsonRejection) -> Self {
        match value {
            JsonRejection::MissingJsonContentType(_) => Self::InvalidContentType("invalid content type".into()),
            JsonRejection::JsonDataError(_) => Self::InvalidValues("invalid field".into()),
            JsonRejection::BytesRejection(_) => Self::BadRequest("bytes rejection".into()),
            _ => Self::BadRequest("invalid body".into()),
        }
    }
}

impl From<std::num::ParseIntError> for ResError {
    fn from(_value: std::num::ParseIntError) -> Self {
        Self::InvalidFields("invalid fields".into())
    }
}
