use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;
    //imports required for this particular function

    let mut db = HashMap::new();
    //our "database". it is just a map, has nothing in it

    let mut connection = Connection::new(socket);
    //This connection object appears to allow us to "process" the string, basically loop through it
    //Think of it like a damn on a river. the river is the "stream" of data coming from the connected client

    while let Some(data) = connection.read_frame().await.unwrap() {
        //the loop, which iterates through the elemnents within the stream while the connection is active
        //disconnection paths specified in main function, see comment there
        let response = match Command::from_frame(data).unwrap() {
            Set(data_in_frame) => {
                db.insert(
                    data_in_frame.key().to_string(),
                    data_in_frame.value().to_vec(),
                );
                Frame::Simple(String::from("OK"))
                //this is actually mandatory for redis, if a set operation was done successfully, a simple "OK" string should be returned
                //any other string will cause an error
            }
            Get(key_to_get) => {
                if let Some(value) = db.get(key_to_get.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("Unimplemented: {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();

        /*
        The above loops through each of the elements received from the stream
        These individual elements are redis commands sent from the client (in other words, this server hosts a database and the client is trying to set key value pairs or get the values of some keys)

        If the command is a "Set" command, the key value pairs embedded in the command are added to the map, local to this server
            In production, this would probably be a bit more robust and saved persistently
        If the command is a "Get", then 2 paths are checked
            If the key embedded in the command is in the map, its corresponding value is cloned and returned
                It is cloned so that it is not moved out of the map
                We use "into" because in order to return it (by adding it to the stream to be sent back to the client), we need to convert the value into a byte array
            Else if the key does not exist in the map, Null is written to the frame and returned back to the sender
        If the command is something other than "Get" or "Set", a panic happens and tells us what command we just tried to run
            Of course, since this process runs in a spawned task, if it panics, it just closes that task and we are free to try again
            Though it does look like that will wipe the "db", meaning any data we stored will be lost
            Which is why in production, we'd use something a bit more robust than just a simple hashmap like this

        The result of the above is a Frame with something written on it, depending on which of the match arms was executed. This is returned to the sender (client)

        Note that after this connection is terminated, the db gets wiped as well. So parallel/subsequent connections cannot access data input from other connections.

        This is remedied later.
         */
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream_from_client, _) = listener.accept().await.unwrap();
        tokio::spawn(async move { process(stream_from_client).await });
    }

    /*
    Communication with the server essentially works like this:
    We first create a listener, which "listens" for incoming connection requests made at a specified address and port
    We then accept the connection, which results in a stream. The source (start) of this stream is the client initiating the connection, and the results are received here
    We then write some logic (in this case the process function) to process the stream contents
    The connection (and stream) remain active until either the client or server (tcp listener object) terminate the connection
        This could be the client having some logic ending communication
        Or when the stream_from_client, which holds the connection, goes out of scope
     */
}
