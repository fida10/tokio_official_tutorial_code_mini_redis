/*
Another possible strategy that can be used to share the state of a database is "Sharding"
This involves breaking the database up into Shards, or individual pieces, and using hashing to determine which shard should contain which key
Explained below
*/

use bytes::Bytes;

use mini_redis::{Connection, Frame};
use mini_redis::Command::{self, Get, Set};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;
/*
Here, we create the type of the Sharded database
The Arc is so this database may be owned by multiple threads/tasks
The outer Vec holds all of the individual shards that comprise the database
Each individual shard is held in a Mutex, so that it may be modified from multiple threads
    This is of course unlikely since there are so many shards and few connections, but for scaling, as connections increase, we can increase the number of shards
    The Mutex is there to handle this situation
The HashMap IS the individual shard, within which keys are of type "String" and values are of type Bytes
    Bytes work similarly to Vec<u8> but they can be passed around easily between threads (and owned by multiple threads)
    Bytes type is roughly an Arc<Vec<u8>> but with some added capabilities
*/

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    //A sharded database must have a specified number of shards on creation. This is specified in the function parameter above
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
        //this loop goes about creating the individual shards, the number of which is specified in the function parameter
    }
    Arc::new(db)
    //the newly created databse is returned in an Arc so it can be shared across multiple threads/tasks
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    let db = new_sharded_db(1000);
    //here we initialize the sharded database, with 1000 shards

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // Clone the handle to the hash map.
        let db = db.clone();
        // "Clone" or shallow copies the database to pass onto a newly spawned task for every connection (possible due to Arc)

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: ShardedDb) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let key = cmd.key().to_string();
                let value = cmd.value().clone();

                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                let shard = (hasher.finish() as usize) % db.len();
                //124884239582 % 1000 = 582
                //9 % 1000 = 9
                //124903 % 1000 = 903
                //each time it generates a (consistent for the key) shard index that will never surpass db.len (number of shards)

                /*
                Explanation of a hasher
                The reason for a hasher is to turn the key into a u64 (We take the key that is received from the connection, hasher takes this key (key.hash and then hasher.finish) and turns it into a unique u64 value
                We then divide this value by the length of db and take the remainder (modulus)
                    Recall that this is the number of created shards
                The result is a value between 0 and the number of shards (1000 in this case, so 0 - 999)
                This value is used to assign which shard this key will be assigned to
                This saves time/computational power because every key will always give the same u64 when hashed
                This value can then be used to find out EXACTLY which shard this key is being stored in
                 */

                let mut db_shard = db[shard].lock().unwrap();
                db_shard.insert(key, value);
                //after the above, we then unlock the shard, input the key value pair into that shard and then deallocate the lock

                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let key = cmd.key().to_string();

                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                let shard = (hasher.finish() as usize) % db.len();

                //Similar to what happened above, we use the unique u64 value of the hashed key to identify exactly which shard to look in to retrieve the value, rather than having to loop through all of them

                let db_shard = db[shard].lock().unwrap();
                if let Some(value) = db_shard.get(&key) {
                    Frame::Bulk(value.clone())
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