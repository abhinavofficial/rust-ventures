//Client Side code

use mini_redis::{client, Result};

// An async fn is used as we want to enter an asynchronous context. However, asynchronous functions must be executed by a runtime. The runtime
// contains the asynchronous task scheduler, provides evented I/O, timers, etc. The runtime does not automatically start, so the main function
// needs to start it. The #[tokio::main] function is a macro. It transforms the async fn main() into a synchronous fn main() that initializes a
// runtime instance and executes the async main function
#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    // It asynchronously establishes a TCP connection with the specified remote address. Once the connection is established, a client handle is
    // returned. Even though the operation is performed asynchronously, the code we write looks synchronous. The only indication that the
    // operation is asynchronous is the .await operator.

    //Primarily, Rust's async operations are lazy. This results in different runtime semantics than other languages. calling an async fn returns
    // a value representing the operation. This is conceptually analogous to a zero-argument closure. To actually run the operation, you should
    // use the .await operator on the return value.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}