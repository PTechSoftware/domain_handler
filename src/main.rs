use clap::Parser;
use commands::{Cli, Commands};
use process::{
    domains::{add_domain, delete_domain, list_domains},
    logger::read_log_errors,
    ubuntu_service::{install_service, set_enable_on_boot, uninstall_service},
};
use service::{start, status, stop};
mod commands;
mod models;
mod process;
mod service;

#[tokio::main]
async fn main() {
    if !std::path::Path::new(".env").exists() {
        let _ = std::fs::write(
            ".env",
            "SMTP_SERVER=\nSMTP_PORT=\nSMTP_SENDER=\nSMTP_PASSWORD=\nSMTP_RECIPIENT=\n",
        );
    }
    dotenvy::dotenv().ok();

    let mail_cfg = process::notifier::MailConfig {
        smtp_server: std::env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".into()),
        smtp_port: std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".into())
            .parse()
            .unwrap_or(587),
        sender: std::env::var("SMTP_SENDER").unwrap_or_else(|_| "mail@gmail.com".into()),
        password: std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set in .env"),
        recipient: std::env::var("SMTP_RECIPIENT").unwrap_or_else(|_| "mail@gmail.com".into()),
    };
    let cli = Cli::parse();
    match cli.command {
        Commands::Start { detached } => {
            if detached {
                println!("Starting supervisor in background...");
                let exe = std::env::current_exe().expect("Failed to get current executable path");
                match std::process::Command::new(exe)
                    .arg("run-supervisor")
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => println!("Supervisor started with PID {}", child.id()),
                    Err(e) => eprintln!("Failed to start supervisor: {}", e),
                }
            } else if let Err(e) = start().await {
                eprintln!("Error starting service: {}", e);
            }
        }
        Commands::Install => {
            _ = install_service();
        }
        Commands::Uninstall => {
            _ = uninstall_service();
        }
        Commands::EnableOnBoot { activate } => {
            _ = set_enable_on_boot(activate);
        }
        Commands::Stop => {
            stop(&mail_cfg).await.unwrap();
        }
        Commands::Status => {
            status().unwrap();
        }
        Commands::Restart => {
            stop(&mail_cfg).await.unwrap();
            let _ = start();
        }
        Commands::AddDomain {
            name,
            token,
            activated,
            txt,
        } => {
            add_domain(&name, &token, activated, txt);
        }
        Commands::DeleteDomain { name } => {
            delete_domain(&name);
        }
        Commands::ListDomain => {
            list_domains();
        }
        Commands::ViewLog => {
            let l = read_log_errors();
            match l {
                Ok(d) => {
                    for el in d {
                        println!("{}", el)
                    }
                }
                _ => {
                    println!("Failed retrive logs")
                }
            }
        }
        Commands::RunSupervisor => {
            let exe = std::env::current_exe().expect("Failed to get executable");
            let lock_path = process::file_lock::get_lock_path().unwrap();

            if let Some(parent) = lock_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&lock_path, "supervisor_starting");

            loop {
                let mut child = std::process::Command::new(&exe)
                    .arg("run-worker")
                    .spawn()
                    .expect("Failed to spawn worker");

                let _ = child.wait(); // Bloquea hasta que el worker muere

                if !lock_path.exists() {
                    break;
                }

                let _ = process::logger::entry_for_errorlog(
                    "[SUPERVISOR] Worker crashed. Restarting in 5s...",
                    true,
                );
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
        Commands::RunWorker => {
            if let Err(e) = start().await {
                eprintln!("Error in worker: {}", e);
            }
        }
    }
}
