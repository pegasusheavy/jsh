# 🧟 Franken Shell

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

```
    ███████╗██████╗  █████╗ ███╗   ██╗██╗  ██╗███████╗███╗   ██╗
    ██╔════╝██╔══██╗██╔══██╗████╗  ██║██║ ██╔╝██╔════╝████╗  ██║
    █████╗  ██████╔╝███████║██╔██╗ ██║█████╔╝ █████╗  ██╔██╗ ██║
    ██╔══╝  ██╔══██╗██╔══██║██║╚██╗██║██╔═██╗ ██╔══╝  ██║╚██╗██║
    ██║     ██║  ██║██║  ██║██║ ╚████║██║  ██╗███████╗██║ ╚████║
    ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝
                        ███████╗██╗  ██╗███████╗██╗     ██╗
                        ██╔════╝██║  ██║██╔════╝██║     ██║
                        ███████╗███████║█████╗  ██║     ██║
                        ╚════██║██╔══██║██╔══╝  ██║     ██║
                        ███████║██║  ██║███████╗███████╗███████╗
                        ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
```

**The vibe-coded shell that does everything stupidly™** 🔩⚡

A monstrous creation stitched together from the best parts of Bash, ZSH, Fish, and POSIX shells — held together with duct tape, good intentions, and an alarming amount of caffeine.

> *"It's alive! IT'S ALIVE!"* — You, after installing this shell

## Why Franken Shell?

Because sometimes you want a shell that:
- ✅ Works like Bash (when it feels like it)
- ✅ Steals features from Fish (with permission, mostly)
- ✅ Has POSIX compliance (we tried our best, okay?)
- ✅ Adds syntax that makes sense to exactly one person
- ✅ Includes a built-in tmux clone (why not?)
- ✅ Has more themes than your IDE
- ✅ Supports plugins you didn't know you needed

## Features

### 🧪 Bash/ZSH Compatible (Ish)
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

### 🐟 Fish Compatible (We Borrowed Some Things)
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

### 🔧 Git Integration (Because We're Not Savages)
- ✅ `git_branch` - Get current git branch name
- ✅ `git_status` - Git status in shell-friendly format
- ✅ `git_info` - Detailed git repository information
- ✅ `git_prompt` - Git info formatted for shell prompts
- ✅ `in_git_repo` - Check if in a git repository

### 📜 Ash/POSIX Compatible (We Read the Manual)
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

### ⚡ Franken Shell's Weird Syntax

Because we couldn't resist adding our own ideas:

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
const PI = "3.14159"  # immutable, like my stubbornness
echo "Hello, $name!"
```

#### Error Handling with `try`/`catch`/`finally`
```bash
try {
    risky_command
} catch err {
    echo "Error occurred: $err"
    # at least we tried
} finally {
    cleanup_resources
}
```

#### Function Shorthand with `fn`
```bash
# Simple fn (uses $1, $2 for args, like a normal person)
fn greet {
    echo "Hello, $1!"
}

# Named parameters (because we're fancy)
fn greet(name, greeting="Hello") {
    echo "$greeting, $name!"
}

