//The full code for this is found here: https://github.com/tokio-rs/website/blob/master/tutorial-code/mini-tokio/src/main.rs

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use futures::task;
//import from here: https://crates.io/crates/futures

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

fn main() {
    let mut mini_tokio = MiniTokio::new();
    //create a new instance of MiniTokio (defined below)

    mini_tokio.spawn(
        async {
            /*
            Interestingly, this whole block is a future
            It fits the needed criteria of a task, in that it returns nothing (Output = (), see below)
            The stuff inside of it does return something (future variable below with Delay instance within)
            Remember, we can have futures within futures

            However, the result of the internal future (and in fact, none of the code below) will be run until the outer future is executed
            Which is what mini-tokio will do, via "mini_tokio.run()"
             */
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };
        //creates a future

        let out = future.await;
        //resolves the created future

        assert_eq!(out, "done");
        //verifies result of the future
    }
);

    mini_tokio.run();
    //executes all of the tasks in the mini tokio queue
    //explained below
}

struct MiniTokio {
    tasks: VecDeque<Task>,
    /*
    MiniTokio is an executor (runtime environment), a sized down version of Tokio
    Here, we add one field, "tasks", which stores all the tasks that the executor must execute
    */
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
/*
Specifying what an asynchronous task truly is
Pin is...complicated. Understand that its job is to ensure a future is not moved as a prerequisite to being polled
We ensure all tasks are Futures with <Box<dyn<Future (trait object)
The output of the future in our particular example is () (empty tuple, returns nothing)
    So essentially a Task is a future that executes but returns nothing
    I think it is just used for example purposes here
Must implement the Send trait (be sendable between threads)
*/

impl MiniTokio {
    fn new() -> MiniTokio {
        MiniTokio {
            tasks: VecDeque::new(),
            //initializes MiniTokio with an empty queue of tasks
        }
    }
    
    /// Spawn a future onto the mini-tokio instance.
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future));
        /*
        The spawn function is used to add tasks (of the type Task, specified above) to the MiniTokio queue for execution
        push_back adds them to the end of the queue  
         */
    }
    
    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        //explained later in waker sections, right now it looks like we put no waker for this task
        
        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task);
            }
        }

        /*
        A loop!
        This loops through the queue of tasks in the tasks field
        It first takes the first task in the queue (removes it from the queue for analysis in the loop) --> self.tasks.pop_front()
        The task is polled --> task.as_mut().poll(&mut cx)
            and the return value is checked, to see if it is pending --> .is_pending()
        If it is pending, add back to the queue, at the back of the queue
        If not...remember we specified in the definition of Task that futures added to the queue won't return anything (Output = ())


        While this is great, it does have a problem
        The loop will just continue to run forever and ever and ever until all the futures inside of it are all resolved
        Think of the analogy of a baker who has a queue of ovens (tasks) and he constantly goes from oven to oven to see if the food inside is down
        He's gonna get tired eh
        It's the same with a computer; it won't necessarily get tired but it will consume resources, having to constantly check each future
        
        It would be a lot more convenient if each oven had some way to signal the baker, like a ding sound, or a "waker", that lets the baker know that the food is ready, so he doesn't have to constantly check each oven...
        */
    }
}