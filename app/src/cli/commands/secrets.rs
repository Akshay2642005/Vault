use anyhow::Result;
use dialoguer::{Password, Confirm, Input, Select};
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    storage::{VaultStorage, AuditLogger, AuditEntry, SecretGenerator},
    cli::output,
    auth::SessionManager,
};

pub async fn put_command(
    storage: &VaultStorage,
    key: &str,
    namespace: Option<&str>,
    value: Option<&str>,
    tags: &[String],
    force: bool,
) -> Result<()> {
    let ns = namespace.unwrap_or("default");
    
    if !force && storage.get(key, ns).await?.is_some() {
        if !Confirm::new()
            .with_prompt(format!("Secret '{}/{}' already exists. Overwrite?", ns, key))
            .interact()?
        {
            output::print_info("Operation cancelled");
            return Ok(());
        }
    }
    
    let secret_value = match value {
        Some(v) => v.to_string(),
        None => {
            let secret_type_choice = Select::new()
                .with_prompt("What type of secret?")
                .items(&[
                    "Simple Password",
                    "API Key", 
                    "Database Credentials",
                    "SSH Key Pair",
                    "Custom Text",
                    "UUID",
                    "Hex Key"
                ])
                .default(0)
                .interact()?;
            
            match secret_type_choice {
                0 => {
                    let generate = Confirm::new()
                        .with_prompt("Generate secure password?")
                        .default(true)
                        .interact()?;
                    
                    if generate {
                        let length = Input::<usize>::new()
                            .with_prompt("Password length")
                            .default(32)
                            .interact()?;
                        
                        let include_symbols = Confirm::new()
                            .with_prompt("Include symbols?")
                            .default(true)
                            .interact()?;
                        
                        SecretGenerator::generate_password(length, include_symbols)
                    } else {
                        Password::new()
                            .with_prompt("Enter password")
                            .interact()?
                    }
                }
                1 => {
                    let prefix = Input::<String>::new()
                        .with_prompt("API key prefix (optional)")
                        .allow_empty(true)
                        .interact()?;
                    
                    let prefix_opt = if prefix.is_empty() { None } else { Some(prefix.as_str()) };
                    SecretGenerator::generate_api_key(prefix_opt)
                }
                2 => {
                    let db_type = Select::new()
                        .with_prompt("Database type")
                        .items(&["postgres", "mysql", "redis", "mongodb"])
                        .default(0)
                        .interact()?;
                    
                    let db_name = ["postgres", "mysql", "redis", "mongodb"][db_type];
                    let creds = SecretGenerator::generate_database_credentials(db_name);
                    serde_json::to_string_pretty(&creds).unwrap_or_default()
                }
                3 => {
                    let (private_key, public_key) = SecretGenerator::generate_ssh_key();
                    format!("Private Key:\n{}\n\nPublic Key:\n{}", private_key, public_key)
                }
                4 => {
                    Input::<String>::new()
                        .with_prompt("Enter custom text")
                        .interact()?
                }
                5 => SecretGenerator::generate_uuid(),
                6 => {
                    let length = Input::<usize>::new()
                        .with_prompt("Hex key length")
                        .default(32)
                        .interact()?;
                    
                    SecretGenerator::generate_hex_key(length)
                }
                _ => {
                    Password::new()
                        .with_prompt("Enter secret value")
                        .interact()?
                }
            }
        }
    };
    
    let password_protected = Confirm::new()
        .with_prompt("Password protect this secret?")
        .default(false)
        .interact()?;
    
    let access_password = if password_protected {
        Some(Password::new()
            .with_prompt("Enter access password for this secret")
            .with_confirmation("Confirm access password", "Passwords do not match")
            .interact()?)
    } else {
        None
    };
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Storing secret...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    if tags.is_empty() {
        storage.put_with_protection(key, &secret_value, ns, &[], access_password.as_deref()).await?;
    } else {
        storage.put_with_protection(key, &secret_value, ns, tags, access_password.as_deref()).await?;
    }
    
    if let Ok(session) = SessionManager::get_current_session() {
        let audit_entry = AuditEntry::new(
            session.tenant_id,
            AuditLogger::EVENT_SECRET_CREATED.to_string(),
            format!("Secret {}/{} created", ns, key),
            session.user_id,
        )
        .with_resource("secret".to_string(), format!("{}/{}", ns, key))
        .with_metadata(serde_json::json!({"namespace": ns, "key": key, "tags": tags}));
        let _ = AuditLogger::log_event(&audit_entry);
    }
    
    pb.finish_with_message(format!("{} Secret stored: {}/{}", "✓".green(), ns.cyan(), key.cyan()));
    
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
    
    // Check if secret is password protected
    let is_protected = storage.is_secret_password_protected(key, ns).await?;
    let access_password = if is_protected {
        Some(Password::new()
            .with_prompt("Enter access password for this secret")
            .interact()?)
    } else {
        None
    };
    
    match storage.get_with_metadata_and_password(key, ns, access_password.as_deref()).await? {
        Some((value, meta)) => {
            if copy {
                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    let mut child = Command::new("cmd")
                        .args(["/C", "echo", &value, "|", "clip"])
                        .spawn()?;
                    child.wait()?;
                }
                
                output::print_success("Secret copied to clipboard");
            } else {
                println!("{}", value);
            }
            
            if metadata {
                println!("\n{}", "Metadata:".bold());
                println!("  ID: {}", meta.id);
                println!("  Created: {}", meta.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("  Updated: {}", meta.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("  Version: {}", meta.version);
                if !meta.tags.is_empty() {
                    println!("  Tags: {}", meta.tags.join(", ").yellow());
                }
            }
            
            if let Ok(session) = SessionManager::get_current_session() {
                let audit_entry = AuditEntry::new(
                    session.tenant_id,
                    AuditLogger::EVENT_SECRET_ACCESSED.to_string(),
                    format!("Secret {}/{} accessed", ns, key),
                    session.user_id,
                )
                .with_context(Some("127.0.0.1".to_string()), Some("vault-cli".to_string()));
                let _ = AuditLogger::log_event(&audit_entry);
            }
        }
        None => {
            output::print_error(&format!("Secret not found: {}/{}", ns, key));
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
        if let Some(tag_filter) = tag {
            output::print_info(&format!("No secrets found with tag '{}' in namespace: {}", tag_filter, ns));
        } else {
            output::print_info(&format!("No secrets found in namespace: {}", ns));
        }
        return Ok(());
    }
    
    if let Some(tag_filter) = tag {
        println!("Secrets with tag '{}' in {}:", tag_filter.yellow(), ns.cyan());
    } else {
        println!("Secrets in {}:", ns.cyan());
    }
    
    if detailed {
        output::print_table_header(&["Key", "Created", "Version", "Tags"]);
        output::print_secret_list(&secrets, true);
    } else {
        output::print_secret_list(&secrets, false);
    }
    
    println!("\n{} Total: {} secret(s)", "📊".cyan(), secrets.len());
    
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
        Ok(_) => {
            println!("{} Secret deleted: {}/{}", "✓".green(), ns.cyan(), key.cyan());
            
            if let Ok(session) = SessionManager::get_current_session() {
                let audit_entry = AuditEntry::new(
                    session.tenant_id,
                    AuditLogger::EVENT_SECRET_UPDATED.to_string(),
                    format!("Secret {}/{} updated (deleted)", ns, key),
                    session.user_id,
                );
                let _ = AuditLogger::log_event(&audit_entry);
            }
        }
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