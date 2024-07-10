use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::{json, Value};
use tower::ServiceExt;
use http_body_util::BodyExt;

use super::{get_app, authorize};

#[tokio::test]
async fn tags_get() {
    let mut app = get_app().await;
    let (at, _) = authorize(&mut app).await;

    let request = Request::builder()
        .uri("/tags")
        .header("cookie", at)
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let body: Value = serde_json::from_slice(&body).unwrap();
    let exp = json!({ "success": true, "error": None::<()>, "data": { "tags": [] } });

    assert_eq!(body, exp);
}
