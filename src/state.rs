use crate::error::AppError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::client::conn::http1::SendRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Application state managing the tls connection to SVSM
#[derive(Clone)]
pub struct AppState {
    sender: Arc<Mutex<Option<SendRequest<Full<Bytes>>>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            sender: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets the sender for managing requests to SVSM
    pub async fn set_sender(&self, sender: SendRequest<Full<Bytes>>) {
        let mut lock = self.sender.lock().await;
        if lock.is_some() {
            println!("[RC] Warning: Overwriting existing sender");
        }
        *lock = Some(sender);
    }

    /// Clears the sender for managing requests to SVSM
    pub async fn clear_sender(&self) {
        let mut lock = self.sender.lock().await;
        *lock = None;
    }

    /// Sends a request to SVSM and returns the response
    pub async fn send_request(
        &self,
        req: hyper::Request<Full<Bytes>>,
    ) -> Result<hyper::Response<hyper::body::Incoming>, AppError> {
        let mut lock = self.sender.lock().await;
        let sender = match lock.as_mut() {
            Some(s) => s,
            None => return Err(AppError::StateError("No sender available".to_string())),
        };

        let Ok(result) =
            tokio::time::timeout(Duration::from_secs(5), sender.send_request(req)).await
        else {
            *lock = None; // Clear the sender on timeout
            println!("[RC] Timeout while sending request to SVSM");
            return Err(AppError::StateError(
                "Timeout while sending request".to_string(),
            ));
        };

        match result {
            Ok(response) => {
                println!("[RC] Received response from SVSM: {:?}", response);
                Ok(response)
            }
            Err(e) => {
                println!("[RC] Error sending request to SVSM: {:?}", e);
                Err(AppError::StateError(format!(
                    "Failed to send request: {e:?}"
                )))
            }
        }
    }

    /// Checks if the sender is set
    pub async fn has_sender(&self) -> bool {
        let lock = self.sender.lock().await;
        lock.is_some()
    }
}
