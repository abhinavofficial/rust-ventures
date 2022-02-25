use tokio::sync::{mpsc, oneshot};
use mini_redis::client;
use bytes::Bytes;

/// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    }
}
/// Provided by the requester and used by the manager task to send the command response back to the requester
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;


#[tokio::main]
async fn main() {
    // Creating the channel returns two values, a sender and a receiver. The two handles are used separately. They may be moved to different tasks.
    // The channel is created with a capacity of 32. If messages are sent faster than they are received, the channel will store them. Once the 32 messages
    // are stored in the channel, calling send(...).await will go to sleep until a message has been removed by the receiver.
    let (tx, mut rx) = mpsc::channel(32);
    // Sending from multiple tasks is done by cloning the sender. However, both messages are sent to the single Receiver. It is not possible to clone the receiver of an mpsc channel.
    // The sender handles are moved into tasks. As there are two tasks, we need a second sender.
    let tx2 = tx.clone();

    // When every Sender has gone out of scope or has otherwise been dropped, it is no longer possible to send more messages into the channel.
    // At this point, the recv call on the Receiver will return None, which means that all senders are gone and the channel is closed.

    // A new task is spawned that processes messages from the channel.
    // The move keyword is used to move the ownership of rx into the task.
    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);

                }
                Command::Set { key, value, resp} => {
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key : "hello".to_string(),
            resp: resp_tx
        };

        // Send the get request
        if tx.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        //Await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res.unwrap());
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_tx,
        };
        //Send the set request
        if tx2.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        let res = resp_rx.await;
        println!("GOT {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}