use clap::Parser;
use commands::{Cli, Commands};
use process::{
    domains::{add_domain, delete_domain, list_domains},
    logger::read_log_errors,
    ubuntu_service::{install_service, set_enable_on_boot, uninstall_service},
};
use service::{start, status, stop};
use std::thread;
use tokio::runtime::Runtime;
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
                //Ver esto
                thread::spawn(|| {
                    let rt = Runtime::new().expect("Failed to create Tokio runtime");
                    if let Err(e) = rt.block_on(start()) {
                        eprintln!("Error running detached service: {}", e);
                    }
                });
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
    }
}
