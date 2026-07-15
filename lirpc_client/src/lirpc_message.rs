use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub(crate) struct LiRpcRequest<P: Serialize> {
    pub headers: LiRpcRequestHeaders,
    pub payload: Option<P>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LiRpcRequestHeaders {
    pub id: u32,
    pub function: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LiRpcResponse<P> {
    pub headers: LiRpcResponseHeaders,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: P,
}

impl LiRpcResponse<Value> {
    pub(crate) fn deserialize_payload<P>(self) -> Result<LiRpcResponse<P>, serde_json::error::Error>
    where
        P: for<'de> Deserialize<'de>,
    {
        Ok(LiRpcResponse {
            headers: self.headers,
            payload: serde_json::from_value(self.payload)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct LiRpcResponseHeaders {
    pub id: u32,
    #[serde(
        skip_serializing_if = "LiRpcResponseResultHeader::is_ok",
        default = "LiRpcResponseResultHeader::ok"
    )]
    pub res: LiRpcResponseResultHeader,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LiRpcResponseResultHeader {
    Ok,
    Err,
}

impl LiRpcResponseResultHeader {
    pub(crate) fn ok() -> Self {
        LiRpcResponseResultHeader::Ok
    }

    pub(crate) fn is_ok(&self) -> bool {
        matches!(self, LiRpcResponseResultHeader::Ok)
    }
}

#[derive(Deserialize)]
pub(crate) struct LiRpcServerError {
    pub error: String,
    pub detail: String,
}
