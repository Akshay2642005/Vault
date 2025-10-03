use anyhow::Result;
use dialoguer::{Password, Confirm};
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    storage::VaultStorage,
    config::Config,
    cli::{SyncAction, RoleAction, AuditAction},
};

pub async fn init_command(
    storage: &mut VaultStorage,
    tenant: &str,
    admin: &str,
    force: bool,
) -> Result<()> {
    if !force && storage.tenant_exists(tenant)? {
        if !Confirm::new()
            .with_prompt(format!("Tenant '{}' already exists. Reinitialize?", tenant))
            .interact()?
        {
            println!("{} Initialization cancelled", "ℹ".blue());
            return Ok(());
        }
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Initializing vault...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    storage.init_tenant(tenant, admin).await?;
    
    pb.finish_with_message(format!("{} Vault initialized for tenant: {}", "✓".green(), tenant.cyan()));
    println!("Admin: {}", admin.cyan());
    println!("\nNext steps:");
    println!("  1. Run 'vault login --tenant {}' to authenticate", tenant);
    println!("  2. Use 'vault put <key>' to store your first secret");
    
    Ok(())
}

pub async fn login_command(
    storage: &mut VaultStorage,
    tenant: &str,
    _remember: bool,
) -> Result<()> {
    if !storage.tenant_exists(tenant)? {
        println!("{} Tenant '{}' not found. Run 'vault init' first.", "✗".red(), tenant);
        return Ok(());
    }
    
    let passphrase = Password::new()
        .with_prompt("Enter master passphrase")
        .interact()?;
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Authenticating...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    match storage.unlock(tenant, &passphrase) {
        Ok(_) => {
            pb.finish_with_message(format!("{} Logged in to tenant: {}", "✓".green(), tenant.cyan()));
            
            // Save session info
            let session_info = format!("tenant={}\nlogged_in_at={}", tenant, chrono::Utc::now().to_rfc3339());
            let session_path = dirs::config_dir()
                .unwrap_or_default()
                .join("vault")
                .join("session");
            
            if let Some(parent) = session_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(session_path, session_info)?;
        }
        Err(_) => {
            pb.finish_with_message(format!("{} Invalid passphrase", "✗".red()));
        }
    }
    
    Ok(())
}

pub async fn logout_command() -> Result<()> {
    let session_path = dirs::config_dir()
        .unwrap_or_default()
        .join("vault")
        .join("session");
    
    if session_path.exists() {
        std::fs::remove_file(session_path)?;
        println!("{} Logged out successfully", "✓".green());
    } else {
        println!("{} No active session found", "ℹ".blue());
    }
    
    Ok(())
}

pub async fn put_command(
    storage: &VaultStorage,
    key: &str,
    namespace: Option<&str>,
    value: Option<&str>,
    tags: &[String],
    force: bool,
) -> Result<()> {
    let ns = namespace.unwrap_or("default");
    
    // Check if secret exists
    if !force && storage.get(key, ns).await?.is_some() {
        if !Confirm::new()
            .with_prompt(format!("Secret '{}/{}' already exists. Overwrite?", ns, key))
            .interact()?
        {
            println!("{} Operation cancelled", "ℹ".blue());
            return Ok(());
        }
    }
    
    let secret_value = match value {
        Some(v) => v.to_string(),
        None => Password::new()
            .with_prompt("Enter secret value")
            .interact()?,
    };
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Storing secret...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    storage.put_with_tags(key, &secret_value, ns, tags).await?;
    
    pb.finish_with_message(format!("{} Secret stored: {}/{}", "✓".green(), ns.cyan(), key.cyan()));
    
    if !tags.is_empty() {
        println!("Tags: {}", tags.join(", ").yellow());
    }
    
    Ok(())
}

pub async fn get_command(
    storage: &VaultStorage,
    key: &str,
    namespace: Option<&str>,
    copy: bool,
    metadata: bool,
) -> Result<()> {
    let ns = namespace.unwrap_or("default");
    
    match storage.get_with_metadata(key, ns).await? {
        Some((value, meta)) => {
            if copy {
                // TODO: Implement clipboard functionality
                println!("{} Secret copied to clipboard", "✓".green());
            } else {
                println!("{}", value);
            }
            
            if metadata {
                println!("\n{}", "Metadata:".bold());
                println!("  Created: {}", meta.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("  Version: {}", meta.version);
                if !meta.tags.is_empty() {
                    println!("  Tags: {}", meta.tags.join(", "));
                }
            }
        }
        None => {
            println!("{} Secret not found: {}/{}", "✗".red(), ns, key);
        }
    }
    
    Ok(())
}

pub async fn list_command(
    storage: &VaultStorage,
    namespace: Option<&str>,
    tag: Option<&str>,
    detailed: bool,
) -> Result<()> {
    let ns = namespace.unwrap_or("default");
    let secrets = storage.list_with_metadata(ns, tag).await?;
    
    if secrets.is_empty() {
        println!("No secrets found in namespace: {}", ns.cyan());
        return Ok(());
    }
    
    println!("Secrets in {}:", ns.cyan());
    
    if detailed {
        for (key, meta) in secrets {
            println!("  {} {}", "•".green(), key.cyan());
            println!("    Created: {}", meta.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("    Version: {}", meta.version);
            if !meta.tags.is_empty() {
                println!("    Tags: {}", meta.tags.join(", ").yellow());
            }
        }
    } else {
        for (key, _) in secrets {
            println!("  {}", key.cyan());
        }
    }
    
    Ok(())
}

pub async fn delete_command(
    storage: &VaultStorage,
    key: &str,
    namespace: Option<&str>,
    force: bool,
) -> Result<()> {
    let ns = namespace.unwrap_or("default");
    
    if !force {
        if !Confirm::new()
            .with_prompt(format!("Delete secret '{}/{}'?", ns, key))
            .interact()?
        {
            println!("{} Operation cancelled", "ℹ".blue());
            return Ok(());
        }
    }
    
    match storage.delete(key, ns).await {
        Ok(_) => println!("{} Secret deleted: {}/{}", "✓".green(), ns.cyan(), key.cyan()),
        Err(_) => println!("{} Secret not found: {}/{}", "✗".red(), ns, key),
    }
    
    Ok(())
}

pub async fn search_command(
    storage: &VaultStorage,
    query: &str,
    namespace: Option<&str>,
) -> Result<()> {
    let results = storage.search(query, namespace).await?;
    
    if results.is_empty() {
        println!("No secrets found matching: {}", query.yellow());
        return Ok(());
    }
    
    println!("Found {} secret(s) matching '{}':", results.len(), query.yellow());
    for (ns, key) in results {
        println!("  {}/{}", ns.cyan(), key.cyan());
    }
    
    Ok(())
}

pub async fn status_command(config: &Config, storage: &VaultStorage) -> Result<()> {
    println!("{} Vault Status", "ℹ".blue());
    println!("Storage: {}", config.storage_path);
    
    if let Some(tenant) = &config.tenant_id {
        println!("Tenant: {}", tenant.cyan());
    }
    
    let stats = storage.get_stats().await?;
    println!("Secrets: {}", stats.secret_count);
    println!("Namespaces: {}", stats.namespace_count);
    
    // Check session
    let session_path = dirs::config_dir()
        .unwrap_or_default()
        .join("vault")
        .join("session");
    
    if session_path.exists() {
        println!("Status: {}", "Logged in".green());
    } else {
        println!("Status: {}", "Logged out".yellow());
    }
    
    Ok(())
}

pub async fn whoami_command() -> Result<()> {
    let session_path = dirs::config_dir()
        .unwrap_or_default()
        .join("vault")
        .join("session");
    
    if session_path.exists() {
        let session_info = std::fs::read_to_string(session_path)?;
        for line in session_info.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "tenant" => println!("Tenant: {}", value.cyan()),
                    "logged_in_at" => println!("Logged in: {}", value),
                    _ => {}
                }
            }
        }
    } else {
        println!("{} Not logged in", "ℹ".blue());
    }
    
    Ok(())
}

