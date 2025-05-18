use clap::Parser;
use commands::{Cli, Commands};
use process::{domains::{add_domain, delete_domain, list_domains}, logger::read_log_errors, ubuntu_service::{install_service, set_enable_on_boot, uninstall_service}};
use service::{start, status, stop};

mod commands;
mod service;
mod models;
mod process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start=> {
            _ = start().await;
        }
        Commands::Install => {
            _ = install_service();
        }
        Commands::Uninstall => {
            _ = uninstall_service();
        }
        Commands::EnableOnBoot {activate } => {
            _ = set_enable_on_boot(activate);
        }
        Commands::Stop => {
            stop().unwrap();
        }
        Commands::Status => {
            status().unwrap();
        }
        Commands::Restart => {
            let _ = stop();
            let  _ = start();
        }
        Commands::AddDomain { name, token, activated, txt } => {
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
