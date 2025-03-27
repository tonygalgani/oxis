use std::{
    env,
    error::Error,
    fs,
    fs::OpenOptions,
    io::{Seek, SeekFrom, Write},
    path,
    time::Instant,
};

fn main() {
    // Collect command-line arguments into a vector of strings
    // This vector will contain the program name, the operand to shred and any additional arguments
    let args: Vec<String> = env::args().collect();
    // Check if the number of arguments is less than 2 which means no operand was given
    // If so, print an error message and return from the function
    if args.len() < 2 {
        // Print an error message indicating that a file operand is missing
        println!("shred: missing file operand");
        // Print a usage message showing the correct syntax
        println!("shred [ file ]");
        // Return from the program
        return;
    }

    // Get the program name and file path from the arguments vector
    // The program name is the first argument, and the file path is the second argument
    let _query = &args[0];
    let file_path = &args[1];

    // Record the start time of the shredding process
    // This will be used to calculate the elapsed time later
    let start_time = Instant::now();

    // Call the shred function to shred the file
    // Match the result of the shred function to handle both success and error cases
    match shred(file_path) {
        // If the shredding was successful, print a success message with the elapsed time
        // The elapsed time is calculated with the elapsed method from the Instant object
        Ok(_) => {
            let elapsed_time = start_time.elapsed();
            println!("File shredded in {elapsed_time:?}");
        }
        // If the shredding has failed, print an error message with the file's path and the error
        // type
        Err(shred_error) => eprintln!("Failed to shred file {file_path:?} : {shred_error}"),
    }
}

fn shred(file_path: &str) -> Result<(), Box<dyn Error>> {
    // Parameters (temporary hardcoded)
    const OVERWRITE_PASSES: u32 = 7;
    const BLOCK_SIZE: usize = 4096; // 4 KB

    // Open the file to shred with solely write permission using the OpenOptions type.
    // Any error is propagated because of the ? operator.
    let mut file = OpenOptions::new().write(true).read(true).open(file_path)?;

    // Seek to the end of the file to get its length
    // The seek method returns the new position in the file, which is the file length
    let file_length = file.seek(SeekFrom::End(0))?; // Get file's length

    // Create a buffer to hold random data for overwriting the file
    // The buffer size is equal to the block size
    let mut buffer = [0u8; BLOCK_SIZE]; // Block sized buffer for random data

    // Perform multiple overwrite passes to ensure the file is thoroughly shredded
    for _ in 0..OVERWRITE_PASSES {
        // Rewind the file to the beginning to start overwriting from the start
        file.rewind()?;

        // Overwrite the file in chunk by chunk.
        for chunk in (0..file_length).step_by(BLOCK_SIZE) {
            // Calculate the size of the current chunk
            // If the chunk is smaller than the block size, use the smaller size
            let chunk_size = std::cmp::min(BLOCK_SIZE, (file_length - chunk) as usize) as usize;
            // Fill the buffer with random data using the fill function
            // The expect method is used to handle any errors that occur during buffer filling
            fill(&mut buffer[..chunk_size]).expect("Filling a buffer with random data failed");
            // Overwrite the current chunk with the random data in the buffer
            file.write_all(&buffer)?; // overwrite the file
        }
        // Sync the file data to ensure that the overwrites are persisted to disk
        file.sync_data()?;
    }
    // Rewind the file to the beginning to truncate it to zero length
    file.rewind()?;

    // Truncate the file to zero length to remove any remaining data
    file.set_len(0)?;

    // Sync the file data again to ensure that the truncation is persisted to disk
    file.sync_data()?;

    // Generate a temporary file name to rename the shredded file
    // The temporary name is a string of 32 zeros
    let new_name = String::from_utf8_lossy(&[b'0'; 32usize]);

    // Construct the full path of the temporary file
    // The temporary file is located in the same directory as the original file
    let new_path = format!(
        "{}/{}",
        path::Path::new(file_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap(),
        new_name
    );
    // Rename the shredded file to the temporary name
    fs::rename(file_path, &new_path)?;

    // Remove the temporary file to complete the shredding process
    fs::remove_file(&new_path)?;

    // Return Ok to indicate that the shredding was successful
    Ok(())
}
