use std::fs::{self, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use chrono::Local;
use crate::process::rutas::log_file;

/// Adds a log entry with a timestamp to the beginning of the file.
/// Creates the file if it doesn't exist.
pub fn entry_for_log(line: &str) -> io::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", timestamp, line);

    let log_path = log_file();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&log_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(log_entry.as_bytes())?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Overwrites a file with the given text, removing any existing content.
/// Creates the file if it doesn't exist.
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
    const MAX_SIZE: u64 = 512 * 512; // 256 KB
    let log_path = log_file();

    match fs::metadata(&log_path) {
        Ok(metadata) => {
            let file_size = metadata.len();
            if file_size > MAX_SIZE {
                fs::remove_file(&log_path)?;
                println!(
                    "Log file exceeded size limit ({} bytes). File deleted.",
                    file_size
                );
            } else {
                println!(
                    "Log file is within size limit ({} bytes).",
                    file_size
                );
            }
        }
        Err(e) => {
            println!("Log file not found or inaccessible. No action taken. ({})", e);
        }
    }

    Ok(())
}

/// Reads and returns all log entries as a vector of strings.
/// If the log file does not exist, returns an empty vector.
#[allow(unused)]
pub fn read_log_errors() -> io::Result<Vec<String>> {
    let log_path = log_file();

    match fs::read_to_string(&log_path) {
        Ok(content) => Ok(content.lines().map(|s| s.to_string()).collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(vec![]),
        Err(e) => Err(e),
    }
}
