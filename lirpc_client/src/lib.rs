mod error;
mod lirpc_message;
mod serializers;
mod transport;

use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        Mutex,
        mpsc::{self, Receiver},
        oneshot,
    },
};
use tokio_rustls::client::TlsStream;
use tokio_util::bytes::Bytes;
use tracing::{error, warn};

use crate::{
    error::Error,
    lirpc_message::{LiRpcRequest, LiRpcRequestHeaders, LiRpcResponse, LiRpcServerError},
    serializers::Serializer,
    transport::{Transport, tcp::Tcp, websocket::Websocket},
};

pub struct Client<T: Transport<F>, F> {
    id_counter: u32,
    transport: T,
    response_pending: Arc<Mutex<BTreeMap<u32, oneshot::Sender<LiRpcResponse<Value>>>>>,
    f: PhantomData<F>,
}

impl Client<Tcp<TcpStream>, Bytes> {
    /// No TLS: unencrypted
    pub async fn new_tcp_plain(address: impl ToSocketAddrs) -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(10);
        Self::new_tcp_with_transport(rx, Tcp::connect(address, tx).await?).await
    }
}

impl Client<Tcp<TlsStream<TcpStream>>, Bytes> {
    /// TLS: encrypted
    pub async fn new_tcp_tls(address: String) -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(10);
        Self::new_tcp_with_transport(rx, Tcp::connect_tls(address, tx).await?).await
    }
}

impl<S> Client<Tcp<S>, Bytes>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    async fn new_tcp_with_transport(rx: Receiver<Bytes>, transport: Tcp<S>) -> Result<Self, Error> {
        let response_pending = Arc::new(Mutex::new(BTreeMap::new()));

        let rp = response_pending.clone();
        tokio::spawn(async move { Self::message_router(rx, rp).await });

        Ok(Self {
            id_counter: 0,
            transport,
            response_pending,
            f: PhantomData,
        })
    }
}

impl Client<Websocket, String> {
    pub async fn new_websocket(url: &str) -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(10);
        let transport = Websocket::connect(url, tx).await?;

        let response_pending = Arc::new(Mutex::new(BTreeMap::new()));

        let rp = response_pending.clone();
        tokio::spawn(async move { Self::message_router(rx, rp).await });

        Ok(Self {
            id_counter: 0,
            transport,
            response_pending,
            f: PhantomData,
        })
    }
}

impl<T, F> Client<T, F>
where
    T: Transport<F>,
{
    fn get_new_request_id(&mut self) -> u32 {
        self.id_counter = self.id_counter.wrapping_add(1);
        self.id_counter
    }

    async fn message_router(
        mut rx: mpsc::Receiver<F>,
        response_pending: Arc<Mutex<BTreeMap<u32, oneshot::Sender<LiRpcResponse<Value>>>>>,
    ) {
        while let Some(msg) = rx.recv().await {
            let deserialized_msg: LiRpcResponse<Value> = match T::Serializer::deserialize(&msg) {
                Ok(m) => m,
                _ => continue,
            };

            let mut rp_lock = response_pending.lock().await;
            let sender = rp_lock.remove(&deserialized_msg.headers.id);
            drop(rp_lock);

            let sender = match sender {
                Some(s) => s,
                None => {
                    warn!("Received message with no listener waiting");
                    continue;
                }
            };

            match sender.send(deserialized_msg) {
                Ok(_) => {}
                Err(e) => error!("error during message forwarding: {e:?}"),
            };
        }
    }

    pub async fn call<M, R>(
        &mut self,
        function: String,
        payload: Option<M>,
    ) -> Result<Call<R>, Error>
    where
        M: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let message = LiRpcRequest {
            headers: LiRpcRequestHeaders {
                id: self.get_new_request_id(),
                function,
            },
            payload,
        };

        let (tx, rx) = oneshot::channel();

        let mut rp_lock = self.response_pending.lock().await;
        rp_lock.insert(message.headers.id, tx);
        drop(rp_lock);

        self.transport.send(message).await?;

        Ok(Call::new(rx))
    }
}

pub struct Call<R>
where
    R: for<'de> Deserialize<'de>,
{
    receiver: oneshot::Receiver<LiRpcResponse<Value>>,
    _response_type: PhantomData<R>,
}

impl<R> Call<R>
where
    R: for<'de> Deserialize<'de>,
{
    fn new(receiver: oneshot::Receiver<LiRpcResponse<Value>>) -> Self {
        Self {
            receiver,
            _response_type: PhantomData,
        }
    }

    pub async fn resolve(self) -> Result<R, Error> {
        let response = self.receiver.await?;

        if !response.headers.res.is_ok() {
            let deserialized_response: LiRpcResponse<LiRpcServerError> =
                response.deserialize_payload::<LiRpcServerError>()?;

            Err(Error::Server {
                error: deserialized_response.payload.error,
                detail: deserialized_response.payload.detail,
            })
        } else {
            let deserialized_response: LiRpcResponse<R> = response.deserialize_payload::<R>()?;

            Ok(deserialized_response.payload)
        }
    }
}
