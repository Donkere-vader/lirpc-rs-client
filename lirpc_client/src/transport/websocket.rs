use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::Serialize;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use tracing::error;

use crate::{
    error::Error,
    serializers::{Serializer, string_serializer::StringSerializer},
    transport::Transport,
};

pub struct Websocket {
    sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl Websocket {
    pub async fn connect(url: &str, forward_to: mpsc::Sender<String>) -> Result<Self, Error> {
        let (stream, _) = connect_async(url).await?;
        let (sender, receiver) = stream.split();

        tokio::spawn(async move { Self::forward_messages(receiver, forward_to).await });

        Ok(Self { sender })
    }

    async fn forward_messages(
        mut receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        forward_to: mpsc::Sender<String>,
    ) {
        while let Some(msg) = receiver.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    error!("Error receiving websocket message: {e}");
                    return;
                }
            };

            let text = match msg {
                Message::Text(text) => text.to_string(),
                Message::Close(_) => return,
                _ => continue,
            };

            if let Err(e) = forward_to.send(text).await {
                error!("Error forwarding message from Websocket transport to Client: {e}");
            };
        }
    }
}

impl Transport<String> for Websocket {
    type Serializer = StringSerializer;

    async fn send(&mut self, message: impl Serialize) -> Result<(), Error> {
        let raw_message = Self::Serializer::serialize(message)?;
        Ok(self.sender.send(Message::text(raw_message)).await?)
    }
}
