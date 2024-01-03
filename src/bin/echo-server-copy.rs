/*
Echo server to demo IO
*/

use tokio::{
    io::{self},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    /*
    even though we specified that this main will return a Result, since an infinite loop exists, the function will never get to the end
    and it is smart enough to realize this so does not return an error even though we are not returning a result
    */

    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    //listens on port 6142
    println!("Listening on port 6142");
    loop {
        let (mut socket, _) = listener.accept().await?;
        //accepts all incoming connections

        /*
        Now the thing with an echo server is it returns whatever it is given
        We would normally do this by taking the contents of socket copying them...back to socket:

        io::copy(&mut socket, &mut socket).await

        This...does not work. As we know, we cannot have multiple mutable references to single thing simultaneously

        Solution: Socket can be both read from and written to
        We can divide up these functionalities (meaning we divide socket into two, one which can be read from, and the other which is written to)
        */

        //let (mut socket_reader, mut socket_writer) = io::split(socket);

        /*
        in fact, any type that can be read from + written to can have the io::split() applied to it, like socket (a TcpStream type)
        while the above code works, it is more suitable for ALL manner of types that can be read and written

        TcpStream itself has 2 built in functions that are more optimized for splitting streams: .split() and .into_split()
            .split provides two instances we can use to read and write, but neither of these instances can be passed to other threads (tasks) (see below)
            .into_split() does the same thing but the instances CAN be passed between threads (tasks)
        */

        tokio::spawn(async move {
            let (mut socket_reader, mut socket_writer) = socket.split();
            /*
            We create the split here because these split instances cannot be passed to other tasks
            As a tradeoff, they require no overhead (no performance cost for using this)
            unlike into_split(), which does require overhead
             */

            let successful_copy = io::copy(&mut socket_reader, &mut socket_writer).await;
            //attempts to copy contents of the reader into the writer, returning contents to client

            if successful_copy.is_err() {
                //if this was unsuccessful, we can print an error message

                eprintln!("failed to copy eh");
            }
        });
        //spawns a new task to handle each connection
    }
}
