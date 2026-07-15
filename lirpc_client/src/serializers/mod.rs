use serde::{Deserialize, Serialize};

pub mod bytes_serializer;
pub mod string_serializer;

pub trait Serializer<F> {
    fn serialize<M: Serialize>(message: M) -> Result<F, serde_json::Error>;
    fn deserialize<M>(raw: &F) -> Result<M, serde_json::Error>
    where
        M: for<'de> Deserialize<'de>;
}
