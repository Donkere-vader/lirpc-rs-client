use crate::serializers::Serializer;

pub struct StringSerializer;

impl Serializer<String> for StringSerializer {
    fn serialize<M: serde::Serialize>(message: M) -> Result<String, serde_json::Error> {
        serde_json::to_string(&message)
    }

    fn deserialize<M>(raw: &String) -> Result<M, serde_json::Error>
    where
        M: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_str(raw)
    }
}
