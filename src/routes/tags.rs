use axum::{extract::{Path, State}, http::StatusCode, routing::{get, patch}, Extension, Router};

use crate::{proto::tags::{CreateTagReq, DeleteTagReq, Empty, ReadTagsReq, Tag, TagList, UpdateTagReq}, types::{call_grpc_service, new_ok_res, AppState, Json, ServerResult}};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/tags", get(tags_get).post(tags_post))
        .route("/tags/:id", patch(tags_patch).delete(tags_delete))
        .with_state(state.clone())
}

async fn tags_get(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<TagList> {

    println!("tags_get with user_id: {}", user_id);

    let tag_list = call_grpc_service(
        ReadTagsReq { user_id },
        |req| state.tags_client.read_tags(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, tag_list)
}

async fn tags_post(
    State(mut state): State<AppState>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<CreateTagReq>,
) -> ServerResult<Tag> {

    println!("tags_post with user_id & body: {}, {:?}", user_id, body);

    body.user_id = user_id;

    let new_tag = call_grpc_service(
        body,
        |req| state.tags_client.create_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::CREATED, new_tag)
}

async fn tags_patch(
    State(mut state): State<AppState>,
    Path(tag_id): Path<i32>,
    Extension(user_id): Extension<i32>,
    Json(mut body): Json<UpdateTagReq>,
) -> ServerResult<Tag> {

    println!("tags_patch with user_id, tag_id, body: {}, {}, {:?}", user_id, tag_id, body);

    body.id = tag_id;
    body.user_id = user_id;

    let updated_tag = call_grpc_service(
        body,
        |req| state.tags_client.update_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, updated_tag)
}

async fn tags_delete(
    State(mut state): State<AppState>,
    Path(tag_id): Path<i32>,
    Extension(user_id): Extension<i32>,
) -> ServerResult<Empty> {

    println!("tags_delete with user_id, tag_id: {}, {}", user_id, tag_id);

    let res_body = call_grpc_service(
        DeleteTagReq { id: tag_id, user_id },
        |req| state.tags_client.delete_tag(req),
        &state.data_token,
    ).await?;

    new_ok_res(StatusCode::OK, res_body)
}
