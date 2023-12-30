/*
We can also avoid the problem of passing MutexGuards across awaits (across tasks) by using tokio's built in MutexGuard
*/

use std::time::Duration;

use tokio::{time::sleep, sync::{Mutex, MutexGuard}};
//using tokio's build in Mutex/MutexGuards, which can be carried between threads/tasks and awaits

async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
    let mut lock: MutexGuard<i32> = mutex.lock().await;
    *lock += 1;

    do_something_async().await; //on this line, we are doing something asynchronous
}
/*
This is the same code as was used in 6_3 (which caused an error since the MutexGuard is still active when another async task begins) but unlike the previous code, this uses Tokio's built in Mutex/MutexGuard

This is not advisable in most cases, because: 
    The tokio::sync::Mutex type provided by Tokio can also be used. The primary feature of the Tokio mutex is that it can be held across an .await without any issues. That said, an asynchronous mutex is more expensive than an ordinary mutex, and it is typically better to use one of the two other approaches.
*/

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