use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Read, Write, Seek, SeekFrom};
use chrono::Local;

/// Adds a log entry with a timestamp to the beginning of the file.
/// Creates the file if it doesn't exist.
///
/// # Arguments
/// - `line`: The log entry to add.
pub fn entry_for_log(line: &str) -> io::Result<()> {
    // Get the current date and time
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Format the log entry
    let log_entry = format!("[{}] {}\n", timestamp, line);

    // Open the file in read and write mode, creating it if it doesn't exist
    let file_path = "log_error.txt"; // Change this to your desired log file path
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)?;

    // Read the current contents of the file
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    // Seek to the beginning of the file
    file.seek(SeekFrom::Start(0))?;
    // Write the new log entry followed by the original contents
    file.write_all(log_entry.as_bytes())?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Overwrites a file with the given text, removing any existing content.
/// Creates the file if it doesn't exist.
///
/// # Arguments
/// - `path`: The path to the file.
/// - `text`: The text to write to the file.
#[allow(unused)]
pub fn overwrite_file(path: &str, text: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(text.as_bytes())?;
    Ok(())
}
#[allow(unused)]
pub fn purge_log() -> io::Result<()> {
    let file_path = "log_error.txt"; // Path to your log file
    const MAX_SIZE: u64 = 512 * 512; 

    if !fs::metadata(file_path).is_ok() {
        println!("File {} does not exist. No action taken.", file_path);
        return Ok(());
    }

    match fs::metadata(file_path) {
        Ok(metadata) => {
            let file_size = metadata.len();
            if file_size > MAX_SIZE {
                fs::remove_file(file_path)?;
                println!("File {} exceeded 3 MB and was deleted.", file_path);
            } else {
                println!(
                    "File {} is within the size limit ({} bytes). No action taken.",
                    file_path, file_size
                );
            }
        }
        Err(e) => eprintln!("Failed to access metadata for {}: {}", file_path, e),
    }
    Ok(())
}