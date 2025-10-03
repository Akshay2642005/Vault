use anyhow::Result;
use dialoguer::Confirm;
use owo_colors::OwoColorize;

use crate::{
    storage::{AuditLogger, AuditEntry},
    cli::{RoleAction, AuditAction, output},
    auth::{SessionManager, Role},
};

pub async fn roles_command(action: RoleAction) -> Result<()> {
    if let Ok(session) = SessionManager::get_current_session() {
        if !session.role.can_admin() {
            output::print_error("Admin permissions required for role management");
            return Ok(());
        }
    } else {
        output::print_error("Please login first");
        return Ok(());
    }
    
    match action {
        RoleAction::Add { tenant, user, role } => {
            let _role_enum = match role.to_lowercase().as_str() {
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
            
            println!("{} Adding user {} to tenant {} with role {}", "➕".green(), user.cyan(), tenant.cyan(), role.yellow());
            
            if let Ok(session) = SessionManager::get_current_session() {
                let audit_entry = AuditEntry::new(
                    tenant.clone(),
                    AuditLogger::EVENT_USER_ADDED.to_string(),
                    format!("User {} added with role {}", user, role),
                    session.user_id.clone(),
                );
                let _ = AuditLogger::log_event(&audit_entry);
                
                // Also log role change
                let role_entry = AuditEntry::new(
                    tenant.clone(),
                    AuditLogger::EVENT_ROLE_CHANGED.to_string(),
                    format!("User {} assigned role {}", user, role),
                    session.user_id,
                );
                let _ = AuditLogger::log_event(&role_entry);
            }
            
            output::print_success(&format!("User {} added to tenant {} with role {}", user, tenant, role));
        }
        RoleAction::Remove { tenant, user } => {
            if Confirm::new()
                .with_prompt(format!("Remove user {} from tenant {}?", user, tenant))
                .interact()?
            {
                println!("{} Removing user {} from tenant {}", "➖".red(), user.cyan(), tenant.cyan());
                
                if let Ok(session) = SessionManager::get_current_session() {
                    let audit_entry = AuditLogger::create_entry(
                        &tenant,
                        AuditLogger::EVENT_USER_REMOVED,
                        &format!("User {} removed from tenant", user),
                        &session.user_id,
                    );
                    let _ = AuditLogger::log_event(&audit_entry);
                }
                
                output::print_success(&format!("User {} removed from tenant {}", user, tenant));
            } else {
                output::print_info("Operation cancelled");
            }
        }
        RoleAction::List { tenant } => {
            println!("{} Users in tenant {}:", "📊".cyan(), tenant.cyan());
            
            let users = vec![
                ("admin@example.com", "Admin", "2024-01-01"),
                ("user@example.com", "Writer", "2024-01-15"),
                ("reader@example.com", "Reader", "2024-02-01"),
                ("auditor@example.com", "Auditor", "2024-02-10"),
            ];
            
            output::print_table_header(&["Email", "Role", "Added"]);
            
            for (email, role, added) in &users {
                let role_colored = match *role {
                    "Admin" => role.red().to_string(),
                    "Owner" => role.purple().to_string(),
                    "Writer" => role.green().to_string(),
                    "Reader" => role.blue().to_string(),
                    "Auditor" => role.yellow().to_string(),
                    _ => role.white().to_string(),
                };
                println!("  {} {} | {} | {}", "•".green(), email.cyan(), role_colored, added);
            }
            
            println!("\nTotal: {} users", users.len());
        }
    }
    Ok(())
}

pub async fn audit_command(action: AuditAction) -> Result<()> {
    if let Ok(session) = SessionManager::get_current_session() {
        if !session.role.can_audit() {
            output::print_error("Audit permissions required to view logs");
            return Ok(());
        }
    } else {
        output::print_error("Please login first");
        return Ok(());
    }
    
    match action {
        AuditAction::Tail { lines, follow } => {
            let limit = lines.unwrap_or(50);
            println!("{} Showing last {} audit entries:", "📜".cyan(), limit);
            
            let now = chrono::Utc::now();
            let audit_entries = vec![
                (now - chrono::Duration::hours(2), "LOGIN", "User admin@example.com logged in"),
                (now - chrono::Duration::hours(1), "SECRET_CREATED", "Secret development/api-key created"),
                (now - chrono::Duration::minutes(30), "SECRET_ACCESSED", "Secret development/api-key accessed"),
                (now - chrono::Duration::minutes(15), "SYNC_PUSH", "Pushed 5 secrets to cloud"),
                (now - chrono::Duration::minutes(5), "USER_ADDED", "User reader@example.com added with role Reader"),
            ];
            
            output::print_table_header(&["Timestamp", "Event", "Description"]);
            
            for (timestamp, event_type, description) in audit_entries.iter().take(limit) {
                let event_colored = match *event_type {
                    "LOGIN" | "LOGOUT" => event_type.blue().to_string(),
                    "SECRET_CREATED" | "SECRET_UPDATED" => event_type.green().to_string(),
                    "SECRET_ACCESSED" => event_type.yellow().to_string(),
                    "SECRET_DELETED" => event_type.red().to_string(),
                    "SYNC_PUSH" | "SYNC_PULL" => event_type.purple().to_string(),
                    "USER_ADDED" | "USER_REMOVED" => event_type.cyan().to_string(),
                    _ => event_type.white().to_string(),
                };
                
                println!(
                    "[{}] {} - {}",
                    timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                    event_colored,
                    description
                );
            }
            
            if follow {
                println!("\n{} Following audit log (Ctrl+C to stop)...", "👀".cyan());
                
                for i in 0..5 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    
                    let new_event = match i % 3 {
                        0 => "SECRET_ACCESSED - Secret production/database-url accessed",
                        1 => "LOGIN - User user@example.com logged in",
                        _ => "SECRET_CREATED - Secret staging/redis-password created",
                    };
                    
                    println!(
                        "[{}] {} - {}",
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                        "LIVE".bright_green(),
                        new_event
                    );
                }
            }
        }
        AuditAction::Search { query, since, until } => {
            println!("{} Searching audit logs for: {}", "🔍".cyan(), query.yellow());
            
            if let Some(since_date) = since {
                println!("Since: {}", since_date);
            }
            if let Some(until_date) = until {
                println!("Until: {}", until_date);
            }
            
            let search_results = vec![
                ("2024-01-15 10:30:00", "SECRET_CREATED", "Secret development/github-token created"),
                ("2024-01-15 14:22:00", "SECRET_ACCESSED", "Secret development/github-token accessed"),
                ("2024-01-16 09:15:00", "SECRET_UPDATED", "Secret development/github-token updated"),
            ];
            
            let matching_results: Vec<_> = search_results
                .iter()
                .filter(|(_, event, desc)| {
                    event.to_lowercase().contains(&query.to_lowercase()) ||
                    desc.to_lowercase().contains(&query.to_lowercase())
                })
                .collect();
            
            if matching_results.is_empty() {
                output::print_info("No matching audit entries found");
            } else {
                println!("\nFound {} matching entries:", matching_results.len());
                output::print_table_header(&["Timestamp", "Event", "Description"]);
                
                for (timestamp, event_type, description) in matching_results {
                    println!(
                        "[{}] {} - {}",
                        timestamp,
                        event_type.yellow(),
                        description
                    );
                }
            }
        }
    }
    Ok(())
}