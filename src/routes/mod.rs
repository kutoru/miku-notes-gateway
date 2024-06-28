use axum::{extract::State, http::{header, Method}, middleware::{self, Next}, response::Response, Router};
use axum_extra::extract::CookieJar;
use tonic::transport::Channel;
use tower_http::cors::CorsLayer;

use crate::proto::{auth::auth_client::AuthClient, notes::notes_client::NotesClient, tags::tags_client::TagsClient, files::files_client::FilesClient};
use crate::{error::ResError, proto::auth::ValidateAtRequest, types::AppState};

mod auth;
mod notes;
mod files;

pub async fn get_rpc_clients(auth_url: String, data_url: String) -> anyhow::Result<(
    AuthClient<Channel>,
    NotesClient<Channel>,
    // TagsClient<Channel>,
    FilesClient<Channel>,
)> {
    Ok((
        AuthClient::connect(auth_url).await?,
        NotesClient::connect(data_url.clone()).await?,
        FilesClient::connect(data_url).await?,
    ))
}

pub fn get_router(state: &AppState) -> anyhow::Result<Router> {
    let origins = [
        ("http://".to_owned() + &state.service_addr).parse()?,
        state.frontend_url.parse()?,
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_origin(origins)
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
        .allow_credentials(true);

    let auth_router = auth::get_router(state);
    let notes_router = notes::get_router(state);
    let files_router = files::get_router(state);

    Ok(
        Router::new()
            .merge(
                auth_router
            )
            .merge(
                notes_router.route_layer(middleware::from_fn_with_state(state.clone(), auth_mw))
            )
            .merge(
                files_router.route_layer(middleware::from_fn_with_state(state.clone(), auth_mw))
            )
            .layer(cors)
            .route_layer(middleware::from_fn(log_mw))
    )
}

async fn auth_mw(
    jar: CookieJar,
    State(mut state): State<AppState>,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, ResError> {
    let token = match jar.get(&state.access_token_key) {
        Some(c) => {
            println!("AT TOKEN: {:?}", c.value());
            c.value()
        },
        None => {
            println!("NO AT TOKEN");
            return Err(ResError::Unauthorized("Invalid creds".into()));
        },
    };

    let request = tonic::Request::new(ValidateAtRequest { access_token: token.into() });
    let response = state.auth_client.validate_access_token(request).await?;
    let user_id = response.into_inner().user_id;

    req.extensions_mut().insert(user_id as i32);
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
