use fs2::FileExt;
use sysinfo::System;
use std::{fs::File, path::PathBuf};

use crate::process::{loop_proc::run_loop, notifier::{MailConfig, send_email_alert}};

#[allow(unused)]
fn lockfile_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("domainhdlr/lock")
}


#[allow(unused)]
pub fn stop(cfg: &MailConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //Force kill
    let mut s = System::new();
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, false);

    for (pid, proc_name) in s.processes().iter() {

        if proc_name.name() == "domainhdlr" {
            let s = proc_name.kill_and_wait();
            if let Err(kill) = s {
                let body = format!("No se pudo matar la instancia con PID: {}",pid.as_u32());
                send_email_alert(cfg, "Kill error", "");
            }else{
                println!("Killed pid -> {}",pid.as_u32())
            }

        }
    }
    Ok(())
}

#[allow(unused)]
pub fn status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut s = System::new();
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, false);

    // Imprimimos el encabezado
    println!("### Estado del Sistema ###");
    println!("=========================");

    let mut found = false;  // Para verificar si encontramos el proceso

    // Iteramos sobre los procesos para encontrar "domainhdlr"
    for (pid, proc) in s.processes().iter() {

        if proc.name() == "domainhdlr" {
            found = true;

            // Si encontramos el proceso, mostramos la información
            println!("Proceso 'domainhdlr' encontrado:");
            println!("PID: {}", pid);
            println!("Nombre: {}", proc.name().to_str().unwrap_or(""));

            let status = if proc.exists() {
                "Corriendo"
            } else {
                "Detenido"
            };

            println!("Estado: {}", status);
            println!("Uso de CPU: {:.2}%", proc.cpu_usage());
            println!("Uso de Memoria: {:.2} MB", proc.memory() as f64 / 1024.0);
            println!("------------------------");
        }
    }

    if !found {
        println!("No se encontró el proceso 'domainhdlr'");
    }

    println!("=========================");
    Ok(())
}


#[allow(unused)]
pub async fn start() ->  Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_loop().await;
    println!("Service started in background.");
    Ok(())
}
