use std::io;

use tokio::{fs::File, io::AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("write.txt").await?;
    /*
    This line creates a new file with the path that is specified in the string literal
        "write.txt" in this case
    Note - Two paths exist here:
        If the file does not already exist, it is created as specified above
        If the file already exists, the existing file is truncated (its contents are cleared)
     */

    file.write_all(b"This file is being written to").await?;
    /*
    We write to the file specified above
    b"" specifies that we want to take the string literal in the quotation marks as a byte array
    This is needed since the write all function takes a byte array as input, which it then writes to the file as a string
    */

    Ok(())
}