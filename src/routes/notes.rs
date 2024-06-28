use crate::proto::notes::{ReadNotesReq, CreateNoteReq, UpdateNoteReq, DeleteNoteReq, Note, NoteList, Empty};
use crate::types::{new_ok_res, AppState, ServerResult};

use axum::{Router, routing::{patch, get}, extract::{State, Path}, Json, http::StatusCode, Extension};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/notes", get(notes_get).post(notes_post))
        .route("/notes/:id", patch(notes_patch).delete(notes_delete))
        .with_state(state.clone())
}

async fn notes_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<NoteList> {

    println!("notes_get with user_id: {}", user_id);

    let request = tonic::Request::new(ReadNotesReq { user_id });
    let response = state.notes_client.read_notes(request).await?;
    let note_list = response.into_inner();

    new_ok_res(StatusCode::OK, note_list)
}

async fn notes_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<CreateNoteReq>,
) -> ServerResult<Note> {

    println!("notes_post with user_id and body: {}, {:#?}", user_id, body);
    body.user_id = user_id;
    println!("Modified body: {:#?}", body);

    let request = tonic::Request::new(body);
    let response = state.notes_client.create_note(request).await?;
    let new_note = response.into_inner();

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
    println!("Modified body: {:#?}", body);

    let request = tonic::Request::new(body);
    let response = state.notes_client.update_note(request).await?;
    let updated_note = response.into_inner();

    new_ok_res(StatusCode::OK, updated_note)
}

async fn notes_delete(
    State(mut state): State<AppState>,
    Path(note_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    println!("notes_delete with note_id & user_id: {}, {}", note_id, user_id);

    let request = tonic::Request::new(DeleteNoteReq { id: note_id, user_id });
    let response = state.notes_client.delete_note(request).await?;
    let res_body = response.into_inner();

    new_ok_res(StatusCode::OK, res_body)
}
