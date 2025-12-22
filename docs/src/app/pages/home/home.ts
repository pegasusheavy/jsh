import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { faGithub, faRust } from '@fortawesome/free-brands-svg-icons';
import { faTerminal, faFish, faCubes, faPalette, faRocket, faCode, faBolt, faShield, faGears } from '@fortawesome/free-solid-svg-icons';

@Component({
  selector: 'app-home',
  imports: [RouterLink, FontAwesomeModule],
  template: `
    <!-- Hero Section -->
    <section class="relative overflow-hidden">
      <div class="absolute inset-0 bg-gradient-to-br from-[var(--rust-orange)]/10 via-transparent to-[var(--rust-brown)]/5"></div>
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-24 relative">
        <div class="text-center">
          <h1 class="text-5xl md:text-7xl font-bold mb-6">
            <fa-icon [icon]="faTerminal" class="text-[var(--color-primary)] mr-4"></fa-icon>
            <span class="text-[var(--color-primary)]">jsh</span>
          </h1>
          <p class="text-2xl md:text-3xl text-[var(--color-text)] mb-2 font-medium">
            Joseph's Shell
          </p>
          <p class="text-xl md:text-2xl text-[var(--color-text-muted)] mb-4 max-w-3xl mx-auto">
            A modern shell that combines Bash, ZSH, Fish, and Ash compatibility with enhanced scripting features
          </p>
          <p class="text-lg text-[var(--color-text-muted)] mb-8 flex items-center justify-center gap-2">
            Written in <fa-icon [icon]="faRust" class="text-[var(--rust-orange)]"></fa-icon> Rust for speed and safety
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <a routerLink="/getting-started"
               class="px-8 py-3 bg-[var(--color-primary)] text-white rounded-lg font-semibold hover:bg-[var(--color-primary-dark)] transition-colors hover:no-underline flex items-center gap-2">
              <fa-icon [icon]="faRocket"></fa-icon>
              Get Started
            </a>
            <a href="https://github.com/pegasusheavy/jsh" target="_blank"
               class="px-8 py-3 border border-[var(--color-border)] text-[var(--color-text)] rounded-lg font-semibold hover:border-[var(--color-primary)] transition-colors hover:no-underline flex items-center gap-2">
              <fa-icon [icon]="faGithub"></fa-icon>
              View on GitHub
            </a>
          </div>
        </div>
      </div>
    </section>

    <!-- Features Grid -->
    <section class="py-20 bg-[var(--color-surface)]">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h2 class="text-3xl font-bold text-center mb-12">Multi-Shell Compatibility</h2>
        <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div class="p-6 bg-[var(--color-surface-elevated)] rounded-xl border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors">
            <div class="text-3xl mb-4 text-[var(--color-primary)]">
              <fa-icon [icon]="faTerminal"></fa-icon>
            </div>
            <h3 class="text-xl font-semibold mb-2">Bash/ZSH</h3>
            <p class="text-[var(--color-text-muted)] text-sm">
              Full compatibility with existing shell scripts. if/for/while/case statements, pipes, redirections, and more.
            </p>
          </div>
          <div class="p-6 bg-[var(--color-surface-elevated)] rounded-xl border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors">
            <div class="text-3xl mb-4 text-[var(--color-primary)]">
              <fa-icon [icon]="faFish"></fa-icon>
            </div>
            <h3 class="text-xl font-semibold mb-2">Fish</h3>
            <p class="text-[var(--color-text-muted)] text-sm">
              Fish-style set command, string manipulation, math operations, and user-friendly syntax.
            </p>
          </div>
          <div class="p-6 bg-[var(--color-surface-elevated)] rounded-xl border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors">
            <div class="text-3xl mb-4 text-[var(--color-primary)]">
              <fa-icon [icon]="faCubes"></fa-icon>
            </div>
            <h3 class="text-xl font-semibold mb-2">Ash/POSIX</h3>
            <p class="text-[var(--color-text-muted)] text-sm">
              POSIX-compliant set options, errexit, nounset, xtrace, and portable scripting support.
            </p>
          </div>
          <div class="p-6 bg-[var(--color-surface-elevated)] rounded-xl border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors">
            <div class="text-3xl mb-4 text-[var(--color-primary)]">
              <fa-icon [icon]="faPalette"></fa-icon>
            </div>
            <h3 class="text-xl font-semibold mb-2">Oh-My-Zsh Themes</h3>
            <p class="text-[var(--color-text-muted)] text-sm">
              Compatible with ZSH-style prompt escape sequences and theme customization.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- jsh Enhanced Syntax -->
    <section class="py-20">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <h2 class="text-3xl font-bold text-center mb-4">Enhanced Scripting Syntax</h2>
        <p class="text-center text-[var(--color-text-muted)] mb-12 max-w-2xl mx-auto">
          jsh introduces modern, readable syntax inspired by Rust while maintaining full shell compatibility
        </p>

        <div class="grid lg:grid-cols-2 gap-8">
          <!-- Match Expression -->
          <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] overflow-hidden">
            <div class="px-6 py-4 border-b border-[var(--color-border)]">
              <h3 class="font-semibold text-[var(--color-primary)]">match - Pattern Matching</h3>
            </div>
            <pre class="p-6 text-sm overflow-x-auto"><code>match $cmd &#123;
    start =&gt; start_service
    stop =&gt; stop_service
    restart =&gt; &#123;
        stop_service
        start_service
    &#125;
    * =&gt; echo "Unknown: $cmd"
&#125;</code></pre>
          </div>

          <!-- Try-Catch -->
          <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] overflow-hidden">
            <div class="px-6 py-4 border-b border-[var(--color-border)]">
              <h3 class="font-semibold text-[var(--color-primary)]">try/catch - Error Handling</h3>
            </div>
            <pre class="p-6 text-sm overflow-x-auto"><code>try &#123;
    let result = $(risky_command)
    echo "Success: $result"
&#125; catch err &#123;
    echo "Error: $err"
&#125; finally &#123;
    cleanup
&#125;</code></pre>
          </div>

          <!-- Let/Const -->
          <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] overflow-hidden">
            <div class="px-6 py-4 border-b border-[var(--color-border)]">
              <h3 class="font-semibold text-[var(--color-primary)]">let/const - Variable Bindings</h3>
            </div>
            <pre class="p-6 text-sm overflow-x-auto"><code>let name = "World"
const VERSION = "1.0.0"

echo "Hello, $name! v$VERSION"</code></pre>
          </div>

          <!-- Loop -->
          <div class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] overflow-hidden">
            <div class="px-6 py-4 border-b border-[var(--color-border)]">
              <h3 class="font-semibold text-[var(--color-primary)]">loop - Infinite Loops</h3>
            </div>
            <pre class="p-6 text-sm overflow-x-auto"><code>loop &#123;
    read -p "Command: " cmd
    if [ "$cmd" = "quit" ]; then
        break
    fi
    eval "$cmd"
&#125;</code></pre>
          </div>
        </div>
      </div>
    </section>

    <!-- Quick Install -->
    <section class="py-20 bg-[var(--color-surface)]">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <h2 class="text-3xl font-bold mb-8">Quick Install</h2>
        <div class="bg-[var(--color-code-bg)] rounded-xl p-6 border border-[var(--color-border)]">
          <pre class="text-left text-sm"><code># Clone and build
git clone https://github.com/pegasusheavy/jsh.git
cd jsh
cargo build --release

# Run
./target/release/jsh</code></pre>
        </div>
        <p class="mt-6 text-[var(--color-text-muted)]">
          Requires Rust 2024 edition. See
          <a routerLink="/getting-started" class="text-[var(--color-primary)]">Getting Started</a>
          for detailed instructions.
        </p>
      </div>
    </section>
  `
})
export class HomePage {
  // FontAwesome icons
  faGithub = faGithub;
  faRust = faRust;
  faTerminal = faTerminal;
  faFish = faFish;
  faCubes = faCubes;
  faPalette = faPalette;
  faRocket = faRocket;
  faCode = faCode;
  faBolt = faBolt;
  faShield = faShield;
  faGears = faGears;
}
