use clap::Parser;
use std::process;

mod cli;
mod crypto;
mod storage;
mod sync;
mod auth;
mod config;

use cli::VaultCli;

#[tokio::main]
async fn main() {
    let cli = VaultCli::parse();
    
    if let Err(e) = cli.run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}