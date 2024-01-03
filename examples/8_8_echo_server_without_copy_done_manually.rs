/*
Executing the same echo server content, but without copy
Instead we use read and write
*/

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            //necessary intermediary buffer since we cannot directly write to the socket from the socket (see echo client using copy for why, this creates 2 mutable references to the same thing simultaneously)

            loop {
                match socket.read(&mut buf).await {
                    // Return value of `Ok(0)` signifies that the remote (client) has closed
                    // This means we have reached the end of the file that was being read
                    Ok(0) => return,
                    //this then breaks the loop since the client is no more
                    //we should take care to do this; if we do not, the loop will run forever

                    Ok(n) => {
                        // Copy the data back to socket
                        if socket.write_all(&buf[..n]).await.is_err() {
                            //writes data back to the socket that was received from the socket
                            //here, we need to use &buf as an intermediary, wheras with split, we did not need to do this

                            // Unexpected socket error. There isn't much we can
                            // do here so just stop processing.
                            return;
                        }
                    }
                    Err(_) => {
                        // Unexpected socket error. There isn't much we can do
                        // here so just stop processing.
                        return;
                    }
                }

            }
        });
    }
}