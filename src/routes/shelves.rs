use axum::{extract::State, http::StatusCode, routing::{get, post}, Extension, Json, Router};

use crate::{proto::shelves::{ClearShelfReq, ConvertToNoteReq, ReadShelfReq, Shelf, UpdateShelfReq}, types::{call_grpc_service, new_ok_res, AppState, ServerResult}};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/shelf", get(shelf_get).patch(shelf_patch).delete(shelf_delete))
        .route("/shelf/to-note", post(shelf_to_note_post))
        .with_state(state.clone())
}

async fn shelf_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Shelf> {

    println!("shelf_get with user_id: {}", user_id);

    let shelf = call_grpc_service(
        ReadShelfReq { user_id },
        |req| state.shelves_client.read_shelf(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}

async fn shelf_patch(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<UpdateShelfReq>,
) -> ServerResult<Shelf> {

    println!("shelf_patch with user_id, body: {}, {:?}", user_id, body);

    body.user_id = user_id;

    let shelf = call_grpc_service(
        body,
        |req| state.shelves_client.update_shelf(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}

async fn shelf_delete(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Shelf> {

    println!("shelf_delete with user_id: {}", user_id);

    let shelf = call_grpc_service(
        ClearShelfReq { user_id },
        |req| state.shelves_client.clear_shelf(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}

async fn shelf_to_note_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<ConvertToNoteReq>,
) -> ServerResult<Shelf> {

    println!("shelf_to_note_post with user_id & body: {}, {:?}", user_id, body);

    body.user_id = user_id;

    let shelf = call_grpc_service(
        body,
        |req| state.shelves_client.convert_to_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, shelf)
}
