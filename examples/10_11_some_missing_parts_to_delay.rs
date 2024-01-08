/*
On the problem of moved futures (moved from one task to another, see main)
*/

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            let waker = cx.waker().clone();
            let when = self.when;

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                waker.wake();
            });

            Poll::Pending
        }
    }
}

/*
Above is same code as before, from 10_9
*/

use futures::future::poll_fn;

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    let mut delay = Some(Delay { when });

    poll_fn(move |cx| {
        //2 poll_fn creates a new future on the spot from a closure
        //2 context comes from the executor (main thread in this case, tokio::main) that is executing 
        //3 one Context/Waker passed here

        let mut delay = delay.take().unwrap();
        let res = Pin::new(&mut delay).poll(cx);
        //1 delay is polled here and partially executed, i.e. it be cooking in one oven

        assert!(res.is_pending());
        tokio::spawn(async move {
            delay.await;
            //1 now delay is moved to another oven and cooked (its execution is continued and finished) here
            //3 another Context/Waker passed here
        });

        Poll::Ready(())
    }).await;
}

/*
Some concepts are shown here, and marked with numbers in the actual code: 

1 - Future Migration: In Rust, a single Future can move across different tasks while it's executing. This means that the Future can start executing in one task, and then continue executing in another task. This is what's happening in the provided code: the Delay future is first polled in the main task, and then it's moved to a new task where it's awaited.

2 - poll_fn Function: The poll_fn function is a helper function from the futures crate that creates a Future from a closure. The closure takes a Context and returns Poll::Ready(()) when it's done. In the provided code, the poll_fn closure polls the Delay future once, and then moves the Delay future to a new task.

In the case of poll_fn, the executor is tokio::main, which is a tokio runtime configured to run until all spawned tasks have completed. When tokio::main polls the Future created by poll_fn, it passes in a Context, which poll_fn then passes to its closure.
Meaning tokio::main gives the context; i.e. context comes from the asnyc fn main itself (context comes from the executor, in this case that is the main thread, so it provides the context here for the waker)
Remember that we need to know the context of each waker to know what future to execute when the waker is called (5 ovens are cooking, one of them makes an alarm saying they're done. Which one made the alarm? Identifying this would prove difficult without an identifier of some kind. This is the job of Context)

3 - Waker Instances: The Waker is used to wake up a task when a Future is ready to make progress. Each call to poll is passed a Context, which contains a Waker. It's important to note that each call to poll could be passed a different Waker. This is because the Future can move to a different task, and each task has its own Waker. In the provided code, the Delay future is polled twice: once in the main task, and once in the new task. Each of these polls is passed a different Waker.

Interesting. So wakers are associated with tasks (threads), rather than futures. In the two points marked //3, we have 2 different wakers
So building on the timer analogy with the oven:
    Food (future) in the first oven (main thread), cooks a bit (executes via .await), has one waker (a timer that goes ding)
    Food (future) is moved to a different oven (spawned task), cooks the rest of the way (.await), has another waker (another timer that goes dong)
Calling the wake() method will only cause the thread that that wake is attached to (which it knows via Context) to be activated
If the future has been moved from there, it will not be executed (as it will be in a different thread)

4 (Not shown in this code, see next section for this). Updating the Waker: When implementing a Future, you must make sure to update any previously recorded Waker with the new one passed to the most recent call to poll. This is because the Future could have moved to a new task, and you need to make sure to wake up the correct task when the Future is ready to make progress. If you don't do this, you could end up waking up a task that no longer has the Future, which would be a waste of resources.

This is done to counter point 3. If the future has been moved to a different thread, the waker must be updated to ensure that the correct waker is called to execute the future (The food is no longer in the first oven, it is in the second oven. So we must wait to hear the second oven's alarm to know it's time to serve the food). 


*/