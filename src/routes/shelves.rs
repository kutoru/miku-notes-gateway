use axum::{extract::State, http::StatusCode, routing::{get, post}, Extension, Router};
use utoipa::OpenApi;

use crate::{proto::shelves::{ClearShelfReq, ConvertToNoteReq, ReadShelfReq, Shelf, UpdateShelfReq}, types::{call_grpc_service, new_ok_res, AppState, ExRes400, ExRes401, ExRes404, ExRes415, ExRes422, ExRes5XX, Json, ServerResult}};

#[derive(OpenApi)]
#[openapi(
    paths(shelf_get, shelf_patch, shelf_delete, shelf_to_note_post),
    components(schemas(Shelf, UpdateShelfReq, ConvertToNoteReq)),
    security(("access_token" = [])),
)]
pub struct Api;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/", get(shelf_get).patch(shelf_patch).delete(shelf_delete))
        .route("/to-note", post(shelf_to_note_post))
        .with_state(state.clone())
}

/// Get user's shelf
#[utoipa::path(
    get, path = "",
    responses(
        (status = 200, description = "Success", body = Shelf),
        ExRes401, ExRes5XX,
    ),
)]
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

/// Update the shelf
///
/// Note that in order to update individual shelf's files, you'll have to call the file routes
#[utoipa::path(
    patch, path = "",
    request_body(content = UpdateShelfReq),
    responses(
        (status = 200, description = "Success", body = Shelf),
        ExRes400, ExRes401, ExRes404, ExRes415, ExRes422, ExRes5XX,
    ),
)]
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

/// Clear the shelf
///
/// Removes the shelf's text and deletes any attached files
#[utoipa::path(
    delete, path = "",
    responses(
        (status = 200, description = "Success", body = Shelf),
        ExRes400, ExRes401, ExRes404, ExRes5XX,
    ),
)]
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

/// Convert the shelf to a note
///
/// Clears the shelf while creating a new note. Any attached files will automatically transfer to the newly created note
#[utoipa::path(
    post, path = "/to-note",
    request_body(content = ConvertToNoteReq),
    responses(
        (status = 201, description = "Success", body = Shelf),
        ExRes400, ExRes401, ExRes415, ExRes422, ExRes5XX,
    ),
)]
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
