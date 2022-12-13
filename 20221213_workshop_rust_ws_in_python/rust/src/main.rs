#![deny(unused_must_use)]

use std::format;
use std::env;
use std::time::{SystemTime, Duration};
use log::{error, info, Level, warn};
use tokio_stream::StreamExt as TokioStreamExt;
use futures_util::SinkExt;
// use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;


fn subscribe(market: &str) -> Message {
    Message::Text(format!("{{\"op\": \"subscribe\", \"channel\": \"ticker\", \"market\": \"{market}\"}}"))
}


fn get_timestamp() -> f64 {
    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    time.as_secs() as f64 + time.subsec_nanos() as f64 / 1e9
}


/*

At first it seemed Rust will be terrible.

https://github.com/serde-rs/json/issues/160

*/
async fn connect(ws_url: &str) {
    let (mut connection, response) = tokio_tungstenite::connect_async(url::Url::parse(ws_url).unwrap()).await.unwrap();
    info!("response: {:?}", response);

    match connection.send(subscribe("BTC-PERP")).await {
        Ok(_) => (),
        Err(err) => {
            error!("Cannot subscribe because of an error: {:?}", err);
            return;
        }
    }

    while let Some(item) = connection.next().await {
        let data = match item {
            Ok(Message::Text(text)) => match json::parse(&text) {
                Ok(data) => data,
                Err(err) => {
                    warn!("{}", err);
                    continue;
                }
            },
            err => {
                error!("Error when receiving a message: {:?}", err);
                break;
            }
        };
        let ex_time = data["data"]["time"].as_f64().unwrap_or(0.);
        let loc_time = get_timestamp();
        println!(
            "{:>10.9}, {:>10.9}, {:>4.6}",
            loc_time,
            ex_time,
            1000. * (loc_time - ex_time),
        );
    };

    match connection.close(None).await {
        Ok(_) => info!("Successfully disconnected"),
        Err(err) => info!("Failed to disconnect because of {:?}", err),
    }
}


#[tokio::main(flavor = "current_thread")]
async fn main() {
    simple_logger::init_with_level(Level::Info)
        .expect("I would log this error, but it's a chicken-egg problem");

    info!("starting up");

    let url = "ws://localhost:49158";
    tokio::spawn(async move { connect(url).await; });

    let args: Vec<String> = env::args().collect();
    let duration_secs: u64 = args[1].parse().unwrap();
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    
    info!("ended");
}
