use crate::types::AppState;

mod types;
mod error;
mod proto;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let rpc_clients = routes::get_rpc_clients(
        dotenvy::var("AUTH_URL")?,
        dotenvy::var("DATA_URL")?,
    ).await?;

    let state = AppState {
        service_addr: dotenvy::var("SERVICE_ADDR")?,
        frontend_url: dotenvy::var("FRONTEND_URL")?,
        access_token_exp: dotenvy::var("ACCESS_TOKEN_EXP")?.parse()?,
        refresh_token_exp: dotenvy::var("REFRESH_TOKEN_EXP")?.parse()?,
        req_body_limit: dotenvy::var("MAX_REQUEST_BODY_SIZE_IN_MB")?.parse()?,
        file_chunk_size: dotenvy::var("MAX_FILE_CHUNK_SIZE_IN_MB")?.parse()?,

        auth_client: rpc_clients.0,
        notes_client: rpc_clients.1,
        // tags_client: rpc_clients.2,
        files_client: rpc_clients.2,
    };

    let app = routes::get_router(&state)?;
    let listener = tokio::net::TcpListener::bind(&state.service_addr).await?;

    println!("Listening on {}", state.service_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
