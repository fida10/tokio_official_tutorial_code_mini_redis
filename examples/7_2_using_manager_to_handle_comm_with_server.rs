use bytes::Bytes;
use mini_redis::{client};
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, value: Bytes },
}
//for simplicity, creating an enum that can be used to send commands to the created client management task

#[tokio::main]
async fn main() {
    let (sender_cloneable, mut receiver) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        //establish connection with the server
        use Command::*;
        while let Some(command) = receiver.recv().await {
            //continuously receives commands until senders are all deallocated
            match command {
                Set { key, value } => {
                    client.set(&key, value).await.unwrap();
                }
                Get { key } => {
                    let value = client.get(&key).await.unwrap();
                    println!("Received key: {:?}", value)
                }
            }
        }
    });
    /*
    This manager thread handles setting/getting values from the server (which, remember, hosts the database)
    Fairly self explanatory
     */

    let sender_copy_one = sender_cloneable.clone();
    let t1_getting = tokio::spawn(async move {
        sender_copy_one
            .send(Command::Get {
                key: "Best FPS".to_string(),
            })
            .await
            .unwrap();
    });
    //spawns new task, creates a Get enum variant, sends it to the manager task which then sends it to the server
    
    let t2_setting = tokio::spawn(async move {
        sender_cloneable.send(
            Command::Set { key: "Best FPS".to_string(), value: "Halo Reach".into() }
        ).await.unwrap();
    });
    //same thing as above task but sets the value with a Set enum variant

    t1_getting.await.unwrap();
    t2_setting.await.unwrap();
    manager.await.unwrap();
    //these three lines force all of these tasks to complete before shutting off the main task

}