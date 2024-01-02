/*
File reorganized here to make it easier to run
cargo run --bin server
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

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening");
    let db = new_sharded_db(1000);
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
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
                let mut db_shard = db[shard].lock().unwrap();
                db_shard.insert(key, value);
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let key = cmd.key().to_string();
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                let shard = (hasher.finish() as usize) % db.len();
                let db_shard = db[shard].lock().unwrap();
                if let Some(value) = db_shard.get(&key) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}