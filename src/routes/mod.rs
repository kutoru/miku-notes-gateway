use axum::{Router, http::{Method, header}, middleware::{Next, self}, response::Response};
use axum_extra::extract::CookieJar;
use tower_http::cors::CorsLayer;

use crate::types::AppState;

mod auth;
// mod notes;

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
    // let notes_router = notes::get_router(state);

    Ok(
        Router::new()
            .merge(auth_router)
            // .merge(notes_router)
            .layer(cors)
            .route_layer(middleware::from_fn(middleware))
    )
}

async fn middleware(
    jar: CookieJar,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    // let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("\n{} -> {}", req.method(), req.uri());

    println!("AT COOKIE: {:?}", jar.get("at"));

    next.run(req).await
}
