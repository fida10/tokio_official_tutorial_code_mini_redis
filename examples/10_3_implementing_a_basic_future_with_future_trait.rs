use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

//A basic implementation of future --> 
struct Delay {
    when: Instant,
}
//a struct meant to be handled asynchronously
//one field given, an Instant, which is a unit of time (meaning the struct will be used like a timer)

//Future implementation
impl Future for Delay {
    type Output = &'static str;
    /*
    A future must have an "output" type
    What that means is when we resolve (usually "await") the future, we need to know what type the future will return
    Kind of like, I'm going to cook, this is the type of food you'll get (type --> indian, thai, brazillian, japanese, etc.)
    */

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<&'static str>
        /*
        Fundamental part of the future. This method specifies HOW the future will be resolved (what does "resolved even mean? It is defined here")
        */
    {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
            /*
            In this case, if the Instant has passed (meaning if the current time right now is greater than whatever time we put in the "when" field), we are returned a "Ready" subtype, part of the Poll enum (explained in 10_2)
            A return of "Ready" means the future is resolved
            As you can imagine, we can put all kinds of logic in here (i.e. API calls, file reads, anything that might take a while but doesn't require much computing power, more like a wait for completion, during which the CPU resources are free to focus on other tasks)
            */
        } else {
            // Ignore this line for now.
            cx.waker().wake_by_ref();

            Poll::Pending
            //and if the future is not done yet, Poll::Pending is returned
            //this signals the future is still pending
        }
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    //here we basically say "when" is a timer of 10 milliseconds
    let future = Delay { when };
    //creates a new struct instance of Delay

    let out = future.await;
    /*
    Await, internally, does the following --> 
    It continuously calls the poll() function inside of the "future" variable
    It does not exactly do this using a loop
    Instead, it calls poll once, and if it gets a Poll::Pending back, it yields control of the executor (the thing in rust that decides what to run)
    The executor then runs other things, and comes back and checks on (calls) the poll function again later
    Thus we achieve concurrency
     */

    assert_eq!(out, "done");
    //resolving the future should return "done", as specified in the poll() function
}