use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;

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
        profile: String,
    },
    /// List all available profiles
    List,
    /// Add a new profile
    Add {
        /// Name of the profile
        name: String,
        /// Git user email
        email: String,
        /// Git user name
        username: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    name: String,
    email: String,
    username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    profiles: Vec<Profile>,
}

impl Config {
    fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        if !config_path.exists() {
            return Ok(Config { profiles: Vec::new() });
        }

        let mut file = File::open(&config_path)
            .with_context(|| format!("Failed to open config file at {:?}", config_path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Failed to read config file")?;

        serde_yaml::from_str(&contents).context("Failed to parse config file")
    }

    fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let contents = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(&config_path, contents).context("Failed to write config file")?;
        Ok(())
    }

    fn add_profile(&mut self, name: String, email: String, username: String) -> Result<()> {
        if self.profiles.iter().any(|p| p.name == name) {
            return Err(anyhow::anyhow!("Profile {} already exists", name));
        }

        self.profiles.push(Profile {
            name,
            email,
            username,
        });
        self.save()?;
        Ok(())
    }

    fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }
}

fn get_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    path.push("github-profile-switcher");
    path.push("config.yml");
    Ok(path)
}

fn switch_profile(profile: &Profile) -> Result<()> {
    // Set global git config
    Command::new("git")
        .args(["config", "--global", "user.email", &profile.email])
        .output()
        .context("Failed to set git email")?;

    Command::new("git")
        .args(["config", "--global", "user.name", &profile.username])
        .output()
        .context("Failed to set git username")?;

    println!("Switched to profile: {}", profile.name);
    println!("Email: {}", profile.email);
    println!("Username: {}", profile.username);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Switch { profile } => {
            let profile = config
                .get_profile(&profile)
                .ok_or_else(|| anyhow::anyhow!("Profile {} not found", profile))?;
            switch_profile(profile)?;
        }
        Commands::List => {
            if config.profiles.is_empty() {
                println!("No profiles configured");
                return Ok(());
            }
            println!("Available profiles:");
            for profile in &config.profiles {
                println!("- {} ({} <{}>)", profile.name, profile.username, profile.email);
            }
        }
        Commands::Add {
            name,
            email,
            username,
        } => {
            config.add_profile(name.clone(), email, username)?;
            println!("Added profile: {}", name);
        }
    }

    Ok(())
}
