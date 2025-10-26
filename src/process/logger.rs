use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use chrono::Local;
use super::rutas::{log_file, log_file_error};

/// Escribe una entrada en el log. Si `overwrite` es true, reemplaza todo el contenido.
/// De lo contrario, inserta la nueva entrada al inicio.
pub fn entry_for_log(line: &str, overwrite: bool) -> io::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", timestamp, line);

    let log_path = log_file();
    let _ = ensure_file_exists(&log_path);

    if overwrite {
        overwrite_file(log_path.to_str().unwrap(), &log_entry)?;
    } else {
        prepend_log_entry(log_path.to_str().unwrap(), &log_entry)?;
    }

    Ok(())
}
pub fn entry_for_errorlog(line: &str, overwrite: bool) -> io::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", timestamp, line);

    let log_path = log_file_error();
    let _ = ensure_file_exists(&log_path);

    if overwrite {
        overwrite_file(log_path.to_str().unwrap(), &log_entry)?;
    } else {
        prepend_log_entry(log_path.to_str().unwrap(), &log_entry)?;
    }

    Ok(())
}

/// Crea el archivo si no existe.
pub fn ensure_file_exists<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if !path.as_ref().exists() {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
    }
    Ok(())
}

/// Sobrescribe el contenido de un archivo con texto nuevo.
pub fn overwrite_file(path: &str, text: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(text.as_bytes())
}

/// Inserta la entrada al inicio del archivo.
pub fn prepend_log_entry(path: &str, new_entry: &str) -> io::Result<()> {
    // Leer el contenido actual (si hay)
    let previous = fs::read_to_string(path).unwrap_or_default();

    // Escribir todo de nuevo: nueva entrada + contenido viejo
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;

    file.write_all(new_entry.as_bytes())?;
    file.write_all(previous.as_bytes())
}


#[allow(unused)]
const MAX_LOG_SIZE: u64 = 25 * 1024; // 25 KB

#[allow(unused)]
pub fn purge_log() -> io::Result<()> {
    let log_path = log_file();

    // ✅ Crear archivo si no existe
    if !log_path.exists() {
        println!("Log file does not exist. Creating it now.");
        fs::File::create(&log_path)?;
        return Ok(()); // No hay nada que purgar
    }

    // ✅ Intentar leer metadata
    match fs::metadata(&log_path) {
        Ok(metadata) => {
            let file_size = metadata.len();
            if file_size > MAX_LOG_SIZE {
                fs::remove_file(&log_path)?; // o renombrar, si preferís rotación
                println!(
                    "Log file exceeded size limit ({} bytes). Deleted.",
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
            eprintln!(
                "Could not access log file '{}': {}",
                log_path.display(),
                e
            );
        }
    }

    Ok(())
}
/// Reads and returns all log entries as a vector of strings.
/// If the log file does not exist, returns an empty vector.
#[allow(unused)]
pub fn read_log_errors() -> io::Result<Vec<String>> {
    let log_path = log_file();
    let error_log_path = log_file_error();

    let log_ok = match fs::read_to_string(&log_path) {
        Ok(ctn) => ctn,
        _ => String::new()
    };

    let log_err = match fs::read_to_string(&error_log_path) {
        Ok(ctn) => ctn,
        _ => String::new()
    };

    Ok(vec![
        "Log OK : ".to_string(),
        log_ok,
        "Log Error: ".to_string(),
        log_err
        ])
}
