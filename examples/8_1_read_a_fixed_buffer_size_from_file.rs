use std::io;

use tokio::{fs::File, io::AsyncReadExt};

/*
Reading asynchronously from a file, up to a certain number of bytes:
*/
// This is a Rust attribute that enables the async features of the Rust runtime, Tokio.
#[tokio::main]

// This is the main function of the program. It's marked as async, which means it can perform asynchronous operations.
// The function returns a Result type, which is a way of handling errors in Rust. If the function completes successfully, it returns Ok(()), otherwise it returns the error.
async fn main() -> io::Result<()> {

    // This line opens a file named "read.txt" for reading. The `await` keyword is used to wait for the operation to complete.
    // The `?` operator is used to propagate errors. If the file can't be opened, the error will be returned from the main function.
    let mut file_to_read = File::open("read.txt").await?;

    // This line creates a buffer as an array of 10 zeros. This buffer will be used to store the data read from the file.
    let mut buffer = [0; 10];

    // This line reads data from the file into the buffer. The `&mut buffer[..]` expression is a mutable reference to the entire buffer.
        //Meaning we are specifying that we want to overwrite the ENTIRE buffer
    // The `await` keyword is used to wait for the operation to complete. The `?` operator is used to propagate errors.
    // If the read operation is successful, the number of bytes read is stored in `read_to_buffer` in the form of an array.
    let read_to_buffer = file_to_read.read(&mut buffer[..]).await?;
        /*
        The purpose of this line is to see how much of the file was read (and actually read the file as well with the .read() function)
        So we specifed that the buffer is 10 bytes long
        What if the file is less than 10 bytes?
        Then, additional nonsense data would be printed in the line below (i.e. the zeros, or possibly the data that was previously stored on the buffer)
        This line ensures that doesn't happen and only what was read from the file is saved in the buffer
        If there are more than 10 bytes of data in the file, this is not a problem
        If there are less than 10 bytes, this line fixes the issue
        Note that reading can also be interrupted, in which case the number of bytes that were successfully read are saved (if the reader is interrupted before the 10 bytes are read successfully, the second point (If these are less than 10 bytes...) still applies, even if the file has more than 10 bytes)

        The result is a usize, which is how many bytes were successfully read. This is used below to only take the read bytes from the buffer
        */

    // This line prints the contents of the buffer to the console. The `&buffer[..read_to_buffer]` expression is a reference to the part of the buffer that contains the data read from the file.
    // The `{:?}` format specifier is used to print the data in a format suitable for debugging.
    println!("contents of buffer - {:?}", &buffer[..read_to_buffer]);
        /*
        why not just print "&buffer"
        
        The expression &buffer[..read_to_buffer] is used to print only the portion of the buffer that was filled with data from the file.

        When you read data from a file into a buffer in Rust, the read method returns the number of bytes that were actually read. This number can be less than the size of the buffer for a couple of reasons:

        The file might be smaller than the buffer.
        The read operation might not fill the entire buffer if it's interrupted (for example, by a signal).
        By using &buffer[..read_to_buffer], you're creating a slice that includes only the part of the buffer that contains data from the file. This way, when you print the buffer, you don't print any uninitialized or leftover data.

        If you were to just print &buffer, you would print the entire buffer, including any parts that weren't filled with data from the file. This could include uninitialized data or leftover data from a previous read operation, depending on how the buffer was created (0's in this case)
        */
    
    // This line returns Ok(()) from the main function, indicating that the function has completed successfully.
    Ok(())
}