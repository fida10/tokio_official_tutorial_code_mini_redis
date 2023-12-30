/*
How to handle contention issues? I.e. if one mutex holding the lock on a shared database is causing an issue, here are some possible solutions:

Switching to a dedicated task to manage state and use message passing.
Shard the mutex.
Restructure the code to avoid the mutex.

In our case, we will use sharding here

Sharding involves dividing the database into multiple instances (multiple hash maps), called "Shards"
Since each instance is independent from the others, we can easily access them all simultaneously
Since they are all part of the same database, they are all linked together too, allowing us to share state
*/

use bytes::Bytes;
use mini_redis::Command::{self, Get, Set};
use mini_redis::{Connection, Frame};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;
//here, we specify the type of the ShardedDb
//Arc allows it to be passed across multiple threads (owned by multiple threads)
//The outer vec is the database of shards. There can be many of these, and each of these "shards" is enclosed inside of a Mutex for usage across multiple threads
//The Hashmap inside used to be a single hashmap we shared across multiple threads; as stated above, this could cause an issue due to contention
//Now, there are many "Shards", each of them containing a hashmap, which can be modified simultaneously by multiple threads since they are technically independent from each other
//At the same time, they all exist in the same ShardedDb instance, so the state is shared

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}
/*
This function creates the Sharded database
We must specify exactly how many shards we want the database to have, specified in the parameter num_shards
That many shards are pushed into the SharedDb by the for loop
The result is returned in an Arc so that it may be owned between many separate tasks
*/

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    let shardeddb = new_sharded_db(1000);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // Clone the handle to the hash map.
        let db = shardeddb.clone();
        // we create a copy to be given to the spawned task that will perform the database query (read or update)
        // in this case it is a shallow copy (allowing multiple ownership thanks to Arc) that is passed to the task, wherein it will make its modifications to its respective shard

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

// This is an asynchronous function named `process` that takes a `TcpStream` and a `ShardedDb` as arguments.
async fn process(socket: TcpStream, db: ShardedDb) {
    // Create a new `Connection` from the `TcpStream`.
    let mut connection = Connection::new(socket);

    // Start a loop that continues as long as `read_frame()` returns `Some(frame)`.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        // Match on the result of converting the frame into a `Command`.
        let response = match Command::from_frame(frame).unwrap() {
            // If the command is a `Set` command...
            Set(cmd) => {
                // Get the key from the command and convert it to a string.
                let key = cmd.key().to_string();
                // Get the value from the command and clone it.
                let value = cmd.value().clone();

                // Create a new `DefaultHasher`.
                let mut hasher = DefaultHasher::new();
                // Hash the key.
                key.hash(&mut hasher);
                // Calculate the shard index by taking the hash modulo the number of shards.
                // Lock the shard for exclusive access.
                let mut shard = db[(hasher.finish() as usize) % db.len()].lock().unwrap();
                /*
                This section deserves further explanation --> 
                    Summary: 

                    Why use a hasher? Because in this line: key.hash, the "key" is hashed and transformed into a hash code, which is a fixed-size numerical representation of the key. Even a small change in the key will produce a very different hash code. In other words, a unique number is generated from the key which remains consistent for that key (meaning if we were to hash the key again, that same unique number would be generated)

                    This is then modulo'd against the length of the db (db.len()), which is the total number of shards we specified this sharded database would have (we specified this as 1000 above)

                    The result of db[(hasher.finish() as usize) % db.len()] gives us one of the 1000 shards from within the database. The benefit of this is since the hash value of the key remains consistent, whenever we try to find that key, we will ALWAYS end up at the same shard, meaning we will always find the key we are looking for (we don't need to go digging through every shard)
                    
                    1. `let mut hasher = DefaultHasher::new();`: This line is creating a new instance of `DefaultHasher`, which is a built-in hasher provided by Rust. A hasher is a function that takes data and returns a hash code, which is a fixed-size numerical or alphanumeric representation of the data.

                    2. `key.hash(&mut hasher);`: This line is hashing the key. The `hash` method is a part of the `Hash` trait, which the `String` type implements. This means that any string (like `key` in this case) can be hashed. The hashed value of the key is stored in `hasher`.

                    3. `let mut shard = db[(hasher.finish() as usize) % db.len()].lock().unwrap();`: This line is doing several things:
                        - `hasher.finish()`: This method finalizes the hashing process and returns the hash code as a `u64`.
                        - `(hasher.finish() as usize) % db.len()`: This is converting the hash code to `usize` (the type used for indexing in Rust) and then taking the modulus of the number of shards (`db.len()`). This effectively maps the hash code to one of the shards in the database.
                        - `db[...].lock().unwrap()`: This is accessing the shard at the calculated index and locking it for exclusive access. The `lock` method returns a `Result`, so `unwrap` is used to get the `MutexGuard` from the `Result`. If the lock could not be acquired (which should not happen in normal operation), `unwrap` will panic.

                    4. The next step (not shown in the selected code) would be to insert the key-value pair into the shard. This is done using the `insert` method of the `HashMap` that is guarded by the mutex.

                    The purpose of this code is to distribute the data across multiple shards based on the hash of the key. This is a common technique used to reduce contention in multi-threaded scenarios. By distributing the data, the chances that two threads will try to access the same data (and thus contend for the same lock) are reduced.
                 */

                // Insert the key-value pair into the shard.
                shard.insert(key, value); 
                //remember, here the key is not being stored in its original form, but as a u64 in hashed form
                //so retrieving it would also mean converting the retrieval key in hash form

                // Return a simple "OK" frame.
                Frame::Simple("OK".to_string())
            }
            // If the command is a `Get` command...
            Get(cmd) => {
                // Get the key from the command and convert it to a string.
                let key = cmd.key().to_string();

                // Create a new `DefaultHasher`.
                let mut hasher = DefaultHasher::new();
                // Hash the key.
                key.hash(&mut hasher);
                // Calculate the shard index by taking the hash modulo the number of shards.
                // Lock the shard for exclusive access.
                let shard = db[(hasher.finish() as usize) % db.len()].lock().unwrap();
                //this finds the unique shard that our desired key is located in (how this works is explained in more detail above in the set section)

                // If the shard contains the key...
                if let Some(value) = shard.get(&key) {
                    // Return a bulk frame containing the value.
                    Frame::Bulk(value.clone())
                } else {
                    // Otherwise, return a null frame.
                    Frame::Null
                }
            }
            // If the command is neither `Set` nor `Get`...
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response frame to the connection.
        connection.write_frame(&response).await.unwrap();
    }
}
