use std::{
    env, fs,
    io::{Result, Seek, SeekFrom, Write},
    path::{self, PathBuf},
    time::Instant,
};

fn main() {
    // Collect command-line arguments into a vector of strings
    // This vector will contain the program name, the elements to shred and option flags
    let cli_arguments: Vec<String> = env::args().collect();

    // Check if the number of arguments is less than 2 which means no elements was given
    // If so, print an error message and return from the function
    if cli_arguments.len() < 2 {
        // Print an error message indicating that a file operand is missing
        eprintln!("ERROR: no elements given to shred");
        // Print a usage message showing the correct syntax
        println!("Usage: oxis [flags] [files | dirs | devices]");
        std::process::exit(1);
    }

    // Time the entire program run duration
    //    let global_start_time = Instant::now();

    // Future flag options
    /*
        let mut zero = false;
        let mut full_zero = false;
        let mut verbose = false;
    */

    // Get the program name and file path from the arguments vector
    // The program name is the first argument, the others are either flag options, file paths,
    // dirs or devices name.
    // let _query = &cli_arguments[0]; // Name of the program itself

    let mut elements_to_shred: Vec<String> = Vec::new();

    // This loop isolates the elements_to_shred from the option flags which are contained within
    // the cli_arguments vector.
    for argument in &cli_arguments[1..] {
        /*match arg {
            "-v" | "--verbose" => verbose = true,
            _ => elements_to_shred.push(arg.to_string()),
        }*/
        elements_to_shred.push(argument.to_string());
    }

    // This one loop evaluates the elements_to_shred vector contents and process to shred the
    // element correctly. It could be a simple file, a directory or an entire device.
    for argument in elements_to_shred {
        if let Err(evaluation_error) = element_evaluation(&PathBuf::from(argument)) {
            eprintln!("ERROR: Failed to evaluate element [{evaluation_error:?}]")
        }
    }
    // let global_duration = global_start_time.elapsed();
    // println!("The entire shredding process lasted {global_duration:?}");
}
fn element_evaluation(element_path: &path::PathBuf) -> Result<()> {
    if element_path.is_dir() {
        for entry in fs::read_dir(element_path)? {
            let entry = entry?;
            let path: PathBuf = entry.path();
            if path.is_dir() {
                element_evaluation(&path)?;
            } else {
                if let Err(shred_error) = shred(&path) {
                    eprintln!("ERROR: Failed to shred {path:?} [{shred_error}]")
                }
            }
        }
        fs::remove_dir(element_path)?;
    } else {
        if let Err(shred_error) = shred(element_path) {
            eprintln!("ERROR: Failed to shred {element_path:?} [{shred_error}]")
        }
    }
    Ok(())
}

fn shred(file_path: &path::PathBuf /*, zero: bool*/) -> Result<()> {
    // Start a timer to time the shredding process
    let start_time = Instant::now();
    // Parameters (temporary hardcoded)
    const OVERWRITE_PASSES: u32 = 7;
    const BLOCK_SIZE: usize = 4096; // 4 KB

    // Use the OpenOptions struct to open the file to shred with write permission
    // Handle all error that could happen through the process with the ? operator.
    let mut file = fs::OpenOptions::new().write(true).open(file_path)?;

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
            // if !full_zero {
            // Fill the buffer with random data using the fill function
            // The expect method is used to handle any errors that occur during buffer filling
            getrandom::fill(&mut buffer[..chunk_size])
                .expect("Filling a buffer with random data failed");
            // }
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

    let parent = path::Path::new(file_path).parent().unwrap();
    let new_path: String;
    if parent.to_str().unwrap().is_empty() {
        new_path = new_name.to_string();
    } else {
        new_path = format!("{}/{}", parent.display(), new_name);
    };

    /* debug print
    println!("{file_path:?}");
    println!("{new_name:?}");
    println!("{new_path:?}");
    */
    // Rename the shredded file to the temporary name
    fs::rename(file_path, &new_path)?;

    // Remove the temporary file to complete the shredding process
    fs::remove_file(&new_path)?;

    let shredding_duration = start_time.elapsed();
    println!("{file_path:?} shredded in {shredding_duration:?}");

    // Return Ok to indicate that the shredding was successful
    Ok(())
}
