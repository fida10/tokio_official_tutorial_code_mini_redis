use std::{
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use tokio::time::sleep;

async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
    // let mut lock: MutexGuard<i32> = mutex.lock().unwrap();
    // *lock += 1;

    // do_something_async().await; //on this line, we are doing something asynchronous

    /*
    for the above code, due to the nature of tokio, we have the ability to send stuff between tasks
    and rust/tokio cannot confirm that do_something_async does not contain another spawned thread
    additionally, the lock we have acquired on the mutex (MutexGuard) has not been deallocated yet (when we invoke do_something_async, the lock is still active), so there is the possiblity that we might try to send it to the other asynchronous task (do_something_async)
        even though we are not, but rust/tokio cannot verify that, so it throws a compiler error

    Moreover, the lock in question (MutexGuard) does not implement the Send trait
    Which also means that it cannot be sent across threads (tasks)
    Again, resulting in the compiler error, because rust/tokio cannot be sure that we are not sending the lock (or trying to, anyway) to a different task, inside of do_something_async

    The above code causes an error so it is commented. The way to fix this is to deallocate (release) the lock before invoking the async method (do_something_async), so that rust/tokio can be sure that we won't try any funny business and send it to another task (which again is not possible anyway because MutexGuard does not implement the Send trait)

    See below for the solution
    */
    {
        let mut lock: MutexGuard<i32> = mutex.lock().unwrap();
        *lock += 1;
        //here, the lock is aquired, a task is done, and the lock is released (deallocated) BEFORE do_something_async
    }
    do_something_async().await; //on this line, we are doing something asynchronous

    /*
    Further explanation from copilot:

    In Rust, the MutexGuard returned by mutex.lock().unwrap() does not implement the Send trait, which means it cannot be sent across different threads. This is a safety feature to prevent data races.

    When you call an async function like do_something_async().await, the Rust compiler treats it as a potential point where the current task could be suspended and other tasks could run. Because the MutexGuard is still in scope and has not been dropped, Rust cannot guarantee that it won't be accidentally sent to other tasks running concurrently, which would violate the safety rules.

    By placing the lock acquisition and modification inside a separate block {...}, you ensure that the MutexGuard is dropped before the async function is called. This allows Rust to guarantee that the MutexGuard won't be sent to other tasks, and thus the code compiles successfully.
    */

    /*
    Note also that this doesn't work:
    let mut lock: MutexGuard<i32> = mutex.lock().unwrap();
    *lock += 1;
    drop(lock); //can't use this, must explicitly exit a scope.

    do_something_async().await;

    Reason:
    This is because the compiler currently calculates whether a future is Send based on scope information only. The compiler will hopefully be updated to support explicitly dropping it in the future, but for now, you must explicitly use a scope.
    */
}

async fn do_something_async() {
    sleep(Duration::from_secs(5)).await;
    println!("Done sleeping!");
}

#[tokio::main]
async fn main() {
    let mutex = Mutex::new(0);

    tokio::spawn(async move {
        increment_and_do_stuff(&mutex).await;
    });
}