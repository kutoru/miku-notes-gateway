use crate::types::AppState;

mod types;
mod error;
mod proto;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let state = AppState {
        service_addr: dotenvy::var("SERVICE_ADDR")?,
        auth_url: dotenvy::var("AUTH_URL")?,
        data_url: dotenvy::var("DATA_URL")?,
        token_exp: dotenvy::var("TOKEN_EXP")?.parse()?,
    };

    let app = routes::get_router(&state)?;
    let listener = tokio::net::TcpListener::bind(&state.service_addr).await?;

    println!("Listening on {}", state.service_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
