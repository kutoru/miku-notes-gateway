use crate::{error::ResError, proto::auth::{GetAtRequest, LoginRequest, RegisterRequest}, types::new_cookie_ok_res};
use crate::types::{CookieResult, AppState, CreateAndAddCookie};

use axum::{extract::State, routing::{get, post}, Json, Router};
use axum_extra::extract::cookie::CookieJar;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/login", post(login_post))
        .route("/register", post(register_post))
        .route("/access", get(access_get))
        .with_state(state.clone())
}

async fn login_post(
    jar: CookieJar,
    State(mut state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> CookieResult {

    // calling the grpc auth api

    let request = tonic::Request::new(body);
    let response = state.auth_client.login(request).await?;
    let res_body = response.into_inner();

    // sending the cookies

    new_cookie_ok_res(
        jar
            .add_new_cookie(state.refresh_token_key, res_body.refresh_token, state.refresh_token_exp)
            .add_new_cookie(state.access_token_key, res_body.access_token, state.access_token_exp)
    )
}

async fn register_post(
    jar: CookieJar,
    State(mut state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> CookieResult {

    let request = tonic::Request::new(body);
    let response = state.auth_client.register(request).await?;
    let res_body = response.into_inner();

    new_cookie_ok_res(
        jar
            .add_new_cookie(state.refresh_token_key, res_body.refresh_token, state.refresh_token_exp)
            .add_new_cookie(state.access_token_key, res_body.access_token, state.access_token_exp)
    )
}

async fn access_get(
    jar: CookieJar,
    State(mut state): State<AppState>,
) -> CookieResult {

    let token = jar.get(&state.refresh_token_key)
        .ok_or(ResError::Unauthorized("Invalid creds".into()))?
        .value();

    let request = tonic::Request::new(GetAtRequest { refresh_token: token.into() });
    let response = state.auth_client.get_access_token(request).await?;
    let res_body = response.into_inner();

    new_cookie_ok_res(
        jar
            .add_new_cookie(state.access_token_key, res_body.access_token, state.access_token_exp)
    )
}
