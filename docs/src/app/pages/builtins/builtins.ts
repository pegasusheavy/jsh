import { Component } from '@angular/core';

@Component({
  selector: 'app-builtins',
  template: `
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <h1 class="text-4xl font-bold mb-4">Built-in Commands</h1>
      <p class="text-[var(--color-text-muted)] mb-8">
        Complete reference of all jsh built-in commands.
      </p>

      <!-- Navigation -->
      <nav class="mb-8 p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
        <div class="flex flex-wrap gap-2 text-sm">
          <a href="#core" class="text-[var(--color-primary)]">Core</a>
          <span class="text-[var(--color-text-muted)]">•</span>
          <a href="#variables" class="text-[var(--color-primary)]">Variables</a>
          <span class="text-[var(--color-text-muted)]">•</span>
          <a href="#io" class="text-[var(--color-primary)]">I/O</a>
          <span class="text-[var(--color-text-muted)]">•</span>
          <a href="#flow" class="text-[var(--color-primary)]">Flow Control</a>
          <span class="text-[var(--color-text-muted)]">•</span>
          <a href="#fish" class="text-[var(--color-primary)]">Fish</a>
          <span class="text-[var(--color-text-muted)]">•</span>
          <a href="#theme" class="text-[var(--color-primary)]">Theme</a>
        </div>
      </nav>

      <!-- Core Commands -->
      <section id="core" class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Core Commands</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">cd [directory]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Change current directory. Without argument, goes to HOME.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">pwd</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Print current working directory.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">exit [code]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Exit the shell with optional status code.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">source|. file</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Execute commands from file in current shell.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">eval string</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Evaluate string as shell command.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">exec command</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Replace shell with command.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">type|which command</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Describe how command would be interpreted.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">help</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Display built-in commands help.</p>
          </div>
        </div>
      </section>

      <!-- Variable Commands -->
      <section id="variables" class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Variable Commands</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">export [name[=value]]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Export variable to environment.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">readonly [name[=value]]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Mark variable as read-only.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">local name[=value]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Create function-local variable.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">unset name</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Remove variable or function.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">set [options] [-- args]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Set shell options or positional parameters.</p>
            <div class="mt-2 text-xs text-[var(--color-text-muted)]">
              Options: -e (errexit), -u (nounset), -x (xtrace), -o name
            </div>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">shift [n]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Shift positional parameters left by n (default 1).</p>
          </div>
        </div>
      </section>

      <!-- I/O Commands -->
      <section id="io" class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">I/O Commands</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">echo [-n] [-e] [string...]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Print arguments to stdout.</p>
            <div class="mt-2 text-xs text-[var(--color-text-muted)]">
              -n: no newline, -e: interpret escapes
            </div>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">printf format [args...]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Formatted output (C-style).</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">read [-p prompt] [name...]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Read line from stdin into variables.</p>
          </div>
        </div>
      </section>

      <!-- Flow Control -->
      <section id="flow" class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Flow Control</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">test|[ expression ]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Evaluate conditional expression.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">true</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Return success (exit 0).</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">false</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Return failure (exit 1).</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">return [n]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Return from function with status.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">break [n]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Break out of n loop levels.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">continue [n]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Continue to next iteration of n-th loop.</p>
          </div>
        </div>
      </section>

      <!-- Fish Builtins -->
      <section id="fish" class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Fish-Compatible</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">string subcommand [args...]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">String manipulation.</p>
            <div class="mt-2 text-xs text-[var(--color-text-muted)]">
              Subcommands: length, upper, lower, trim, split, join, replace, match, escape, repeat
            </div>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">math expression</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Evaluate arithmetic expression.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">contains item [list...]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Check if item is in list.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">status subcommand</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Query shell status.</p>
            <div class="mt-2 text-xs text-[var(--color-text-muted)]">
              Subcommands: is-interactive, is-login, filename
            </div>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">functions [-q|-e|-n] [name]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">List or manage functions.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">abbr [-a|-e|-l] [name] [expansion]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Manage abbreviations.</p>
          </div>
        </div>
      </section>

      <!-- Theme Commands -->
      <section id="theme" class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Theme System</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">theme list</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">List available themes.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">theme set name</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Set current theme.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">theme preview</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Preview all available themes.</p>
          </div>
        </div>
      </section>

      <!-- Directory Stack -->
      <section>
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">Directory Stack</h2>

        <div class="space-y-4">
          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">pushd [directory]</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Push directory onto stack and cd to it.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">popd</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Pop directory from stack and cd to it.</p>
          </div>

          <div class="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
            <h3 class="font-mono font-semibold">dirs</h3>
            <p class="text-sm text-[var(--color-text-muted)] mt-1">Display directory stack.</p>
          </div>
        </div>
      </section>
    </div>
  `
})
export class BuiltinsPage {}
