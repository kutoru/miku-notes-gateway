use axum::{response::{IntoResponse, Response}, http::{StatusCode, header::InvalidHeaderValue}, Json};
use crate::{result::{ResultBody, ServerResult}, res};

#[derive(Debug)]
pub enum ResError {
    MissingFields(String),  // one or more fields in headers or body are missing
    InvalidFields(String),  // one or more fields are present but have invalid values
    NotFound(String),  // when getting, patching or deleting something that doesn't exist
    BadRequest(String),  // when the issue with the request is too hard to explain

    GRPCError(String),

    FSError(String),
    DBError(String),
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
            Self::MissingFields(msg) => err_res(StatusCode::UNPROCESSABLE_ENTITY, msg),
            Self::InvalidFields(msg) => err_res(StatusCode::UNPROCESSABLE_ENTITY, msg),
            Self::NotFound(msg) => err_res(StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => err_res(StatusCode::BAD_REQUEST, msg),

            Self::GRPCError(msg) => err_res(StatusCode::BAD_REQUEST, msg),

            Self::FSError(msg) => err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::DBError(msg) => err_res(StatusCode::INTERNAL_SERVER_ERROR, msg),
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

    println!("{msg}");
    msg
}

// impl From<sqlx::error::Error> for ResError {
//     fn from(value: sqlx::error::Error) -> Self {
//         let msg = get_msg(&value);
//         match value {
//             sqlx::Error::Database(e) => match e.kind() {
//                 ErrorKind::ForeignKeyViolation => Self::BadRequest(msg),
//                 _ => Self::DBError(msg),
//             },
//             sqlx::Error::RowNotFound => Self::NotFound(msg),
//             _ => Self::DBError(msg),
//         }
//     }
// }

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
        Self::GRPCError(get_msg(value.message()))
    }
}

impl From<std::env::VarError> for ResError {
    fn from(value: std::env::VarError) -> Self {
        Self::ServerError(get_msg(value))
    }
}

impl From<anyhow::Error> for ResError {
    fn from(value: anyhow::Error) -> Self {
        Self::ServerError(get_msg(value))
    }
}
