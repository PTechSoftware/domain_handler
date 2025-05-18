use std::fs;
use std::path::PathBuf;
use serde_json;
use crate::models::entry::Entry;

use super::rutas::config_file;

#[allow(unused)]
fn config_path() -> PathBuf {
    config_file()
}
#[allow(unused)]
fn load_domains() -> Vec<Entry> {
    let path = config_path();
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_else(|_| vec![])
}
#[allow(unused)]
fn save_domains(entries: &[Entry]) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(entries).unwrap();
    let _ = fs::write(path, json);
}
#[allow(unused)]
pub fn add_domain(name: &String, token: &String, activated: Option<bool>, txt: Option<String>) {
    let mut entries = load_domains();

    if entries.iter().any(|e| e.name == *name) {
        println!("Domain '{}' already exists. Use delete first if you want to replace it.", name);
        return;
    }

    entries.push(Entry {
        name: name.clone(),
        token: token.clone(),
        activated: activated.unwrap_or(true),
        txt,
    });

    save_domains(&entries);
    println!("Domain '{}' added.", name);
}

#[allow(unused)]
pub fn delete_domain(name: &String) {
    let mut entries = load_domains();
    let initial_len = entries.len();
    entries.retain(|e| e.name != *name);
    save_domains(&entries);
    if entries.len() < initial_len {
        println!("Domain '{}' deleted.", name);
    } else {
        println!("Domain '{}' not found.", name);
    }
}
#[allow(unused)]
pub fn list_domains() -> Vec<Entry> {
    let entries = load_domains();
    for e in &entries {
        println!(
            "- {} (active: {}, txt: {})",
            e.name,
            e.activated,
            e.txt.as_deref().unwrap_or("None")
        );
    }
    entries
}
