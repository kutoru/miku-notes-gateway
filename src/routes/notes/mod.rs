use crate::proto::files::File;
use crate::proto::notes::{AttachTagReq, CreateNoteReq, DeleteNoteReq, DetachTagReq, Empty, Note, NoteList, UpdateNoteReq};
use crate::proto::tags::Tag;
use crate::types::{call_grpc_service, new_ok_res, AppState, ExRes400, ExRes401, ExRes404, ExRes415, ExRes422, ExRes5XX, Json, ServerResult};

use axum::extract::Query;
use axum::routing::{delete, post};
use axum::{Router, routing::{patch, get}, extract::{State, Path}, http::StatusCode, Extension};
use utoipa::OpenApi;

use helpers::{parse_note_query, NoteQuery};
mod helpers;

#[derive(OpenApi)]
#[openapi(
    paths(notes_get, notes_post, notes_patch, notes_delete, notes_tag_post, notes_tag_delete),
    components(schemas(File, Tag, Note, NoteList, CreateNoteReq, UpdateNoteReq, Empty, AttachTagReq)),
    security(("access_token" = [])),
)]
pub struct Api;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/", get(notes_get).post(notes_post))
        .route("/:id", patch(notes_patch).delete(notes_delete))
        .route("/:id/tag", post(notes_tag_post))
        .route("/:id/tag/:id", delete(notes_tag_delete))
        .with_state(state.clone())
}

/// Get user's notes
#[utoipa::path(
    get, path = "",
    params(
        ("page" = Option<i32>, Query, description = "Which page number to get. v > 0."),
        ("per_page" = Option<i32>, Query, description = "How many notes to get per page. v > 0 && v <= 100."),
        ("sort_by" = Option<String>, Query, description = "By which field to sort the notes. v can be one of: `date`, `date_modif`, `title`."),
        ("sort_type" = Option<String>, Query, description = "How to sort the notes. v can be one of: `asc`, `desc`."),
        ("tags" = Option<String>, Query, description = "List of tag ids to filter the notes by. v must follow the `(\\d*,)*` regex.<br>You can also get list of notes that don't have any tags attached to them, by specifying this parameter but leaving its value empty.<br>**Example**: `352,853,9235,`"),
        ("date" = Option<String>, Query, description = "Date range, in the form of two unix integers, to filter the notes by. v must follow the `(\\d+)-(\\d+)` regex. The range is inclusive on both ends.<br>The first integer in the range can go down to 0, which will ignore the lower boundary. Same thing for the second integer: when it's 0, the upper boundary will be ignored.<br>**Example**: `1695988727-0`"),
        ("date_modif" = Option<String>, Query, description = "Date modified range, in the form of two unix integers, to filter the notes by. v must follow the same rules as the `date` parameter.<br>**Example**: `1709985600-1725105639`"),
        ("title" = Option<String>, Query, description = "Filter the notes by checking if their title contains this parameter's value"),
    ),
    responses(
        (status = 200, description = "Success", body = NoteList),
        ExRes400, ExRes401, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn notes_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Query(query): Query<NoteQuery>,
) -> ServerResult<NoteList> {

    let body = parse_note_query(user_id, query)?;

    let note_list = call_grpc_service(
        body,
        |req| state.notes_client.read_notes(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, note_list)
}

/// Create a note
///
/// Note that you cannot attach files or tags during note creation. First, you have to create a note, and then call the respective attach routes
#[utoipa::path(
    post, path = "",
    request_body(content = CreateNoteReq),
    responses(
        (status = 201, description = "Success", body = Note),
        ExRes400, ExRes401, ExRes415, ExRes422, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn notes_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<CreateNoteReq>,
) -> ServerResult<Note> {

    body.user_id = user_id;

    let new_note = call_grpc_service(
        body,
        |req| state.notes_client.create_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::CREATED, new_note)
}

/// Update a note
#[utoipa::path(
    patch, path = "/{note_id}",
    request_body(content = UpdateNoteReq),
    responses(
        (status = 200, description = "Success", body = Note),
        ExRes400, ExRes401, ExRes404, ExRes415, ExRes422, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn notes_patch(
    State(mut state): State<AppState>,
    Path(note_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<UpdateNoteReq>,
) -> ServerResult<Note> {

    body.id = note_id;
    body.user_id = user_id;

    let updated_note = call_grpc_service(
        body,
        |req| state.notes_client.update_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, updated_note)
}

/// Delete a note
#[utoipa::path(
    delete, path = "/{note_id}",
    responses(
        (status = 200, description = "Success", body = Empty),
        ExRes400, ExRes401, ExRes404, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn notes_delete(
    State(mut state): State<AppState>,
    Path(note_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    let res_body = call_grpc_service(
        DeleteNoteReq { id: note_id, user_id },
        |req| state.notes_client.delete_note(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}

/// Add tag to a note
#[utoipa::path(
    post, path = "/{note_id}/tag",
    request_body(content = AttachTagReq),
    responses(
        (status = 200, description = "Success", body = Empty),
        ExRes400, ExRes401, ExRes404, ExRes415, ExRes422, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn notes_tag_post(
    State(mut state): State<AppState>,
    Path(note_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<AttachTagReq>,
) -> ServerResult<Empty> {

    body.note_id = note_id;
    body.user_id = user_id;

    let res_body = call_grpc_service(
        body,
        |req| state.notes_client.attach_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}

/// Remove tag from a note
#[utoipa::path(
    delete, path = "/{note_id}/tag/{tag_id}",
    responses(
        (status = 200, description = "Success", body = Empty),
        ExRes400, ExRes401, ExRes404, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn notes_tag_delete(
    State(mut state): State<AppState>,
    Path((note_id, tag_id)): Path<(i32, i32)>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    let res_body = call_grpc_service(
        DetachTagReq { user_id, note_id, tag_id },
        |req| state.notes_client.detach_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}
