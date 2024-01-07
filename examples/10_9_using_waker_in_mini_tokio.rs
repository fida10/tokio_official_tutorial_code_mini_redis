/*
Must now make it so mini-tokio (which, remember, is our executor) is notified when waker is called
A note that tokio already has this programmed within it to do this
Now, we see a (very basic) implementation of this
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
                /*
                Through wake_by_ref method definied in the Task wrapper (far) below, the call to wake() here reschedules the task for execution
                Which makes sense at this point because the timer is complete, so this future is done
                Reschedule meaning this poll() functions is called again, and this time, it will return Poll::Ready, thereby completing
                 */
            });

            Poll::Pending
        }
    }
}

/*
Code with the waker implementation on Delay
A note that Wakers must implement Send and Sync (which they do in almost all instances, unless they have non Send/Sync types in them like Rc)
This is because they will be sent between threads (sent from wherever "Delay" struct is being awaited, to the main thread)
*/

use std::sync::Arc;
use std::sync::{mpsc, Mutex};

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
    /*
    MiniTokio has two fields: scheduled and sender. 
    These are the two halves of a multi-producer, single-consumer (mpsc) channel. 
    The sender is used to send tasks (futures wrapped in a Task struct) to be executed, 
    and scheduled is used to receive these tasks.
     */
}

impl MiniTokio {
    /// Initialize a new mini-tokio instance.
    fn new() -> MiniTokio {
        let (sender, scheduled) = mpsc::channel();

        MiniTokio { scheduled, sender }
    }
    /*
    The new function creates a new MiniTokio instance. 
    It creates a new mpsc channel and assigns the sender and receiver to the sender and scheduled fields, respectively.
     */

    /// Spawn a future onto the mini-tokio instance.
    ///
    /// The given future is wrapped with the `Task` harness and pushed into the
    /// `scheduled` queue. The future will be executed when `run` is called.
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }
    /*
    The spawn function is used to spawn a new task onto the executor. 
    It takes a future, wraps it in a Task struct, and sends it to the scheduled queue via the sender.
    We send a reference to sender (&self.sender) because the Task::spawn method will clone this sender and insert it into the correct field in the Task
        Remember, mpsc channels are multi-producer, single consumer
        So each task will have a sender half of the channel which leads back to MiniTokio, which holds the receiver half in its "scheduled" field
    
    This method is called in the main function
    The future in this spawn is just a container, that's why it doesn't return anything
    However, for it to resolve, the future within must complete and return something
    And THIS is where we put the Delay instance

    See (far) below in the Task::spawn method and the main method for more details
     */

    fn run(&self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
    /*
    The run function is the main loop of the executor. 
    It continuously receives tasks from the scheduled queue and polls them. 
        Again, tasks have a sender half which they use to send themselves to mini-tokio
        They are received and processed here via while let
    If a task is not ready yet, its poll method will ensure that it gets re-scheduled for polling when it becomes ready.
        Task's poll method is below 

    Unlike before, where tasks were continuously being polled before they were ready, task.poll() logic will instead not re-send the task to mini-tokio UNTIL it is ready to proceed
    See below task.poll() method for how
     */
}

struct Task {
    // The `Mutex` is to make `Task` implement `Sync`. Only
    // one thread accesses `future` at any given time. The
    // `Mutex` is not required for correctness. Real Tokio
    // does not use a mutex here, but real Tokio has
    // more lines of code than can fit in a single tutorial
    // page.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    //task definition (what futures can be put into tasks) defined here
    executor: mpsc::Sender<Arc<Task>>,
    //the sender with which tasks will be sent to the executor (our mini-tokio, in this case)
        //remember, mini-tokio has an mpsc channel as well, which can have one receiver (mini-tokio) and mutliple senders (for multiple tasks)
}
    /*
    The job of the task struct is to serve as a wrapper around a future
    In this particular case, "Delay" struct instances
    The task itself is a wrapper future which resolves when the future it is wrapped around also completes
    It is mini-tokios way of tracking when a future is done
    
    Its like a labelled oven-proof smart container that food is put into before being putting into the oven
    It is not really food but makes it easier for the baker to handle and move the food
    It also signals to the baker if the food it holds is complete

    A better example might be an apple product
    You can have all of your own data (your own music, pictures, emails, etc.) (futures being inserted into mini-tokio)
    But for it to work on apple's ecosystem, it must all be placed inside of an apple device (the "Task" wrapper in this case)

    An "executor" field exists as well, which holds a sender half of an mpsc channel
    The reciever half of this channel is on the "receiver" field of mini-tokio
    */

