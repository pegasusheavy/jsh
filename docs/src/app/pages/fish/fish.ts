import { Component } from '@angular/core';

@Component({
  selector: 'app-fish',
  template: `
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <h1 class="text-4xl font-bold mb-4">Fish Compatibility</h1>
      <p class="text-[var(--color-text-muted)] mb-8">
        jsh supports Fish shell syntax and builtins for a more user-friendly scripting experience.
      </p>

      <!-- set command -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">set - Variable Management</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Set a variable
set name "World"
set count 42

# Export variable
set -x PATH "/usr/local/bin" $PATH

# Global variable
set -g GLOBAL_VAR "value"

# Erase variable
set -e name

# Query if variable exists
set -q name &amp;&amp; echo "exists"</code></pre>
      </section>

      <!-- string builtin -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">string - String Manipulation</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Length
string length "hello"           # 5

# Case conversion
string upper "hello"           # HELLO
string lower "HELLO"           # hello

# Trim whitespace
string trim "  hello  "       # hello

# Split
string split "," "a,b,c"       # a\\nb\\nc

# Join
string join "-" a b c          # a-b-c

# Replace
string replace "old" "new" "old text"  # new text

# Match (regex)
string match "*.txt" "file.txt"  # file.txt

# Repeat
string repeat -n 3 "ab"        # ababab

# Escape
string escape "hello world"    # hello\\ world</code></pre>
      </section>

      <!-- math builtin -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">math - Arithmetic</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Basic operations
math "2 + 3"           # 5
math "10 - 4"          # 6
math "3 * 4"           # 12
math "20 / 4"          # 5
math "17 % 5"          # 2

# Exponentiation
math "2 ^ 10"          # 1024

# Complex expressions
math "2 + 3 * 4"       # 14
math "(2 + 3) * 4"     # 20</code></pre>
      </section>

      <!-- contains -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">contains - List Membership</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Check if item is in list
contains apple apple banana cherry &amp;&amp; echo "Found!"

# Use in conditionals
if contains $fruit apple orange banana; then
    echo "It's a fruit!"
fi

# With variables
set fruits apple banana cherry
contains grape $fruits || echo "Not found"</code></pre>
      </section>

      <!-- status -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">status - Shell State</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Check if interactive
if status is-interactive; then
    echo "Running interactively"
fi

# Check if login shell
status is-login &amp;&amp; echo "Login shell"

# Get current filename
status filename</code></pre>
      </section>

      <!-- functions -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">functions - Function Management</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># List all functions
functions

# Check if function exists
functions -q myfunction &amp;&amp; echo "Exists"

# Erase a function
functions -e myfunction

# List function names only
functions -n</code></pre>
      </section>

      <!-- abbr -->
      <section class="mb-12">
        <h2 class="text-2xl font-semibold mb-4 text-[var(--color-primary)]">abbr - Abbreviations</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 mb-4 overflow-x-auto"><code># Add abbreviation
abbr -a gs "git status"
abbr -a gp "git push"

# List abbreviations
abbr -l

# Erase abbreviation
abbr -e gs</code></pre>
      </section>

      <!-- Example Script -->
      <section>
        <h2 class="text-2xl font-semibold mb-4">Complete Example</h2>
        <pre class="bg-[var(--color-code-bg)] rounded-lg p-4 overflow-x-auto"><code>#!/usr/bin/env jsh
# Fish-style configuration

# Variables
set name "Developer"
set -x EDITOR "vim"

# String operations
set greeting $(string upper "hello")
echo "$greeting, $name!"

# Math
set result $(math "2 ^ 8")
echo "2^8 = $result"

# List operations
set colors red green blue
if contains green $colors; then
    echo "Green is available"
fi

# Check shell status
if status is-interactive; then
    abbr -a ll "ls -la"
fi</code></pre>
      </section>
    </div>
  `
})
export class FishPage {}