greet "World"                    # Hello, World!
greet "World" "Sup"              # Sup, World!
greet name="Universe"            # Hello, Universe!
```

### 🎨 Themes (So Many Themes)

Built-in themes that range from practical to questionable:

**137 built-in themes** including all Oh-My-ZSH themes!

| Category | Themes |
|----------|--------|
| **FSH Native** | `fsh`, `gallifrey`, `skaro` |
| **Classic OMZ** | `robbyrussell`, `agnoster`, `avit`, `bira`, `bureau`, `fino`, `pure` |
| **Minimal** | `minimal`, `simple`, `clean`, `refined`, `sorin`, `nicoulaj` |
| **Powerline** | `powerlevel`, `bullet-train`, `spaceship`, `agnoster` |
| **Color Schemes** | `dracula`, `nord`, `gruvbox`, `catppuccin`, `tokyo-night`, `onedark`, `solarized`, `monokai` |
| **Fun** | `emotty`, `cloud`, `fox`, `awesomepanda`, `junkfood`, `terminalparty`, `half-life`, `lambda` |
| **Box Drawing** | `bira`, `fino`, `fino-time`, `bureau`, `jonathan`, `rkj`, `gnzh`, `darkblood` |
| **Classic Unix** | `gentoo`, `linuxonly`, `dst`, `dpoggi`, `norm` |

<details>
<summary>📋 All 137 Themes (Click to expand)</summary>

**Core**: `fsh`, `default`, `robbyrussell`, `agnoster`, `minimal`, `powerlevel`, `simple`, `pure`, `gallifrey`

**Oh-My-ZSH Themes**: `af-magic`, `afowler`, `alanpeabody`, `amuse`, `apple`, `arrow`, `aussiegeek`, `avit`, `awesomepanda`, `bira`, `blinks`, `bureau`, `candy`, `clean`, `cloud`, `crunch`, `cypher`, `dallas`, `darkblood`, `dieter`, `dpoggi`, `dst`, `dstufft`, `eastwood`, `emotty`, `essembeh`, `evan`, `fino`, `fino-time`, `fishy`, `flazz`, `fletcherm`, `fox`, `frisk`, `frontcube`, `funky`, `fwalch`, `gallois`, `gentoo`, `geoffgarside`, `gianu`, `gnzh`, `gozilla`, `half-life`, `humza`, `imajes`, `intheloop`, `itchy`, `jaischeema`, `jbergantine`, `jispwoso`, `jnrowe`, `jonathan`, `josh`, `jreese`, `jtriley`, `juanghurtado`, `junkfood`, `kafeitu`, `kardan`, `kennethreitz`, `kiwi`, `kolo`, `kphoen`, `lambda`, `linuxonly`, `lukerandall`, `macovsky`, `maran`, `mgutz`, `mh`, `michelebologna`, `mikeh`, `miloshadzic`, `mira`, `mortalscumbag`, `mrtazz`, `murilasso`, `muse`, `nanotech`, `nebirhos`, `nicoulaj`, `norm`, `obraun`, `peepcode`, `philips`, `pmcgee`, `pygmalion`, `re5et`, `refined`, `rgm`, `risto`, `rixius`, `rkj`, `sammy`, `simonoff`, `skaro`, `smt`, `sonicradish`, `sorin`, `sporty_256`, `steeef`, `strug`, `sunaku`, `sunrise`, `superjarin`, `suvash`, `takashiyoshida`, `terminalparty`, `theunraveler`, `tjkirch`, `tonotdo`, `trapd00r`, `wedisagree`, `wezm`, `wuffers`, `xiong-chiamiov`, `ys`, `zhann`

**Modern Color Schemes**: `spaceship`, `bullet-train`, `dracula`, `gruvbox`, `nord`, `catppuccin`, `tokyo-night`, `onedark`, `solarized`, `monokai`
</details>

```bash
# Set a theme
theme set dracula

# List all available themes
theme list

# Try some favorites
theme set robbyrussell    # The OMZ classic
theme set agnoster        # Powerline-style
theme set bira            # Box drawing
theme set catppuccin      # Modern color scheme
theme set half-life       # λ for the scientists
theme set gallifrey       # Time Lord approved ⧖

# Create your own monster
theme set-prompt "%F{green}⚡%f %~ %F{yellow}→%f "
```

### 🔌 Plugin Manager

A `zplug`-inspired plugin manager, because dependency management is fun:

```bash
# Register plugins
plug "zsh-users/zsh-syntax-highlighting"
plug "zsh-users/zsh-autosuggestions"
plug "oh-my-zsh" lib:git.zsh
plug "oh-my-zsh" plugins:git

# Install all the things
plug-install

# Update everything
plug-update

# See what chaos you've created
plug-list
```

### 🖥️ Built-in Tmux (Because Why Not?)

A full tmux-compatible multiplexer built right in:

```bash
# Sessions
tmux new-session -s dev
tmux list-sessions
tmux attach -t dev
tmux kill-session -t dev

# Windows
tmux new-window -n editor
tmux select-window -t 2
tmux rename-window -t 0 main

# Panes
tmux split-window -h
tmux split-window -v
tmux select-pane -L

# Theming
tmux theme set dracula
tmux theme list
```

### 🔍 FZF Integration

Fuzzy finding for everything:

```bash
fzf-history    # Search command history
fzf-file       # Find files
fzf-dir        # Find directories
fzf-cd         # cd with fuzzy finding
fzf-kill       # Kill processes fuzzy-style
fzf-git        # Git branch/file selection
fzf-env        # Browse environment variables
```

## Installation

### From Source (The Fun Way)

```bash
# Clone this monster
git clone https://github.com/quinnjr/jsh
cd franken-shell

