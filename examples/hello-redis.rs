use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    //before running, start up the redis database with mini-redis-server
    let mut client = client::connect("127.0.0.1:6379").await?;
    // this line connects us (the client) to the mini redis server at the above address and port
    // the connection happens asynchronously because it is a network connection happening over the internet
    // may fail, so adding a question mark operator at the end

    client.set("hello", "world".into()).await?;
    // this line adds a key value pair of "hello": "world" INTO the redis database

    let result = client.get("hello").await?;
    // this line retrieves the value of a key from the redis database. Looks like it is returned as an Option of bytes rather than the direct value
    // meaning redis stores all values as bytes

    println!("The result of the operation: {:?}", result);
    //prints out the result from the above operation (the "GET" from the redis db) in bytes
    
    Ok(())
}
