use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

// Note, std::sync::Mutex and not tokio::sync::Mutex is used to guard the HashMap. A common
// error is to unconditionally use tokio::sync::Mutex from within async code. An async mutex
// is a mutex that is locked across calls to .await.
//
// A synchronous mutex will block the current thread when waiting to acquire the lock. This, in
// turn, will block other tasks from processing. However, switching to tokio::sync::Mutex
// usually does not help as the asynchronous mutex uses a synchronous mutex internally.
//
// As a rule of thumb, using a synchronous mutex from within asynchronous code is fine as long
// as contention remains low and the lock is not held across calls to .await. Additionally,
// consider using parking_lot::Mutex as a faster alternative to std::sync::Mutex.

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

struct CanIncrement {
    mutex: Mutex<i32>,
}

impl CanIncrement {
    //this function is not marked async
    fn increment(&self) {
        let mut lock = self.mutex.lock().unwrap();
        *lock += 1;
    }

}

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}

async fn increment_and_do_stuff(can_increment: &CanIncrement) {
    //This pattern guarantees that you won't run into the Send error,
    // because the mutex guard does not appear anywhere in an async function.
    can_increment.increment();
    //do_something_async().await;
}

// Server side code

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        // Non-concurrent
        //process(socket).await;

        let db = db.clone();
        println!("Accepted");

        // A new task is spawned for each inbound socket. The socket is moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    // The `Connection` lets us read/write redis **frames** instead of byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                // The value is stored as `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().clone());
                // Return OK. Mind the Upper case. Ok/ ok will not work.
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key())  {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`. Now into() is not required. Hence, removed
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Bulk("Not found".into())
                }
            }
            cmd=> panic!("Unimplemented {:?}", cmd),
        };
        // write the response back to client
        connection.write_frame(&response).await.unwrap();
    }
}