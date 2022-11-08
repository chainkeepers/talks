#![deny(unused_must_use)]

use std::format;
use std::time::SystemTime;
use std::future::pending;
use log::{error, info, Level, warn};
use tokio_stream::StreamExt as TokioStreamExt;
use futures_util::SinkExt;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;


fn subscribe(market: &str) -> Message {
    Message::Text(format!("{{\"op\": \"subscribe\", \"channel\": \"ticker\", \"market\": \"{market}\"}}"))
}


fn get_timestamp() -> f64 {
    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    time.as_secs() as f64 + time.subsec_nanos() as f64 / 1e9
}


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
        let data: Value = match item {
            Ok(Message::Text(text)) => match serde_json::from_str(&text) {
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
        let loc_time = get_timestamp();
        let ex_time = data["data"]["time"].as_f64().unwrap_or(0.);
        info!(
            "{:>10.9}, {:>10.9}, {:>4.6}: {:>10.6}  {} {}  {: <10.6} ",
            loc_time,
            ex_time,
            1000. * (loc_time - ex_time),
            data["data"]["bidSize"].as_f64().unwrap_or(0.),
            data["data"]["bid"],
            data["data"]["ask"],
            data["data"]["askSize"].as_f64().unwrap_or(0.)
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

    let url = "wss://ftx.com/ws/";
    tokio::spawn(async move { connect(url).await; });

    pending::<()>().await;

    info!("ended");
}
