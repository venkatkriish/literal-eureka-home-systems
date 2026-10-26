use axum::serve;
use tokio::{net::TcpListener, signal};

use backend::{config::AppConfig, db::AppState, routes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;
    let state = AppState::seeded();
    let listener = TcpListener::bind(config.bind_addr).await?;
    let app = routes::router(state);
    serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
