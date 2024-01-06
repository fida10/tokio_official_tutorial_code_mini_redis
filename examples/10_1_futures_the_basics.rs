use tokio::net::TcpStream;

async fn my_async_fn() {
    println!("hello from async");
    let _socket = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    println!("async TCP operation complete");
}

#[tokio::main]
async fn main() {
    let what_is_this = my_async_fn();
    // Nothing has been printed yet.
    /* 
    This is because futures in rust are lazy; they are only executed when they are called (or "polled", as we will see in the upcoming files)
    Think of it like buying all the ingredients to cook with but not actually cooking
    Or planning on learning to code but not actually doing anything ;)
     */

    what_is_this.await;
    // Text has been printed and socket has been
    // established and closed.
    //Future is called here so it is actually executed
}