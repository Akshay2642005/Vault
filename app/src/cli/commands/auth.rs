use anyhow::Result;
use dialoguer::{Password, Confirm};
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    storage::{VaultStorage, AuditLogger, AuditEntry},
    cli::output,
    auth::{SessionManager, Session, Role},
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
    remember: bool,
) -> Result<()> {
    if !storage.tenant_exists(tenant)? {
        output::print_error(&format!("Tenant '{}' not found. Run 'vault init' first.", tenant));
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
            
            let duration_hours = if remember { 168 } else { 24 };
            let session = Session::new(
                tenant.to_string(),
                "admin".to_string(),
                Role::Admin,
                duration_hours,
            );
            
            SessionManager::save_session(&session)?;
            
            let audit_entry = AuditEntry::new(
                tenant.to_string(),
                AuditLogger::EVENT_LOGIN.to_string(),
                format!("User logged in to tenant {}", tenant),
                "admin".to_string(),
            );
            let _ = AuditLogger::log_event(&audit_entry);
            
            output::print_success(&format!("Successfully logged in to tenant: {}", tenant));
        }
        Err(_) => {
            pb.finish_with_message(format!("{} Invalid passphrase", "✗".red()));
            output::print_error("Authentication failed");
        }
    }
    
    Ok(())
}

pub async fn logout_command() -> Result<()> {
    if let Ok(session) = SessionManager::get_current_session() {
        let audit_entry = AuditEntry::new(
            session.tenant_id.clone(),
            AuditLogger::EVENT_LOGOUT.to_string(),
            "User logged out".to_string(),
            session.user_id,
        );
        let _ = AuditLogger::log_event(&audit_entry);
        
        SessionManager::clear_session()?;
        output::print_success("Logged out successfully");
    } else {
        output::print_info("No active session found");
    }
    
    Ok(())
}

pub async fn whoami_command() -> Result<()> {
    match SessionManager::get_current_session() {
        Ok(session) => {
            println!("{} Current Session", "👤".cyan());
            println!("Tenant: {}", session.tenant_id.cyan());
            println!("User: {}", session.user_id.cyan());
            println!("Role: {:?}", session.role);
            println!("Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Expires: {}", session.expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if session.is_valid() {
                println!("Status: {}", "Active".green());
            } else {
                println!("Status: {}", "Expired".red());
                output::print_warning("Session has expired. Please login again.");
            }
        }
        Err(_) => {
            output::print_info("Not logged in");
            println!("Run 'vault login --tenant <tenant>' to authenticate");
        }
    }
    
    Ok(())
}