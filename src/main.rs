mod result;
mod error;
mod proto;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Start");
    
    dotenvy::dotenv()?;

    let app = routes::get_router()?;
    let addr = std::env::var("SERVICE_ADDR")?;

    println!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    println!("End");

    Ok(())
}
