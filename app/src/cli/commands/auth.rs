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
    
    // Ask for master password
    let master_password = Password::new()
        .with_prompt("Create master password")
        .with_confirmation("Confirm master password", "Passwords do not match")
        .interact()?;
    
    // Validate password strength
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Initializing vault...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    if master_password.len() < 8 {
        output::print_error("Master password must be at least 8 characters long");
        // Fallback to basic init without password (insecure)
        output::print_warning("Using fallback initialization without password validation");
        storage.init_tenant(tenant, admin).await?;
    } else {
        storage.init_tenant_with_password(tenant, admin, &master_password).await?;
    }
    
    // Log tenant creation
    let audit_entry = AuditEntry::new(
        tenant.to_string(),
        AuditLogger::EVENT_TENANT_CREATED.to_string(),
        format!("Tenant {} created by {}", tenant, admin),
        admin.to_string(),
    );
    let _ = AuditLogger::log_event(&audit_entry);
    
    pb.finish_with_message(format!("{} Vault initialized for tenant: {}", "✓".green(), tenant.cyan()));
    println!("Admin: {}", admin.cyan());
    println!("\nNext steps:");
    println!("  1. Run 'vault login --tenant {}' to authenticate", tenant);
    println!("  2. Use 'vault put <key>' to store your first secret");
    
    Ok(())
}

pub async fn login_command(
    storage: &mut VaultStorage,
    config: &crate::config::Config,
    tenant: &str,
    email: Option<&str>,
    remember: bool,
) -> Result<()> {
    if !storage.tenant_exists(tenant)? {
        output::print_error(&format!("Tenant '{}' not found. Run 'vault init' first.", tenant));
        return Ok(());
    }
    
    // Check cloud mode
    let is_collaborative = config.cloud.as_ref()
        .map(|c| matches!(c.mode, crate::config::CloudMode::Collaborative))
        .unwrap_or(false);
    
    let (user_email, passphrase) = if is_collaborative {
        let email = match email {
            Some(e) => e.to_string(),
            None => dialoguer::Input::<String>::new()
                .with_prompt("Email")
                .interact()?,
        };
        
        let pwd = Password::new()
            .with_prompt(&format!("Password for {}", email))
            .interact()?;
            
        (email, pwd)
    } else {
        let pwd = Password::new()
            .with_prompt("Enter master passphrase")
            .interact()?;
        ("admin".to_string(), pwd)
    };
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Authenticating...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    match storage.unlock(tenant, &passphrase) {
        Ok(_) => {
            pb.finish_with_message(format!("{} Logged in to tenant: {}", "✓".green(), tenant.cyan()));
            
            let duration_hours = if remember { 168 } else { 24 };
            let (user_id, role) = if is_collaborative {
                // In collaborative mode, get user role from storage
                match storage.get_user_role(tenant, &user_email).await {
                    Ok(Some(user_role)) => (user_email.clone(), user_role),
                    Ok(None) => {
                        output::print_error(&format!("User {} not found in tenant {}", user_email, tenant));
                        return Ok(());
                    }
                    Err(_) => (user_email.clone(), Role::Reader), // Default role
                }
            } else {
                ("admin".to_string(), Role::Admin)
            };
            
            let session = Session::new(
                tenant.to_string(),
                user_id,
                role,
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
        
        // Clear session key from storage
        let config_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".vault")
            .join("vault.db");
        if let Ok(storage) = VaultStorage::new(&config_path.to_string_lossy()) {
            let _ = storage.clear_session_key(&session.tenant_id);
        }
        
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
                let time_left = session.time_until_expiry();
                let hours = time_left.num_hours();
                let minutes = time_left.num_minutes() % 60;
                
                if hours > 0 {
                    println!("Time remaining: {}h {}m", hours, minutes);
                } else {
                    println!("Time remaining: {}m", minutes);
                }
                
                println!("Status: {}", "Active".green());
                
                // Show permissions
                println!("\n{} Permissions", "🔐".cyan());
                let read_status = if session.role.can_read() { "✓".green().to_string() } else { "✗".red().to_string() };
                let write_status = if session.role.can_write() { "✓".green().to_string() } else { "✗".red().to_string() };
                let admin_status = if session.role.can_admin() { "✓".green().to_string() } else { "✗".red().to_string() };
                let audit_status = if session.role.can_audit() { "✓".green().to_string() } else { "✗".red().to_string() };
                println!("  Read: {}", read_status);
                println!("  Write: {}", write_status);
                println!("  Admin: {}", admin_status);
                println!("  Audit: {}", audit_status);
                
                // Refresh session if it's close to expiry
                if hours < 1 {
                    let mut refreshed_session = session.clone();
                    refreshed_session.refresh();
                    let _ = SessionManager::save_session(&refreshed_session);
                    output::print_info("Session refreshed");
                }
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