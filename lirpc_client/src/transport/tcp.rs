use std::sync::Arc;

use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use rustls::pki_types::ServerName;
use serde::Serialize;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc,
};
use tokio_rustls::{TlsConnector, client::TlsStream};
use tokio_util::{
    bytes::Bytes,
    codec::{Framed, LengthDelimitedCodec},
};
use tracing::error;

use crate::{
    error::Error, serializers::Serializer, serializers::bytes_serializer::BytesSerializer,
    transport::Transport,
};

/// Maximum size (in bytes) of a single length-prefixed TCP frame, guarding
/// against a malformed/oversized length prefix causing an unbounded allocation.
const MAX_TCP_FRAME_LENGTH: usize = 8 * 1024 * 1024;

pub struct Tcp<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    sender: SplitSink<Framed<S, LengthDelimitedCodec>, Bytes>,
}

impl Tcp<TcpStream> {
    pub async fn connect(
        address: impl ToSocketAddrs,
        forward_to: mpsc::Sender<Bytes>,
    ) -> Result<Self, Error> {
        let stream = TcpStream::connect(address).await?;
        Self::setup_with_stream(stream, forward_to).await
    }
}

impl Tcp<TlsStream<TcpStream>> {
    pub async fn connect_tls(
        address: String,
        forward_to: mpsc::Sender<Bytes>,
    ) -> Result<Self, Error> {
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));

        let tcp_stream = TcpStream::connect(&address).await?;
        let domain = ServerName::try_from(address.to_string())
            .map_err(|_| Error::InvalidAddress(address))?
            .to_owned();
        let tls_stream = connector.connect(domain, tcp_stream).await?;

        Self::setup_with_stream(tls_stream, forward_to).await
    }
}

impl<S> Tcp<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    async fn setup_with_stream(stream: S, forward_to: mpsc::Sender<Bytes>) -> Result<Self, Error> {
        let framed = LengthDelimitedCodec::builder()
            .max_frame_length(MAX_TCP_FRAME_LENGTH)
            .new_framed(stream);

        let (sender, receiver) = framed.split();

        tokio::spawn(async move { Tcp::forward_messages(receiver, forward_to).await });

        Ok(Self { sender })
    }

    async fn forward_messages(
        mut receiver: SplitStream<Framed<S, LengthDelimitedCodec>>,
        forward_to: mpsc::Sender<Bytes>,
    ) {
        while let Some(msg) = receiver.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    error!("Error receiving TCP frame: {e}");
                    return;
                }
            };

            if let Err(e) = forward_to.send(msg.into()).await {
                error!("Error forwarding message from TCP transport to Client: {e}");
            };
        }
    }
}

impl<S> Transport<Bytes> for Tcp<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Serializer = BytesSerializer;

    async fn send(&mut self, message: impl Serialize) -> Result<(), Error> {
        let raw_message = Self::Serializer::serialize(message)?;
        Ok(self.sender.send(raw_message).await?)
    }
}
