use crate::error::AppError;
use crate::state::AppState;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use rustls::ServerConfig;
use rustls::pki_types::CertificateDer;
use rustls::pki_types::PrivateKeyDer;
use rustls::pki_types::pem::PemObject;
use rustls::server::WebPkiClientVerifier;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

/// Reads the entire contents of a file asynchronously.
async fn read_file(path: &str) -> Result<Vec<u8>, AppError> {
    let mut file = File::open(path)
        .await
        .map_err(|e| AppError::FileError(e.to_string()))?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .await
        .map_err(|e| AppError::FileError(e.to_string()))?;
    Ok(contents)
}

/// Sets up a TLS server with client authentication
/// Update the AppState to manage the connection sender
pub async fn setup_tls_with_client_auth(state: AppState) -> Result<(), AppError> {
    println!("Connecting to SVSM via TLS...");
    let mut connection_set = JoinSet::new();

    // Load server certificate
    let pem_cert = read_file("../svsm/certificates/server.crt").await?;
    let certificate_der = CertificateDer::from_pem_slice(&pem_cert)?;
    let cert_chain = vec![certificate_der.clone()];

    // Load server private key
    let pem_key = read_file("../svsm/certificates/server.key").await?;
    let key = PrivateKeyDer::from_pem_slice(&pem_key)?;

    // Load CA certificate for client authentication
    let pem_ca = read_file("../svsm/certificates/ca.crt").await?;
    let certificate_der = CertificateDer::from_pem_slice(&pem_ca)?;
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add(certificate_der)?;

    let client_auth = WebPkiClientVerifier::builder(Arc::new(root_store)).build()?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_auth)
        .with_single_cert(cert_chain, key)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind("127.0.0.1:4433").await?;
    println!("[RC] TLS server listening on 127.0.0.1:4433");

    let (tcp_stream, addr) = listener.accept().await?;
    // TODO: handle multiple connections
    // a customer can have multiple svsm
    // let acceptor = acceptor.clone();
    println!("[RC] Accepted connection from {addr}");

    let Ok(tls_stream) = acceptor.accept(tcp_stream).await else {
        panic!("TLS handshake failed"); // todo: proper error handling
    };

    if let Some(cn) = get_cn_from_cert(&tls_stream) {
        println!("[RC] Client Common Name (CN): {cn}");
    } else {
        println!("[RC] Could not extract Common Name (CN) from client certificate");
    }

    let io = TokioIo::new(tls_stream);
    let (sender, connection) = http1::Builder::new()
        .handshake::<_, Full<Bytes>>(io)
        .await?;

    let state_clone = state.clone();
    connection_set.spawn(async move {
        if let Err(e) = connection.await {
            println!("[RC] Connection error: {e:?}");
        }
        state_clone.clear_sender().await;
        println!("[RC] Connection closed, sender cleared from AppState");
    });
    state.set_sender(sender).await;
    println!("[RC] TLS connection established and sender set in AppState");

    while let Some(res) = connection_set.join_next().await {
        match res {
            Ok(_) => println!("[RC] Connection task completed successfully"),
            Err(e) => println!("[RC] Connection task failed: {e:?}"),
        }
    }
    Ok(())
}

/// Extracts the Common Name (CN) from the client certificate in the TLS stream.
/// Can be used for determining which SVSM is connecting when multiple are supported.
fn get_cn_from_cert(tls_stream: &TlsStream<TcpStream>) -> Option<String> {
    let certs = tls_stream.get_ref().1.peer_certificates()?;
    if certs.is_empty() {
        return None;
    }

    let cert = &certs[0];
    let parsed_cert = x509_parser::parse_x509_certificate(cert).ok()?.1;

    let subject = &parsed_cert.tbs_certificate.subject;
    if let Some(attr) = subject.iter_common_name().next() {
        return Some(attr.as_str().ok()?.to_string());
    }
    None
}
