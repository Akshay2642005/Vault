use clap::{Parser, Subcommand};
// use owo_colors::OwoColorize; // TODO: Use for colored output
use anyhow::Result;

mod commands;
mod output;

use crate::{storage::VaultStorage, config::Config};
use commands::*;

#[derive(Parser)]
#[command(name = "vault")]
#[command(about = "A local-first, multi-tenant password manager")]
#[command(version)]
pub struct VaultCli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(long, global = true, help = "Enable verbose output")]
    pub verbose: bool,
    
    #[arg(long, global = true, help = "Configuration file path")]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        #[arg(long, help = "Tenant identifier")]
        tenant: String,
        #[arg(long, help = "Admin email address")]
        admin: String,
        #[arg(long, help = "Force initialization even if vault exists")]
        force: bool,
    },
    
    /// Login to a tenant
    Login {
        #[arg(long, help = "Tenant identifier")]
        tenant: String,
        #[arg(long, help = "User email (for collaborative mode)")]
        email: Option<String>,
        #[arg(long, help = "Remember session for longer")]
        remember: bool,
    },
    
    /// Logout from current session
    Logout,
    
    /// Show current user info
    Whoami,
    
    /// Store a secret
    Put {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(long, help = "Namespace for the secret")]
        namespace: Option<String>,
        #[arg(long, help = "Secret value (will prompt if not provided)")]
        value: Option<String>,
        #[arg(long, help = "Tags for the secret")]
        tags: Vec<String>,
        #[arg(long, help = "Force overwrite existing secret")]
        force: bool,
    },
    
    /// Retrieve a secret
    Get {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(long, help = "Namespace for the secret")]
        namespace: Option<String>,
        #[arg(long, help = "Copy to clipboard instead of printing")]
        copy: bool,
        #[arg(long, help = "Show secret metadata")]
        metadata: bool,
    },
    
    /// List secrets
    List {
        #[arg(long, help = "Namespace to list")]
        namespace: Option<String>,
        #[arg(long, help = "Filter by tag")]
        tag: Option<String>,
        #[arg(long, help = "Show detailed information")]
        detailed: bool,
    },
    
    /// Delete a secret
    Delete {
        #[arg(help = "Secret key")]
        key: String,
        #[arg(long, help = "Namespace for the secret")]
        namespace: Option<String>,
        #[arg(long, help = "Force deletion without confirmation")]
        force: bool,
    },
    
    /// Search secrets
    Search {
        #[arg(help = "Search query")]
        query: String,
        #[arg(long, help = "Namespace to search in")]
        namespace: Option<String>,
    },
    
    /// Sync with cloud
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    
    /// Manage roles and permissions
    Roles {
        #[command(subcommand)]
        action: RoleAction,
    },
    
    /// Manage users (collaborative mode only)
    Users {
        #[command(subcommand)]
        action: UserAction,
    },
    
    /// Audit operations
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
    
    /// Export secrets
    Export {
        #[arg(long, help = "Output file path")]
        output: String,
        #[arg(long, help = "Export format", default_value = "json")]
        format: String,
        #[arg(long, help = "Namespace to export")]
        namespace: Option<String>,
    },
    
    /// Import secrets
    Import {
        #[arg(help = "Input file path")]
        input: String,
        #[arg(long, help = "Import format", default_value = "json")]
        format: String,
        #[arg(long, help = "Target namespace")]
        namespace: Option<String>,
    },
    
    /// Show vault status
    Status,
    
    /// Run diagnostics
    Doctor,
    
    /// Generate shell completions
    Completions {
        #[arg(help = "Shell type")]
        shell: String,
    },
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Push secrets to cloud
    Push {
        #[arg(long, help = "Force push even with conflicts")]
        force: bool,
    },
    /// Pull secrets from cloud
    Pull {
        #[arg(long, help = "Force pull and overwrite local changes")]
        force: bool,
    },
    /// Enable automatic sync
    Auto {
        #[arg(long, help = "Sync interval in minutes")]
        interval: Option<u64>,
    },
    /// Show sync status
    Status,
    /// Configure sync backend
    Configure,
}

#[derive(Subcommand)]
pub enum RoleAction {
    /// Add user to tenant
    Add {
        #[arg(long)]
        tenant: String,
        #[arg(long)]
        user: String,
        #[arg(long)]
        role: String,
    },
    /// Remove user from tenant
    Remove {
        #[arg(long)]
        tenant: String,
        #[arg(long)]
        user: String,
    },
    /// List users in tenant
    List {
        #[arg(long)]
        tenant: String,
    },
}

#[derive(Subcommand)]
pub enum AuditAction {
    /// Show recent audit logs
    Tail {
        #[arg(long, help = "Number of entries to show")]
        lines: Option<usize>,
        #[arg(long, help = "Follow log updates")]
        follow: bool,
    },
    /// Search audit logs
    Search {
        #[arg(help = "Search query")]
        query: String,
        #[arg(long, help = "Start date")]
        since: Option<String>,
        #[arg(long, help = "End date")]
        until: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum UserAction {
    /// Invite user to tenant
    Invite {
        #[arg(long)]
        email: String,
        #[arg(long)]
        role: String,
    },
    /// List users in current tenant
    List,
    /// Remove user from tenant
    Remove {
        #[arg(long)]
        email: String,
    },
    /// Change user role
    ChangeRole {
        #[arg(long)]
        email: String,
        #[arg(long)]
        role: String,
    },
    /// Accept invitation
    Accept {
        #[arg(long)]
        token: String,
    },
}

impl VaultCli {
    pub async fn run(self) -> Result<()> {
        let config = Config::load(self.config.as_deref())?;
        let mut storage = VaultStorage::new(&config.storage_path)?;
        
        match self.command {
            Commands::Init { tenant, admin, force } => {
                init_command(&mut storage, &tenant, &admin, force).await
            }
            Commands::Login { tenant, email, remember } => {
                login_command(&mut storage, &config, &tenant, email.as_deref(), remember).await
            }
            Commands::Logout => {
                logout_command().await
            }
            Commands::Put { key, namespace, value, tags, force } => {
                put_command(&storage, &key, namespace.as_deref(), value.as_deref(), &tags, force).await
            }
            Commands::Get { key, namespace, copy, metadata } => {
                get_command(&storage, &key, namespace.as_deref(), copy, metadata).await
            }
            Commands::List { namespace, tag, detailed } => {
                list_command(&storage, namespace.as_deref(), tag.as_deref(), detailed).await
            }
            Commands::Delete { key, namespace, force } => {
                delete_command(&storage, &key, namespace.as_deref(), force).await
            }
            Commands::Search { query, namespace } => {
                search_command(&storage, &query, namespace.as_deref()).await
            }
            Commands::Status => {
                status_command(&config, &storage).await
            }
            Commands::Whoami => {
                whoami_command().await
            }
            Commands::Doctor => {
                doctor_command(&config, &storage).await
            }
            Commands::Sync { action } => {
                sync_command(action, &config).await
            }
            Commands::Roles { action } => {
                roles_command(action).await
            }
            Commands::Audit { action } => {
                audit_command(action).await
            }
            Commands::Users { action } => {
                users_command(action, &storage, &config).await
            }
            Commands::Export { output, format, namespace } => {
                export_command(&storage, &output, &format, namespace.as_deref()).await
            }
            Commands::Import { input, format, namespace } => {
                import_command(&storage, &input, &format, namespace.as_deref()).await
            }
            Commands::Completions { shell } => {
                completions_command(&shell).await
            }
        }
    }
}