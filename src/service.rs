use anyhow::Ok;
use fs2::FileExt;
use std::{fs::File, path::PathBuf};

use crate::process::loop_proc::run_loop;

#[allow(unused)]
fn lockfile_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("domainhdlr/lock")
}


#[allow(unused)]
pub fn stop() -> anyhow::Result<()> {
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
pub fn status() -> anyhow::Result<()> {
    let lock = lockfile_path();
    if lock.exists() {
        println!("Service is running.");
    } else {
        println!("Service stopped.");
    }
    Ok(())
}

#[allow(unused)]
pub async fn start() -> anyhow::Result<()> {
    //Create the lock file
    let lock_path = lockfile_path();
    let lock_file = File::create(&lock_path)?;
    if let Err(e) = lock_file.try_lock_exclusive() {
        eprintln!("Service is running, no need to start again: {e}");
        return Ok(());
    }
    //Do start stuff


    run_loop().await;
    println!("Service started in background.");
    Ok(())
}
