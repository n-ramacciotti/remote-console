use rustls::server::VerifierBuilderError;

// todo: review error types and messages
/// Application errors
#[derive(Debug)]
pub enum AppError {
    FileError(String),
    StateError(String),
    HttpError(String),
    TlsError(String),
    IoError(String),
    VerifierBuilderError(String),
    HyperError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::FileError(msg) => write!(f, "File error: {msg}"),
            AppError::StateError(msg) => write!(f, "State error: {msg}"),
            AppError::HttpError(msg) => write!(f, "HTTP error: {msg}"),
            AppError::TlsError(msg) => write!(f, "TLS error: {msg}"),
            AppError::IoError(msg) => write!(f, "IO error: {msg}"),
            AppError::VerifierBuilderError(msg) => write!(f, "Verifier Builder error: {msg}"),
            AppError::HyperError(msg) => write!(f, "Hyper error: {msg}"),
        }
    }
}

impl From<rustls::Error> for AppError {
    fn from(err: rustls::Error) -> Self {
        AppError::TlsError(err.to_string())
    }
}

impl From<rustls::pki_types::pem::Error> for AppError {
    fn from(err: rustls::pki_types::pem::Error) -> Self {
        AppError::TlsError(err.to_string())
    }
}

impl From<VerifierBuilderError> for AppError {
    fn from(err: VerifierBuilderError) -> Self {
        AppError::VerifierBuilderError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

impl From<hyper::Error> for AppError {
    fn from(err: hyper::Error) -> Self {
        AppError::HyperError(err.to_string())
    }
}
