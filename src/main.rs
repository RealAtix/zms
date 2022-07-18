use ::futures::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use zms::Msg;

#[tokio::main]
async fn main() {
    let address = "localhost:8080";
    let listener = TcpListener::bind(address)
        .await
        .unwrap_or_else(|_| panic!("Could not bind to address {}", address));
    println!(
        "Zoomies server is listening on {:?}",
        listener.local_addr().unwrap()
    );

    let (tx, _rx) = broadcast::channel(16);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("Connection from {}", addr);
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let transport = Framed::new(socket, LengthDelimitedCodec::new());
            let (mut sink, mut stream) = transport.split();

            loop {
                tokio::select! {
                    result = stream.next() => {
                        println!("Received from {}: {:?}", addr, result);
                        let result = match result.transpose() {
                            Ok(o) => o,
                            Err(_) => break
                        };
                        let result = match result {
                            Some(value) => value,
                            None => {
                                println!("Client {} disconnected", addr);
                                break;
                            }
                        };
                        let msg : Msg = bincode::deserialize(&result[..]).unwrap();
                        tx.send((msg, addr)).unwrap();
                    }
                    result = rx.recv() => {
                        let (msg, sender_addr) = result.unwrap();

                        if addr != sender_addr {
                        println!("Sending from {} to {}: {:?}", sender_addr, addr, msg);
                            let encoded = bincode::serialize(&msg).unwrap();
                            sink.send(encoded.into()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