# Build it
cargo build --release

# Install it (at your own risk)
sudo cp target/release/fsh /usr/local/bin/

# Add to shells (optional but recommended)
echo "/usr/local/bin/fsh" | sudo tee -a /etc/shells

# Make it your default (no turning back now)
chsh -s /usr/local/bin/fsh
```

### From Cargo

```bash
cargo install franken-shell
```

### From Package Manager

```bash
# Debian/Ubuntu (coming soon™)
sudo apt install franken-shell

# Arch (btw) (coming soon™)
yay -S franken-shell

# Homebrew (coming soon™)
brew install franken-shell
```

## Configuration

Franken Shell looks for configuration in these places:

| File | When Sourced | Purpose |
|------|--------------|---------|
| `/etc/fshenv` | Always | System-wide environment |
| `~/.fshenv` | Always | User environment |
| `$XDG_CONFIG_HOME/fsh/env` | Always | XDG environment |
| `/etc/fsh_profile` | Login shell | System-wide profile |
| `~/.fsh_profile` | Login shell | User profile |
| `$XDG_CONFIG_HOME/fsh/profile` | Login shell | XDG profile |
| `/etc/fshrc` | Interactive | System-wide rc |
| `~/.fshrc` | Interactive | User rc |
| `$XDG_CONFIG_HOME/fsh/fshrc` | Interactive | XDG rc |

### Example `.fshrc`

```bash
# ~/.fshrc - Franken Shell configuration
# The vibe-coded shell that does everything stupidly™

# Set your theme (pick your poison)
theme set powerline

# Enable ssh-agent auto-start (optional)
export FSH_SSH_AGENT_AUTO_START=1

# Aliases (because typing is hard)
alias ll='ls -la'
alias gs='git status'
alias gc='git commit'
alias yolo='git push --force'  # please don't

# Functions
fn mkcd(dir) {
    mkdir -p "$dir" && cd "$dir"
}

# Plugins
plug "zsh-users/zsh-syntax-highlighting"
plug "zsh-users/zsh-autosuggestions"
plug-load

# Custom prompt (optional)
# theme set-prompt "%F{red}🧟%f %~ → "

# Welcome message
echo "🧟 Welcome to Franken Shell - It's alive!"
```

## XDG Base Directory Support

Franken Shell automatically initializes XDG Base Directory environment variables on startup if they're not already set. This ensures scripts and tools that depend on these variables work correctly.

| Variable | Default | Usage |
|----------|---------|-------|
| `XDG_CONFIG_HOME` | `~/.config` | Config files in `fsh/` |
| `XDG_DATA_HOME` | `~/.local/share` | Data files in `fsh/` |
| `XDG_STATE_HOME` | `~/.local/state` | State/history in `fsh/` |
| `XDG_CACHE_HOME` | `~/.cache` | Cache files in `fsh/` |
| `XDG_RUNTIME_DIR` | (system-set) | Runtime files (set by login manager) |

The shell also automatically creates the `fsh/` subdirectories inside each XDG location on first startup.

## Performance

Despite all the features, Franken Shell is surprisingly fast:

| Benchmark | Time |
|-----------|------|
| Startup (non-interactive) | ~5ms |
| Simple command | ~1ms |
| Complex script | It depends™ |

*Results may vary. No shells were harmed in these benchmarks.*

## Contributing

Found a bug? Want to add a feature? Have a complaint?

1. Fork the repo
2. Create a branch (`git checkout -b feature/amazing-thing`)
3. Write some code (tests appreciated but not required, we're not that organized)
4. Submit a PR
5. Wait patiently while we review it (or impatiently, we understand)

## FAQ

**Q: Is this production ready?**
A: Define "production"

**Q: Why is it called Franken Shell?**
A: Because it's stitched together from the best (and worst) parts of every shell we could find

**Q: Should I use this as my daily driver?**
A: If you like living dangerously, absolutely

**Q: Why does [feature] work like that?**
A: It made sense at 3am

**Q: Can I contribute?**
A: Yes! We accept PRs, bug reports, and emotional support

## License

MIT OR Apache-2.0 (pick your favorite, we won't judge)

---

<p align="center">
  <b>Made with ☕ and questionable decisions by Joseph R. Quinn</b>
  <br>
  <i>The vibe-coded shell that does everything stupidly™</i>
  <br><br>
  🧟⚡🔩
</p>
