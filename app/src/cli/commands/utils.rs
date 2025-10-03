use anyhow::Result;
use dialoguer::Confirm;
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    storage::{VaultStorage, AuditLogger, AuditEntry},
    config::Config,
    cli::output,
    auth::SessionManager,
};

pub async fn status_command(config: &Config, storage: &VaultStorage) -> Result<()> {
    println!("{} Vault Status", "ℹ".blue());
    println!("Storage: {}", config.storage_path);
    
    if let Some(tenant) = &config.tenant_id {
        println!("Tenant: {}", tenant.cyan());
    }
    
    // Cloud configuration
    if let Some(cloud_config) = &config.cloud {
        match cloud_config.mode {
            crate::config::CloudMode::None => {
                println!("Cloud mode: {}", "None (fully local)".yellow());
            }
            crate::config::CloudMode::Backup => {
                println!("Cloud mode: {}", "Backup".blue());
                if let Some(backend) = &cloud_config.backend {
                    println!("Backend: {:?}", backend);
                }
            }
            crate::config::CloudMode::Collaborative => {
                println!("Cloud mode: {}", "Collaborative".purple());
                if let Some(backend) = &cloud_config.backend {
                    println!("Backend: {:?}", backend);
                }
            }
        }
    } else {
        println!("Cloud mode: {}", "Not configured".yellow());
    }
    
    let stats = storage.get_stats().await?;
    println!("Secrets: {}", stats.secret_count);
    println!("Namespaces: {}", stats.namespace_count);
    println!("Tenants: {}", stats.tenant_count);
    
    // Session information
    match SessionManager::get_current_session() {
        Ok(session) => {
            println!("Status: {}", "Logged in".green());
            println!("User: {}", session.user_id.cyan());
            println!("Role: {:?}", session.role);
            
            let time_left = session.time_until_expiry();
            let hours = time_left.num_hours();
            let minutes = time_left.num_minutes() % 60;
            
            if hours > 0 {
                println!("Session expires in: {}h {}m", hours, minutes);
            } else if minutes > 0 {
                println!("Session expires in: {}m", minutes);
            } else {
                println!("Session: {}", "Expired".red());
            }
        }
        Err(_) => {
            println!("Status: {}", "Logged out".yellow());
        }
    }
    
    Ok(())
}

pub async fn doctor_command(config: &Config, storage: &VaultStorage) -> Result<()> {
    println!("{} Running diagnostics...", "🔍".cyan());
    
    println!("Checking storage...");
    match storage.health_check().await {
        Ok(_) => println!("  {} Storage is healthy", "✓".green()),
        Err(e) => println!("  {} Storage error: {}", "✗".red(), e),
    }
    
    println!("Checking configuration...");
    if std::path::Path::new(&config.storage_path).parent().unwrap().exists() {
        println!("  {} Storage directory exists", "✓".green());
    } else {
        println!("  {} Storage directory missing", "✗".red());
    }
    
    // Test basic operations
    println!("Testing basic operations...");
    let test_secrets = storage.list("default").await?;
    println!("  {} Found {} secrets in default namespace", "ℹ".blue(), test_secrets.len());
    
    println!("{} Diagnostics complete", "✓".green());
    Ok(())
}

