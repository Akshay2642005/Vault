use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod crypto;
mod storage;
mod auth;
mod sync;
mod error;

use cli::VaultCli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    let cli = VaultCli::parse();
    
    match cli.run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}