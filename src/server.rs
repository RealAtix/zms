use std::process;

use ::futures::{SinkExt, StreamExt};
use env_logger::Env;
use log::{debug, info};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use zms::Msg;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let host: String = "localhost".to_string();
    let port: u16 = 8080;
    let address = format!("{}:{}", host, port);

    info!(
        "Zoomies server starting : pid={} host={} port={}",
        process::id(),
        host,
        port
    );
    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind address");
    info!("Listening on {:?}", listener.local_addr().unwrap());

    let (tx, _rx) = broadcast::channel(16);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        debug!("Connection from {}", addr);
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let transport = Framed::new(socket, LengthDelimitedCodec::new());
            let (mut sink, mut stream) = transport.split();

            loop {
                tokio::select! {
                    result = stream.next() => {
                        debug!("Received from {} frame: {:?}", addr, result);
                        let result = match result.transpose() {
                            Ok(o) => o,
                            Err(_) => break
                        };
                        let result = match result {
                            Some(value) => value,
                            None => {
                                debug!("Client {} disconnected", addr);
                                break;
                            }
                        };
                        let msg : Msg = bincode::deserialize(&result[..]).unwrap();
                        tx.send((msg, addr)).unwrap();
                    }
                    result = rx.recv() => {
                        let (msg, sender_addr) = result.unwrap();

                        if addr != sender_addr {
                        debug!("Sending from {} to {}: {:?}", sender_addr, addr, msg);
                            let encoded = bincode::serialize(&msg).unwrap();
                            sink.send(encoded.into()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
