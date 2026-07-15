use lirpc_client::Client;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, Debug)]
struct GreetingResponse {
    msg: String,
}

#[tokio::main]
async fn main() {
    let mut tcp_client = Client::new_tcp_plain("127.0.0.1:5000").await.unwrap();

    let res = tcp_client
        .call::<Value, GreetingResponse>("greet".to_string(), Some(json!({"name": "Cas"})))
        .await
        .unwrap()
        .resolve()
        .await;

    println!("{}", res.unwrap().msg);
}
