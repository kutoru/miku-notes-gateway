use axum::{extract::State, http::{header, Method}, middleware::{self, Next}, response::Response, Router};
use axum_extra::extract::CookieJar;
use rand::{thread_rng, Rng};
use tonic::transport::Channel;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{proto::{auth::auth_client::AuthClient, files::files_client::FilesClient, notes::notes_client::NotesClient, shelves::shelves_client::ShelvesClient, tags::tags_client::TagsClient}, types::call_grpc_service};
use crate::{error::ResError, proto::auth::ValidateAtRequest, types::AppState};

mod auth;
mod notes;
mod tags;
mod files;
mod shelves;

pub async fn get_rpc_clients(auth_url: String, data_url: String, max_chunk_size: usize) -> anyhow::Result<(
    AuthClient<Channel>,
    NotesClient<Channel>,
    TagsClient<Channel>,
    FilesClient<Channel>,
    ShelvesClient<Channel>,
)> {
    Ok((
        AuthClient::connect(auth_url).await?,
        NotesClient::connect(data_url.clone()).await?,
        TagsClient::connect(data_url.clone()).await?,
        FilesClient::connect(data_url.clone()).await?
            .max_decoding_message_size(1024 * 1024 * (max_chunk_size + 1)),
        ShelvesClient::connect(data_url).await?,
    ))
}

pub fn get_router(state: &AppState) -> anyhow::Result<Router> {
    let origins = [
        format!("http://{}", state.service_addr).parse()?,
        state.frontend_url.parse()?,
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_origin(origins)
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
        .allow_credentials(true);

    let auth_router = auth::get_router(state);
    let notes_router = notes::get_router(state);
    let tags_router = tags::get_router(state);
    let files_router = files::get_router(state);
    let shelves_router = shelves::get_router(state);

    setup_tracing(&state.log_level);

    Ok(
        Router::new()
            .merge(notes_router)
            .merge(tags_router)
            .merge(files_router)
            .merge(shelves_router)
            .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .merge(auth_router)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &axum::extract::Request<_>| {
                        info_span!(
                            "request",
                            req_id = thread_rng().gen_range(10000..100000),
                            method = ?request.method(),
                            uri = ?request.uri(),
                        )
                    })
                    .on_response(|response: &Response, _, _: &_| {
                        info!(response_code = response.status().as_u16());
                    })
            )
    )
}

async fn auth_middleware(
    jar: CookieJar,
    State(mut state): State<AppState>,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, ResError> {
    let token = match jar.get(&state.access_token_key) {
        Some(c) => c.value(),
        None => return Err(ResError::Unauthorized("Could not get access token from the cookie jar".into())),
    };

    let res_body = call_grpc_service(
        ValidateAtRequest { access_token: token.into() },
        |req| state.auth_client.validate_access_token(req),
        &state.auth_token,
    ).await?;

    req.extensions_mut().insert(res_body.user_id);
    Ok(next.run(req).await)
}

fn setup_tracing(log_level: &tracing::Level) {

    // https://stackoverflow.com/questions/70013172/how-to-use-the-tracing-library

    let format = time::format_description::parse(
        "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]",
    ).unwrap();
    let offset = time::UtcOffset::current_local_offset()
        .unwrap_or(time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(offset, format);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new::<&str>(
                &format!("RUST_LOG=error,miku_notes_gateway={log_level}"),
            )
        )
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_timer(timer)
                .with_file(true)
                // .with_target(false)
        )
        .init()
}
