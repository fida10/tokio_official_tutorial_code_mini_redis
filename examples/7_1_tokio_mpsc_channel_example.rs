use tokio::{spawn, sync::mpsc};

/*
#[tokio::main]
async fn main() {
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    let t1 = spawn(async {
        println!("{:?}", client.get("Howdy").await.unwrap());
    });

    let t2 = spawn(async {
        client.set("Howdy", "Salam".into()).await.unwrap();
    });

    t1.await;
    t2.await;
}

The above code does not work for the following reason
"client" must be accessed by both tasks, but client does not implement the copy trait
    Moving it into one task would not possible since then the other task couldn't use it
Enclosing client inside of a mutex wouldn't work since we are using await and the lock would be held across the await
Using tokio mutex would work but it would not be ideal since we'd have to wait for one task to drop the lock on client before the other could access it
    Also not fully optimizing resources if we are using pipelining: https://redis.io/docs/manual/pipelining/
Enforcing a "1 task per client" would work but it would not be ideal since we'd have to create a new connection EVERY time we wanted to do something

We answer all of these questions with the concept of a "Channel", where tasks may communicate with each other
*/

/*
Basic program layout: A single task takes ownership of and manages the channel. All other tasks communicate with this single task if they want to access the client. The channel picks up these tasks concurrently whenever it is available.

Note that async channels are different from the sync channels we learned about in the book. Sync channels send and receive messages by blocking execution, which is not allowed in async

Types of tokio channels:
Tokio provides a number of channels, each serving a different purpose.

mpsc: multi-producer, single-consumer channel. Many values can be sent. (note that this was the only one we saw in the book)
oneshot: single-producer, single consumer channel. A single value can be sent.
broadcast: multi-producer, multi-consumer. Many values can be sent. Each receiver sees every value.
watch: single-producer, multi-consumer. Many values can be sent, but no history is kept. Receivers only see the most recent value.

If you need a multi-producer multi-consumer channel where only one consumer sees each message, you can use the async-channel crate. There are also channels for use outside of asynchronous Rust, such as std::sync::mpsc and crossbeam::channel. These channels wait for messages by blocking the thread, which is not allowed in asynchronous code.
*/

#[tokio::main]
async fn main() {
    let (sender_copyable, mut receiver) = mpsc::channel(32);
    /*
    We create an mpsc channel here that can have many senders and 1 receiver. 
    It has a capacity of 32, which means that up to 32 messages can be saved inside the channel for the receiver to handle 
    The receiver will handle the messages 1 by 1 of course, whenever it is available
    If more than 32 messages are currently awaiting processing by the receiver, all other senders must wait (concurrently of course, non blocking) until some messages are handled and capacity emerges
    Like sharding, this is scalable, meaning we can increase the channel capacity as needed at the cost of memory

    The non-blocking nature of the senders means that they can continue doing other work if the channel is full, rather than being blocked until there is space in the channel. This is a key feature of asynchronous programming.
     */

    let sender_clone = sender_copyable.clone();
    //we clone the sender to be able to send it to different threads

    spawn(async move {
        sender_clone.send("Sending a value").await.unwrap();
        //sends a value, which is receivable in "receiver"
    });

    spawn(async move {
        sender_copyable.send("Sending a second value").await.unwrap();
        //sends another value, which is receivable in "receiver"
    });

    while let Some(message) = receiver.recv().await {
        println!("Received: {message}");
    }
    /*
    We loop through the receiver; this loop will remain active until either the receiver or all the senders go out of scope
    Nearly identical to its synchronous counterpart
    Note that receiver is receiving asynchronously
     */
}
