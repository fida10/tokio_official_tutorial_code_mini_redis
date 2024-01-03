use tokio::{
    fs::File,
    io::{self, AsyncWriteExt},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("foo.txt").await?;

    let buffer = b"some bytes and some more bytes and more";
    let n = file.write(&buffer[..11]).await?;
    /*
    Writing in a buffered way
    The "write()" function will attempt to write the entire byte array (specified in buffer) but this is not guaranteed to succeed
    'n' will store the number of bytes successfully written
    We can also control the number of bytes written by specifying a slice of the buffer byte array
        As we did above with [..11]
    
    To ensure the entire byte array is written, we can use "write_all()", shown in 8_4
     */

    println!("Wrote the first {} bytes of 'some bytes'.", n);
    //prints out the number of bytes writte; in our case, 11, due to us specifying it in the slice
    Ok(())
}
