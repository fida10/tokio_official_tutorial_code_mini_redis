/*
Implement wake methodology (add the code for when "wake" should be called)

Note that the below implementation is not fully complete; there are some loose ends that will be covered in loose ends section
*/

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::thread;

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
    // cx is something that is intrinsically linked with this task, as mentioned previously
        -> Poll<&'static str>
    {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            // Get a handle to the waker for the current task
            let waker = cx.waker().clone();
            let when = self.when;

            // Spawn a timer thread.
            thread::spawn(move || {
                //note that this is not a tokio green thread; we are not using tokio in these few tutorials, we are making our own tokio
                //thread::spawn has the same effect, though it is a lot more memory intensive than using tokio threads
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }
                //this checks to see if the timer has elapsed (time specified in "when" field)
                //if it has not, wait that time with thread::sleep

                waker.wake();
                //regardless, call wake, letting the executor that this function is ready
            });

            Poll::Pending
            /*
            A note from the book:

            When a future returns Poll::Pending, it must ensure that the waker is signalled at some point. Forgetting to do this results in the task hanging indefinitely.

            Forgetting to wake a task after returning Poll::Pending is a common source of bugs.
             */
        }
    }
}

fn main() {}
//added to remove errors

/*
Note that in our previous implementation of Future on Delay, we did this: 

        else {
            // Ignore this line for now.
            cx.waker().wake_by_ref();
            Poll::Pending
        }

cx.waker().wake_by_ref(); essentially calls the walker every time the poll() is about to return pending
This worked for us then (because we did not know what it was) but it is not good practice
Essentially it keeps pinging the executor to check on this future whenever it returns pending
While the code will still work, it wastes CPU resources
*/