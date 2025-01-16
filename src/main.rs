use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::fs;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Switch to a specific profile
    Switch {
        /// Name of the profile to switch to
        profile: Option<String>,
    },
    /// List all available profiles
    List,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProfileConfig {
    email: String,
    username: String,
    token_env: String,
    ssh_config: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    profiles: HashMap<String, ProfileConfig>,
    default_profile: String,
}

impl Config {
    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Current directory
        paths.push(PathBuf::from("config.yaml"));
        
        // XDG config directory
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("switch_profile");
            config_dir.push("config.yaml");
            paths.push(config_dir);
        }
        
        paths
    }

    fn load() -> Result<Self> {
        let paths = Self::get_config_paths();
        let mut last_error = None;

        for path in paths {
            if path.exists() {
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file at {:?}", path))?;
                return serde_yaml::from_str(&contents)
                    .with_context(|| format!("Failed to parse configuration from {:?}", path));
            } else {
                last_error = Some(format!("Configuration file not found at {:?}", path));
            }
        }

        Err(anyhow::anyhow!(last_error.unwrap_or_else(|| 
            "No configuration file found. Create config.yaml in the current directory or ~/.config/switch_profile/".to_string())))
    }

    fn get_profile(&self, name: Option<String>) -> Result<(String, &ProfileConfig)> {
        let profile_name = name.unwrap_or_else(|| self.default_profile.clone());
        let profile = self.profiles.get(&profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile {} not found", profile_name))?;
        Ok((profile_name, profile))
    }
}

fn ensure_config_dir() -> Result<PathBuf> {
    let mut config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    config_dir.push("switch_profile");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory at {:?}", config_dir))?;
    }
    
    Ok(config_dir)
}

fn switch_profile(name: &str, profile: &ProfileConfig) -> Result<()> {
    // Set global git config
    Command::new("git")
        .args(["config", "--global", "user.email", &profile.email])
        .output()
        .context("Failed to set git email")?;

    Command::new("git")
        .args(["config", "--global", "user.name", &profile.username])
        .output()
        .context("Failed to set git username")?;

    // Set SSH config
    let expanded_ssh_path = shellexpand::tilde(&profile.ssh_config);
    if let Ok(ssh_config) = std::fs::read_to_string(expanded_ssh_path.as_ref()) {
        let ssh_config_path = PathBuf::from(shellexpand::tilde("~/.ssh/config").as_ref());
        std::fs::write(&ssh_config_path, ssh_config)
            .context("Failed to update SSH config")?;
    }

    // Print environment variable export command
    println!("export GITHUB_TOKEN=\"${{{}}}\"", profile.token_env);
    
    // Print status as a comment
    println!("# Profile switched to: {}", name);
    println!("# Email: {}", profile.email);
    println!("# Username: {}", profile.username);
    println!("# Using token from: {}", profile.token_env);
    println!("# SSH config: {}", profile.ssh_config);
    Ok(())
}

fn main() -> Result<()> {
    // Ensure config directory exists
    ensure_config_dir()?;

    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Commands::Switch { profile } => {
            let (name, profile) = config.get_profile(profile)?;
            switch_profile(&name, profile)?;
        }
        Commands::List => {
            println!("# Available profiles (default: {}):", config.default_profile);
            for (name, profile) in &config.profiles {
                println!("# - {} ({} <{}>)", name, profile.username, profile.email);
                println!("#   Token environment: {}", profile.token_env);
                println!("#   SSH config: {}", profile.ssh_config);
            }
        }
    }

    Ok(())
}
