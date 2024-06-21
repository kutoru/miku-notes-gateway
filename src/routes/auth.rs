use crate::proto::auth::auth_client::AuthClient;
use crate::proto::auth::{LoginRequest, RegisterRequest};

use axum::{Router, routing::post, http::StatusCode, Json, extract::State};
use axum_extra::extract::cookie::{CookieJar, Cookie, SameSite};

use crate::{jar_res, types::{ResultBody, CookieResult, AppState}};

pub fn get_router(state: &AppState) -> Router {
    Router::new()
        .route("/login", post(login_post))
        .route("/register", post(register_post))
        .with_state(state.clone())
}

async fn login_post(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> CookieResult {

    // calling the grpc auth api

    let mut client = AuthClient::connect(state.auth_url).await?;
    let request = tonic::Request::new(LoginRequest {
        email: body.email,
        password: body.password,
    });

    let response = client.login(request).await?;
    println!("grpc login response = {:#?}", response);
    let token = response.into_inner().token;

    // sending the cookie

    let jar = add_cookie(jar, token, state.token_exp);
    jar_res!(StatusCode::OK, jar, true, None)
}

async fn register_post(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> CookieResult {

    let mut client = AuthClient::connect(state.auth_url).await?;
    let request = tonic::Request::new(RegisterRequest {
        email: body.email,
        password: body.password,
    });

    let response = client.register(request).await?;
    println!("grpc register response = {:#?}", response);
    let token = response.into_inner().token;

    let jar = add_cookie(jar, token, state.token_exp);
    jar_res!(StatusCode::OK, jar, true, None)
}

fn add_cookie(jar: CookieJar, token: String, token_exp: i64) -> CookieJar {
    let exp_time = time::Duration::seconds(token_exp);

    let cookie = Cookie::build(("at", token))
        .max_age(exp_time)
        .path("/")
        .same_site(SameSite::None)
        .http_only(true)
        .secure(false);

    jar.add(cookie)
}
