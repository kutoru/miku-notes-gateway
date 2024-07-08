use axum::{extract::{Path, State}, http::StatusCode, routing::{get, patch}, Extension, Router};

use crate::{proto::tags::{CreateTagReq, DeleteTagReq, Empty, ReadTagsReq, Tag, TagList, UpdateTagReq}, types::{call_grpc_service, new_ok_res, AppState, Json, ServerResult}};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/tags", get(tags_get).post(tags_post))
        .route("/tags/:id", patch(tags_patch).delete(tags_delete))
        .with_state(state.clone())
}

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
