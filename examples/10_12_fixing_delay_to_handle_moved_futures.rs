/*
On handling the problem of moved futures

The idea is, on each call to poll, the future checks if the supplied waker matches the previously recorded waker.
If the two wakers match, then there is nothing else to do.
If they do not match, then the recorded waker must be updated
*/

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
    // This is Some when we have spawned a thread, and None otherwise.
    waker: Option<Arc<Mutex<Waker>>>,
    //a waker field is added to keep track of Delay's current waker
}

impl Future for Delay {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        // Check the current instant. If the duration has elapsed, then
        // this future has completed so we return `Poll::Ready` as usual, no chance of this getting moved since it is already done
        if Instant::now() >= self.when {
            return Poll::Ready(String::from("Done!"));
        }

        // The duration has not elapsed. If this is the first time the future
        // is called, spawn the timer thread. If the timer thread is already
        // running, ensure the stored `Waker` matches the current task's waker.
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();
            /*
            this is the current waker inside the Delay instance (field "waker")
            we use this to check against the current waker in the task where Delay is being polled from
                Which can be different if Delay has been moved
            As mentioned in the comment, the first time this runs, the waker from the th
            */

            // Check if the stored waker matches the current task's waker.
            // This is necessary as the `Delay` future instance may move to
            // a different task between calls to `poll`. If this happens, the
            // waker contained by the given `Context` will differ and we
            // must update our stored waker to reflect this change.
            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
                /*
                This checks to see if the waker stored inside of the Delay instance will wake the current Context's waker (meaning is the current waker in Delay linked with the current Context's (which comes from the thread Delay is being polled from) waker)
                    Specifically it checks if it will NOT wake the current Contexts waker (!waker)

                And if it finds out that the current waker in the Delay instance will not wake the current Context's waker, we replace the current waker in Delay with the waker from context
                    This is an indication that the Delay task has moved. This requires a modification of the waker in Delay to make sure we wake Delay properly from the right thread
                 */
                println!("Updating the Waker because the Future has moved to a new task.");
                //adding print statement to show that waker was updated because Future was moved
            }
        } else {
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());
            /*
            This handles the first time the function is polled
            waker field is set to "None" initially (see main function below)
            on the first run of poll(), the above if statement (if let Some(waker) = &self.waker) is false, since the initial value is None

            for this case, the waker for the current context is cloned and input into the "waker" field in Delay
            the next time the function is polled, the above if (if let Some(waker) = &self.waker) is true, and will compare the waker in Delay with the waker in the current context
            */

            // This is the first time `poll` is called, spawn the timer thread.
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                // The duration has elapsed. Notify the caller by invoking
                // the waker.
                let waker = waker.lock().unwrap();
                println!("Waking up the task because the duration has elapsed.");
                //print out statement letting us know future is done and can be processed, and waker is being called to do so
                waker.wake_by_ref();
            });
        }

        // By now, the waker is stored and the timer thread is started.
        // The duration has not elapsed (recall that we checked for this
        // first thing), ergo the future has not completed so we must
        // return `Poll::Pending`.
        //
        // The `Future` trait contract requires that when `Pending` is
        // returned, the future ensures that the given waker is signalled
        // once the future should be polled again. In our case, by
        // returning `Pending` here, we are promising that we will
        // invoke the given waker included in the `Context` argument
        // once the requested duration has elapsed. We ensure this by
        // spawning the timer thread above.
        //
        // If we forget to invoke the waker, the task will hang
        // indefinitely.
        Poll::Pending
    }
}

use futures::future::poll_fn;

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(2000); //increasing the time of the timer to allow future to not return Poll::Ready immediately but be pending for a bit, ensuring the future is moved
    let mut delay = Some(Delay {
        when,
        waker: None, //adding a value of None here initially; this will be filled when the function is polled at least once
    });

    poll_fn(move |cx| {
        let mut delay = delay.take().unwrap();
        let res = Pin::new(&mut delay).poll(cx);
        assert!(res.is_pending());
        println!("Is pending");
        tokio::spawn(async move {
            println!("{}", delay.await);
            /*
            A note; this is for example purposes only
            What will happen if this code was run just like that this thread will not have a chance to execute
            The main thread will continue executing (with this future returning Poll::Ready(()))
            And then it will complete
                The .await attached to this poll_fn will only await this new future we made
                It will NOT await the new thread (task) we spawned
                Meaning, again, the task will not have a chance to execute its code and we won't see the desired results if we ran this

            To fix this, added a thread::sleep below. This is not an optimal solution but it works fine for learning purposes
            Now, we can clearly see the Delay future being moved from the main thread to the spawned thread
                And associated print messages are also visible when this is run
            */
        });

        Poll::Ready(())
    })
    .await;

    // Sleep the main thread for 5 seconds
    std::thread::sleep(Duration::from_secs(5));
}
/*
Same main code as was in 10_11, with some modifications to clearly show futures moving and different wakers being called
*/