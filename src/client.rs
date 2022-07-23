use ::futures::{SinkExt, StreamExt};
use clap::Parser;
use tokio::{net::TcpStream, signal};
use tokio_util::codec::{Framed, FramedRead, LengthDelimitedCodec, LinesCodec};
use zms::message::{Message, MessageKind};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, value_parser, default_value = "8080")]
    pub port: u16,

    #[clap(short, long, value_parser, default_value = "localhost")]
    pub host: String,

    #[clap(short, long, value_parser, default_value = "general")]
    pub channel: String,

    #[clap(short, long, value_parser)]
    pub name: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let address = format!("{}:{}", &args.host, &args.port);

    let socket = TcpStream::connect(&address)
        .await
        .unwrap_or_else(|_| panic!("Could not connect to {}", address));
    let transport = Framed::new(socket, LengthDelimitedCodec::new());
    let (mut sink, mut stream) = transport.split();

    let mut reader = FramedRead::new(tokio::io::stdin(), LinesCodec::new());

    // Connect to channel
    let msg_connect = Message {
        kind: MessageKind::Connect,
        channel: args.channel.to_owned(),
    };
    let msg_connect =
        bincode::serialize(&msg_connect).expect("Could not serialize connection message");
    sink.send(msg_connect.into()).await.unwrap();

    if let Some(name) = args.name {
        let msg_setname = Message {
            kind: MessageKind::SetName(name),
            channel: args.channel.to_owned(),
        };
        let msg_setname =
            bincode::serialize(&msg_setname).expect("Could not serialize setname message");
        sink.send(msg_setname.into()).await.unwrap();
    }

    println!("[Server]: Connected to channel {}", args.channel);

    // Process incoming messages
    tokio::spawn(async move {
        loop {
            let result = stream.next().await;
            let result = match result.transpose() {
                Ok(o) => o,
                Err(_) => break,
            };
            match result {
                Some(bytes) => {
                    let msg: Message = bincode::deserialize(&bytes[..]).unwrap();
                    match msg.kind {
                        MessageKind::Message(sender, message) => {
                            println!("[{}]: {}", sender, message)
                        }
                        _ => {}
                    }
                }
                None => {
                    println!("[Server]: Lost connection to host, exiting now");
                    std::process::exit(1);
                }
            }
        }
    });

    // Process stdin
    tokio::spawn(async move {
        loop {
            let result = reader.next().await;
            let result = match result.transpose() {
                Ok(Some(o)) => o,
                _ => break,
            };

            let msg = Message {
                kind: MessageKind::Relay(result),
                channel: args.channel.to_owned(),
            };
            let encoded = bincode::serialize(&msg).unwrap();

            sink.send(encoded.into()).await.unwrap();
        }
    });

    // Keep tasks alive
    match signal::ctrl_c().await {
        _ => std::process::exit(0),
    }
}
