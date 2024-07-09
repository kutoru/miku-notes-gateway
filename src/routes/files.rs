use crate::proto::files::create_file_metadata::AttachId;
use crate::proto::files::{CreateFileMetadata, CreateFileReq, DeleteFileReq, DownloadFileMetadata, DownloadFileReq, Empty, File, FileData};
use crate::types::{call_grpc_service, new_ok_res, ExRes400, ExRes401, ExRes404, ExRes5XX};
use crate::{types::{AppState, ServerResult}, error::ResError};

use std::cmp::min;
use std::fmt::Debug;
use axum::body::Body;
use axum::extract::{multipart, Multipart};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::{Router, routing::{post, get}, extract::{DefaultBodyLimit, State, Path}, Extension, http::StatusCode};
use futures_util::StreamExt;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::debug;
use utoipa::{OpenApi, ToSchema};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/", post(files_post))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            1024 * 1024 * state.req_body_limit,
        ))
        .route("/:id", delete(files_delete))
        .route("/dl/:hash", get(files_dl_get))
        .with_state(state.clone())
}

#[derive(OpenApi)]
#[openapi(
    paths(files_post, files_dl_get, files_delete),
    components(schemas(ExampleMultipartBody, File, Empty)),
    security(("access_token" = [])),
)]
pub struct Api;

#[derive(ToSchema)]
#[schema(title = "MultipartBody")]
#[allow(dead_code)]
struct ExampleMultipartBody {
    note_id: Option<i32>,
    shelf_id: Option<i32>,
    file: Vec<u8>,
}

// impl to convert tonic stream directly into axum body (for files_dl_get)
impl From<FileData> for axum::body::Bytes {
    fn from(value: FileData) -> Self {
        value.data.into()
    }
}

#[derive(Debug)]
struct MultipartBody<'a> {
    attach_id: AttachId,
    file: multipart::Field<'a>,
}

async fn parse_multipart(multipart: &mut Multipart) -> Result<MultipartBody<'_>, ResError> {

    let attach_id = match multipart.next_field().await? {
        Some(f) if f.name() == Some("note_id") => AttachId::NoteId(f.text().await?.parse()?),
        Some(f) if f.name() == Some("shelf_id") => AttachId::ShelfId(f.text().await?.parse()?),
        _ => return Err(ResError::InvalidFields("Could not get either a note_id nor shelf_id from the multipart body".into())),
    };

    let file = match multipart.next_field().await? {
        Some(f) if f.name() == Some("file") => f,
        _ => return Err(ResError::InvalidFields("Could not get the file from the multipart body".into())),
    };

    Ok(MultipartBody { attach_id, file })
}

/// Create a new file
///
/// Post (upload) a new file and immediately attach it to either a note or a shelf
#[utoipa::path(
    post, path = "",
    request_body(content = ExampleMultipartBody, content_type = "multipart/form-data", description = "Note that despite `note_id` and `shelf_id` are showing as optional, you must always specify exactly one of them"),
    responses(
        (status = 201, description = "File has been successfully created", body = File),
        ExRes400, ExRes401, ExRes404, ExRes5XX,
    ),
)]
#[tracing::instrument(fields(attach_id, file_name), skip(state, multipart), err(level = tracing::Level::DEBUG))]
async fn files_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    mut multipart: Multipart,
) -> ServerResult<File> {

    let chunk_size = 1024 * 1024 * state.file_chunk_size;
    let span = tracing::Span::current();

    // defining a stream that yields CreateFileReq objects with file data.
    // the setup logic is located in the stream as well,
    // and that's because grpc call requires multipart to live as long as the stream

    let file_stream = async_stream::stream! {

        // extracting basic file info

        let MultipartBody { attach_id, mut file } = match parse_multipart(&mut multipart).await {
            Ok(b) => b,
            Err(e) => return debug!(parent: &span, "{e}"),
        };

        let name = file.file_name().map(String::from).unwrap_or_default();

        span.record("attach_id", format!("{attach_id:?}"));
        span.record("file_name", name.clone());

        // sending the first part containing only the metadata

        yield CreateFileReq {
            metadata: Some(CreateFileMetadata {
                user_id, name, attach_id: Some(attach_id),
            }),
            data: Vec::new(),
        };

        let mut i = 0;
        let mut chunks = Vec::new();
        while let Some(Ok(chunk)) = file.next().await {

            // making sure that the chunk does not exceed the max chunk size.
            // if it does, splitting it

            let mut j = 0;
            while chunk_size * j < chunk.len() {
                let new_chunk_range = chunk_size * j..min(chunk_size * (j + 1), chunk.len());
                let new_chunk = chunk.slice(new_chunk_range);
                chunks.push(new_chunk);
                j += 1;
            }

            // yielding chunks

            while !chunks.is_empty() {
                let data = chunks.remove(0).to_vec();

                i += 1;
                if i < 10 || (i < 100 && i % 10 == 0) || (i < 1000 && i % 100 == 0) || i % 1000 == 0 {
                    debug!(parent: &span, "chunk {}: {}", i, data.len());
                }

                yield CreateFileReq { metadata: None, data };
            }
        }
    };

    // the usual rpc stuff

    let new_file = call_grpc_service(
        file_stream,
        |req| state.files_client.create_file(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::CREATED, new_file)
}

/// Download a file
#[utoipa::path(
    get, path = "/dl/{file_hash}",
    responses(
        (status = 200, description = "File has been successfully sent", body = Vec<u8>, content_type = "*/*"),
        ExRes401, ExRes404, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn files_dl_get(
    State(mut state): State<AppState>,
    Path(file_hash): Path<String>,
    Extension(user_id): Extension<i32>,
) -> impl IntoResponse {

    let mut stream = call_grpc_service(
        DownloadFileReq { user_id, file_hash },
        |req| state.files_client.download_file(req),
        &state.data_token,
    ).await?;

    let first_part = stream.next().await
        .ok_or(ResError::ServerError("Could not get the first message from a file stream".into()))??;

    let Some(DownloadFileMetadata {
        name: file_name,
        size: file_size,
    }) = first_part.metadata else {
        return Err(ResError::ServerError("Could not get metadata from the first message in a file stream".into()));
    };

    let body = Body::from_stream(stream);

    let content_length = (header::CONTENT_LENGTH, file_size.to_string());
    let content_disposition = (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name));

    // https://stackoverflow.com/a/28652339
    match mime_guess::from_path(&file_name).first().map(|h| h.to_string()) {
        Some(content_type) => Ok((
            [content_length, content_disposition, (header::CONTENT_TYPE, content_type)],
            body,
        ).into_response()),
        None => Ok((
            [content_length, content_disposition],
            body,
        ).into_response()),
    }
}

/// Delete a file
#[utoipa::path(
    delete, path = "/{file_id}",
    responses(
        (status = 200, description = "File has been successfully deleted", body = Empty),
        ExRes400, ExRes401, ExRes404, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn files_delete(
    State(mut state): State<AppState>,
    Path(file_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    let res_body = call_grpc_service(
        DeleteFileReq { id: file_id, user_id: user_id },
        |req| state.files_client.delete_file(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}
