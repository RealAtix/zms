use tokio::{net::TcpListener, sync::broadcast, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("localhost:8080").await.expect("Could not bind to address localhost:8080");
    println!("Zoomies server is listening on {:?}", listener.local_addr().unwrap());
    
    let (tx, _rx) = broadcast::channel(16);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, sender_addr) = result.unwrap();

                        if addr != sender_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}