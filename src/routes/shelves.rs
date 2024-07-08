use axum::{extract::State, http::StatusCode, routing::{get, post}, Extension, Router};

use crate::{proto::shelves::{ClearShelfReq, ConvertToNoteReq, ReadShelfReq, Shelf, UpdateShelfReq}, types::{call_grpc_service, new_ok_res, AppState, Json, ServerResult}};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/shelf", get(shelf_get).patch(shelf_patch).delete(shelf_delete))
        .route("/shelf/to-note", post(shelf_to_note_post))
        .with_state(state.clone())
}

#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn shelf_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Shelf> {

    let shelf = call_grpc_service(
        ReadShelfReq { user_id },
        |req| state.shelves_client.read_shelf(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}

#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn shelf_patch(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<UpdateShelfReq>,
) -> ServerResult<Shelf> {

    body.user_id = user_id;

    let shelf = call_grpc_service(
        body,
        |req| state.shelves_client.update_shelf(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}

#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn shelf_delete(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Shelf> {

    let shelf = call_grpc_service(
        ClearShelfReq { user_id },
        |req| state.shelves_client.clear_shelf(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}

#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn shelf_to_note_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<ConvertToNoteReq>,
) -> ServerResult<Shelf> {

    body.user_id = user_id;

    let shelf = call_grpc_service(
        body,
        |req| state.shelves_client.convert_to_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}
