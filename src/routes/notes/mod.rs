use crate::proto::notes::{CreateNoteReq, DeleteNoteReq, Empty, Note, NoteList, UpdateNoteReq};
use crate::types::{call_grpc_service, new_ok_res, AppState, ServerResult};

use axum::extract::Query;
use axum::{Router, routing::{patch, get}, extract::{State, Path}, Json, http::StatusCode, Extension};

use helpers::{parse_note_query, NoteQuery};
mod helpers;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/notes", get(notes_get).post(notes_post))
        .route("/notes/:id", patch(notes_patch).delete(notes_delete))
        .with_state(state.clone())
}

async fn notes_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Query(query): Query<NoteQuery>,
) -> ServerResult<NoteList> {

    println!("notes_get with user_id and query: {}, {:?}", user_id, query);

    let body = parse_note_query(user_id, query)?;

    let note_list = call_grpc_service(
        body,
        |req| state.notes_client.read_notes(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, note_list)
}

async fn notes_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<CreateNoteReq>,
) -> ServerResult<Note> {

    println!("notes_post with user_id and body: {}, {:#?}", user_id, body);

    body.user_id = user_id;

    let new_note = call_grpc_service(
        body,
        |req| state.notes_client.create_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, new_note)
}

async fn notes_patch(
    State(mut state): State<AppState>,
    Path(note_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<UpdateNoteReq>,
) -> ServerResult<Note> {

    println!("notes_patch with note_id, user_id & body: {}, {}, {:#?}", note_id, user_id, body);

    body.id = note_id;
    body.user_id = user_id;

    let updated_note = call_grpc_service(
        body,
        |req| state.notes_client.update_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, updated_note)
}

async fn notes_delete(
    State(mut state): State<AppState>,
    Path(note_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    println!("notes_delete with note_id & user_id: {}, {}", note_id, user_id);

    let res_body = call_grpc_service(
        DeleteNoteReq { id: note_id, user_id },
        |req| state.notes_client.delete_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}
