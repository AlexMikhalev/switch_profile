# GitHub Profile Switcher

A command-line tool written in Rust to easily switch between different GitHub profiles, managing Git configurations, SSH settings, and GitHub tokens.

## Features

- Switch between multiple GitHub profiles
- Manage different SSH configurations per profile
- Handle GitHub tokens through environment variables
- Automatic Git email and username switching
- Shell integration with error handling
- Support for 1Password SSH agent

## Prerequisites

- Rust toolchain (cargo)
- Git
- SSH key pairs for different profiles
- GitHub personal access tokens
- 1Password SSH agent (optional)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd switch_profile
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

### 1. SSH Keys

Create SSH key pairs for each profile or use existing ones from 1Password following [link](https://developer.1password.com/docs/ssh/agent/advanced/#use-multiple-github-accounts):

```bash
# For personal profile
ssh-keygen -t ed25519 -f ~/.ssh/personal

# For work profile
ssh-keygen -t ed25519 -f ~/.ssh/klarian
```

Make sure to set correct permissions:
```bash
chmod 600 ~/.ssh/personal ~/.ssh/klarian
```

### 2. SSH Configurations

Create separate SSH config files for each profile:

1. Personal Profile (`~/.ssh/config.personal`):
```ssh-config
# Added by OrbStack: 'orb' SSH host for Linux machines
Include ~/.orbstack/ssh/config

# Personal GitHub
Host github.com
    HostName github.com
    User git
    IdentityFile ~/.ssh/personal.pub
    IdentitiesOnly yes

# Registry
Host registrygit
    HostName registry.terraphim.io
    User alex
    IdentityFile ~/.ssh/id_rsa.pub
    IdentitiesOnly yes

Host *
    IdentityAgent "~/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock"
```

2. Work Profile (`~/.ssh/config.klarian`):
```ssh-config
# Added by OrbStack: 'orb' SSH host for Linux machines
Include ~/.orbstack/ssh/config

# Klarian GitHub
Host github.com
    HostName github.com
    User git
    IdentityFile ~/.ssh/klarian.pub
    IdentitiesOnly yes

# Klarian Bitbucket
Host bitbucket.org
    HostName bitbucket.org
    User git
    IdentityFile ~/.ssh/klarian.pub
    IdentitiesOnly yes

# Registry
Host registrygit
    HostName registry.terraphim.io
    User alex
    IdentityFile ~/.ssh/id_rsa.pub
    IdentitiesOnly yes

Host *
    IdentityAgent "~/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock"
```

### 3. GitHub Tokens

Set up environment variables for your GitHub tokens:

```bash
# Add to your ~/.zshrc or ~/.bashrc
export GITHUB_TOKEN_PERSONAL="your-personal-github-token"
export GITHUB_TOKEN_KLARIAN="your-klarian-github-token"
```

### 4. Profile Configuration

Create a `config.yaml` file in the project directory:

```yaml
profiles:
  personal:
    email: alex@metacortex.engineer
    username: "Alex Mikhalev"
    token_env: GITHUB_TOKEN_PERSONAL
    ssh_config: ~/.ssh/config.personal
  klarian:
    email: alex.mikhalev@klarian.com
    username: "Alex Mikhalev"
    token_env: GITHUB_TOKEN_KLARIAN
    ssh_config: ~/.ssh/config.klarian

default_profile: personal
```

### 5. Shell Integration

Add these functions and aliases to your `~/.zshrc` and/or `~/.bashrc`:

```bash
# GitHub Profile Switcher
gp-switch() {
    local output
    output=$($HOME/projects/personal/switch_profile/target/release/switch_profile switch "$1")
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        echo "Error switching profile: $output"
        return $exit_code
    fi
    
    echo "$output" | grep '^#' >&2
    eval "$(echo "$output" | grep -v '^#')"
}
alias gp-personal="gp-switch personal"
alias gp-klarian="gp-switch klarian"
alias gp-list="$HOME/projects/personal/switch_profile/target/release/switch_profile list"
```

After adding the shell integration, reload your shell configuration:
```bash
source ~/.zshrc  # or source ~/.bashrc for bash
```

## Usage

1. List available profiles:
```bash
gp-list
```

2. Switch to personal profile:
```bash
gp-personal
```

3. Switch to work profile:
```bash
gp-klarian
```

When switching profiles, the tool will:
- Update Git global email and username
- Switch SSH configuration
- Export the appropriate GitHub token
- Display the current profile settings

The tool provides feedback with comments (prefixed with #) showing:
- Profile name
- Email configuration
- Username configuration
- Token environment variable being used
- SSH config path

## Troubleshooting

1. SSH Key Issues:
   - Ensure SSH keys have correct permissions: `chmod 600 ~/.ssh/personal ~/.ssh/klarian`
   - Test SSH connection: `ssh -T git@github.com`
   - Verify SSH agent: `ssh-add -l`

2. Token Issues:
   - Verify environment variables are set: `echo $GITHUB_TOKEN_PERSONAL`
   - Check token permissions on GitHub
   - Try re-exporting tokens manually

3. Git Configuration:
   - Verify current git config: `git config --global --list`
   - Check if email and username are set correctly

4. Shell Integration:
   - Make sure the shell function is loaded: `type gp-switch`
   - Check if the binary is accessible: `ls -l $HOME/projects/personal/switch_profile/target/release/switch_profile`
   - Verify shell script permissions

## Dependencies

- clap: Command line argument parsing
- serde: Serialization/deserialization for YAML
- anyhow: Error handling
- twelf: Configuration management with YAML support
- shellexpand: Shell path expansion

## License

MIT License 