use fs2::FileExt;
use std::{fs::File, path::PathBuf};

use crate::{process::loop_proc::run_loop};

#[allow(unused)]
fn lockfile_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("domainhdlr/lock")
}


#[allow(unused)]
pub fn stop() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lock = lockfile_path();
    if lock.exists() {
        std::fs::remove_file(lock)?;
        println!("Service stopped.");
    } else {
        println!("Service was not running, not need to stop.");
    }
    Ok(())
}

#[allow(unused)]
pub fn status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lock = lockfile_path();
    if lock.exists() {
        println!("Service is running.");
    } else {
        println!("Service stopped.");
    }
    Ok(())
}

#[allow(unused)]
pub async fn start() ->  Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //Create the lock file
    let lock_path = lockfile_path();
    let lock_file = File::create(&lock_path)?;
    if let Err(e) = lock_file.try_lock_exclusive() {
        eprintln!("Service is running, no need to start again: {e}");
        _ = std::fs::remove_file(lock_path);
        return Ok(());
    }
    run_loop().await;
    println!("Service started in background.");
    Ok(())
}
