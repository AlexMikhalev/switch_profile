use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

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

#[derive(Debug, Default)]
#[config]
struct Config {
    profiles: HashMap<String, ProfileConfig>,
    default_profile: String,
}

impl Config {
    fn load() -> Result<Self> {
        Config::with_layers(&[
            Layer::Yaml("config.yaml".into()),
        ])
        .context("Failed to load configuration")
    }

    fn get_profile(&self, name: Option<String>) -> Result<(String, &ProfileConfig)> {
        let profile_name = name.unwrap_or_else(|| self.default_profile.clone());
        let profile = self.profiles.get(&profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile {} not found", profile_name))?;
        Ok((profile_name, profile))
    }
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

    // Print environment variable that needs to be set
    println!("# Run this command to set up the environment:");
    println!("export GITHUB_TOKEN=\"${{{0}}}\"", profile.token_env);

    println!("\nSwitched to profile: {}", name);
    println!("Email: {}", profile.email);
    println!("Username: {}", profile.username);
    println!("Using token from: {}", profile.token_env);
    println!("SSH config: {}", profile.ssh_config);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Commands::Switch { profile } => {
            let (name, profile) = config.get_profile(profile)?;
            switch_profile(&name, profile)?;
        }
        Commands::List => {
            println!("Available profiles (default: {}):", config.default_profile);
            for (name, profile) in &config.profiles {
                println!("- {} ({} <{}>)", name, profile.username, profile.email);
                println!("  Token environment: {}", profile.token_env);
                println!("  SSH config: {}", profile.ssh_config);
            }
        }
    }

    Ok(())
}
