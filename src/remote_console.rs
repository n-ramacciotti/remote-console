use crate::api::app;
use crate::error::AppError;
use crate::state::AppState;
use crate::tls::setup_tls_with_client_auth;
use std::net::SocketAddr;

/// Run the remote console server
pub async fn run() -> Result<(), AppError> {
    let state: AppState = AppState::new();
    let state_clone = state.clone();

    tokio::spawn(async move {
        if let Err(e) = setup_tls_with_client_auth(state_clone).await {
            println!("[RC] TLS server error: {e:?}");
        }
    });

    let app = app(state)?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
