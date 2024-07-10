use axum::{body::Body, http::Request, Router};
use serde_json::{json, Value};
use tower::Service;

use crate::{load_state, routes::get_router};

mod tags;

async fn get_app() -> Router {
    let mut state = load_state().await.expect("Could not load the app state");
    state.log_level = tracing::Level::ERROR;
    get_router(&state).expect("Could not get the app router")
}

fn new_body(json: Value) -> Body {
    Body::from(serde_json::to_vec(&json).unwrap())
}

/// Get a pair of cookie tokens for use in other requests
async fn authorize(app: &mut Router) -> (String, String) {
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(new_body(
            json!({ "email": "nexochan@mail.ru", "password": "1234" }),
        ))
        .expect("Could not build an auth request");

    let mut response = app
        .call(request)
        .await
        .expect("Could not get an auth response");

    let mut cookie_values = response
        .headers_mut()
        .get_all("set-cookie")
        .into_iter()
        .map(|v| v
            .to_str()
            .expect("Could not convert auth header value to string")
            .split(';')
            .next()
            .expect("Could not parse an auth cookies value")
            .to_string()
        );

    let cookie_1 = cookie_values.next();
    let cookie_2 = cookie_values.next();

    let (access_token, refresh_token) = match (cookie_1, cookie_2) {
        (Some(at), Some(rt)) if at.starts_with("at=") && rt.starts_with("rt=") => (at, rt),
        (Some(rt), Some(at)) if rt.starts_with("rt=") && at.starts_with("at=") => (at, rt),
        _ => panic!("Did not get the expected cookies, expected keys \"at\" and \"rt\""),
    };

    (access_token, refresh_token)
}
