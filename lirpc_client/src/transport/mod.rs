pub mod tcp;
pub mod websocket;

use serde::Serialize;

use crate::{error::Error, serializers::Serializer};

pub trait Transport<F> {
    type Serializer: Serializer<F>;

    fn send(&mut self, message: impl Serialize) -> impl Future<Output = Result<(), Error>>;
}
