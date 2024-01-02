/*
File reorganized here to make it easier to run
cargo run --bin client
*/


use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        response: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        response: Responder<()>, //leaving this as empty as setting does not return anything from the server
    },
}
/*
modified enum to have a "Responder" field, which can be used to send a one off message with the response from the server
    i.e. for a get request, the server will return a value, which will be sent back
*/
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
/*
the one off message is sent with a oneshot channel
    this channel is optimized for point to point messages
    Neither its sender nor receiver can be cloned, and its capacity is one

    Here, we are forcing the Command enum variants to have the sender portion
    This will be used by the manager to send back values to the originator task
*/

#[tokio::main]
async fn main() {
    let (sender_cloneable, mut receiver) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        use Command::*;
        while let Some(command) = receiver.recv().await {
            match command {
                Set { 
                    key, 
                    value,
                    response
                    //response here is not used but in a production environment, we would probably want a 200 or something to be sent back to the originator, indicating that the set operation was a success

                } => {
                    client.set(&key, value).await.unwrap();
                }
                Get { 
                    key,
                    response
                } => {
                    let value = client.get(&key).await;
                    println!("Received key: {:?}", value);
                    response.send(value).unwrap();
                    /*
                    the returned value from the server is sent back to the originator task
                    no await required here because remember, this is a oneshot channel
                    meaning there are no other tasks that have the sender other than this one
                        again not possible since senders and receivers in oneshot channels are not cloneable
                    which means this executes immediately.
                    */
                }
            }
        }
    });

    let sender_copy_one = sender_cloneable.clone();
    let t1_getting = tokio::spawn(async move {
        let (send_to_task, receive_from_manager) = oneshot::channel();
        //initialize the channel

        sender_copy_one
            .send(Command::Get {
                key: "Best FPS".to_string(),
                response: send_to_task
                //sends the sender to the manager task, which uses it to send stuff back to the receive_from_manager instance
            })
            .await
            .unwrap();
        println!("Received in originator task: {:?}", receive_from_manager.await);
        //process receiver to get value received from server, and print it
    });

    let t2_setting = tokio::spawn(async move {
        let (send_to_task, receive_from_manager) = oneshot::channel();
        //initialize the channel

        sender_cloneable
            .send(Command::Set {
                key: "Best FPS".to_string(),
                value: "Halo Reach".into(),
                response: send_to_task

            })
            .await
            .unwrap();

        //receive_from_manager here is not used but in a production environment, we would probably want a 200 or something to be printed, indicating that the set operation was a success
    });

    t1_getting.await.unwrap();
    t2_setting.await.unwrap();
    manager.await.unwrap();

        /*
    Note that this code will not quite work as expected
    When it first runs, "Best FPS" key is empty, so when we do a get on it, it will return nothing (if t1_getting executes first)
    However the second time we run this client, it will return the expected value of "Halo Reach"
    We can resolve this by other functions/macros explained later to ensure the setter executes asynchronously first before the getter
     */
}
