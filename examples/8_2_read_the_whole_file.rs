use tokio::io::{self, AsyncReadExt};
use tokio::fs::File;

/*
Reading asynchronously from a file, reading the entire file:
*/

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("read.txt").await?;
    let mut buffer = Vec::new();
    /*
    Unlike the fixed buffer size from before, this will store the contents of the entire file
    Previously, we used an array of a fixed size, wheras this vector takes on the size of whatever we put in it
     */

    // read the whole file
    f.read_to_end(&mut buffer).await?;

    println!("File contents in their entirety (as bytes) {:?}", buffer);
    Ok(())
}