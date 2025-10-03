use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use crate::{storage::VaultStorage, config::Config};

#[derive(Parser)]
#[command(name = "vault")]
#[command(about = "A local-first, multi-tenant password manager")]
pub struct VaultCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        #[arg(long)]
        tenant: String,
        #[arg(long)]
        admin: String,
    },
    /// Login to a tenant
    Login {
        #[arg(long)]
        tenant: String,
    },
    /// Show current user info
    Whoami,
    /// Store a secret
    Put {
        key: String,
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long)]
        value: Option<String>,
    },
    /// Retrieve a secret
    Get {
        key: String,
        #[arg(long)]
        namespace: Option<String>,
    },
    /// List secrets
    List {
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Delete a secret
    Delete {
        key: String,
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Sync with cloud
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    /// Show vault status
    Status,
}

#[derive(Subcommand)]
pub enum SyncAction {
    Push,
    Pull,
    Auto,
    Status,
}

impl VaultCli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::load()?;
        let storage = VaultStorage::new(&config.storage_path)?;
        
        let mut storage = storage;
        
        match self.command {
            Commands::Init { tenant, admin } => {
                println!("{} Initializing vault for tenant: {}", "✓".green(), tenant.cyan());
                storage.init_tenant(&tenant, &admin).await?;
                println!("{} Vault initialized successfully", "✓".green());
            }
            Commands::Login { tenant } => {
                use dialoguer::Password;
                let passphrase = Password::new()
                    .with_prompt("Enter master passphrase")
                    .interact()?;
                storage.unlock(&tenant, &passphrase)?;
                println!("{} Logged in to tenant: {}", "✓".green(), tenant.cyan());
            }
            Commands::Put { key, namespace, value } => {
                let ns = namespace.unwrap_or_else(|| "default".to_string());
                let val = match value {
                    Some(v) => v,
                    None => {
                        use dialoguer::Password;
                        Password::new()
                            .with_prompt("Enter secret value")
                            .interact()?
                    }
                };
                storage.put(&key, &val, &ns).await?;
                println!("{} Secret stored: {}/{}", "✓".green(), ns.cyan(), key.cyan());
            }
            Commands::Get { key, namespace } => {
                let ns = namespace.unwrap_or_else(|| "default".to_string());
                match storage.get(&key, &ns).await? {
                    Some(value) => println!("{}", value),
                    None => println!("{} Secret not found: {}/{}", "✗".red(), ns, key),
                }
            }
            Commands::List { namespace } => {
                let ns = namespace.unwrap_or_else(|| "default".to_string());
                let secrets = storage.list(&ns).await?;
                if secrets.is_empty() {
                    println!("No secrets found in namespace: {}", ns.cyan());
                } else {
                    println!("Secrets in {}:", ns.cyan());
                    for secret in secrets {
                        println!("  {}", secret.cyan());
                    }
                }
            }
            Commands::Delete { key, namespace } => {
                let ns = namespace.unwrap_or_else(|| "default".to_string());
                storage.delete(&key, &ns).await?;
                println!("{} Secret deleted: {}/{}", "✓".green(), ns.cyan(), key.cyan());
            }
            Commands::Status => {
                println!("{} Vault Status", "ℹ".blue());
                println!("Storage: {}", config.storage_path);
                if let Some(tenant) = &config.tenant_id {
                    println!("Tenant: {}", tenant.cyan());
                }
            }
            Commands::Whoami => {
                println!("{} Current session info not implemented", "⚠".yellow());
            }
            Commands::Sync { action } => {
                match action {
                    SyncAction::Push => println!("{} Sync push not implemented", "⚠".yellow()),
                    SyncAction::Pull => println!("{} Sync pull not implemented", "⚠".yellow()),
                    SyncAction::Auto => println!("{} Auto sync not implemented", "⚠".yellow()),
                    SyncAction::Status => println!("{} Sync status not implemented", "⚠".yellow()),
                }
            }
        }
        
        Ok(())
    }
}