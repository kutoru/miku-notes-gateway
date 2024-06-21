use axum::{Router, http::{Method, header}, middleware::{Next, self}, response::Response};
use axum_extra::extract::CookieJar;
use tower_http::cors::CorsLayer;
use chrono::Local;
use anyhow::Result;

mod auth;
// mod notes;

pub fn get_router() -> Result<Router> {
    let origins = [
        ("http://".to_owned() + &std::env::var("SERVICE_ADDR")?).parse()?,
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_origin(origins)
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
        .allow_credentials(true);

    let auth_router = auth::get_router();
    // let notes_router = notes::get_router();

    Ok(
        Router::new()
            .merge(auth_router)
            // .merge(notes_router)
            .layer(cors)
            .route_layer(middleware::from_fn(route_logging))
    )
}

async fn route_logging(
    jar: CookieJar,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("\n{}:\n{} {}", now, req.method(), req.uri());

    println!("AT COOKIE: {:?}", jar.get("at"));

    // for key in req.headers().keys() {
    //     println!("{} = {}", key, req.headers().get(key).unwrap().to_str().unwrap());
    // }

    next.run(req).await
}
