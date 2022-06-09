use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]

async fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:8080").await.unwrap();

    let (tx, _rx) = broadcast::channel(10);
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        //Allows for concurrent tasks
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split(); //: (ReadHalf, WriteHalf)

            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                // Similar to Promise.race in Node.js
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            let final_msg = "(".to_string() + &other_addr.to_string() + ") " + &String::from(msg) ;
                            writer.write_all(final_msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
