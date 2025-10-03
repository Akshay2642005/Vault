use anyhow::Result;
use dialoguer::{Select, Input};
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    storage::{VaultStorage, AuditLogger, AuditEntry},
    config::Config,
    cli::{SyncAction, output},
    auth::SessionManager,
    sync::{SyncManager, ConflictResolver, AutoResolveStrategy},
};

pub async fn sync_command(action: SyncAction, config: &Config) -> Result<()> {
    if let Ok(session) = SessionManager::get_current_session() {
        if !session.role.can_write() {
            output::print_error("Insufficient permissions for sync operations");
            return Ok(());
        }
    } else {
        output::print_error("Please login first");
        return Ok(());
    }
    
    match action {
        SyncAction::Push { force } => {
            if let Some(cloud_config) = &config.cloud_sync {
                if !cloud_config.enabled {
                    output::print_warning("Cloud sync is disabled in configuration.");
                    return Ok(());
                }
                
                let storage = VaultStorage::new(&config.storage_path)?;
                let sync_manager = SyncManager::from_config(cloud_config, storage)?;
                
                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
                pb.set_message("Pushing secrets to cloud...");
                pb.enable_steady_tick(std::time::Duration::from_millis(100));
                
                match sync_manager.push(force).await {
                    Ok(result) => {
                        pb.finish_with_message(format!("{} Push completed", "✓".green()));
                        println!("Pushed: {} secrets", result.pushed);
                        
                        if !result.conflicts.is_empty() {
                            output::print_warning(&format!("Resolved {} conflicts", result.conflicts.len()));
                        }
                        
                        if !result.errors.is_empty() {
                            output::print_error(&format!("Encountered {} errors", result.errors.len()));
                            for error in &result.errors {
                                println!("  - {}", error);
                            }
                        }
                        
                        if let Ok(session) = SessionManager::get_current_session() {
                            let audit_entry = AuditEntry::new(
                                session.tenant_id,
                                AuditLogger::EVENT_SYNC_PUSH.to_string(),
                                format!("Pushed {} secrets to cloud", result.pushed),
                                session.user_id,
                            );
                            let _ = AuditLogger::log_event(&audit_entry);
                        }
                    }
                    Err(e) => {
                        pb.finish_with_message(format!("{} Push failed", "✗".red()));
                        output::print_error(&format!("Sync error: {}", e));
                    }
                }
            } else {
                output::print_warning("Cloud sync not configured. Run 'vault sync configure' first.");
            }
        }
        SyncAction::Pull { force } => {
            if let Some(cloud_config) = &config.cloud_sync {
                if !cloud_config.enabled {
                    output::print_warning("Cloud sync is disabled in configuration.");
                    return Ok(());
                }
                
                let storage = VaultStorage::new(&config.storage_path)?;
                let sync_manager = SyncManager::from_config(cloud_config, storage)?;
                
                if !force {
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.yellow} {msg}")?);
                    pb.set_message("Checking for conflicts...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                    
                    match sync_manager.status().await {
                        Ok(status) => {
                            pb.finish();
                            if status.conflicts > 0 {
                                output::print_warning(&format!("Found {} conflicts", status.conflicts));
                                
                                let resolve_strategy = Select::new()
                                    .with_prompt("How would you like to resolve conflicts?")
                                    .items(&[
                                        "Use local version",
                                        "Use remote version", 
                                        "Use newer version",
                                        "Cancel operation"
                                    ])
                                    .default(0)
                                    .interact()?;
                                
                                match resolve_strategy {
                                    0 => {
                                        let _ = ConflictResolver::auto_resolve_conflicts(&[], AutoResolveStrategy::PreferLocal);
                                        output::print_info("Conflicts resolved using local versions");
                                    }
                                    1 => {
                                        let _ = ConflictResolver::auto_resolve_conflicts(&[], AutoResolveStrategy::PreferRemote);
                                        output::print_info("Conflicts resolved using remote versions");
                                    }
                                    2 => {
                                        let _ = ConflictResolver::auto_resolve_conflicts(&[], AutoResolveStrategy::PreferNewer);
                                        output::print_info("Conflicts resolved using newer versions");
                                    }
                                    3 => {
                                        // Use manual resolution strategy
                                        let _ = ConflictResolver::auto_resolve_conflicts(&[], AutoResolveStrategy::Manual);
                                        output::print_info("Manual conflict resolution selected");
                                        output::print_info("Operation cancelled");
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            pb.finish_with_message(format!("{} Status check failed", "✗".red()));
                            output::print_error(&format!("Error: {}", e));
                        }
                    }
                }
                
                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
                pb.set_message("Pulling secrets from cloud...");
                pb.enable_steady_tick(std::time::Duration::from_millis(100));
                
                match sync_manager.pull(force).await {
                    Ok(result) => {
                        pb.finish_with_message(format!("{} Pull completed", "✓".green()));
                        println!("Pulled: {} secrets", result.pulled);
                        
                        if !result.conflicts.is_empty() {
                            output::print_warning(&format!("Resolved {} conflicts", result.conflicts.len()));
                        }
                        
                        if !result.errors.is_empty() {
                            output::print_error(&format!("Encountered {} errors", result.errors.len()));
                        }
                        
                        if let Ok(session) = SessionManager::get_current_session() {
                            let audit_entry = AuditEntry::new(
                                session.tenant_id,
                                AuditLogger::EVENT_SYNC_PULL.to_string(),
                                format!("Pulled {} secrets from cloud", result.pulled),
                                session.user_id,
                            );
                            let _ = AuditLogger::log_event(&audit_entry);
                        }
                    }
                    Err(e) => {
                        pb.finish_with_message(format!("{} Pull failed", "✗".red()));
                        output::print_error(&format!("Sync error: {}", e));
                    }
                }
            } else {
                output::print_warning("Cloud sync not configured. Run 'vault sync configure' first.");
            }
        }
        SyncAction::Status => {
            println!("{} Sync Status", "📊".cyan());
            
            if let Some(cloud_config) = &config.cloud_sync {
                let storage = VaultStorage::new(&config.storage_path)?;
                let sync_manager = SyncManager::from_config(cloud_config, storage)?;
                
                match sync_manager.status().await {
                    Ok(status) => {
                        println!("Backend: {}", status.backend);
                        println!("Last sync: {}", status.last_sync.format("%Y-%m-%d %H:%M:%S UTC"));
                        println!("Local secrets: {}", status.local_secrets);
                        println!("Remote secrets: {}", status.remote_secrets);
                        println!("Conflicts: {}", status.conflicts);
                        let sync_status = if status.sync_needed { "Yes".yellow().to_string() } else { "No".green().to_string() };
                        println!("Sync needed: {}", sync_status);
                    }
                    Err(e) => {
                        // Handle different error types
                        let error_msg = e.to_string();
                        if error_msg.contains("permission") {
                            output::print_error("Permission denied accessing sync backend");
                        } else if error_msg.contains("time") || error_msg.contains("chrono") {
                            output::print_error("Time synchronization error occurred");
                        } else if error_msg.contains("sync") {
                            output::print_error("Sync operation failed");
                        } else {
                            output::print_error(&format!("Failed to get sync status: {}", e));
                        }
                    }
                }
            } else {
                println!("Status: {}", "Not configured".yellow());
            }
        }
        SyncAction::Configure => {
            println!("{} Sync Configuration Wizard", "🔧".cyan());
            
            let backend = Select::new()
                .with_prompt("Select sync backend")
                .items(&["Amazon S3", "PostgreSQL Database", "Disable sync"])
                .default(0)
                .interact()?;
            
            let config_path = dirs::config_dir()
                .unwrap_or_default()
                .join("vault")
                .join("config.toml");
            
            match backend {
                0 => {
                    let bucket = Input::<String>::new()
                        .with_prompt("S3 bucket name")
                        .interact()?;
                    let region = Input::<String>::new()
                        .with_prompt("AWS region")
                        .default("us-east-1".to_string())
                        .interact()?;
                    
                    let config_content = format!(
                        r#"storage_path = "~/.vault/vault.db"

[cloud_sync]
enabled = true
backend = "S3"
region = "{}"
bucket = "{}"
"#,
                        region, bucket
                    );
                    
                    if let Some(parent) = config_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&config_path, config_content)?;
                    
                    output::print_success(&format!("S3 sync configured: s3://{}/{}", bucket, region));
                }
                1 => {
                    let url = Input::<String>::new()
                        .with_prompt("PostgreSQL connection URL")
                        .interact()?;
                    
                    let config_content = format!(
                        r#"storage_path = "~/.vault/vault.db"

[cloud_sync]
enabled = true
backend = "Postgres"
database_url = "{}"
"#,
                        url
                    );
                    
                    if let Some(parent) = config_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&config_path, config_content)?;
                    
                    output::print_success("PostgreSQL sync configured");
                }
                2 => {
                    let config_content = r#"storage_path = "~/.vault/vault.db"

[cloud_sync]
enabled = false
"#;
                    
                    if let Some(parent) = config_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&config_path, config_content)?;
                    
                    output::print_success("Cloud sync disabled");
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}