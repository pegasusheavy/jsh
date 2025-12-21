# JSH Themes

JSH includes oh-my-zsh compatible theme support. Themes can be set in several ways:

## Quick Start

In your `.jshrc`:
```bash
export JSH_THEME="robbyrussell"
```

Or interactively:
```bash
theme robbyrussell
```

## Available Built-in Themes

- **jsh** (default) - Two-line prompt with user, host, git, and colored status
- **robbyrussell** - Classic oh-my-zsh default theme
- **agnoster** - Powerline-style segments
- **minimal** - Clean, simple prompt
- **powerlevel** - Powerline-inspired with segments
- **simple** - Basic user@host:path$ format
- **pure** - ZSH Pure theme inspired

## Custom Prompts

Set `PROMPT` and `RPROMPT` environment variables directly:

```bash
export PROMPT='%F{cyan}%n%f@%F{blue}%m%f:%F{yellow}%~%f$ '
export RPROMPT='%F{8}%T%f'
```

## ZSH Prompt Escape Sequences

### Directory
- `%~` - Current directory with `~` for home
- `%/` - Full path
- `%c` or `%.` - Trailing component of path
- `%c2` - Last 2 components of path

### User/Host
- `%n` - Username
- `%m` - Short hostname
- `%M` - Full hostname

### Time/Date
- `%T` - 24-hour time (HH:MM)
- `%t` or `%@` - 12-hour time with AM/PM
- `%*` - 24-hour time with seconds
- `%D` - Date (YY-MM-DD)
- `%D{format}` - Custom date format
- `%w` - Day and date (e.g., "Mon Jan 01")

### Shell State
- `%#` - `#` for root, `%` otherwise
- `%?` - Exit status of last command
- `%h` or `%!` - History number

### Colors
- `%F{color}...%f` - Foreground color
- `%K{color}...%k` - Background color

Colors can be:
- Names: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- Bright variants: `bright_red`, etc.
- 256-color palette: `0`-`255`
- True color hex: `#ff5500`

### Formatting
- `%B...%b` - Bold
- `%U...%u` - Underline
- `%S...%s` - Strikethrough

### Conditionals
- `%(?.true.false)` - Based on last exit status (0 = true)
- `%(#.true.false)` - Based on root status

### Git Info
- `$(git_prompt_info)` - Git branch and status indicator

## Example Custom Themes

### Minimal with Git
```bash
export PROMPT='%F{blue}%~%f $(git_prompt_info)%F{green}❯%f '
```

### Powerline Style
```bash
export PROMPT='%K{blue}%F{white} %n %f%k%K{cyan}%F{blue}%f%F{black} %~ %f%k%F{cyan}%f '
```

### Two-line with Time
```bash
export PROMPT='%F{8}[%T]%f %F{cyan}%n%f@%F{blue}%m%f:%F{yellow}%~%f
%(?.%F{green}.%F{red})❯%f '
```

### Exit Status Indicator
```bash
export PROMPT='%F{yellow}%~%f %(?.%F{green}✓.%F{red}✗)%f '
```

