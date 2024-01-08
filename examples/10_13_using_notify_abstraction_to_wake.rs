/*
We can abstract away a lot of the change of wakers by tokio's "Notify" module, below
*/

use tokio::sync::Notify;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// This is an asynchronous function named `delay`. It takes a `Duration` as an argument.
// Asynchronous functions in Rust return a type that implements the `Future` trait.
// This means you can `await` the result of this function in an async context.
async fn delay(dur: Duration) {
    // `Instant::now()` gets the current time, and `+ dur` adds the duration to it.
    // The result is the time at which the delay should end.
    let when = Instant::now() + dur;

    // `Notify` is a type from the `tokio::sync` module that provides a mechanism to notify a task (thread, executor) when a particular event has occurred.
        /*
        Its job is primarily to abstract away the hassle of Wakers and wake()
        When a future is ready to be executed, simply call notify_one()
         */
    // `Arc` is a type of smart pointer in Rust that allows multiple threads to have read access to some data and ensures that the data gets cleaned up once all threads are done with it.
    // `Arc::new(Notify::new())` creates a new `Notify` instance and wraps it in an `Arc` so it can be shared across threads.
    let notify = Arc::new(Notify::new());

    // Cloning an `Arc` increases its reference count and allows the clone to be moved to another thread.
    // This clone will be moved into the thread we spawn below.
    let notify_clone = notify.clone();

    // `thread::spawn` starts a new OS thread and runs the provided closure in it.
    // The `move` keyword means that the closure takes ownership of the values it uses from the environment, in this case `when` and `notify_clone`.
    thread::spawn(move || {
        // Get the current time.
        let now = Instant::now();

        // If the current time is less than the end time of the delay, sleep the thread for the remaining duration.
        if now < when {
            thread::sleep(when - now);
        }

        // Once the delay has elapsed, call `notify_one` on the `Notify` instance.
        // This wakes up one task that called `notified().await` on the `Notify` instance.
        notify_clone.notify_one();
        /*this essentially does the same thing as wake, but abstracted away.
        //it invokes the stored waker to wake the task
        //this lets whatever executor is handling this "delay" method that the task is ready for execution and it is scheduled (in the ways we have already seen)
        */
    });

    // `notified().await` suspends the current task until the `Notify` instance is notified.
    // In this case, it will be notified when the delay has elapsed.
    notify.notified().await;
    /*
    Specifically, this waits until notify is "notified", meaining it waits until notify_one is called. 
    Note that notify_one is called in another thread; this thread may finish executing and exit before notify_one has a chance to execute
    So we run notified() here, which waits until notify_one is called
    Remember as well that both notify and notify_clone come from the same instance, and they are just shared (notify_clone was cloned, see above)
    So if we call notify_one on notify_clone, it affects notify as well since the 2 are linked
     */
}

#[tokio::main]
async fn main() {
    let when = Duration::from_millis(2000);
    let delay = delay(when);

    let task = tokio::spawn(async move {
        delay.await;
        println!("Delay completed");
    });

    task.await.unwrap();
}
/*
Usage of delay with Notify module example
*/