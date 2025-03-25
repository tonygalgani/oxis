use std::{
    env,
    fs::{self, OpenOptions},
    io::{Seek, Write},
    path,
    time::Instant,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("shred: missing file operand");
        println!("shred [options] [ dir | file1 file2 ... ]");
        return;
    }

    let _query = &args[0];
    let file_path = &args[1];

    let start_time = Instant::now();
    match shred(file_path) {
        Ok(_) => {
            let elapsed_time = start_time.elapsed();
            println!("File shredded in {elapsed_time:?}");
        }

        Err(e) => println!("File shredding failed because: {e:?}"),
    }
}

fn shred(file_path: &str) -> std::io::Result<()> {
    const OVERWRITE_PASSES: u32 = 7;
    const BLOCK_SIZE: usize = 4096; // 4 KB
    let mut file = OpenOptions::new().write(true).read(true).open(file_path)?;

    let file_length = file.metadata()?.len();
    let mut buffer = [0u8; BLOCK_SIZE];

    for _ in 0..OVERWRITE_PASSES {
        file.rewind()?;

        for chunk in (0..file_length).step_by(BLOCK_SIZE) {
            let chunk_size = std::cmp::min(BLOCK_SIZE, (file_length - chunk) as usize) as usize;
            let _ = getrandom::fill(&mut buffer[..chunk_size]);
            file.write_all(&buffer)?;
        }
        file.sync_all()?;
    }
    // truncating the file to length 0 bytes
    file.rewind()?;
    file.set_len(0)?;
    file.sync_all()?; // sync to ensure streamflow

    // rename the file
    let mut new_name = [0u8; 32usize];
    let _ = getrandom::fill(&mut new_name);
    let new_name = String::from_utf8_lossy(&new_name);
    let new_path = format!(
        "{}/{}",
        path::Path::new(file_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap(),
        new_name
    );
    fs::rename(file_path, &new_path)?;

    fs::remove_file(&new_path)?;
    Ok(())
}
