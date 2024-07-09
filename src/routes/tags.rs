use axum::{extract::{Path, State}, http::StatusCode, routing::{get, patch}, Extension, Router};
use utoipa::OpenApi;

use crate::{proto::tags::{CreateTagReq, DeleteTagReq, Empty, ReadTagsReq, Tag, TagList, UpdateTagReq}, types::{call_grpc_service, new_ok_res, AppState, ExRes400, ExRes5XX, ExRes415, ExRes404, ExRes401, ExRes422, Json, ServerResult}};

#[derive(OpenApi)]
#[openapi(
    paths(tags_get, tags_post, tags_patch, tags_delete),
    components(schemas(Tag, TagList, CreateTagReq, UpdateTagReq, Empty)),
    security(("access_token" = [])),
)]
pub struct Api;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/", get(tags_get).post(tags_post))
        .route("/:id", patch(tags_patch).delete(tags_delete))
        .with_state(state.clone())
}

/// Get user's tags
#[utoipa::path(
    get, path = "",
    responses(
        (status = 200, description = "Success", body = TagList),
        ExRes401, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn tags_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<TagList> {

    let tag_list = call_grpc_service(
        ReadTagsReq { user_id },
        |req| state.tags_client.read_tags(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, tag_list)
}

/// Create a tag
#[utoipa::path(
    post, path = "",
    request_body(content = CreateTagReq),
    responses(
        (status = 201, description = "Success", body = Tag),
        ExRes400, ExRes401, ExRes415, ExRes422, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn tags_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<CreateTagReq>,
) -> ServerResult<Tag> {

    body.user_id = user_id;

    let new_tag = call_grpc_service(
        body,
        |req| state.tags_client.create_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::CREATED, new_tag)
}

/// Update a tag
#[utoipa::path(
    patch, path = "/{tag_id}",
    request_body(content = UpdateTagReq),
    responses(
        (status = 200, description = "Success", body = Tag),
        ExRes400, ExRes401, ExRes404, ExRes415, ExRes422, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn tags_patch(
    State(mut state): State<AppState>,
    Path(tag_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<UpdateTagReq>,
) -> ServerResult<Tag> {

    body.id = tag_id;
    body.user_id = user_id;

    let updated_tag = call_grpc_service(
        body,
        |req| state.tags_client.update_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, updated_tag)
}

/// Delete a tag
#[utoipa::path(
    delete, path = "/{tag_id}",
    responses(
        (status = 200, description = "Success", body = Empty),
        ExRes400, ExRes401, ExRes404, ExRes5XX,
    ),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn tags_delete(
    State(mut state): State<AppState>,
    Path(tag_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    let res_body = call_grpc_service(
        DeleteTagReq { id: tag_id, user_id },
        |req| state.tags_client.delete_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}
