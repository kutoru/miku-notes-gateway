use axum::{Router, http::{Method, header, StatusCode}, middleware::{Next, self}, response::Response};
use axum_extra::extract::CookieJar;
use tower_http::cors::CorsLayer;

use crate::types::AppState;

mod auth;
mod notes;

pub fn get_router(state: &AppState) -> anyhow::Result<Router> {
    let origins = [
        ("http://".to_owned() + &state.service_addr).parse()?,
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_origin(origins)
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
        .allow_credentials(true);

    let auth_router = auth::get_router(state);
    let notes_router = notes::get_router(state);

    Ok(
        Router::new()
            .merge(
                auth_router
            )
            .merge(
                notes_router.route_layer(middleware::from_fn(auth_mw))
            )
            .layer(cors)
            .route_layer(middleware::from_fn(log_mw))
    )
}

async fn auth_mw(
    jar: CookieJar,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = match jar.get("at") {
        Some(c) => {
            println!("AT TOKEN: {:?}", c.value());
            c.value()
        },
        None => {
            println!("NO AT TOKEN");
            return Err(StatusCode::UNAUTHORIZED);
        },
    };

    // extract user id from the token
    let user_id = 2;

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}

async fn log_mw(
    req: axum::extract::Request,
    next: Next,
) -> Response {
    // let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("\nRequest: {} -> {}", req.method(), req.uri());
    next.run(req).await
}
