/*
Client below attempts to connect with the echo server
*/

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut rd, mut wr) = io::split(socket);
    //uses the split function from tokio io library to split the read functionality from the write functionality

    // Write data in the background
    tokio::spawn(async move {
        wr.write_all(b"hi there\r\n").await?;
        wr.write_all(b"how are you\r\n").await?;
        //writes something to the stream (sent to the server)

        // Sometimes, the rust type inferencer needs
        // a little help (need to tell it that this function returns a Result so we can use the question mark operator)
        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 128];

    loop {
        let n = rd.read(&mut buf).await?;
        //reads data coming back from the server in a 128 byte buffer

        if n == 0 {
            break;
        }
        //breaks if the buffer size is 0 (meaning no more data is being sent from the server)

        let s = std::str::from_utf8(&buf[..n]).unwrap().to_string();
        println!("GOT {}", s);
    }

    Ok(())
}
