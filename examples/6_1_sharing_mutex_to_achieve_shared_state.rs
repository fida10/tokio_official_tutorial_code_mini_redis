use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
/*
Here we are specifying the object that will be shared among multiple threads
(The database)
    By using "type", we avoid having to write the whole thing out every time and can just specify 'Db' whenever we need the whole type
It can be passed and owned by multiple threads without going out of scope, because of Arc
It can be modified from multiple threads, because of Mutex
Within the hashmap, we are using Strings as keys, and Bytes (mentioned in detail in the cargo.toml) as values
    In summary, Bytes are like vector byte arrays (Vec<u8>) but better :D
*/

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    let db = Arc::new(Mutex::new(HashMap::new()));
    //creates a Hashmap instance (our database, or DB, remember) that can be passed around to various threads (Arc) and modified in various threads (Mutex)
    //The instance starts of empty
    //remember also that the Mutex being used is from std, not tokio
    //this causes whatever thread holds the lock of the Mutex to pause execution until the lock is released
        //from documentation - However, switching to tokio::sync::Mutex usually does not help as the asynchronous mutex uses a synchronous mutex internally.

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // Clone the handle to the hash map.
        let db = db.clone();
        // we create a copy to be given to the spawned task that will perform the database query (read or update)

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
            // the db instance is given over to the spawned task, which then uses it to perform DB operation
        });
    }

    //Of course, this strategy has a disadvantage: if one thread holds the lock, all other threads are blocked until the one thread releases the lock
    //This is called Contention, and it is a problem, especially with scaling applications
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                //this line puts a lock on the passed database instance from main so that we can modify it here
                db.insert(cmd.key().to_string(), cmd.value().clone());
                //this line makes the modification (inserts a value) into the database
                //note that it clones the value inside the cmd using the bytes library
                //this clone does not make a full copy of what is inside cmd.value(), rather, it allows for both the database and the cmd to own it concurrently due to the nature of Bytes
                    //this is more efficient than making a copy
                Frame::Simple("OK".to_string())
                //Lock is released, allowing other tasks to read/modify
            }           
            Get(cmd) => {
                let db = db.lock().unwrap();
                //db is locked so that the database can be read without it also concurrently being changed elsewhere
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                    //cloned via Bytes crate again, which allows for multiple ownership and avoids making multiple copies
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}