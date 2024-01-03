/*
Execute with: 
cargo run --bin main
*/

use tokio::io::{self, AsyncReadExt};
use tokio::fs::File;

/*
Reading asynchronously from a file, reading the entire file:
*/

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = Vec::new();
    /*
    Unlike the fixed buffer size from before, this will store the contents of the entire file
     */

    // read the whole file
    f.read_to_end(&mut buffer).await?;
    Ok(())
}