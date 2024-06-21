use axum::{Router, routing::post, http::StatusCode, Json};
use axum_extra::extract::cookie::{CookieJar, Cookie, SameSite};
use anyhow::Result;

use crate::{jar_res, result::{ResultBody, CookieResult}};

pub mod auth_api {
    tonic::include_proto!("auth");
}

pub fn get_router() -> Router {
    Router::new()
        .route("/login", post(login_post))
        .route("/register", post(register_post))
}

async fn login_post(
    jar: CookieJar,
    Json(body): Json<auth_api::LoginRequest>,
) -> CookieResult {

    // calling the grpc auth api

    let mut client = auth_api::auth_client::AuthClient::connect(std::env::var("AUTH_URL")?).await?;
    let request = tonic::Request::new(auth_api::LoginRequest {
        email: body.email,
        password: body.password,
    });

    let response = client.login(request).await?;
    println!("grpc login response = {:#?}", response);
    let token = response.into_inner().token;

    // sending the cookie

    let jar = add_cookie(jar, token)?;
    jar_res!(StatusCode::OK, jar, true, None)
}

async fn register_post(
    jar: CookieJar,
    Json(body): Json<auth_api::LoginRequest>,
) -> CookieResult {

    let mut client = auth_api::auth_client::AuthClient::connect(std::env::var("AUTH_URL")?).await?;
    let request = tonic::Request::new(auth_api::RegisterRequest {
        email: body.email,
        password: body.password,
    });

    let response = client.register(request).await?;
    println!("grpc register response = {:#?}", response);
    let token = response.into_inner().token;

    let jar = add_cookie(jar, token)?;
    jar_res!(StatusCode::OK, jar, true, None)
}

fn add_cookie(jar: CookieJar, token: String) -> Result<CookieJar> {
    let exp_time = time::Duration::seconds(
        std::env::var("TOKEN_EXP")?.parse()?
    );

    let cookie = Cookie::build(("at", token))
        .max_age(exp_time)
        .path("/")
        .same_site(SameSite::None)
        .http_only(true)
        .secure(false);

    Ok(jar.add(cookie))
}
