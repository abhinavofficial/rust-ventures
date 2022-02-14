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
