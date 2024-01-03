use tokio::{fs::File, io};

/*
Helper functions also exist for read and write, i.e. copy example: 
*/
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut data: &[u8] = b"data stored raw";
    //raw byte array data stored here
    
    let mut file = File::create("written_for_raw.txt").await?;
    //create a new file

    io::copy(&mut data, &mut file).await?;
    //copies the contents of "data" and writes them onto written_for_raw.txt asynchronously


    let mut data_two = File::open("read.txt").await?;

    let mut file_two = File::create("written_from_file_data.txt").await?;
    io::copy(&mut data_two, &mut file_two).await?;
    /*
    This copies the contents of read.txt to written_from_file_data.txt
    The helper method "copy" handles the buffer part so we need not create a buffer manually in this case
    Can directly copy one file onto another
     */

    Ok(())
}

/*
Some other helpers: 

copyio-util	Asynchronously copies the entire contents of a reader into a writer.
copy_bidirectionalio-util	Copies data in both directions between a and b.
copy_bufio-util	Asynchronously copies the entire contents of a reader into a writer.
duplexio-util	Create a new pair of DuplexStreams that act like a pair of connected sockets.
emptyio-util	Creates a new empty async reader.
repeatio-util	Creates an instance of an async reader that infinitely repeats one byte.
sinkio-util	Creates an instance of an async writer which will successfully consume all data.
splitio-util	Splits a single value implementing AsyncRead + AsyncWrite into separate AsyncRead and AsyncWrite handles.
stderrio-std	Constructs a new handle to the standard error of the current process.
stdinio-std	Constructs a new handle to the standard input of the current process.
stdoutio-std	Constructs a new handle to the standard output of the current process.

Find them here: https://docs.rs/tokio/1.35.1/tokio/io/index.html
*/