pub async fn export_command(
    storage: &VaultStorage,
    output: &str,
    format: &str,
    namespace: Option<&str>,
) -> Result<()> {
    if let Ok(session) = SessionManager::get_current_session() {
        if !session.role.can_read() {
            output::print_error("Read permissions required for export");
            return Ok(());
        }
    } else {
        output::print_error("Please login first");
        return Ok(());
    }
    
    let ns = namespace.unwrap_or("default");
    
    let confirm_msg = if namespace.is_some() {
        format!("Export all secrets from namespace '{}'?", ns)
    } else {
        "Export all secrets from default namespace?".to_string()
    };
    
    if !Confirm::new()
        .with_prompt(confirm_msg)
        .default(false)
        .interact()?
    {
        output::print_info("Export cancelled");
        return Ok(());
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Exporting secrets...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let secrets = storage.list_with_metadata(ns, None).await?;
    
    let mut export_data = serde_json::Map::new();
    let mut exported_count = 0;
    
    for (key, metadata) in secrets {
        if let Some((value, _)) = storage.get_with_metadata(&key, ns).await? {
            let secret_data = serde_json::json!({
                "id": metadata.id,
                "value": value,
                "namespace": metadata.namespace,
                "created_at": metadata.created_at,
                "updated_at": metadata.updated_at,
                "created_by": metadata.created_by,
                "version": metadata.version,
                "tags": metadata.tags
            });
            export_data.insert(key, secret_data);
            exported_count += 1;
        }
    }
    
    let export_metadata = serde_json::json!({
        "export_info": {
            "exported_at": chrono::Utc::now(),
            "exported_by": SessionManager::get_current_session().map(|s| s.user_id).unwrap_or_default(),
            "namespace": ns,
            "format": format,
            "vault_version": env!("CARGO_PKG_VERSION"),
            "secret_count": exported_count
        },
        "secrets": export_data
    });
    
    let content = match format {
        "json" => serde_json::to_string_pretty(&export_metadata)?,
        "yaml" => {
            format!(
                "# Vault Export\n# Exported at: {}\n# Namespace: {}\n# Count: {}\n\n{}",
                chrono::Utc::now(),
                ns,
                exported_count,
                serde_json::to_string_pretty(&export_metadata)?
            )
        }
        _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use 'json' or 'yaml'", format)),
    };
    
    std::fs::write(output, content)?;
    
    if let Ok(session) = SessionManager::get_current_session() {
        let audit_entry = AuditEntry::new(
            session.tenant_id,
            AuditLogger::EVENT_EXPORT.to_string(),
            format!("Exported {} secrets from namespace {} to {}", exported_count, ns, output),
            session.user_id,
        );
        let _ = AuditLogger::log_event(&audit_entry);
    }
    
    pb.finish_with_message(format!("{} Export completed", "✓".green()));
    output::print_success(&format!("Exported {} secrets to {}", exported_count, output));
    
    Ok(())
}

pub async fn import_command(
    storage: &VaultStorage,
    input: &str,
    format: &str,
    namespace: Option<&str>,
) -> Result<()> {
    if let Ok(session) = SessionManager::get_current_session() {
        if !session.role.can_write() {
            output::print_error("Write permissions required for import");
            return Ok(());
        }
    } else {
        output::print_error("Please login first");
        return Ok(());
    }
    
    if !std::path::Path::new(input).exists() {
        output::print_error(&format!("Import file not found: {}", input));
        return Ok(());
    }
    
    let content = std::fs::read_to_string(input)?;
    let ns = namespace.unwrap_or("default");
    
    let import_data: serde_json::Value = match format {
        "json" => serde_json::from_str(&content)?,
        "yaml" => {
            return Err(anyhow::anyhow!("YAML import not fully supported yet. Please use JSON format."));
        }
        _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use 'json' or 'yaml'", format)),
    };
    
    let secrets_data = if let Some(secrets) = import_data.get("secrets") {
        secrets.as_object().unwrap_or(&serde_json::Map::new()).clone()
    } else {
        import_data.as_object().unwrap_or(&serde_json::Map::new()).clone()
    };
    
    if secrets_data.is_empty() {
        output::print_warning("No secrets found in import file");
        return Ok(());
    }
    
    println!("{} Import Preview", "📊".cyan());
    println!("File: {}", input);
    println!("Target namespace: {}", ns.cyan());
    println!("Secrets to import: {}", secrets_data.len());
    
    if !Confirm::new()
        .with_prompt("Proceed with import?")
        .default(false)
        .interact()?
    {
        output::print_info("Import cancelled");
        return Ok(());
    }
    
    let pb = ProgressBar::new(secrets_data.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    
    let mut imported = 0;
    let mut errors = Vec::new();
    
    for (key, data) in secrets_data {
        pb.set_message(format!("Importing {}", key));
        
        match data.get("value").and_then(|v| v.as_str()) {
            Some(value) => {
                let tags: Vec<String> = data.get("tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_else(Vec::new);
                
                match storage.put_with_tags(&key, value, ns, &tags).await {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        errors.push(format!("Failed to import {}: {}", key, e));
                    }
                }
            }
            None => {
                errors.push(format!("No value found for secret: {}", key));
            }
        }
        
        pb.inc(1);
    }
    
    pb.finish_with_message("Import completed");
    
    if let Ok(session) = SessionManager::get_current_session() {
        let audit_entry = AuditEntry::new(
            session.tenant_id,
            AuditLogger::EVENT_IMPORT.to_string(),
            format!("Imported {} secrets from {} to namespace {}", imported, input, ns),
            session.user_id,
        );
        let _ = AuditLogger::log_event(&audit_entry);
    }
    
    println!("\n{} Import Results", "📊".green());
    output::print_success(&format!("Successfully imported: {}", imported));
    
    if !errors.is_empty() {
        output::print_error(&format!("Errors: {}", errors.len()));
        for error in &errors {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}

pub async fn completions_command(shell: &str) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell};
    use std::io;
    
    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            println!("{} Unsupported shell: {}. Supported: bash, zsh, fish, powershell", "✗".red(), shell);
            return Ok(());
        }
    };
    
    let mut cmd = crate::cli::VaultCli::command();
    generate(shell, &mut cmd, "vault", &mut io::stdout());
    
    Ok(())
}