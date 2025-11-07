use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::get};
use core::result::Result;
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper::body::Bytes;
use serde::Serialize;

/// API response structure
#[derive(Serialize)]
struct Response {
    data: String,
}

/// Sends a request to the SVSM and returns the response body as a vector of bytes.
async fn request_to_svsm(state: &AppState, req: Request<Full<Bytes>>) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();

    let mut resp = state.send_request(req).await?;

    while let Some(next) = resp.frame().await {
        let Ok(chunk) = next else {
            let e = format!("Error reading chunk: {:?}", next.err());
            return Err(AppError::StateError(e));
        };

        let data = chunk
            .into_data()
            .map_err(|_| AppError::HttpError("Error converting chunk to data".to_string()))?;
        buf.extend_from_slice(&data);
    }

    Ok(buf)
}

/// Health check endpoint to verify if the connection to SVSM is established.
async fn health_check(State(state): State<AppState>) -> Json<Response> {
    if !state.has_sender().await {
        Json(Response {
            data: false.to_string(),
        })
    } else {
        Json(Response {
            data: true.to_string(),
        })
    }
}

/// Endpoint to reboot the guest system via SVSM.
async fn reboot_guest(State(state): State<AppState>) -> Json<Response> {
    let req = Request::builder()
        .method("POST")
        .uri("http://localhost/reboot")
        .header("Host", "localhost")
        .header("Content-Length", "7")
        .body(Full::<Bytes>::from("Reboot!"))
        .expect("Failed to build request");

    match request_to_svsm(&state, req).await {
        Ok(response) => Json(Response {
            data: String::from_utf8(response).unwrap(),
        }),
        Err(err) => Json(Response {
            data: format!("Error sending request: {err}"),
        }),
    }
}

/// Endpoint to retrieve logs from the SVSM.
async fn get_log(State(state): State<AppState>) -> Json<Response> {
    let req = Request::builder()
        .method("POST")
        .uri("http://localhost/get_log")
        .header("Host", "localhost")
        .header("Content-Length", "6")
        .body(Full::<Bytes>::from("Hello!"))
        .expect("Failed to build request");

    match request_to_svsm(&state, req).await {
        Ok(response) => Json(Response {
            data: String::from_utf8(response).unwrap(),
        }),
        Err(err) => Json(Response {
            data: format!("Error sending request: {err}"),
        }),
    }
}

/// Constructs the API router with defined routes and state(tls connection).
pub fn app(state: AppState) -> Result<Router, AppError> {
    let app = Router::new()
        .route("/api/health_check", get(health_check))
        .route("/api/get_log", get(get_log))
        .route("/api/reboot_guest", get(reboot_guest))
        .fallback_service(tower_http::services::ServeDir::new("static"))
        .with_state(state);
    Ok(app)
}
