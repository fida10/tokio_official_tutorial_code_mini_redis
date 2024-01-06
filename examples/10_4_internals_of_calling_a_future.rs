/*
How exactly does await work? How are futures resolved?
Below we see a rough explanation
*/

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
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
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/*
Implementing future trait on Delay, like we did before
*/

enum MainFuture {
    // Initialized, never polled
    State0,
    // Waiting on `Delay`, i.e. the `future.await` line.
    State1(Delay),
    // The future has completed.
    Terminated,
}
/*
Essentially, a future can be in one of three states, represented by the enum above.

State0 means the future has not even been called yet
    Remember futures are lazy; if they are not called, they won't execute

State1(Delay) means that the future has been called but has not resolved yet, it is still in Poll::Pending status

Terminated means that the future has been resolved and poll() returns Poll::Ready
*/

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<()>
    {
        use MainFuture::*;

        loop {
            /*
            So internally, rust does not use a loop, the executor simply shifts its attentions to other tasks
            But we simulate that behavior of rechecking if the future is ready with a loop here
             */
            match *self {
                State0 => {
                    let when = Instant::now() +
                        Duration::from_millis(10);
                    let future = Delay { when };
                    *self = State1(future);
                    /*
                    If the future has not even been called yet (it is in state0), and we await it, it will be called and checked
                     */
                }
                State1(ref mut my_future) => {
                    match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = Terminated;
                            return Poll::Ready(());
                            /*
                            If the future is resolved (completed), then change its state to Terminated, and return Poll::Ready
                             */
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                            /*
                            If the future is not resolved, then keep it in State1 and return Poll::Pending, to be checked again later
                             */
                        }
                    }
                }
                Terminated => {
                    panic!("future polled after completion")
                    /*
                    If we call await on an already completed future...it will panic
                    It's like eating the food that's been cooked by the future and asking "where's the food?"
                     */
                }
            }
        }
    }
}

fn main() {}
//added just so this code by itself does not give an error