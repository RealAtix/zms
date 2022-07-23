#[macro_use]
extern crate lazy_static;

use ::futures::{SinkExt, StreamExt};
use clap::Parser;
use dashmap::DashMap;
use env_logger::Env;
use log::{debug, error, info};
use std::{collections::HashSet, net::SocketAddr, process};
use tokio::{net::TcpListener, sync::broadcast, task};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use zms::message::{Message, MessageKind};

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    #[clap(short, long, default_value = "8080")]
    port: u16,

    #[clap(short, long, value_parser, default_value = "localhost")]
    host: String,
}

#[derive(Debug, Default)]
struct Client {
    pub name: String,
    pub channels: HashSet<String>,
}

lazy_static! {
    static ref CLIENTS: DashMap<SocketAddr, Client> = DashMap::new();
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let connection_settings = Args::parse();
    let address = format!("{}:{}", connection_settings.host, connection_settings.port);

    info!(
        "Zoomies server starting : pid={} host={} port={}",
        process::id(),
        connection_settings.host,
        connection_settings.port
    );
    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind address");
    info!("Listening on {:?}", listener.local_addr().unwrap());

    let (tx, _rx) = broadcast::channel(32);

    loop {
        let (socket, addr) = match listener.accept().await {
            Ok((socket, addr)) => (socket, addr),
            Err(e) => {
                error!("{:?}", e);
                break;
            }
        };
        debug!("Connection from {}", addr);
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let mut client_entry = CLIENTS.entry(addr).or_insert(Client {
            name: "Anon".to_string(),
            channels: HashSet::new(),
        });

        tokio::spawn(async move {
            let transport = Framed::new(socket, LengthDelimitedCodec::new());
            let (mut sink, mut stream) = transport.split();

            loop {
                tokio::select! {
                    result = stream.next() => {
                        let client = client_entry.value_mut();

                        debug!("Received from {} frame: {:?}", addr, result);
                        let result = match result.transpose() {
                            Ok(o) => o,
                            Err(_) => break
                        };
                        let result = match result {
                            Some(value) => value,
                            None => {
                                debug!("Client {} disconnected", &addr);
                                task::spawn_blocking(move || {
                                    CLIENTS.remove(&addr);
                                });
                                client.channels.iter().for_each(|x| {
                                    debug!("{:?}", x.as_str());
                                    let msg = Message {
                                        kind: MessageKind::Message("Server".to_string(), format!("{} has disconnected", client.name)),
                                        channel: x.as_str().into()
                                    };
                                    tx.send((msg, addr)).unwrap();
                                });
                                break;
                            }
                        };

                        let msg : Message = bincode::deserialize(&result[..]).unwrap();
                        debug!("Received from {} message: {:?}", addr, msg);
                        match &msg.kind {
                            MessageKind::Connect => {
                                debug!("Client {} connected", addr);
                                client.channels.insert(msg.channel.to_owned());
                                let msg = Message {
                                    kind: MessageKind::Message("Server".to_string(), format!("{} has connected", client.name)),
                                    channel: msg.channel
                                };
                                tx.send((msg, addr)).unwrap();
                            },
                            MessageKind::Disconnect => {
                                debug!("Client {} disconnected", addr);
                                client.channels.remove(&msg.channel);
                                let msg = Message {
                                    kind: MessageKind::Message("Server".to_string(), format!("{} has disconnected", client.name)),
                                    channel: msg.channel
                                };
                                tx.send((msg, addr)).unwrap();
                            },
                            MessageKind::SetName(name) => {
                                debug!("Client {} changed name from {} to {}", addr, client.name, &name);
                                let msg = Message {
                                    kind: MessageKind::Message("Server".to_string(), format!("{} changed name to {}", client.name, name)),
                                    channel: msg.channel
                                };
                                client.name = name.to_owned();
                                tx.send((msg, addr)).unwrap();
                            },
                            MessageKind::Relay(content) => {
                                debug!("Client {} sent message {}", addr, content);
                                let msg = Message {
                                    kind: MessageKind::Message(client.name.to_owned(), content.to_owned()),
                                    channel: msg.channel
                                };
                                tx.send((msg, addr)).unwrap();
                            },
                            _ => {}
                        }
                    }
                    result = rx.recv() => {
                        let (msg, sender_addr) = match result {
                            Ok((x,y)) => {(x, y)},
                            Err(e) => {
                                error!("{:?}", e);
                                break;
                            }
                        };
                        let client = client_entry.value_mut();

                        if addr != sender_addr && client.channels.contains(&msg.channel) {
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