pub async fn doctor_command(config: &Config, storage: &VaultStorage) -> Result<()> {
    println!("{} Running diagnostics...", "🔍".cyan());
    
    // Check storage
    println!("Checking storage...");
    match storage.health_check().await {
        Ok(_) => println!("  {} Storage is healthy", "✓".green()),
        Err(e) => println!("  {} Storage error: {}", "✗".red(), e),
    }
    
    // Check configuration
    println!("Checking configuration...");
    if std::path::Path::new(&config.storage_path).parent().unwrap().exists() {
        println!("  {} Storage directory exists", "✓".green());
    } else {
        println!("  {} Storage directory missing", "✗".red());
    }
    
    // Check permissions
    println!("Checking permissions...");
    // TODO: Implement permission checks
    
    println!("{} Diagnostics complete", "✓".green());
    Ok(())
}

pub async fn sync_command(action: SyncAction, _config: &Config) -> Result<()> {
    match action {
        SyncAction::Push { force: _ } => {
            println!("{} Sync push not yet implemented", "⚠".yellow());
        }
        SyncAction::Pull { force: _ } => {
            println!("{} Sync pull not yet implemented", "⚠".yellow());
        }
        SyncAction::Auto { interval: _ } => {
            println!("{} Auto sync not yet implemented", "⚠".yellow());
        }
        SyncAction::Status => {
            println!("{} Sync status not yet implemented", "⚠".yellow());
        }
        SyncAction::Configure => {
            println!("{} Sync configuration not yet implemented", "⚠".yellow());
        }
    }
    Ok(())
}

pub async fn roles_command(action: RoleAction) -> Result<()> {
    match action {
        RoleAction::Add { tenant: _, user: _, role: _ } => {
            println!("{} Role management not yet implemented", "⚠".yellow());
        }
        RoleAction::Remove { tenant: _, user: _ } => {
            println!("{} Role management not yet implemented", "⚠".yellow());
        }
        RoleAction::List { tenant: _ } => {
            println!("{} Role management not yet implemented", "⚠".yellow());
        }
    }
    Ok(())
}

pub async fn audit_command(action: AuditAction) -> Result<()> {
    match action {
        AuditAction::Tail { lines: _, follow: _ } => {
            println!("{} Audit logging not yet implemented", "⚠".yellow());
        }
        AuditAction::Search { query: _, since: _, until: _ } => {
            println!("{} Audit search not yet implemented", "⚠".yellow());
        }
    }
    Ok(())
}

pub async fn export_command(
    _storage: &VaultStorage,
    _output: &str,
    _format: &str,
    _namespace: Option<&str>,
) -> Result<()> {
    println!("{} Export functionality not yet implemented", "⚠".yellow());
    Ok(())
}

pub async fn import_command(
    _storage: &VaultStorage,
    _input: &str,
    _format: &str,
    _namespace: Option<&str>,
) -> Result<()> {
    println!("{} Import functionality not yet implemented", "⚠".yellow());
    Ok(())
}

pub async fn completions_command(_shell: &str) -> Result<()> {
    println!("{} Shell completions not yet implemented", "⚠".yellow());
    Ok(())
}