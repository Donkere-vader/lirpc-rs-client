#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("SerdeError: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Error receiving message from oneshot channel: {0}")]
    TokioRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error(
        "The address '{0}' is not a valid address and couldn't be converted to a servername for use with PKI"
    )]
    InvalidAddress(String),
    #[error("WebsocketError: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("ServerError: {error}: {detail}")]
    Server { error: String, detail: String },
}
