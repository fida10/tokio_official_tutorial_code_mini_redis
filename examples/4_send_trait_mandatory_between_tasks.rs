use tokio::task::yield_now;
use std::rc::Rc;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        // The scope forces `rc` to drop before `.await`.
        {
            let rc = Rc::new("hello");
            println!("{}", rc);
        }

        // `rc` is no longer used. It is **not** persisted (doesn't stay alive) when
        // the task yields to the scheduler, so no issues caused here
        yield_now().await;
    });
}

// use tokio::task::yield_now;
// use std::rc::Rc;

// #[tokio::main]
// async fn main() {
//     tokio::spawn(async {
//         let rc = Rc::new("hello");

//         // `rc` is used after `.await`. It must be persisted to
//         // the task's state.
//         yield_now().await;

//         println!("{}", rc);
//     });
// }

/*
The Rc (Reference Counting) type in Rust is not thread-safe, which means it's not safe to share between threads. This is because Rc uses non-atomic operations to increment and decrement the reference count, which can lead to data races if two threads modify the reference count at the same time.

When you use tokio::spawn to create a new task, that task could potentially be executed on a different thread. Therefore, all data used within the task must be safe to send across threads, which in Rust is represented by the Send trait. Because Rc is not Send, you can't use it within a tokio::spawn task.

If you need to share data between tasks in a tokio::spawn, you can use Arc (Atomic Reference Counting) instead of Rc. Arc is a thread-safe version of Rc that uses atomic operations to update the reference count, so it is Send and can be used safely within a tokio::spawn task.


//A more clear example of why use of Rc within threads is not allowed: 

use tokio::task;
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let task1 = task::spawn(async {
        let rc = Rc::new("hello");
        task::yield_now().await;
        rc
    });

    let rc = task1.await.unwrap();
//the value returned by task1 is saved in "rc"

    let task2 = task::spawn(async move {
        println!("Task 2: {}", rc);
    });
//we then attempt to use it again (potentially at the same time as when we use in task1) in task2
//if we wanted to do this, we need to use something that is thread safe (implements "Send" trait), like Arc
    task2.await.unwrap();
}
*/
