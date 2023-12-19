use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

#[tokio::main]
async fn main(){
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    //this creates an instance called a "listener", which listens to the address/port specified.
    //basically creating a mini server at this port which listens and responds
    //this returns a future, which is handled with await

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        process(socket).await;
    }
    //the above loop continuously waits for a connection to be established with the client IP address specified above
    //once a connection is attempted, it is accepted (using accept()) and the clients connection details are stored in a tuple (socket, _)
    
    //this is then processed using the logic below (which receives something from the cliend and sends something back using "socket")

    //the way this loop works is:
    // - it gets a connection from a client
    // - it processes the data received from the client
    // - the loop then runs again, deallocating the connection with the client that was previously made
    // - and waiting for a new connection to be made
}

async fn process(socket: TcpStream){
    //this process function will process the "stream" or river of data that we receive from the above address

    let mut connection = Connection::new(socket);
    //this creates a new connection struct instance that collects data from the stream, for processing below

    if let Some(frame) = connection.read_frame().await.unwrap() {
        //we read the data from the connection (which might take a while as we wait for something to be sent from the client) and then unwrap the result
        println!("Received: {:?}", frame);
        //when we receive something (data as a "frame") from the client, it is printed out here
        //the client in examples/hello-redis.rs is sending us some data in the form of a key value pair being set inside of a redis database
        //this is subsequently printed out

        let response: Frame = Frame::Error("some other error 2".to_string());
        //this creates a new piece of data that is an error, which can be read from the stream

        connection.write_frame(&response).await.unwrap();
        //this error message is then sent back to the client, which gets a printout in the terminal

        //if we comment the response line and connection.write_frame line, then the client gets a default error output since the connection is reset (because of the loop above)
    } 
}

