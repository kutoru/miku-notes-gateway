use crate::types::AppState;

mod types;
mod error;
mod proto;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let state = load_state().await?;
    let addr = format!("127.0.0.1:{}", state.service_port);
    let app = routes::get_router(&state)?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("Gateway service listening on {addr}\n");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn load_state() -> anyhow::Result<AppState> {
    let file_chunk_size = dotenvy::var("MAX_FILE_CHUNK_SIZE")?.parse()?;

    let rpc_clients = routes::get_rpc_clients(
        dotenvy::var("AUTH_URL")?,
        dotenvy::var("DATA_URL")?,
        file_chunk_size,
    ).await?;

    Ok(AppState {
        log_level: dotenvy::var("LOG_LEVEL")?.parse()?,
        service_port: dotenvy::var("SERVICE_PORT")?.parse()?,
        frontend_url: dotenvy::var("FRONTEND_URL")?,
        req_body_limit: dotenvy::var("MAX_REQUEST_BODY_SIZE")?.parse()?,
        file_chunk_size,

        auth_token: dotenvy::var("AUTH_TOKEN")?,
        data_token: dotenvy::var("DATA_TOKEN")?,

        access_token_ttl: dotenvy::var("ACCESS_TOKEN_TTL")?.parse()?,
        refresh_token_ttl: dotenvy::var("REFRESH_TOKEN_TTL")?.parse()?,
        access_token_key: dotenvy::var("ACCESS_TOKEN_KEY")?,
        refresh_token_key: dotenvy::var("REFRESH_TOKEN_KEY")?,

        auth_client: rpc_clients.0,
        notes_client: rpc_clients.1,
        tags_client: rpc_clients.2,
        files_client: rpc_clients.3,
        shelves_client: rpc_clients.4,
    })
}
