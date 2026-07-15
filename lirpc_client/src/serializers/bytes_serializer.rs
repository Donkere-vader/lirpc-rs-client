use tokio_util::bytes::Bytes;

use crate::serializers::Serializer;

pub struct BytesSerializer;

impl Serializer<Bytes> for BytesSerializer {
    fn serialize<M: serde::Serialize>(message: M) -> Result<Bytes, serde_json::Error> {
        Ok(Bytes::from(serde_json::to_vec(&message)?))
    }

    fn deserialize<M>(raw: &Bytes) -> Result<M, serde_json::Error>
    where
        M: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_slice(raw)
    }
}
