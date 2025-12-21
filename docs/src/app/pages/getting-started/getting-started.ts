import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-getting-started',
  imports: [RouterLink],
  template: `
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <h1 class="text-4xl font-bold mb-8">Getting Started</h1>

      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4">Requirements</h2>
        <ul class="list-disc list-inside space-y-2 text-[var(--color-text-muted)]">
          <li>Rust toolchain (2024 edition)</li>
          <li>Cargo package manager</li>
          <li>Git</li>
        </ul>
      </section>

      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4">Installation</h2>

        <h3 class="text-xl font-medium mb-3 mt-6">From Source</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Clone the repository
git clone https://github.com/pegasusheavy/jsh.git
cd jsh

# Build release binary
cargo build --release

# Install to ~/.cargo/bin (optional)
cargo install --path .</code></pre>

        <h3 class="text-xl font-medium mb-3 mt-6">Verify Installation</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code>./target/release/jsh --version
# jsh 0.1.0</code></pre>
      </section>

      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4">Running jsh</h2>

        <h3 class="text-xl font-medium mb-3">Interactive Mode</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Start interactive shell
jsh

# With a specific config
jsh --rcfile ~/.jshrc</code></pre>

        <h3 class="text-xl font-medium mb-3 mt-6">Execute Commands</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Run a single command
jsh -c 'echo "Hello, World!"'

# Execute a script
jsh script.jsh</code></pre>
      </section>

      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4">Configuration</h2>
        <p class="text-[var(--color-text-muted)] mb-4">
          jsh reads configuration from <code>~/.jshrc</code> on startup.
        </p>

        <h3 class="text-xl font-medium mb-3">Example ~/.jshrc</h3>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code># Set theme
JSH_THEME="robbyrussell"

# Custom prompt
PROMPT='%F&#123;green&#125;%n&#64;%m%f:%F&#123;blue&#125;%~%f$ '

# Enable shell options
set -o errexit
set -o nounset

# Aliases
alias ll='ls -la'
alias gs='git status'

# Custom functions
fn mkcd &#123;
    mkdir -p "$1"
    cd "$1"
&#125;</code></pre>
      </section>

      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4">Getting Help</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code># Show all available commands
jsh -c 'help'

# Command line options
jsh --help</code></pre>
      </section>

      <section>
        <h2 class="text-2xl font-semibold mb-4">Next Steps</h2>
        <div class="grid md:grid-cols-2 gap-4">
          <a routerLink="/bash-zsh" class="block p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors hover:no-underline">
            <h3 class="font-semibold text-[var(--color-primary)]">Bash/ZSH Compatibility →</h3>
            <p class="text-sm text-[var(--color-text-muted)]">Learn about traditional shell syntax</p>
          </a>
          <a routerLink="/jsh-syntax" class="block p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors hover:no-underline">
            <h3 class="font-semibold text-[var(--color-primary)]">jsh Enhanced Syntax →</h3>
            <p class="text-sm text-[var(--color-text-muted)]">Explore match, loop, let, try/catch</p>
          </a>
        </div>
      </section>
    </div>
  `
})
export class GettingStartedPage {}
