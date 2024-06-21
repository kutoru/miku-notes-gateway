use crate::proto::notes::notes_client::NotesClient;
use crate::proto::notes::{ReadNotesReq, CreateNoteReq, UpdateNoteReq, DeleteNoteReq, Note, NoteList};
use crate::{types::{AppState, ServerResult, ResultBody}, res};

use axum::{Router, routing::{patch, get}, extract::{State, Path}, Json, http::StatusCode, Extension};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/notes", get(notes_get).post(notes_post))
        .route("/notes/:id", patch(notes_patch).delete(notes_delete))
        .with_state(state.clone())
}

async fn notes_get(
    State(state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<NoteList> {

    println!("notes_get with user_id: {}", user_id);

    let mut client = NotesClient::connect(state.data_url).await?;
    let request = tonic::Request::new(ReadNotesReq { user_id });

    let response = client.read_notes(request).await?;
    let note_list = response.into_inner();

    res!(StatusCode::OK, true, None, Some(note_list))
}

async fn notes_post(
    State(state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<CreateNoteReq>,
) -> ServerResult<Note> {

    println!("notes_post with user_id and body: {}, {:#?}", user_id, body);
    body.user_id = user_id;
    println!("Modified body: {:#?}", body);

    let mut client = NotesClient::connect(state.data_url).await?;
    let request = tonic::Request::new(body);

    let response = client.create_note(request).await?;
    let new_note = response.into_inner();

    res!(StatusCode::OK, true, None, Some(new_note))
}

async fn notes_patch(
    State(state): State<AppState>,
    Path(note_id): Path<i32>,
    Json(body): Json<UpdateNoteReq>,
) -> ServerResult<()> {
    res!(StatusCode::NOT_IMPLEMENTED, false, None, None)
}

async fn notes_delete(
    State(state): State<AppState>,
    Path(note_id): Path<i32>,
) -> ServerResult<()> {
    res!(StatusCode::NOT_IMPLEMENTED, false, None, None)
}
