use axum::{extract::rejection::JsonRejection, http::{header::InvalidHeaderValue, StatusCode}, response::{IntoResponse, Response}};
use crate::types::new_err_res;

// about 4xx status codes
// https://stackoverflow.com/a/52098667

/// Custom error type that is used across the entire crate
#[derive(Debug)]
pub enum ResError {
    /// When one or more fields are missing
    InvalidFields(String),
    /// When the fields are present but the value types are invalid
    InvalidValues(String),
    /// When the content type is wrong for whatever reason
    InvalidContentType(String),
    /// When getting, patching or deleting something that doesn't exist
    NotFound(String),
    /// When the request is lacking credentials
    Unauthorized(String),
    /// When the request is authenticated but not allowed to access a resource
    Forbidden(String),
    /// When the issue with the request is too hard to explain
    BadRequest(String),

    NotImplemented(String),
    /// Any error that is the service's fault
    ServerError(String),
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
            Self::InvalidFields(m) => new_err_res(StatusCode::BAD_REQUEST, "invalid fields", m),
            Self::InvalidValues(m) => new_err_res(StatusCode::UNPROCESSABLE_ENTITY, "invalid values", m),
            Self::InvalidContentType(m) => new_err_res(StatusCode::UNSUPPORTED_MEDIA_TYPE, "invalid content type", m),
            Self::NotFound(m) => new_err_res(StatusCode::NOT_FOUND, "not found", m),
            Self::Unauthorized(m) => new_err_res(StatusCode::UNAUTHORIZED, "unauthorized", m),
            Self::Forbidden(m) => new_err_res(StatusCode::FORBIDDEN, "forbidden", m),
            Self::BadRequest(m) => new_err_res(StatusCode::BAD_REQUEST, "bad request", m),

            Self::NotImplemented(m) => new_err_res(StatusCode::NOT_IMPLEMENTED, "not implemented", m),
            Self::ServerError(m) => new_err_res(StatusCode::INTERNAL_SERVER_ERROR, "server error", m),
        }.into_response()
    }
}

fn msg<T: ToString>(value: T) -> String {
    value.to_string()
}

impl From<axum::extract::multipart::MultipartError> for ResError {
    fn from(value: axum::extract::multipart::MultipartError) -> Self {
        Self::InvalidFields(msg(value))
    }
}

impl From<tokio::io::Error> for ResError {
    fn from(value: tokio::io::Error) -> Self {
        Self::ServerError(msg(value))
    }
}

impl From<InvalidHeaderValue> for ResError {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::ServerError(msg(value))
    }
}

impl From<tonic::transport::Error> for ResError {
    fn from(value: tonic::transport::Error) -> Self {
        Self::ServerError(msg(value))
    }
}

impl From<tonic::Status> for ResError {
    fn from(value: tonic::Status) -> Self {
        let msg = msg(&value);

        match value.code() {
            tonic::Code::NotFound => Self::NotFound(msg),
            tonic::Code::PermissionDenied => Self::Forbidden(msg),
            tonic::Code::InvalidArgument => Self::InvalidFields(msg),
            tonic::Code::AlreadyExists => Self::BadRequest(msg),
            tonic::Code::Unimplemented => Self::NotImplemented(msg),
            tonic::Code::Unauthenticated => match value.message() {
                "invalid authorization token" => Self::ServerError(msg),
                _ => Self::Unauthorized(msg),
            },

            _ => Self::ServerError(msg),
        }
    }
}

impl From<JsonRejection> for ResError {
    fn from(value: JsonRejection) -> Self {
        let msg = msg(&value);

        match value {
            JsonRejection::MissingJsonContentType(_) => Self::InvalidContentType(msg),
            JsonRejection::JsonDataError(_) => Self::InvalidValues(msg),
            _ => Self::BadRequest(msg),
        }
    }
}

impl From<std::num::ParseIntError> for ResError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidFields(msg(value))
    }
}
