use sysinfo::System;

use crate::process::{
    file_lock::{get_lock_path, remove_cfg_file},
    loop_proc::run_loop,
    notifier::MailConfig,
};

#[allow(unused)]
pub async fn stop(cfg: &MailConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lock_path = get_lock_path().unwrap();
    if lock_path.exists() {
        if let Ok(pid_str) = tokio::fs::read_to_string(&lock_path).await {
            // BORRAMOS EL LOCK ANTES PARA QUE EL SUPERVISOR NO LO REINICIE
            remove_cfg_file().await;

            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let mut s = System::new();
                s.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
                for (proc_pid, proc) in s.processes().iter() {
                    if proc_pid.as_u32() == pid {
                        let _ = proc.kill_and_wait();
                        println!("Killed pid -> {}", pid);
                    }
                }
            }
        }
    } else {
        println!("Lock file no existe. El servicio probablemente no esté corriendo.");
    }
    Ok(())
}

#[allow(unused)]
pub fn status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("### Estado del Sistema ###");
    println!("=========================");

    let lock_path = get_lock_path().unwrap();
    println!("Estado File: existe [{}]", lock_path.exists());

    if lock_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let mut s = System::new();
                s.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
                let mut found = false;
                for (proc_pid, proc) in s.processes().iter() {
                    if proc_pid.as_u32() == pid {
                        found = true;
                        println!("Proceso 'domainhdlr' corriendo (PID: {})", pid);
                        println!("Uso de CPU: {:.2}%", proc.cpu_usage());
                        println!("Uso de Memoria: {:.2} MB", proc.memory() as f64 / 1024.0);
                    }
                }
                if !found {
                    println!(
                        "Lock file existe pero el proceso con PID {} no está corriendo.",
                        pid
                    );
                }
            }
        }
    } else {
        println!("No se encontró el proceso 'domainhdlr' (lock file no existe).");
    }

    println!("-------------------------");
    let status_path = crate::process::rutas::status_file();
    if status_path.exists() {
        if let Ok(status_str) = std::fs::read_to_string(&status_path) {
            println!("Reporte del proceso:\n{}", status_str);
        }
    } else {
        println!(
            "Aún no hay información de estado (el servicio acaba de iniciar o no está corriendo)."
        );
    }

    println!("=========================");
    Ok(())
}

#[allow(unused)]
pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lock_path = get_lock_path().unwrap();
    if let Some(parent) = lock_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let pid = std::process::id();
    tokio::fs::write(&lock_path, pid.to_string()).await?;
    println!("Worker running with PID {}", pid);

    run_loop().await;
    println!("Worker stopped.");
    Ok(())
}
