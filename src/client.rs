use ::futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, FramedRead, LengthDelimitedCodec, LinesCodec};
use zms::Msg;

#[tokio::main]
async fn main() {
    let address = "localhost:8080";
    let socket = TcpStream::connect(address)
        .await
        .unwrap_or_else(|_| panic!("Could not connect to {}", address));

    let transport = Framed::new(socket, LengthDelimitedCodec::new());
    let (mut sink, mut stream) = transport.split();

    let mut reader = FramedRead::new(tokio::io::stdin(), LinesCodec::new());

    // Process incoming messages
    tokio::spawn(async move {
        loop {
            let result = stream.next().await;
            println!("Incoming: {:?}", result);
            let result = match result.transpose() {
                Ok(o) => o,
                Err(_) => break,
            };
            match result {
                Some(bytes) => {
                    let msg: Msg = bincode::deserialize(&bytes[..]).unwrap();
                    println!("Received {:?}", msg);
                }
                None => {
                    println!("Lost connection to host, exiting now");
                    std::process::exit(1);
                }
            }
        }
    });

    // Process stdin
    tokio::spawn(async move {
        loop {
            let result = reader.next().await;
            println!("Reading stdin: {:?}", result);
            let result = match result.transpose() {
                Ok(o) => o,
                Err(_) => break,
            };
            let result = match result {
                Some(value) => value,
                None => break,
            };

            let msg = Msg { content: result };
            println!("Created message: {:?}", msg);
            let encoded = bincode::serialize(&msg).unwrap();
            println!("Sending encoded message: {:?}", encoded);

            sink.send(encoded.into()).await.unwrap();
        }
    });

    // Keep tasks alive
    loop {}
}
