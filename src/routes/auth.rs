use crate::{error::ResError, proto::auth::{GetAtRequest, LoginRequest, LogoutRequest, RegisterRequest}, types::{call_grpc_service, new_cookie_ok_res, Json}};
use crate::types::{CookieResult, AppState, CreateAndAddCookie};

use axum::{extract::State, routing::{get, post}, Router};
use axum_extra::extract::cookie::CookieJar;
use utoipa::OpenApi;

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/login", post(login_post))
        .route("/register", post(register_post))
        .route("/access", get(access_get))
        .route("/logout", get(logout_get))
        .with_state(state.clone())
}

#[derive(OpenApi)]
#[openapi(
    paths(login_post, register_post, access_get, logout_get),
    components(schemas(LoginRequest, RegisterRequest)),
)]
pub struct Api;

/// Log in
///
/// Log in as an existing user using credentials
#[utoipa::path(
    post, path = "login",
    responses(
        (status = 200, description = "Success", headers(("set-cookie", description = "Two cookies that include new access and refresh tokens"))),
        (status = 400, description = "The client did something wrong. Most likely the body format was incorrect"),
        (status = 415, description = "Request's content type was incorrect"),
        (status = 422, description = "There was something wrong with the request's body fields"),
        (status = "5XX", description = "Some internal server error that isn't the client's fault"),
    ),
    security(()),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
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

/// Register
///
/// Register as a new user using new credentials
#[utoipa::path(
    post, path = "register",
    responses(
        (status = 200, description = "Success", headers(("set-cookie", description = "Two cookies that include new access and refresh tokens"))),
        // (status = 400, description = "The client did something wrong. Most likely the body format was incorrect"),
        (status = 415, description = "Request's content type was incorrect"),
        (status = 422, description = "There was something wrong with the request's body fields"),
        (status = "5XX", description = "Some internal server error that isn't the client's fault"),
    ),
    security(()),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
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

/// Get a new access token
///
/// Once an existing access token expires, this route should be called to get a new one. If both access and refresh tokens expire, then the user will have to log in manually
#[utoipa::path(
    get, path = "access",
    responses(
        (status = 200, description = "Success", headers(("set-cookie", description = "Cookie that includes the new access token"))),
        (status = 401, description = "The refresh token is either missing or invalid"),
        (status = "5XX", description = "Some internal server error that isn't the client's fault"),
    ),
    security(("refresh_token" = [])),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn access_get(
    jar: CookieJar,
    State(mut state): State<AppState>,
) -> CookieResult {

    let fingerprint = "asdfasdf";

    let token = jar.get(&state.refresh_token_key)
        .ok_or(ResError::Unauthorized("Could not get a refresh token".into()))?
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

/// Log out
#[utoipa::path(
    get, path = "logout",
    responses(
        (status = 200, description = "Success", headers(("set-cookie", description = "Two cookies that erase access and refresh tokens"))),
        (status = 401, description = "The access token is either missing or invalid"),
        (status = "5XX", description = "Some internal server error that isn't the client's fault"),
    ),
    security(("access_token" = [])),
)]
#[tracing::instrument(skip(state), err(level = tracing::Level::DEBUG))]
async fn logout_get(
    jar: CookieJar,
    State(mut state): State<AppState>,
) -> CookieResult {

    let fingerprint = "asdfasdf";

    let token = jar.get(&state.access_token_key)
        .ok_or(ResError::Unauthorized("Could not get an access token".into()))?
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
