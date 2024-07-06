use crate::{error::ResError, proto::auth::{GetAtRequest, LoginRequest, LogoutRequest, RegisterRequest}, types::{call_grpc_service, new_cookie_ok_res, Json}};
use crate::types::{CookieResult, AppState, CreateAndAddCookie};

use axum::{extract::State, routing::{get, post}, Router};
use axum_extra::extract::cookie::CookieJar;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/login", post(login_post))
        .route("/register", post(register_post))
        .route("/access", get(access_get))
        .route("/logout", get(logout_get))
        .with_state(state.clone())
}

async fn login_post(
    jar: CookieJar,
    State(mut state): State<AppState>,
    Json(mut body): Json<LoginRequest>,
) -> CookieResult {

    // in the future, the frontend will generate a fingerprint and pass it in the headers

    let fingerprint = "asdfasdf";
    body.fingerprint = fingerprint.into();

    // calling the grpc auth api

    let res_body = call_grpc_service(
        body,
        |req| state.auth_client.login(req),
        &state.auth_token,
    ).await?;

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
    Json(mut body): Json<RegisterRequest>,
) -> CookieResult {

    let fingerprint = "asdfasdf";
    body.fingerprint = fingerprint.into();

    let res_body = call_grpc_service(
        body,
        |req| state.auth_client.register(req),
        &state.auth_token,
    ).await?;

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

    let fingerprint = "asdfasdf";

    let token = jar.get(&state.refresh_token_key)
        .ok_or(ResError::Unauthorized("Unauthorized".into()))?
        .value();

    let res_body = call_grpc_service(
        GetAtRequest { refresh_token: token.into(), fingerprint: fingerprint.into() },
        |req| state.auth_client.get_access_token(req),
        &state.auth_token,
    ).await?;

    new_cookie_ok_res(
        jar
            .add_new_cookie(state.access_token_key, res_body.access_token, state.access_token_exp)
    )
}

async fn logout_get(
    jar: CookieJar,
    State(mut state): State<AppState>,
) -> CookieResult {

    let fingerprint = "asdfasdf";

    let token = jar.get(&state.access_token_key)
        .ok_or(ResError::Unauthorized("Unauthorized".into()))?
        .value();

    call_grpc_service(
        LogoutRequest { access_token: token.into(), fingerprint: fingerprint.into() },
        |req| state.auth_client.logout(req),
        &state.auth_token,
    ).await?;

    new_cookie_ok_res(
        jar
            .add_new_cookie(state.refresh_token_key, "".into(), 0)
            .add_new_cookie(state.access_token_key, "".into(), 0)
    )
}
