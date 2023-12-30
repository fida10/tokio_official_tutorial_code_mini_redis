/*
How can we avoid sending locks (MutexGuards) across an await?
One solution is to refactor code so that the mutex is only ever accessed in non-async (basically synchronous) methods
*/

use std::{sync::Mutex, time::Duration};

use tokio::time::sleep;

struct CanIncrement {
    mutex: Mutex<i32>,
}
impl CanIncrement {
    // This function is not marked async.
    fn increment(&self) {
        let mut lock = self.mutex.lock().unwrap();
        *lock += 1;
    }
}

async fn increment_and_do_stuff(can_incr: &CanIncrement) {
    can_incr.increment();
    do_something_async().await;
}
/*
Here, we encapsulate the Mutex inside of a struct, whose functions are synchronous
Meaning the mutex does not ever come in contact with async directly
*/

async fn do_something_async() {
    sleep(Duration::from_secs(5)).await;
    println!("Done sleeping!");
}

#[tokio::main]
async fn main() {
    let mutex = Mutex::new(0);
    let increment_struct = CanIncrement { mutex };

    increment_and_do_stuff(&increment_struct).await;
}