use futures::task::{self, ArcWake};
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}
/*
This is an implementation of the ArcWake trait for Task. 
ArcWake is a trait provided by the futures crate that defines a method for waking up a task. 
Here, wake_by_ref is implemented to call the schedule method on Task.

This is what allows a taks to schedule itself once it is ready to proceed
The logic for how a task schedules itself is refined in the schedule method below
But in summary, the schedule() method is sends the task to the executor (mini-tokio)

If you look through the code, you will see that wake_by_ref is never actually explicitly called
So how does it work?
The wake_by_ref method is attached to a Waker; when the waker's wake() method is called, that's when this method is called too
Remember, Task is a wrapper future around an actual future we are trying to resolve 
In this case, Delay
Notice in Delay, we have this code in the else block (executes if the timer is not finished (not Poll::Ready, but Poll::Pending) when we poll() the method): 
                waker.wake();
Through the code below for Task (in Task's impl block), we link Task's wake method with the wake method of the future it wraps around (Delay)
Sooooo essentially what is happening is when the wake method is called in the Delay struct for Poll::Pending, this wake_by_ref method is also called, which reschedules the task for execution
*/

impl Task {
    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone()).unwrap();
    }
    /*
    As explained above, this re-adds the Task to the executor (mini-tokio)
    Called when a task was pending but now is ready to proceed (wake() is called on Waker)
     */

    fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);
        //Creates a waker, then links the waker to this task with Context (cx)

        // No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();

        // Poll the future
        let _ = future.as_mut().poll(&mut cx);
        //and this activates the poll method in Delay, causing the events described above (both in the delay poll() function AND the wake_by_ref() method in ArcWake)
    }
    /*
    The poll method is where the task's future gets polled. 
    A Waker is created from the task, and a Context is created from the Waker. 
    The future is then polled with this Context.
    */

    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
        /*
        this line is to send the task to the executor, to be executed initially (polled)
        successful execution ends the story
        unsuccessful execution (Poll::Pending) triggers the logic we wrote for if the task is not complete (Which is inside the Delay struct)
            This eventually results in the wake() method being called, scheduling (sending) the task again to the executor
        All this is explained in more detail above
        */
    }
    /*
    The spawn method is used to create a new Task from a future and send it to the executor. 
    The future is wrapped in a Box and Pinned, and then wrapped in a Mutex for thread safety. 
    The Task is then wrapped in an Arc for shared ownership and sent to the executor.

    Some more details! This is NOT directly called in main (when we spawn in main, we are calling the mini-tokio spawn method)
    Instead, this spawn method is called (far) above in the mini-tokio spawn() method: 
            Task::spawn(future, &self.sender);
    As described in comments in that method, the "future" is just a holder/container that doesn't return anything
    But any futures we put INSIDE this future must resolve for this holder future to complete
    And again, THIS is where we put the Delay instance
     */
}

fn main() {
    let mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async 
        {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };
        /*
        We insert the Delay future here; until it completes, THIS ENTIRE FUTURE cannot complete
        That's how it works as a container/holder
         */

        let out = future.await;
        //.await calls the poll() function on Delay, which triggers all of our logic above

        assert_eq!(out, "done");
    }
    /*
    THIS BLOCK IS THE CONTAINER/HOLDER FUTURE INSIDE OF THE MINI-TOKIO SPAWN AND TASK SPAWN METHODS
     */
);

    mini_tokio.run();
}

/*
Full explanation of the above: 

Delay Struct and Future Implementation: The Delay struct represents a future that completes after a certain time. The poll method checks if the current time is past the when time. If it is, it prints "Hello world", and returns Poll::Ready("done"). If it's not, it spawns a new thread that sleeps until the when time, then calls waker.wake(). This will cause the executor to poll the future again.

MiniTokio Struct and Implementation: MiniTokio is a simple executor. It has a channel for sending and receiving tasks. The spawn method wraps a future in a Task and sends it to the channel. The run method continuously receives tasks from the channel and polls them.

Task Struct and Implementation: Task is a wrapper for a future. It contains the future and a sender for the executor's channel. The poll method creates a Waker from the Task, wraps it in a Context, and polls the future with it. If the future returns Poll::Pending, the Waker will be used to wake up the task, causing it to be polled again.

ArcWake Implementation for Task: This allows a Task to be woken up. The wake_by_ref method calls the schedule method on Task, which sends a clone of the task back to the executor's channel.

Main Function: The main function creates a new MiniTokio executor, spawns a Delay future onto it, and runs the executor. The Delay future will complete after 10 milliseconds, print "Hello world", and return "done".

In summary, this code demonstrates a simple executor that can run futures. The executor polls futures until they're ready. If a future is not ready, it can be woken up to be polled again. This is done using the Waker and Context types from the std::task module, and the ArcWake trait from the futures crate.
*/