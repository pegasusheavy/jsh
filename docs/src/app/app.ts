import { Component } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { FaIconLibrary, FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { faGithub, faPatreon } from '@fortawesome/free-brands-svg-icons';
import { faBook, faTerminal, faFish, faCode, faCog, faRocket, faBug, faHeart } from '@fortawesome/free-solid-svg-icons';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, RouterLink, RouterLinkActive, FontAwesomeModule],
  template: `
    <div class="min-h-screen flex flex-col">
      <!-- Header -->
      <header class="sticky top-0 z-50 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
        <nav class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex items-center justify-between h-16">
            <div class="flex items-center gap-8">
              <a routerLink="/" class="flex items-center gap-3 text-xl font-bold text-[var(--color-primary)] hover:no-underline">
                <fa-icon [icon]="faTerminal" class="text-2xl"></fa-icon>
                <span>jsh</span>
              </a>
              <div class="hidden md:flex items-center gap-6">
                <a routerLink="/getting-started" routerLinkActive="text-[var(--color-primary)]"
                   class="text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] flex items-center gap-1.5">
                  <fa-icon [icon]="faRocket" class="text-xs"></fa-icon>
                  Getting Started
                </a>
                <a routerLink="/bash-zsh" routerLinkActive="text-[var(--color-primary)]"
                   class="text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
                  Bash/ZSH
                </a>
                <a routerLink="/fish" routerLinkActive="text-[var(--color-primary)]"
                   class="text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
                  Fish
                </a>
                <a routerLink="/ash-posix" routerLinkActive="text-[var(--color-primary)]"
                   class="text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
                  Ash/POSIX
                </a>
                <a routerLink="/jsh-syntax" routerLinkActive="text-[var(--color-primary)]"
                   class="text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] flex items-center gap-1.5">
                  <fa-icon [icon]="faCode" class="text-xs"></fa-icon>
                  jsh Syntax
                </a>
                <a routerLink="/builtins" routerLinkActive="text-[var(--color-primary)]"
                   class="text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] flex items-center gap-1.5">
                  <fa-icon [icon]="faCog" class="text-xs"></fa-icon>
                  Builtins
                </a>
              </div>
            </div>
            <div class="flex items-center gap-4">
              <a href="https://github.com/pegasusheavy/jsh" target="_blank" rel="noopener"
                 class="text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                 title="View on GitHub">
                <fa-icon [icon]="faGithub" class="text-xl"></fa-icon>
              </a>
            </div>
          </div>
        </nav>
      </header>

      <!-- Main content -->
      <main class="flex-1">
        <router-outlet />
      </main>

      <!-- Footer -->
      <footer class="border-t border-[var(--color-border)] bg-[var(--color-surface)] py-8">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex flex-col md:flex-row justify-between items-center gap-4">
            <div class="text-[var(--color-text-muted)] text-sm">
              © 2025 Pegasus Heavy Industries LLC. Dual-licensed under MIT and Apache 2.0.
            </div>
            <div class="flex items-center gap-6 text-sm">
              <a href="https://github.com/pegasusheavy/jsh" target="_blank" class="text-[var(--color-text-muted)] hover:text-[var(--color-text)] flex items-center gap-1.5">
                <fa-icon [icon]="faGithub"></fa-icon>
                GitHub
              </a>
              <a href="https://github.com/pegasusheavy/jsh/issues" target="_blank" class="text-[var(--color-text-muted)] hover:text-[var(--color-text)] flex items-center gap-1.5">
                <fa-icon [icon]="faBug"></fa-icon>
                Issues
              </a>
              <a href="https://patreon.com/c/PegasusHeavyIndustries" target="_blank" class="text-[var(--color-text-muted)] hover:text-[var(--color-text)] flex items-center gap-1.5">
                <fa-icon [icon]="faHeart"></fa-icon>
                Support
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  `,
  styles: ``
})
export class App {
  // FontAwesome icons
  faGithub = faGithub;
  faPatreon = faPatreon;
  faBook = faBook;
  faTerminal = faTerminal;
  faCode = faCode;
  faCog = faCog;
  faRocket = faRocket;
  faBug = faBug;
  faHeart = faHeart;

  constructor(library: FaIconLibrary) {
    // Add icons to the library for use throughout the app
    library.addIcons(faGithub, faPatreon, faBook, faTerminal, faCode, faCog, faRocket, faBug, faHeart);
  }
}
