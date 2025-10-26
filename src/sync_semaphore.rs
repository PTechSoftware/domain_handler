use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};

#[allow(unused)]
static SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(1)));

/// Intenta adquirir el lock global.
/// Devuelve `None` si ya hay una instancia corriendo.
pub async fn acquire_single_instance() -> Option<OwnedSemaphorePermit> {
    let sem = SEMAPHORE.clone();
    match sem.try_acquire_owned() {
        Ok(permit) => {
            println!("✅ Lock global adquirido.");
            Some(permit)
        }
        Err(_) => {
            eprintln!("⚠️ Ya existe una instancia activa. Abortando ejecución.");
            None
        }
    }
}
