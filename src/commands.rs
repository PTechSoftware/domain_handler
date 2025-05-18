use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "domainhdlr", about = "Cli service to handle domains on Duckdns", author = "PTechSoftware - Ignacio Perez")]
pub struct Cli {
    #[command(subcommand)]
    pub command : Commands
}


#[derive(Subcommand)]
pub enum Commands{
    #[command(name="install" , alias="i")]
    Install,
    #[command(name="uninstall", alias="u")]
    Uninstall,
    #[command(name="start", alias="s")]
    Start,
    #[command(name="stop", alias="sp")]
    Stop,
    #[command(name="status", alias="st")]
    Status,
    #[command(name="restart", alias="rs")]
    Restart,
    #[command(name = "enable-on-boot", alias = "b")]
    EnableOnBoot {
        #[arg(long, short)] activate: bool,
    },
    #[command(name="view-log", alias="l")]
    ViewLog,
    #[command(name="add-domain", alias="ad")]
    AddDomain{
        #[arg(long, short)] name: String,
        #[arg(long, short)] token: String,
        #[arg(long, short)] activated: Option<bool>,
        #[arg(long, short)] txt: Option<String>,
    },
    #[command(name="delete-domain", alias="dd")]
    DeleteDomain {
        #[arg(long, short)] name: String,
    },
    #[command(name="list-domain", alias="ld")]
    ListDomain

}