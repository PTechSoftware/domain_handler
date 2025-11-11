#[allow(dead_code)]

use std::{io::ErrorKind, path::PathBuf, sync::Mutex};
use tokio::fs;

// El nombre del archivo que será eliminado
const APP_LOCK_FILE: &str = "domain.lock"; 


#[allow(dead_code, unused,non_upper_case_globals)]
const lck: std::sync::Mutex<bool> = Mutex::new(true);

#[allow(dead_code, unused)]
/// Construye la ruta completa al lock file.
pub fn get_lock_path() -> Result<PathBuf, String> {
    lck.lock();
    // ... (lógica para encontrar el directorio de configuración) ...
    let cfg_dir = dirs::config_dir()
        .ok_or_else(|| "No se pudo determinar el directorio de configuración.".to_string())?;

    let app_dir = cfg_dir.join("domainhdlr"); 
    Ok(app_dir.join(APP_LOCK_FILE)) // -> Esta es la ruta del lock file
}

#[allow(dead_code, unused)]
///Elimina el lock file.
pub async fn remove_cfg_file() -> Result<(), Box<dyn std::error::Error>> {

    lck.lock();
    // Obtiene la ruta del lock file.
    let lock_path = get_lock_path().map_err(|e| e.to_string())?;

    if lock_path.exists() {
        // Usa fs::remove_file para eliminarlo.
        fs::remove_file(&lock_path).await?;
        println!("Lock file eliminado: {}", lock_path.display());
    } else {
        println!("Lock file no encontrado para eliminar: {}", lock_path.display());
    }

    Ok(())
}


#[allow(dead_code)]
pub async fn create_lock_file() -> Result<bool, Box<dyn std::error::Error>> {
    let lock_path = get_lock_path().map_err(|e| e.to_string())?;
    
    // Aseguramos que el directorio de la aplicación exista
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Usamos OpenOptions para intentar crear el archivo de forma exclusiva.
    // O_EXCL (exclusive) significa que fallará si el archivo ya existe.
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true) // <- Esta opción fuerza la creación exclusiva
        .open(&lock_path)
        .await 
    {
        Ok(_) => {
            // Éxito: el archivo fue creado, bloqueo obtenido.
            println!("✅ Lock file creado. Bloqueo adquirido en: {}", lock_path.display());
            Ok(true) 
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            // Fallo: el archivo ya existe, otra instancia tiene el bloqueo.
            println!("❌ El lock file ya existe. No se puede adquirir el bloqueo.");
            Ok(false) 
        }
        Err(e) => Err(e.into()), // Otro error de E/S
    }
}