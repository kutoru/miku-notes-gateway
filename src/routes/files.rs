use crate::proto::files::{CreateFileMetadata, CreateFileReq, DeleteFileReq, File, Empty};
use crate::types::{call_grpc_service, new_ok_res};
use crate::{types::{AppState, ServerResult, MultipartRequest}, error::ResError};

use axum::{Router, routing::{post, get}, extract::{DefaultBodyLimit, State, Path}, Extension, http::StatusCode};
use tower_http::limit::RequestBodyLimitLayer;
use axum_typed_multipart::TypedMultipart;
use tokio::io::AsyncReadExt;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/files", post(files_post))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            1024 * 1024 * state.req_body_limit,
        ))
        .route("/files/:id", get(files_get).delete(files_delete))
        .with_state(state.clone())
}

async fn files_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    TypedMultipart(body): TypedMultipart<MultipartRequest>,
) -> ServerResult<File> {

    // preparing basic file info

    let note_id = body.note_id;
    let name = body.file.metadata.file_name.unwrap_or("".into());

    println!("files_post with user_id, note_id, name: {}, {}, {}", user_id, note_id, name);

    // preparing the file and size info

    let mut file = tokio::fs::File::from_std(body.file.contents.into_file());

    let file_size = file.metadata().await.unwrap().len();
    let chunk_size = (1024 * 1024 * state.file_chunk_size) as u64;
    let expected_parts = (file_size / chunk_size) as i32 + (file_size % chunk_size > 0) as i32;
    let last_part_len = (file_size % chunk_size) as usize;

    file.set_max_buf_size(chunk_size as usize);
    let mut buffer = vec![0; chunk_size as usize];
    
    println!("size, chunk, parts: {}, {}, {}", file_size, chunk_size, expected_parts);

    // defining a stream that yields CreateFileReq objects with file data

    let file_stream = async_stream::stream! {
        println!("stream start");
        for i in 1..=expected_parts {
            match file.read(&mut buffer).await {
                Ok(len) => if len == 0 { println!("FILE READ LEN == 0"); break; },
                Err(e) => { println!("FILE READ ERR: {:#?}", e); break; },
            }

            let metadata = match i == 1 {
                true => Some(CreateFileMetadata {
                    user_id: user_id,
                    note_id: note_id,
                    name: name.clone(),
                    expected_parts: expected_parts,
                }),
                false => None,
            };

            let data = match i == expected_parts {
                true => buffer[0..last_part_len].to_vec(),
                false => buffer.clone(),
            };

            println!("buf {}: {}", i, data.len());

            yield CreateFileReq { metadata, data };
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

async fn files_get(
    State(mut state): State<AppState>,
    Path(file_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<File> {

    println!("files_get with file_id and user_id: {}, {}", file_id, user_id);

    Err(ResError::NotImplemented("".into()))
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
