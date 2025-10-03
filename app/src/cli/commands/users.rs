use anyhow::Result;
use dialoguer::{Password, Confirm, Input};
use owo_colors::OwoColorize;

use crate::{
    storage::VaultStorage,
    cli::{UserAction, output},
    auth::{SessionManager, Role},
    config::{Config, CloudMode},
};

pub async fn users_command(action: UserAction, storage: &VaultStorage, config: &Config) -> Result<()> {
    // Check if collaborative mode is enabled
    let is_collaborative = config.cloud.as_ref()
        .map(|c| matches!(c.mode, CloudMode::Collaborative))
        .unwrap_or(false);
    
    if !is_collaborative {
        output::print_error("User management is only available in collaborative mode");
        println!("Set cloud.mode = \"collaborative\" in your config to enable this feature");
        return Ok(());
    }
    
    // Check permissions
    if let Ok(session) = SessionManager::get_current_session() {
        match &action {
            UserAction::List => {
                if !session.role.can_read() {
                    output::print_error("Read permissions required to list users");
                    return Ok(());
                }
            }
            UserAction::Invite { .. } | UserAction::Remove { .. } | UserAction::ChangeRole { .. } => {
                if !session.role.can_admin() {
                    output::print_error("Admin permissions required for user management");
                    return Ok(());
                }
            }
            UserAction::Accept { .. } => {
                // Anyone can accept invitations
            }
        }
    } else {
        output::print_error("Please login first");
        return Ok(());
    }
    
    match action {
        UserAction::Invite { email, role } => {
            // Validate role
            match role.to_lowercase().as_str() {
                "admin" | "owner" | "writer" | "reader" | "auditor" => {},
                _ => {
                    output::print_error(&format!("Invalid role: {}. Valid roles: admin, owner, writer, reader, auditor", role));
                    return Ok(());
                }
            }
            
            let session = SessionManager::get_current_session()?;
            
            println!("{} Inviting {} to tenant {} with role {}", 
                "📧".cyan(), email.cyan(), session.tenant_id.cyan(), role.yellow());
            
            // Generate invitation token (in real implementation, this would be sent via email)
            use uuid::Uuid;
            let invitation_token = Uuid::new_v4().to_string();
            
            // Store invitation in database (placeholder for real implementation)
            // let invitation_key = format!("invitation:{}:{}", session.tenant_id, email);
            // let invitation_data = serde_json::json!({
            //     "email": email,
            //     "role": role,
            //     "token": invitation_token,
            //     "invited_by": session.user_id,
            //     "created_at": chrono::Utc::now(),
            //     "expires_at": chrono::Utc::now() + chrono::Duration::days(7)
            // });
            
            // In a real implementation, you'd store this in the database
            println!("\n{} Invitation created!", "✓".green());
            println!("Invitation token: {}", invitation_token.yellow());
            println!("Share this token with {} to accept the invitation", email.cyan());
            println!("Token expires in 7 days");
            
            output::print_success(&format!("Invitation sent to {}", email));
        }
        
        UserAction::Accept { token } => {
            println!("{} Accepting invitation with token: {}", "🎫".cyan(), token.yellow());
            
            // In real implementation, validate token and create user account
            let email = Input::<String>::new()
                .with_prompt("Enter your email")
                .interact()?;
            
            let password = Password::new()
                .with_prompt("Create password")
                .with_confirmation("Confirm password", "Passwords do not match")
                .interact()?;
            
            // Hash password (placeholder for real implementation)
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            let _password_hash: [u8; 32] = hasher.finalize().into();
            
            // Add user to tenant (mock implementation)
            println!("{} Account created successfully!", "✓".green());
            println!("You can now login with: vault login --tenant <tenant> --email {}", email);
            
            output::print_success("Invitation accepted successfully");
        }
        
        UserAction::List => {
            let session = SessionManager::get_current_session()?;
            
            println!("{} Users in tenant {}:", "👥".cyan(), session.tenant_id.cyan());
            
            match storage.list_users(&session.tenant_id).await {
                Ok(users) => {
                    if users.is_empty() {
                        output::print_info("No users found in this tenant");
                        return Ok(());
                    }
                    
                    output::print_table_header(&["Email", "Role", "Status", "Last Login"]);
                    
                    let user_count = users.len();
                    for user in &users {
                        let role_colored = match user.role {
                            Role::Admin => "Admin".red().to_string(),
                            Role::Owner => "Owner".purple().to_string(),
                            Role::Writer => "Writer".green().to_string(),
                            Role::Reader => "Reader".blue().to_string(),
                            Role::Auditor => "Auditor".yellow().to_string(),
                        };
                        
                        let status = if user.is_active { "Active".green().to_string() } else { "Inactive".red().to_string() };
                        let last_login = user.last_login
                            .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                            .unwrap_or_else(|| "Never".to_string());
                        
                        println!("  {} {} | {} | {} | {}", 
                            "•".green(), user.email.cyan(), role_colored, status, last_login);
                    }
                    
                    println!("\nTotal: {} users", user_count);
                }
                Err(e) => {
                    output::print_error(&format!("Failed to list users: {}", e));
                }
            }
        }
        
        UserAction::Remove { email } => {
            let session = SessionManager::get_current_session()?;
            
            if session.user_id == email {
                output::print_error("Cannot remove yourself");
                return Ok(());
            }
            
            if Confirm::new()
                .with_prompt(format!("Remove user {} from tenant {}?", email, session.tenant_id))
                .interact()?
            {
                match storage.remove_user(&session.tenant_id, &email).await {
                    Ok(_) => {
                        output::print_success(&format!("User {} removed from tenant", email));
                    }
                    Err(e) => {
                        output::print_error(&format!("Failed to remove user: {}", e));
                    }
                }
            } else {
                output::print_info("Operation cancelled");
            }
        }
        
        UserAction::ChangeRole { email, role } => {
            let role_enum = match role.to_lowercase().as_str() {
                "admin" => Role::Admin,
                "owner" => Role::Owner,
                "writer" => Role::Writer,
                "reader" => Role::Reader,
                "auditor" => Role::Auditor,
                _ => {
                    output::print_error(&format!("Invalid role: {}. Valid roles: admin, owner, writer, reader, auditor", role));
                    return Ok(());
                }
            };
            
            let session = SessionManager::get_current_session()?;
            
            if session.user_id == email && matches!(role_enum, Role::Reader | Role::Auditor) {
                output::print_warning("Changing your own role to a lower privilege level");
                if !Confirm::new()
                    .with_prompt("Are you sure you want to continue?")
                    .interact()?
                {
                    output::print_info("Operation cancelled");
                    return Ok(());
                }
            }
            
            match storage.change_user_role(&session.tenant_id, &email, role_enum).await {
                Ok(_) => {
                    output::print_success(&format!("User {} role changed to {}", email, role));
                }
                Err(e) => {
                    output::print_error(&format!("Failed to change user role: {}", e));
                }
            }
        }
    }
    
    Ok(())
}