# jsh - Joseph's Shell

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**jsh** (Joseph's Shell) is a modern ZSH/Bash-compatible shell that combines full compatibility with existing scripts while introducing cleaner, more intuitive syntax for flow control and scripting.

## Features

### Bash/ZSH Compatible
- ✅ If-then-else-elif-fi statements
- ✅ For loops (`for x in ...; do ...; done`)
- ✅ While and until loops
- ✅ Case statements
- ✅ Functions (`function name { }` and `name() { }`)
- ✅ Pipes (`|`) and redirections (`>`, `>>`, `<`, `2>`)
- ✅ Command substitution `$(...)`
- ✅ Variable expansion `$var`, `${var}`, `${var:-default}`
- ✅ Background jobs (`&`)
- ✅ Logical operators (`&&`, `||`)
- ✅ Glob expansion (`*.txt`, `file?.log`)

### Fish Compatible
- ✅ `set` command with Fish-style flags (`-x`, `-e`, `-q`, `-g`)
- ✅ `string` builtin (length, upper, lower, split, join, replace, match, etc.)
- ✅ `math` builtin for arithmetic
- ✅ `contains` for list membership
- ✅ `status` for shell state queries
- ✅ `functions` for function management
- ✅ `abbr` for abbreviations
- ✅ `begin...end` blocks
- ✅ `switch...case...end` statements
- ✅ `and`, `or`, `not` keywords

### Git Integration
- ✅ `git_branch` - Get current git branch name
- ✅ `git_status` - Git status in shell-friendly format
- ✅ `git_info` - Detailed git repository information
- ✅ `git_prompt` - Git info formatted for shell prompts
- ✅ `in_git_repo` - Check if in a git repository

### Ash/POSIX Compatible
- ✅ POSIX `set` options (`-e`, `-u`, `-x`, `-n`, `-a`, `-C`, `-b`, `-f`)
- ✅ `set -o optname` / `set +o optname` for named options
- ✅ `set --` for positional parameters
- ✅ `readonly` for read-only variables
- ✅ `local` for function-local variables
- ✅ `.` (dot) command for sourcing
- ✅ `trap` for signal handling
- ✅ `getopts` for option parsing
- ✅ Here documents (`<<EOF`)
- ✅ Arithmetic expansion `$((...))`
- ✅ POSIX test command `[` and `test`

### jsh Enhanced Syntax

#### Pattern Matching with `match`
```bash
match $value {
    1 => echo "one"
    2 | 3 => echo "two or three"
    4..10 => echo "between 4 and 10"
    /^hello/ => echo "starts with hello"
    * => echo "anything else"
}
```

#### Infinite Loop with `loop`
```bash
loop {
    read -p "Enter command: " cmd
    if [ "$cmd" = "quit" ]; then
        break
    fi
    eval "$cmd"
}
```

#### Variable Bindings with `let` and `const`
```bash
let name = "world"
const PI = "3.14159"
echo "Hello, $name!"
```

#### Error Handling with `try`/`catch`/`finally`
```bash
try {
    risky_command
} catch err {
    echo "Error occurred: $err"
} finally {
    cleanup_resources
}
```

#### Function Shorthand with `fn`
```bash
fn greet {
    echo "Hello, $1!"
}

greet "World"
```

#### Named Function Parameters
```bash
# Traditional shell uses $1, $2 for positional arguments
# jsh allows named parameters for clearer code

greet(name, greeting) {
    echo "$greeting, $name!"
}
greet "World" "Hello"  # Output: Hello, World!

# Works with all function syntax styles
fn add(a, b) {
    echo $((a + b))
}
add 10 20  # Output: 30

function multiply(x, y) {
    echo $((x * y))
}
multiply 6 7  # Output: 42

# Named parameters are also accessible as $1, $2, etc.
show(first, second) {
    echo "Named: first=$first second=$second"
    echo "Positional: \$1=$1 \$2=$2"
}
```

#### Git Integration in Scripts
```bash
# Check if in a git repository
if in_git_repo; then
    echo "Current branch: $(git_branch)"
fi

# Conditional logic based on branch
if [ "$(git_branch)" = "main" ]; then
    echo "On main branch - be careful!"
fi

# Check for uncommitted changes before deploying
if git_status --dirty; then
    echo "Error: Uncommitted changes detected"
    exit 1
fi

# Get detailed git info
git_info --query branch    # Just the branch name
git_info --query hash      # Full commit hash
git_info --query remote    # Remote name

# Custom formatted output
git_info --format '%b (%h)'  # "main (abc123)"

# Use git_prompt in your PS1
PS1="\u@\h:\w$(git_prompt -f ' [%b%d]')$ "
# Output: user@host:~/project [main✗]$
```

## Installation

### Pre-built Packages

#### Debian/Ubuntu (.deb)

```bash
# Download the latest release
wget https://github.com/pegasusheavy/jsh/releases/latest/download/jsh_0.1.0_amd64.deb

# Install
sudo apt install ./jsh_0.1.0_amd64.deb

# Or with dpkg
sudo dpkg -i jsh_0.1.0_amd64.deb
```

#### Fedora/RHEL/CentOS (.rpm)

```bash
# Download the latest release
wget https://github.com/pegasusheavy/jsh/releases/latest/download/jsh-0.1.0-1.x86_64.rpm

# Install (Fedora/RHEL 8+)
sudo dnf install jsh-0.1.0-1.x86_64.rpm

# Or with rpm
sudo rpm -i jsh-0.1.0-1.x86_64.rpm
```

#### Arch Linux (AUR)

```bash
# Using an AUR helper (e.g., yay, paru)
yay -S jsh

# Or manually with PKGBUILD
git clone https://github.com/pegasusheavy/jsh.git
cd jsh/packaging
makepkg -si
```

### From Source

```bash
# Clone the repository
git clone https://github.com/pegasusheavy/jsh.git
cd jsh

# Build with Cargo
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .
```

### Building Packages

To build distribution packages yourself:

```bash
# Build Debian package
cargo install cargo-deb
./packaging/build-deb.sh

# Build RPM package
cargo install cargo-generate-rpm
./packaging/build-rpm.sh

# Build both
./packaging/build-all.sh
```

See [packaging/README.md](packaging/README.md) for detailed instructions.

### Requirements

- Rust 1.85+ (Edition 2024)
- Unix-like operating system (Linux, macOS, BSD)

## Usage

### Interactive Shell
```bash
jsh
```

### Run a Script
```bash
jsh script.sh
jsh script.sh arg1 arg2
```

### Run a Command
```bash
jsh -c 'echo "Hello, World!"'
```

### Command Line Options
```
OPTIONS:
    -c <command>    Execute command string and exit
    -s              Read commands from stdin
    -i              Force interactive mode
    -l, --login     Start as login shell
    -h, --help      Print help message
    -V, --version   Print version information
```

## Shell Initialization

jsh follows POSIX conventions for shell initialization, automatically picking up environment variables and sourcing profile files.

### Environment Variables

jsh automatically inherits all environment variables from the parent process and sets these shell-specific variables:

| Variable | Description |
|----------|-------------|
| `SHELL` | Path to jsh executable |
| `SHLVL` | Shell nesting level (incremented for each subshell) |
| `PWD` | Current working directory |
| `OLDPWD` | Previous working directory |
| `HOME` | User's home directory |
| `USER` | Current username |
| `HOSTNAME` | System hostname |
| `TERM` | Terminal type (defaults to xterm-256color) |
| `PATH` | Command search path |
| `IFS` | Internal field separator |
| `HISTFILE` | History file location |
| `HISTSIZE` | Maximum history entries |

### Environment File (Always Sourced)

`~/.jshenv` is sourced for ALL shell invocations (login, interactive, non-interactive scripts):

1. `/etc/jshenv` - System-wide environment
2. `~/.jshenv` - User environment
3. `~/.config/jsh/env` - XDG location

Use `.jshenv` for:
- Environment variables
- PATH modifications
- Settings that should apply to all shells and scripts

### Login Shell Initialization

When started as a login shell (`jsh -l`, `jsh --login`, or with `-jsh` as argv[0]):

1. `~/.jshenv` (always first)
2. `/etc/environment` - Parses KEY=VALUE pairs
3. `/etc/profile` - System-wide login profile
4. `/etc/profile.d/*.sh` - Additional system scripts
5. `/etc/jsh_profile` - System-wide jsh login profile
6. First of (in order):
   - `~/.jsh_profile`
   - `~/.bash_profile`
   - `~/.bash_login`
   - `~/.profile`

### Interactive Shell Initialization

For interactive shells (not running a script):

1. `~/.jshenv` (always first)
2. `/etc/jsh.jshrc`, `/etc/jshrc`, or `/etc/bash.bashrc` - System-wide config
3. First of (in order):
   - `~/.jshrc`
   - `~/.config/jsh/jshrc` (XDG config)
   - `~/.bashrc` (fallback for compatibility)
   - `~/.zshrc` (fallback for compatibility)

### System-Wide Configuration Files

| File | When Sourced | Purpose |
|------|--------------|---------|
| `/etc/jshenv` | Always (all shells) | System environment variables |
| `/etc/jsh_profile` | Login shells | System login initialization |
| `/etc/jshrc` | Interactive shells | System interactive config |

### Configuration Files

**`~/.jshenv`** - Environment settings (sourced for ALL shells):

```bash
# ~/.jshenv - Always sourced, even for non-interactive scripts

# Path modifications
export PATH="$HOME/.local/bin:$HOME/bin:$PATH"

# Editor and pager
export EDITOR="vim"
export PAGER="less"

# Language settings
export LANG="en_US.UTF-8"

# Application-specific
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"
```

**`~/.jshrc`** - Interactive shell configuration:

```bash
# ~/.jshrc - Interactive shell settings
export JSH_THEME="robbyrussell"

# Aliases
alias ll="ls -la"
alias gs="git status"
alias gp="git push"

# Functions
fn mkcd {
    mkdir -p "$1" && cd "$1"
}

# Prompt customization
export PROMPT='%F{cyan}%n%f@%F{blue}%m%f:%F{yellow}%~%f$ '
```

**`~/.jsh_profile`** - Login shell configuration:

```bash
# ~/.jsh_profile - Login shell settings (sourced once at login)

# Enable automatic ssh-agent startup (jsh built-in feature)
export JSH_SSH_AGENT_AUTO_START=true

# Or use a custom socket path
# export JSH_SSH_AGENT_SOCKET="/run/user/$UID/ssh-agent.socket"

# Load any secrets
if [ -f ~/.secrets ]; then
    source ~/.secrets
fi

# Welcome message
echo "Welcome back, $USER!"
```

**`~/.config/jsh/jshrc`** - XDG-compliant configuration:

```bash
# XDG config location for jsh (~/.config/jsh/jshrc)
export JSH_THEME="pure"
```

**`~/.config/jsh/env`** - XDG-compliant environment:

```bash
# XDG environment file (~/.config/jsh/env)
# Alternative to ~/.jshenv
export PATH="$HOME/.local/bin:$PATH"
```

## Themes (Oh-My-Zsh Compatible)

jsh includes an oh-my-zsh compatible theme system with prompt escape sequences, colors, and git integration.

### Built-in Themes

| Theme | Description |
|-------|-------------|
| `jsh` | Default two-line theme with user, host, git, and status |
| `robbyrussell` | Classic oh-my-zsh default |
| `agnoster` | Powerline-style segments |
| `minimal` | Clean, simple prompt |
| `powerlevel` | Powerline-inspired with time |
| `simple` | Basic `user@host:path$` format |
| `pure` | ZSH Pure theme inspired |

### Setting a Theme

**In `.jshrc`:**
```bash
export JSH_THEME="robbyrussell"
```

**Interactively:**
```bash
theme robbyrussell       # Set theme directly
theme set agnoster       # Set with explicit command
theme list               # List available themes
theme preview            # Preview all themes
```

### Custom Prompts

Set `PROMPT` and optionally `RPROMPT` for full control:

```bash
export PROMPT='%F{cyan}%n%f@%F{blue}%m%f:%F{yellow}%~%f$ '
export RPROMPT='%F{8}%T%f'
```

### Prompt Escape Sequences

| Sequence | Description |
|----------|-------------|
| `%n` | Username |
| `%m` / `%M` | Short / full hostname |
| `%~` | Current directory (~ for home) |
| `%c` | Current directory name only |
| `%T` / `%*` | Time (HH:MM / HH:MM:SS) |
| `%D{fmt}` | Custom date format |
| `%?` | Last exit status |
| `%#` | `#` for root, `%` otherwise |
| `%F{color}...%f` | Foreground color |
| `%K{color}...%k` | Background color |
| `%B...%b` | Bold |
| `%(?.true.false)` | Conditional on exit status |
| `$(git_prompt_info)` | Git branch and status |

### Colors

Colors can be specified as:
- Names: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- Bright: `bright_red`, etc.
- 256-color: `0`-`255`
- True color: `#ff5500`

### Example Custom Theme

```bash
# Two-line prompt with git
export PROMPT='%F{cyan}%n%f %F{8}@%f %F{blue}%m%f$(git_prompt_info)
%F{yellow}%~%f %(?.%F{green}.%F{red})❯%f '
```

## Built-in Commands

jsh includes many built-in commands:

| Command | Description |
|---------|-------------|
| `cd` | Change directory |
| `pwd` | Print working directory |
| `echo` | Print arguments |
| `printf` | Formatted output |
| `export` | Export variables |
| `set` | Set shell options |
| `unset` | Unset variables |
| `source` / `.` | Execute script in current shell |
| `eval` | Evaluate string as command |
| `test` / `[` | Evaluate conditional |
| `true` / `false` | Return success/failure |
| `exit` | Exit the shell |
| `return` | Return from function |
| `break` / `continue` | Loop control |
| `read` | Read input |
| `type` / `which` | Describe command |
| `help` | Display help |
| `ssh_agent` | SSH agent control |

## SSH Agent Integration

jsh includes built-in support for automatic ssh-agent management.

### Automatic Startup

Enable automatic ssh-agent startup in your `~/.jshrc`, `~/.jsh_profile`, or `~/.jshenv`:

```bash
# Enable automatic ssh-agent startup
export JSH_SSH_AGENT_AUTO_START=true

# Optional: Use a custom socket path (e.g., for systemd user units)
export JSH_SSH_AGENT_SOCKET="/run/user/$UID/ssh-agent.socket"
```

When enabled, jsh will:
1. Check if `SSH_AUTH_SOCK` is set and the socket exists
2. If not, automatically start `ssh-agent`
3. Export `SSH_AUTH_SOCK` and `SSH_AGENT_PID` environment variables

### Manual Control with `ssh_agent` Builtin

```bash
ssh_agent              # Show status (default)
ssh_agent status       # Show status and loaded keys
ssh_agent start        # Start ssh-agent if not running
ssh_agent stop         # Stop the current ssh-agent
ssh_agent add          # Add default SSH keys (~/.ssh/id_*)
```

### Example Configuration

**For Login Shells (`~/.jsh_profile`):**
```bash
# Automatically start ssh-agent on login
export JSH_SSH_AGENT_AUTO_START=true

# Optionally auto-add keys (requires ssh-add to be set up with keychain or similar)
ssh_agent add 2>/dev/null
```

**For systemd User Units:**
```bash
# Use systemd-managed ssh-agent socket
export JSH_SSH_AGENT_SOCKET="$XDG_RUNTIME_DIR/ssh-agent.socket"
```

**For GPG-Agent SSH Support:**
```bash
# Use gpg-agent for SSH (if gpg-agent is configured with enable-ssh-support)
export JSH_SSH_AGENT_SOCKET="$(gpgconf --list-dirs agent-ssh-socket)"
```

## Examples

### Traditional Bash Style
```bash
#!/usr/bin/env jsh

# Variables
name="World"
echo "Hello, $name!"

# Conditionals
if [ -f "config.txt" ]; then
    echo "Config exists"
elif [ -f "config.default.txt" ]; then
    echo "Using default config"
else
    echo "No config found"
fi

# Loops
for file in *.txt; do
    echo "Processing: $file"
done

# Case statement
case "$1" in
    start)
        echo "Starting..."
        ;;
    stop)
        echo "Stopping..."
        ;;
    *)
        echo "Unknown command"
        ;;
esac
```

### jsh Enhanced Style
```bash
#!/usr/bin/env jsh

# Constants and variables
const VERSION = "1.0.0"
let counter = 0

# Pattern matching
match $1 {
    start => {
        echo "Starting version $VERSION..."
        start_service
    }
    stop => {
        echo "Stopping..."
        stop_service
    }
    status => echo "Running"
    * => echo "Usage: $0 {start|stop|status}"
}

# Error handling
try {
    let result = $(risky_operation)
    echo "Success: $result"
} catch e {
    echo "Failed: $e"
    exit 1
} finally {
    cleanup
}

# Modern function syntax
fn process_file {
    let file = $1
    echo "Processing: $file"
}

# Infinite loop with break
loop {
    let counter = $((counter + 1))
    echo "Iteration: $counter"
    if [ $counter -ge 5 ]; then
        break
    fi
}
```

### Fish-Style Scripting
```fish
#!/usr/bin/env jsh

# Fish-style variable assignment
set name "World"
set -x PATH "/usr/local/bin" $PATH  # Export

# String manipulation
string upper "hello"                 # HELLO
string split "," "a,b,c"             # a\nb\nc
string join "-" a b c                # a-b-c
string replace "old" "new" "old text"

# Math operations
math "2 + 3 * 4"                     # 14
math "2 ^ 10"                        # 1024

# List operations
set fruits apple banana cherry
contains apple $fruits && echo "Found!"

# Fish-style switch
switch $cmd
    case start
        echo "Starting..."
    case stop
        echo "Stopping..."
    case '*'
        echo "Unknown command"
end

# Status checks
if status is-interactive
    echo "Interactive mode"
end

# Function management
functions                            # List all functions
functions -q myfunction && echo "Exists"
```

## License

This project is dual-licensed under either:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Copyright

Copyright © 2025 Pegasus Heavy Industries LLC
