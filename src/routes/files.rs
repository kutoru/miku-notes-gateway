use crate::proto::files::create_file_metadata::AttachId;
use crate::proto::files::{CreateFileMetadata, CreateFileReq, DeleteFileReq, DownloadFileMetadata, DownloadFileReq, Empty, File, FileData};
use crate::types::{call_grpc_service, new_ok_res};
use crate::{types::{AppState, ServerResult}, error::ResError};

use std::cmp::min;
use axum::body::Body;
use axum::extract::{multipart, Multipart};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::{Router, routing::{post, get}, extract::{DefaultBodyLimit, State, Path}, Extension, http::StatusCode};
use futures_util::StreamExt;
use tower_http::limit::RequestBodyLimitLayer;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/files", post(files_post))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            1024 * 1024 * state.req_body_limit,
        ))
        .route("/files/:id", delete(files_delete))
        .route("/files/dl/:hash", get(files_dl_get))
        .with_state(state.clone())
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
        _ => return Err(ResError::InvalidFields("invalid fields".into())),
    };

    let file = match multipart.next_field().await? {
        Some(f) if f.name() == Some("file") => f,
        _ => return Err(ResError::InvalidFields("invalid fields".into())),
    };

    Ok(MultipartBody { attach_id, file })
}

async fn files_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    mut multipart: Multipart,
) -> ServerResult<File> {

    let chunk_size = 1024 * 1024 * state.file_chunk_size;

    // defining a stream that yields CreateFileReq objects with file data.
    // the setup logic is located in the stream as well,
    // and that's because grpc call requires multipart to live as long as the stream

    let file_stream = async_stream::stream! {

        // extracting basic file info

        let Ok(MultipartBody { attach_id, mut file }) = parse_multipart(&mut multipart).await else {
            return println!("multipart parse error");
        };

        let name = file.file_name().map(String::from).unwrap_or_default();

        println!("user_id, name, attach_id, chunk_size: {}, {}, {:?}, {}", user_id, name, attach_id, chunk_size);

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
                    println!("chunk {}: {}", i, data.len());
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

async fn files_dl_get(
    State(mut state): State<AppState>,
    Path(file_hash): Path<String>,
    Extension(user_id): Extension<i32>,
) -> impl IntoResponse {

    println!("files_dl_get with file_hash and user_id: {}, {}", file_hash, user_id);

    let mut stream = call_grpc_service(
        DownloadFileReq { user_id, file_hash },
        |req| state.files_client.download_file(req),
        &state.data_token,
    ).await?;

    let first_part = stream.next().await
        .ok_or(ResError::ServerError("server error".into()))??;

    let Some(DownloadFileMetadata {
        name: file_name,
        size: file_size,
    }) = first_part.metadata else {
        return Err(ResError::ServerError("server error".into()));
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

async fn files_delete(
    State(mut state): State<AppState>,
    Path(file_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    println!("files_delete with file_id and user_id: {}, {}", file_id, user_id);

    let res_body = call_grpc_service(
        DeleteFileReq { id: file_id, user_id: user_id },
        |req| state.files_client.delete_file(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}
