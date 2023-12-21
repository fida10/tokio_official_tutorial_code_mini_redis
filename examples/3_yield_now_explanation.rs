use tokio::task;

fn fib(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}
//computationally heavy fibonacci series calculation, done with recursion

#[tokio::main]
async fn main() {
    let fib_task = task::spawn(async {
        for i in 0..10 {
            println!("Fibonacci of {} is {}", i * 5, fib(i * 5));
            task::yield_now().await;
        }
    });

    let periodic_task = task::spawn(async {
        for _ in 0..10 {
            println!("Periodic task running");
        }
    });

    let _ = tokio::try_join!(fib_task, periodic_task);
}

/*
The above does the following
We are spawning 2 tasks, asynchronously, fib_task and periodic_task
The first task (fib_task) progressively becomes VERY computationally intensive, as we are getting the next 5 fibonacci numbers
It appears that tokio green threads (the "tasks" being spawned) are scaleable; one task that requires more resources will be given those resources
However, the computer only has a finite number of resources
So at some point, very computationally heavy tasks will just eat up ALL of the CPU resources, even if they are on their own "thread" or task
    Kind of like a nest of baby birds (each bird is a task); a bird that want to eat more will be fed more by the mother, at the cost of the other baby birds

Yield now seeks to fix this
When we reach the line in the "fib_task" invoking yield_now, the periodic task releases ("yields" or surrenders) ALL of its CPU resources
This then gives these resources to other tasks, like periodic_task
If there are any futures ready in these other tasks, they are executed
Essentially this give a chance for other tasks to run when a computationally intensive task is running repeatedly, so that nothing stays pending for too long

A note: this is not the only way to yield cpu resources
When we call sleeps, or ticks and resolve their futures, we are also yielding CPU resources until the future resolves


All of that being said, this is a special case. If we run with the interval, we see this: 
lfenix-youtubes-Mac-mini:tokio_official_tutorial_code_mini_redis lfenix_rust$ cargo run
   Compiling tokio_official_tutorial_code_mini_redis v0.1.0 (/Users/lfenix_rust/Programming/RustProjects/tokio_practice/tokio_official_tutorial_code_mini_redis)
    Finished dev [unoptimized + debuginfo] target(s) in 1.77s
     Running `target/debug/tokio_official_tutorial_code_mini_redis`
Fibonacci of 0 is 0
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Fibonacci of 5 is 5
Fibonacci of 10 is 55
Fibonacci of 15 is 610
Fibonacci of 20 is 6765
Fibonacci of 25 is 75025
Fibonacci of 30 is 832040
Fibonacci of 35 is 9227465
Fibonacci of 40 is 102334155
Fibonacci of 45 is 1134903170

This is expected. The fib_task executes one iteration, then hands its resources back to the tokio runtime (specifically a "scheduler" that allocates resources)
The scheduler than gives all the resources to any task that needs it, in this case the "periodic_task". This is just printing the numbers 1 - 10, which executes very fast, and is able to complete almost instataneously (cpu resources not needed)
Once this task is done, resources are yielded back to the fib_task, which then executes
It yields resources with each iteration, but since there are no further tasks running now, there is nothing to yield to so it just continues in sequence



On the other hand, if we comment out the task::yield_now().await; line, we get the following: 

lfenix-youtubes-Mac-mini:tokio_official_tutorial_code_mini_redis lfenix_rust$ cargo run
   Compiling tokio_official_tutorial_code_mini_redis v0.1.0 (/Users/lfenix_rust/Programming/RustProjects/tokio_practice/tokio_official_tutorial_code_mini_redis)
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
     Running `target/debug/tokio_official_tutorial_code_mini_redis`
Fibonacci of 0 is 0
Fibonacci of 5 is 5
Fibonacci of 10 is 55
Fibonacci of 15 is 610
Fibonacci of 20 is 6765
Fibonacci of 25 is 75025
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Periodic task running
Fibonacci of 30 is 832040
Fibonacci of 35 is 9227465
Fibonacci of 40 is 102334155
Fibonacci of 45 is 1134903170

According to copilot: 

The behavior you're observing is due to the nature of asynchronous programming and the way the Tokio runtime schedules tasks.

When you spawn multiple tasks, the Tokio runtime schedules them to run concurrently. However, the exact order in which the tasks are run and how the runtime switches between tasks is not deterministic and can vary based on a number of factors, including the specifics of the tasks and the system's current load.

In your case, the fib_task starts running first and calculates the Fibonacci numbers for 0, 5, and 10. These calculations are relatively quick, so the fib_task doesn't yield control back to the scheduler.

IMPORTANT: When the fib_task starts calculating the Fibonacci number for 15, this calculation takes a bit longer. During this time, the Tokio runtime decides to switch to the periodic_task, which quickly prints its messages.

After the periodic_task has finished, the fib_task resumes and continues calculating the remaining Fibonacci numbers. These calculations are quite intensive, so they take some time to complete.

The task::yield_now().await function can be used to explicitly yield control back to the scheduler, but it's not a guarantee that other tasks will run immediately. It's just a hint to the scheduler that it might be a good time to switch to another task. The exact scheduling behavior can vary based on the specifics of the tasks and the system's current load.




Some further points from copilot about this: 
Your comments do a good job of explaining the concept of yielding in asynchronous programming and how it applies in the context of your code. However, there are a few points that could be clarified:

1. `task::yield_now().await;` doesn't release all of its CPU resources. It simply hints the scheduler that it might be a good time to switch to another task. The scheduler might or might not switch to another task immediately.

2. The `yield_now` function doesn't give resources to other tasks. It simply pauses the execution of the current task and gives the scheduler an opportunity to run other tasks.

3. The behavior of `yield_now` is not directly related to the computational intensity of the tasks. Even a task that is not computationally intensive might want to yield to allow other tasks to run.

4. The `yield_now` function is not the only way to yield control to the scheduler. Any operation that involves awaiting a future will also yield control back to the scheduler.

5. The exact scheduling behavior can vary based on the specifics of the tasks and the system's current load. The `yield_now` function is just a hint to the scheduler, and the scheduler might decide to continue running the current task if there are no other tasks ready to run.

6. The `yield_now` function is most useful in scenarios where a task is running a tight loop that doesn't involve any asynchronous operations. In such cases, the task would monopolize the CPU unless it explicitly yields control back to the scheduler. But in your case, the `periodic_task` is already yielding control back to the scheduler due to the `interval.tick().await`, so the `task::yield_now().await` in the Fibonacci task doesn't make a noticeable difference.
*/