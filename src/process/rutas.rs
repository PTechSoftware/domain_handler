// src/paths.rs
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("domainhdlr")
}

pub fn config_file() -> PathBuf {
    config_dir().join("domainhdlr.json")
}

pub fn log_file() -> PathBuf {
    config_dir().join("log_error.txt")
}

pub fn lock_file() -> PathBuf {
    config_dir().join("lock")
}

pub fn bin_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/bin")
}

pub fn bin_path() -> PathBuf {
    bin_dir().join("domainhdlr")
}

pub fn systemd_user_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/systemd/user")
}

pub fn service_path() -> PathBuf {
    systemd_user_dir().join("domainhdlr.service")
}
