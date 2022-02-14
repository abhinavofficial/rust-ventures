use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

// Server side code

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        // Non-concurrent
        //process(socket).await;

        // A new task is spawned for each inbound socket. The socket is moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // Hashmap to save data
    let mut db = HashMap::new();

    // The `Connection` lets us read/write redis **frames** instead of byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // The value is stored as `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                // Return OK. Mind the Upper case. Ok/ ok will not work.
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key())  {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd=> panic!("Unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}