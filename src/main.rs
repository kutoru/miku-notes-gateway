use crate::types::AppState;

mod types;
mod error;
mod proto;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let file_chunk_size = dotenvy::var("MAX_FILE_CHUNK_SIZE")?.parse()?;
    let rpc_clients = routes::get_rpc_clients(
        dotenvy::var("AUTH_URL")?,
        dotenvy::var("DATA_URL")?,
        file_chunk_size,
    ).await?;

    let state = AppState {
        log_level: dotenvy::var("LOG_LEVEL")?.parse()?,
        service_addr: dotenvy::var("SERVICE_ADDR")?,
        frontend_url: dotenvy::var("FRONTEND_URL")?,
        req_body_limit: dotenvy::var("MAX_REQUEST_BODY_SIZE")?.parse()?,
        file_chunk_size,

        access_token_exp: dotenvy::var("ACCESS_TOKEN_EXP")?.parse()?,
        refresh_token_exp: dotenvy::var("REFRESH_TOKEN_EXP")?.parse()?,
        access_token_key: dotenvy::var("ACCESS_TOKEN_KEY")?,
        refresh_token_key: dotenvy::var("REFRESH_TOKEN_KEY")?,

        auth_token: dotenvy::var("AUTH_TOKEN")?,
        data_token: dotenvy::var("DATA_TOKEN")?,

        auth_client: rpc_clients.0,
        notes_client: rpc_clients.1,
        tags_client: rpc_clients.2,
        files_client: rpc_clients.3,
        shelves_client: rpc_clients.4,
    };

    let app = routes::get_router(&state)?;
    let listener = tokio::net::TcpListener::bind(&state.service_addr).await?;

    println!("Listening on {}\n", state.service_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
