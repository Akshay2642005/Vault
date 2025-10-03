use owo_colors::OwoColorize;
use serde_json::Value;

pub struct OutputFormatter {
    pub json: bool,
    pub quiet: bool,
}

impl OutputFormatter {
    pub fn new(json: bool, quiet: bool) -> Self {
        Self { json, quiet }
    }
    
    pub fn success(&self, message: &str) {
        if self.quiet {
            return;
        }
        
        if self.json {
            let output = serde_json::json!({
                "status": "success",
                "message": message
            });
            println!("{}", output);
        } else {
            println!("{} {}", "✓".green(), message);
        }
    }
    
    pub fn error(&self, message: &str) {
        if self.json {
            let output = serde_json::json!({
                "status": "error",
                "message": message
            });
            eprintln!("{}", output);
        } else {
            eprintln!("{} {}", "✗".red(), message);
        }
    }
    
    pub fn info(&self, message: &str) {
        if self.quiet {
            return;
        }
        
        if self.json {
            let output = serde_json::json!({
                "status": "info",
                "message": message
            });
            println!("{}", output);
        } else {
            println!("{} {}", "ℹ".blue(), message);
        }
    }
    
    pub fn warning(&self, message: &str) {
        if self.quiet {
            return;
        }
        
        if self.json {
            let output = serde_json::json!({
                "status": "warning",
                "message": message
            });
            println!("{}", output);
        } else {
            println!("{} {}", "⚠".yellow(), message);
        }
    }
    
    pub fn data(&self, data: &Value) {
        if self.json {
            println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
        } else {
            // Format data for human-readable output
            self.format_data(data, 0);
        }
    }
    
    fn format_data(&self, data: &Value, indent: usize) {
        let prefix = "  ".repeat(indent);
        
        match data {
            Value::Object(map) => {
                for (key, value) in map {
                    match value {
                        Value::Object(_) | Value::Array(_) => {
                            println!("{}{}:", prefix, key.cyan());
                            self.format_data(value, indent + 1);
                        }
                        _ => {
                            println!("{}{}: {}", prefix, key.cyan(), self.format_value(value));
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (i, value) in arr.iter().enumerate() {
                    println!("{}[{}]:", prefix, i);
                    self.format_data(value, indent + 1);
                }
            }
            _ => {
                println!("{}{}", prefix, self.format_value(data));
            }
        }
    }
    
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => if *b { "true".green().to_string() } else { "false".red().to_string() },
            Value::Null => "null".dimmed().to_string(),
            _ => value.to_string(),
        }
    }
}