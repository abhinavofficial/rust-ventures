# Tokio

Source: [Tokio Tutorial](https://tokio.rs/tokio/tutorial) 

## Spawning

### Concurrency
> Concurrency and parallelism are not the same thing. If you alternate between two tasks, then you are working on both tasks concurrently, but not in parallel. For it to qualify as parallel, you would need two people, one dedicated to each task.

> One of the advantages of using Tokio is that asynchronous code allows you to work on many tasks concurrently, without having to work on them in parallel using ordinary threads. In fact, Tokio can run many tasks concurrently on a single thread!

To process connections concurrently, a new task is spawned using ```tokio::spawn``` for each inbound connection. The connection is processed on this task.

### Tasks
A Tokio task is an asynchronous green thread. They are created by passing an ```async``` block to ```tokio::spawn```. The ```tokio::spawn``` function returns a ```JoinHandle```, which the caller may use to interact with the spawned task. The async block may have a return value. The caller may obtain the return value using ```.await``` on the ```JoinHandle```.

For example:
```
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
    // Do some async work
    "return value"
    });

    // Do some other work

    let out = handle.await.unwrap();
    println!("GOT {}", out);
}
```

Awaiting on ```JoinHandle``` returns a ```Result```. When a task encounters an error during execution, the ```JoinHandle``` will return an ```Err```. This happens when the task either panics, or if the task is forcefully cancelled by the runtime shutting down.

> Tasks are the unit of execution managed by the scheduler. Spawning the task submits it to the Tokio scheduler, which then ensures that the task executes when it has work to do. The spawned task may be executed on the same thread as where it was spawned, or it may execute on a different runtime thread. The task can also be moved between threads after being spawned.

Tasks in Tokio are very lightweight. Under the hood, they require only **a single allocation and 64 bytes of memory**. Applications should feel free to spawn thousands, if not millions of tasks.

When you spawn a task on the Tokio runtime, its type's lifetime must be ```'static```. This means that the spawned task must not contain any references to data owned outside the task. For example, the below code would not work.
```
use tokio::task;

#[tokio::main]
async fn main() {
    let v = vec![1, 2, 3];

    task::spawn(async {
        println!("Here's a vec: {:?}", v);
    });
}
```
Of course, you can use ```move``` to transfer the ownership in the println! and then the code works.

Note that the error message talks about the argument type _outliving_ the ```'static``` lifetime. This terminology can be rather confusing because the ```'static``` lifetime lasts until the end of the program, so if it outlives it, don't you have a memory leak? The explanation is that it is the **type**, _not the value_ that must outlive the ```'static``` lifetime, and the value may be destroyed before its type is no longer valid.

When we say that a value is ```'static```, all that means is that it would not be incorrect to keep that value around forever. This is important because the compiler is unable to reason about how long a newly spawned task stays around, so the only way it can be sure that the task doesn't live too long is to make sure it may live forever.

[The article](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program) uses the terminology "bounded by ```'static```" rather than "its type outlives ```'static```" or "the value is ```'static```" to refer to T: ```'static```. These all mean the same thing, but are different from "annotated with ```'static```" as in ```&'static T```.

If a single piece of data must be accessible from more than one task concurrently, then it must be shared using synchronization primitives such as ```Arc```.

> It is a common misconception that 'static always means "lives forever", but this is not the case. Just because a value is 'static does not mean that you have a memory leak. You should read about common rust misconception regarding lifetime [Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)

Tasks spawned by tokio::spawn must implement **Send**. This allows the Tokio runtime to move the tasks between threads while they are suspended at an ```.await```.

Tasks are ```Send``` when all data that is held **across** ```.await``` calls is ```Send```. This is a bit subtle. When ```.await``` is called, **the task yields back to the scheduler**. The next time the task is executed, it resumes from the point it last yielded. To make this work, all state that is used **after** ```.await``` must be saved by the task. If this state is ```Send```, i.e. can be moved across threads, then the task itself can be moved across threads. Conversely, if the state is not ```Send```, then neither is the task.

## Shared State

### Strategies
There are a couple of different ways to share state in Tokio.
* Guard the shared state with a Mutex - used for simple data and sync processes
* Spawn a task to manage the state and use message passing to operate on it - used when process required async processing.

 _Add bytes dependency_: Instead of Vec<u8>, we will use bytes (which is roughly Arc<Vec<u8>>). Since the Hashmap will be shared across many tasks and potentially many threads. To support this, it is wrapped in Arc<Mutex<_>>.

### Guard the shared state with a Mutex
#### Initialize the Hashmap
using ```std::sync::Mutex```

#### Tasks, threads, and contention
Using a blocking mutex to guard short critical sections is an acceptable strategy when contention is minimal. When a lock is contended, the thread executing the task must block and wait on the mutex. This will not only block the current task, but it will also block all other tasks scheduled on the current thread.

By default, the Tokio runtime uses a multithreaded scheduler. Tasks are scheduled on any number of threads managed by the runtime. If a large number of tasks are scheduled to execute, and they all require access to the mutex, then there will be contention. On the other hand, if the current_thread runtime flavor is used, then the mutex will never be contended.

> The ```current_thread``` runtime flavor is a lightweight, single-threaded runtime. It is a good choice when only spawning a few tasks and opening a handful of sockets. For example, this option works well when providing a synchronous API bridge on top of an asynchronous client library.

If contention on a synchronous mutex becomes a problem, the best fix is rarely to switch to the Tokio mutex. Instead, options to consider are:
* Switching to a dedicated task to manage state and use message passing.
* Shard the mutex.
* Restructure the code to avoid the mutex.
In our case, as each key is independent, mutex sharding will work well. To do this, instead of having a single Mutex<HashMap<_, _>> instance, we would introduce N distinct instances. Then, finding the cell for any given key becomes a two-step process. First, the key is used to identify which shard it is part of. Then, the key is looked up in the HashMap. The simple implementation outlined above requires using a fixed number of shards, and the number of shards cannot be changed once the sharded map is created. The ```dashmap``` crate provides an implementation of a more sophisticated sharded hash map.

#### Holding a MutexGuard
Since ```std::sync::MutexGuard``` type is not Send, you can't send a mutex lock to another thread. Please note that Tokio runtime can move a task between threads at every .await. To avoid this, you should **restructure your code** such that the mutex lock's destructor runs before the .await The compiler currently calculates whether a future is Send based on **scope information** only. The compiler will hopefully be updated to support explicitly dropping it in the future, but for now, you must explicitly use a scope.

> You should not try to circumvent this issue by spawning the task in a way that does not require it to be Send, because if Tokio suspends your task at an .await while the task is holding the lock, some other task may be scheduled to run on the same thread, and this other task may also try to lock that mutex, which would result in a deadlock as the task waiting to lock the mutex would prevent the task holding the mutex from releasing the mutex.

One way to restructure your code to not hold back across an .await is that you can wrap the mutex in a struct, and only ever lock the mutex inside non-async methods on that struct. Please see implementation.

### Spawn a task to manage the state and use message passing to operate on it
This is often used when the shared resource is an I/O resource.

#### Use Tokio's asynchronous mutex
The ```tokio::sync::Mutex``` type provided by Tokio can also be used. The primary feature of the Tokio mutex is that it can be held across an .await without any issues. That said, an asynchronous mutex is more expensive than an ordinary mutex, and it is typically better to use one of the two other approaches.

```
use tokio::sync::Mutex; // note! This uses the Tokio mutex

// This compiles!
// (but restructuring the code would be better in this case)
async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
let mut lock = mutex.lock().await;
*lock += 1;

    do_something_async().await;
} // lock goes out of scope here
```

## Channels
If we want to run two concurrent Redis command, we can spawn one task per command and then two commands would run concurrently. If both the commands try to get access to client somehow, it won't work because client does not implement ```Copy``` to facilitate this sharing. Also, if one the command calls ```set```, it would need exclusive access. Options are:
* Open connection per task - this is not ideal.
* Use ```std::sync::Mutex``` over client - this cannot be used as ```.await``` would need to be called with lock held.
* Use of ```tokio::sync::Mutex``` is possible, but that would allow only a single in-flight request. If the client implements ```pipelining```, an async mutex results in underutilizing the connection.
* The answer really in such case is - Message passing

### Message Passing
The pattern involves spawning a dedicated task (channel) to manage the client resource. Any task that wishes to issue a request sends a message to the client task. The client task issues the request on behalf of the sender, and the response is sent back to the sender.

Using this strategy, a single connection is established. The task managing the client is able to get exclusive access in order to call get and set. Additionally, the channel works as a buffer. Operations may be sent to the client task while the client task is busy. Once the client task is available to process new requests, it pulls the next request from the channel. This can result in better throughput, and be extended to support connection pooling.

### Tokio's channel primitives
Tokio provides a [number of channels](https://docs.rs/tokio/1.17.0/tokio/sync/index.html), each serving a different purpose.

* [mpsc](https://docs.rs/tokio/1.17.0/tokio/sync/mpsc/index.html): multi-producer, single-consumer channel. Many values can be sent.
* [oneshot](https://docs.rs/tokio/1.17.0/tokio/sync/oneshot/index.html): single-producer, single consumer channel. A single value can be sent.
* [broadcast](https://docs.rs/tokio/1.17.0/tokio/sync/broadcast/index.html): multi-producer, multi-consumer. Many values can be sent. Each receiver sees every value.
* [watch](https://docs.rs/tokio/1.17.0/tokio/sync/watch/index.html): single-producer, multi-consumer. Many values can be sent, but no history is kept. Receivers only see the most recent value.

If you need a multi-producer multi-consumer channel where only one consumer sees each message, you can use the ```async-channel``` crate. 

_There are also channels for use outside asynchronous Rust, such as ```std::sync::mpsc``` and ```crossbeam::channel```. These channels wait for messages by blocking the thread, which is not allowed in asynchronous code._

Let's use mpsc and oneshot in this example. First, **define the message type**. Second, **create the channel**. Then, **spawn manager task**. Finally, **receive responses**.

### Backpressure and bounded channels
Concurrency and queuing must be explicitly introduced. Ways to do this include:
* tokio::spawn
* select!
* join!
* mpsc::channel
While doing so, take care to ensure that total amount of concurrency is bounded. For example, when writing a TCP accept loop, ensure that the total number of open sockets is bounded. When using mpsc::channel, pick a manageable channel capacity. Specific bound values will be application specific.

Taking care and picking good bounds is a big part of writing reliable Tokio applications.